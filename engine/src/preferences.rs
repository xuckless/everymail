//! The user's display preferences, each one explicit value the views follow.

use crate::proto::ui::v1::{Density, Theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Preferences {
    pub theme: Theme,
    pub density: Density,
}

#[derive(Clone, Copy, Debug)]
pub enum Preference {
    Theme(Theme),
    Density(Density),
}

pub const START: Preferences = Preferences {
    theme: Theme::System,
    density: Density::Comfortable,
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        demo,
        proto::ui::v1::{SettingsSection, settings_view::Section},
        view,
    };

    #[test]
    fn density_is_set_apart_from_theme_and_marked_in_settings() {
        let set = [
            Preference::Theme(Theme::Dark),
            Preference::Density(Density::Compact),
        ]
        .into_iter()
        .fold(START, transition);
        assert_eq!(
            set,
            Preferences {
                theme: Theme::Dark,
                density: Density::Compact
            }
        );
        let world = demo::world();
        let Some(Section::Appearance(appearance)) =
            view::settings(SettingsSection::Appearance, set, &world).section
        else {
            panic!("the appearance section");
        };
        let selected: Vec<i32> = appearance
            .densities
            .iter()
            .filter(|d| d.selected)
            .map(|d| d.density)
            .collect();
        assert_eq!(selected, [i32::from(Density::Compact)]);
    }
}
