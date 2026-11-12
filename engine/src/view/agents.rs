use super::plural;
use crate::{
    demo::{Agent, Chat, Edition, World},
    navigation::{Navigation, Selection},
    proto::ui::v1::{
        self as ui, AgentDetailPane, AgentsPane, BuilderStep, agent_detail::Kind,
        agent_status::State as Status, agents_view::Detail, agents_view::Queue,
        approval_state::State as Approval, briefing_view::State as Briefing,
    },
};

fn is(selection: &Selection, id: &str) -> bool {
    matches!(selection, Selection::Id(selected) if selected == id)
}

const STEPS: [(BuilderStep, &str); 4] = [
    (BuilderStep::Instructions, "What should it do"),
    (BuilderStep::Triggers, "When does it wake up"),
    (BuilderStep::Permissions, "What it may do"),
    (BuilderStep::Summary, "Review"),
];

pub fn builder(step: BuilderStep, world: &World) -> ui::AgentBuilderView {
    ui::AgentBuilderView {
        steps: STEPS
            .iter()
            .map(|(s, label)| ui::BuilderStepTab {
                step: (*s).into(),
                label: (*label).into(),
                current: *s == step,
            })
            .collect(),
        ..world.builder.clone()
    }
}

pub fn agents(navigation: &Navigation, world: &World) -> ui::AgentsView {
    let selection = &navigation.agent;
    let rows = |system: bool| {
        world
            .agents
            .iter()
            .filter(|a| matches!(a.detail.kind, Some(Kind::System(_))) == system)
            .map(|a| ui::AgentRow {
                id: a.detail.id.clone(),
                name: a.detail.name.clone(),
                status: a.detail.status,
                next: a.next.into(),
                last_run: a.last_run.into(),
                selected: is(selection, &a.detail.id),
            })
            .collect()
    };
    let detail = world
        .agents
        .iter()
        .find(|a| is(selection, &a.detail.id))
        .map_or(Detail::None(ui::NoAgentSelected {}), |a| {
            Detail::Agent(ui::AgentDetail {
                chat: Some(match &a.chat {
                    Some(chat) => ui::agent_detail::Chat::Available(self::chat(
                        a,
                        chat,
                        navigation.detail_pane,
                        world,
                    )),
                    None => ui::agent_detail::Chat::None(ui::NoChat {}),
                }),
                ..a.detail.clone()
            })
        });
    let waiting: Vec<ui::ApprovalItem> = world
        .approvals
        .iter()
        .filter(|a| {
            matches!(
                a.item.state.as_ref().and_then(|s| s.state.as_ref()),
                Some(Approval::Waiting(_))
            )
        })
        .map(|a| a.item.clone())
        .collect();
    let approvals = if waiting.is_empty() {
        "Approvals".into()
    } else {
        format!("Approvals · {}", waiting.len())
    };
    let panes = [
        (AgentsPane::Agents, "Running".to_string()),
        (AgentsPane::Approvals, approvals),
    ]
    .into_iter()
    .map(|(pane, label)| ui::AgentsPaneTab {
        pane: pane.into(),
        label,
        current: pane == navigation.pane,
    })
    .collect();
    let queue = if waiting.is_empty() {
        Queue::Clear(ui::QueueClear {})
    } else {
        Queue::Waiting(ui::ApprovalQueue {
            count: plural(waiting.len(), "item"),
            items: waiting,
        })
    };
    ui::AgentsView {
        system_agents: rows(true),
        user_agents: rows(false),
        detail: Some(detail),
        queue: Some(queue),
        panes,
    }
}

pub fn briefing(selection: &Selection, world: &World) -> ui::BriefingView {
    ui::BriefingView {
        editions: world
            .editions
            .iter()
            .map(|e| ui::BriefingEditionTab {
                id: e.id.into(),
                label: e.label.into(),
                selected: is(selection, e.id),
            })
            .collect(),
        state: Some(
            world
                .editions
                .iter()
                .find(|e| is(selection, e.id))
                .map_or(Briefing::None(ui::NoBriefing {}), |e| {
                    Briefing::Edition(edition(e))
                }),
        ),
    }
}

fn lines(lines: &[ui::BriefingLine], empty: &str) -> ui::LineSection {
    use ui::line_section::State;
    ui::LineSection {
        state: Some(if lines.is_empty() {
            State::Empty(section_empty(empty))
        } else {
            State::Lines(ui::BriefingLines {
                lines: lines.to_vec(),
            })
        }),
    }
}

fn section_empty(text: &str) -> ui::SectionEmpty {
    ui::SectionEmpty { text: text.into() }
}

fn edition(edition: &Edition) -> ui::BriefingEdition {
    use ui::{calendar_section::State as Calendar, waiting_section::State as Waiting};
    ui::BriefingEdition {
        id: edition.id.into(),
        byline: edition.byline.into(),
        activity: Some(lines(&edition.activity, "Your agents had nothing to do")),
        caught_mail: edition.caught_mail,
        waiting: Some(ui::WaitingSection {
            state: Some(if edition.waiting.is_empty() {
                Waiting::Empty(section_empty("Nothing waits for you"))
            } else {
                Waiting::Waiting(ui::BriefingWaitingList {
                    items: edition.waiting.clone(),
                })
            }),
        }),
        newsletters: Some(lines(&edition.newsletters, "No newsletters")),
        updates: Some(lines(&edition.updates, "No updates")),
        calendar: Some(ui::CalendarSection {
            title: edition.calendar_title.into(),
            state: Some(if edition.calendar.is_empty() {
                Calendar::Empty(section_empty("Nothing planned"))
            } else {
                Calendar::Events(ui::BriefingEvents {
                    events: edition.calendar.clone(),
                })
            }),
        }),
    }
}

const DETAIL_PANES: [(AgentDetailPane, &str); 2] = [
    (AgentDetailPane::Activity, "Activity"),
    (AgentDetailPane::Chat, "Chat"),
];

fn chat(agent: &Agent, chat: &Chat, pane: AgentDetailPane, world: &World) -> ui::AgentChat {
    use ui::{agent_chat::Availability, agent_chat::Conversation, chat_turn::State as Turn};
    let name = &agent.detail.name;
    let deactivated = matches!(
        agent.detail.status.and_then(|s| s.state),
        Some(Status::Deactivated(_))
    );
    let missing_key = world
        .settings
        .keys
        .iter()
        .find(|(provider, _, state)| {
            *provider == agent.provider && !matches!(state, ui::provider_key::State::Set(_))
        })
        .map(|(_, key, _)| *key);
    let busy = |text: String| Availability::Busy(ui::ChatBusy { text });
    let availability = match (
        deactivated,
        missing_key,
        chat.turns.last().and_then(|t| t.state.as_ref()),
    ) {
        (true, _, _) => Availability::Deactivated(ui::ChatDeactivated {
            text: format!("{name} is off. Turn it on to chat"),
        }),
        (false, Some(key), _) => Availability::Off(ui::ChatOff {
            text: format!("Chat is off until you add a {key} key"),
            action: "Add a key".into(),
        }),
        (false, None, Some(Turn::Running(_))) => busy(format!("{name} is answering")),
        (false, None, Some(Turn::Waiting(_))) => {
            busy(format!("{name} is waiting for your approval"))
        }
        (false, None, Some(Turn::Answered(_) | Turn::Failed(_) | Turn::Stopped(_)) | None) => {
            Availability::Ready(ui::ChatReady {
                placeholder: format!("Message {name}"),
            })
        }
    };
    let conversation = if chat.turns.is_empty() {
        Conversation::Empty(ui::ChatEmpty {
            hint: format!("Ask {name} about its work, or give it a one-off task"),
        })
    } else {
        Conversation::Turns(ui::ChatTurns {
            turns: chat.turns.clone(),
        })
    };
    ui::AgentChat {
        panes: DETAIL_PANES
            .iter()
            .map(|(p, label)| ui::AgentDetailPaneTab {
                pane: (*p).into(),
                label: (*label).into(),
                current: *p == pane,
            })
            .collect(),
        availability: Some(availability),
        conversation: Some(conversation),
    }
}
