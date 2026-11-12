//! Navigation and selection: which screen is shown, what is selected and what is open on each, as
//! one explicit state with one pure transition function. Which ids exist comes from a `Catalog`,
//! so a selection always names something visible, or is explicitly none.

use std::collections::BTreeSet;

use crate::{
    date::Date,
    path::{self, Parsed},
    proto::ui::v1::{
        AgentDetailPane, BuilderStep, CalendarMode, InboxCategory, MailProvider, ModelProvider,
        SettingsSection,
    },
};

/// What the navigation needs to know about the data: ids in display order, `None` for an unknown
/// account, folder, thread or agent. Ids are URL-safe tokens.
pub trait Catalog {
    /// A user is signed in: every screen but sign-in needs one.
    fn signed_in(&self) -> bool;
    fn listing(&self, listing: &Listing) -> Option<Vec<&str>>;
    fn messages(&self, thread: &str) -> Option<Vec<&str>>;
    /// The messages of a thread shown expanded when it opens.
    fn opened(&self, thread: &str) -> Vec<&str>;
    /// The panes of an agent's page.
    fn panes(&self, agent: &str) -> Option<AgentPanes>;
    /// Events of the calendars that are shown.
    fn events(&self) -> Vec<&str>;
    /// The day an event is on.
    fn day_of(&self, event: &str) -> Option<Date>;
    /// A calendar a new event can be saved to exists.
    fn has_calendar(&self) -> bool;
    fn quarantined(&self) -> Vec<&str>;
    /// Briefing editions, newest first.
    fn editions(&self) -> Vec<&str>;
    /// Drafts the composer can open.
    fn drafts(&self) -> Vec<&str>;
    /// What the disclosure is about is shown: the key is set, the invitation and the run exist.
    fn discloses(&self, disclosure: &Disclosure) -> bool;
    /// Today, by the Backend's clock.
    fn today(&self) -> Date;
}

/// The settings sections, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Accounts,
    AgentPermissions,
    Rules,
    BlockList,
    Models,
    Appearance,
    YourData,
}

/// The calendar's view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Week,
    Month,
    Year,
}

/// A pane of an agent's page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pane {
    Results,
    Activity,
    Chat,
    Settings,
}

/// A step of the narrow agent builder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Instructions,
    Triggers,
    Permissions,
    Summary,
}

/// The builder's steps, in order.
pub const STEPS: [Step; 4] = [
    Step::Instructions,
    Step::Triggers,
    Step::Permissions,
    Step::Summary,
];

/// The wire forms of the navigation's enums: every value has one, and an unspecified one decodes
/// to none.
macro_rules! wire {
    ($($engine:ident <=> $proto:ident { $($variant:ident),+ $(,)? })+) => {$(
        impl From<$engine> for $proto {
            fn from(value: $engine) -> Self {
                match value { $($engine::$variant => $proto::$variant,)+ }
            }
        }

        impl TryFrom<$proto> for $engine {
            type Error = ();

            fn try_from(value: $proto) -> Result<Self, ()> {
                match value {
                    $($proto::$variant => Ok($engine::$variant),)+
                    $proto::Unspecified => Err(()),
                }
            }
        }
    )+};
}

wire! {
    Section <=> SettingsSection {
        Accounts, AgentPermissions, Rules, BlockList, Models, Appearance, YourData,
    }
    Mode <=> CalendarMode { Week, Month, Year }
    Pane <=> AgentDetailPane { Results, Activity, Chat, Settings }
    Step <=> BuilderStep { Instructions, Triggers, Permissions, Summary }
}

/// The panes an agent's page has, in order, and the one it opens on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AgentPanes {
    pub panes: &'static [Pane],
    pub default: Pane,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selection {
    None,
    Id(String),
}

/// The calendar's event drawer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Drawer {
    Closed,
    Event(String),
    /// A new event, not saved until the user saves it.
    New,
}

/// A part of a screen the user opens and closes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Disclosure {
    NewFolder,
    NewRule,
    Budget,
    /// A set key being replaced.
    Key(ModelProvider),
    /// An answered invitation being changed, by event.
    Invitation(String),
    /// A run shown with its steps.
    Run(String),
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
    /// The threads matching a query; clearing it shows `scope` again.
    Search {
        query: String,
        scope: Box<Listing>,
    },
}

impl Listing {
    pub const ALL: Self = Self::Mailbox(Mailbox::All, InboxCategory::ForYou);

    /// The listing a search runs over, or this one.
    pub fn scope(&self) -> &Listing {
        match self {
            Listing::Search { scope, .. } => scope,
            Listing::Mailbox(..) | Listing::Folder(_) => self,
        }
    }
}

/// Connecting an account: picking a provider, then that provider's sign-in or IMAP form, then syncing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectStep {
    Picking,
    Connecting(MailProvider),
    Syncing(MailProvider),
}

/// A pane of an agent's page: its default one, or another it has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneChoice {
    Default,
    Other(Pane),
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
    /// The list of agents (the phone's Agents tab).
    Agents,
    Agent {
        id: String,
        pane: PaneChoice,
    },
    AgentBuilder,
    Approvals,
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
    pub section: Section,
    pub event: Drawer,
    pub quarantined: Selection,
    pub edition: Selection,
    /// The agent builder step a narrow layout shows.
    pub step: Step,
    pub calendar: Mode,
    /// The calendar's selected day; the period shown is the week, month or year holding it.
    pub day: Date,
    /// The draft open in the composer, over any screen.
    pub composer: Selection,
    /// What the user opened, on any screen.
    pub open: BTreeSet<Disclosure>,
}

/// The part of the navigation a web path carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    SignIn,
    Connect,
    Inbox(Listing),
    Thread(String),
    Agents,
    Agent(String, PaneChoice),
    AgentBuilder,
    Approvals,
    Settings(Section),
    /// The calendar as it was left, or in this view on this day.
    Calendar(Option<(Mode, Date)>),
    Quarantine,
    Briefing,
}

/// How the address bar takes the path of a new navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    Push,
    Replace,
}

/// What moved the navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mover {
    /// The user, by an intent.
    User,
    /// The address bar: a path loaded, or the browser's back and forward.
    Address,
    /// The data, or the time.
    Data,
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
    /// Searches the current listing's scope; an empty query clears the search.
    Search(String),
    SelectThread(String),
    /// The row after or before the selected one in the screen's list.
    SelectNext,
    SelectPrevious,
    ExpandMessage(String),
    CollapseMessage(String),
    SelectAgent(String),
    SelectEvent(String),
    NewEvent,
    CloseEvent,
    SelectQuarantined(String),
    SelectEdition(String),
    SelectDetailPane(Pane),
    SelectStep(Step),
    SelectDay(String),
    SelectMonth(String),
    SelectCalendarMode(Mode),
    PreviousPeriod,
    NextPeriod,
    Today,
    OpenDraft(String),
    CloseComposer,
    Disclose(Disclosure),
    Conceal(Disclosure),
    /// The data changed: selections of things that are gone move on, and a screen whose subject is
    /// gone closes.
    Refresh,
}

pub fn start(catalog: &impl Catalog) -> Navigation {
    Navigation {
        screen: Screen::SignIn,
        thread: first(catalog.listing(&Listing::ALL).unwrap_or_default()),
        listing: Listing::ALL,
        section: Section::Accounts,
        event: Drawer::Closed,
        quarantined: first(catalog.quarantined()),
        edition: first(catalog.editions()),
        step: Step::Instructions,
        calendar: Mode::Week,
        day: catalog.today(),
        composer: Selection::None,
        open: BTreeSet::new(),
    }
}

pub fn route(navigation: &Navigation) -> Route {
    match &navigation.screen {
        Screen::SignIn => Route::SignIn,
        Screen::Connect(_) => Route::Connect,
        // A search is not a path: the address shows its scope.
        Screen::Inbox => Route::Inbox(navigation.listing.scope().clone()),
        Screen::Thread { id, .. } => Route::Thread(id.clone()),
        Screen::Agents => Route::Agents,
        Screen::Agent { id, pane } => Route::Agent(id.clone(), *pane),
        Screen::AgentBuilder => Route::AgentBuilder,
        Screen::Approvals => Route::Approvals,
        Screen::Settings => Route::Settings(navigation.section),
        Screen::Calendar => Route::Calendar(Some((navigation.calendar, navigation.day))),
        Screen::Quarantine => Route::Quarantine,
        Screen::Briefing => Route::Briefing,
    }
}

/// A user's navigation to another path is a new history entry; anything else replaces the entry.
pub fn entry(before: &Route, after: &Route, mover: Mover) -> Entry {
    match mover {
        Mover::User if before != after => Entry::Push,
        Mover::User | Mover::Address | Mover::Data => Entry::Replace,
    }
}

/// Whether a path names something the data holds. A signed-out user is shown sign-in for any path,
/// so every path is found.
pub fn found(path: &str, catalog: &impl Catalog) -> bool {
    if !catalog.signed_in() {
        return true;
    }
    match path::parse(path) {
        Parsed::NotFound => false,
        Parsed::Route(Route::Thread(id)) => catalog.messages(&id).is_some(),
        Parsed::Route(Route::Agent(id, _)) => catalog.panes(&id).is_some(),
        Parsed::Route(Route::Inbox(listing)) => catalog.listing(&listing).is_some(),
        Parsed::Route(_) => true,
    }
}

/// `pane` if the agent has it, as the page's choice: the default pane is not named.
fn choose(panes: AgentPanes, pane: Pane) -> Option<PaneChoice> {
    if pane == panes.default {
        Some(PaneChoice::Default)
    } else if panes.panes.contains(&pane) {
        Some(PaneChoice::Other(pane))
    } else {
        None
    }
}

/// The step before (-1) or after (1) `step`, if there is one.
pub fn step_by(step: Step, by: isize) -> Option<Step> {
    let at = STEPS.iter().position(|s| *s == step)?;
    STEPS.get(at.checked_add_signed(by)?).copied()
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
            next.screen = match &navigation.screen {
                Screen::Thread { .. } | Screen::Quarantine => Screen::Inbox,
                Screen::Agent { .. } | Screen::AgentBuilder | Screen::Approvals => Screen::Agents,
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
            Listing::Folder(_) | Listing::Search { .. } => {}
        },
        Event::Search(query) => {
            let scope = navigation.listing.scope().clone();
            let listing = if query.trim().is_empty() {
                scope
            } else {
                Listing::Search {
                    query,
                    scope: Box::new(scope),
                }
            };
            let mut next = show(navigation, listing, catalog);
            next.thread = first(catalog.listing(&next.listing).unwrap_or_default());
            return next;
        }
        Event::SelectThread(id) => pick(&mut next.thread, id, rows(navigation, catalog)),
        Event::SelectNext | Event::SelectPrevious => {
            let by = if event == Event::SelectNext { 1 } else { -1 };
            match navigation.screen {
                Screen::Inbox => adjacent(&mut next.thread, rows(navigation, catalog), by),
                Screen::Quarantine => adjacent(&mut next.quarantined, catalog.quarantined(), by),
                _ => {}
            }
        }
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
            if catalog.panes(&id).is_some() {
                next.screen = Screen::Agent {
                    id,
                    pane: PaneChoice::Default,
                };
            }
        }
        Event::SelectDetailPane(pane) => {
            if let Screen::Agent { id, .. } = &navigation.screen
                && let Some(choice) = catalog.panes(id).and_then(|panes| choose(panes, pane))
            {
                next.screen = Screen::Agent {
                    id: id.clone(),
                    pane: choice,
                };
            }
        }
        Event::SelectEvent(id) => {
            if catalog.events().contains(&id.as_str()) {
                if let Some(day) = catalog.day_of(&id) {
                    next.day = day;
                }
                next.event = Drawer::Event(id);
                next.screen = Screen::Calendar;
            }
        }
        Event::NewEvent => {
            if catalog.has_calendar() {
                next.event = Drawer::New;
                next.screen = Screen::Calendar;
            }
        }
        Event::CloseEvent => next.event = Drawer::Closed,
        Event::SelectQuarantined(id) => {
            if catalog.quarantined().contains(&id.as_str()) {
                next.quarantined = Selection::Id(id);
                next.screen = Screen::Quarantine;
            }
        }
        Event::SelectEdition(id) => pick(&mut next.edition, id, catalog.editions()),
        Event::SelectStep(step) => next.step = step,
        Event::SelectDay(id) => {
            if let Some(day) = Date::parse(&id) {
                next.day = day;
                next.calendar = Mode::Week;
            }
        }
        Event::SelectMonth(id) => {
            if let Some(first) = Date::parse_month(&id) {
                next.day = first;
                next.calendar = Mode::Month;
            }
        }
        Event::SelectCalendarMode(mode) => next.calendar = mode,
        Event::PreviousPeriod => next.day = shift(navigation, -1),
        Event::NextPeriod => next.day = shift(navigation, 1),
        Event::Today => next.day = catalog.today(),
        Event::OpenDraft(id) => pick(&mut next.composer, id, catalog.drafts()),
        Event::CloseComposer => next.composer = Selection::None,
        Event::Disclose(disclosure) => {
            if catalog.discloses(&disclosure) {
                next.open.insert(disclosure);
            }
        }
        Event::Conceal(disclosure) => {
            next.open.remove(&disclosure);
        }
        Event::Refresh => return refresh(navigation, catalog),
    }
    next
}

fn refresh(navigation: &Navigation, catalog: &impl Catalog) -> Navigation {
    if !catalog.signed_in() {
        return open(navigation, Route::SignIn, catalog);
    }
    let gone = |selection: &Selection, ids: Vec<&str>| match selection {
        Selection::Id(id) if !ids.contains(&id.as_str()) => true,
        Selection::Id(_) | Selection::None => false,
    };
    let mut next = navigation.clone();
    let rows = rows(navigation, catalog);
    if rows.is_empty() || gone(&navigation.thread, rows.clone()) {
        next.thread = first(rows);
    }
    next.event = match &navigation.event {
        Drawer::Event(id) if !catalog.events().contains(&id.as_str()) => Drawer::Closed,
        Drawer::New if !catalog.has_calendar() => Drawer::Closed,
        drawer @ (Drawer::Closed | Drawer::Event(_) | Drawer::New) => drawer.clone(),
    };
    if navigation.quarantined == Selection::None
        || gone(&navigation.quarantined, catalog.quarantined())
    {
        next.quarantined = first(catalog.quarantined());
    }
    if navigation.edition == Selection::None || gone(&navigation.edition, catalog.editions()) {
        next.edition = first(catalog.editions());
    }
    if gone(&navigation.composer, catalog.drafts()) {
        next.composer = Selection::None;
    }
    next.open.retain(|disclosure| catalog.discloses(disclosure));
    match &navigation.screen {
        Screen::Thread { id, .. } if catalog.messages(id).is_none() => {
            return show(&next, navigation.listing.clone(), catalog);
        }
        Screen::Agent { id, .. } if catalog.panes(id).is_none() => next.screen = Screen::Agents,
        _ => {}
    }
    next
}

/// The selected day one period before (-1) or after (1).
fn shift(navigation: &Navigation, by: i32) -> Date {
    let day = navigation.day;
    match navigation.calendar {
        Mode::Month => day.add_months(by),
        Mode::Year => day.add_months(12 * by),
        Mode::Week => day.add_days(7 * i64::from(by)),
    }
}

fn open(navigation: &Navigation, route: Route, catalog: &impl Catalog) -> Navigation {
    let mut next = navigation.clone();
    let route = if catalog.signed_in() {
        route
    } else {
        Route::SignIn
    };
    next.screen = match route {
        Route::SignIn if catalog.signed_in() => return show(navigation, Listing::ALL, catalog),
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
        Route::Agents => Screen::Agents,
        Route::Agent(id, pane) => match catalog.panes(&id) {
            None => Screen::Agents,
            Some(panes) => {
                let pane = match pane {
                    PaneChoice::Default => PaneChoice::Default,
                    PaneChoice::Other(pane) => choose(panes, pane).unwrap_or(PaneChoice::Default),
                };
                Screen::Agent { id, pane }
            }
        },
        Route::AgentBuilder => {
            next.step = Step::Instructions;
            Screen::AgentBuilder
        }
        Route::Approvals => Screen::Approvals,
        Route::Settings(section) => {
            next.section = section;
            Screen::Settings
        }
        Route::Calendar(at) => {
            if let Some((mode, day)) = at {
                next.calendar = mode;
                next.day = day;
            }
            Screen::Calendar
        }
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

/// Moves the selection `by` rows in `ids`, from the first when none is selected; past either end it
/// stays.
fn adjacent(selection: &mut Selection, ids: Vec<&str>, by: isize) {
    let at = match selection {
        Selection::Id(id) => ids.iter().position(|i| i == id),
        Selection::None => None,
    };
    let to = match at {
        Some(at) => at
            .checked_add_signed(by)
            .filter(|to| *to < ids.len())
            .unwrap_or(at),
        None => 0,
    };
    if let Some(id) = ids.get(to) {
        *selection = Selection::Id((*id).into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two accounts; account "a" has threads 1 and 2 in For you and 3 in Updates, "b" has 4 in For
    /// you. Folder "f" holds 2 and 4. Thread 1 has messages m1 and m2 and opens with m2 expanded.
    struct Fixture {
        signed_in: bool,
    }

    const FIXTURE: Fixture = Fixture { signed_in: true };

    impl Catalog for Fixture {
        fn signed_in(&self) -> bool {
            self.signed_in
        }
        fn drafts(&self) -> Vec<&str> {
            vec!["d1"]
        }
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
                // A search finds thread 4 in any listing that exists.
                Listing::Search { scope, .. } => {
                    self.listing(scope)?;
                    vec!["4"]
                }
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
        /// "scribe" has no chat, "assistant" no results, "mine" every pane.
        fn panes(&self, agent: &str) -> Option<AgentPanes> {
            use Pane::{Activity, Chat, Results, Settings};
            Some(match agent {
                "scribe" => AgentPanes {
                    panes: &[Results, Activity, Settings],
                    default: Results,
                },
                "assistant" => AgentPanes {
                    panes: &[Activity, Chat, Settings],
                    default: Chat,
                },
                "mine" => AgentPanes {
                    panes: &[Results, Activity, Chat, Settings],
                    default: Results,
                },
                _ => return None,
            })
        }
        fn events(&self) -> Vec<&str> {
            vec!["e1", "e2"]
        }
        fn day_of(&self, event: &str) -> Option<Date> {
            match event {
                "e1" => Some(date("2026-11-09")),
                "e2" => Some(date("2026-11-10")),
                _ => None,
            }
        }
        fn quarantined(&self) -> Vec<&str> {
            vec!["q1", "q2"]
        }
        fn editions(&self) -> Vec<&str> {
            vec!["morning", "evening"]
        }
        fn has_calendar(&self) -> bool {
            true
        }
        /// Run "r1" and invitation "e2" exist; no key is set.
        fn discloses(&self, disclosure: &Disclosure) -> bool {
            match disclosure {
                Disclosure::NewFolder | Disclosure::NewRule | Disclosure::Budget => true,
                Disclosure::Key(_) => false,
                Disclosure::Invitation(id) => id == "e2",
                Disclosure::Run(id) => id == "r1",
            }
        }
        fn today(&self) -> Date {
            date("2026-11-11")
        }
    }

    fn id(id: &str) -> Selection {
        Selection::Id(id.into())
    }

    fn date(text: &str) -> Date {
        Date::parse(text).expect(text)
    }

    fn agent(id: &str, pane: PaneChoice) -> Screen {
        Screen::Agent {
            id: id.into(),
            pane,
        }
    }

    fn account(id: &str, category: InboxCategory) -> Listing {
        Listing::Mailbox(Mailbox::Account(id.into()), category)
    }

    fn run(from: &Navigation, events: impl IntoIterator<Item = Event>) -> Navigation {
        events
            .into_iter()
            .fold(from.clone(), |nav, event| transition(&nav, event, &FIXTURE))
    }

    fn inbox() -> Navigation {
        run(&start(&FIXTURE), [Event::Open(Route::Inbox(Listing::ALL))])
    }

    #[test]
    fn starts_on_sign_in_with_first_items_selected() {
        let nav = start(&FIXTURE);
        assert_eq!(nav.screen, Screen::SignIn);
        assert_eq!(nav.listing, Listing::ALL);
        assert_eq!(nav.thread, id("1"));
        assert_eq!(nav.event, Drawer::Closed);
        assert_eq!(nav.quarantined, id("q1"));
        assert_eq!(nav.edition, id("morning"));
        assert_eq!(nav.step, Step::Instructions);
        assert_eq!((nav.calendar, nav.day), (Mode::Week, date("2026-11-11")));
    }

    #[test]
    fn builder_steps() {
        let nav = run(
            &inbox(),
            [
                Event::Open(Route::AgentBuilder),
                Event::SelectStep(Step::Summary),
            ],
        );
        assert_eq!(nav.step, Step::Summary);
        let nav = run(&nav, [Event::Back, Event::Open(Route::AgentBuilder)]);
        assert_eq!(
            nav.step,
            Step::Instructions,
            "the builder opens on its first step"
        );
    }

    #[test]
    fn opening_each_screen() {
        let from = inbox();
        let cases = [
            // Signed in, the sign-in path shows the inbox.
            (Route::SignIn, Screen::Inbox),
            (Route::Connect, Screen::Connect(ConnectStep::Picking)),
            (Route::Agents, Screen::Agents),
            (Route::AgentBuilder, Screen::AgentBuilder),
            (Route::Approvals, Screen::Approvals),
            (Route::Calendar(None), Screen::Calendar),
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
        let nav = run(&from, [Event::Open(Route::Settings(Section::Rules))]);
        assert_eq!(
            (nav.screen, nav.section),
            (Screen::Settings, Section::Rules)
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
            (
                &id("2"),
                &Drawer::Event("e2".into()),
                &id("q2"),
                &id("evening")
            )
        );
        for event in [
            Event::SelectThread("3".into()),
            Event::SelectEvent("nope".into()),
            Event::SelectAgent("nope".into()),
            Event::SelectQuarantined("nope".into()),
            Event::SelectEdition("nope".into()),
        ] {
            assert_eq!(run(&from, [event.clone()]), from, "{event:?}");
        }
    }

    #[test]
    fn selecting_an_agent_opens_its_page_on_its_default_pane() {
        let nav = run(&inbox(), [Event::SelectAgent("scribe".into())]);
        assert_eq!(nav.screen, agent("scribe", PaneChoice::Default));
        assert_eq!(
            route(&nav),
            Route::Agent("scribe".into(), PaneChoice::Default)
        );
        let from = inbox();
        assert_eq!(run(&from, [Event::SelectAgent("nope".into())]), from);
    }

    #[test]
    fn agent_panes() {
        let nav = run(
            &inbox(),
            [
                Event::SelectAgent("mine".into()),
                Event::SelectDetailPane(Pane::Chat),
            ],
        );
        assert_eq!(nav.screen, agent("mine", PaneChoice::Other(Pane::Chat)));
        let nav = run(&nav, [Event::SelectDetailPane(Pane::Results)]);
        assert_eq!(
            nav.screen,
            agent("mine", PaneChoice::Default),
            "the default pane is not named"
        );
        let scribe = run(&nav, [Event::SelectAgent("scribe".into())]);
        assert_eq!(
            run(&scribe, [Event::SelectDetailPane(Pane::Chat)]),
            scribe,
            "an agent without chat has no chat pane"
        );
        let assistant = run(&nav, [Event::SelectAgent("assistant".into())]);
        assert_eq!(
            run(&assistant, [Event::SelectDetailPane(Pane::Results)]),
            assistant,
            "Assistant has no results pane"
        );
        assert_eq!(
            run(&inbox(), [Event::SelectDetailPane(Pane::Chat)]),
            inbox(),
            "no agent page is shown"
        );
    }

    #[test]
    fn agent_paths() {
        let from = inbox();
        let nav = run(&from, [Event::OpenPath("/u/0/agents/mine/settings".into())]);
        assert_eq!(nav.screen, agent("mine", PaneChoice::Other(Pane::Settings)));
        let nav = run(
            &from,
            [Event::OpenPath("/u/0/agents/assistant/chat".into())],
        );
        assert_eq!(
            route(&nav),
            Route::Agent("assistant".into(), PaneChoice::Default),
            "a default pane named in the path is its plain path"
        );
        let nav = run(&from, [Event::OpenPath("/u/0/agents/scribe/chat".into())]);
        assert_eq!(nav.screen, agent("scribe", PaneChoice::Default));
        let nav = run(&from, [Event::OpenPath("/u/0/agents/nope".into())]);
        assert_eq!(nav.screen, Screen::Agents);
    }

    #[test]
    fn calendar_event_selects_and_closes() {
        let nav = run(&inbox(), [Event::SelectEvent("e1".into())]);
        assert_eq!(
            (&nav.screen, &nav.event, nav.day),
            (
                &Screen::Calendar,
                &Drawer::Event("e1".into()),
                date("2026-11-09")
            ),
            "selecting an event opens the calendar on its day"
        );
        assert_eq!(run(&nav, [Event::CloseEvent]).event, Drawer::Closed);
        let new = run(&inbox(), [Event::NewEvent]);
        assert_eq!((&new.screen, &new.event), (&Screen::Calendar, &Drawer::New));
        assert_eq!(
            run(&new, [Event::SelectEvent("e2".into())]).event,
            Drawer::Event("e2".into()),
            "selecting another event drops the unsaved one"
        );
        assert_eq!(run(&new, [Event::CloseEvent]).event, Drawer::Closed);
    }

    #[test]
    fn calendar_periods() {
        let nav = run(&inbox(), [Event::Open(Route::Calendar(None))]);
        let days = |nav: &Navigation, events: Vec<Event>| {
            events
                .into_iter()
                .scan(nav.clone(), |nav, event| {
                    *nav = transition(nav, event, &FIXTURE);
                    Some(nav.day.id())
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            days(
                &nav,
                vec![Event::NextPeriod, Event::NextPeriod, Event::PreviousPeriod]
            ),
            ["2026-11-18", "2026-11-25", "2026-11-18"],
            "a week at a time"
        );
        let month = run(&nav, [Event::SelectCalendarMode(Mode::Month)]);
        assert_eq!(
            days(
                &month,
                vec![Event::NextPeriod, Event::NextPeriod, Event::Today]
            ),
            ["2026-12-11", "2027-01-11", "2026-11-11"],
            "a month at a time, and back to today"
        );
        let year = run(&nav, [Event::SelectCalendarMode(Mode::Year)]);
        assert_eq!(days(&year, vec![Event::PreviousPeriod]), ["2025-11-11"]);
        let opened = run(&year, [Event::SelectMonth("2027-02".into())]);
        assert_eq!(
            (opened.calendar, opened.day),
            (Mode::Month, date("2027-02-01")),
            "a month of the year opens on its first day"
        );
        let week = run(&opened, [Event::SelectDay("2027-02-17".into())]);
        assert_eq!(
            (week.calendar, week.day),
            (Mode::Week, date("2027-02-17")),
            "a day of the month opens its week"
        );
        for bad in [
            Event::SelectDay("2027-02-30".into()),
            Event::SelectMonth("2027-13".into()),
        ] {
            assert_eq!(run(&week, [bad.clone()]), week, "{bad:?}");
        }
        let back = run(
            &week,
            [
                Event::Open(Route::Inbox(Listing::ALL)),
                Event::Open(Route::Calendar(None)),
            ],
        );
        assert_eq!(
            route(&back),
            Route::Calendar(Some((Mode::Week, date("2027-02-17")))),
            "the calendar opens as it was left"
        );
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
            (Route::Approvals, Screen::Agents, id("2")),
            (
                Route::Agent("mine".into(), PaneChoice::Default),
                Screen::Agents,
                id("2"),
            ),
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
            Route::Settings(Section::Models),
            Route::Calendar(None),
            Route::Briefing,
        ] {
            let nav = run(&from, [Event::Open(route.clone())]);
            assert_eq!(run(&nav, [Event::Back]), nav, "{route:?}");
        }
    }

    #[test]
    fn connect_steps() {
        let picking = run(&start(&FIXTURE), [Event::Open(Route::Connect)]);
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
        let from = start(&FIXTURE);
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

    #[test]
    fn signed_out_every_route_shows_sign_in() {
        let signed_out = Fixture { signed_in: false };
        let nav = transition(
            &start(&signed_out),
            Event::Open(Route::Calendar(None)),
            &signed_out,
        );
        assert_eq!(nav.screen, Screen::SignIn);
        let nav = transition(&inbox(), Event::Refresh, &signed_out);
        assert_eq!(nav.screen, Screen::SignIn, "signing out closes the screen");
    }

    #[test]
    fn the_composer_opens_only_saved_drafts() {
        let nav = run(&inbox(), [Event::OpenDraft("nope".into())]);
        assert_eq!(nav.composer, Selection::None);
        let nav = run(&nav, [Event::OpenDraft("d1".into())]);
        assert_eq!(nav.composer, id("d1"));
        assert_eq!(
            nav.screen,
            Screen::Inbox,
            "the composer opens over the screen"
        );
        let nav = run(&nav, [Event::CloseComposer]);
        assert_eq!(nav.composer, Selection::None);
    }

    #[test]
    fn refresh_moves_on_from_what_is_gone() {
        let mut nav = inbox();
        nav.thread = id("9");
        nav.event = Drawer::Event("e9".into());
        nav.open = [Disclosure::Run("gone".into()), Disclosure::Run("r1".into())].into();
        nav.quarantined = id("q9");
        nav.composer = id("d9");
        nav.screen = agent("gone", PaneChoice::Default);
        let nav = run(&nav, [Event::Refresh]);
        assert_eq!(
            (
                nav.thread,
                nav.event,
                nav.quarantined,
                nav.composer,
                nav.screen
            ),
            (
                id("1"),
                Drawer::Closed,
                id("q1"),
                Selection::None,
                Screen::Agents
            )
        );
        assert_eq!(nav.open, [Disclosure::Run("r1".into())].into());
    }

    #[test]
    fn search_keeps_its_scope_and_clearing_restores_it() {
        let folder = run(
            &inbox(),
            [Event::Open(Route::Inbox(Listing::Folder("f".into())))],
        );
        let searching = run(&folder, [Event::Search("exam".into())]);
        assert_eq!(
            searching.listing,
            Listing::Search {
                query: "exam".into(),
                scope: Box::new(Listing::Folder("f".into())),
            }
        );
        assert_eq!(route(&searching), Route::Inbox(Listing::Folder("f".into())));
        let again = run(&searching, [Event::Search("slides".into())]);
        assert_eq!(
            again.listing.scope(),
            &Listing::Folder("f".into()),
            "a new search keeps the scope"
        );
        let cleared = run(&again, [Event::Search(" ".into())]);
        assert_eq!(cleared.listing, Listing::Folder("f".into()));
    }

    #[test]
    fn next_and_previous_move_through_the_screens_list_and_stop_at_its_ends() {
        let inbox = inbox();
        let ids = |nav: &Navigation, events: Vec<Event>| {
            events
                .into_iter()
                .scan(nav.clone(), |nav, event| {
                    *nav = transition(nav, event, &FIXTURE);
                    Some(match (&nav.screen, &nav.thread, &nav.quarantined) {
                        (Screen::Quarantine, _, Selection::Id(id)) | (_, Selection::Id(id), _) => {
                            id.clone()
                        }
                        _ => String::new(),
                    })
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            ids(
                &inbox,
                vec![
                    Event::SelectNext,
                    Event::SelectNext,
                    Event::SelectNext,
                    Event::SelectPrevious
                ]
            ),
            ["2", "4", "4", "2"]
        );
        assert_eq!(ids(&inbox, vec![Event::SelectPrevious]), ["1"]);
        let quarantine = run(&inbox, [Event::Open(Route::Quarantine)]);
        assert_eq!(
            ids(&quarantine, vec![Event::SelectNext, Event::SelectNext]),
            ["q2", "q2"]
        );
        let empty = run(&inbox, [Event::SelectCategory(InboxCategory::Reading)]);
        assert_eq!(run(&empty, [Event::SelectNext]), empty, "an empty list");
        let calendar = run(&inbox, [Event::Open(Route::Calendar(None))]);
        assert_eq!(run(&calendar, [Event::SelectNext]), calendar, "no list");
    }

    #[test]
    fn disclosures_open_only_on_what_is_shown_and_close() {
        let nav = run(
            &inbox(),
            [
                Event::Disclose(Disclosure::NewFolder),
                Event::Disclose(Disclosure::Run("r1".into())),
                Event::Disclose(Disclosure::Run("nope".into())),
                Event::Disclose(Disclosure::Key(ModelProvider::Openai)),
                Event::Disclose(Disclosure::Invitation("e2".into())),
            ],
        );
        assert_eq!(
            nav.open,
            [
                Disclosure::NewFolder,
                Disclosure::Invitation("e2".into()),
                Disclosure::Run("r1".into()),
            ]
            .into()
        );
        let nav = run(&nav, [Event::Conceal(Disclosure::NewFolder)]);
        assert!(!nav.open.contains(&Disclosure::NewFolder));
    }

    #[test]
    fn a_users_move_to_another_path_pushes_and_everything_else_replaces() {
        let inbox = Route::Inbox(Listing::ALL);
        let thread = Route::Thread("1".into());
        assert_eq!(entry(&inbox, &thread, Mover::User), Entry::Push);
        assert_eq!(entry(&inbox, &inbox, Mover::User), Entry::Replace);
        assert_eq!(entry(&inbox, &thread, Mover::Address), Entry::Replace);
        assert_eq!(entry(&thread, &inbox, Mover::Data), Entry::Replace);
    }

    #[test]
    fn a_path_naming_nothing_is_not_found() {
        for path in [
            "/u/0/inbox",
            "/u/0/thread/1",
            "/u/0/agents/mine",
            "/",
            "/sign-in",
        ] {
            assert!(found(path, &FIXTURE), "{path}");
        }
        for path in [
            "/u/0/nope",
            "/u/0/thread/nope",
            "/u/0/agents/nope",
            "/u/0/account/nope",
            "/u/0/folder/nope",
        ] {
            assert!(!found(path, &FIXTURE), "{path}");
        }
        assert!(
            found("/u/0/nope", &Fixture { signed_in: false }),
            "signed out, every path shows sign-in"
        );
    }

    #[test]
    fn builder_steps_have_neighbours_inside_the_four() {
        assert_eq!(step_by(Step::Instructions, -1), None);
        assert_eq!(step_by(Step::Instructions, 1), Some(Step::Triggers));
        assert_eq!(step_by(Step::Summary, -1), Some(Step::Permissions));
        assert_eq!(step_by(Step::Summary, 1), None);
    }
}
