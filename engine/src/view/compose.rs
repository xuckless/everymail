use crate::{
    action::send_later_choices,
    navigation::Selection,
    proto::ui::v1::{self as ui, composer::State},
    time,
    world::{DraftKind, World, attachment_kind, size_label},
};

/// The composer over the screen: the draft the navigation holds open, or closed.
pub fn composer(selection: &Selection, world: &World) -> ui::Composer {
    let draft = match selection {
        Selection::Id(id) => world.draft(id),
        Selection::None => None,
    };
    let state = match draft {
        None => State::Closed(ui::ComposerClosed {}),
        Some(d) => {
            let thread = d.thread.as_deref().and_then(|t| world.thread(t));
            State::Open(ui::ComposerOpen {
                draft_id: d.id.clone(),
                title: match (d.kind, thread) {
                    (DraftKind::Reply | DraftKind::ReplyAll, Some(t)) => {
                        format!("Reply to {}", t.from)
                    }
                    (DraftKind::Forward, _) => "Forward".into(),
                    (DraftKind::New | DraftKind::Reply | DraftKind::ReplyAll, _) => {
                        "New message".into()
                    }
                },
                from: world
                    .accounts
                    .iter()
                    .map(|a| ui::AccountChoice {
                        id: a.id.clone(),
                        label: format!("{} · {}", a.name, a.address),
                        provider: a.provider.into(),
                        selected: a.id == d.account,
                    })
                    .collect(),
                to: d.recipients.clone(),
                subject: d.subject.clone(),
                body: d.body.clone(),
                attachments: d
                    .attachments
                    .iter()
                    .map(|(id, name, size)| ui::Attachment {
                        id: id.clone(),
                        name: name.clone(),
                        size: size_label(*size),
                        kind: attachment_kind(name).into(),
                    })
                    .collect(),
                saved: format!("Saved {}", time::short(d.updated_at, world.now)),
                send_later: send_later_choices(world.now)
                    .into_iter()
                    .map(|(label, day, minute)| ui::SendLaterChoice {
                        label: format!("{label} · {}", time::clock(minute)),
                        day_id: day.id(),
                        minute,
                    })
                    .collect(),
            })
        }
    };
    ui::Composer { state: Some(state) }
}
