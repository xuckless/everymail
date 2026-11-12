use crate::proto::ui::v1::{
    self as ui, connection_failed::Reason, connection_view::State as ViewState,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connection {
    Connecting,
    Connected,
    Failed(Failure),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    BackendUnreachable,
    DatabaseUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    DatabaseReady,
    DatabaseUnavailable,
    BackendUnreachable,
    RetryRequested,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    None,
    StartCheck,
}

pub const START: (Connection, Effect) = (Connection::Connecting, Effect::StartCheck);

pub fn transition(connection: Connection, event: Event) -> (Connection, Effect) {
    use Connection::*;
    match (connection, event) {
        (Connecting, Event::DatabaseReady) => (Connected, Effect::None),
        (Connecting, Event::DatabaseUnavailable) => {
            (Failed(Failure::DatabaseUnavailable), Effect::None)
        }
        (Connecting, Event::BackendUnreachable) => {
            (Failed(Failure::BackendUnreachable), Effect::None)
        }
        (Connecting, Event::RetryRequested) => (connection, Effect::None),
        (Connected, _) => (connection, Effect::None),
        (Failed(_), Event::RetryRequested) => (Connecting, Effect::StartCheck),
        (
            Failed(_),
            Event::DatabaseReady | Event::DatabaseUnavailable | Event::BackendUnreachable,
        ) => (connection, Effect::None),
    }
}

pub fn view(connection: Connection) -> ui::ConnectionView {
    let state = match connection {
        Connection::Connecting => ViewState::Connecting(ui::Connecting {}),
        Connection::Connected => ViewState::Connected(ui::Connected {}),
        Connection::Failed(failure) => ViewState::Failed(ui::ConnectionFailed {
            reason: Some(match failure {
                Failure::BackendUnreachable => {
                    Reason::BackendUnreachable(ui::BackendUnreachable {})
                }
                Failure::DatabaseUnavailable => {
                    Reason::DatabaseUnavailable(ui::DatabaseUnavailable {})
                }
            }),
        }),
    };
    ui::ConnectionView { state: Some(state) }
}

#[cfg(test)]
mod tests {
    use super::{Connection::*, Effect, Event, Failure::*, transition};

    #[test]
    fn every_transition() {
        let table = [
            (Connecting, Event::DatabaseReady, Connected, Effect::None),
            (
                Connecting,
                Event::DatabaseUnavailable,
                Failed(DatabaseUnavailable),
                Effect::None,
            ),
            (
                Connecting,
                Event::BackendUnreachable,
                Failed(BackendUnreachable),
                Effect::None,
            ),
            (Connecting, Event::RetryRequested, Connecting, Effect::None),
            (Connected, Event::DatabaseReady, Connected, Effect::None),
            (
                Connected,
                Event::DatabaseUnavailable,
                Connected,
                Effect::None,
            ),
            (
                Connected,
                Event::BackendUnreachable,
                Connected,
                Effect::None,
            ),
            (Connected, Event::RetryRequested, Connected, Effect::None),
        ];
        let failed = [BackendUnreachable, DatabaseUnavailable]
            .into_iter()
            .flat_map(|failure| {
                [
                    (
                        Failed(failure),
                        Event::DatabaseReady,
                        Failed(failure),
                        Effect::None,
                    ),
                    (
                        Failed(failure),
                        Event::DatabaseUnavailable,
                        Failed(failure),
                        Effect::None,
                    ),
                    (
                        Failed(failure),
                        Event::BackendUnreachable,
                        Failed(failure),
                        Effect::None,
                    ),
                    (
                        Failed(failure),
                        Event::RetryRequested,
                        Connecting,
                        Effect::StartCheck,
                    ),
                ]
            });
        for (from, event, to, effect) in table.into_iter().chain(failed) {
            assert_eq!(
                transition(from, event),
                (to, effect),
                "{from:?} on {event:?}"
            );
        }
    }
}
