mod body;
mod connection;
mod demo;
mod navigation;
mod path;
mod platform;
mod preferences;
mod view;
#[cfg(target_arch = "wasm32")]
mod web;

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

use std::sync::{Arc, Mutex};

use connection::Connection;
use demo::World;
use navigation::{Listing, Mailbox, Navigation, Route};
use platform::{Sink, Spawn};
use preferences::{Preference, Preferences};
use prost::Message;
use proto::{
    api::v1::{CheckRequest, CheckResponse, check_response::Database},
    ui::v1::{
        AgentDetailPane, AgentsPane, BuilderStep, Density, InboxCategory, InboxScope, Intent,
        MailProvider, SettingsSection, SignInProvider, Theme, inbox_scope::Scope, intent::Kind,
    },
};
use reqwest::{StatusCode, header::CONTENT_TYPE};

pub use body::SANDBOX as BODY_SANDBOX;

#[derive(Clone)]
pub struct Engine(Arc<Shared>);

struct Shared {
    http: reqwest::Client,
    check_url: String,
    spawn: Spawn,
    world: World,
    state: Mutex<State>,
}

struct State {
    connection: Connection,
    navigation: Navigation,
    preferences: Preferences,
    subscriber: Option<Sink>,
}

impl State {
    fn emit(&self, world: &World) {
        if let Some(sink) = &self.subscriber {
            sink(
                view::shell(&self.navigation, self.connection, self.preferences, world)
                    .encode_to_vec(),
            );
        }
    }
}

#[derive(Debug)]
pub struct InvalidIntent;

impl std::fmt::Display for InvalidIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("bytes are not a ui.v1.Intent with its oneofs set and its enums specified")
    }
}

impl std::error::Error for InvalidIntent {}

/// What an intent changes.
enum Input {
    Connection(connection::Event),
    Navigation(navigation::Event),
    Preference(Preference),
}

/// An enum value the Frontend sent: unknown and unspecified values are decode errors.
fn known<E: Copy + Into<i32>>(
    value: Result<E, prost::UnknownEnumValue>,
) -> Result<E, InvalidIntent> {
    match value {
        Ok(value) if value.into() != 0 => Ok(value),
        Ok(_) | Err(_) => Err(InvalidIntent),
    }
}

fn listing(scope: Option<InboxScope>) -> Result<Listing, InvalidIntent> {
    Ok(match scope.and_then(|s| s.scope).ok_or(InvalidIntent)? {
        Scope::All(_) => Listing::ALL,
        Scope::AccountId(id) => Listing::Mailbox(Mailbox::Account(id), InboxCategory::ForYou),
        Scope::FolderId(id) => Listing::Folder(id),
    })
}

/// The one dispatch from the `Intent` oneof to the state each intent changes.
fn input(kind: Kind) -> Result<Input, InvalidIntent> {
    use navigation::Event as Nav;
    let open = |route| Ok(Input::Navigation(Nav::Open(route)));
    let nav = |event| Ok(Input::Navigation(event));
    match kind {
        Kind::RetryConnection(_) => Ok(Input::Connection(connection::Event::RetryRequested)),
        Kind::SetTheme(set) => Ok(Input::Preference(Preference::Theme(known(
            Theme::try_from(set.theme),
        )?))),
        Kind::SetDensity(set) => Ok(Input::Preference(Preference::Density(known(
            Density::try_from(set.density),
        )?))),
        Kind::OpenPath(open_path) => nav(Nav::OpenPath(open_path.path)),
        Kind::Back(_) => nav(Nav::Back),
        // Signing in for real is a later roadmap task; the demo goes straight to connecting accounts.
        Kind::ContinueWithProvider(sign_in) => {
            known(SignInProvider::try_from(sign_in.provider))?;
            open(Route::Connect)
        }
        Kind::OpenConnect(_) => open(Route::Connect),
        Kind::PickMailProvider(pick) => nav(Nav::PickProvider(known(MailProvider::try_from(
            pick.provider,
        ))?)),
        Kind::ConfirmConnect(_) => nav(Nav::ConfirmConnect),
        Kind::OpenInbox(inbox) => open(Route::Inbox(listing(inbox.scope)?)),
        Kind::SelectCategory(select) => nav(Nav::SelectCategory(known(InboxCategory::try_from(
            select.category,
        ))?)),
        Kind::SelectThread(select) => nav(Nav::SelectThread(select.thread_id)),
        Kind::OpenThread(thread) => open(Route::Thread(thread.thread_id)),
        Kind::ExpandMessage(message) => nav(Nav::ExpandMessage(message.message_id)),
        Kind::CollapseMessage(message) => nav(Nav::CollapseMessage(message.message_id)),
        Kind::OpenAgents(_) => open(Route::Agents),
        Kind::SelectAgent(select) => nav(Nav::SelectAgent(select.agent_id)),
        Kind::OpenAgentBuilder(_) => open(Route::AgentBuilder),
        Kind::OpenSettings(settings) => open(Route::Settings(known(SettingsSection::try_from(
            settings.section,
        ))?)),
        Kind::OpenCalendar(_) => open(Route::Calendar),
        Kind::SelectEvent(select) => nav(Nav::SelectEvent(select.event_id)),
        Kind::CloseEvent(_) => nav(Nav::CloseEvent),
        Kind::OpenQuarantine(_) => open(Route::Quarantine),
        Kind::SelectQuarantined(select) => nav(Nav::SelectQuarantined(select.quarantined_id)),
        Kind::OpenBriefing(_) => open(Route::Briefing),
        Kind::SelectEdition(select) => nav(Nav::SelectEdition(select.edition_id)),
        Kind::SelectAgentsPane(select) => {
            nav(Nav::SelectPane(known(AgentsPane::try_from(select.pane))?))
        }
        Kind::SelectBuilderStep(select) => {
            nav(Nav::SelectStep(known(BuilderStep::try_from(select.step))?))
        }
        Kind::SelectDay(select) => nav(Nav::SelectDay(select.day_id)),
        Kind::SelectAgentDetailPane(select) => nav(Nav::SelectDetailPane(known(
            AgentDetailPane::try_from(select.pane),
        )?)),
    }
}

impl Engine {
    /// Starts on the sign-in screen, in `Connecting` with a Check in flight. Views go to the sink
    /// given to `subscribe`.
    #[cfg_attr(
        target_arch = "wasm32",
        expect(
            clippy::arc_with_non_send_sync,
            reason = "the browser runs the engine on one thread"
        )
    )]
    pub fn new(backend: &str, spawn: Spawn) -> Self {
        let (connection, effect) = connection::START;
        let world = demo::world();
        let engine = Self(Arc::new(Shared {
            http: reqwest::Client::new(),
            check_url: format!("{backend}/api.v1.HealthService/Check"),
            spawn,
            state: Mutex::new(State {
                connection,
                navigation: navigation::start(&world),
                preferences: preferences::START,
                subscriber: None,
            }),
            world,
        }));
        engine.run(effect);
        engine
    }

    /// Replaces the subscriber and sends it the current view.
    pub fn subscribe(&self, sink: Sink) {
        let mut state = self.lock();
        state.subscriber = Some(sink);
        state.emit(&self.0.world);
    }

    pub fn intent(&self, bytes: &[u8]) -> Result<(), InvalidIntent> {
        let kind = Intent::decode(bytes)
            .map_err(|_| InvalidIntent)?
            .kind
            .ok_or(InvalidIntent)?;
        let input = input(kind)?;
        let world = &self.0.world;
        match input {
            Input::Connection(event) => self.dispatch(event),
            Input::Navigation(event) => {
                let mut state = self.lock();
                let navigation = navigation::transition(&state.navigation, event, world);
                if navigation != state.navigation {
                    state.navigation = navigation;
                    state.emit(world);
                }
            }
            Input::Preference(preference) => {
                let mut state = self.lock();
                let preferences = preferences::transition(state.preferences, preference);
                if preferences != state.preferences {
                    state.preferences = preferences;
                    state.emit(world);
                }
            }
        }
        Ok(())
    }

    /// A message body as a complete HTML document, or `None` for an unknown message id. The
    /// request is what follows the origin of a `body_url`: `<message id>?theme=<theme>`.
    pub fn body(&self, request: &str) -> Option<Vec<u8>> {
        let (message_id, theme) = body::parse(request);
        let paragraphs = self.0.world.body(message_id)?;
        // Without randomness for the nonce the body is not served.
        Some(body::document(paragraphs, theme, &body::nonce()?))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.0.state.lock().expect("a view sink panicked")
    }

    /// Emits under the lock so subscribers see views in transition order.
    fn dispatch(&self, event: connection::Event) {
        let effect = {
            let mut state = self.lock();
            let (connection, effect) = connection::transition(state.connection, event);
            if connection != state.connection {
                state.connection = connection;
                state.emit(&self.0.world);
            }
            effect
        };
        self.run(effect);
    }

    fn run(&self, effect: connection::Effect) {
        match effect {
            connection::Effect::None => {}
            connection::Effect::StartCheck => {
                let engine = self.clone();
                (self.0.spawn)(Box::pin(async move {
                    let event = engine.check().await;
                    engine.dispatch(event);
                }));
            }
        }
    }

    async fn check(&self) -> connection::Event {
        let sent = self
            .0
            .http
            .post(&self.0.check_url)
            .header(CONTENT_TYPE, "application/proto")
            .body(CheckRequest {}.encode_to_vec())
            .send()
            .await;
        let response = match sent {
            Ok(response) if response.status() == StatusCode::OK => response,
            _ => return connection::Event::BackendUnreachable,
        };
        let Ok(body) = response.bytes().await else {
            return connection::Event::BackendUnreachable;
        };
        match CheckResponse::decode(body).map(|response| response.database) {
            Ok(Some(Database::Ready(_))) => connection::Event::DatabaseReady,
            Ok(Some(Database::Unavailable(_))) => connection::Event::DatabaseUnavailable,
            Ok(None) | Err(_) => connection::Event::BackendUnreachable,
        }
    }
}
