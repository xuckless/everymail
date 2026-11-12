//! The web path of a route and its inverse: `/u/<n>/` followed by the view and what it shows, such
//! as a thread's id, an agent's id and pane, or the calendar's view and day (docs/BACKEND.md,
//! Landing page). `<n>` is 0 until several sign-ins are supported.

use crate::{
    date::Date,
    navigation::{Listing, Mailbox, Mode, Pane, PaneChoice, Route, Section},
    proto::ui::v1::InboxCategory,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Parsed {
    Route(Route),
    NotFound,
}

const SIGN_IN: &str = "/sign-in";
/// The site's root shows all inboxes.
const ROOT: &str = "/";
const USER: &str = "/u/0/";

/// For you is a mailbox's own path; the other categories add a segment.
const CATEGORIES: [(InboxCategory, &str); 2] = [
    (InboxCategory::Updates, "updates"),
    (InboxCategory::Reading, "reading"),
];

const SECTIONS: [(Section, &str); 7] = [
    (Section::Accounts, "accounts"),
    (Section::AgentPermissions, "agent-permissions"),
    (Section::Rules, "rules"),
    (Section::BlockList, "block-list"),
    (Section::Models, "models"),
    (Section::Appearance, "appearance"),
    (Section::YourData, "your-data"),
];

const PANES: [(Pane, &str); 4] = [
    (Pane::Results, "results"),
    (Pane::Activity, "activity"),
    (Pane::Chat, "chat"),
    (Pane::Settings, "settings"),
];

const MODES: [(Mode, &str); 3] = [
    (Mode::Week, "week"),
    (Mode::Month, "month"),
    (Mode::Year, "year"),
];

fn segment<T: PartialEq + Copy>(table: &[(T, &'static str)], value: T) -> &'static str {
    table
        .iter()
        .find(|(v, _)| *v == value)
        .map_or("", |(_, segment)| segment)
}

fn value<T: Copy>(table: &[(T, &str)], segment: &str) -> Option<T> {
    table.iter().find(|(_, s)| *s == segment).map(|(v, _)| *v)
}

pub fn path(route: &Route) -> String {
    let view = match route {
        Route::SignIn => return SIGN_IN.into(),
        Route::Connect => "connect".into(),
        Route::Inbox(Listing::Mailbox(mailbox, category)) => {
            let mailbox = match mailbox {
                Mailbox::All => "inbox".into(),
                Mailbox::Account(id) => format!("account/{id}"),
            };
            match CATEGORIES.iter().find(|(c, _)| c == category) {
                Some((_, segment)) => format!("{mailbox}/{segment}"),
                None => mailbox,
            }
        }
        Route::Inbox(Listing::Folder(id)) => format!("folder/{id}"),
        Route::Inbox(Listing::Search { scope, .. }) => {
            return path(&Route::Inbox((**scope).clone()));
        }
        Route::Thread(id) => format!("thread/{id}"),
        Route::Agents => "agents".into(),
        Route::Agent(id, PaneChoice::Default) => format!("agents/{id}"),
        Route::Agent(id, PaneChoice::Other(pane)) => {
            format!("agents/{id}/{}", segment(&PANES, *pane))
        }
        Route::Approvals => "approvals".into(),
        Route::AgentBuilder => "agents/new".into(),
        Route::Settings(section) => format!("settings/{}", segment(&SECTIONS, *section)),
        Route::Calendar(None) => "calendar".into(),
        Route::Calendar(Some((mode, day))) => {
            format!("calendar/{}/{}", segment(&MODES, *mode), day.id())
        }
        Route::Quarantine => "quarantine".into(),
        Route::Briefing => "briefing".into(),
    };
    format!("{USER}{view}")
}

pub fn parse(path: &str) -> Parsed {
    match path {
        SIGN_IN => return Parsed::Route(Route::SignIn),
        ROOT => return Parsed::Route(Route::Inbox(Listing::ALL)),
        _ => {}
    }
    let segments: Vec<&str> = match path.strip_prefix(USER) {
        Some(view) => view.split('/').collect(),
        None => return Parsed::NotFound,
    };
    let mailbox = |mailbox: Mailbox, category: Option<&str>| {
        let category = match category {
            None => Some(InboxCategory::ForYou),
            Some(segment) => CATEGORIES
                .iter()
                .find(|(_, s)| *s == segment)
                .map(|(c, _)| *c),
        };
        category.map(|c| Route::Inbox(Listing::Mailbox(mailbox, c)))
    };
    let route = match segments.as_slice() {
        ["connect"] => Some(Route::Connect),
        ["inbox"] => mailbox(Mailbox::All, None),
        ["inbox", category] => mailbox(Mailbox::All, Some(category)),
        ["account", id] if !id.is_empty() => mailbox(Mailbox::Account((*id).into()), None),
        ["account", id, category] if !id.is_empty() => {
            mailbox(Mailbox::Account((*id).into()), Some(category))
        }
        ["folder", id] if !id.is_empty() => Some(Route::Inbox(Listing::Folder((*id).into()))),
        ["thread", id] if !id.is_empty() => Some(Route::Thread((*id).into())),
        ["agents"] => Some(Route::Agents),
        ["agents", "new"] => Some(Route::AgentBuilder),
        ["agents", id] if !id.is_empty() => Some(Route::Agent((*id).into(), PaneChoice::Default)),
        ["agents", id, pane] if !id.is_empty() && *id != "new" => {
            value(&PANES, pane).map(|pane| Route::Agent((*id).into(), PaneChoice::Other(pane)))
        }
        ["approvals"] => Some(Route::Approvals),
        ["settings", segment] => value(&SECTIONS, segment).map(Route::Settings),
        ["calendar"] => Some(Route::Calendar(None)),
        ["calendar", mode, day] => value(&MODES, mode)
            .zip(Date::parse(day))
            .map(|at| Route::Calendar(Some(at))),
        ["quarantine"] => Some(Route::Quarantine),
        ["briefing"] => Some(Route::Briefing),
        _ => None,
    };
    route.map_or(Parsed::NotFound, Parsed::Route)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn routes() -> Vec<Route> {
        let mut routes = vec![
            Route::SignIn,
            Route::Connect,
            Route::Inbox(Listing::Folder("receipts".into())),
            Route::Thread("t-slides".into()),
            Route::Agents,
            Route::Agent("assistant".into(), PaneChoice::Default),
            Route::AgentBuilder,
            Route::Approvals,
            Route::Calendar(None),
            Route::Quarantine,
            Route::Briefing,
        ];
        for category in [
            InboxCategory::ForYou,
            InboxCategory::Updates,
            InboxCategory::Reading,
        ] {
            routes.push(Route::Inbox(Listing::Mailbox(Mailbox::All, category)));
            routes.push(Route::Inbox(Listing::Mailbox(
                Mailbox::Account("personal".into()),
                category,
            )));
        }
        routes.extend(SECTIONS.map(|(section, _)| Route::Settings(section)));
        routes.extend(
            PANES.map(|(pane, _)| Route::Agent("gatekeeper".into(), PaneChoice::Other(pane))),
        );
        let day = Date::parse("2026-11-11").expect("a date");
        routes.extend(MODES.map(|(mode, _)| Route::Calendar(Some((mode, day)))));
        routes
    }

    #[test]
    fn every_route_round_trips() {
        for route in routes() {
            assert_eq!(
                parse(&path(&route)),
                Parsed::Route(route.clone()),
                "{route:?}"
            );
        }
    }

    #[test]
    fn paths_are_distinct() {
        let mut paths: Vec<String> = routes().iter().map(path).collect();
        let count = paths.len();
        paths.sort();
        paths.dedup();
        assert_eq!(paths.len(), count);
    }

    #[test]
    fn known_paths() {
        let cases = [
            ("/sign-in", Route::SignIn),
            (
                "/u/0/inbox",
                Route::Inbox(Listing::Mailbox(Mailbox::All, InboxCategory::ForYou)),
            ),
            (
                "/u/0/account/laurentian/reading",
                Route::Inbox(Listing::Mailbox(
                    Mailbox::Account("laurentian".into()),
                    InboxCategory::Reading,
                )),
            ),
            ("/u/0/thread/t-slides", Route::Thread("t-slides".into())),
            (
                "/u/0/agents/assistant",
                Route::Agent("assistant".into(), PaneChoice::Default),
            ),
            (
                "/u/0/agents/job-tracker/activity",
                Route::Agent("job-tracker".into(), PaneChoice::Other(Pane::Activity)),
            ),
            ("/u/0/approvals", Route::Approvals),
            (
                "/u/0/calendar/month/2026-12-01",
                Route::Calendar(Some((
                    Mode::Month,
                    Date::parse("2026-12-01").expect("a date"),
                ))),
            ),
            (
                "/u/0/settings/block-list",
                Route::Settings(Section::BlockList),
            ),
        ];
        for (text, route) in cases {
            assert_eq!(parse(text), Parsed::Route(route.clone()), "{text}");
            assert_eq!(path(&route), text);
        }
    }

    #[test]
    fn unknown_paths_are_not_found() {
        for text in [
            "",
            "//",
            "/u/0",
            "/u/0/",
            "/u/1/inbox",
            "/u/0/inbox/",
            "/u/0/inbox/for-you",
            "/u/0/inbox/spam",
            "/u/0/account/",
            "/u/0/account//updates",
            "/u/0/folder/",
            "/u/0/thread/",
            "/u/0/thread/a/b",
            "/u/0/agents/edit/x",
            "/u/0/agents//chat",
            "/u/0/agents/assistant/chat/",
            "/u/0/agents/assistant/runs",
            "/u/0/agents/new/chat",
            "/u/0/approvals/a-ostra",
            "/u/0/calendar/week",
            "/u/0/calendar/day/2026-11-11",
            "/u/0/calendar/week/2026-11-31",
            "/u/0/calendar/week/2026-11-11/",
            "/u/0/settings/",
            "/u/0/settings/nope",
            "/u/0/body/m-1",
            "/sign-in/",
        ] {
            assert_eq!(parse(text), Parsed::NotFound, "{text}");
        }
    }
}
