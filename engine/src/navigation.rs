//! Navigation and selection: which screen is shown and what is selected on each, as one explicit
//! state with one pure transition function. Which ids exist comes from a `Catalog`, so a selection
//! always names something visible, or is explicitly none.

use std::collections::BTreeSet;

use crate::{
    path::{self, Parsed},
    proto::ui::v1::{
        AgentDetailPane, AgentsPane, BuilderStep, InboxCategory, MailProvider, SettingsSection,
    },
};

/// What the navigation needs to know about the data: ids in display order, `None` for an unknown
/// account, folder or thread. Ids are URL-safe tokens.
pub trait Catalog {
    fn listing(&self, listing: &Listing) -> Option<Vec<&str>>;
    fn messages(&self, thread: &str) -> Option<Vec<&str>>;
    /// The messages of a thread shown expanded when it opens.
    fn opened(&self, thread: &str) -> Vec<&str>;
    fn agents(&self) -> Vec<&str>;
    /// The pane an agent that can chat opens on; `None` for an agent without chat.
    fn chat(&self, agent: &str) -> Option<AgentDetailPane>;
    /// Events of the calendars that are shown.
    fn events(&self) -> Vec<&str>;
    /// The day an event is on.
    fn day_of(&self, event: &str) -> Option<&str>;
    fn quarantined(&self) -> Vec<&str>;
    /// Briefing editions, newest first.
    fn editions(&self) -> Vec<&str>;
    /// Calendar days in order.
    fn days(&self) -> Vec<&str>;
    fn today(&self) -> Option<&str>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selection {
    None,
    Id(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mailbox {
    All,
    Account(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Listing {
    Mailbox(Mailbox, InboxCategory),
    Folder(String),
}

impl Listing {
    pub const ALL: Self = Self::Mailbox(Mailbox::All, InboxCategory::ForYou);
}

/// Connecting an account: picking a provider, then that provider's sign-in or IMAP form, then syncing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectStep {
    Picking,
    Connecting(MailProvider),
    Syncing(MailProvider),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Screen {
    SignIn,
    Connect(ConnectStep),
    Inbox,
    Thread {
        id: String,
        expanded: BTreeSet<String>,
    },
    Agents,
    AgentBuilder,
    Settings,
    Calendar,
    Quarantine,
    Briefing,
}

/// The current screen, plus what each screen shows when it is opened again.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Navigation {
    pub screen: Screen,
    pub listing: Listing,
    /// The row in the inbox reading pane.
    pub thread: Selection,
    pub agent: Selection,
    /// Activity or chat of the selected agent; chat only for an agent that has it.
    pub detail_pane: AgentDetailPane,
    pub section: SettingsSection,
    pub event: Selection,
    pub quarantined: Selection,
    pub edition: Selection,
    /// The agents pane a narrow layout shows.
    pub pane: AgentsPane,
    /// The agent builder step a narrow layout shows.
    pub step: BuilderStep,
    /// The calendar day a narrow layout shows.
    pub day: Selection,
}

/// The part of the navigation a web path carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    SignIn,
    Connect,
    Inbox(Listing),
    Thread(String),
    Agents,
    /// The agents screen with this agent's chat.
    AgentChat(String),
    AgentBuilder,
    Settings(SettingsSection),
    Calendar,
    Quarantine,
    Briefing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[expect(
    clippy::enum_variant_names,
    reason = "SelectEvent and CloseEvent name calendar events"
)]
pub enum Event {
    Open(Route),
    OpenPath(String),
    Back,
    PickProvider(MailProvider),
    ConfirmConnect,
    SelectCategory(InboxCategory),
    SelectThread(String),
    ExpandMessage(String),
    CollapseMessage(String),
    SelectAgent(String),
    SelectEvent(String),
    CloseEvent,
    SelectQuarantined(String),
    SelectEdition(String),
    SelectPane(AgentsPane),
    SelectDetailPane(AgentDetailPane),
    SelectStep(BuilderStep),
    SelectDay(String),
}

pub fn start(catalog: &impl Catalog) -> Navigation {
    Navigation {
        screen: Screen::SignIn,
        thread: first(catalog.listing(&Listing::ALL).unwrap_or_default()),
        listing: Listing::ALL,
        agent: first(catalog.agents()),
        detail_pane: AgentDetailPane::Activity,
        section: SettingsSection::Accounts,
        event: Selection::None,
        quarantined: first(catalog.quarantined()),
        edition: first(catalog.editions()),
        pane: AgentsPane::Agents,
        step: BuilderStep::Instructions,
        day: catalog
            .today()
            .map_or(Selection::None, |day| Selection::Id(day.into())),
    }
}

pub fn route(navigation: &Navigation) -> Route {
    match &navigation.screen {
        Screen::SignIn => Route::SignIn,
        Screen::Connect(_) => Route::Connect,
        Screen::Inbox => Route::Inbox(navigation.listing.clone()),
        Screen::Thread { id, .. } => Route::Thread(id.clone()),
        Screen::Agents => match (&navigation.agent, navigation.detail_pane) {
            (Selection::Id(id), AgentDetailPane::Chat) => Route::AgentChat(id.clone()),
            _ => Route::Agents,
        },
        Screen::AgentBuilder => Route::AgentBuilder,
        Screen::Settings => Route::Settings(navigation.section),
        Screen::Calendar => Route::Calendar,
        Screen::Quarantine => Route::Quarantine,
        Screen::Briefing => Route::Briefing,
    }
}

pub fn transition(navigation: &Navigation, event: Event, catalog: &impl Catalog) -> Navigation {
    let mut next = navigation.clone();
    match event {
        Event::Open(route) => return open(navigation, route, catalog),
        Event::OpenPath(path) => {
            let route = match path::parse(&path) {
                Parsed::Route(route) => route,
                Parsed::NotFound => Route::Inbox(Listing::ALL),
            };
            return open(navigation, route, catalog);
        }
        Event::Back => {
            if navigation.screen == Screen::Agents
                && navigation.detail_pane == AgentDetailPane::Chat
            {
                next.detail_pane = AgentDetailPane::Activity;
                return next;
            }
            next.screen = match &navigation.screen {
                Screen::Thread { .. } | Screen::Quarantine => Screen::Inbox,
                Screen::AgentBuilder => Screen::Agents,
                Screen::Connect(ConnectStep::Connecting(_)) => {
                    Screen::Connect(ConnectStep::Picking)
                }
                screen @ (Screen::SignIn
                | Screen::Connect(ConnectStep::Picking | ConnectStep::Syncing(_))
                | Screen::Inbox
                | Screen::Agents
                | Screen::Settings
                | Screen::Calendar
                | Screen::Briefing) => screen.clone(),
            }
        }
        Event::PickProvider(provider) => {
            if navigation.screen == Screen::Connect(ConnectStep::Picking) {
                next.screen = Screen::Connect(ConnectStep::Connecting(provider));
            }
        }
        Event::ConfirmConnect => {
            if let Screen::Connect(ConnectStep::Connecting(provider)) = navigation.screen {
                next.screen = Screen::Connect(ConnectStep::Syncing(provider));
            }
        }
        Event::SelectCategory(category) => match &navigation.listing {
            Listing::Mailbox(mailbox, _) => {
                return show(
                    navigation,
                    Listing::Mailbox(mailbox.clone(), category),
                    catalog,
                );
            }
            Listing::Folder(_) => {}
        },
        Event::SelectThread(id) => pick(&mut next.thread, id, rows(navigation, catalog)),
        Event::ExpandMessage(message) => {
            if let Screen::Thread { id, expanded } = &mut next.screen
                && catalog
                    .messages(id)
                    .is_some_and(|messages| messages.contains(&message.as_str()))
            {
                expanded.insert(message);
            }
        }
        Event::CollapseMessage(message) => {
            if let Screen::Thread { expanded, .. } = &mut next.screen {
                expanded.remove(&message);
            }
        }
        Event::SelectAgent(id) => {
            if catalog.agents().contains(&id.as_str()) {
                next.detail_pane = catalog.chat(&id).unwrap_or(AgentDetailPane::Activity);
                next.agent = Selection::Id(id);
                next.pane = AgentsPane::Agents;
                next.screen = Screen::Agents;
            }
        }
        Event::SelectEvent(id) => {
            if catalog.events().contains(&id.as_str()) {
                if let Some(day) = catalog.day_of(&id) {
                    next.day = Selection::Id(day.into());
                }
                next.event = Selection::Id(id);
                next.screen = Screen::Calendar;
            }
        }
        Event::CloseEvent => next.event = Selection::None,
        Event::SelectQuarantined(id) => pick(&mut next.quarantined, id, catalog.quarantined()),
        Event::SelectEdition(id) => pick(&mut next.edition, id, catalog.editions()),
        Event::SelectPane(pane) => {
            next.pane = pane;
            // The queue is its own pane: leaving for it leaves the phone's full-screen chat.
            if pane == AgentsPane::Approvals {
                next.detail_pane = AgentDetailPane::Activity;
            }
        }
        Event::SelectDetailPane(pane) => {
            let chats = match &navigation.agent {
                Selection::Id(id) => catalog.chat(id).is_some(),
                Selection::None => false,
            };
            if pane == AgentDetailPane::Activity || chats {
                next.detail_pane = pane;
            }
        }
        Event::SelectStep(step) => next.step = step,
        Event::SelectDay(id) => pick(&mut next.day, id, catalog.days()),
    }
    next
}

fn open(navigation: &Navigation, route: Route, catalog: &impl Catalog) -> Navigation {
    let mut next = navigation.clone();
    next.screen = match route {
        Route::SignIn => Screen::SignIn,
        Route::Connect => Screen::Connect(ConnectStep::Picking),
        Route::Inbox(listing) => return show(navigation, listing, catalog),
        Route::Thread(id) => {
            if catalog.messages(&id).is_none() {
                return show(navigation, Listing::ALL, catalog);
            }
            if rows(navigation, catalog).contains(&id.as_str()) {
                next.thread = Selection::Id(id.clone());
            }
            let expanded = catalog.opened(&id).into_iter().map(String::from).collect();
            Screen::Thread { id, expanded }
        }
        Route::Agents => {
            next.detail_pane = AgentDetailPane::Activity;
            Screen::Agents
        }
        Route::AgentChat(id) => {
            if catalog.agents().contains(&id.as_str()) && catalog.chat(&id).is_some() {
                next.agent = Selection::Id(id);
                next.detail_pane = AgentDetailPane::Chat;
                next.pane = AgentsPane::Agents;
            } else {
                next.detail_pane = AgentDetailPane::Activity;
            }
            Screen::Agents
        }
        Route::AgentBuilder => {
            next.step = BuilderStep::Instructions;
            Screen::AgentBuilder
        }
        Route::Settings(section) => {
            next.section = section;
            Screen::Settings
        }
        Route::Calendar => Screen::Calendar,
        Route::Quarantine => Screen::Quarantine,
        Route::Briefing => Screen::Briefing,
    };
    next
}

/// Shows a listing in the inbox, keeping the selected row if it is still visible and selecting the
/// first row otherwise. An unknown account or folder is not found: it lands on all inboxes.
fn show(navigation: &Navigation, listing: Listing, catalog: &impl Catalog) -> Navigation {
    let (listing, rows) = match catalog.listing(&listing) {
        Some(rows) => (listing, rows),
        None => (
            Listing::ALL,
            catalog.listing(&Listing::ALL).unwrap_or_default(),
        ),
    };
    let thread = match &navigation.thread {
        Selection::Id(id) if rows.contains(&id.as_str()) => navigation.thread.clone(),
        Selection::Id(_) | Selection::None => first(rows),
    };
    Navigation {
        screen: Screen::Inbox,
        listing,
        thread,
        ..navigation.clone()
    }
}

fn rows<'a>(navigation: &Navigation, catalog: &'a impl Catalog) -> Vec<&'a str> {
    catalog.listing(&navigation.listing).unwrap_or_default()
}

fn first(ids: Vec<&str>) -> Selection {
    ids.first()
        .map_or(Selection::None, |id| Selection::Id((*id).into()))
}

/// Selects `id` if it is one of `ids`; an id that is not shown changes nothing.
fn pick(selection: &mut Selection, id: String, ids: Vec<&str>) {
    if ids.contains(&id.as_str()) {
        *selection = Selection::Id(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two accounts; account "a" has threads 1 and 2 in For you and 3 in Updates, "b" has 4 in For
    /// you. Folder "f" holds 2 and 4. Thread 1 has messages m1 and m2 and opens with m2 expanded.
    struct Fixture;

    impl Catalog for Fixture {
        fn listing(&self, listing: &Listing) -> Option<Vec<&str>> {
            Some(match listing {
                Listing::Mailbox(Mailbox::All, InboxCategory::ForYou) => vec!["1", "2", "4"],
                Listing::Mailbox(Mailbox::Account(a), InboxCategory::ForYou) if a == "a" => {
                    vec!["1", "2"]
                }
                Listing::Mailbox(Mailbox::Account(b), InboxCategory::ForYou) if b == "b" => {
                    vec!["4"]
                }
                Listing::Mailbox(Mailbox::All, InboxCategory::Updates) => vec!["3"],
                Listing::Mailbox(Mailbox::Account(a), InboxCategory::Updates) if a == "a" => {
                    vec!["3"]
                }
                Listing::Mailbox(Mailbox::All | Mailbox::Account(_), InboxCategory::Reading) => {
                    vec![]
                }
                Listing::Folder(f) if f == "f" => vec!["2", "4"],
                _ => return None,
            })
        }
        fn messages(&self, thread: &str) -> Option<Vec<&str>> {
            match thread {
                "1" => Some(vec!["m1", "m2"]),
                "2" | "3" | "4" => Some(vec![]),
                _ => None,
            }
        }
        fn opened(&self, thread: &str) -> Vec<&str> {
            match thread {
                "1" => vec!["m2"],
                _ => vec![],
            }
        }
        fn agents(&self) -> Vec<&str> {
            vec!["gatekeeper", "scribe", "assistant", "mine"]
        }
        fn chat(&self, agent: &str) -> Option<AgentDetailPane> {
            match agent {
                "assistant" => Some(AgentDetailPane::Chat),
                "mine" => Some(AgentDetailPane::Activity),
                _ => None,
            }
        }
        fn events(&self) -> Vec<&str> {
            vec!["e1", "e2"]
        }
        fn day_of(&self, event: &str) -> Option<&str> {
            match event {
                "e1" => Some("mon"),
                "e2" => Some("tue"),
                _ => None,
            }
        }
        fn quarantined(&self) -> Vec<&str> {
            vec!["q1", "q2"]
        }
        fn editions(&self) -> Vec<&str> {
            vec!["morning", "evening"]
        }
        fn days(&self) -> Vec<&str> {
            vec!["mon", "tue"]
        }
        fn today(&self) -> Option<&str> {
            Some("tue")
        }
    }

    fn id(id: &str) -> Selection {
        Selection::Id(id.into())
    }

    fn account(id: &str, category: InboxCategory) -> Listing {
        Listing::Mailbox(Mailbox::Account(id.into()), category)
    }

    fn run(from: &Navigation, events: impl IntoIterator<Item = Event>) -> Navigation {
        events
            .into_iter()
            .fold(from.clone(), |nav, event| transition(&nav, event, &Fixture))
    }

    fn inbox() -> Navigation {
        run(&start(&Fixture), [Event::Open(Route::Inbox(Listing::ALL))])
    }

    #[test]
    fn starts_on_sign_in_with_first_items_selected() {
        let nav = start(&Fixture);
        assert_eq!(nav.screen, Screen::SignIn);
        assert_eq!(nav.listing, Listing::ALL);
        assert_eq!(nav.thread, id("1"));
        assert_eq!(nav.agent, id("gatekeeper"));
        assert_eq!(nav.event, Selection::None);
        assert_eq!(nav.quarantined, id("q1"));
        assert_eq!(nav.edition, id("morning"));
        assert_eq!(nav.pane, AgentsPane::Agents);
        assert_eq!(nav.step, BuilderStep::Instructions);
        assert_eq!(nav.day, id("tue"));
    }

    #[test]
    fn narrow_layout_selections() {
        let nav = run(
            &inbox(),
            [
                Event::SelectPane(AgentsPane::Approvals),
                Event::SelectDay("mon".into()),
                Event::SelectDay("nope".into()),
                Event::Open(Route::AgentBuilder),
                Event::SelectStep(BuilderStep::Summary),
            ],
        );
        assert_eq!(
            (nav.pane, nav.step, &nav.day),
            (AgentsPane::Approvals, BuilderStep::Summary, &id("mon"))
        );
        let nav = run(&nav, [Event::Back, Event::Open(Route::AgentBuilder)]);
        assert_eq!(
            nav.step,
            BuilderStep::Instructions,
            "the builder opens on its first step"
        );
        let nav = run(&nav, [Event::SelectAgent("scribe".into())]);
        assert_eq!(nav.pane, AgentsPane::Agents, "a selected agent is shown");
    }

    #[test]
    fn opening_each_screen() {
        let from = inbox();
        let cases = [
            (Route::SignIn, Screen::SignIn),
            (Route::Connect, Screen::Connect(ConnectStep::Picking)),
            (Route::Agents, Screen::Agents),
            (Route::AgentBuilder, Screen::AgentBuilder),
            (Route::Calendar, Screen::Calendar),
            (Route::Quarantine, Screen::Quarantine),
            (Route::Briefing, Screen::Briefing),
        ];
        for (route, screen) in cases {
            let nav = run(&from, [Event::Open(route.clone())]);
            assert_eq!(nav.screen, screen, "{route:?}");
            assert_eq!(
                nav.thread, from.thread,
                "{route:?} keeps the inbox selection"
            );
        }
        let nav = run(
            &from,
            [Event::Open(Route::Settings(SettingsSection::Rules))],
        );
        assert_eq!(
            (nav.screen, nav.section),
            (Screen::Settings, SettingsSection::Rules)
        );
    }

    #[test]
    fn opening_a_thread_expands_its_default_messages_and_selects_its_row() {
        let nav = run(
            &inbox(),
            [
                Event::SelectThread("2".into()),
                Event::Open(Route::Thread("1".into())),
            ],
        );
        assert_eq!(
            nav.screen,
            Screen::Thread {
                id: "1".into(),
                expanded: ["m2".into()].into()
            }
        );
        assert_eq!(nav.thread, id("1"));
    }

    #[test]
    fn opening_a_thread_outside_the_listing_keeps_the_selection() {
        let nav = run(&inbox(), [Event::Open(Route::Thread("3".into()))]);
        assert!(matches!(nav.screen, Screen::Thread { ref id, .. } if id == "3"));
        assert_eq!(nav.thread, id("1"));
    }

    #[test]
    fn unknown_ids_land_on_all_inboxes() {
        let from = run(
            &inbox(),
            [Event::Open(Route::Inbox(account(
                "a",
                InboxCategory::Updates,
            )))],
        );
        for route in [
            Route::Thread("nope".into()),
            Route::Inbox(account("nope", InboxCategory::ForYou)),
            Route::Inbox(Listing::Folder("nope".into())),
        ] {
            let nav = run(&from, [Event::Open(route.clone())]);
            assert_eq!(
                (&nav.screen, &nav.listing, &nav.thread),
                (&Screen::Inbox, &Listing::ALL, &id("1")),
                "{route:?}"
            );
        }
    }

    #[test]
    fn selection_follows_the_category() {
        let nav = run(
            &inbox(),
            [
                Event::SelectThread("2".into()),
                Event::SelectCategory(InboxCategory::Updates),
            ],
        );
        assert_eq!(nav.thread, id("3"));
        let nav = run(&nav, [Event::SelectCategory(InboxCategory::Reading)]);
        assert_eq!(nav.thread, Selection::None);
        let nav = run(&nav, [Event::SelectCategory(InboxCategory::ForYou)]);
        assert_eq!(nav.thread, id("1"));
    }

    #[test]
    fn selection_survives_a_scope_change_only_when_visible() {
        let nav = run(
            &inbox(),
            [
                Event::SelectThread("4".into()),
                Event::Open(Route::Inbox(Listing::Folder("f".into()))),
            ],
        );
        assert_eq!(nav.thread, id("4"));
        let nav = run(
            &nav,
            [Event::Open(Route::Inbox(account(
                "a",
                InboxCategory::ForYou,
            )))],
        );
        assert_eq!(nav.thread, id("1"));
    }

    #[test]
    fn a_folder_has_no_categories() {
        let from = run(
            &inbox(),
            [Event::Open(Route::Inbox(Listing::Folder("f".into())))],
        );
        assert_eq!(
            run(&from, [Event::SelectCategory(InboxCategory::Updates)]),
            from
        );
    }

    #[test]
    fn selecting_something_not_shown_changes_nothing() {
        let from = run(
            &inbox(),
            [
                Event::SelectThread("2".into()),
                Event::SelectEvent("e2".into()),
                Event::SelectQuarantined("q2".into()),
                Event::SelectEdition("evening".into()),
            ],
        );
        assert_eq!(
            (&from.thread, &from.event, &from.quarantined, &from.edition),
            (&id("2"), &id("e2"), &id("q2"), &id("evening"))
        );
        for event in [
            Event::SelectThread("3".into()),
            Event::SelectEvent("nope".into()),
            Event::SelectQuarantined("nope".into()),
            Event::SelectEdition("nope".into()),
        ] {
            assert_eq!(run(&from, [event.clone()]), from, "{event:?}");
        }
    }

    #[test]
    fn selecting_an_agent_opens_the_agents_screen() {
        let nav = run(&inbox(), [Event::SelectAgent("scribe".into())]);
        assert_eq!((nav.screen, nav.agent), (Screen::Agents, id("scribe")));
        let from = inbox();
        assert_eq!(run(&from, [Event::SelectAgent("nope".into())]), from);
    }

    #[test]
    fn agent_detail_panes() {
        let nav = run(&inbox(), [Event::SelectAgent("assistant".into())]);
        assert_eq!(
            nav.detail_pane,
            AgentDetailPane::Chat,
            "Assistant opens on chat"
        );
        assert_eq!(route(&nav), Route::AgentChat("assistant".into()));
        let back = run(&nav, [Event::Back]);
        assert_eq!(
            (&back.screen, back.detail_pane),
            (&Screen::Agents, AgentDetailPane::Activity),
            "back leaves the chat for the activity"
        );
        assert_eq!(run(&back, [Event::Back]), back);
        let nav = run(&nav, [Event::SelectAgent("mine".into())]);
        assert_eq!(
            nav.detail_pane,
            AgentDetailPane::Activity,
            "the agent's default"
        );
        let nav = run(&nav, [Event::SelectDetailPane(AgentDetailPane::Chat)]);
        assert_eq!(nav.detail_pane, AgentDetailPane::Chat);
        let queue = run(&nav, [Event::SelectPane(AgentsPane::Approvals)]);
        assert_eq!(queue.detail_pane, AgentDetailPane::Activity);
        let nav = run(&nav, [Event::SelectAgent("scribe".into())]);
        assert_eq!(nav.detail_pane, AgentDetailPane::Activity);
        assert_eq!(
            run(&nav, [Event::SelectDetailPane(AgentDetailPane::Chat)]),
            nav,
            "an agent without chat has no chat pane"
        );
    }

    #[test]
    fn chat_paths() {
        let from = inbox();
        let nav = run(&from, [Event::OpenPath("/u/0/agents/mine/chat".into())]);
        assert_eq!(
            (&nav.screen, &nav.agent, nav.detail_pane),
            (&Screen::Agents, &id("mine"), AgentDetailPane::Chat)
        );
        let nav = run(&nav, [Event::OpenPath("/u/0/agents".into())]);
        assert_eq!(nav.detail_pane, AgentDetailPane::Activity);
        for path in ["/u/0/agents/scribe/chat", "/u/0/agents/nope/chat"] {
            let nav = run(&from, [Event::OpenPath(path.into())]);
            assert_eq!(route(&nav), Route::Agents, "{path}");
            assert_eq!(nav.agent, from.agent, "{path}");
        }
    }

    #[test]
    fn calendar_event_selects_and_closes() {
        let nav = run(&inbox(), [Event::SelectEvent("e1".into())]);
        assert_eq!(
            (&nav.screen, &nav.event, &nav.day),
            (&Screen::Calendar, &id("e1"), &id("mon")),
            "selecting an event opens the calendar on its day"
        );
        assert_eq!(run(&nav, [Event::CloseEvent]).event, Selection::None);
    }

    #[test]
    fn messages_expand_and_collapse_in_the_open_thread_only() {
        let thread = run(&inbox(), [Event::Open(Route::Thread("1".into()))]);
        let nav = run(
            &thread,
            [
                Event::ExpandMessage("m1".into()),
                Event::ExpandMessage("nope".into()),
                Event::CollapseMessage("m2".into()),
            ],
        );
        assert_eq!(
            nav.screen,
            Screen::Thread {
                id: "1".into(),
                expanded: ["m1".into()].into()
            }
        );
        let from = inbox();
        assert_eq!(run(&from, [Event::ExpandMessage("m1".into())]), from);
    }

    #[test]
    fn back() {
        let from = run(&inbox(), [Event::SelectThread("2".into())]);
        let cases = [
            (Route::Thread("1".into()), Screen::Inbox, id("1")),
            (Route::Quarantine, Screen::Inbox, id("2")),
            (Route::AgentBuilder, Screen::Agents, id("2")),
        ];
        for (route, screen, thread) in cases {
            let nav = run(&from, [Event::Open(route.clone()), Event::Back]);
            assert_eq!((nav.screen, nav.thread), (screen, thread), "{route:?}");
        }
        for route in [
            Route::SignIn,
            Route::Connect,
            Route::Inbox(Listing::ALL),
            Route::Agents,
            Route::Settings(SettingsSection::Models),
            Route::Calendar,
            Route::Briefing,
        ] {
            let nav = run(&from, [Event::Open(route.clone())]);
            assert_eq!(run(&nav, [Event::Back]), nav, "{route:?}");
        }
    }

    #[test]
    fn connect_steps() {
        let picking = run(&start(&Fixture), [Event::Open(Route::Connect)]);
        let connecting = run(&picking, [Event::PickProvider(MailProvider::Icloud)]);
        assert_eq!(
            connecting.screen,
            Screen::Connect(ConnectStep::Connecting(MailProvider::Icloud))
        );
        assert_eq!(run(&connecting, [Event::Back]), picking);
        let syncing = run(&connecting, [Event::ConfirmConnect]);
        assert_eq!(
            syncing.screen,
            Screen::Connect(ConnectStep::Syncing(MailProvider::Icloud))
        );
        for event in [
            Event::Back,
            Event::ConfirmConnect,
            Event::PickProvider(MailProvider::Gmail),
        ] {
            assert_eq!(run(&syncing, [event.clone()]), syncing, "{event:?}");
        }
        assert_eq!(run(&picking, [Event::ConfirmConnect]), picking);
        assert_eq!(
            run(&connecting, [Event::Open(Route::Connect)]).screen,
            Screen::Connect(ConnectStep::Picking)
        );
    }

    #[test]
    fn paths_open_their_route_and_unknown_paths_land_on_the_inbox() {
        let from = start(&Fixture);
        let nav = run(&from, [Event::OpenPath("/u/0/thread/1".into())]);
        assert_eq!(route(&nav), Route::Thread("1".into()));
        let nav = run(&from, [Event::OpenPath("/u/0/account/b".into())]);
        assert_eq!(
            (route(&nav), nav.thread),
            (Route::Inbox(account("b", InboxCategory::ForYou)), id("4"))
        );
        for unknown in ["/", "/u/0/nope", "/u/0/thread/nope"] {
            let nav = run(&from, [Event::OpenPath(unknown.into())]);
            assert_eq!(route(&nav), Route::Inbox(Listing::ALL), "{unknown}");
        }
    }
}
