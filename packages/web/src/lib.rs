//! TreeKnit in the browser: [`analysis::analyze`] exported to JavaScript.

pub mod analysis;

use serde::Serialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;

/// Show Rust panics in the browser console.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Run TreeKnit on `{trees: [{label, newick}, ...], settings: {...}}` and return the results
/// as plain JavaScript objects (see [`analysis::Request`] and [`analysis::Analysis`]).
#[wasm_bindgen]
pub fn analyze(request: JsValue) -> Result<JsValue, JsError> {
    let request: analysis::Request =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&format!("invalid request: {e}")))?;
    let result = analysis::analyze(&request).map_err(|e| JsError::new(&e))?;
    // JSON-compatible output: objects instead of `Map`s, numbers instead of `BigInt`s.
    result
        .serialize(&Serializer::json_compatible())
        .map_err(|e| JsError::new(&e.to_string()))
}
