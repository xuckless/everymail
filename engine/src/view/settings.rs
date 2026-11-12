use super::{
    account_state, disclosure, grouped,
    mail::{model, model_choices},
    state_label,
};
use crate::{
    live::{Export, Field, Live},
    navigation::{Disclosure, Navigation, Section as Shown},
    preferences::Preferences,
    proto::ui::v1::{
        self as ui, Density, Theme,
        block_list_settings::State as Blocked,
        models_settings::{Budget, Keys},
        rule_action::Action as Act,
        rule_condition::Condition as When,
        rule_condition_choice::Parameter,
        settings_account::Signature,
        settings_view::Section,
        your_data_settings::Export as ExportView,
    },
    time,
    world::{Condition, FolderKind, Levels, Permission, Rule, RuleAction, World},
};

const SECTIONS: [(Shown, &str); 7] = [
    (Shown::Accounts, "Accounts"),
    (Shown::AgentPermissions, "Agent permissions"),
    (Shown::Rules, "Rules"),
    (Shown::BlockList, "Block list"),
    (Shown::Models, "Models and keys"),
    (Shown::Appearance, "Appearance"),
    (Shown::YourData, "Your data"),
];

const THEMES: [(Theme, &str); 3] = [
    (Theme::System, "System"),
    (Theme::Dark, "Dark"),
    (Theme::Light, "Light"),
];

const DENSITIES: [(Density, &str); 3] = [
    (Density::Compact, "Compact"),
    (Density::Comfortable, "Comfortable"),
    (Density::Roomy, "Roomy"),
];

/// Permissions as a control shows them, with the levels the Backend offers.
pub fn permissions(permissions: &[Permission]) -> Vec<ui::Permission> {
    use ui::permission::Levels as View;
    permissions
        .iter()
        .map(|p| {
            let (hint, levels) = match &p.levels {
                Levels::All => (
                    p.hint.clone().unwrap_or_default(),
                    View::All(ui::AllLevels {}),
                ),
                Levels::AskAtMost(reason) => (
                    reason.clone(),
                    View::AskAtMost(ui::AskAtMost {
                        reason: reason.clone(),
                    }),
                ),
            };
            ui::Permission {
                id: p.id.clone(),
                action: p.label.clone(),
                hint,
                level: p.level.into(),
                levels: Some(levels),
            }
        })
        .collect()
}

fn condition(condition: &Condition) -> (ui::RuleCondition, String) {
    let (when, text) = match condition {
        Condition::SenderContains(text) => (
            When::SenderContains(text.clone()),
            format!("sender contains {text}"),
        ),
        Condition::SenderDomain(domain) => (
            When::SenderDomain(domain.clone()),
            format!("sender domain is {domain}"),
        ),
        Condition::SubjectContains(words) => (
            When::SubjectContains(ui::SubjectWords {
                words: words.clone(),
            }),
            format!(
                "subject contains {}",
                words
                    .iter()
                    .map(|w| format!("“{w}”"))
                    .collect::<Vec<_>>()
                    .join(" or ")
            ),
        ),
        Condition::Newsletter => (
            When::Newsletter(ui::NewsletterMail {}),
            "mail is a newsletter".into(),
        ),
        Condition::NewSenderWithLink => (
            When::NewSenderWithLink(ui::NewSenderWithLink {}),
            "sender is new and mail has a link".into(),
        ),
    };
    (
        ui::RuleCondition {
            condition: Some(when),
        },
        text,
    )
}

fn action(action: &RuleAction, world: &World) -> (ui::RuleAction, String) {
    let (act, text) = match action {
        RuleAction::MoveTo(folder) => (
            Act::MoveToFolderId(folder.clone()),
            format!(
                "move to {}",
                world
                    .folders
                    .iter()
                    .find(|f| f.id == *folder)
                    .map_or("", |f| &f.name)
            ),
        ),
        RuleAction::Archive => (Act::Archive(ui::ArchiveMail {}), "archive".into()),
        RuleAction::Trash => (Act::Trash(ui::TrashMail {}), "move to trash".into()),
        RuleAction::MarkImportant => (
            Act::MarkImportant(ui::MarkImportant {}),
            "mark important".into(),
        ),
        RuleAction::HandTo(agent) => (
            Act::HandToAgentId(agent.clone()),
            format!(
                "hand to {}",
                world.agent(agent).map_or("", |a| a.name.as_str())
            ),
        ),
        RuleAction::KeepForBriefing => (
            Act::KeepForBriefing(ui::KeepForBriefing {}),
            "keep for the briefing".into(),
        ),
    };
    (ui::RuleAction { action: Some(act) }, text)
}

fn rule(rule: &Rule, world: &World) -> ui::Rule {
    let (when, when_text) = condition(&rule.condition);
    let (actions, texts): (Vec<ui::RuleAction>, Vec<String>) =
        rule.actions.iter().map(|a| action(a, world)).unzip();
    ui::Rule {
        id: rule.id.clone(),
        enabled: rule.enabled,
        text: format!("{when_text} → {}", texts.join(", ")),
        condition: Some(when),
        actions,
    }
}

/// The conditions a new rule can use: (condition with an empty parameter, label, placeholder).
fn condition_choices() -> Vec<ui::RuleConditionChoice> {
    let text = |placeholder: &str| {
        Parameter::Text(ui::RuleTextParameter {
            placeholder: placeholder.into(),
        })
    };
    let none = || Parameter::None(ui::RuleNoParameter {});
    [
        (
            Condition::SenderContains(String::new()),
            "Sender contains",
            text("mira@laurentian.ca"),
        ),
        (
            Condition::SenderDomain(String::new()),
            "Sender domain is",
            text("laurentian.ca"),
        ),
        (
            Condition::SubjectContains(Vec::new()),
            "Subject contains any of",
            text("receipt, invoice"),
        ),
        (Condition::Newsletter, "Mail is a newsletter", none()),
        (
            Condition::NewSenderWithLink,
            "Sender is new and mail has a link",
            none(),
        ),
    ]
    .into_iter()
    .map(|(c, label, parameter)| ui::RuleConditionChoice {
        condition: Some(condition(&c).0),
        label: label.into(),
        parameter: Some(parameter),
    })
    .collect()
}

/// The actions a new rule can take: the fixed ones, then one per folder and per agent.
fn action_choices(world: &World) -> Vec<ui::RuleActionChoice> {
    [
        RuleAction::Archive,
        RuleAction::Trash,
        RuleAction::MarkImportant,
        RuleAction::KeepForBriefing,
    ]
    .into_iter()
    .chain(
        world
            .folders
            .iter()
            .filter(|f| f.kind == FolderKind::User)
            .map(|f| RuleAction::MoveTo(f.id.clone())),
    )
    .chain(
        world
            .agents
            .iter()
            .map(|a| RuleAction::HandTo(a.id.clone())),
    )
    .map(|a| {
        let (action, text) = action(&a, world);
        ui::RuleActionChoice {
            action: Some(action),
            label: crate::schedule::capitalized(&text),
        }
    })
    .collect()
}

/// "Blocked by Security Guard · today", "Blocked by you · Oct 2"
fn blocked_by(by: &str, at: i64, now: i64) -> String {
    let day = time::date(at);
    let when = if day == time::date(now) {
        "today".into()
    } else {
        time::day_month(day)
    };
    format!("Blocked by {by} · {when}")
}

pub fn settings(
    navigation: &Navigation,
    preferences: Preferences,
    world: &World,
    live: &Live,
) -> ui::SettingsView {
    let settings = &world.settings;
    let section = navigation.section;
    let content = match section {
        Shown::Accounts => Section::Accounts(ui::AccountsSettings {
            accounts: world
                .accounts
                .iter()
                .map(|a| ui::SettingsAccount {
                    id: a.id.clone(),
                    name: a.name.clone(),
                    provider: a.provider.into(),
                    state: Some(account_state(a, world.now)),
                    signature: Some(match &a.signature {
                        Some(text) => Signature::Text(text.clone()),
                        None => Signature::None(ui::NoSignature {}),
                    }),
                    status_line: format!("{} · {}", a.provider_name, state_label(a, world.now)),
                })
                .collect(),
        }),
        Shown::AgentPermissions => Section::AgentPermissions(ui::AgentPermissionsSettings {
            defaults: permissions(&settings.defaults),
            notify_when_waiting: settings.notify_when_waiting,
            agents_paused: settings.agents_paused,
        }),
        Shown::Rules => Section::Rules(ui::RulesSettings {
            rules: settings.rules.iter().map(|r| rule(r, world)).collect(),
            conditions: condition_choices(),
            actions: action_choices(world),
            new_rule: Some(disclosure(navigation, &Disclosure::NewRule)),
            new_rule_revision: live.revisions.of(&Field::NewRule),
        }),
        Shown::BlockList => Section::BlockList(ui::BlockListSettings {
            state: Some(if settings.blocked.is_empty() {
                Blocked::Nobody(ui::NobodyBlocked {})
            } else {
                Blocked::Blocked(ui::BlockedSenders {
                    senders: settings
                        .blocked
                        .iter()
                        .map(|b| ui::BlockedSender {
                            address: b.address.clone(),
                            blocked_by: blocked_by(&b.by, b.at, world.now),
                        })
                        .collect(),
                })
            }),
        }),
        Shown::Models => Section::Models(ui::ModelsSettings {
            keys: Some(if world.ai_on() {
                Keys::Some(ui::SomeKeySet {})
            } else {
                Keys::None(ui::NoKeySet {})
            }),
            providers: settings
                .keys
                .iter()
                .map(|k| {
                    use ui::provider_key::State;
                    ui::ProviderKey {
                        provider: k.provider.into(),
                        name: k.name.clone(),
                        key_label: format!("{} key", k.name),
                        state: Some(
                            match (
                                k.set,
                                navigation.open.contains(&Disclosure::Key(k.provider)),
                            ) {
                                (true, true) => State::Replacing(ui::KeyReplacing {}),
                                (true, false) => State::Set(ui::KeySet {}),
                                (false, _) => State::Missing(ui::KeyMissing {}),
                            },
                        ),
                        revision: live.revisions.of(&Field::Key(k.provider)),
                    }
                })
                .collect(),
            tasks: settings
                .tasks
                .iter()
                .map(|t| ui::TaskModel {
                    id: t.id.clone(),
                    task: t.label.clone(),
                    hint: t.hint.clone(),
                    model: world.model(&t.model).map(model),
                    choices: model_choices(&t.model, world),
                })
                .collect(),
            budget: Some(
                match (
                    settings.budget_tokens,
                    navigation.open.contains(&Disclosure::Budget),
                ) {
                    (None, false) => Budget::Unlimited(ui::NoBudget {}),
                    (None, true) => Budget::Entering(ui::BudgetEntering {}),
                    (Some(tokens), _) => Budget::Daily(ui::DailyBudget {
                        tokens: grouped(tokens),
                        used: format!("{} used today", grouped(settings.used_tokens)),
                    }),
                },
            ),
            budget_revision: live.revisions.of(&Field::Budget),
        }),
        Shown::Appearance => Section::Appearance(ui::AppearanceSettings {
            themes: THEMES
                .iter()
                .map(|(choice, label)| ui::ThemeChoice {
                    theme: (*choice).into(),
                    label: (*label).into(),
                    selected: *choice == preferences.theme,
                })
                .collect(),
            densities: DENSITIES
                .iter()
                .map(|(density, label)| ui::DensityChoice {
                    density: (*density).into(),
                    label: (*label).into(),
                    selected: *density == preferences.density,
                })
                .collect(),
        }),
        Shown::YourData => Section::YourData(ui::YourDataSettings {
            export: Some(match &live.export {
                Export::Idle => ExportView::Idle(ui::ExportIdle {}),
                Export::Running(percent) => ExportView::Running(ui::ExportRunning {
                    label: format!("Preparing your export · {percent}%"),
                    percent: *percent,
                }),
                Export::Ready { text, file } => ExportView::Ready(ui::ExportReady {
                    text: text.clone(),
                    file: file.clone(),
                }),
                Export::Failed(reason) => ExportView::Failed(ui::ExportFailed {
                    reason: reason.clone(),
                }),
            }),
        }),
    };
    ui::SettingsView {
        sections: SECTIONS
            .iter()
            .map(|(s, label)| ui::SettingsTab {
                section: ui::SettingsSection::from(*s).into(),
                label: (*label).into(),
                selected: *s == section,
            })
            .collect(),
        section: Some(content),
    }
}
