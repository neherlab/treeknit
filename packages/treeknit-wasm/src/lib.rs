mod log_capture;

use js_sys::{Error, Function, JSON};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::cell::RefCell;
use treeknit_core::Options;
use treeknit_io::analysis::{self, AnalysisRequest, Settings, TreeText, ValidationError};
use treeknit_io::display::{self, ArgView, ConstellationTable, DrawingRules, PairView, Scale, TreeVersion};
use treeknit_io::figure::FigureOptions;
use treeknit_io::inspect::{self, Overlap, TreeInspection};
use treeknit_io::output::{self, FileEntry, OutputFile, OutputOptions};
use treeknit_io::palette::{self, Palette};
use treeknit_io::progress::Progress;
use treeknit_io::run::{self, RunResult};
use treeknit_io::schema::{self, SettingsSchema};
use treeknit_io::summary::{Diagnostic, Level, Summary};
use treeknit_io::version::AppVersion;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
  console_error_panic_hook::set_once();
  log_capture::install();
}

/// The settings that a request without settings uses: the defaults of the command line.
#[wasm_bindgen(js_name = defaultSettings)]
pub fn default_settings() -> Result<Ts<Settings>, JsError> {
  let _log = log_capture::discard();
  to_js(&Settings::default())
}

/// Defaults, ranges, applicability, and help of every setting, for `k` trees and `settings`.
#[wasm_bindgen(js_name = settingsSchema)]
pub fn settings_schema(k: usize, settings: &Ts<Settings>) -> Result<Ts<SettingsSchema>, JsError> {
  let _log = log_capture::discard();
  to_js(&schema::settings_schema(k, &from_js("settings", settings)?))
}

/// Leaf, node, and polytomy counts, branch lengths, warnings, and parse error of one tree.
#[wasm_bindgen(js_name = inspectTree)]
pub fn inspect_tree(label: &str, text: &str) -> Result<Ts<TreeInspection>, JsError> {
  let _log = log_capture::discard();
  to_js(&inspect::inspect_tree(label, text))
}

/// Leaf overlap of the trees and of each pair, with the pairs that block a run.
#[wasm_bindgen]
#[expect(
  clippy::needless_pass_by_value,
  reason = "wasm-bindgen takes JavaScript arrays only by value"
)]
pub fn overlap(trees: Vec<Ts<TreeText>>) -> Result<Ts<Overlap>, JsError> {
  let _log = log_capture::discard();
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
  let _log = log_capture::discard();
  analysis::validate(&from_js("request", request)?)
    .iter()
    .map(to_js)
    .collect()
}

/// The request of a session file (`treeknit_request.json`); throws with the messages when its
/// JSON structure is invalid.
#[wasm_bindgen(js_name = readRequest)]
pub fn read_request(text: &str) -> Result<Ts<AnalysisRequest>, JsError> {
  let _log = log_capture::discard();
  match analysis::read_request(text) {
    Ok(request) => to_js(&request),
    Err(errors) => Err(JsError::new(&messages(&errors))),
  }
}

/// The session file of a request, `treeknit_request.json`.
#[wasm_bindgen(js_name = requestFile)]
pub fn request_file(request: &Ts<AnalysisRequest>) -> Result<Ts<OutputFile>, JsError> {
  let _log = log_capture::discard();
  to_js(&output::request_file(&from_js("request", request)?))
}

/// Labels for trees loaded from `fileNames`: the file name without its last extension, with
/// `_2`, `_3`, ... where it collides with `existingLabels` or an earlier new label.
#[wasm_bindgen(js_name = treeLabels)]
#[expect(
  clippy::needless_pass_by_value,
  reason = "wasm-bindgen takes JavaScript arrays only by value"
)]
pub fn tree_labels(
  #[wasm_bindgen(js_name = fileNames)] file_names: Vec<String>,
  #[wasm_bindgen(js_name = existingLabels)] existing_labels: Vec<String>,
) -> Vec<String> {
  let _log = log_capture::discard();
  analysis::tree_labels(&file_names, &existing_labels)
}

/// The TreeKnit version and the source repository.
#[wasm_bindgen]
pub fn version() -> Result<Ts<AppVersion>, JsError> {
  let _log = log_capture::discard();
  to_js(&AppVersion::new(env!("TREEKNIT_LONG_VERSION")))
}

/// The drawing colors of the light and the dark theme.
#[wasm_bindgen]
pub fn palette() -> Result<Ts<Palette>, JsError> {
  let _log = log_capture::discard();
  to_js(&palette::palette())
}

/// The thresholds of the drawing rules that depend on the drawn row height.
#[wasm_bindgen(js_name = drawingRules)]
pub fn drawing_rules() -> Result<Ts<DrawingRules>, JsError> {
  let _log = log_capture::discard();
  to_js(&display::DRAWING_RULES)
}

/// One run of TreeKnit and its results, kept for later queries.
#[wasm_bindgen]
pub struct Session {
  /// Everything the run produced; display data and figures read from it.
  run: RunResult,
  /// The options of the run; the display data sorts the trees of a pair as the run did.
  options: Options,
  /// The Warn and Error records of the run, in the order they occurred.
  diagnostics: Vec<Diagnostic>,
  /// The output files with their text, in the order of `files()`: the session file, the files of
  /// the command line, `parameters.json`, and `log.txt`.
  files: Vec<OutputFile>,
}

#[wasm_bindgen]
impl Session {
  /// Validate and run `request`, calling `onProgress` with each `Progress`. Throws an `Error`
  /// named `ValidationError` when the request does not validate, the error that `onProgress`
  /// throws (after the run completes without further progress calls), and an `Error` otherwise.
  #[wasm_bindgen]
  pub fn run(
    request: &Ts<AnalysisRequest>,
    #[wasm_bindgen(js_name = onProgress, unchecked_param_type = "(progress: Progress) => void")] on_progress: &Function,
  ) -> Result<Session, JsValue> {
    let _log = log_capture::discard();
    let request = from_js("request", request)?;
    log::info!("TreeKnit {}", env!("TREEKNIT_LONG_VERSION"));
    let k = request.trees.len();
    // The checks of `analysis::validate`, in its order, keeping the parsed trees and the options.
    let (parsed, opts) = match (
      analysis::parse_trees(&request.trees),
      analysis::options(&request.settings, k, false),
    ) {
      (Ok(p), Ok(o)) => (p, o),
      (parsed, opts) => {
        let mut errors = parsed.err().unwrap_or_default();
        errors.extend(opts.err().unwrap_or_default());
        return Err(validation_error(&errors));
      },
    };
    run::report_overlap(&parsed.trees, &parsed.taxa);
    let seed = request.settings.seed;
    let failure: RefCell<Option<JsValue>> = RefCell::new(None);
    let observe = |p: treeknit_core::Progress| {
      if failure.borrow().is_some() {
        return;
      }
      let called = to_js(&Progress::from(p))
        .map_err(JsValue::from)
        .and_then(|p| on_progress.call1(&JsValue::UNDEFINED, &p.into()));
      if let Err(e) = called {
        failure.replace(Some(e));
      }
    };
    let result = run::run(parsed, &opts, seed, &observe);
    if let Some(e) = failure.into_inner() {
      return Err(e);
    }
    let records = log_capture::take();
    let output_options = OutputOptions {
      extensions: vec![".nwk".to_owned(); k],
      imputed: true,
      auspice: true,
      figures: false,
    };
    let mut files = vec![output::request_file(&request)];
    files.extend(output::output_files(&result, &opts, &output_options));
    files.push(output::parameters_file(&opts, seed));
    files.push(output::log_file(&records));
    Ok(Session {
      run: result,
      options: opts,
      diagnostics: records
        .into_iter()
        .filter(|r| matches!(r.level, Level::Error | Level::Warn))
        .collect(),
      files,
    })
  }

  /// The MCCs of each pair, the ARG outcome, and the diagnostics.
  #[wasm_bindgen]
  pub fn summary(&self) -> Result<Ts<Summary>, JsError> {
    let _log = log_capture::discard();
    to_js(&Summary::new(&self.run, self.diagnostics.clone()))
  }

  /// Every output file of the run, without its text.
  #[wasm_bindgen]
  pub fn files(&self) -> Result<Vec<Ts<FileEntry>>, JsError> {
    let _log = log_capture::discard();
    self
      .files
      .iter()
      .map(|f| {
        to_js(&FileEntry {
          path: f.path.clone(),
          media_type: f.media_type.clone(),
          size: Some(f.text.len()),
          figure: None,
        })
      })
      .collect()
  }

  /// The text of the listed file at `path`.
  #[wasm_bindgen(js_name = fileText)]
  pub fn file_text(&self, path: &str) -> Result<String, JsError> {
    let _log = log_capture::discard();
    self
      .files
      .iter()
      .find(|f| f.path == path)
      .map(|f| f.text.clone())
      .ok_or_else(|| JsError::new(&format!("no file {path}")))
  }

  /// A ZIP archive of every listed file, under `treeknit_results/`.
  #[wasm_bindgen]
  pub fn zip(&self) -> Result<Vec<u8>, JsError> {
    let _log = log_capture::discard();
    output::zip_archive(&self.files).map_err(|e| JsError::new(&e.to_string()))
  }

  /// The command that reproduces the file set of the run from the extracted archive.
  #[wasm_bindgen(js_name = commandLine)]
  pub fn command_line(&self) -> String {
    let _log = log_capture::discard();
    output::command_line()
  }

  /// The tanglegram of pair `pair` (pipeline order) in `version`, laid out with `scale`.
  #[wasm_bindgen(js_name = pairView)]
  pub fn pair_view(&self, pair: usize, version: &Ts<TreeVersion>, scale: &Ts<Scale>) -> Result<Ts<PairView>, JsError> {
    let _log = log_capture::discard();
    let version = from_js("version", version)?;
    let scale = from_js("scale", scale)?;
    let view = display::pair_view(&self.run, &self.options, pair, version, scale)
      .ok_or_else(|| JsError::new(&format!("no pair {pair}: the run has {} pairs", self.run.pairs.len())))?;
    to_js(&view)
  }

  /// The ARG laid out with `scale`; `undefined` for more than two trees or a failed ARG.
  #[wasm_bindgen(js_name = argView)]
  pub fn arg_view(&self, scale: &Ts<Scale>) -> Result<Option<Ts<ArgView>>, JsError> {
    let _log = log_capture::discard();
    let scale = from_js("scale", scale)?;
    display::arg_view(&self.run, scale).as_ref().map(to_js).transpose()
  }

  /// The MCC of every leaf in every pair.
  #[wasm_bindgen]
  pub fn constellation(&self) -> Result<Ts<ConstellationTable>, JsError> {
    let _log = log_capture::discard();
    to_js(&display::constellation(&self.run, &self.options))
  }

  /// The SVG tanglegram of pair `pair` in `version`.
  #[wasm_bindgen]
  #[expect(unused_variables, reason = "figures are not built yet")]
  pub fn figure(&self, pair: usize, version: &Ts<TreeVersion>, options: &Ts<FigureOptions>) -> Result<String, JsError> {
    let _log = log_capture::discard();
    Err(not_implemented("Session.figure"))
  }

  /// The SVG figure of the ARG.
  #[wasm_bindgen(js_name = argFigure)]
  #[expect(unused_variables, reason = "figures are not built yet")]
  pub fn arg_figure(&self, options: &Ts<FigureOptions>) -> Result<String, JsError> {
    let _log = log_capture::discard();
    Err(not_implemented("Session.argFigure"))
  }
}

/// A JavaScript `Error` named `ValidationError` whose message joins the messages of `errors`, one
/// per line, so the caller tells an invalid request from an internal failure.
fn validation_error(errors: &[ValidationError]) -> JsValue {
  let error = Error::new(&messages(errors));
  error.set_name("ValidationError");
  error.into()
}

/// The messages of `errors`, one per line.
fn messages(errors: &[ValidationError]) -> String {
  errors.iter().map(|e| e.message.as_str()).collect::<Vec<_>>().join("\n")
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
