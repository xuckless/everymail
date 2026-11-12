use crate::{
    demo::World,
    preferences::Preferences,
    proto::ui::v1::{
        self as ui, Density, SettingsSection, Theme, block_list_settings::State as Blocked,
        models_settings::Keys, settings_view::Section,
    },
};

const SECTIONS: [(SettingsSection, &str); 7] = [
    (SettingsSection::Accounts, "Accounts"),
    (SettingsSection::AgentPermissions, "Agent permissions"),
    (SettingsSection::Rules, "Rules"),
    (SettingsSection::BlockList, "Block list"),
    (SettingsSection::Models, "Models and keys"),
    (SettingsSection::Appearance, "Appearance"),
    (SettingsSection::YourData, "Your data"),
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

pub fn settings(
    section: SettingsSection,
    preferences: Preferences,
    world: &World,
) -> ui::SettingsView {
    let settings = &world.settings;
    let content = match section {
        // The dispatch rejects an unspecified section, so navigation never holds one.
        SettingsSection::Accounts | SettingsSection::Unspecified => {
            Section::Accounts(ui::AccountsSettings {
                accounts: world
                    .accounts
                    .iter()
                    .map(|a| ui::SettingsAccount {
                        id: a.id.into(),
                        name: a.name.into(),
                        provider: a.provider.into(),
                        status: Some(ui::AccountStatus {
                            state: Some(a.status.clone()),
                        }),
                        signature: Some(a.signature.clone()),
                        status_line: format!("{} · {}", a.provider_name, status_label(&a.status)),
                    })
                    .collect(),
            })
        }
        SettingsSection::AgentPermissions => {
            Section::AgentPermissions(ui::AgentPermissionsSettings {
                defaults: settings.defaults.clone(),
                notify_when_waiting: settings.notify_when_waiting,
                agents_paused: settings.agents_paused,
            })
        }
        SettingsSection::Rules => Section::Rules(ui::RulesSettings {
            rules: settings.rules.clone(),
        }),
        SettingsSection::BlockList => Section::BlockList(ui::BlockListSettings {
            state: Some(if settings.blocked.is_empty() {
                Blocked::Nobody(ui::NobodyBlocked {})
            } else {
                Blocked::Blocked(ui::BlockedSenders {
                    senders: settings.blocked.clone(),
                })
            }),
        }),
        SettingsSection::Models => Section::Models(ui::ModelsSettings {
            keys: Some(if world.has_key() {
                Keys::Some(ui::SomeKeySet {})
            } else {
                Keys::None(ui::NoKeySet {})
            }),
            providers: settings
                .keys
                .iter()
                .map(|(provider, name, state)| ui::ProviderKey {
                    provider: (*provider).into(),
                    name: (*name).into(),
                    key_label: format!("{name} key"),
                    state: Some(*state),
                })
                .collect(),
            tasks: settings.tasks.clone(),
            budget: Some(settings.budget.clone()),
        }),
        SettingsSection::Appearance => Section::Appearance(ui::AppearanceSettings {
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
        SettingsSection::YourData => Section::YourData(ui::YourDataSettings {}),
    };
    ui::SettingsView {
        sections: SECTIONS
            .iter()
            .map(|(s, label)| ui::SettingsTab {
                section: (*s).into(),
                label: (*label).into(),
                selected: *s == section,
            })
            .collect(),
        section: Some(content),
    }
}

fn status_label(status: &ui::account_status::State) -> &str {
    use ui::account_status::State;
    match status {
        State::Synced(synced) => &synced.label,
        State::SignInExpired(expired) => &expired.label,
    }
}
