#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use everymail_engine::{BODY_SANDBOX, Engine};
use tauri::{
    Manager, State,
    http::{
        HeaderValue, Response, StatusCode,
        header::{CONTENT_SECURITY_POLICY, CONTENT_TYPE},
    },
    ipc::{Channel, InvokeBody, InvokeResponseBody, Request},
};

#[tauri::command]
fn subscribe(engine: State<'_, Engine>, views: Channel) {
    engine.subscribe(Box::new(move |view| {
        // Fails only once the webview is gone; its next `subscribe` replaces this channel.
        let _ = views.send(InvokeResponseBody::Raw(view));
    }));
}

#[tauri::command]
fn intent(engine: State<'_, Engine>, request: Request<'_>) -> Result<(), String> {
    let InvokeBody::Raw(bytes) = request.body() else {
        return Err("intent takes the raw bytes of a ui.v1.Intent".into());
    };
    engine.intent(bytes).map_err(|error| error.to_string())
}

/// `body://localhost/<message id>?theme=<theme>`: the message body as a sandboxed HTML document.
fn body(engine: &Engine, uri: &tauri::http::Uri) -> Response<Vec<u8>> {
    let request = uri
        .path_and_query()
        .map_or("", |p| p.as_str())
        .trim_start_matches('/');
    match engine.body(request) {
        Some(document) => {
            let mut response = Response::new(document);
            let headers = response.headers_mut();
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            headers.insert(
                CONTENT_SECURITY_POLICY,
                HeaderValue::from_static(BODY_SANDBOX),
            );
            response
        }
        None => {
            let mut response = Response::new(Vec::new());
            *response.status_mut() = StatusCode::NOT_FOUND;
            response
        }
    }
}

fn main() {
    let backend =
        std::env::var("EVERYMAIL_BACKEND_URL").unwrap_or_else(|_| "http://127.0.0.1:8787".into());
    let engine = Engine::new(&backend, |task| {
        tauri::async_runtime::spawn(task);
    });
    tauri::Builder::default()
        .manage(engine)
        .register_uri_scheme_protocol("body", |context, request| {
            body(&context.app_handle().state::<Engine>(), request.uri())
        })
        .invoke_handler(tauri::generate_handler![subscribe, intent])
        .run(tauri::generate_context!())
        .expect("EveryMail failed to start");
}
