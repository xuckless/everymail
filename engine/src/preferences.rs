//! The user's display preferences, each one explicit value the views follow.

use crate::proto::ui::v1::{Density, Sidebar, Theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Preferences {
    pub theme: Theme,
    pub density: Density,
    pub sidebar: Sidebar,
}

#[derive(Clone, Copy, Debug)]
pub enum Preference {
    Theme(Theme),
    Density(Density),
    Sidebar(Sidebar),
}

pub const START: Preferences = Preferences {
    theme: Theme::System,
    density: Density::Comfortable,
    sidebar: Sidebar::Expanded,
};

pub fn transition(preferences: Preferences, preference: Preference) -> Preferences {
    match preference {
        Preference::Theme(theme) => Preferences {
            theme,
            ..preferences
        },
        Preference::Density(density) => Preferences {
            density,
            ..preferences
        },
        Preference::Sidebar(sidebar) => Preferences {
            sidebar,
            ..preferences
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_preference_is_set_apart() {
        let set = [
            Preference::Theme(Theme::Dark),
            Preference::Density(Density::Compact),
            Preference::Sidebar(Sidebar::Collapsed),
        ]
        .into_iter()
        .fold(START, transition);
        assert_eq!(
            set,
            Preferences {
                theme: Theme::Dark,
                density: Density::Compact,
                sidebar: Sidebar::Collapsed,
            }
        );
    }
}
