#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use everymail_engine::Engine;
use tauri::{
    State,
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

fn main() {
    let backend =
        std::env::var("EVERYMAIL_BACKEND_URL").unwrap_or_else(|_| "http://127.0.0.1:8787".into());
    let engine = Engine::new(&backend, |task| {
        tauri::async_runtime::spawn(task);
    });
    tauri::Builder::default()
        .manage(engine)
        .invoke_handler(tauri::generate_handler![subscribe, intent])
        .run(tauri::generate_context!())
        .expect("EveryMail failed to start");
}
