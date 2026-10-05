pub mod analysis;

use analysis::{Analysis, AnalysisRequest, Settings};
use js_sys::{Error, JSON};
use serde::Serialize;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
  console_error_panic_hook::set_once();
}

/// Run TreeKnit on the trees of the request.
#[wasm_bindgen]
pub fn analyze(request: &Ts<AnalysisRequest>) -> Result<Ts<Analysis>, JsError> {
  // JSON text instead of serde-wasm-bindgen: serde_json errors say where the request is wrong.
  let text = JSON::stringify(&request.js_value())
    .map_err(|e| JsError::new(&format!("invalid request: {}", js_message(&e))))?
    .as_string()
    .ok_or_else(|| JsError::new("invalid request: expected an object"))?;
  let request: AnalysisRequest =
    serde_json::from_str(&text).map_err(|e| JsError::new(&format!("invalid request: {e}")))?;
  let result = analysis::analyze(&request).map_err(|e| JsError::new(&e))?;
  to_js(&result)
}

/// The settings that a request without settings uses: the defaults of the command line.
#[wasm_bindgen(js_name = defaultSettings)]
pub fn default_settings() -> Result<Ts<Settings>, JsError> {
  to_js(&Settings::default())
}

fn to_js<T: Serialize + Tsify>(value: &T) -> Result<Ts<T>, JsError> {
  let text = serde_json::to_string(value).map_err(|e| JsError::new(&e.to_string()))?;
  let js = JSON::parse(&text).map_err(|e| JsError::new(&js_message(&e)))?;
  Ok(Ts::new_unchecked(js))
}

fn js_message(e: &JsValue) -> String {
  e.dyn_ref::<Error>()
    .map_or_else(|| format!("{e:?}"), |e| e.message().into())
}
