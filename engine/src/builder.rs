//! What the agent builder offers, and what it makes of the user's choices: the triggers and their
//! filters, the schedules, and whether the agent can be created.

use crate::{
    proto::ui::v1::PermissionLevel,
    schedule::{self, Schedule},
    world::{Builder, CATEGORIES, FolderKind, Permission, Wakes, World},
};

/// The choices a trigger filter takes, by id.
pub enum Options {
    /// Fixed (id, label) pairs, the default first.
    Fixed(&'static [(&'static str, &'static str)]),
    /// "All", then every account.
    Accounts,
    /// "Any", then every folder the user made.
    Folders,
    /// The inbox categories, then every folder the user made: the watch folder.
    Into,
    /// "Any agent", then every agent.
    AnyAgent,
    /// Every agent; required, so there is no default.
    Agent,
    /// "All", then every calendar.
    Calendars,
    /// Fixed (id, label) pairs; required, so there is no default.
    Files(&'static [(&'static str, &'static str)]),
}

pub struct Filter {
    pub id: &'static str,
    pub label: &'static str,
    pub options: Options,
}

pub struct Trigger {
    pub id: &'static str,
    pub label: &'static str,
    pub filters: &'static [Filter],
}

const SENDER: Filter = Filter {
    id: "sender",
    label: "Sender",
    options: Options::Fixed(&[
        ("anyone", "Anyone"),
        ("known", "Known contacts"),
        ("new", "New senders"),
    ]),
};
const ACCOUNT: Filter = Filter {
    id: "account",
    label: "Account",
    options: Options::Accounts,
};
const FOLDER: Filter = Filter {
    id: "folder",
    label: "Folder",
    options: Options::Folders,
};
/// The default ids of "All" accounts or calendars, "Any" folder and "Any agent".
pub const ALL: &str = "all";
pub const ANY: &str = "any";

/// The lead times "Calendar event approaching" offers: (id, label, seconds).
pub const LEAD_TIMES: [(&str, &str, i64); 3] = [
    ("1-day", "1 day before", 86_400),
    ("1-hour", "1 hour before", 3_600),
    ("15-minutes", "15 minutes before", 900),
];

pub const TRIGGERS: [Trigger; 8] = [
    Trigger {
        id: "new-mail",
        label: "New mail arrived",
        filters: &[SENDER, ACCOUNT, FOLDER],
    },
    Trigger {
        id: "sorted",
        label: "Mail sorted into a category or folder",
        filters: &[
            Filter {
                id: "into",
                label: "Into",
                options: Options::Into,
            },
            SENDER,
            ACCOUNT,
        ],
    },
    Trigger {
        id: "flagged",
        label: "Mail flagged",
        filters: &[SENDER, ACCOUNT, FOLDER],
    },
    Trigger {
        id: "labelled",
        label: "Label or star added",
        filters: &[SENDER, ACCOUNT, FOLDER],
    },
    Trigger {
        id: "agent-finished",
        label: "Another agent finished",
        filters: &[Filter {
            id: "agent",
            label: "Agent",
            options: Options::Agent,
        }],
    },
    Trigger {
        id: "approval-resolved",
        label: "Approval resolved",
        filters: &[Filter {
            id: "from",
            label: "From",
            options: Options::AnyAgent,
        }],
    },
    Trigger {
        id: "event-approaching",
        label: "Calendar event approaching",
        filters: &[
            Filter {
                id: "lead-time",
                label: "Lead time",
                options: Options::Fixed(&[
                    (LEAD_TIMES[0].0, LEAD_TIMES[0].1),
                    (LEAD_TIMES[1].0, LEAD_TIMES[1].1),
                    (LEAD_TIMES[2].0, LEAD_TIMES[2].1),
                ]),
            },
            Filter {
                id: "calendar",
                label: "Calendar",
                options: Options::Calendars,
            },
        ],
    },
    Trigger {
        id: "file-changed",
        label: "File changed",
        filters: &[Filter {
            id: "file",
            label: "File",
            options: Options::Files(&[
                ("budget-sheet", "Budget sheet"),
                ("applications-sheet", "Applications sheet"),
            ]),
        }],
    },
];

pub fn trigger(id: &str) -> Option<&'static Trigger> {
    TRIGGERS.iter().find(|t| t.id == id)
}

/// A lead time's label, by id.
pub fn lead_label(id: &str) -> &str {
    LEAD_TIMES
        .iter()
        .find(|(i, ..)| *i == id)
        .map_or(id, |(_, label, _)| label)
}

/// What a required filter shows while nothing is picked, and what the trigger then still needs.
pub fn required(filter: &Filter) -> Option<(&'static str, &'static str)> {
    match filter.options {
        Options::Agent => Some(("Pick an agent", "Pick the agent it waits for")),
        Options::Files(_) => Some(("Pick a file", "Pick the file it watches")),
        Options::Fixed(_)
        | Options::Accounts
        | Options::Folders
        | Options::Into
        | Options::AnyAgent
        | Options::Calendars => None,
    }
}

/// The choices a filter takes as (id, label), an optional filter's default first.
pub fn options(filter: &Filter, world: &World) -> Vec<(String, String)> {
    let pairs = |items: &[(&str, &str)]| -> Vec<(String, String)> {
        items
            .iter()
            .map(|(id, label)| ((*id).to_string(), (*label).to_string()))
            .collect()
    };
    let user_folders = || {
        world
            .folders
            .iter()
            .filter(|f| f.kind == FolderKind::User)
            .map(|f| (f.id.clone(), f.name.clone()))
    };
    let agents = || world.agents.iter().map(|a| (a.id.clone(), a.name.clone()));
    match &filter.options {
        Options::Fixed(items) | Options::Files(items) => pairs(items),
        Options::Accounts => pairs(&[(ALL, "All")])
            .into_iter()
            .chain(
                world
                    .accounts
                    .iter()
                    .map(|a| (a.id.clone(), a.name.clone())),
            )
            .collect(),
        Options::Folders => pairs(&[(ANY, "Any")])
            .into_iter()
            .chain(user_folders())
            .collect(),
        Options::Into => CATEGORIES
            .iter()
            .map(|(_, id, name)| ((*id).to_string(), (*name).to_string()))
            .chain(user_folders())
            .collect(),
        Options::AnyAgent => pairs(&[(ANY, "Any agent")])
            .into_iter()
            .chain(agents())
            .collect(),
        Options::Agent => agents().collect(),
        Options::Calendars => pairs(&[(ALL, "All")])
            .into_iter()
            .chain(
                world
                    .calendars
                    .iter()
                    .map(|c| (c.id.clone(), c.name.clone())),
            )
            .collect(),
    }
}

/// A filter's value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pick {
    Picked {
        id: String,
        label: String,
    },
    /// A required filter with nothing picked.
    NotPicked,
}

/// The choice picked for a filter, if it is still among the choices; else an optional filter's
/// default, or nothing for a required one.
pub fn pick(wakes: &Wakes, trigger: &str, filter: &Filter, world: &World) -> Pick {
    let options = options(filter, world);
    let stored = wakes
        .filters
        .iter()
        .find(|(t, f, _)| t == trigger && f == filter.id)
        .and_then(|(_, _, choice)| options.iter().find(|(id, _)| id == choice));
    let default = required(filter)
        .is_none()
        .then(|| options.first())
        .flatten();
    match stored.or(default) {
        Some((id, label)) => Pick::Picked {
            id: id.clone(),
            label: label.clone(),
        },
        None => Pick::NotPicked,
    }
}

/// The id picked for a filter of a trigger, by filter id.
pub fn picked(wakes: &Wakes, trigger: &Trigger, filter: &str, world: &World) -> Option<String> {
    let filter = trigger.filters.iter().find(|f| f.id == filter)?;
    match pick(wakes, trigger.id, filter, world) {
        Pick::Picked { id, .. } => Some(id),
        Pick::NotPicked => None,
    }
}

/// What a chosen trigger still needs: a required filter not picked.
pub fn missing(trigger: &Trigger, wakes: &Wakes, world: &World) -> Option<&'static str> {
    trigger
        .filters
        .iter()
        .find_map(|f| match (pick(wakes, trigger.id, f, world), required(f)) {
            (Pick::NotPicked, Some((_, missing))) => Some(missing),
            _ => None,
        })
}

/// The filters in one line: "Into For you, from anyone, all accounts"
pub fn filter_summary(trigger: &Trigger, wakes: &Wakes, world: &World) -> String {
    let parts: Vec<String> = trigger
        .filters
        .iter()
        .map(|f| match pick(wakes, trigger.id, f, world) {
            Pick::NotPicked => required(f)
                .map_or("", |(placeholder, _)| placeholder)
                .into(),
            Pick::Picked { id, label } => match (f.id, id.as_str()) {
                ("sender", _) => format!("from {}", label.to_lowercase()),
                ("account", ALL) => "all accounts".into(),
                ("folder", ANY) => "any folder".into(),
                ("folder", _) => format!("in {label}"),
                ("into", _) => format!("into {label}"),
                ("calendar", ALL) => "all calendars".into(),
                ("calendar", _) => format!("the {label} calendar"),
                _ => label,
            },
        })
        .collect();
    schedule::capitalized(&parts.join(", "))
}

/// A schedule the builder offers.
pub enum Choice {
    None,
    Every(u32),
    At {
        weekdays: bool,
        minute: u32,
    },
    /// Every `Builder.interval_seconds`.
    Custom,
}

pub struct ScheduleOption {
    pub id: &'static str,
    pub choice: Choice,
}

pub const NO_SCHEDULE: &ScheduleOption = &SCHEDULES[0];
pub const CUSTOM: &ScheduleOption = &SCHEDULES[5];

pub const SCHEDULES: [ScheduleOption; 6] = [
    ScheduleOption {
        id: "none",
        choice: Choice::None,
    },
    ScheduleOption {
        id: "every-15-minutes",
        choice: Choice::Every(schedule::MIN_INTERVAL_SECONDS),
    },
    ScheduleOption {
        id: "hourly",
        choice: Choice::Every(3_600),
    },
    ScheduleOption {
        id: "daily-8",
        choice: Choice::At {
            weekdays: false,
            minute: 8 * 60,
        },
    },
    ScheduleOption {
        id: "weekdays-9",
        choice: Choice::At {
            weekdays: true,
            minute: 9 * 60,
        },
    },
    ScheduleOption {
        id: "custom",
        choice: Choice::Custom,
    },
];

pub fn schedule_option(id: &str) -> Option<&'static ScheduleOption> {
    SCHEDULES.iter().find(|s| s.id == id)
}

/// The schedule an option makes, with the custom interval for Custom.
pub fn schedule_of(option: &ScheduleOption, interval_seconds: u32) -> Schedule {
    match option.choice {
        Choice::None => Schedule::Triggers,
        Choice::Every(seconds) => Schedule::Interval(seconds),
        Choice::At { weekdays, minute } => Schedule::Times {
            weekdays,
            minutes: vec![minute],
        },
        Choice::Custom => Schedule::Interval(interval_seconds),
    }
}

/// The menu's label: "No schedule", "Every hour", "Custom schedule".
pub fn option_label(option: &ScheduleOption) -> String {
    match option.choice {
        Choice::None => "No schedule".into(),
        Choice::Custom => "Custom schedule".into(),
        Choice::Every(_) | Choice::At { .. } => schedule_of(option, 0).label().unwrap_or_default(),
    }
}

/// The schedule the builder makes.
pub fn schedule(builder: &Builder) -> Schedule {
    schedule_of(builder.schedule, builder.interval_seconds)
}

/// The chosen triggers' labels, in order.
pub fn trigger_labels(wakes: &Wakes) -> Vec<&'static str> {
    TRIGGERS
        .iter()
        .filter(|t| wakes.triggers.iter().any(|c| c == t.id))
        .map(|t| t.label)
        .collect()
}

/// When the agent wakes, in one line: "Mail sorted into a category or folder, daily at 8:00"
pub fn wakes(builder: &Builder) -> String {
    schedule::wakes(&trigger_labels(&builder.wakes), &schedule(builder))
}

/// The actions at a level: "Read and label, Calendar"
pub fn at_level(permissions: &[Permission], level: PermissionLevel) -> String {
    let names: Vec<&str> = permissions
        .iter()
        .filter(|p| p.level == level)
        .map(|p| p.label.as_str())
        .collect();
    if names.is_empty() {
        "Nothing".into()
    } else {
        names.join(", ")
    }
}

/// Why the agent cannot be created yet.
pub fn incomplete(builder: &Builder, world: &World) -> Option<String> {
    if builder.name.trim().is_empty() {
        return Some("Name it".into());
    }
    if builder.instructions.trim().is_empty() {
        return Some("Say what it should do".into());
    }
    if builder.wakes.triggers.is_empty() && matches!(builder.schedule.choice, Choice::None) {
        return Some("Pick when it wakes up".into());
    }
    TRIGGERS
        .iter()
        .filter(|t| builder.wakes.triggers.iter().any(|c| c == t.id))
        .find_map(|t| missing(t, &builder.wakes, world).map(|m| format!("{m} on “{}”", t.label)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        store::tests::{NOW, fixture},
        world::Levels,
    };

    fn builder() -> Builder {
        Builder {
            name: "Deadline keeper".into(),
            instructions: "Add deadlines".into(),
            schedule: NO_SCHEDULE,
            interval_seconds: 900,
            ask_before_new_recipient: true,
            wakes: Wakes::default(),
            accounts: vec![],
            permissions: vec![Permission {
                id: "calendar".into(),
                label: "Calendar".into(),
                hint: None,
                level: PermissionLevel::Auto,
                levels: Levels::All,
            }],
        }
    }

    #[test]
    fn wakes_lists_triggers_in_order_then_the_schedule() {
        let mut b = builder();
        b.wakes.triggers = vec!["event-approaching".into(), "sorted".into()];
        b.schedule = CUSTOM;
        b.interval_seconds = 1200;
        assert_eq!(
            wakes(&b),
            "Mail sorted into a category or folder, Calendar event approaching, every 20 minutes"
        );
        assert_eq!(schedule(&b), Schedule::Interval(1200));
        b.schedule = schedule_option("weekdays-9").unwrap();
        assert_eq!(
            schedule(&b),
            Schedule::Times {
                weekdays: true,
                minutes: vec![540]
            }
        );
        assert_eq!(option_label(b.schedule), "Weekdays at 9:00");
        assert_eq!(option_label(NO_SCHEDULE), "No schedule");
    }

    #[test]
    fn at_level_names_the_actions_or_nothing() {
        let b = builder();
        assert_eq!(at_level(&b.permissions, PermissionLevel::Auto), "Calendar");
        assert_eq!(at_level(&b.permissions, PermissionLevel::Ask), "Nothing");
    }

    #[test]
    fn filters_resolve_ids_to_choices_defaults_or_not_picked() {
        let w = fixture().load(NOW, None).unwrap();
        let sorted = trigger("sorted").unwrap();
        let finished = trigger("agent-finished").unwrap();
        let mut wakes = Wakes::default();
        // Unpicked: an optional filter takes its default, a required one is not picked.
        assert_eq!(
            pick(&wakes, "sorted", &sorted.filters[0], &w),
            Pick::Picked {
                id: "for-you".into(),
                label: "For you".into()
            }
        );
        assert_eq!(
            pick(&wakes, "agent-finished", &finished.filters[0], &w),
            Pick::NotPicked
        );
        assert_eq!(
            missing(finished, &wakes, &w),
            Some("Pick the agent it waits for")
        );
        wakes.filters = vec![
            ("sorted".into(), "into".into(), "bills".into()),
            ("agent-finished".into(), "agent".into(), "sorter".into()),
            ("sorted".into(), "account".into(), "gone".into()),
        ];
        assert_eq!(picked(&wakes, sorted, "into", &w).as_deref(), Some("bills"));
        assert_eq!(
            picked(&wakes, finished, "agent", &w).as_deref(),
            Some("sorter")
        );
        assert_eq!(
            picked(&wakes, sorted, "account", &w).as_deref(),
            Some(ALL),
            "an id no longer offered falls back to the default"
        );
        assert_eq!(
            filter_summary(sorted, &wakes, &w),
            "Into Bills, from anyone, all accounts"
        );
        assert_eq!(missing(finished, &wakes, &w), None);
    }
}
