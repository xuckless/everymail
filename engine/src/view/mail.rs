use std::collections::BTreeSet;

use super::{count, is, plural};
use crate::{
    action::selected,
    live::{Field, Generation, Live, Resolution, Summary, Target},
    navigation::{Disclosure, Listing, Mailbox, Navigation, Selection},
    platform,
    proto::ui::v1::{
        self as ui, SnoozePreset, Theme,
        approval_state::State as Approval,
        inbox_view::{Categories, Reading, Search},
        mail_row::AgentNote,
        thread_message::Display,
    },
    time,
    world::{
        Answer, ApprovalStatus, CATEGORIES, FolderKind, Message, Place, Snooze, Thread, World,
        category_name, paragraphs, recipients, size_label,
    },
};

/// A snooze as the menu shows it.
fn snooze(preset: Snooze) -> (SnoozePreset, &'static str) {
    match preset {
        Snooze::LaterToday => (SnoozePreset::LaterToday, "Later today"),
        Snooze::Tomorrow => (SnoozePreset::Tomorrow, "Tomorrow"),
        Snooze::ThisWeekend => (SnoozePreset::ThisWeekend, "This weekend"),
        Snooze::NextWeek => (SnoozePreset::NextWeek, "Next week"),
    }
}

fn title<'a>(listing: &Listing, world: &'a World) -> &'a str {
    match listing {
        Listing::Mailbox(Mailbox::All, _) => "All inboxes",
        Listing::Mailbox(Mailbox::Account(id), _) => world.account(id).map_or("", |a| &a.name),
        Listing::Folder(id) => world
            .folders
            .iter()
            .find(|f| f.id == *id)
            .map_or("", |f| &f.name),
        Listing::Search { scope, .. } => title(scope, world),
    }
}

pub fn inbox(navigation: &Navigation, theme: Theme, world: &World, live: &Live) -> ui::InboxView {
    let chosen = selected(&navigation.thread);
    let scope = navigation.listing.scope();
    let categories = match scope {
        Listing::Mailbox(mailbox, current) => {
            let account = match mailbox {
                Mailbox::All => None,
                Mailbox::Account(id) => Some(id.as_str()),
            };
            Categories::Tabs(ui::CategoryTabs {
                tabs: CATEGORIES
                    .iter()
                    .map(|(category, _, label)| ui::CategoryTab {
                        category: (*category).into(),
                        label: (*label).into(),
                        selected: category == current,
                        unread: count(world.unread(account, Some(*category))),
                    })
                    .collect(),
            })
        }
        Listing::Folder(_) | Listing::Search { .. } => {
            Categories::Uncategorized(ui::Uncategorized {})
        }
    };
    let threads = world.listing(&navigation.listing).unwrap_or_default();
    let search = match &navigation.listing {
        Listing::Search { query, scope } => Search::Results(ui::SearchShown {
            query: query.clone(),
            count: match threads.len() {
                0 => format!("No results in {}", title(scope, world)),
                n => format!("{} in {}", plural(n, "result"), title(scope, world)),
            },
        }),
        Listing::Mailbox(..) | Listing::Folder(_) => Search::NotSearching(ui::NotSearching {}),
    };
    // A thread of an account the replica does not hold is not shown.
    let rows = threads
        .iter()
        .filter_map(|t| Some((t, world.account(&t.account)?)))
        .map(|(t, account)| ui::MailRow {
            thread_id: t.id.clone(),
            from: t.from.clone(),
            provider: account.provider.into(),
            subject: t.subject.clone(),
            agent_note: Some(match &t.note {
                Some((text, kind)) => AgentNote::Noted(ui::RowNote {
                    text: text.clone(),
                    kind: (*kind).into(),
                }),
                None => AgentNote::None(ui::NoRowNote {}),
            }),
            time: time::short(t.latest_at, world.now),
            unread: t.unread,
            selected: chosen == Some(t.id.as_str()),
            account: account.name.clone(),
            starred: t.starred(),
            message_id: t.messages.last().map_or(String::new(), |m| m.id.clone()),
            snoozable: snoozable(t),
        })
        .collect();
    let reading = chosen
        .and_then(|id| world.thread(id))
        .and_then(|t| preview(t, theme, world, live))
        .unwrap_or(Reading::Nothing(ui::NothingSelected {}));
    ui::InboxView {
        title: match &navigation.listing {
            Listing::Search { .. } => "Search".into(),
            listing => title(listing, world).into(),
        },
        categories: Some(categories),
        rows,
        reading: Some(reading),
        search: Some(search),
        snooze: snooze_choices(world),
    }
}

fn preview(thread: &Thread, theme: Theme, world: &World, live: &Live) -> Option<Reading> {
    let latest = thread.messages.last()?;
    let to = world.account(&thread.account).map_or("", |a| &a.name);
    Some(Reading::Preview(ui::ReadingPreview {
        thread_id: thread.id.clone(),
        subject: thread.subject.clone(),
        from: author(latest),
        to_line: format!("to {to}"),
        time: time::short(thread.latest_at, world.now),
        scribe: Some(scribe(thread, world, live)),
        body_url: platform::body_url(&latest.id, theme),
        drafts: drafts(thread, world, live),
        recipients: recipients(&thread.from, &thread.others, latest.from_you),
        starred: thread.starred(),
        place: place(thread, world),
        attachments: attachments(latest),
        message_subject: subject(thread, thread.messages.len() > 1),
        actions: Some(actions(thread, world)),
        message_id: latest.id.clone(),
    }))
}

/// "Inbox · For you", "Archive", "Inbox · Updates · Receipts", "Snoozed until Thu 8:00"
fn place(thread: &Thread, world: &World) -> String {
    let category = category_name(thread.category).to_string();
    let mut parts: Vec<String> = match (thread.place, thread.snoozed(world.now)) {
        (Place::Inbox, Some(until)) => {
            vec![format!("Snoozed until {}", time::long(until, world.now))]
        }
        (Place::Inbox, None) => vec!["Inbox".into(), category],
        (Place::Archive, _) => vec!["Archive".into()],
        (Place::Trash, _) => vec!["Trash".into()],
        (Place::Sent, _) => vec!["Sent".into()],
    };
    parts.extend(
        thread
            .folders
            .iter()
            .filter_map(|id| world.folders.iter().find(|f| f.id == *id))
            .map(|f| f.name.clone()),
    );
    parts.join(" · ")
}

/// A thread can be snoozed anywhere but in the trash.
fn snoozable(thread: &Thread) -> bool {
    thread.place != Place::Trash
}

/// The snooze menu, with when each preset ends as the Backend says.
fn snooze_choices(world: &World) -> Vec<ui::SnoozeChoice> {
    world
        .snoozes
        .iter()
        .map(|(preset, until)| {
            let (preset, label) = snooze(*preset);
            ui::SnoozeChoice {
                preset: preset.into(),
                label: label.into(),
                when: time::long(*until, world.now),
            }
        })
        .collect()
}

fn actions(thread: &Thread, world: &World) -> ui::MailActions {
    use ui::move_choice::To;
    let in_inbox = thread.place == Place::Inbox && thread.snoozed(world.now).is_none();
    let inbox = ui::MoveChoice {
        to: Some(To::Inbox(ui::MoveToInbox {})),
        label: "Inbox".into(),
        current: in_inbox,
    };
    let folders = world
        .folders
        .iter()
        .filter(|f| f.kind == FolderKind::User)
        .map(|f| ui::MoveChoice {
            to: Some(To::FolderId(f.id.clone())),
            label: f.name.clone(),
            current: thread.folders.contains(&f.id),
        });
    ui::MailActions {
        unread: thread.unread,
        snooze: if snoozable(thread) {
            snooze_choices(world)
        } else {
            vec![]
        },
        r#move: std::iter::once(inbox).chain(folders).collect(),
        archive: !matches!(thread.place, Place::Archive | Place::Trash),
        trash: thread.place != Place::Trash,
    }
}

/// The first message carries the thread's subject; a reply adds "Re: ".
fn subject(thread: &Thread, reply: bool) -> String {
    if reply {
        format!("Re: {}", thread.subject)
    } else {
        thread.subject.clone()
    }
}

fn author(message: &Message) -> String {
    if message.from_you {
        "You".into()
    } else {
        message.sender.clone()
    }
}

fn attachments(message: &Message) -> Vec<ui::Attachment> {
    message
        .attachments
        .iter()
        .map(|a| ui::Attachment {
            id: a.id.clone(),
            name: a.name.clone(),
            size: size_label(a.size_bytes),
            kind: a.kind.into(),
        })
        .collect()
}

/// An invitation as a thread, the calendar and its drawer show it, with what the user opened.
pub fn invitation(
    event: &crate::world::CalendarEvent,
    open: &BTreeSet<Disclosure>,
    world: &World,
) -> Option<ui::Invitation> {
    use ui::{invitation::Prompt, invitation_answer::State};
    let invitation = event.invitation.as_ref()?;
    let calendar = world.calendar(&event.calendar);
    let place = if event.place.is_empty() {
        String::new()
    } else {
        format!(" · {}", event.place)
    };
    let busy = world.events.iter().any(|e| {
        e.id != event.id
            && e.date == event.date
            && e.start < event.end
            && event.start < e.end
            && e.invitation
                .as_ref()
                .is_none_or(|i| i.answer == Answer::Accepted)
    });
    let (answer, status) = match invitation.answer {
        Answer::Unanswered => (
            State::Unanswered(ui::InvitationUnanswered {}),
            "waiting for your answer",
        ),
        Answer::Accepted => (State::Accepted(ui::InvitationAccepted {}), "you accepted"),
        Answer::Tentative => (
            State::Tentative(ui::InvitationTentative {}),
            "you said maybe",
        ),
        Answer::Declined => (State::Declined(ui::InvitationDeclined {}), "you declined"),
    };
    Some(ui::Invitation {
        event_id: event.id.clone(),
        invited_by: format!("{} invited you", invitation.invited_by),
        title: event.title.clone(),
        when: format!(
            "{}, {} {} · {}–{}{place}",
            crate::date::WEEKDAYS[event.date.weekday()],
            event.date.month_short(),
            event.date.day,
            time::clock(event.start),
            time::clock(event.end)
        ),
        details: format!(
            "{} · {} · {}",
            calendar.map_or("", |c| &c.name),
            invitation.people,
            if busy {
                "you have something else then"
            } else {
                "nothing else on your calendars then"
            }
        ),
        status: format!(
            "Invitation · {} calendar · {status}",
            calendar.map_or("", |c| &c.name)
        ),
        answer: Some(ui::InvitationAnswer {
            state: Some(answer),
        }),
        prompt: Some(
            if invitation.answer == Answer::Unanswered
                || open.contains(&Disclosure::Invitation(event.id.clone()))
            {
                Prompt::Ask(ui::InvitationAsk {})
            } else {
                Prompt::Change(ui::InvitationChange {})
            },
        ),
    })
}

pub fn thread(
    thread: &Thread,
    expanded: &BTreeSet<String>,
    navigation: &Navigation,
    theme: Theme,
    world: &World,
    live: &Live,
) -> ui::ThreadView {
    let account = world.account(&thread.account).map_or("", |a| &a.name);
    ui::ThreadView {
        thread_id: thread.id.clone(),
        subject: thread.subject.clone(),
        context: format!("{account} · {}", plural(thread.messages.len(), "message")),
        starred: thread.starred(),
        place: place(thread, world),
        scribe: Some(scribe(thread, world, live)),
        messages: thread
            .messages
            .iter()
            .enumerate()
            .map(|(i, m)| ui::ThreadMessage {
                message_id: m.id.clone(),
                recipients: recipients(&thread.from, &thread.others, m.from_you),
                subject: subject(thread, i > 0),
                starred: m.starred,
                attachments: attachments(m),
                from: author(m),
                preview: m.preview.clone(),
                time: time::long(m.sent_at, world.now),
                display: Some(if expanded.contains(&m.id) {
                    Display::Expanded(ui::MessageExpanded {
                        body_url: platform::body_url(&m.id, theme),
                        invitations: m
                            .invitation
                            .as_deref()
                            .and_then(|event| world.event(event))
                            .and_then(|event| invitation(event, &navigation.open, world))
                            .into_iter()
                            .collect(),
                    })
                } else {
                    Display::Collapsed(ui::MessageCollapsed {})
                }),
            })
            .collect(),
        drafts: drafts(thread, world, live),
        reply: Some(reply(thread, world, live)),
        actions: Some(actions(thread, world)),
        message_id: thread
            .messages
            .last()
            .map_or(String::new(), |m| m.id.clone()),
    }
}

/// Scribe's summary.
fn scribe(thread: &Thread, world: &World, live: &Live) -> ui::ScribeSummary {
    use ui::scribe_summary::State;
    let state = match (&thread.summary, live.summaries.get(&thread.id)) {
        _ if !world.ai_on() => State::Off(ui::SummaryOff {}),
        (Some(text), _) => State::Ready(ui::SummaryReady { text: text.clone() }),
        (None, Some(Summary::Failed(reason))) => State::Failed(ui::SummaryFailed {
            reason: reason.clone(),
        }),
        (None, Some(Summary::Reading)) => State::Pending(ui::SummaryPending {
            label: "Scribe is reading the thread".into(),
        }),
        (None, None) => State::Pending(ui::SummaryPending {
            label: "Scribe summarises it when you open it".into(),
        }),
    };
    ui::ScribeSummary { state: Some(state) }
}

/// An approval's state, while it is shown: waiting and resolving always, done and declined while
/// their outcome lasts.
pub fn approval_state(approval: &crate::world::Approval, live: &Live) -> Option<ui::ApprovalState> {
    let lasting = matches!(
        live.lasting(&Target::Approval(approval.id.clone())),
        Some(Resolution::Done(_))
    );
    let state = match &approval.status {
        ApprovalStatus::Waiting => Approval::Waiting(ui::ApprovalWaiting {}),
        ApprovalStatus::Resolving => Approval::Resolving(ui::ApprovalResolving {}),
        ApprovalStatus::Done if lasting => Approval::Done(ui::ApprovalDone {}),
        ApprovalStatus::Declined(reason) if lasting => Approval::Declined(ui::ApprovalDeclined {
            reason: reason.clone(),
        }),
        ApprovalStatus::Done | ApprovalStatus::Declined(_) => return None,
    };
    Some(ui::ApprovalState { state: Some(state) })
}

/// The agents' drafts answering a thread, while they are shown.
fn drafts(thread: &Thread, world: &World, live: &Live) -> Vec<ui::AgentDraft> {
    world
        .drafts_for(&thread.id)
        .filter_map(|a| {
            let status = match &a.status {
                ApprovalStatus::Waiting => "waiting for you",
                ApprovalStatus::Resolving => "sending",
                ApprovalStatus::Done => "sent",
                ApprovalStatus::Declined(_) => "declined",
            };
            Some(ui::AgentDraft {
                approval_id: a.id.clone(),
                status: format!("Draft by {} · {status}", a.agent_name),
                paragraphs: paragraphs(&a.draft).into_iter().map(String::from).collect(),
                state: Some(approval_state(a, live)?),
            })
        })
        .collect()
}

/// The provider's name as the model menu prints it: "Anthropic", "OpenAI".
pub fn provider_name(provider: ui::ModelProvider) -> &'static str {
    match provider {
        ui::ModelProvider::Anthropic => "Anthropic",
        ui::ModelProvider::Openai => "OpenAI",
        ui::ModelProvider::GoogleGemini => "Gemini",
        ui::ModelProvider::Openrouter => "OpenRouter",
        ui::ModelProvider::Unspecified => "",
    }
}

/// The model menu: every model, the selected one marked, those without a key needing one.
pub fn model_choices(selected: &str, world: &World) -> Vec<ui::ModelChoice> {
    use ui::model_choice::Availability;
    world
        .models
        .iter()
        .map(|m| ui::ModelChoice {
            model: Some(model(m)),
            selected: m.name == selected,
            availability: Some(if world.key_set(m.provider) {
                Availability::Ready(ui::ModelReady {
                    provider_name: provider_name(m.provider).into(),
                })
            } else {
                Availability::NeedsKey(ui::ModelNeedsKey {
                    label: format!("Add {} key", provider_name(m.provider)),
                })
            }),
        })
        .collect()
}

pub fn model(model: &crate::world::Model) -> ui::Model {
    ui::Model {
        name: model.name.clone(),
        provider: model.provider.into(),
    }
}

fn reply(thread: &Thread, world: &World, live: &Live) -> ui::ReplyPanel {
    use ui::{compose_ready::Generation as View, reply_panel::State};
    if !world.ai_on() {
        return ui::ReplyPanel {
            state: Some(State::Off(ui::ComposeOff {})),
        };
    }
    let settings = &world.settings;
    let chosen = world
        .task_model(crate::world::TASK_REPLIES)
        .map_or("", |m| m.name.as_str());
    let account = world.account(&thread.account);
    let generation = match live.replies.get(&thread.id) {
        None => View::Empty(ui::GenerationEmpty {
            hint: format!(
                "Pick a quick reply or say what you want, and {chosen} writes it in your voice"
            ),
        }),
        Some(Generation::Writing) => View::Writing(ui::GenerationWriting {
            label: format!("Writing with {chosen}"),
        }),
        Some(Generation::Written(text)) => View::Draft(ui::GenerationDraft {
            addressing: format!(
                "To {} · from {}",
                thread.from,
                account.map_or("", |a| &a.address)
            ),
            paragraphs: paragraphs(text).into_iter().map(String::from).collect(),
        }),
        Some(Generation::Failed(reason)) => View::Failed(ui::GenerationFailed {
            reason: reason.clone(),
        }),
    };
    ui::ReplyPanel {
        state: Some(State::Ready(ui::ComposeReady {
            model: world.model(chosen).map(model),
            models: model_choices(chosen, world),
            quick_replies: thread.quick_replies.clone(),
            tones: crate::world::TONES
                .iter()
                .map(|t| ui::ToneChoice {
                    label: (*t).into(),
                    selected: *t == settings.compose_tone,
                })
                .collect(),
            generation: Some(generation),
            title: format!("Reply to {}", thread.from),
            revision: live.revisions.of(&Field::Reply(thread.id.clone())),
        })),
    }
}

pub fn quarantine(selection: &Selection, theme: Theme, world: &World) -> ui::QuarantineView {
    use ui::quarantine_view::State;
    let items = &world.quarantine;
    let lede = format!(
        "Caught by Security Guard or sent by a blocked sender. Each copy is dropped {} days after it lands here",
        world.settings.quarantine_days
    );
    // Navigation keeps a row selected while there are rows.
    let Some(chosen) = items.iter().find(|q| is(selection, &q.id)) else {
        return ui::QuarantineView {
            state: Some(State::Empty(ui::QuarantineEmpty {})),
            lede,
        };
    };
    let account = |id: &str| world.account(id);
    let dropped = |at: i64| time::day_month(time::date(at));
    ui::QuarantineView {
        state: Some(State::List(ui::QuarantineList {
            // Mail of an account the replica does not hold is not shown.
            rows: items
                .iter()
                .filter_map(|q| Some((q, account(&q.account)?)))
                .map(|(q, a)| ui::QuarantineRow {
                    id: q.id.clone(),
                    from: q.sender.clone(),
                    provider: a.provider.into(),
                    subject: q.subject.clone(),
                    reason: q.reason.clone(),
                    outcome: format!("{} · dropped {}", q.outcome, dropped(q.drops_at)),
                    time: time::short(q.received_at, world.now),
                    selected: q.id == chosen.id,
                    account: a.name.clone(),
                })
                .collect(),
            selected: Some(ui::QuarantineDetail {
                id: chosen.id.clone(),
                subject: chosen.subject.clone(),
                from: chosen.sender.clone(),
                to_line: format!("to {}", account(&chosen.account).map_or("", |a| &a.name)),
                time: time::long(chosen.received_at, world.now),
                caught_by: chosen.caught_by.clone(),
                why: chosen.why.clone(),
                outcome: format!(
                    "{}. EveryMail drops its copy {}",
                    chosen.outcome,
                    dropped(chosen.drops_at)
                ),
                body_url: platform::body_url(&chosen.id, theme),
            }),
        })),
        lede,
    }
}
