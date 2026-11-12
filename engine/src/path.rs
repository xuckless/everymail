//! The web path of a route and its inverse: `/u/<n>/` followed by the view and, for a thread or an
//! agent's chat, its id (docs/BACKEND.md, Landing page). `<n>` is 0 until several sign-ins are supported.

use crate::{
    navigation::{Listing, Mailbox, Route},
    proto::ui::v1::{InboxCategory, SettingsSection},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Parsed {
    Route(Route),
    NotFound,
}

const SIGN_IN: &str = "/sign-in";
const USER: &str = "/u/0/";

/// For you is a mailbox's own path; the other categories add a segment.
const CATEGORIES: [(InboxCategory, &str); 2] = [
    (InboxCategory::Updates, "updates"),
    (InboxCategory::Reading, "reading"),
];

const SECTIONS: [(SettingsSection, &str); 7] = [
    (SettingsSection::Accounts, "accounts"),
    (SettingsSection::AgentPermissions, "agent-permissions"),
    (SettingsSection::Rules, "rules"),
    (SettingsSection::BlockList, "block-list"),
    (SettingsSection::Models, "models"),
    (SettingsSection::Appearance, "appearance"),
    (SettingsSection::YourData, "your-data"),
];

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
        Route::Thread(id) => format!("thread/{id}"),
        Route::Agents => "agents".into(),
        Route::AgentChat(id) => format!("agents/{id}/chat"),
        Route::AgentBuilder => "agents/new".into(),
        Route::Settings(section) => {
            let segment = SECTIONS
                .iter()
                .find(|(s, _)| s == section)
                .map_or("", |(_, segment)| segment);
            format!("settings/{segment}")
        }
        Route::Calendar => "calendar".into(),
        Route::Quarantine => "quarantine".into(),
        Route::Briefing => "briefing".into(),
    };
    format!("{USER}{view}")
}

pub fn parse(path: &str) -> Parsed {
    if path == SIGN_IN {
        return Parsed::Route(Route::SignIn);
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
        ["agents", id, "chat"] if !id.is_empty() => Some(Route::AgentChat((*id).into())),
        ["settings", segment] => SECTIONS
            .iter()
            .find(|(_, s)| s == segment)
            .map(|(section, _)| Route::Settings(*section)),
        ["calendar"] => Some(Route::Calendar),
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
            Route::AgentChat("assistant".into()),
            Route::AgentBuilder,
            Route::Calendar,
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
                "/u/0/agents/assistant/chat",
                Route::AgentChat("assistant".into()),
            ),
            (
                "/u/0/settings/block-list",
                Route::Settings(SettingsSection::BlockList),
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
            "/",
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
            "/u/0/agents/edit",
            "/u/0/agents/assistant",
            "/u/0/agents//chat",
            "/u/0/agents/assistant/chat/",
            "/u/0/settings/",
            "/u/0/settings/nope",
            "/u/0/body/m-1",
            "/sign-in/",
        ] {
            assert_eq!(parse(text), Parsed::NotFound, "{text}");
        }
    }
}
