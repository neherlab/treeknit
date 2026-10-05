//! Output files of a run, as the command line writes them and the web app lists them.

use crate::analysis::{AnalysisRequest, ValidationError};
use crate::display::{self, Scale, TreeVersion};
use crate::figure::{self, FigureOptions};
use crate::run::RunResult;
use crate::summary::Diagnostic;
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
/// line runs with `--request`.
pub const REQUEST_FILE: &str = "treeknit_request.json";

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
  pub media_type: String,
  /// Size in bytes; `None` for a figure not rendered yet.
  pub size: Option<usize>,
  /// The figure the file holds; `None` for the other files.
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
  /// The entry of the file at `path`, with its `file_name`.
  pub fn new(path: String, media_type: String, size: Option<usize>, figure: Option<Figure>) -> Self {
    let file_name = path.rsplit_once('/').map_or(path.as_str(), |(_, name)| name).to_owned();
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
  /// A file at `path` with its media type taken from the extension.
  pub fn new(path: String, text: String) -> Self {
    let media_type = match Path::new(&path).extension().and_then(|e| e.to_str()) {
      Some(e) if e.eq_ignore_ascii_case("json") => "application/json",
      Some(e) if e.eq_ignore_ascii_case("svg") => "image/svg+xml",
      _ => "text/plain",
    };
    OutputFile {
      path,
      media_type: media_type.to_owned(),
      text,
    }
  }
}

/// Every output file of `run` except `parameters.json` and `log.txt`, at its path in the results
/// directory, with the bytes the command line writes: the MCCs as JSON and as lines, the
/// resolved trees, the imputed trees and Auspice files when `options` asks for them, for two
/// trees the ARG files, and the figures when `options` asks for them. File names follow the tree
/// labels. `opts` are the options of the run, by which the figures sort the trees of a pair.
pub fn output_files(run: &RunResult, opts: &Options, options: &OutputOptions) -> Vec<OutputFile> {
  let RunResult {
    trees,
    taxa,
    pairs,
    imputed,
    ..
  } = run;
  let tree_file = |dir: &str, suffix: &str, i: usize, t: &Tree| {
    OutputFile::new(
      tree_path(dir, &t.label, suffix, options.extension(i)),
      format!("{}\n", newick::write(t)),
    )
  };
  let mut files = vec![OutputFile::new(
    MCCS_JSON.to_owned(),
    format!("{:#}\n", mccs::to_json(pairs, trees, taxa)),
  )];
  files.extend(pairs.iter().map(|p| {
    let path = mccs_lines_path(trees.len(), &trees[p.i].label, &trees[p.j].label);
    let names: Vec<Vec<String>> = p.mccs.iter().map(|m| taxa.names_of(m)).collect();
    OutputFile::new(path, mccs::to_lines(&names))
  }));
  files.extend(trees.iter().enumerate().map(|(i, t)| tree_file("", "_resolved", i, t)));
  if options.imputed {
    files.extend(imputed.iter().enumerate().map(|(i, t)| tree_file("", "_imputed", i, t)));
  }
  if options.auspice {
    files.extend(trees.iter().enumerate().map(|(i, t)| {
      OutputFile::new(
        auspice_path(&t.label),
        format!("{:#}", auspice::auspice_json(i, trees, pairs, taxa)),
      )
    }));
  }
  if let Some(a) = run.built_arg() {
    files.push(OutputFile::new(
      ARG_NEWICK.to_owned(),
      format!("{}\n", arg::extended_newick(a)),
    ));
    files.push(OutputFile::new(
      ARG_NODES.to_owned(),
      format!("{}\n", arg::node_table(a)),
    ));
    files.extend(
      a.trees
        .iter()
        .enumerate()
        .map(|(i, t)| tree_file("ARG/", "_liberal_resolved", i, t)),
    );
  }
  if options.figures {
    // `figure_files` lists only figures that `run` has, so each has a text.
    files.extend(
      figure_files(run)
        .into_iter()
        .filter_map(|f| Some(OutputFile::new(f.path, figure_text(run, opts, f.figure)?))),
    );
  }
  files
}

/// The figures of `run`: the tanglegram of each pair, `tanglegram_<a>_<b>.svg`, then for two
/// trees with a built ARG `ARG/arg.svg`.
pub fn figure_files(run: &RunResult) -> Vec<FigureFile> {
  let pairs = run.pairs.iter().enumerate().map(|(i, p)| FigureFile {
    path: tanglegram_path(&run.trees[p.i].label, &run.trees[p.j].label),
    figure: Figure::Pair { pair: i },
  });
  let arg = run.built_arg().map(|_| FigureFile {
    path: ARG_FIGURE.to_owned(),
    figure: Figure::Arg,
  });
  pairs.chain(arg).collect()
}

/// The file set of a run of the web app for `request`, in order: the session file, the files of
/// `output_files` with `OutputOptions::web`, the figures of `figure_files`, `parameters.json`,
/// and `log.txt` of `records`. `command_line` writes the same files. `opts` and `seed` are the
/// options and the seed of the run.
pub fn web_files(
  request: &AnalysisRequest,
  run: &RunResult,
  opts: &Options,
  seed: u64,
  records: &[Diagnostic],
) -> Vec<WebFile> {
  let options = OutputOptions {
    figures: false,
    ..OutputOptions::web(run.trees.len())
  };
  let mut files = vec![WebFile::Text(request_file(request))];
  files.extend(output_files(run, opts, &options).into_iter().map(WebFile::Text));
  files.extend(figure_files(run).into_iter().map(WebFile::Figure));
  files.push(WebFile::Text(parameters_file(opts, seed)));
  files.push(WebFile::Text(log_file(records)));
  files
}

/// Every path that a run of the trees labeled `labels` can write with `options`, in the order of
/// `output_files`, followed by `parameters.json`, `log.txt`, and the session file. For two trees
/// the ARG files are listed, although a run writes them only when the ARG is built.
pub fn output_paths(labels: &[String], options: &OutputOptions) -> Vec<String> {
  let k = labels.len();
  let pairs: Vec<(&str, &str)> = (0..k)
    .flat_map(|i| (i + 1..k).map(move |j| (labels[i].as_str(), labels[j].as_str())))
    .collect();
  let trees = |dir: &'static str, suffix: &'static str| {
    labels
      .iter()
      .enumerate()
      .map(move |(i, l)| tree_path(dir, l, suffix, options.extension(i)))
  };
  let mut paths = vec![MCCS_JSON.to_owned()];
  paths.extend(pairs.iter().map(|(a, b)| mccs_lines_path(k, a, b)));
  paths.extend(trees("", "_resolved"));
  if options.imputed {
    paths.extend(trees("", "_imputed"));
  }
  if options.auspice {
    paths.extend(labels.iter().map(|l| auspice_path(l)));
  }
  if k == 2 {
    paths.extend([ARG_NEWICK.to_owned(), ARG_NODES.to_owned()]);
    paths.extend(trees("ARG/", "_liberal_resolved"));
  }
  if options.figures {
    paths.extend(pairs.iter().map(|(a, b)| tanglegram_path(a, b)));
    if k == 2 {
      paths.push(ARG_FIGURE.to_owned());
    }
  }
  paths.extend([PARAMETERS_FILE, LOG_FILE, REQUEST_FILE].map(str::to_owned));
  paths
}

/// Output files of different trees or kinds that would get the same name, ignoring case as the
/// label check does: the resolved tree of `MCCs_a` from `MCCs_a.dat` and the MCCs of the pair
/// (`a`, `resolved`) are both `MCCs_a_resolved.dat`. The command line would overwrite one with the
/// other, and the ZIP archive of the web app cannot hold both. `labels` must have passed the
/// checks of `analysis::parse_trees`, which reports repeated labels and pair names.
pub fn check_output_paths(labels: &[String], options: &OutputOptions) -> Vec<ValidationError> {
  let mut first: BTreeMap<String, String> = BTreeMap::new();
  let mut errors = Vec::new();
  for path in output_paths(labels, options) {
    match first.entry(analysis::label_key(&path)) {
      Entry::Vacant(e) => {
        e.insert(path);
      },
      Entry::Occupied(e) => {
        let message = if *e.get() == path {
          format!("two output files are named {path:?}; rename a tree")
        } else {
          format!(
            "the output files {:?} and {path:?} differ only in case; rename a tree",
            e.get()
          )
        };
        errors.push(ValidationError {
          field: Some("trees".to_owned()),
          message,
          line: None,
          column: None,
        });
      },
    }
  }
  errors
}

/// The SVG text of `figure` of `run` with the default `FigureOptions`, a tanglegram of the
/// resolved trees; `None` when `run` lacks the pair or the ARG. `opts` are the options of the
/// run. Trees without branch lengths are drawn as cladograms (scale `depth`), because their
/// divergence is 0 everywhere and the `div` scale would draw every node at the root.
pub fn figure_text(run: &RunResult, opts: &Options, figure: Figure) -> Option<String> {
  let div = FigureOptions::default();
  let depth = FigureOptions {
    scale: Scale::Depth,
    ..div
  };
  let svg = match figure {
    Figure::Pair { pair } => {
      let view = display::pair_view(run, opts, pair, TreeVersion::Resolved, div.scale)?;
      let flat = view.left.nodes.iter().chain(&view.right.nodes).all(|n| n.x_div <= 0.0);
      if flat {
        let view = display::pair_view(run, opts, pair, TreeVersion::Resolved, depth.scale)?;
        figure::tanglegram_svg(&view, &depth)
      } else {
        figure::tanglegram_svg(&view, &div)
      }
    },
    Figure::Arg => {
      let segments = segment_labels(run)?;
      let view = display::arg_view(run, div.scale)?;
      if view.nodes.iter().all(|n| n.x_div <= 0.0) {
        figure::arg_svg(&display::arg_view(run, depth.scale)?, segments, &depth)
      } else {
        figure::arg_svg(&view, segments, &div)
      }
    },
  };
  #[expect(clippy::expect_used, reason = "the default figure options are valid")]
  Some(svg.expect("default figure options"))
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

/// A ZIP archive of `files`, each under `treeknit_results/` at its path. Every entry is deflated,
/// dated 1980-01-01 00:00 (the earliest ZIP time), and made by a Unix system with the permissions
/// `rw-r--r--`, so equal files give a byte-identical archive on every host. Fails when two files
/// have one path.
pub fn zip_archive(files: &[OutputFile]) -> Result<Vec<u8>, ArchiveError> {
  let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated)
    .last_modified_time(DateTime::DEFAULT)
    .system(System::Unix)
    .unix_permissions(0o644);
  let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
  for f in files {
    zip.start_file(format!("{RESULTS_DIR}/{}", f.path), options)?;
    zip.write_all(f.text.as_bytes()).map_err(ZipError::from)?;
  }
  Ok(zip.finish()?.into_inner())
}

/// Failure to build the ZIP archive of `zip_archive`.
#[derive(Debug)]
pub struct ArchiveError(ZipError);

impl fmt::Display for ArchiveError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "cannot build the ZIP archive: {}", self.0)
  }
}

impl std::error::Error for ArchiveError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    Some(&self.0)
  }
}

impl From<ZipError> for ArchiveError {
  fn from(e: ZipError) -> Self {
    ArchiveError(e)
  }
}

/// The session file of `request`, `treeknit_request.json`: its trees and settings as pretty
/// JSON, which `analysis::read_request` reads back.
pub fn request_file(request: &AnalysisRequest) -> OutputFile {
  #[expect(
    clippy::expect_used,
    reason = "a request has string keys and serde_json writes a non-finite number as null"
  )]
  let json = serde_json::to_string_pretty(request).expect("a request serializes to JSON");
  OutputFile::new(REQUEST_FILE.to_owned(), format!("{json}\n"))
}

/// The command that writes the file set of the web app (`web_files`), run in the directory where
/// its ZIP archive was extracted: the session file in `treeknit_results/`, with the flags of
/// `OutputOptions::web`.
pub fn command_line() -> String {
  let flags = OutputOptions::web(0).flags().join(" ");
  format!("treeknit --request {RESULTS_DIR}/{REQUEST_FILE} {flags}")
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
  use crate::run;
  use crate::summary::Level;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::json;
  use std::collections::BTreeSet;

  /// The two-tree example: X moved between the trees.
  const HA: &str = "((A,B),(C,(D,X)));";
  const NA: &str = "((A,(B,X)),(C,D));";

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

  /// The options of `run_trees` for `k` trees.
  fn run_options(k: usize) -> Options {
    analysis::options(&Settings::default(), k, false).unwrap()
  }

  /// The output files of the trees `trees` with `options`.
  fn files_of(trees: &[(&str, &str)], options: &OutputOptions) -> Vec<OutputFile> {
    output_files(&run_trees(trees), &run_options(trees.len()), options)
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
    let fixed = [PARAMETERS_FILE, LOG_FILE, REQUEST_FILE];
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
      field: Some("trees".to_owned()),
      message: message.to_owned(),
      line: None,
      column: None,
    }];
    assert_eq!(expected, check_output_paths(&labels(names), &options));
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
    let (r, opts) = (run_trees(&trees), run_options(2));
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
    let files = web_files(&request, &r, &opts, 1, &[]);
    let options = OutputOptions {
      figures: false,
      ..OutputOptions::web(2)
    };
    let mut expected = vec![WebFile::Text(request_file(&request))];
    expected.extend(output_files(&r, &opts, &options).into_iter().map(WebFile::Text));
    expected.extend(figure_files(&r).into_iter().map(WebFile::Figure));
    expected.push(WebFile::Text(parameters_file(&opts, 1)));
    expected.push(WebFile::Text(log_file(&[])));
    assert_eq!(expected, files);
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
    let (r, opts) = (run_trees(&trees), run_options(2));
    let files = output_files(&r, &opts, &options);
    let without = files_of(&trees, &all_files(2));
    // The files without figures keep their bytes, and the figures follow them.
    assert_eq!(without, files[..without.len()]);
    let figures: Vec<OutputFile> = files[without.len()..].to_vec();
    let expected = vec![
      OutputFile {
        path: "tanglegram_ha_na.svg".to_owned(),
        media_type: "image/svg+xml".to_owned(),
        text: figure_text(&r, &opts, Figure::Pair { pair: 0 }).unwrap(),
      },
      OutputFile {
        path: "ARG/arg.svg".to_owned(),
        media_type: "image/svg+xml".to_owned(),
        text: figure_text(&r, &opts, Figure::Arg).unwrap(),
      },
    ];
    assert_eq!(expected, figures);
  }

  #[test]
  fn figure_text_draws_trees_without_branch_lengths_as_cladograms() {
    let (r, opts) = (run_trees(&[("ha", HA), ("na", NA)]), run_options(2));
    let depth = FigureOptions {
      scale: Scale::Depth,
      ..FigureOptions::default()
    };
    let view = display::pair_view(&r, &opts, 0, TreeVersion::Resolved, Scale::Depth).unwrap();
    let expected = figure::tanglegram_svg(&view, &depth).unwrap();
    assert_eq!(Some(expected), figure_text(&r, &opts, Figure::Pair { pair: 0 }));
    let arg = display::arg_view(&r, Scale::Depth).unwrap();
    let expected = figure::arg_svg(&arg, ["ha", "na"], &depth).unwrap();
    assert_eq!(Some(expected), figure_text(&r, &opts, Figure::Arg));
  }

  #[test]
  fn figure_text_draws_trees_with_branch_lengths_by_divergence() {
    let (ha, na) = (
      "((A:1,B:1):1,(C:1,(D:1,X:1):1):1);",
      "((A:1,(B:1,X:1):1):1,(C:1,D:1):1);",
    );
    let (r, opts) = (run_trees(&[("ha", ha), ("na", na)]), run_options(2));
    let view = display::pair_view(&r, &opts, 0, TreeVersion::Resolved, Scale::Div).unwrap();
    let expected = figure::tanglegram_svg(&view, &FigureOptions::default()).unwrap();
    assert_eq!(Some(expected), figure_text(&r, &opts, Figure::Pair { pair: 0 }));
  }

  #[test]
  fn figure_text_of_a_missing_pair_or_arg_is_none() {
    let t = "((A,B),(C,D));";
    let r = run_trees(&[("ha", t), ("na", t), ("pb2", t)]);
    let opts = run_options(3);
    assert_eq!(
      (None, None),
      (
        figure_text(&r, &opts, Figure::Pair { pair: 3 }),
        figure_text(&r, &opts, Figure::Arg)
      )
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
  fn request_file_round_trips_through_read_request() {
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
    let file = request_file(&request);
    assert_eq!(
      ("treeknit_request.json", "application/json"),
      (file.path.as_str(), file.media_type.as_str())
    );
    assert!(file.text.ends_with("}\n"));
    assert_eq!(Ok(request), analysis::read_request(&file.text));
  }

  #[test]
  fn command_line_runs_the_session_file_of_the_extracted_archive() {
    assert_eq!(
      "treeknit --request treeknit_results/treeknit_request.json --impute --auspice-view --plot",
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
    // Oracle: APPNOTE.TXT 4.3.12 and 4.4.2: a central directory header starts with
    // `PK\x01\x02`, its byte 5 is the host system of "version made by" (3 for Unix), and bytes
    // 38 to 41 are the external attributes, which hold the Unix mode in their upper 16 bits.
    let bytes = zip_archive(&fixed_files()).unwrap();
    let headers: Vec<(u8, u32)> = bytes
      .windows(4)
      .enumerate()
      .filter(|(_, w)| *w == b"PK\x01\x02")
      .map(|(i, _)| {
        #[expect(clippy::little_endian_bytes, reason = "ZIP headers are little-endian")]
        let attributes = u32::from_le_bytes(bytes[i + 38..i + 42].try_into().unwrap());
        (bytes[i + 5], attributes)
      })
      .collect();
    assert_eq!(vec![(3, 0o100_644 << 16); 3], headers);
  }

  #[test]
  fn zip_archive_of_equal_files_is_byte_identical() {
    assert_eq!(
      zip_archive(&fixed_files()).unwrap(),
      zip_archive(&fixed_files()).unwrap()
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

  #[test]
  fn zip_archive_rejects_a_repeated_path() {
    let files = vec![
      OutputFile::new("MCCs.json".to_owned(), String::new()),
      OutputFile::new("MCCs.json".to_owned(), String::new()),
    ];
    let error = zip_archive(&files).unwrap_err();
    assert!(matches!(error, ArchiveError(ZipError::InvalidArchive(_))));
    assert_eq!(
      "cannot build the ZIP archive: invalid Zip archive: Duplicate filename: treeknit_results/MCCs.json",
      error.to_string()
    );
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
    let entry = FileEntry::new(
      "tanglegram_ha_na.svg".into(),
      "image/svg+xml".into(),
      None,
      Some(Figure::Pair { pair: 2 }),
    );
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
    let entry = FileEntry::new(path.to_owned(), "text/plain".to_owned(), Some(0), None);
    assert_eq!(expected, entry.file_name);
  }
}
