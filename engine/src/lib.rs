mod action;
mod backend;
mod body;
mod builder;
mod connection;
mod date;
mod demo;
mod intent;
mod live;
mod navigation;
mod path;
mod platform;
mod preferences;
mod schedule;
mod store;
mod time;
mod view;
#[cfg(target_arch = "wasm32")]
mod web;
mod world;

mod proto {
    pub mod api {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/api.v1.rs"));
        }
    }
    pub mod ui {
        #[allow(
            clippy::large_enum_variant,
            reason = "prost generates oneofs unboxed; a view lives only until it is encoded"
        )]
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/ui.v1.rs"));
        }
    }
}

use std::sync::{Arc, Mutex, MutexGuard, Weak};

use action::{Action, Context, selected};
use backend::{Link, Request, Update, Work};
use connection::Connection;
use intent::Input;
use live::{Field, Live, Target, Undo};
use navigation::{
    ConnectStep, Disclosure, Entry, Listing, Mailbox, Mover, Navigation, Route, Screen,
};
use platform::{Sink, Spawn};
use preferences::Preferences;
use prost::Message;
use proto::{
    api::v1::{CheckRequest, CheckResponse, check_response::Database},
    ui::v1::{Form, InboxCategory, Intent},
};
use store::Store;
use world::{CATEGORIES, SendState, World};

pub use backend::Backend;
pub use body::SANDBOX as BODY_SANDBOX;
pub use demo::Demo;
pub use intent::InvalidIntent;
pub use store::StoreError;

#[derive(Clone)]
pub struct Engine(Arc<Shared>);

struct Shared {
    check_url: String,
    spawn: Spawn,
    backend: Box<dyn Backend>,
    /// The new mail shortcut as this device writes it.
    shortcut: &'static str,
    state: Mutex<State>,
    /// Taken while the state is still locked and called after it is released, so views go out in
    /// order and the sink never runs under the state lock.
    subscriber: Mutex<Option<Sink>>,
}

/// What the address bar shows when a path names nothing.
const NOT_FOUND: &str = "Nothing is at that address";

struct State {
    connection: Connection,
    navigation: Navigation,
    preferences: Preferences,
    store: Store,
    live: Live,
    /// How the address bar takes the path of the next view.
    entry: Entry,
    /// The device time a timer is set for, if one is.
    armed: Option<i64>,
    /// The Backend's now while one input is handled.
    now: i64,
    /// The world as last read in this input: the store's change count and the search it holds.
    read: Option<(u64, Option<String>, Arc<World>)>,
}

/// Why an intent was not carried out.
#[derive(Debug)]
pub enum Failure {
    Invalid(InvalidIntent),
    Store(StoreError),
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Failure::Invalid(error) => error.fmt(f),
            Failure::Store(error) => write!(f, "the local database failed: {error}"),
        }
    }
}

impl std::error::Error for Failure {}

impl From<StoreError> for Failure {
    fn from(error: StoreError) -> Self {
        Failure::Store(error)
    }
}

/// The engine as the Backend reaches it, held weakly so the engine can go.
struct Replica(Weak<Shared>);

impl backend::Replica for Replica {
    fn deliver(&self, update: Update) {
        if let Some(shared) = self.0.upgrade() {
            // Nothing to show for an update the replica could not take; the next sync brings it
            // again.
            let _ = Engine(shared).dispatch(Input::Update(update));
        }
    }

    fn world(&self) -> Option<World> {
        let shared = self.0.upgrade()?;
        let state = shared.state.lock().ok()?;
        state.store.load(shared.backend.now(), None).ok()
    }
}

/// The search the navigation shows, if any.
fn query(navigation: &Navigation) -> Option<&str> {
    match &navigation.listing {
        Listing::Search { query, .. } => Some(query),
        Listing::Mailbox(..) | Listing::Folder(_) => None,
    }
}

/// The thread an event opened or previewed, as the navigation now shows it.
fn opens(event: &navigation::Event, navigation: &Navigation) -> Option<String> {
    use navigation::Event as Nav;
    match (event, &navigation.screen) {
        (Nav::Open(Route::Thread(_)), Screen::Thread { id, .. }) => Some(id.clone()),
        (Nav::SelectThread(_) | Nav::SelectNext | Nav::SelectPrevious, Screen::Inbox) => {
            selected(&navigation.thread).map(String::from)
        }
        _ => None,
    }
}

/// Device milliseconds at a Backend instant.
fn device_ms(at: i64, now: i64, now_ms: i64) -> i64 {
    now_ms + (at - now) * 1000
}

impl Engine {
    /// Opens the local database at `database` (in memory for `None`) and starts on the screen the
    /// user left, or on sign-in. `backend` is the Backend the engine syncs with. Views go to the
    /// sink given to `subscribe`.
    pub fn new(
        backend_url: &str,
        spawn: Spawn,
        database: Option<&str>,
        backend: Box<dyn Backend>,
    ) -> Result<Self, StoreError> {
        let (connection, effect) = connection::START;
        let store = Store::open(database)?;
        let now = backend.now();
        let world = store.load(now, None)?;
        let start = navigation::start(&world);
        let navigation = match (&world.user, store.navigation_path()?) {
            (Some(_), Some(path)) => {
                navigation::transition(&start, navigation::Event::OpenPath(path), &world)
            }
            (Some(_), None) => navigation::transition(
                &start,
                navigation::Event::Open(Route::Inbox(Listing::ALL)),
                &world,
            ),
            (None, _) => start,
        };
        let preferences = store.preferences()?.unwrap_or(preferences::START);
        let engine = Self(platform::shared(Shared {
            check_url: format!("{backend_url}/api.v1.HealthService/Check"),
            spawn,
            backend,
            shortcut: platform::new_mail_shortcut(),
            state: Mutex::new(State {
                connection,
                navigation,
                preferences,
                store,
                live: live::START,
                entry: Entry::Replace,
                armed: None,
                now,
                read: None,
            }),
            subscriber: Mutex::new(None),
        }));
        engine.0.backend.start(engine.link());
        engine.run(effect);
        Ok(engine)
    }

    /// Replaces the subscriber and sends it the current view.
    pub fn subscribe(&self, sink: Sink) {
        let mut state = self.lock();
        state.now = self.0.backend.now();
        state.read = None;
        state.entry = Entry::Replace;
        let mut subscriber = self.subscriber();
        *subscriber = Some(sink);
        // A failed read leaves the subscriber without a first view; the next change sends one.
        let view = self.view(&mut state);
        drop(state);
        if let (Ok(view), Some(sink)) = (view, subscriber.as_ref()) {
            sink(view);
        }
    }

    pub fn intent(&self, bytes: &[u8]) -> Result<(), Failure> {
        let kind = Intent::decode(bytes)
            .ok()
            .and_then(|intent| intent.kind)
            .ok_or(Failure::Invalid(InvalidIntent))?;
        let input = intent::input(kind).map_err(Failure::Invalid)?;
        Ok(self.dispatch(input)?)
    }

    /// A message body as a complete HTML document, or `None` for an unknown message id. The
    /// request is what follows the origin of a `body_url`: `<message id>?theme=<theme>`.
    pub fn body(&self, request: &str) -> Option<Vec<u8>> {
        let (message_id, theme) = body::parse(request);
        let state = self.lock();
        let world = state.store.load(self.0.backend.now(), None).ok()?;
        let text = world.body(message_id)?;
        // Without randomness for the nonce the body is not served.
        Some(body::document(
            &world::paragraphs(text),
            theme,
            &body::nonce()?,
        ))
    }

    fn link(&self) -> Link {
        platform::shared(Replica(Arc::downgrade(&self.0)))
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.0.state.lock().expect("an input panicked")
    }

    fn subscriber(&self) -> MutexGuard<'_, Option<Sink>> {
        self.0.subscriber.lock().expect("a view sink panicked")
    }

    /// The one dispatch: every input is handled under the lock, then the navigation follows the
    /// data and the view is built. The view goes out after the lock is released, in order, and a
    /// connection check the input starts runs after that.
    fn dispatch(&self, input: Input) -> Result<(), StoreError> {
        let mut state = self.lock();
        state.now = self.0.backend.now();
        state.read = None;
        let before = navigation::route(&state.navigation);
        let mover = match &input {
            Input::Navigation(navigation::Event::OpenPath(_)) => Mover::Address,
            Input::Navigation(_) | Input::Action(_) | Input::Preference(_) => Mover::User,
            Input::Connection(_) | Input::Update(_) | Input::Tick => Mover::Data,
        };
        let handled = self.handle(&mut state, input);
        let followed = self.follow(&mut state);
        state.entry = navigation::entry(&before, &navigation::route(&state.navigation), mover);
        let view = self.view(&mut state);
        let subscriber = self.subscriber();
        drop(state);
        let emitted = view.map(|view| {
            if let Some(sink) = subscriber.as_ref() {
                sink(view);
            }
        });
        drop(subscriber);
        let effect = handled.as_ref().map_or(connection::Effect::None, |e| *e);
        self.run(effect);
        handled.and(followed).and(emitted)
    }

    /// The world as the replica holds it now, with this search, read once until the next write.
    fn read(&self, state: &mut State, search: Option<&str>) -> Result<Arc<World>, StoreError> {
        let changes = state.store.changes();
        if let Some((at, searched, world)) = &state.read
            && *at == changes
            && searched.as_deref() == search
        {
            return Ok(world.clone());
        }
        let world = Arc::new(state.store.load(state.now, search)?);
        state.read = Some((changes, search.map(String::from), world.clone()));
        Ok(world)
    }

    fn world(&self, state: &mut State) -> Result<Arc<World>, StoreError> {
        let search = query(&state.navigation).map(String::from);
        self.read(state, search.as_deref())
    }

    fn handle(&self, state: &mut State, input: Input) -> Result<connection::Effect, StoreError> {
        match input {
            Input::Connection(event) => {
                let (connection, effect) = connection::transition(state.connection, event);
                state.connection = connection;
                return Ok(effect);
            }
            Input::Preference(preference) => {
                let p = preferences::transition(state.preferences, preference);
                state.preferences = p;
                state.store.save_preferences(p)?;
            }
            Input::Navigation(event) => {
                let search = match &event {
                    navigation::Event::Search(q) if !q.trim().is_empty() => Some(q.clone()),
                    _ => query(&state.navigation).map(String::from),
                };
                let world = self.read(state, search.as_deref())?;
                if let navigation::Event::OpenPath(path) = &event
                    && !navigation::found(path, &*world)
                {
                    state.live.info(NOT_FOUND.into(), platform::wall_ms());
                }
                state.navigation =
                    navigation::transition(&state.navigation, event.clone(), &*world);
                if let Some(thread) = opens(&event, &state.navigation) {
                    self.opened(state, &world, &thread)?;
                }
            }
            Input::Action(action) => self.act(state, action)?,
            Input::Update(update) => self.take(state, update)?,
            Input::Tick => {
                state.armed = None;
                state.live.expire(platform::wall_ms());
            }
        }
        Ok(connection::Effect::None)
    }

    /// Opening or previewing a thread marks it read, and Scribe summarises it the first time.
    fn opened(&self, state: &mut State, world: &World, thread: &str) -> Result<(), StoreError> {
        let Some(t) = world.thread(thread) else {
            return Ok(());
        };
        if t.unread {
            self.act(state, Action::Unread(thread.into(), false))?;
        }
        if t.summary.is_none()
            && world.ai_on()
            && state.live.summaries.get(thread) != Some(&live::Summary::Reading)
        {
            state
                .live
                .summaries
                .insert(thread.into(), live::Summary::Reading);
            self.request(Work::Summarize(thread.into()));
        }
        Ok(())
    }

    fn act(&self, state: &mut State, action: Action) -> Result<(), StoreError> {
        let world = self.world(state)?;
        let changes = state.store.changes();
        let reported = action.clone();
        let effects = Context {
            store: &state.store,
            world: &world,
            navigation: &state.navigation,
            live: &mut state.live,
            now: state.now,
            now_ms: platform::wall_ms(),
        }
        .act(action)?;
        if state.store.changes() != changes {
            self.0.backend.report(&reported, self.link());
        }
        for work in effects.work {
            self.request(work);
        }
        self.navigate(state, effects.navigate)
    }

    fn navigate(
        &self,
        state: &mut State,
        events: impl IntoIterator<Item = navigation::Event>,
    ) -> Result<(), StoreError> {
        for event in events {
            let world = self.world(state)?;
            state.navigation = navigation::transition(&state.navigation, event, &*world);
        }
        Ok(())
    }

    fn request(&self, work: Work) {
        self.0.backend.request(work, self.link());
    }

    /// An update from the Backend, applied as sync applies it.
    fn take(&self, state: &mut State, update: Update) -> Result<(), StoreError> {
        let now = state.now;
        let now_ms = platform::wall_ms();
        let before = self.world(state)?;
        state.live.apply(&update);
        state.store.apply(&update, now)?;
        let world = self.world(state)?;
        let live = &mut state.live;
        let event = match &update {
            Update::SignedIn { .. } => Some(navigation::Event::Open(Route::Inbox(Listing::ALL))),
            Update::Snapshot(_) => Some(navigation::Event::Open(navigation::route(
                &state.navigation,
            ))),
            Update::AccountAdded(account) => match state.navigation.screen {
                Screen::Connect(ConnectStep::Connecting(provider))
                    if provider == account.provider =>
                {
                    Some(navigation::Event::ConfirmConnect)
                }
                _ => None,
            },
            Update::AccountSynced(account) => {
                let name = world.account(account).map_or("", |a| a.name.as_str());
                match state.navigation.screen {
                    Screen::Connect(ConnectStep::Syncing(_)) => {
                        live.info(format!("{name} is ready"), now_ms);
                        // The first category that holds its mail, For you when none does.
                        let category = CATEGORIES
                            .iter()
                            .map(|(c, ..)| Listing::Mailbox(Mailbox::Account(account.clone()), *c))
                            .find(|l| world.listing(l).is_some_and(|rows| !rows.is_empty()))
                            .unwrap_or(Listing::Mailbox(
                                Mailbox::Account(account.clone()),
                                InboxCategory::ForYou,
                            ));
                        Some(navigation::Event::Open(Route::Inbox(category)))
                    }
                    _ => {
                        live.done(Target::Account(account.clone()), "Synced", now_ms);
                        None
                    }
                }
            }
            Update::ConnectRejected(reason) => {
                live.failed(Target::Form(Form::Imap), reason, now_ms);
                None
            }
            Update::Outbox { draft, state: send } => {
                let to = world
                    .draft(draft)
                    .map_or(String::new(), |d| action::first_name(&d.recipients));
                match send {
                    SendState::Undoable(until) => live.undo(
                        format!("Sending to {to}"),
                        Undo::Send(draft.clone()),
                        device_ms(*until, now, now_ms),
                    ),
                    SendState::Scheduled(at) => live.undo(
                        format!("Sends {}", time::long(*at, now)),
                        Undo::Send(draft.clone()),
                        now_ms + live::UNDO_MS,
                    ),
                    SendState::Sending => {}
                    SendState::Failed(reason) => {
                        live.failed(Target::Draft(draft.clone()), reason, now_ms);
                        live.info(reason.clone(), now_ms);
                    }
                }
                None
            }
            Update::SendCancelled(draft) => {
                if matches!(&live.notice, live::Notice::Undo { undo: Undo::Send(d), .. } if d == draft)
                {
                    live.notice = live::Notice::None;
                }
                Some(navigation::Event::OpenDraft(draft.clone()))
            }
            Update::Sent { draft, .. } => {
                let to = before
                    .draft(draft)
                    .map_or(String::new(), |d| action::first_name(&d.recipients));
                live.info(format!("Sent to {to}"), now_ms);
                None
            }
            Update::Snoozed { thread, until } => {
                if let Some(t) = before.thread(thread) {
                    live.undo(
                        format!("Snoozed until {}", time::long(*until, now)),
                        Undo::Thread {
                            thread: thread.clone(),
                            place: t.place,
                            snoozed_until: t.snoozed_until,
                            folders: t.folders.clone(),
                        },
                        now_ms + live::UNDO_MS,
                    );
                }
                None
            }
            Update::RunStarted {
                agent,
                cause: world::Cause::RunNow,
                ..
            } => {
                live.done(Target::Agent(agent.clone()), "Started", now_ms);
                None
            }
            Update::Refused { target, reason } => {
                live.failed(target.clone(), reason, now_ms);
                None
            }
            Update::Resolved { approval, status } => {
                let text = match (status, before.approval(approval).map(|a| &a.action)) {
                    (world::ApprovalStatus::Declined(reason), _) => reason.as_str(),
                    (_, Some(world::ApprovalAction::Unsubscribe)) => "Unsubscribed",
                    (_, Some(world::ApprovalAction::Send(_)) | None) => "Sent",
                };
                live.done(Target::Approval(approval.clone()), text, now_ms);
                None
            }
            Update::TopicAdded(_) => {
                live.done(Target::Form(Form::NewsTopic), "Added", now_ms);
                live.revisions.revise(Field::NewsTopic);
                None
            }
            Update::Key {
                provider,
                set: true,
                ..
            } => {
                live.done(Target::Key(*provider), "Saved", now_ms);
                live.revisions.revise(Field::Key(*provider));
                Some(navigation::Event::Conceal(Disclosure::Key(*provider)))
            }
            Update::Budget(budget) => {
                if budget.is_some() {
                    live.done(Target::Form(Form::Budget), "Saved", now_ms);
                    live.revisions.revise(Field::Budget);
                }
                Some(navigation::Event::Conceal(Disclosure::Budget))
            }
            Update::Failed {
                request: Request::SignIn,
                ..
            }
            | Update::UserDeleted => Some(navigation::Event::Refresh),
            _ => None,
        };
        self.navigate(state, event)
    }

    /// Brings the navigation in line with the data and records what it shows: editions on screen
    /// are read and the path is saved. Sets the timer for the next outcome or notice to run out.
    fn follow(&self, state: &mut State) -> Result<(), StoreError> {
        self.navigate(state, [navigation::Event::Refresh])?;
        let world = self.world(state)?;
        if state.navigation.screen == Screen::Briefing && world.editions.iter().any(|e| e.unread) {
            state.store.read_editions()?;
        }
        let route = navigation::route(&state.navigation);
        if route != Route::SignIn {
            state.store.save_navigation(&path::path(&route))?;
        }
        let now_ms = platform::wall_ms();
        if let Some(at) = state.live.next_expiry()
            && state.armed.is_none_or(|armed| at < armed)
        {
            state.armed = Some(at);
            let engine = self.clone();
            platform::after(
                u64::try_from(at - now_ms).unwrap_or(0) + 5,
                Box::new(move || {
                    // A failed read leaves the view as it was; the next input sends one.
                    let _ = engine.dispatch(Input::Tick);
                }),
            );
        }
        Ok(())
    }

    /// The view as it stands, encoded: reads only.
    fn view(&self, state: &mut State) -> Result<Vec<u8>, StoreError> {
        let world = self.world(state)?;
        Ok(view::shell(
            &view::Shell {
                navigation: &state.navigation,
                connection: state.connection,
                preferences: state.preferences,
                entry: state.entry,
                shortcut: self.0.shortcut,
            },
            &world,
            &state.live,
            platform::wall_ms(),
        )
        .encode_to_vec())
    }

    fn run(&self, effect: connection::Effect) {
        match effect {
            connection::Effect::None => {}
            connection::Effect::StartCheck => {
                let engine = self.clone();
                (self.0.spawn)(Box::pin(async move {
                    let event = engine.check().await;
                    // A failed read leaves the view as it was; the next input sends one.
                    let _ = engine.dispatch(Input::Connection(event));
                }));
            }
        }
    }

    async fn check(&self) -> connection::Event {
        let Some(body) = platform::post(&self.0.check_url, CheckRequest {}.encode_to_vec()).await
        else {
            return connection::Event::BackendUnreachable;
        };
        match CheckResponse::decode(body.as_slice()).map(|response| response.database) {
            Ok(Some(Database::Ready(_))) => connection::Event::DatabaseReady,
            Ok(Some(Database::Unavailable(_))) => connection::Event::DatabaseUnavailable,
            Ok(None) | Err(_) => connection::Event::BackendUnreachable,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use prost::Message;

    use super::*;
    use crate::{
        backend::Request,
        proto::ui::v1::{self as ui, intent::Kind},
        store::tests::{FIXTURE, NOW},
        world::Identity,
    };

    /// A Backend that only notes what it is asked for, at a fixed now.
    #[derive(Clone, Default)]
    struct Recorder(Arc<Mutex<Vec<Work>>>);

    impl Backend for Recorder {
        fn now(&self) -> i64 {
            NOW
        }
        fn start(&self, _: Link) {}
        fn report(&self, _: &Action, _: Link) {}
        fn request(&self, work: Work, _: Link) {
            self.0.lock().unwrap().push(work);
        }
    }

    impl Recorder {
        fn take(&self) -> Vec<Work> {
            std::mem::take(&mut self.0.lock().unwrap())
        }
    }

    fn send(engine: &Engine, kind: Kind) {
        let bytes = Intent { kind: Some(kind) }.encode_to_vec();
        engine.intent(&bytes).expect("a valid intent");
    }

    fn screen(engine: &Engine) -> Screen {
        engine.lock().navigation.screen.clone()
    }

    #[test]
    fn sign_in_sync_and_leaving_a_thread_go_through_the_one_dispatch() {
        let backend = Recorder::default();
        let engine = Engine::new("http://backend", |_| {}, None, Box::new(backend.clone()))
            .expect("an engine");
        assert_eq!(screen(&engine), Screen::SignIn);
        assert!(
            engine
                .lock()
                .store
                .load(NOW, None)
                .unwrap()
                .accounts
                .is_empty(),
            "a signed-out device holds no data"
        );
        send(
            &engine,
            Kind::ContinueWithProvider(ui::ContinueWithProvider {
                provider: ui::SignInProvider::Microsoft.into(),
            }),
        );
        assert!(matches!(
            backend.take().as_slice(),
            [Work::SignIn(Identity::Microsoft)]
        ));
        assert_eq!(
            engine.lock().live.sign_in,
            live::SignIn::Waiting(Identity::Microsoft)
        );
        send(&engine, Kind::CancelSignIn(ui::CancelSignIn {}));
        assert!(matches!(
            backend.take().as_slice(),
            [Work::Cancel(Request::SignIn)]
        ));
        assert_eq!(engine.lock().live.sign_in, live::SignIn::Idle);

        // The Backend's answers come in as inputs too.
        for update in [
            Update::SignedIn {
                user_name: "Kai".into(),
                provider: Identity::Microsoft,
            },
            Update::Snapshot(FIXTURE.into()),
        ] {
            engine.dispatch(Input::Update(update)).unwrap();
        }
        assert_eq!(screen(&engine), Screen::Inbox);

        send(
            &engine,
            Kind::OpenThread(ui::OpenThread {
                thread_id: "t-lake".into(),
            }),
        );
        assert!(matches!(screen(&engine), Screen::Thread { id, .. } if id == "t-lake"));
        assert!(
            !engine
                .lock()
                .store
                .load(NOW, None)
                .unwrap()
                .thread("t-lake")
                .unwrap()
                .unread,
            "opening marks it read"
        );
        assert!(matches!(
            backend.take().as_slice(),
            [Work::Summarize(t)] if t == "t-lake"
        ));
        send(
            &engine,
            Kind::ArchiveThread(ui::ArchiveThread {
                thread_id: "t-lake".into(),
            }),
        );
        assert_eq!(screen(&engine), Screen::Inbox, "the archived thread closes");

        // A snooze is the Backend's: the thread leaves now, the Backend says until when.
        send(
            &engine,
            Kind::SnoozeThread(ui::SnoozeThread {
                thread_id: "t-boat".into(),
                preset: ui::SnoozePreset::Tomorrow.into(),
            }),
        );
        assert!(matches!(
            backend.take().as_slice(),
            [Work::Snooze { thread, until: world::Snooze::Tomorrow }] if thread == "t-boat"
        ));
        let unspecified = Intent {
            kind: Some(Kind::SnoozeThread(ui::SnoozeThread {
                thread_id: "t-boat".into(),
                preset: ui::SnoozePreset::Unspecified.into(),
            })),
        };
        assert!(engine.intent(&unspecified.encode_to_vec()).is_err());
    }

    /// An engine on the fixture, signed in, on all inboxes.
    fn signed_in() -> Engine {
        let engine = Engine::new(
            "http://backend",
            |_| {},
            None,
            Box::new(Recorder::default()),
        )
        .expect("an engine");
        for update in [
            Update::SignedIn {
                user_name: "Kai".into(),
                provider: Identity::Google,
            },
            Update::Snapshot(FIXTURE.into()),
        ] {
            engine.dispatch(Input::Update(update)).unwrap();
        }
        engine
    }

    fn notice(engine: &Engine) -> live::Notice {
        engine.lock().live.notice.clone()
    }

    #[test]
    fn a_path_naming_nothing_lands_on_the_inbox_with_a_notice() {
        let engine = signed_in();
        send(
            &engine,
            Kind::OpenPath(ui::OpenPath {
                path: "/u/0/inbox".into(),
            }),
        );
        assert_eq!(notice(&engine), live::Notice::None);
        for path in ["/u/0/nope", "/u/0/thread/nope"] {
            send(&engine, Kind::OpenPath(ui::OpenPath { path: path.into() }));
            assert_eq!(screen(&engine), Screen::Inbox, "{path}");
            assert!(
                matches!(notice(&engine), live::Notice::Info { text, .. } if text == NOT_FOUND),
                "{path}"
            );
        }
    }

    #[test]
    fn a_users_navigation_pushes_a_history_entry_and_the_address_bar_replaces() {
        let engine = signed_in();
        let entry = |engine: &Engine| engine.lock().entry;
        send(
            &engine,
            Kind::OpenThread(ui::OpenThread {
                thread_id: "t-boat".into(),
            }),
        );
        assert_eq!(entry(&engine), Entry::Push);
        send(&engine, Kind::DismissNotice(ui::DismissNotice {}));
        assert_eq!(entry(&engine), Entry::Replace, "the path did not change");
        send(
            &engine,
            Kind::OpenPath(ui::OpenPath {
                path: "/u/0/agents".into(),
            }),
        );
        assert_eq!(entry(&engine), Entry::Replace);
        send(&engine, Kind::Back(ui::Back {}));
        assert_eq!(
            entry(&engine),
            Entry::Replace,
            "back on the agents list stays"
        );
        send(
            &engine,
            Kind::OpenSettings(ui::OpenSettings {
                section: ui::SettingsSection::Rules.into(),
            }),
        );
        assert_eq!(entry(&engine), Entry::Push);
    }

    #[test]
    fn the_view_goes_out_after_the_state_lock_is_released() {
        let engine = signed_in();
        let free = Arc::new(Mutex::new(Vec::new()));
        let (seen, probe) = (free.clone(), engine.clone());
        engine.subscribe(Box::new(move |_| {
            seen.lock().unwrap().push(probe.0.state.try_lock().is_ok());
        }));
        send(&engine, Kind::OpenAgents(ui::OpenAgents {}));
        assert_eq!(*free.lock().unwrap(), [true, true]);
    }

    #[test]
    fn a_created_folder_closes_its_field_and_clears_its_text_and_a_refused_one_keeps_both() {
        let engine = signed_in();
        let new_folder = |engine: &Engine| {
            let state = engine.lock();
            (
                state.navigation.open.contains(&Disclosure::NewFolder),
                state.live.revisions.of(&Field::NewFolder),
            )
        };
        send(
            &engine,
            Kind::OpenDisclosure(ui::OpenDisclosure {
                target: Some(ui::DisclosureTarget {
                    target: Some(ui::disclosure_target::Target::Form(
                        ui::Form::NewFolder.into(),
                    )),
                }),
            }),
        );
        assert_eq!(new_folder(&engine), (true, 0));
        send(
            &engine,
            Kind::CreateFolder(ui::CreateFolder {
                name: "bills".into(),
            }),
        );
        assert_eq!(
            new_folder(&engine),
            (true, 0),
            "a folder named Bills exists"
        );
        send(
            &engine,
            Kind::CreateFolder(ui::CreateFolder {
                name: "Travel".into(),
            }),
        );
        let (open, revision) = new_folder(&engine);
        assert!(!open && revision > 0);
    }

    #[test]
    fn moving_the_selection_previews_the_row_and_marks_it_read() {
        let engine = signed_in();
        send(
            &engine,
            Kind::SelectCategory(ui::SelectCategory {
                category: ui::InboxCategory::Updates.into(),
            }),
        );
        let unread = |engine: &Engine| {
            engine
                .lock()
                .store
                .load(NOW, None)
                .unwrap()
                .thread("t-power")
                .unwrap()
                .unread
        };
        assert!(
            unread(&engine),
            "showing a category does not read its first row"
        );
        send(&engine, Kind::SelectNext(ui::SelectNext {}));
        assert!(!unread(&engine));
    }

    #[test]
    fn a_new_event_is_kept_only_when_saved() {
        let engine = signed_in();
        let events = |engine: &Engine| engine.lock().store.load(NOW, None).unwrap().events.len();
        let count = events(&engine);
        send(&engine, Kind::NewEvent(ui::NewEvent {}));
        assert_eq!(engine.lock().navigation.event, navigation::Drawer::New);
        assert_eq!(events(&engine), count, "nothing is saved yet");
        send(&engine, Kind::CloseEvent(ui::CloseEvent {}));
        assert_eq!(engine.lock().navigation.event, navigation::Drawer::Closed);
        send(&engine, Kind::NewEvent(ui::NewEvent {}));
        let save = |title: &str| {
            Kind::SaveEvent(ui::SaveEvent {
                event: Some(ui::save_event::Event::Unsaved(ui::UnsavedEvent {})),
                title: title.into(),
                day_id: "2026-11-12".into(),
                start_minute: 600,
                end_minute: 660,
                calendar_id: "cal".into(),
                guests: vec![],
                place: None,
            })
        };
        send(&engine, save(" "));
        assert_eq!(events(&engine), count, "refused: no title");
        assert_eq!(engine.lock().navigation.event, navigation::Drawer::New);
        send(&engine, save("Lunch"));
        assert_eq!(events(&engine), count + 1);
        assert!(matches!(
            engine.lock().navigation.event,
            navigation::Drawer::Event(_)
        ));
    }
}
