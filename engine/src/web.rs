use js_sys::{Function, Uint8Array};
use wasm_bindgen::prelude::*;

use crate::Engine;

#[wasm_bindgen]
pub struct WebEngine(Engine);

#[wasm_bindgen]
impl WebEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(
        #[wasm_bindgen(js_name = "backendOrigin")] backend_origin: &str,
        #[wasm_bindgen(
            js_name = "onView",
            unchecked_param_type = "(bytes: Uint8Array) => void"
        )]
        on_view: Function,
    ) -> Self {
        let engine = Engine::new(backend_origin, |task| {
            wasm_bindgen_futures::spawn_local(task)
        });
        engine.subscribe(Box::new(move |view| {
            // An exception in the subscriber is the subscriber's to handle; the engine state is unaffected.
            let _ = on_view.call1(&JsValue::NULL, &Uint8Array::from(view.as_slice()));
        }));
        Self(engine)
    }

    pub fn intent(&self, bytes: &[u8]) -> Result<(), JsError> {
        self.0.intent(bytes).map_err(JsError::from)
    }

    /// The bytes of a message body (an HTML document) for the Service Worker to answer
    /// `/body/<request>` with, or `undefined` for an unknown message. `request` is everything
    /// after `/body/`, query included: `m-slides-4?theme=dark`.
    pub fn body(&self, request: &str) -> Option<Vec<u8>> {
        self.0.body(request)
    }

    /// The value of the `Content-Security-Policy` header to send with every body response, which
    /// sandboxes the body since its iframe carries no sandbox attribute on web.
    #[wasm_bindgen(js_name = "bodySandbox")]
    pub fn body_sandbox() -> String {
        crate::BODY_SANDBOX.into()
    }
}
