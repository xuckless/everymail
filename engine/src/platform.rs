//! The target differences above the HTTP client: native hosts run tasks on a multi-threaded
//! runtime, so tasks and sinks must be `Send`; the browser has one thread. Bodies are served by a
//! custom URI scheme on desktop and by the Frontend's Service Worker on web.

use std::{future::Future, pin::Pin};

#[cfg(not(target_arch = "wasm32"))]
pub type Task = Pin<Box<dyn Future<Output = ()> + Send>>;
#[cfg(target_arch = "wasm32")]
pub type Task = Pin<Box<dyn Future<Output = ()>>>;

#[cfg(not(target_arch = "wasm32"))]
pub type Sink = Box<dyn Fn(Vec<u8>) + Send>;
#[cfg(target_arch = "wasm32")]
pub type Sink = Box<dyn Fn(Vec<u8>)>;

/// Runs a task to completion on the host's executor.
pub type Spawn = fn(Task);

/// Where a sandboxed iframe loads a message body from. Desktop: the `body` scheme registered by
/// the Tauri host, which Windows and Android webviews reach as `http://body.localhost`. Web: a
/// same-origin path the Service Worker answers with `WebEngine::body`. What follows the origin is
/// the body request: the message id and the theme.
pub fn body_url(message_id: &str, theme: crate::proto::ui::v1::Theme) -> String {
    #[cfg(target_arch = "wasm32")]
    let origin = "/body/";
    #[cfg(all(
        not(target_arch = "wasm32"),
        any(target_os = "windows", target_os = "android")
    ))]
    let origin = "http://body.localhost/";
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(any(target_os = "windows", target_os = "android"))
    ))]
    let origin = "body://localhost/";
    format!("{origin}{}", crate::body::request(message_id, theme))
}
