pub mod analysis;

use js_sys::{Error, JSON};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Takes and returns plain objects; see [`analysis::Request`] and [`analysis::Analysis`].
#[wasm_bindgen]
pub fn analyze(request: &JsValue) -> Result<JsValue, JsError> {
    // JSON text instead of serde-wasm-bindgen: serde_json errors say where the request is wrong.
    let text = JSON::stringify(request)
        .map_err(|e| JsError::new(&format!("invalid request: {}", js_message(&e))))?
        .as_string()
        .ok_or_else(|| JsError::new("invalid request: expected an object"))?;
    let request: analysis::Request =
        serde_json::from_str(&text).map_err(|e| JsError::new(&format!("invalid request: {e}")))?;
    let result = analysis::analyze(&request).map_err(|e| JsError::new(&e))?;
    let text = serde_json::to_string(&result).map_err(|e| JsError::new(&e.to_string()))?;
    JSON::parse(&text).map_err(|e| JsError::new(&js_message(&e)))
}

fn js_message(e: &JsValue) -> String {
    e.dyn_ref::<Error>()
        .map_or_else(|| format!("{e:?}"), |e| e.message().into())
}
