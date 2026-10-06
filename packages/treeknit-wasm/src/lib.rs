mod log_capture;

use js_sys::{Error, Function, JSON};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::cell::{OnceCell, RefCell};
use treeknit_io::analysis::{self, AnalysisRequest, Settings, TreeText, ValidationError};
use treeknit_io::display::{
  self, ArgView, AuspicePair, AuspiceTrees, ConstellationTable, DrawingRules, PairView, Scale, TreeVersion,
};
use treeknit_io::examples::{self, ExampleInfo};
use treeknit_io::figure::FigureOptions;
use treeknit_io::inspect::{self, Overlap, TreeInspection};
use treeknit_io::launch::{self, LaunchKeyInfo, LaunchParse, LinkEntry, LinkLimits, LinkSource, SettingsPatch};
use treeknit_io::output::{
  self, Archive, AuspiceFiles, FigureDownload, FigureFile, FileEntry, OutputFile, OutputOptions, WebFile,
};
use treeknit_io::palette::{self, Palette};
use treeknit_io::progress::Progress;
use treeknit_io::run::{self, RunResult};
use treeknit_io::schema::{self, SettingsSchema};
use treeknit_io::summary::{Diagnostic, Level, Summary};
use treeknit_io::version::AppVersion;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

/// Set up the module: panics go to the console, and the log of the Rust code is captured for the
/// diagnostics and `log.txt` of each run. When another logger is installed, the module still
/// loads: the failure goes to the console, and each run reports it as a warning.
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

/// The request of a session file (`treeknit_session.json`); throws with the messages when its
/// JSON structure is invalid.
#[wasm_bindgen(js_name = readSession)]
pub fn read_session(text: &str) -> Result<Ts<AnalysisRequest>, JsError> {
  let _log = log_capture::discard();
  match analysis::read_session(text) {
    Ok(request) => to_js(&request),
    Err(errors) => Err(JsError::new(&messages(&errors))),
  }
}

/// The session file of a request, `treeknit_session.json`.
#[wasm_bindgen(js_name = sessionFile)]
pub fn session_file(request: &Ts<AnalysisRequest>) -> Result<Ts<OutputFile>, JsError> {
  let _log = log_capture::discard();
  to_js(&output::session_file(&from_js("request", request)?))
}

/// Labels for trees loaded from `fileNames`: the file name without its last extension, with the
/// characters that a label must not hold replaced by `_`, and `_2`, `_3`, ... where it collides
/// with `existingLabels` or an earlier new label. Every label passes the label check.
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

/// The launch of the key-value pairs `entries` of a link, in their order: its input, settings, and
/// run, every error at its key, and the keys that are neither launch keys nor `viewKeys`, the
/// keys of the display.
#[wasm_bindgen(js_name = parseLaunch)]
#[expect(
  clippy::needless_pass_by_value,
  reason = "wasm-bindgen takes JavaScript arrays only by value"
)]
pub fn parse_launch(
  entries: Vec<Ts<LinkEntry>>,
  #[wasm_bindgen(js_name = viewKeys)] view_keys: Vec<String>,
) -> Result<Ts<LaunchParse>, JsError> {
  let _log = log_capture::discard();
  let entries = entries
    .iter()
    .enumerate()
    .map(|(i, e)| from_js(&format!("entries[{i}]"), e))
    .collect::<Result<Vec<_>, _>>()?;
  to_js(&launch::parse_launch(&LinkEntry::pairs(&entries), &view_keys))
}

/// The settings of `base` with the values of the link's `patch`.
#[wasm_bindgen(js_name = applySettings)]
pub fn apply_settings(base: &Ts<Settings>, patch: &Ts<SettingsPatch>) -> Result<Ts<Settings>, JsError> {
  let _log = log_capture::discard();
  to_js(&launch::apply(&from_js("base", base)?, &from_js("patch", patch)?))
}

/// The text of a downloaded tree or session file: decompressed when gzip-compressed; throws when
/// it is not UTF-8 text or is a web page.
#[wasm_bindgen(js_name = decodeTreeBytes)]
pub fn decode_tree_bytes(bytes: &[u8]) -> Result<String, JsError> {
  let _log = log_capture::discard();
  launch::decode_tree_bytes(bytes).map_err(|e| JsError::new(&e))
}

/// The built-in examples with their trees, in the order of the Examples menu.
#[wasm_bindgen]
pub fn examples() -> Result<Vec<Ts<ExampleInfo>>, JsError> {
  let _log = log_capture::discard();
  examples::example_infos().iter().map(to_js).collect()
}

/// Every key of links besides the keys of the display, with its value and description.
#[wasm_bindgen(js_name = launchKeys)]
pub fn launch_keys() -> Result<Vec<Ts<LaunchKeyInfo>>, JsError> {
  let _log = log_capture::discard();
  launch::launch_keys().iter().map(to_js).collect()
}

/// The limits of downloads and links.
#[wasm_bindgen(js_name = linkLimits)]
pub fn link_limits() -> Result<Ts<LinkLimits>, JsError> {
  let _log = log_capture::discard();
  to_js(&launch::link_limits())
}

/// The key-value pairs of the canonical link of `source`, or `undefined` when a tree has no
/// address or a label cannot be written.
#[wasm_bindgen(js_name = launchPairs)]
pub fn launch_pairs(source: &Ts<LinkSource>) -> Result<Option<Vec<Ts<LinkEntry>>>, JsError> {
  let _log = log_capture::discard();
  let source = from_js("source", source)?;
  launch::launch_pairs(&source.request, &source.addresses, &Settings::default(), source.run)
    .map(|pairs| LinkEntry::of_pairs(pairs).iter().map(to_js).collect())
    .transpose()
}

/// The inline session of `request`: its session file, gzip-compressed and base64-encoded, as a
/// `data:` location for the fragment of a link.
#[wasm_bindgen(js_name = inlineSession)]
pub fn inline_session(request: &Ts<AnalysisRequest>) -> Result<String, JsError> {
  let _log = log_capture::discard();
  Ok(launch::inline_session(&from_js("request", request)?))
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
  /// The Warn and Error records of the run, in the order they occurred.
  diagnostics: Vec<Diagnostic>,
  /// The output files of `output::web_files`, in the order of `files()`.
  files: Vec<SessionFile>,
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
    let unavailable = log_capture::unavailable();
    log::info!("TreeKnit {}", env!("TREEKNIT_LONG_VERSION"));
    let k = request.trees.len();
    let opts = analysis::options(&request.settings, k, false);
    let (parsed, opts) =
      analysis::prepare(&request.trees, &OutputOptions::web(k), opts).map_err(|e| validation_error(&e))?;
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
    let records: Vec<Diagnostic> = unavailable.into_iter().chain(log_capture::take()).collect();
    let files = output::web_files(&request, &result, seed, &records)
      .map_err(|e| JsError::new(&e.to_string()))?
      .into_iter()
      .map(|f| match f {
        WebFile::Text(file) => SessionFile::Text(file),
        WebFile::Figure(file) => SessionFile::Figure {
          file,
          svg: OnceCell::new(),
        },
      })
      .collect();
    Ok(Session {
      run: result,
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

  /// Every output file of the run, without its text. A figure has the size `null` until its
  /// text is first read, and names the figure it holds.
  #[wasm_bindgen]
  pub fn files(&self) -> Result<Vec<Ts<FileEntry>>, JsError> {
    let _log = log_capture::discard();
    self.files.iter().map(|f| to_js(&f.entry())).collect()
  }

  /// The text of the listed file at `path`. A figure is rendered with the default options on
  /// first use and kept.
  #[wasm_bindgen(js_name = fileText)]
  pub fn file_text(&self, path: &str) -> Result<String, JsError> {
    let _log = log_capture::discard();
    let file = self
      .files
      .iter()
      .find(|f| f.path() == path)
      .ok_or_else(|| JsError::new(&format!("no file {path}")))?;
    Ok(file.text(&self.run)?.to_owned())
  }

  /// A ZIP archive of every listed file, under `treeknit_results/`, figures included. A figure
  /// not yet read is rendered into the archive and not kept, so the archive needs the memory of
  /// one figure at a time.
  #[wasm_bindgen]
  pub fn zip(&self) -> Result<Vec<u8>, JsError> {
    let _log = log_capture::discard();
    let archive_error = |e: output::ArchiveError| JsError::new(&e.to_string());
    // The texts known so far bound the archive; a figure not yet read lets the buffer grow.
    let mut archive = Archive::with_capacity(self.files.iter().filter_map(|f| f.entry().size).sum());
    for f in &self.files {
      match f {
        SessionFile::Text(file) => archive.add(&file.path, &file.text),
        SessionFile::Figure { file, svg } => match svg.get() {
          Some(text) => archive.add(&file.path, text),
          None => archive.add(&file.path, &render(file, &self.run)?),
        },
      }
      .map_err(archive_error)?;
    }
    archive.finish().map_err(archive_error)
  }

  /// The command that reproduces the file set of the run from the extracted archive.
  #[wasm_bindgen(js_name = commandLine)]
  pub fn command_line(&self) -> String {
    let _log = log_capture::discard();
    output::command_line()
  }

  /// The tanglegram of pair `pair` (pipeline order) in `version`, laid out with `scale`, or with
  /// `depth` when `scale` is `div` and a tree of the pair has no branch lengths: `PairView.scale`
  /// tells which.
  #[wasm_bindgen(js_name = pairView)]
  pub fn pair_view(&self, pair: usize, version: &Ts<TreeVersion>, scale: &Ts<Scale>) -> Result<Ts<PairView>, JsError> {
    let _log = log_capture::discard();
    let version = from_js("version", version)?;
    let scale = from_js("scale", scale)?;
    let view = display::pair_view(&self.run, pair, version, scale).ok_or_else(|| no_pair(&self.run, pair))?;
    to_js(&view)
  }

  /// The trees of pair `pair` (pipeline order) in `version` as Auspice datasets, with `div` from
  /// `scale`: the trees of `pairView`, colored by MCC.
  #[wasm_bindgen(js_name = auspiceView)]
  pub fn auspice_view(
    &self,
    pair: usize,
    version: &Ts<TreeVersion>,
    scale: &Ts<Scale>,
  ) -> Result<Ts<AuspicePair>, JsError> {
    let _log = log_capture::discard();
    let version = from_js("version", version)?;
    let scale = from_js("scale", scale)?;
    let view = display::auspice_view(&self.run, pair, version, scale).ok_or_else(|| no_pair(&self.run, pair))?;
    to_js(&view)
  }

  /// The downloads of the Auspice view of pair `pair` (pipeline order) in `version` with `scale`
  /// and the shown `trees`: the name of its SVG figure and the datasets of `auspiceView` as JSON
  /// files.
  #[wasm_bindgen(js_name = auspiceFiles)]
  pub fn auspice_files(
    &self,
    pair: usize,
    version: &Ts<TreeVersion>,
    scale: &Ts<Scale>,
    trees: &Ts<AuspiceTrees>,
  ) -> Result<Ts<AuspiceFiles>, JsError> {
    let _log = log_capture::discard();
    let version = from_js("version", version)?;
    let scale = from_js("scale", scale)?;
    let trees = from_js("trees", trees)?;
    let files =
      output::auspice_files(&self.run, pair, version, scale, trees).ok_or_else(|| no_pair(&self.run, pair))?;
    to_js(&files)
  }

  /// The ARG laid out with `scale`, or with `depth` when `scale` is `div` and the ARG has no
  /// branch lengths (neither tree has them): `ArgView.scale` tells which. `undefined` for more
  /// than two trees or a failed ARG.
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
    to_js(&display::constellation(&self.run))
  }

  /// The SVG tanglegram of pair `pair` (pipeline order) in `version` with `options`, and its file
  /// name as a download. It shows the scale of `pairView` for `options.scale`. With the version
  /// and options of the listed figure, the text and the name are those of its file in `files()`;
  /// other figures get a name of their own. Throws an `Error` named `ValidationError` when
  /// `options` are invalid.
  #[wasm_bindgen]
  pub fn figure(
    &self,
    pair: usize,
    version: &Ts<TreeVersion>,
    options: &Ts<FigureOptions>,
  ) -> Result<Ts<FigureDownload>, JsValue> {
    let _log = log_capture::discard();
    let version = from_js("version", version)?;
    let options = from_js("options", options)?;
    let figure = output::pair_figure(&self.run, pair, version, &options)
      .map_err(|e| validation_error(&e))?
      .ok_or_else(|| no_pair(&self.run, pair))?;
    Ok(to_js(&figure)?)
  }

  /// The SVG figure of the ARG with `options`, and its file name as a download, as `figure`
  /// gives them. It shows the scale of `argView`: with the scale `div`, an ARG without branch
  /// lengths (neither tree has them) is drawn as a cladogram. Throws an `Error` named
  /// `ValidationError` when `options` are invalid, and an `Error` for more than two trees or a
  /// failed ARG.
  #[wasm_bindgen(js_name = argFigure)]
  pub fn arg_figure(&self, options: &Ts<FigureOptions>) -> Result<Ts<FigureDownload>, JsValue> {
    let _log = log_capture::discard();
    let options = from_js("options", options)?;
    let figure = output::arg_figure(&self.run, &options)
      .map_err(|e| validation_error(&e))?
      .ok_or_else(|| JsError::new("the run has no ARG: it needs two trees and a built ARG"))?;
    Ok(to_js(&figure)?)
  }
}

/// The error for a pair index that `run` does not have.
fn no_pair(run: &RunResult, pair: usize) -> JsError {
  let pairs = match run.pairs().len() {
    1 => "1 pair".to_owned(),
    n => format!("{n} pairs"),
  };
  JsError::new(&format!("no pair {pair}: the run has {pairs}"))
}

/// An output file of a session: its text, or a figure rendered on first use.
enum SessionFile {
  Text(OutputFile),
  Figure { file: FigureFile, svg: OnceCell<String> },
}

impl SessionFile {
  fn path(&self) -> &str {
    match self {
      SessionFile::Text(f) => &f.path,
      SessionFile::Figure { file, .. } => &file.path,
    }
  }

  /// The text of the file: a figure of `run` is rendered on first use and kept.
  fn text(&self, run: &RunResult) -> Result<&str, JsError> {
    match self {
      SessionFile::Text(f) => Ok(&f.text),
      SessionFile::Figure { file, svg } => {
        if let Some(text) = svg.get() {
          return Ok(text);
        }
        let text = render(file, run)?;
        Ok(svg.get_or_init(|| text))
      },
    }
  }

  fn entry(&self) -> FileEntry {
    match self {
      SessionFile::Text(f) => FileEntry::new(f.path.clone(), Some(f.text.len()), None),
      SessionFile::Figure { file, svg } => {
        FileEntry::new(file.path.clone(), svg.get().map(String::len), Some(file.figure))
      },
    }
  }
}

/// The SVG text of the figure `file` of `run` with the default options.
fn render(file: &FigureFile, run: &RunResult) -> Result<String, JsError> {
  output::figure_text(run, file.figure).ok_or_else(|| JsError::new(&format!("no figure {}", file.path)))
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
