mod connection;
mod platform;
#[cfg(target_arch = "wasm32")]
mod web;

mod proto {
    pub mod api {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/api.v1.rs"));
        }
    }
    pub mod ui {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/ui.v1.rs"));
        }
    }
}

use std::sync::{Arc, Mutex};

use connection::{Connection, Effect, Event};
use platform::{Sink, Spawn};
use prost::Message;
use proto::{
    api::v1::{CheckRequest, CheckResponse, check_response::Database},
    ui::v1::{Intent, intent::Kind},
};
use reqwest::{StatusCode, header::CONTENT_TYPE};

#[derive(Clone)]
pub struct Engine(Arc<Shared>);

struct Shared {
    http: reqwest::Client,
    check_url: String,
    spawn: Spawn,
    state: Mutex<State>,
}

struct State {
    connection: Connection,
    subscriber: Option<Sink>,
}

impl State {
    fn emit(&self) {
        if let Some(sink) = &self.subscriber {
            sink(connection::view(self.connection).encode_to_vec());
        }
    }
}

#[derive(Debug)]
pub struct InvalidIntent;

impl std::fmt::Display for InvalidIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("bytes are not a ui.v1.Intent with a kind set")
    }
}

impl std::error::Error for InvalidIntent {}

impl Engine {
    /// Starts in `Connecting` with a Check in flight. Views go to the sink given to `subscribe`.
    #[cfg_attr(
        target_arch = "wasm32",
        expect(
            clippy::arc_with_non_send_sync,
            reason = "the browser runs the engine on one thread"
        )
    )]
    pub fn new(backend: &str, spawn: Spawn) -> Self {
        let (connection, effect) = connection::START;
        let engine = Self(Arc::new(Shared {
            http: reqwest::Client::new(),
            check_url: format!("{backend}/api.v1.HealthService/Check"),
            spawn,
            state: Mutex::new(State {
                connection,
                subscriber: None,
            }),
        }));
        engine.run(effect);
        engine
    }

    /// Replaces the subscriber and sends it the current view.
    pub fn subscribe(&self, sink: Sink) {
        let mut state = self.lock();
        state.subscriber = Some(sink);
        state.emit();
    }

    pub fn intent(&self, bytes: &[u8]) -> Result<(), InvalidIntent> {
        let kind = Intent::decode(bytes)
            .map_err(|_| InvalidIntent)?
            .kind
            .ok_or(InvalidIntent)?;
        match kind {
            Kind::RetryConnection(_) => self.dispatch(Event::RetryRequested),
        }
        Ok(())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.0.state.lock().expect("a view sink panicked")
    }

    /// Emits under the lock so subscribers see views in transition order.
    fn dispatch(&self, event: Event) {
        let effect = {
            let mut state = self.lock();
            let (connection, effect) = connection::transition(state.connection, event);
            if connection != state.connection {
                state.connection = connection;
                state.emit();
            }
            effect
        };
        self.run(effect);
    }

    fn run(&self, effect: Effect) {
        match effect {
            Effect::None => {}
            Effect::StartCheck => {
                let engine = self.clone();
                (self.0.spawn)(Box::pin(async move {
                    let event = engine.check().await;
                    engine.dispatch(event);
                }));
            }
        }
    }

    async fn check(&self) -> Event {
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
            _ => return Event::BackendUnreachable,
        };
        let Ok(body) = response.bytes().await else {
            return Event::BackendUnreachable;
        };
        match CheckResponse::decode(body).map(|response| response.database) {
            Ok(Some(Database::Ready(_))) => Event::DatabaseReady,
            Ok(Some(Database::Unavailable(_))) => Event::DatabaseUnavailable,
            Ok(None) | Err(_) => Event::BackendUnreachable,
        }
    }
}
