//! Output files of a run, as the command line writes them and the web app lists them.

mod archive;
mod figures;
mod kinds;

pub use archive::{ARCHIVE_MEDIA_TYPE, Archive, ArchiveError};
pub use figures::{AuspiceFiles, FigureDownload, arg_figure, auspice_files, figure_text, pair_figure};
pub use kinds::{FileReference, MAX_FILE_NAME_BYTES, auspice_path, check_output_paths, file_reference};

use crate::analysis::AnalysisRequest;
use crate::newick;
use crate::run::RunResult;
use crate::summary::Diagnostic;
use kinds::{FileKind, run_files};
use serde::Serialize;
use serde_json::{Value, json};
use std::fmt::Write as _;
use std::path::Path;
use treeknit_core::Options;
#[cfg(feature = "tsify")]
use tsify::Tsify;

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

/// Results directory of `command_line`, next to the extracted `treeknit_results/`, so that the
/// command keeps the files it reproduces and the two sets can be compared.
pub const COMMAND_LINE_RESULTS_DIR: &str = "treeknit_results_cli";

/// Every output file of `run` except `parameters.json` and `log.txt`, at its path in the results
/// directory, with the bytes the command line writes: the MCCs as JSON and as lines, the
/// resolved trees, the imputed trees and Auspice files when `options` asks for them, for two
/// trees the ARG files, and the figures when `options` asks for them. File names follow the tree
/// labels. The run of a request that passed `analysis::prepare` has every file it lists, so
/// `FileError::Missing` marks a broken run.
pub fn output_files(run: &RunResult, options: &OutputOptions) -> Result<Vec<OutputFile>, FileError> {
  run_files(run, options)
    .into_iter()
    .map(|(kind, path)| {
      let text = kind.text(run, &path)?;
      Ok(OutputFile::new(path, text))
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
) -> Result<Vec<WebFile>, FileError> {
  let mut files = vec![WebFile::Text(session_file(request))];
  for (kind, path) in run_files(run, &OutputOptions::web(run.trees.len())) {
    files.push(if let FileKind::Figure { figure, .. } = kind {
      WebFile::Figure(FigureFile { path, figure })
    } else {
      let text = kind.text(run, &path)?;
      WebFile::Text(OutputFile::new(path, text))
    });
  }
  files.push(WebFile::Text(parameters_file(&run.opts, seed)));
  files.push(WebFile::Text(log_file(records)));
  Ok(files)
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

/// The command that writes the file set of the web app (`web_files`), run in the directory where
/// its ZIP archive was extracted: the session file in `treeknit_results/`, the results directory
/// `treeknit_results_cli/`, and the flags of `OutputOptions::web`.
pub fn command_line() -> String {
  let flags = OutputOptions::web(0).flags().join(" ");
  format!("treeknit --session {RESULTS_DIR}/{SESSION_FILE} --outdir {COMMAND_LINE_RESULTS_DIR} {flags}")
}

/// The names of the result files, with the Auspice files of the trees labeled `labels`.
pub fn result_names(labels: [&str; 2]) -> ResultNames {
  ResultNames {
    archive: format!("{RESULTS_DIR}.zip"),
    archive_media_type: ARCHIVE_MEDIA_TYPE.to_owned(),
    results_dir: RESULTS_DIR.to_owned(),
    command_line_results_dir: COMMAND_LINE_RESULTS_DIR.to_owned(),
    arg_newick: ARG_NEWICK.to_owned(),
    auspice_files: labels.map(auspice_path),
  }
}

/// The labels of the two trees of an ARG, segment A and then B; `None` for another number of
/// trees.
pub(crate) fn segment_labels(run: &RunResult) -> Option<[&str; 2]> {
  match run.trees.as_slice() {
    [a, b] => Some([a.label.as_str(), b.label.as_str()]),
    _ => None,
  }
}

/// The names of the result files that the web app downloads and its help texts name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ResultNames {
  /// File name of the ZIP archive of `results_dir`.
  pub archive: String,
  pub archive_media_type: String,
  pub results_dir: String,
  /// Results directory of `command_line`.
  pub command_line_results_dir: String,
  /// Path of the ARG in extended Newick, written for two trees.
  pub arg_newick: String,
  /// Paths of the Auspice JSON files of the two trees of a pair.
  pub auspice_files: [String; 2],
}

/// A file of the file set of the web app: a text, or a figure that is rendered when it is read,
/// because a run of many large trees has figures of many megabytes that most sessions never read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebFile {
  Text(OutputFile),
  Figure(FigureFile),
}

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

/// A figure of a run at its path in the results directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FigureFile {
  pub path: String,
  pub figure: Figure,
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
  /// Write the SVG figures (`--plot`): the tanglegram of each pair, `tanglegram_<a>_<b>.svg`, then
  /// for two trees with a built ARG `ARG/arg.svg`.
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

/// A listed file of a run whose text cannot be made.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum FileError {
  /// The run lacks the pair or the ARG of the file.
  #[error("the run has no data for the output file {path}")]
  Missing { path: String },
  /// Newick cannot hold a tree of the file, such as a branch length that is not a finite number.
  #[error("cannot write the output file {path}: {error}")]
  Newick { path: String, error: newick::WriteError },
}

/// The media type of an output file at `path`, from its extension: JSON, SVG, or plain text.
fn media_type(path: &str) -> &'static str {
  match Path::new(path).extension().and_then(|e| e.to_str()) {
    Some(e) if e.eq_ignore_ascii_case("json") => "application/json",
    Some(e) if e.eq_ignore_ascii_case("svg") => "image/svg+xml",
    _ => "text/plain",
  }
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
  use crate::summary::Level;
  use crate::test_support::{HA, NA, clades, run_trees};
  use indoc::indoc;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::json;

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
        .map(|(label, newick)| TreeText::new(*label, *newick))
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
  #[case::mccs_json("MCCs.json",                   true)]
  #[case::mccs_dat( "MCCs.dat",                    false)]
  #[case::resolved( "ha_resolved.nwk",             true)]
  #[case::imputed(  "ha_imputed.nwk",              true)]
  #[case::auspice(  "auspice_ha.json",             false)]
  #[case::arg(      "ARG/arg.nwk",                 true)]
  #[case::nodes(    "ARG/nodes.dat",               true)]
  #[case::liberal(  "ARG/ha_liberal_resolved.nwk", true)]
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
      trees: vec![TreeText::new("ha", HA), TreeText::new("na", NA)],
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
  fn result_names_name_the_archive_and_the_files_of_the_help_texts() {
    // Oracle: the results directory `treeknit_results` (the default `--outdir`), zipped; the
    // command-line directory and the ARG path as the command line writes them; one Auspice file
    // per label.
    let expected = ResultNames {
      archive: "treeknit_results.zip".to_owned(),
      archive_media_type: "application/zip".to_owned(),
      results_dir: "treeknit_results".to_owned(),
      command_line_results_dir: "treeknit_results_cli".to_owned(),
      arg_newick: "ARG/arg.nwk".to_owned(),
      auspice_files: ["auspice_<a>.json".to_owned(), "auspice_ha.json".to_owned()],
    };
    assert_eq!(expected, result_names(["<a>", "ha"]));
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
      text: indoc! {"
        2026-10-05T12:00:00.000Z [INFO] TreeKnit 1.0
        2026-10-05T12:00:00.000Z [WARN] ignoring x
      "}
      .to_owned(),
    };
    assert_eq!(expected, file);
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
