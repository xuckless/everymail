use std::collections::BTreeSet;

use super::plural;
use crate::{
    demo::{Author, Thread, World},
    navigation::{Catalog, Listing, Mailbox, Navigation, Selection},
    platform,
    proto::ui::v1::{
        self as ui, InboxCategory, Theme,
        approval_state::State as Approval,
        inbox_view::{Categories, Reading},
        thread_message::Display,
    },
};

const CATEGORIES: [(InboxCategory, &str); 3] = [
    (InboxCategory::ForYou, "For you"),
    (InboxCategory::Updates, "Updates"),
    (InboxCategory::Reading, "Reading"),
];

fn selected(selection: &Selection) -> Option<&str> {
    match selection {
        Selection::Id(id) => Some(id),
        Selection::None => None,
    }
}

pub fn inbox(navigation: &Navigation, theme: Theme, world: &World) -> ui::InboxView {
    let selected = selected(&navigation.thread);
    let title = match &navigation.listing {
        Listing::Mailbox(Mailbox::All, _) => "All inboxes",
        Listing::Mailbox(Mailbox::Account(id), _) => world.account(id).map_or("", |a| a.name),
        Listing::Folder(id) => world
            .folders
            .iter()
            .find(|f| f.id == id)
            .map_or("", |f| f.name),
    };
    let categories = match &navigation.listing {
        Listing::Mailbox(_, current) => Categories::Tabs(ui::CategoryTabs {
            tabs: CATEGORIES
                .iter()
                .map(|(category, label)| ui::CategoryTab {
                    category: (*category).into(),
                    label: (*label).into(),
                    selected: category == current,
                })
                .collect(),
        }),
        Listing::Folder(_) => Categories::Uncategorized(ui::Uncategorized {}),
    };
    let rows = world
        .listing(&navigation.listing)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|id| world.thread(id))
        .map(|t| ui::MailRow {
            thread_id: t.id.into(),
            from: t.from.into(),
            provider: world.account(t.account).map_or(0, |a| a.provider.into()),
            subject: t.subject.into(),
            note: Some(ui::RowNote {
                text: t.note.0.into(),
                kind: t.note.1.into(),
            }),
            time: t.time.into(),
            unread: t.unread,
            selected: selected == Some(t.id),
            account: world.account(t.account).map_or("", |a| a.name).into(),
        })
        .collect();
    let reading = selected
        .and_then(|id| world.thread(id))
        .and_then(|t| preview(t, theme, world))
        .unwrap_or(Reading::Nothing(ui::NothingSelected {}));
    ui::InboxView {
        title: title.into(),
        categories: Some(categories),
        rows,
        reading: Some(reading),
    }
}

fn preview(thread: &Thread, theme: Theme, world: &World) -> Option<Reading> {
    let latest = thread.messages.last()?;
    let to = world.account(thread.account).map_or("", |a| a.name);
    Some(Reading::Preview(ui::ReadingPreview {
        thread_id: thread.id.into(),
        subject: thread.subject.into(),
        from: author(&latest.author).into(),
        to_line: format!("to {to}"),
        time: thread.time.into(),
        summary: Some(summary(thread, world)),
        body_url: platform::body_url(latest.id, theme),
        drafts: drafts(thread, world),
    }))
}

pub fn thread(
    thread: &Thread,
    expanded: &BTreeSet<String>,
    theme: Theme,
    world: &World,
) -> ui::ThreadView {
    let account = world.account(thread.account).map_or("", |a| a.name);
    ui::ThreadView {
        thread_id: thread.id.into(),
        subject: thread.subject.into(),
        context: format!("{account} · {}", plural(thread.messages.len(), "message")),
        summary: Some(summary(thread, world)),
        messages: thread
            .messages
            .iter()
            .map(|m| ui::ThreadMessage {
                message_id: m.id.into(),
                from: author(&m.author).into(),
                preview: m.preview.into(),
                time: m.time.into(),
                display: Some(if expanded.contains(m.id) {
                    Display::Expanded(ui::MessageExpanded {
                        body_url: platform::body_url(m.id, theme),
                        invitations: m
                            .invitation
                            .and_then(|event| world.event(event))
                            .and_then(|event| event.invitation.clone())
                            .into_iter()
                            .collect(),
                    })
                } else {
                    Display::Collapsed(ui::MessageCollapsed {})
                }),
            })
            .collect(),
        drafts: drafts(thread, world),
        reply: Some(reply(thread, world)),
    }
}

fn author(author: &Author) -> &'static str {
    match author {
        Author::You => "You",
        Author::Contact(name) => name,
    }
}

fn summary(thread: &Thread, world: &World) -> ui::Summary {
    use ui::summary::State;
    ui::Summary {
        state: Some(if world.has_key() {
            State::Ready(ui::SummaryReady {
                text: thread.summary.into(),
            })
        } else {
            State::Off(ui::SummaryOff {})
        }),
    }
}

fn drafts(thread: &Thread, world: &World) -> Vec<ui::AgentDraft> {
    world
        .drafts(thread.id)
        .map(|a| {
            let state = a.item.state.unwrap_or_default();
            let status = match state.state {
                Some(Approval::Done(_)) => "sent",
                Some(Approval::Waiting(_)) | None => "waiting for you",
            };
            ui::AgentDraft {
                approval_id: a.item.id.clone(),
                status: format!("Draft by {} · {status}", a.item.agent),
                paragraphs: a.draft.iter().map(|p| (*p).into()).collect(),
                state: Some(state),
            }
        })
        .collect()
}

fn reply(thread: &Thread, world: &World) -> ui::ReplyPanel {
    use ui::{
        compose_ready::Generation, model_choice::Availability, provider_key::State as Key,
        reply_panel::State,
    };
    if !world.has_key() {
        return ui::ReplyPanel {
            state: Some(State::Off(ui::ComposeOff {})),
        };
    }
    let compose = &world.compose;
    let key_set = |model: &ui::Model| {
        world.settings.keys.iter().any(|(provider, _, key)| {
            i32::from(*provider) == model.provider && matches!(key, Key::Set(_))
        })
    };
    ui::ReplyPanel {
        state: Some(State::Ready(ui::ComposeReady {
            model: compose
                .models
                .iter()
                .find(|m| m.name == compose.model)
                .cloned(),
            models: compose
                .models
                .iter()
                .map(|m| ui::ModelChoice {
                    model: Some(m.clone()),
                    selected: m.name == compose.model,
                    availability: Some(if key_set(m) {
                        Availability::Ready(ui::ModelReady {
                            provider_name: provider_name(m.provider).into(),
                        })
                    } else {
                        Availability::NeedsKey(ui::ModelNeedsKey {
                            label: format!("Add {} key", provider_name(m.provider)),
                        })
                    }),
                })
                .collect(),
            quick_replies: thread.quick_replies.iter().map(|q| (*q).into()).collect(),
            tones: compose
                .tones
                .iter()
                .map(|t| ui::ToneChoice {
                    label: (*t).into(),
                    selected: *t == compose.tone,
                })
                .collect(),
            generation: Some(Generation::Empty(ui::GenerationEmpty {
                hint: format!(
                    "Pick a quick reply or say what you want, and {} writes it in your voice",
                    compose.model
                ),
            })),
            title: format!("Reply to {}", thread.from),
        })),
    }
}

pub fn quarantine(selection: &Selection, theme: Theme, world: &World) -> ui::QuarantineView {
    use ui::quarantine_view::State;
    let items = &world.quarantine;
    // Navigation keeps a row selected while there are rows.
    let Some(chosen) = items
        .iter()
        .find(|q| selected(selection) == Some(q.id))
        .or(items.first())
    else {
        return ui::QuarantineView {
            state: Some(State::Empty(ui::QuarantineEmpty {})),
        };
    };
    let account = |id: &str| world.account(id);
    ui::QuarantineView {
        state: Some(State::List(ui::QuarantineList {
            rows: items
                .iter()
                .map(|q| ui::QuarantineRow {
                    id: q.id.into(),
                    from: q.from.into(),
                    provider: account(q.account).map_or(0, |a| a.provider.into()),
                    subject: q.subject.into(),
                    reason: q.reason.into(),
                    outcome: format!("{} · dropped {}", q.outcome, q.drop),
                    time: q.time.into(),
                    selected: q.id == chosen.id,
                    account: account(q.account).map_or("", |a| a.name).into(),
                })
                .collect(),
            selected: Some(ui::QuarantineDetail {
                id: chosen.id.into(),
                subject: chosen.subject.into(),
                from: chosen.from.into(),
                to_line: format!("to {}", account(chosen.account).map_or("", |a| a.name)),
                time: chosen.time.into(),
                caught_by: chosen.caught_by.into(),
                why: chosen.why.into(),
                outcome: format!(
                    "{}. EveryMail drops its copy {}",
                    chosen.outcome, chosen.drop
                ),
                body_url: platform::body_url(chosen.id, theme),
            }),
        })),
    }
}

/// The provider's name as the model menu prints it: "Anthropic", "OpenAI".
fn provider_name(provider: i32) -> &'static str {
    match ui::ModelProvider::try_from(provider) {
        Ok(ui::ModelProvider::Anthropic) => "Anthropic",
        Ok(ui::ModelProvider::Openai) => "OpenAI",
        Ok(ui::ModelProvider::GoogleGemini) => "Gemini",
        Ok(ui::ModelProvider::Openrouter) => "OpenRouter",
        Ok(ui::ModelProvider::Unspecified) | Err(_) => "",
    }
}
