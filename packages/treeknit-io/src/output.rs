//! Output files of a run, as the command line writes them and the web app lists them.

use crate::analysis::{AnalysisRequest, Field, ValidationError};
use crate::display::{self, AuspiceTrees, Scale, TreeVersion};
use crate::figure::{self, FigureOptions};
use crate::run::RunResult;
use crate::summary::Diagnostic;
use crate::wire::wire_name;
use crate::{analysis, arg, auspice, mccs, newick};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::fmt::{self, Write as _};
use std::io::{Cursor, Write};
use std::path::Path;
use treeknit_core::{Options, Tree};
#[cfg(feature = "tsify")]
use tsify::Tsify;
use zip::result::ZipError;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, System, ZipWriter};

/// Default results directory of the command line, and the directory of the files in the ZIP
/// archive.
pub const RESULTS_DIR: &str = "treeknit_results";

/// File name of the session file: the analysis request that the web app saves and the command
/// line runs with `--session`.
pub const SESSION_FILE: &str = "treeknit_session.json";

/// File name of the core options and the seed of a run.
pub const PARAMETERS_FILE: &str = "parameters.json";

/// File name of the log of a run.
pub const LOG_FILE: &str = "log.txt";

const MCCS_JSON: &str = "MCCs.json";
const ARG_NEWICK: &str = "ARG/arg.nwk";
const ARG_NODES: &str = "ARG/nodes.dat";
const ARG_FIGURE: &str = "ARG/arg.svg";

/// A result file with its text, at its path in the results directory of the command line.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct OutputFile {
  /// Path relative to the results directory, with `/` separators, such as `ARG/arg.nwk`.
  pub path: String,
  /// Media type of the text, such as `application/json`.
  pub media_type: String,
  /// The bytes the command line writes, newline rule included.
  pub text: String,
}

/// An entry of the file list of a run, without its text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
  /// Path relative to the results directory, as in `OutputFile`.
  pub path: String,
  /// The last segment of `path`, the name of the file as a download: `arg.nwk` for
  /// `ARG/arg.nwk`.
  pub file_name: String,
  /// Media type of the file, as in `OutputFile`.
  pub media_type: String,
  /// Size in bytes; `null` in TypeScript (`None`) for a figure not rendered yet.
  pub size: Option<usize>,
  /// The figure the file holds; `null` in TypeScript (`None`) for the other files.
  pub figure: Option<Figure>,
}

/// A figure of a run, by what it shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Figure {
  /// The tanglegram of pair `pair`, in pipeline order.
  Pair { pair: usize },
  /// The ARG of two trees.
  Arg,
}

/// A figure of a run at its path in the results directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FigureFile {
  pub path: String,
  pub figure: Figure,
}

/// A file of the file set of the web app: a text, or a figure that is rendered when it is read,
/// because a run of many large trees has figures of many megabytes that most sessions never read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebFile {
  Text(OutputFile),
  Figure(FigureFile),
}

/// Which output files a run writes besides the MCCs and the resolved trees.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputOptions {
  /// File extension of each tree's output files, in the order of the trees, with its dot
  /// (`.nwk`), or empty for none.
  pub extensions: Vec<String>,
  /// Write each tree with the leaves only other trees have, placed by imputation
  /// (`<label>_imputed<ext>`, `--impute`).
  pub imputed: bool,
  /// Write an Auspice JSON file per tree (`auspice_<label>.json`, `--auspice-view`).
  pub auspice: bool,
  /// Write the SVG figures of `figure_files` (`--plot`).
  pub figures: bool,
}

impl OutputOptions {
  /// The output files of the web app for `k` trees: every optional file, and the tree files
  /// with the extension `.nwk`, because the web app has no input file names to take an
  /// extension from.
  pub fn web(k: usize) -> Self {
    OutputOptions {
      extensions: vec![".nwk".to_owned(); k],
      imputed: true,
      auspice: true,
      figures: true,
    }
  }

  /// The command-line flags that select the optional files of these options.
  pub fn flags(&self) -> Vec<&'static str> {
    [
      (self.imputed, "--impute"),
      (self.auspice, "--auspice-view"),
      (self.figures, "--plot"),
    ]
    .into_iter()
    .filter_map(|(on, flag)| on.then_some(flag))
    .collect()
  }

  /// The extension of the files of tree `i`, with its dot, or empty for none.
  fn extension(&self, i: usize) -> &str {
    self.extensions.get(i).map_or("", String::as_str)
  }
}

impl FileEntry {
  /// The entry of the file at `path`, with its `file_name` and the media type of the path.
  pub fn new(path: String, size: Option<usize>, figure: Option<Figure>) -> Self {
    let file_name = path.rsplit_once('/').map_or(path.as_str(), |(_, name)| name).to_owned();
    let media_type = media_type(&path).to_owned();
    FileEntry {
      path,
      file_name,
      media_type,
      size,
      figure,
    }
  }
}

impl OutputFile {
  /// A file at `path` with the `media_type` of the path.
  pub fn new(path: String, text: String) -> Self {
    OutputFile {
      media_type: media_type(&path).to_owned(),
      path,
      text,
    }
  }
}

/// The media type of an output file at `path`, from its extension: JSON, SVG, or plain text.
fn media_type(path: &str) -> &'static str {
  match Path::new(path).extension().and_then(|e| e.to_str()) {
    Some(e) if e.eq_ignore_ascii_case("json") => "application/json",
    Some(e) if e.eq_ignore_ascii_case("svg") => "image/svg+xml",
    _ => "text/plain",
  }
}

/// A file of the output set, by what it holds; `file_kinds` lists them in the order of the set,
/// and `FileKind::path` and `FileKind::text` make their paths and texts, so the paths of a run
/// that has not happened yet and the files of a finished run follow one list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileKind {
  /// `MCCs.json`.
  Mccs,
  /// The MCCs of pair `pair` (pipeline order) of the trees `i` and `j`, one per line.
  MccLines { pair: usize, i: usize, j: usize },
  /// The resolved tree `tree`.
  Resolved { tree: usize },
  /// The imputed tree `tree`.
  Imputed { tree: usize },
  /// The Auspice JSON of tree `tree`.
  Auspice { tree: usize },
  /// `ARG/arg.nwk`.
  ArgNewick,
  /// `ARG/nodes.dat`.
  ArgNodes,
  /// The liberally resolved tree `tree` of the ARG.
  ArgTree { tree: usize },
  /// A figure, tanglegram `i` and `j` of a pair or the ARG.
  Figure { figure: Figure, i: usize, j: usize },
}

/// The files of a run of `k` trees with `options`, in their order; `arg` lists the ARG files.
fn file_kinds(k: usize, options: &OutputOptions, arg: bool) -> Vec<FileKind> {
  let pairs: Vec<(usize, usize, usize)> = (0..k)
    .flat_map(|i| (i + 1..k).map(move |j| (i, j)))
    .enumerate()
    .map(|(pair, (i, j))| (pair, i, j))
    .collect();
  let mut kinds = vec![FileKind::Mccs];
  kinds.extend(pairs.iter().map(|&(pair, i, j)| FileKind::MccLines { pair, i, j }));
  kinds.extend((0..k).map(|tree| FileKind::Resolved { tree }));
  if options.imputed {
    kinds.extend((0..k).map(|tree| FileKind::Imputed { tree }));
  }
  if options.auspice {
    kinds.extend((0..k).map(|tree| FileKind::Auspice { tree }));
  }
  if arg {
    kinds.extend([FileKind::ArgNewick, FileKind::ArgNodes]);
    kinds.extend((0..k).map(|tree| FileKind::ArgTree { tree }));
  }
  if options.figures {
    kinds.extend(pairs.iter().map(|&(pair, i, j)| FileKind::Figure {
      figure: Figure::Pair { pair },
      i,
      j,
    }));
    if arg {
      kinds.push(FileKind::Figure {
        figure: Figure::Arg,
        i: 0,
        j: 1,
      });
    }
  }
  kinds
}

impl FileKind {
  /// The path of the file for the trees labeled `labels`.
  fn path(self, labels: &[&str], options: &OutputOptions) -> String {
    let tree = |dir: &str, suffix: &str, i: usize| tree_path(dir, labels[i], suffix, options.extension(i));
    match self {
      FileKind::Mccs => MCCS_JSON.to_owned(),
      FileKind::MccLines { i, j, .. } => mccs_lines_path(labels.len(), labels[i], labels[j]),
      FileKind::Resolved { tree: i } => tree("", "_resolved", i),
      FileKind::Imputed { tree: i } => tree("", "_imputed", i),
      FileKind::Auspice { tree: i } => auspice_path(labels[i]),
      FileKind::ArgNewick => ARG_NEWICK.to_owned(),
      FileKind::ArgNodes => ARG_NODES.to_owned(),
      FileKind::ArgTree { tree: i } => tree("ARG/", "_liberal_resolved", i),
      FileKind::Figure {
        figure: Figure::Pair { .. },
        i,
        j,
      } => tanglegram_path(labels[i], labels[j]),
      FileKind::Figure {
        figure: Figure::Arg, ..
      } => ARG_FIGURE.to_owned(),
    }
  }

  /// The text of the file of `run`, with the bytes the command line writes; `None` when `run`
  /// lacks the pair or the ARG of the file.
  fn text(self, run: &RunResult) -> Option<String> {
    let RunResult {
      trees,
      taxa,
      pairs,
      imputed,
      ..
    } = run;
    let newick = |t: &Tree| format!("{}\n", newick::write(t));
    Some(match self {
      FileKind::Mccs => format!("{:#}\n", mccs::to_json(pairs, trees, taxa)),
      FileKind::MccLines { pair, .. } => {
        let names: Vec<Vec<String>> = pairs.get(pair)?.mccs.iter().map(|m| taxa.names_of(m)).collect();
        mccs::to_lines(&names)
      },
      FileKind::Resolved { tree } => newick(trees.get(tree)?),
      FileKind::Imputed { tree } => newick(imputed.get(tree)?),
      FileKind::Auspice { tree } => format!("{:#}", auspice::auspice_json(tree, trees, pairs, taxa)),
      FileKind::ArgNewick => format!("{}\n", arg::extended_newick(run.built_arg()?)),
      FileKind::ArgNodes => format!("{}\n", arg::node_table(run.built_arg()?)),
      FileKind::ArgTree { tree } => newick(run.built_arg()?.trees.get(tree)?),
      FileKind::Figure { figure, .. } => figure_text(run, figure)?,
    })
  }
}

/// The labels of the trees of `run`.
fn run_labels(run: &RunResult) -> Vec<&str> {
  run.trees.iter().map(|t| t.label.as_str()).collect()
}

/// The files of `run` with `options`, each at its path with the kind of file it is.
fn run_files(run: &RunResult, options: &OutputOptions) -> Vec<(FileKind, String)> {
  let labels = run_labels(run);
  file_kinds(labels.len(), options, run.built_arg().is_some())
    .into_iter()
    .map(|kind| (kind, kind.path(&labels, options)))
    .collect()
}

/// A listed file of a run whose text cannot be made, because the run lacks its pair or its ARG.
#[derive(Debug, PartialEq, Eq)]
pub struct MissingFile {
  pub path: String,
}

impl fmt::Display for MissingFile {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "the run has no data for the output file {}", self.path)
  }
}

impl std::error::Error for MissingFile {}

/// Every output file of `run` except `parameters.json` and `log.txt`, at its path in the results
/// directory, with the bytes the command line writes: the MCCs as JSON and as lines, the
/// resolved trees, the imputed trees and Auspice files when `options` asks for them, for two
/// trees the ARG files, and the figures when `options` asks for them. File names follow the tree
/// labels. The run of a request that passed `analysis::prepare` has every file it lists, so the
/// error marks a broken run.
pub fn output_files(run: &RunResult, options: &OutputOptions) -> Result<Vec<OutputFile>, MissingFile> {
  run_files(run, options)
    .into_iter()
    .map(|(kind, path)| match kind.text(run) {
      Some(text) => Ok(OutputFile::new(path, text)),
      None => Err(MissingFile { path }),
    })
    .collect()
}

/// The figures of `run`: the tanglegram of each pair, `tanglegram_<a>_<b>.svg`, then for two
/// trees with a built ARG `ARG/arg.svg`.
pub fn figure_files(run: &RunResult) -> Vec<FigureFile> {
  let options = OutputOptions {
    imputed: false,
    auspice: false,
    ..OutputOptions::web(run.trees.len())
  };
  run_files(run, &options)
    .into_iter()
    .filter_map(|(kind, path)| match kind {
      FileKind::Figure { figure, .. } => Some(FigureFile { path, figure }),
      _ => None,
    })
    .collect()
}

/// The file set of a run of the web app for `request`, in order: the session file, the files of
/// `output_files` with `OutputOptions::web`, the figures as `FigureFile`s, `parameters.json`, and
/// `log.txt` of `records`. `command_line` writes the same files. `seed` is the seed of the run.
pub fn web_files(
  request: &AnalysisRequest,
  run: &RunResult,
  seed: u64,
  records: &[Diagnostic],
) -> Result<Vec<WebFile>, MissingFile> {
  let mut files = vec![WebFile::Text(session_file(request))];
  for (kind, path) in run_files(run, &OutputOptions::web(run.trees.len())) {
    files.push(match kind {
      FileKind::Figure { figure, .. } => WebFile::Figure(FigureFile { path, figure }),
      _ => match kind.text(run) {
        Some(text) => WebFile::Text(OutputFile::new(path, text)),
        None => return Err(MissingFile { path }),
      },
    });
  }
  files.push(WebFile::Text(parameters_file(&run.opts, seed)));
  files.push(WebFile::Text(log_file(records)));
  Ok(files)
}

/// Every path that a run of the trees labeled `labels` can write with `options`, in the order of
/// `output_files`, followed by `parameters.json`, `log.txt`, and the session file. For two trees
/// the ARG files are listed, although a run writes them only when the ARG is built.
pub fn output_paths(labels: &[String], options: &OutputOptions) -> Vec<String> {
  let labels: Vec<&str> = labels.iter().map(String::as_str).collect();
  let mut paths: Vec<String> = file_kinds(labels.len(), options, labels.len() == 2)
    .into_iter()
    .map(|kind| kind.path(&labels, options))
    .collect();
  paths.extend([PARAMETERS_FILE, LOG_FILE, SESSION_FILE].map(str::to_owned));
  paths
}

/// Longest file name of the file systems of the release targets, in bytes: 255 bytes in ext4,
/// APFS, and NTFS (where it is 255 UTF-16 units, which UTF-8 bytes bound from above).
pub const MAX_FILE_NAME_BYTES: usize = 255;

/// Output file names that a file system cannot hold: names longer than `MAX_FILE_NAME_BYTES`,
/// such as `tanglegram_<a>_<b>.svg` of two long labels, and files of different trees or kinds
/// that would get the same name, ignoring case as the label check does: the resolved tree of
/// `MCCs_a` from `MCCs_a.dat` and the MCCs of the pair (`a`, `resolved`) are both
/// `MCCs_a_resolved.dat`. The command line would fail after the inference or overwrite one file
/// with the other, and the ZIP archive of the web app cannot hold both. A clash needs a tree
/// extension of the command line, such as `.dat`: the file set of the web app gives each kind of
/// file its own prefix, suffix, extension, or folder. `labels` must have passed the checks of
/// `analysis::parse_trees`, which reports repeated labels and pair names.
pub fn check_output_paths(labels: &[String], options: &OutputOptions) -> Vec<ValidationError> {
  let error = |message: String| ValidationError {
    field: Some(Field::Trees),
    message,
    line: None,
    column: None,
  };
  let paths = output_paths(labels, options);
  let mut errors = Vec::new();
  let too_long: Vec<&str> = paths
    .iter()
    .flat_map(|p| p.split('/'))
    .filter(|name| name.len() > MAX_FILE_NAME_BYTES)
    .collect();
  if let Some(longest) = too_long.iter().max_by_key(|name| name.len()) {
    let (count, bytes) = (too_long.len(), longest.len());
    errors.push(error(if count == 1 {
      format!(
        "the output file name {longest:?} has {bytes} bytes, more than the {MAX_FILE_NAME_BYTES} of a file name; \
         shorten the tree labels"
      )
    } else {
      format!(
        "{count} output file names have more than the {MAX_FILE_NAME_BYTES} bytes of a file name, such as \
         {longest:?} with {bytes} bytes; shorten the tree labels"
      )
    }));
  }
  let mut first: BTreeMap<String, &str> = BTreeMap::new();
  for path in &paths {
    match first.entry(analysis::label_key(path)) {
      Entry::Vacant(e) => {
        e.insert(path);
      },
      Entry::Occupied(e) => errors.push(error(if *e.get() == path {
        format!("two output files are named {path:?}; rename a tree")
      } else {
        format!(
          "the output files {:?} and {path:?} differ only in case; rename a tree",
          e.get()
        )
      })),
    }
  }
  errors
}

/// The SVG text of `figure` of `run` with the default `FigureOptions`, a tanglegram of the
/// resolved trees, as `pair_figure` and `arg_figure` draw it; `None` when `run` lacks the pair or
/// the ARG.
pub fn figure_text(run: &RunResult, figure: Figure) -> Option<String> {
  let options = FigureOptions::default();
  let svg = match figure {
    Figure::Pair { pair } => pair_figure(run, pair, TreeVersion::Resolved, &options),
    Figure::Arg => arg_figure(run, &options),
  };
  #[expect(clippy::expect_used, reason = "the default figure options are valid")]
  svg.expect("default figure options").map(|f| f.text)
}

/// An SVG figure with the name of its file as a download.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct FigureDownload {
  /// The name of the listed figure file (`tanglegram_ha_na.svg`, `arg.svg`) when the figure has
  /// its version and options; otherwise its stem with the version, the shown scale, and every
  /// option that differs from the default, such as `tanglegram_ha_na_imputed_depth_w800.svg`, so
  /// that a figure of other options never takes the name of the listed one.
  pub file_name: String,
  /// The SVG text.
  pub text: String,
}

/// The SVG tanglegram of pair `pair` (pipeline order) of `run` in `version` with `options`;
/// `Ok(None)` when `run` lacks the pair, and the errors of `figure::check_figure_options` when
/// `options` are invalid. The scale is that of the pair view (`PairView.scale`), so the figure
/// shows what the interactive view shows.
pub fn pair_figure(
  run: &RunResult,
  pair: usize,
  version: TreeVersion,
  options: &FigureOptions,
) -> Result<Option<FigureDownload>, Vec<ValidationError>> {
  figure::checked(options)?;
  let Some(view) = display::pair_view(run, pair, version, options.scale) else {
    return Ok(None);
  };
  let p = &run.pairs[pair];
  let path = tanglegram_path(&run.trees[p.i].label, &run.trees[p.j].label);
  Ok(Some(FigureDownload {
    file_name: download_name(&path, Some(version), options, view.scale),
    text: figure::tanglegram_svg(&view, options)?,
  }))
}

/// The SVG figure of the ARG of `run` with `options`; `Ok(None)` for more than two trees or a
/// failed ARG, and the errors of `figure::check_figure_options` when `options` are invalid. The
/// scale is that of the ARG view (`ArgView.scale`).
pub fn arg_figure(run: &RunResult, options: &FigureOptions) -> Result<Option<FigureDownload>, Vec<ValidationError>> {
  figure::checked(options)?;
  let (Some(segments), Some(view)) = (segment_labels(run), display::arg_view(run, options.scale)) else {
    return Ok(None);
  };
  Ok(Some(FigureDownload {
    file_name: download_name(ARG_FIGURE, None, options, view.scale),
    text: figure::arg_svg(&view, segments, options)?,
  }))
}

/// The downloads of the Auspice view: the name of its SVG figure and the datasets of the shown
/// trees.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct AuspiceFiles {
  /// File name of the SVG figure without `.svg`, which Auspice's figure download appends:
  /// `auspice_<a>_<b>` for both trees, with the tree label appended for one tree, and with the
  /// version and the shown scale appended unless they are `resolved` and `div`.
  pub svg_prefix: String,
  /// One Auspice v2 dataset per shown tree, as pretty-printed JSON named `<stem>_<label>.json`.
  /// The names differ from the listed `auspice_<label>.json` files of the command line, which
  /// hold other fields.
  pub json: Vec<OutputFile>,
}

/// The downloads of the Auspice view of pair `pair` (pipeline order) of `run` in `version` with
/// `scale` and the shown `trees`; `None` when `run` lacks the pair. The datasets are those of
/// `display::auspice_view`, and the names tell the shown scale of that view.
pub fn auspice_files(
  run: &RunResult,
  pair: usize,
  version: TreeVersion,
  scale: Scale,
  trees: AuspiceTrees,
) -> Option<AuspiceFiles> {
  let view = display::auspice_view(run, pair, version, scale)?;
  let p = &run.pairs[pair];
  let [a, b] = [&run.trees[p.i].label, &run.trees[p.j].label];
  let pair_stem = analysis::pair_stem(a, b);
  let stem = if version == TreeVersion::Resolved && view.scale == Scale::Div {
    format!("auspice_{pair_stem}")
  } else {
    format!("auspice_{pair_stem}_{}_{}", wire_name(&version), wire_name(&view.scale))
  };
  let json = [(true, a, &view.left), (false, b, &view.right)]
    .into_iter()
    .filter(|&(left, _, _)| trees.shows(left))
    .map(|(_, label, dataset)| {
      #[expect(clippy::expect_used, reason = "an Auspice dataset serializes to JSON")]
      let text = serde_json::to_string_pretty(dataset).expect("a dataset serializes to JSON");
      OutputFile {
        path: format!("{stem}_{label}.json"),
        media_type: "application/json".to_owned(),
        text,
      }
    })
    .collect();
  let svg_prefix = match trees {
    AuspiceTrees::Both => stem,
    AuspiceTrees::Left => format!("{stem}_{a}"),
    AuspiceTrees::Right => format!("{stem}_{b}"),
  };
  Some(AuspiceFiles { svg_prefix, json })
}

/// The download name of a figure whose listed file is at `path`, drawn in `version` (`None` for
/// the ARG) with `options` and the shown `scale`; see `FigureDownload.file_name`.
#[expect(
  clippy::float_cmp,
  reason = "any other width or row height gives another figure, so the name must tell it"
)]
fn download_name(path: &str, version: Option<TreeVersion>, options: &FigureOptions, scale: Scale) -> String {
  let listed = FigureOptions::default();
  let name = path.rsplit_once('/').map_or(path, |(_, name)| name);
  if version.is_none_or(|v| v == TreeVersion::Resolved) && *options == listed {
    return name.to_owned();
  }
  let stem = name.strip_suffix(".svg").unwrap_or(name);
  let mut parts: Vec<String> = vec![stem.to_owned()];
  parts.extend(version.as_ref().map(wire_name));
  parts.push(wire_name(&scale));
  if options.width != listed.width {
    parts.push(format!("w{}", options.width));
  }
  if options.row_height != listed.row_height {
    parts.push(format!("row{}", options.row_height));
  }
  if options.labels != listed.labels {
    parts.push(format!("labels-{}", wire_name(&options.labels)));
  }
  format!("{}.svg", parts.join("_"))
}

/// The labels of the two trees of an ARG, segment A and then B; `None` for another number of
/// trees.
pub fn segment_labels(run: &RunResult) -> Option<[&str; 2]> {
  match run.trees.as_slice() {
    [a, b] => Some([a.label.as_str(), b.label.as_str()]),
    _ => None,
  }
}

/// `parameters.json`: the core options of a run and its seed. The command line writes it before
/// the inference, so it exists when a run fails.
pub fn parameters_file(o: &Options, seed: u64) -> OutputFile {
  OutputFile::new(PARAMETERS_FILE.to_owned(), format!("{:#}", params_json(o, seed)))
}

/// `log.txt` of the web app: one line `<time> [LEVEL] <message>` per record, the layout of the
/// command-line log without its thread ID, because the browser runs on one thread.
pub fn log_file(records: &[Diagnostic]) -> OutputFile {
  let text = records.iter().fold(String::new(), |mut text, r| {
    #[expect(clippy::expect_used, reason = "writing to a String does not fail")]
    writeln!(text, "{} [{}] {}", r.time, r.level, r.message).expect("writing to a String");
    text
  });
  OutputFile::new(LOG_FILE.to_owned(), text)
}

/// A ZIP archive of `files`, as `Archive` writes it.
pub fn zip_archive(files: &[OutputFile]) -> Result<Vec<u8>, ArchiveError> {
  let mut archive = Archive::with_capacity(files.iter().map(|f| f.text.len()).sum());
  for f in files {
    archive.add(&f.path, &f.text)?;
  }
  archive.finish()
}

/// A ZIP archive under construction, so that each file can be added while its text exists. Each
/// file is under `treeknit_results/` at its path. Every entry is deflated, dated 1980-01-01 00:00
/// (the earliest ZIP time), and made by a Unix system with the permissions `rw-r--r--`, so equal
/// files give a byte-identical archive on every host.
pub struct Archive {
  zip: ZipWriter<Cursor<Vec<u8>>>,
  /// The path of each file by its `analysis::label_key`, to reject paths that differ only in case.
  paths: BTreeMap<String, String>,
}

impl Archive {
  pub fn new() -> Archive {
    Archive::with_capacity(0)
  }

  /// An archive whose buffer holds `bytes` before it grows: the sum of the text lengths of the
  /// files bounds the archive of texts that deflate compresses, apart from the headers.
  pub fn with_capacity(bytes: usize) -> Archive {
    Archive {
      zip: ZipWriter::new(Cursor::new(Vec::with_capacity(bytes))),
      paths: BTreeMap::new(),
    }
  }

  /// Add the file `text` at `path`; fails when the archive has a file at `path` or at a path that
  /// differs only in case, because extracting it on macOS or Windows would overwrite one file
  /// with the other.
  pub fn add(&mut self, path: &str, text: &str) -> Result<(), ArchiveError> {
    match self.paths.entry(analysis::label_key(path)) {
      Entry::Occupied(e) => {
        return Err(ArchiveError::Repeated {
          first: e.get().clone(),
          path: path.to_owned(),
        });
      },
      Entry::Vacant(e) => {
        e.insert(path.to_owned());
      },
    }
    let options = SimpleFileOptions::default()
      .compression_method(CompressionMethod::Deflated)
      .last_modified_time(DateTime::DEFAULT)
      .system(System::Unix)
      .unix_permissions(0o644);
    self.zip.start_file(format!("{RESULTS_DIR}/{path}"), options)?;
    self.zip.write_all(text.as_bytes()).map_err(ZipError::from)?;
    Ok(())
  }

  /// The bytes of the archive.
  pub fn finish(self) -> Result<Vec<u8>, ArchiveError> {
    Ok(self.zip.finish()?.into_inner())
  }
}

impl Default for Archive {
  fn default() -> Self {
    Archive::new()
  }
}

/// Failure to build a ZIP archive.
#[derive(Debug)]
pub enum ArchiveError {
  /// Two files have the same path, ignoring case.
  Repeated { first: String, path: String },
  /// The ZIP writer failed.
  Zip(ZipError),
}

impl fmt::Display for ArchiveError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ArchiveError::Repeated { first, path } if first == path => {
        write!(f, "cannot build the ZIP archive: two files are named {path:?}")
      },
      ArchiveError::Repeated { first, path } => write!(
        f,
        "cannot build the ZIP archive: the files {first:?} and {path:?} differ only in case"
      ),
      ArchiveError::Zip(e) => write!(f, "cannot build the ZIP archive: {e}"),
    }
  }
}

impl std::error::Error for ArchiveError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      ArchiveError::Repeated { .. } => None,
      ArchiveError::Zip(e) => Some(e),
    }
  }
}

impl From<ZipError> for ArchiveError {
  fn from(e: ZipError) -> Self {
    ArchiveError::Zip(e)
  }
}

/// The session file of `request`, `treeknit_session.json`: its trees and settings as pretty
/// JSON, which `analysis::read_session` reads back.
pub fn session_file(request: &AnalysisRequest) -> OutputFile {
  #[expect(
    clippy::expect_used,
    reason = "a request has string keys and serde_json writes a non-finite number as null"
  )]
  let json = serde_json::to_string_pretty(request).expect("a request serializes to JSON");
  OutputFile::new(SESSION_FILE.to_owned(), format!("{json}\n"))
}

/// Results directory of `command_line`, next to the extracted `treeknit_results/`, so that the
/// command keeps the files it reproduces and the two sets can be compared.
pub const COMMAND_LINE_RESULTS_DIR: &str = "treeknit_results_cli";

/// The command that writes the file set of the web app (`web_files`), run in the directory where
/// its ZIP archive was extracted: the session file in `treeknit_results/`, the results directory
/// `treeknit_results_cli/`, and the flags of `OutputOptions::web`.
pub fn command_line() -> String {
  let flags = OutputOptions::web(0).flags().join(" ");
  format!("treeknit --session {RESULTS_DIR}/{SESSION_FILE} --outdir {COMMAND_LINE_RESULTS_DIR} {flags}")
}

/// Path of a tree file: `<dir><label><suffix><ext>`, such as `ARG/ha_liberal_resolved.nwk`.
fn tree_path(dir: &str, label: &str, suffix: &str, ext: &str) -> String {
  format!("{dir}{label}{suffix}{ext}")
}

/// Path of the MCCs of the pair of trees labeled `a` and `b` in the legacy text format of
/// TreeKnit.jl < 0.5 (one MCC per line): `MCCs.dat` for two trees (`k`), `MCCs_<a>_<b>.dat` per
/// pair otherwise.
fn mccs_lines_path(k: usize, a: &str, b: &str) -> String {
  if k == 2 {
    "MCCs.dat".to_owned()
  } else {
    format!("MCCs_{}.dat", analysis::pair_stem(a, b))
  }
}

/// Path of the Auspice JSON file of the tree labeled `label`.
fn auspice_path(label: &str) -> String {
  format!("auspice_{label}.json")
}

/// Path of the tanglegram figure of the pair of trees labeled `a` and `b`.
fn tanglegram_path(a: &str, b: &str) -> String {
  format!("tanglegram_{}.svg", analysis::pair_stem(a, b))
}

fn params_json(o: &Options, seed: u64) -> Value {
  json!({
      "gamma": o.gamma,
      "itmax": o.itmax,
      "likelihood_sort": o.likelihood_sort,
      "resolution": format!("{:?}", o.resolution).to_lowercase(),
      "seq_lengths": o.seq_lengths,
      "pre_resolve": o.pre_resolve,
      "rounds": o.rounds,
      "final_unresolved_round": o.final_unresolved_round,
      "nMCMC": o.n_mcmc,
      "sa_rep": o.sa_rep,
      "Tmin": o.t_min,
      "Tmax": o.t_max,
      "nT": o.n_t,
      "cooling_schedule": format!("{:?}", o.cooling).to_lowercase(),
      "naive": o.naive,
      "seed": seed,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::analysis::{self, Settings, TreeText};
  use crate::figure::LabelMode;
  use crate::run;
  use crate::summary::Level;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::json;
  use std::collections::BTreeSet;

  /// The two-tree example: X moved between the trees.
  const HA: &str = "((A,B),(C,(D,X)));";
  const NA: &str = "((A,(B,X)),(C,D));";
  /// The two-tree example with branch lengths, so that the scale `div` is shown.
  const HA_LENGTHS: &str = "((A:1,B:1):1,(C:1,(D:1,X:1):1):1);";
  const NA_LENGTHS: &str = "((A:1,(B:1,X:1):1):1,(C:1,D:1):1);";

  fn run_trees(trees: &[(&str, &str)]) -> RunResult {
    let texts: Vec<TreeText> = trees
      .iter()
      .map(|(label, newick)| TreeText {
        label: (*label).to_owned(),
        newick: (*newick).to_owned(),
      })
      .collect();
    let s = Settings::default();
    let opts = analysis::options(&s, texts.len(), false).unwrap();
    run::run(analysis::parse_trees(&texts).unwrap(), &opts, s.seed, &|_| {})
  }

  /// The output files of the trees `trees` with `options`.
  fn files_of(trees: &[(&str, &str)], options: &OutputOptions) -> Vec<OutputFile> {
    output_files(&run_trees(trees), options).unwrap()
  }

  fn all_files(k: usize) -> OutputOptions {
    OutputOptions {
      extensions: vec![".nwk".to_owned(); k],
      imputed: true,
      auspice: true,
      figures: false,
    }
  }

  fn paths(files: &[OutputFile]) -> Vec<&str> {
    files.iter().map(|f| f.path.as_str()).collect()
  }

  fn text<'a>(files: &'a [OutputFile], path: &str) -> &'a str {
    &files.iter().find(|f| f.path == path).unwrap().text
  }

  fn clades(newick: &str) -> BTreeSet<BTreeSet<String>> {
    let t = newick::parse(newick, "t").unwrap();
    t.internals()
      .into_iter()
      .filter(|&n| n != t.root)
      .map(|n| t.leaves_below(n).into_iter().map(|l| t.name(l).to_owned()).collect())
      .collect()
  }

  #[test]
  fn output_files_of_two_trees_are_those_of_the_command_line() {
    let files = files_of(&[("ha", HA), ("na", NA)], &all_files(2));
    let expected = vec![
      "MCCs.json",
      "MCCs.dat",
      "ha_resolved.nwk",
      "na_resolved.nwk",
      "ha_imputed.nwk",
      "na_imputed.nwk",
      "auspice_ha.json",
      "auspice_na.json",
      "ARG/arg.nwk",
      "ARG/nodes.dat",
      "ARG/ha_liberal_resolved.nwk",
      "ARG/na_liberal_resolved.nwk",
    ];
    assert_eq!(expected, paths(&files));
  }

  #[test]
  fn output_files_of_three_trees_have_every_pair_and_no_arg() {
    let t = "((A,B),(C,D));";
    let files = files_of(&[("ha", t), ("na", t), ("pb2", t)], &all_files(3));
    let expected = vec![
      "MCCs.json",
      "MCCs_ha_na.dat",
      "MCCs_ha_pb2.dat",
      "MCCs_na_pb2.dat",
      "ha_resolved.nwk",
      "na_resolved.nwk",
      "pb2_resolved.nwk",
      "ha_imputed.nwk",
      "na_imputed.nwk",
      "pb2_imputed.nwk",
      "auspice_ha.json",
      "auspice_na.json",
      "auspice_pb2.json",
    ];
    assert_eq!(expected, paths(&files));
  }

  #[test]
  fn output_files_without_switches_leave_out_imputed_and_auspice_files() {
    let options = OutputOptions {
      imputed: false,
      auspice: false,
      ..all_files(2)
    };
    let files = files_of(&[("ha", HA), ("na", NA)], &options);
    let expected = vec![
      "MCCs.json",
      "MCCs.dat",
      "ha_resolved.nwk",
      "na_resolved.nwk",
      "ARG/arg.nwk",
      "ARG/nodes.dat",
      "ARG/ha_liberal_resolved.nwk",
      "ARG/na_liberal_resolved.nwk",
    ];
    assert_eq!(expected, paths(&files));
  }

  #[test]
  fn output_files_keep_the_extension_of_each_tree() {
    let options = OutputOptions {
      extensions: vec![".tree".to_owned(), String::new()],
      ..all_files(2)
    };
    let files = files_of(&[("ha", HA), ("na", NA)], &options);
    let trees: Vec<&str> = paths(&files)
      .into_iter()
      .filter(|p| p.contains("resolved") || p.contains("imputed"))
      .collect();
    let expected = vec![
      "ha_resolved.tree",
      "na_resolved",
      "ha_imputed.tree",
      "na_imputed",
      "ARG/ha_liberal_resolved.tree",
      "ARG/na_liberal_resolved",
    ];
    assert_eq!(expected, trees);
  }

  fn labels(v: &[&str]) -> Vec<String> {
    v.iter().map(|&l| l.to_owned()).collect()
  }

  #[test]
  fn output_paths_list_every_file_that_a_run_writes() {
    let fixed = [PARAMETERS_FILE, LOG_FILE, SESSION_FILE];
    let t = "((A,B),(C,D));";
    for trees in [&[("ha", HA), ("na", NA)][..], &[("ha", t), ("na", t), ("pb2", t)]] {
      let options = OutputOptions {
        extensions: vec![".tree".to_owned(); trees.len()],
        ..OutputOptions::web(trees.len())
      };
      let written = files_of(trees, &options);
      let expected: Vec<&str> = paths(&written).into_iter().chain(fixed).collect();
      let names: Vec<String> = trees.iter().map(|(l, _)| (*l).to_owned()).collect();
      assert_eq!(expected, output_paths(&names, &options));
    }
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::pair_and_tree(  &["a", "resolved", "MCCs_a"], &[".nwk", ".nwk", ".dat"], "two output files are named \"MCCs_a_resolved.dat\"; rename a tree")]
  #[case::auspice_and_tree(&["auspice_x", "x_resolved"], &[".json", ".nwk"],       "two output files are named \"auspice_x_resolved.json\"; rename a tree")]
  #[case::case_only(      &["a", "Resolved", "mccs_a"], &[".nwk", ".nwk", ".dat"], "the output files \"MCCs_a_Resolved.dat\" and \"mccs_a_resolved.dat\" differ only in case; rename a tree")]
  #[trace]
  fn check_output_paths_reports_files_of_different_kinds_with_one_name(
    #[case] names: &[&str],
    #[case] extensions: &[&str],
    #[case] message: &str,
  ) {
    let options = OutputOptions {
      extensions: extensions.iter().map(|&e| e.to_owned()).collect(),
      ..OutputOptions::web(names.len())
    };
    let expected = vec![ValidationError {
      field: Some(Field::Trees),
      message: message.to_owned(),
      line: None,
      column: None,
    }];
    assert_eq!(expected, check_output_paths(&labels(names), &options));
  }

  #[test]
  fn check_output_paths_reports_file_names_longer_than_a_file_system_holds() {
    let (a, b) = ("a".repeat(125), "b".repeat(125));
    let error = |message: String| ValidationError {
      field: Some(Field::Trees),
      message,
      line: None,
      column: None,
    };
    // Oracle: `tanglegram_` (11 bytes), 125 + 1 + 125 bytes of labels, and `.svg` (4) make 266
    // bytes; the other names of two such labels stay below 255.
    let one = format!(
      "the output file name \"tanglegram_{a}_{b}.svg\" has 266 bytes, more than the 255 of a file name; shorten \
       the tree labels"
    );
    assert_eq!(
      vec![error(one)],
      check_output_paths(&labels(&[&a, &b]), &OutputOptions::web(2))
    );
    // Without figures, a label of 250 bytes gives four long names: the resolved (263 bytes),
    // imputed (262), and Auspice (263) files, and the ARG tree file (271).
    let c = "c".repeat(250);
    let many = format!(
      "4 output file names have more than the 255 bytes of a file name, such as \"{c}_liberal_resolved.nwk\" with \
       271 bytes; shorten the tree labels"
    );
    let options = OutputOptions {
      figures: false,
      ..OutputOptions::web(2)
    };
    assert_eq!(vec![error(many)], check_output_paths(&labels(&[&c, "na"]), &options));
  }

  #[test]
  fn check_output_paths_accepts_distinct_names() {
    let options = OutputOptions::web(3);
    assert_eq!(
      Vec::<ValidationError>::new(),
      check_output_paths(&labels(&["ha", "na", "pb2"]), &options)
    );
  }

  #[test]
  fn output_options_of_the_web_select_every_file_by_its_flag() {
    let expected = OutputOptions {
      extensions: vec![".nwk".to_owned(); 2],
      imputed: true,
      auspice: true,
      figures: true,
    };
    assert_eq!(expected, OutputOptions::web(2));
    assert_eq!(vec!["--impute", "--auspice-view", "--plot"], expected.flags());
    let none = OutputOptions {
      imputed: false,
      auspice: false,
      figures: false,
      ..expected
    };
    assert_eq!(Vec::<&str>::new(), none.flags());
  }

  #[test]
  fn web_files_are_the_session_file_the_output_files_the_figures_and_the_run_files() {
    let trees = [("ha", HA), ("na", NA)];
    let r = run_trees(&trees);
    let request = AnalysisRequest {
      trees: trees
        .iter()
        .map(|(label, newick)| TreeText {
          label: (*label).to_owned(),
          newick: (*newick).to_owned(),
        })
        .collect(),
      settings: Settings::default(),
    };
    let files = web_files(&request, &r, 1, &[]).unwrap();
    let listed: Vec<(&str, Option<Figure>)> = files
      .iter()
      .map(|f| match f {
        WebFile::Text(f) => (f.path.as_str(), None),
        WebFile::Figure(f) => (f.path.as_str(), Some(f.figure)),
      })
      .collect();
    let plain = |path| (path, None);
    let expected = vec![
      plain("treeknit_session.json"),
      plain("MCCs.json"),
      plain("MCCs.dat"),
      plain("ha_resolved.nwk"),
      plain("na_resolved.nwk"),
      plain("ha_imputed.nwk"),
      plain("na_imputed.nwk"),
      plain("auspice_ha.json"),
      plain("auspice_na.json"),
      plain("ARG/arg.nwk"),
      plain("ARG/nodes.dat"),
      plain("ARG/ha_liberal_resolved.nwk"),
      plain("ARG/na_liberal_resolved.nwk"),
      ("tanglegram_ha_na.svg", Some(Figure::Pair { pair: 0 })),
      ("ARG/arg.svg", Some(Figure::Arg)),
      plain("parameters.json"),
      plain("log.txt"),
    ];
    assert_eq!(expected, listed);
    // The texts are those of the command line with the web options.
    let texts: Vec<&OutputFile> = files
      .iter()
      .filter_map(|f| match f {
        WebFile::Text(f) => Some(f),
        WebFile::Figure(_) => None,
      })
      .collect();
    assert_eq!(&session_file(&request), texts[0]);
    assert_eq!(text(&files_of(&trees, &all_files(2)), "MCCs.json"), texts[1].text);
  }

  #[test]
  fn figure_files_are_a_tanglegram_per_pair_and_the_arg_of_two_trees() {
    let file = |path: &str, figure| FigureFile {
      path: path.to_owned(),
      figure,
    };
    let two = figure_files(&run_trees(&[("ha", HA), ("na", NA)]));
    let expected = vec![
      file("tanglegram_ha_na.svg", Figure::Pair { pair: 0 }),
      file("ARG/arg.svg", Figure::Arg),
    ];
    assert_eq!(expected, two);
    let t = "((A,B),(C,D));";
    let three = figure_files(&run_trees(&[("ha", t), ("na", t), ("pb2", t)]));
    let expected = vec![
      file("tanglegram_ha_na.svg", Figure::Pair { pair: 0 }),
      file("tanglegram_ha_pb2.svg", Figure::Pair { pair: 1 }),
      file("tanglegram_na_pb2.svg", Figure::Pair { pair: 2 }),
    ];
    assert_eq!(expected, three);
  }

  #[test]
  fn output_files_with_figures_end_with_the_figure_files() {
    let trees = [("ha", HA), ("na", NA)];
    let options = OutputOptions {
      figures: true,
      ..all_files(2)
    };
    let r = run_trees(&trees);
    let files = output_files(&r, &options).unwrap();
    let without = files_of(&trees, &all_files(2));
    // The files without figures keep their bytes, and the figures follow them.
    assert_eq!(without, files[..without.len()]);
    let figures: Vec<OutputFile> = files[without.len()..].to_vec();
    let expected = vec![
      OutputFile {
        path: "tanglegram_ha_na.svg".to_owned(),
        media_type: "image/svg+xml".to_owned(),
        text: figure_text(&r, Figure::Pair { pair: 0 }).unwrap(),
      },
      OutputFile {
        path: "ARG/arg.svg".to_owned(),
        media_type: "image/svg+xml".to_owned(),
        text: figure_text(&r, Figure::Arg).unwrap(),
      },
    ];
    assert_eq!(expected, figures);
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::none(        HA,                                      NA,                                      Scale::Depth, Scale::Depth)]
  #[case::both(        "((A:1,B:1):1,(C:1,(D:1,X:1):1):1);",    "((A:1,(B:1,X:1):1):1,(C:1,D:1):1);",    Scale::Div,   Scale::Div)]
  #[case::one_tree(    "((A:1,B:1):1,(C:1,(D:1,X:1):1):1);",    NA,                                      Scale::Depth, Scale::Div)]
  #[case::some_branches("((A:1,B),(C,(D,X)));",                 "((A:1,(B:1,X:1):1):1,(C:1,D:1):1);",    Scale::Div,   Scale::Div)]
  #[trace]
  fn figures_with_the_div_scale_show_the_scale_of_the_views(
    #[case] ha: &str,
    #[case] na: &str,
    #[case] pair_scale: Scale,
    #[case] arg_scale: Scale,
  ) {
    // A tree without branch lengths draws the pair as cladograms; the ARG takes a missing length
    // from the other tree, so it needs lengths in neither tree to become a cladogram.
    let r = run_trees(&[("ha", ha), ("na", na)]);
    let view = display::pair_view(&r, 0, TreeVersion::Resolved, Scale::Div).unwrap();
    let arg = display::arg_view(&r, Scale::Div).unwrap();
    assert_eq!((pair_scale, arg_scale), (view.scale, arg.scale));
    let options = FigureOptions::default();
    let expected = (
      figure::tanglegram_svg(&view, &options).unwrap(),
      figure::arg_svg(&arg, ["ha", "na"], &options).unwrap(),
    );
    assert_eq!(
      (Some(expected.0), Some(expected.1)),
      (figure_text(&r, Figure::Pair { pair: 0 }), figure_text(&r, Figure::Arg))
    );
  }

  #[test]
  fn pair_figure_with_the_depth_scale_keeps_it() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    let depth = FigureOptions {
      scale: Scale::Depth,
      width: 500.0,
      ..FigureOptions::default()
    };
    let view = display::pair_view(&r, 0, TreeVersion::Input, Scale::Depth).unwrap();
    let expected = FigureDownload {
      file_name: "tanglegram_ha_na_input_depth_w500.svg".to_owned(),
      text: figure::tanglegram_svg(&view, &depth).unwrap(),
    };
    assert_eq!(Some(expected), pair_figure(&r, 0, TreeVersion::Input, &depth).unwrap());
    assert_eq!(None, pair_figure(&r, 1, TreeVersion::Input, &depth).unwrap());
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::resolved_div_both(("ha", "na", true),                        TreeVersion::Resolved, Scale::Div,   AuspiceTrees::Both,  ("auspice_ha_na",                                             vec!["auspice_ha_na_ha.json", "auspice_ha_na_na.json"]))]
  #[case::imputed_depth_both(("ha", "na", true),                        TreeVersion::Imputed,  Scale::Depth, AuspiceTrees::Both,  ("auspice_ha_na_imputed_depth",                               vec!["auspice_ha_na_imputed_depth_ha.json", "auspice_ha_na_imputed_depth_na.json"]))]
  #[case::input_div_left(("ha", "na", true),                        TreeVersion::Input,    Scale::Div,   AuspiceTrees::Left,  ("auspice_ha_na_input_div_ha",                                vec!["auspice_ha_na_input_div_ha.json"]))]
  #[case::resolved_div_right(("ha", "na", true),                        TreeVersion::Resolved, Scale::Div,   AuspiceTrees::Right, ("auspice_ha_na_na",                                          vec!["auspice_ha_na_na.json"]))]
  #[case::div_shown_as_depth_without_lengths_is_in_the_name(("ha", "na", false), TreeVersion::Resolved, Scale::Div, AuspiceTrees::Both, ("auspice_ha_na_resolved_depth", vec!["auspice_ha_na_resolved_depth_ha.json", "auspice_ha_na_resolved_depth_na.json"]))]
  #[case::long_labels(("segment_4_hemagglutinin", "segment_6_neuraminidase", true), TreeVersion::Resolved, Scale::Div, AuspiceTrees::Both, ("auspice_segment_4_hemagglutinin_segment_6_neuraminidase", vec!["auspice_segment_4_hemagglutinin_segment_6_neuraminidase_segment_4_hemagglutinin.json", "auspice_segment_4_hemagglutinin_segment_6_neuraminidase_segment_6_neuraminidase.json"]))]
  #[trace]
  fn auspice_files_names_tell_the_pair_version_scale_and_shown_trees(
    #[case] (a, b, lengths): (&str, &str, bool),
    #[case] version: TreeVersion,
    #[case] scale: Scale,
    #[case] trees: AuspiceTrees,
    #[case] (svg_prefix, json): (&str, Vec<&str>),
  ) {
    let (ha, na) = if lengths { (HA_LENGTHS, NA_LENGTHS) } else { (HA, NA) };
    let r = run_trees(&[(a, ha), (b, na)]);
    let files = auspice_files(&r, 0, version, scale, trees).unwrap();
    let paths: Vec<&str> = files.json.iter().map(|f| f.path.as_str()).collect();
    assert_eq!((svg_prefix, json), (files.svg_prefix.as_str(), paths));
  }

  #[test]
  fn auspice_files_hold_the_datasets_of_the_auspice_view_as_json() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    let view = display::auspice_view(&r, 0, TreeVersion::Resolved, Scale::Div).unwrap();
    let files = auspice_files(&r, 0, TreeVersion::Resolved, Scale::Div, AuspiceTrees::Both).unwrap();
    let parsed: Vec<Value> = files
      .json
      .iter()
      .map(|f| serde_json::from_str(&f.text).unwrap())
      .collect();
    let expected = vec![
      serde_json::to_value(&view.left).unwrap(),
      serde_json::to_value(&view.right).unwrap(),
    ];
    assert_eq!(expected, parsed);
    assert!(files.json.iter().all(|f| f.media_type == "application/json"));
    assert_eq!(
      None,
      auspice_files(&r, 1, TreeVersion::Resolved, Scale::Div, AuspiceTrees::Both)
    );
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::listed(   TreeVersion::Resolved, 1200.0, 12.0, Scale::Div,   LabelMode::Auto, "tanglegram_ha_na.svg",                                  "arg.svg")]
  #[case::version(  TreeVersion::Imputed,  1200.0, 12.0, Scale::Div,   LabelMode::Auto, "tanglegram_ha_na_imputed_depth.svg",                    "arg.svg")]
  #[case::scale(    TreeVersion::Resolved, 1200.0, 12.0, Scale::Depth, LabelMode::Auto, "tanglegram_ha_na_resolved_depth.svg",                   "arg_depth.svg")]
  #[case::options(  TreeVersion::Input,    640.5,  20.0, Scale::Div,   LabelMode::Off,  "tanglegram_ha_na_input_depth_w640.5_row20_labels-off.svg", "arg_depth_w640.5_row20_labels-off.svg")]
  #[trace]
  fn figure_downloads_name_the_version_and_the_options_that_differ_from_the_listed_figure(
    #[case] version: TreeVersion,
    #[case] width: f64,
    #[case] row_height: f64,
    #[case] scale: Scale,
    #[case] labels: LabelMode,
    #[case] pair_name: &str,
    #[case] arg_name: &str,
  ) {
    // The example trees have no branch lengths, so every figure shows the scale `depth`.
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    let options = FigureOptions { width, row_height, scale, labels };
    let pair = pair_figure(&r, 0, version, &options).unwrap().unwrap();
    let arg = arg_figure(&r, &options).unwrap().unwrap();
    assert_eq!((pair_name, arg_name), (pair.file_name.as_str(), arg.file_name.as_str()));
  }

  #[test]
  fn figure_text_of_a_missing_pair_or_arg_is_none() {
    let t = "((A,B),(C,D));";
    let r = run_trees(&[("ha", t), ("na", t), ("pb2", t)]);
    assert_eq!(
      (None, None),
      (figure_text(&r, Figure::Pair { pair: 3 }), figure_text(&r, Figure::Arg))
    );
  }

  #[test]
  fn output_files_hold_the_mccs_of_the_reference() {
    let files = files_of(&[("ha", HA), ("na", NA)], &all_files(2));
    // Oracle: fixtures/doc_mccs_1.json (TreeKnit.jl).
    let expected = json!({"MCC_dict": {"1": {"trees": ["ha", "na"], "mccs": [["X"], ["A", "B", "C", "D"]]}}});
    assert_eq!(
      expected,
      serde_json::from_str::<Value>(text(&files, "MCCs.json")).unwrap()
    );
    assert_eq!("X\nA,B,C,D", text(&files, "MCCs.dat"));
  }

  #[test]
  fn output_files_place_a_leaf_missing_from_one_tree() {
    let files = files_of(&[("ha", "((A,B),(C,(D,P)));"), ("na", "((A,B),(C,D));")], &all_files(2));
    let expected = json!({"MCC_dict": {"1": {
        "trees": ["ha", "na"],
        "mccs": [["A", "B", "C", "D", "P"]],
        "imputed": [{"leaf": "P", "tree": "ha", "mcc": 0, "ambiguous": false}],
    }}});
    assert_eq!(
      expected,
      serde_json::from_str::<Value>(text(&files, "MCCs.json")).unwrap()
    );
    // Oracle: P is the sister of D in ha, the only tree that has it.
    assert_eq!(clades("((A,B),(C,(D,P)));"), clades(text(&files, "na_imputed.nwk")));
    assert_eq!(clades("((A,B),(C,D));"), clades(text(&files, "na_resolved.nwk")));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::mccs_json(  "MCCs.json",                   true)]
  #[case::mccs_dat(   "MCCs.dat",                    false)]
  #[case::resolved(   "ha_resolved.nwk",             true)]
  #[case::imputed(    "ha_imputed.nwk",              true)]
  #[case::auspice(    "auspice_ha.json",             false)]
  #[case::arg(        "ARG/arg.nwk",                 true)]
  #[case::nodes(      "ARG/nodes.dat",               true)]
  #[case::liberal(    "ARG/ha_liberal_resolved.nwk", true)]
  #[trace]
  fn output_files_end_with_a_newline_as_the_command_line_writes_them(#[case] path: &str, #[case] newline: bool) {
    // Oracle: the newline rule of the command line before the shared output files.
    let files = files_of(&[("ha", HA), ("na", NA)], &all_files(2));
    assert_eq!(newline, text(&files, path).ends_with('\n'));
  }

  #[test]
  fn parameters_file_holds_the_options_and_the_seed_without_a_final_newline() {
    let opts = analysis::options(
      &Settings {
        gamma: 3.5,
        ..Settings::default()
      },
      2,
      true,
    )
    .unwrap();
    let file = parameters_file(&opts, 7);
    assert_eq!(
      ("parameters.json", "application/json"),
      (file.path.as_str(), file.media_type.as_str())
    );
    assert!(!file.text.ends_with('\n'));
    let v: Value = serde_json::from_str(&file.text).unwrap();
    assert_eq!(
      (json!(3.5), json!(7), json!("matched")),
      (v["gamma"].clone(), v["seed"].clone(), v["resolution"].clone())
    );
  }

  #[test]
  fn session_file_round_trips_through_read_session() {
    let request = AnalysisRequest {
      trees: vec![
        TreeText {
          label: "ha".to_owned(),
          newick: HA.to_owned(),
        },
        TreeText {
          label: "na".to_owned(),
          newick: NA.to_owned(),
        },
      ],
      settings: Settings {
        seq_lengths: Some(vec![1700.0, 1400.0]),
        seed: analysis::MAX_SEED,
        ..Settings::default()
      },
    };
    let file = session_file(&request);
    assert_eq!(
      ("treeknit_session.json", "application/json"),
      (file.path.as_str(), file.media_type.as_str())
    );
    assert!(file.text.ends_with("}\n"));
    assert_eq!(Ok(request), analysis::read_session(&file.text));
  }

  #[test]
  fn command_line_runs_the_session_file_of_the_extracted_archive() {
    assert_eq!(
      "treeknit --session treeknit_results/treeknit_session.json --outdir treeknit_results_cli --impute --auspice-view \
       --plot",
      command_line()
    );
  }

  #[test]
  fn log_file_writes_a_line_per_record_as_the_command_line_log() {
    let record = |level, message: &str| Diagnostic {
      level,
      message: message.to_owned(),
      time: "2026-10-05T12:00:00.000Z".to_owned(),
    };
    let file = log_file(&[record(Level::Info, "TreeKnit 1.0"), record(Level::Warn, "ignoring x")]);
    // Oracle: simplelog with `set_time_format_rfc3339` and target off, as `setup_logging` of the
    // command line configures it, writes `<time> [LEVEL] <message>` and a newline.
    let expected = OutputFile {
      path: "log.txt".to_owned(),
      media_type: "text/plain".to_owned(),
      text: "2026-10-05T12:00:00.000Z [INFO] TreeKnit 1.0\n2026-10-05T12:00:00.000Z [WARN] ignoring x\n".to_owned(),
    };
    assert_eq!(expected, file);
  }

  fn fixed_files() -> Vec<OutputFile> {
    vec![
      OutputFile::new("MCCs.json".to_owned(), "{}\n".to_owned()),
      OutputFile::new("ARG/arg.nwk".to_owned(), "(A,B);\n".to_owned()),
      OutputFile::new("MCCs.dat".to_owned(), "A,B".to_owned()),
    ]
  }

  #[test]
  fn zip_archive_holds_every_file_under_the_results_directory() {
    let bytes = zip_archive(&fixed_files()).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let entries: Vec<ZipEntry> = (0..archive.len())
      .map(|i| {
        let mut entry = archive.by_index(i).unwrap();
        let mut text = String::new();
        std::io::Read::read_to_string(&mut entry, &mut text).unwrap();
        ZipEntry {
          name: entry.name().to_owned(),
          text,
          modified: entry.last_modified(),
          compression: entry.compression(),
          unix_mode: entry.unix_mode(),
        }
      })
      .collect();
    let entry = |name: &str, text: &str| ZipEntry {
      name: name.to_owned(),
      text: text.to_owned(),
      modified: Some(DateTime::DEFAULT),
      compression: CompressionMethod::Deflated,
      // Oracle: a regular file (S_IFREG, 0o100000) with the permissions rw-r--r--.
      unix_mode: Some(0o100_644),
    };
    let expected = vec![
      entry("treeknit_results/MCCs.json", "{}\n"),
      entry("treeknit_results/ARG/arg.nwk", "(A,B);\n"),
      entry("treeknit_results/MCCs.dat", "A,B"),
    ];
    assert_eq!(expected, entries);
  }

  /// An entry of a ZIP archive as `zip::ZipArchive` reads it back.
  #[derive(Debug, PartialEq)]
  struct ZipEntry {
    name: String,
    text: String,
    modified: Option<DateTime>,
    compression: CompressionMethod,
    unix_mode: Option<u32>,
  }

  #[test]
  fn zip_archive_entries_are_made_by_unix_on_every_host() {
    // Oracle: APPNOTE.TXT 4.4.2 and 4.4.15: byte 5 of a central directory header is the host
    // system of "version made by" (3 for Unix), and bytes 38 to 41 are the external attributes,
    // which hold the Unix mode in their upper 16 bits.
    let bytes = zip_archive(&fixed_files()).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes.clone())).unwrap();
    let headers: Vec<(u8, u32)> = (0..archive.len())
      .map(|i| {
        let start = usize::try_from(archive.by_index(i).unwrap().central_header_start()).unwrap();
        #[expect(clippy::little_endian_bytes, reason = "ZIP headers are little-endian")]
        let attributes = u32::from_le_bytes(bytes[start + 38..start + 42].try_into().unwrap());
        (bytes[start + 5], attributes)
      })
      .collect();
    assert_eq!(vec![(3, 0o100_644 << 16); 3], headers);
  }

  #[test]
  fn zip_archive_of_equal_files_is_byte_identical_within_a_run() {
    // The fixed time, host system, and permissions of the tests around this one make the bytes
    // independent of the host.
    assert_eq!(
      zip_archive(&fixed_files()).unwrap(),
      zip_archive(&fixed_files()).unwrap()
    );
  }

  #[test]
  fn zip_archive_keeps_non_ascii_names_as_utf8() {
    let files = vec![OutputFile::new("Ålesund_résolu.nwk".to_owned(), "(A,B);\n".to_owned())];
    let mut archive = zip::ZipArchive::new(Cursor::new(zip_archive(&files).unwrap())).unwrap();
    assert_eq!(
      "treeknit_results/Ålesund_résolu.nwk",
      archive.by_index(0).unwrap().name()
    );
  }

  #[test]
  fn zip_archive_dates_entries_at_the_earliest_zip_time() {
    // Oracle: the MS-DOS date of 1980-01-01 is 0x0021 (day 1, month 1, year 1980 + 0) and the
    // time of 00:00:00 is 0; the local file header holds the time at byte 10 and the date at byte
    // 12 (APPNOTE.TXT 4.3.7).
    let bytes = zip_archive(&fixed_files()).unwrap();
    assert_eq!(
      [0x50, 0x4b, 0x03, 0x04, 0x00, 0x00, 0x21, 0x00],
      [&bytes[..4], &bytes[10..14]].concat()[..]
    );
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::repeated("MCCs.json", "cannot build the ZIP archive: two files are named \"MCCs.json\"")]
  #[case::case_only("mccs.JSON", "cannot build the ZIP archive: the files \"MCCs.json\" and \"mccs.JSON\" differ only in case")]
  #[trace]
  fn zip_archive_rejects_a_path_repeated_ignoring_case(#[case] second: &str, #[case] message: &str) {
    let files = vec![
      OutputFile::new("MCCs.json".to_owned(), String::new()),
      OutputFile::new(second.to_owned(), String::new()),
    ];
    let error = zip_archive(&files).unwrap_err();
    assert!(matches!(&error, ArchiveError::Repeated { first, path } if first == "MCCs.json" && path == second));
    assert_eq!(message, error.to_string());
  }
  #[rustfmt::skip]
  #[rstest]
  #[case::json(  "MCCs.json",   "application/json")]
  #[case::upper( "A.JSON",      "application/json")]
  #[case::newick("ha.nwk",      "text/plain")]
  #[case::table( "ARG/nodes.dat", "text/plain")]
  #[case::none(  "ha_resolved", "text/plain")]
  #[case::svg(   "ARG/arg.svg", "image/svg+xml")]
  #[trace]
  fn output_file_media_type_follows_the_extension(#[case] path: &str, #[case] expected: &str) {
    assert_eq!(expected, OutputFile::new(path.to_owned(), String::new()).media_type);
  }

  #[test]
  fn file_entry_serializes_camel_case_with_null_size() {
    let entry = FileEntry::new("tanglegram_ha_na.svg".into(), None, Some(Figure::Pair { pair: 2 }));
    let expected = json!({
      "path": "tanglegram_ha_na.svg", "fileName": "tanglegram_ha_na.svg", "mediaType": "image/svg+xml", "size": null,
      "figure": {"kind": "pair", "pair": 2},
    });
    assert_eq!(expected, serde_json::to_value(&entry).unwrap());
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::top(      "MCCs.json",                   "MCCs.json")]
  #[case::directory("ARG/ha_liberal_resolved.nwk", "ha_liberal_resolved.nwk")]
  #[trace]
  fn file_entry_names_the_file_by_the_last_segment_of_its_path(#[case] path: &str, #[case] expected: &str) {
    let entry = FileEntry::new(path.to_owned(), Some(0), None);
    assert_eq!(expected, entry.file_name);
  }
}
