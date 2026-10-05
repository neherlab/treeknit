use js_sys::{Error, Function, JSON};
use serde::Serialize;
use serde::de::DeserializeOwned;
use treeknit_io::analysis::{self, AnalysisRequest, Settings, TreeText, ValidationError};
use treeknit_io::display::{ArgView, ConstellationTable, PairView, Scale, Version};
use treeknit_io::figure::FigureOptions;
use treeknit_io::inspect::{self, Overlap, TreeInspection};
use treeknit_io::output::{FileEntry, OutputFile};
use treeknit_io::palette::{self, Palette};
use treeknit_io::progress::Progress;
use treeknit_io::schema::{self, SettingsSchema};
use treeknit_io::summary::Summary;
use treeknit_io::version::AppVersion;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

/// Links the `treeknit_io::progress` object file, and with it the TypeScript declarations of
/// `Progress` and `Phase`: wasm-ld takes an object out of an rlib only when a symbol of it is
/// referenced, and no export has these types in its signature, because `onProgress` receives them.
#[used]
static PROGRESS_DECLARATIONS: fn(treeknit_core::Progress) -> Progress = Progress::from;

#[wasm_bindgen(start)]
pub fn start() {
  console_error_panic_hook::set_once();
}

/// The settings that a request without settings uses: the defaults of the command line.
#[wasm_bindgen(js_name = defaultSettings)]
pub fn default_settings() -> Result<Ts<Settings>, JsError> {
  to_js(&Settings::default())
}

/// Defaults, ranges, applicability, and help of every setting, for `k` trees and `settings`.
#[wasm_bindgen(js_name = settingsSchema)]
pub fn settings_schema(k: usize, settings: &Ts<Settings>) -> Result<Ts<SettingsSchema>, JsError> {
  to_js(&schema::settings_schema(k, &from_js("settings", settings)?))
}

/// Leaf, node, and polytomy counts, branch lengths, warnings, and parse error of one tree.
#[wasm_bindgen(js_name = inspectTree)]
pub fn inspect_tree(label: &str, text: &str) -> Result<Ts<TreeInspection>, JsError> {
  to_js(&inspect::inspect_tree(label, text))
}

/// Leaf overlap of the trees and of each pair, with the pairs that block a run.
#[wasm_bindgen]
#[expect(
  clippy::needless_pass_by_value,
  reason = "wasm-bindgen takes JavaScript arrays only by value"
)]
pub fn overlap(trees: Vec<Ts<TreeText>>) -> Result<Ts<Overlap>, JsError> {
  let trees = trees
    .iter()
    .enumerate()
    .map(|(i, t)| from_js(&format!("trees[{i}]"), t))
    .collect::<Result<Vec<_>, _>>()?;
  to_js(&inspect::overlap(&trees))
}

/// Every problem with the trees and the settings of the request; none when it runs.
#[wasm_bindgen]
pub fn validate(request: &Ts<AnalysisRequest>) -> Result<Vec<Ts<ValidationError>>, JsError> {
  analysis::validate(&from_js("request", request)?)
    .iter()
    .map(to_js)
    .collect()
}

/// The request of a session file (`treeknit_request.json`); throws with the messages when its
/// JSON structure is invalid.
#[wasm_bindgen(js_name = readRequest)]
#[expect(unused_variables, reason = "session files are not read yet")]
pub fn read_request(text: &str) -> Result<Ts<AnalysisRequest>, JsError> {
  Err(not_implemented("readRequest"))
}

/// The session file of a request, `treeknit_request.json`.
#[wasm_bindgen(js_name = requestFile)]
#[expect(unused_variables, reason = "session files are not written yet")]
pub fn request_file(request: &Ts<AnalysisRequest>) -> Result<Ts<OutputFile>, JsError> {
  Err(not_implemented("requestFile"))
}

/// Labels for trees loaded from `fileNames`: the file name without its last extension, with
/// `_2`, `_3`, ... where it collides with `existingLabels` or an earlier new label.
#[wasm_bindgen(js_name = treeLabels)]
#[expect(
  clippy::needless_pass_by_value,
  unused_variables,
  reason = "wasm-bindgen takes JavaScript arrays only by value; the web label policy is not built yet"
)]
pub fn tree_labels(
  #[wasm_bindgen(js_name = fileNames)] file_names: Vec<String>,
  #[wasm_bindgen(js_name = existingLabels)] existing_labels: Vec<String>,
) -> Result<Vec<String>, JsError> {
  Err(not_implemented("treeLabels"))
}

/// The TreeKnit version and the source repository.
#[wasm_bindgen]
pub fn version() -> Result<Ts<AppVersion>, JsError> {
  to_js(&AppVersion::new(env!("TREEKNIT_LONG_VERSION")))
}

/// The drawing colors of the light and the dark theme.
#[wasm_bindgen]
pub fn palette() -> Result<Ts<Palette>, JsError> {
  to_js(&palette::palette())
}

/// One run of TreeKnit and its results, kept for later queries.
#[wasm_bindgen]
pub struct Session;

#[wasm_bindgen]
impl Session {
  /// Validate and run `request`, calling `onProgress` with each `Progress`. Throws an `Error`
  /// named `ValidationError` when the request does not validate, and an `Error` otherwise.
  #[wasm_bindgen]
  #[expect(unused_variables, reason = "the run is not built yet")]
  pub fn run(
    request: &Ts<AnalysisRequest>,
    #[wasm_bindgen(js_name = onProgress, unchecked_param_type = "(progress: Progress) => void")] on_progress: &Function,
  ) -> Result<Session, JsValue> {
    let errors = analysis::validate(&from_js("request", request)?);
    if !errors.is_empty() {
      return Err(validation_error(&errors));
    }
    Err(not_implemented("Session.run").into())
  }

  /// The MCCs of each pair, the ARG outcome, and the diagnostics.
  #[wasm_bindgen]
  pub fn summary(&self) -> Result<Ts<Summary>, JsError> {
    Err(not_implemented("Session.summary"))
  }

  /// Every output file of the run, without its text.
  #[wasm_bindgen]
  pub fn files(&self) -> Result<Vec<Ts<FileEntry>>, JsError> {
    Err(not_implemented("Session.files"))
  }

  /// The text of the listed file at `path`.
  #[wasm_bindgen(js_name = fileText)]
  #[expect(unused_variables, reason = "the output files are not built yet")]
  pub fn file_text(&self, path: &str) -> Result<String, JsError> {
    Err(not_implemented("Session.fileText"))
  }

  /// A ZIP archive of every listed file, under `treeknit_results/`.
  #[wasm_bindgen]
  pub fn zip(&self) -> Result<Vec<u8>, JsError> {
    Err(not_implemented("Session.zip"))
  }

  /// The command that reproduces the file set of the run from the extracted archive.
  #[wasm_bindgen(js_name = commandLine)]
  pub fn command_line(&self) -> Result<String, JsError> {
    Err(not_implemented("Session.commandLine"))
  }

  /// The tanglegram of pair `pair` (pipeline order) in `version`, laid out with `scale`.
  #[wasm_bindgen(js_name = pairView)]
  #[expect(unused_variables, reason = "the pair view is not built yet")]
  pub fn pair_view(&self, pair: usize, version: &Ts<Version>, scale: &Ts<Scale>) -> Result<Ts<PairView>, JsError> {
    Err(not_implemented("Session.pairView"))
  }

  /// The ARG laid out with `scale`; `undefined` for more than two trees or a failed ARG.
  #[wasm_bindgen(js_name = argView)]
  #[expect(unused_variables, reason = "the ARG view is not built yet")]
  pub fn arg_view(&self, scale: &Ts<Scale>) -> Result<Option<Ts<ArgView>>, JsError> {
    Err(not_implemented("Session.argView"))
  }

  /// The MCC of every leaf in every pair.
  #[wasm_bindgen]
  pub fn constellation(&self) -> Result<Ts<ConstellationTable>, JsError> {
    Err(not_implemented("Session.constellation"))
  }

  /// The SVG tanglegram of pair `pair` in `version`.
  #[wasm_bindgen]
  #[expect(unused_variables, reason = "figures are not built yet")]
  pub fn figure(&self, pair: usize, version: &Ts<Version>, options: &Ts<FigureOptions>) -> Result<String, JsError> {
    Err(not_implemented("Session.figure"))
  }

  /// The SVG figure of the ARG.
  #[wasm_bindgen(js_name = argFigure)]
  #[expect(unused_variables, reason = "figures are not built yet")]
  pub fn arg_figure(&self, options: &Ts<FigureOptions>) -> Result<String, JsError> {
    Err(not_implemented("Session.argFigure"))
  }
}

/// A JavaScript `Error` named `ValidationError` whose message joins the messages of `errors`, one
/// per line, so the caller tells an invalid request from an internal failure.
fn validation_error(errors: &[ValidationError]) -> JsValue {
  let message = errors.iter().map(|e| e.message.as_str()).collect::<Vec<_>>().join("\n");
  let error = Error::new(&message);
  error.set_name("ValidationError");
  error.into()
}

fn not_implemented(name: &str) -> JsError {
  JsError::new(&format!("not implemented: {name}"))
}

/// The Rust value of the argument `name`; errors start with `invalid <name>:`.
fn from_js<T: DeserializeOwned + Tsify>(name: &str, value: &Ts<T>) -> Result<T, JsError> {
  // JSON text instead of serde-wasm-bindgen: serde_json errors say where the value is wrong.
  let text = JSON::stringify(&value.js_value())
    .map_err(|e| JsError::new(&format!("invalid {name}: {}", js_message(&e))))?
    .as_string()
    .ok_or_else(|| JsError::new(&format!("invalid {name}: expected a JSON value")))?;
  serde_json::from_str(&text).map_err(|e| JsError::new(&format!("invalid {name}: {e}")))
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
