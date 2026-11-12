//! Every target difference, so nothing above this module is aware of the target: native hosts run
//! tasks on a multi-threaded runtime, so tasks, sinks, callbacks and what they share must be `Send`;
//! the browser has one thread. The clock, the timer, HTTP and the device's keyboard come from the
//! host. Bodies are served by a custom URI scheme on desktop and by the Frontend's Service Worker
//! on web.

use std::{future::Future, pin::Pin, sync::Arc};

/// What may be shared with the host's threads: everything `Send + Sync` natively, anything in the
/// browser.
#[cfg(not(target_arch = "wasm32"))]
pub trait Threaded: Send + Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send + Sync + ?Sized> Threaded for T {}
#[cfg(target_arch = "wasm32")]
pub trait Threaded {}
#[cfg(target_arch = "wasm32")]
impl<T: ?Sized> Threaded for T {}

/// Shares `value` between the engine's handles and its tasks: across threads natively, where it is
/// `Threaded`, and on the browser's one thread, where it need not be `Send`.
pub fn shared<T: Threaded>(value: T) -> Arc<T> {
    Arc::new(value)
}

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

#[cfg(not(target_arch = "wasm32"))]
pub type Callback = Box<dyn FnOnce() + Send>;
#[cfg(target_arch = "wasm32")]
pub type Callback = Box<dyn FnOnce()>;

/// The device's clock: milliseconds since 1970.
pub fn wall_ms() -> i64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_millis() as i64)
    }
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as i64
    }
}

/// Calls `callback` once, `ms` milliseconds from now.
pub fn after(ms: u64, callback: Callback) {
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(ms));
        callback();
    });
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_name = setTimeout)]
            fn set_timeout(callback: &JsValue, ms: f64) -> JsValue;
        }
        set_timeout(&Closure::once_into_js(callback), ms as f64);
    }
}

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

/// POSTs protobuf bytes and returns the response body of a 200 answer; anything else is `None`.
pub async fn post(url: &str, body: Vec<u8>) -> Option<Vec<u8>> {
    let response = reqwest::Client::new()
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/proto")
        .body(body)
        .send()
        .await
        .ok()
        .filter(|r| r.status() == reqwest::StatusCode::OK)?;
    response.bytes().await.ok().map(|b| b.to_vec())
}

/// The new mail shortcut as this device's keyboard writes it.
pub fn new_mail_shortcut() -> &'static str {
    if apple() { "⌘N" } else { "Ctrl+N" }
}

#[cfg(not(target_arch = "wasm32"))]
fn apple() -> bool {
    cfg!(any(target_os = "macos", target_os = "ios"))
}

/// The browser's platform, from the Worker's navigator: `userAgentData.platform` where it exists
/// ("macOS"), `platform` elsewhere ("MacIntel", "iPhone").
#[cfg(target_arch = "wasm32")]
fn apple() -> bool {
    use js_sys::Reflect;
    use wasm_bindgen::JsValue;
    let get = |of: &JsValue, key: &str| {
        Reflect::get(of, &JsValue::from_str(key))
            .ok()
            .filter(|v| !v.is_undefined() && !v.is_null())
    };
    let Some(navigator) = get(&js_sys::global(), "navigator") else {
        return false;
    };
    let platform = get(&navigator, "userAgentData")
        .and_then(|data| get(&data, "platform"))
        .or_else(|| get(&navigator, "platform"))
        .and_then(|p| p.as_string())
        .unwrap_or_default()
        .to_lowercase();
    ["mac", "iphone", "ipad"]
        .iter()
        .any(|name| platform.contains(name))
}
