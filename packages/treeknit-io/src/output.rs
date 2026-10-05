//! Output files of a run, as the command line writes them and the web app lists them.

use crate::analysis::AnalysisRequest;
use crate::run::RunResult;
use crate::summary::Diagnostic;
use crate::{analysis, arg, auspice, mccs, newick};
use serde::Serialize;
use serde_json::{Value, json};
use std::io::{Cursor, Write};
use std::path::Path;
use treeknit_core::{Options, Tree};
#[cfg(feature = "tsify")]
use tsify::Tsify;
use zip::result::ZipError;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipWriter};

/// Default results directory of the command line, and the directory of the files in the ZIP
/// archive.
pub const RESULTS_DIR: &str = "treeknit_results";

/// File name of the session file: the analysis request that the web app saves and the command
/// line runs with `--request`.
pub const REQUEST_FILE: &str = "treeknit_request.json";

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
  pub media_type: String,
  /// Size in bytes; `None` for a figure not rendered yet.
  pub size: Option<usize>,
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
}

impl OutputFile {
  /// A file at `path` with its media type taken from the extension.
  pub fn new(path: String, text: String) -> Self {
    let json = Path::new(&path)
      .extension()
      .is_some_and(|e| e.eq_ignore_ascii_case("json"));
    let media_type = if json { "application/json" } else { "text/plain" };
    OutputFile {
      path,
      media_type: media_type.to_owned(),
      text,
    }
  }
}

/// Every output file of `run` except `parameters.json` and `log.txt`, at its path in the results
/// directory, with the bytes the command line writes: the MCCs as JSON and as lines, the
/// resolved trees, the imputed trees and Auspice files when `options` asks for them, and for
/// two trees the ARG files. File names follow the tree labels.
pub fn output_files(run: &RunResult, options: &OutputOptions) -> Vec<OutputFile> {
  let RunResult {
    trees,
    taxa,
    pairs,
    imputed,
    ..
  } = run;
  let ext = |i: usize| options.extensions.get(i).map_or("", String::as_str);
  let tree_file = |dir: &str, suffix: &str, i: usize, t: &Tree| {
    OutputFile::new(
      format!("{dir}{}{suffix}{}", t.label, ext(i)),
      format!("{}\n", newick::write(t)),
    )
  };
  let mut files = vec![OutputFile::new(
    "MCCs.json".to_owned(),
    format!("{:#}\n", mccs::to_json(pairs, trees, taxa)),
  )];
  // Legacy text format of TreeKnit.jl < 0.5 (one MCC per line): `MCCs.dat` for two trees,
  // `MCCs_<a>_<b>.dat` per pair otherwise.
  files.extend(pairs.iter().map(|p| {
    let path = if trees.len() == 2 {
      "MCCs.dat".to_owned()
    } else {
      format!("MCCs_{}.dat", analysis::pair_stem(&trees[p.i].label, &trees[p.j].label))
    };
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
        format!("auspice_{}.json", t.label),
        format!("{:#}", auspice::auspice_json(i, trees, pairs, taxa)),
      )
    }));
  }
  if let Some(a) = run.built_arg() {
    files.push(OutputFile::new(
      "ARG/arg.nwk".to_owned(),
      format!("{}\n", arg::extended_newick(a)),
    ));
    files.push(OutputFile::new(
      "ARG/nodes.dat".to_owned(),
      format!("{}\n", arg::node_table(a)),
    ));
    files.extend(
      a.trees
        .iter()
        .enumerate()
        .map(|(i, t)| tree_file("ARG/", "_liberal_resolved", i, t)),
    );
  }
  files
}

/// `parameters.json`: the core options of a run and its seed. The command line writes it before
/// the inference, so it exists when a run fails.
pub fn parameters_file(o: &Options, seed: u64) -> OutputFile {
  OutputFile::new("parameters.json".to_owned(), format!("{:#}", params_json(o, seed)))
}

/// `log.txt` of the web app: one line `<time> [LEVEL] <message>` per record, the layout of the
/// command-line log without its thread ID, because the browser runs on one thread.
pub fn log_file(records: &[Diagnostic]) -> OutputFile {
  let text = records
    .iter()
    .map(|r| format!("{} [{}] {}\n", r.time, r.level, r.message))
    .collect();
  OutputFile::new("log.txt".to_owned(), text)
}

/// A ZIP archive of `files`, each under `treeknit_results/` at its path. Every entry is deflated
/// and dated 1980-01-01 00:00, the earliest ZIP time, so equal files give a byte-identical
/// archive. Fails when two files have one path.
pub fn zip_archive(files: &[OutputFile]) -> Result<Vec<u8>, ZipError> {
  let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated)
    .last_modified_time(DateTime::DEFAULT);
  let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
  for f in files {
    zip.start_file(format!("{RESULTS_DIR}/{}", f.path), options)?;
    zip.write_all(f.text.as_bytes())?;
  }
  Ok(zip.finish()?.into_inner())
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

/// The command that writes the file set of the web app, run in the directory where its ZIP
/// archive was extracted: the session file in `treeknit_results/`, with imputed trees and
/// Auspice files.
pub fn command_line() -> String {
  format!("treeknit --request {RESULTS_DIR}/{REQUEST_FILE} --impute --auspice-view")
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

  fn all_files(k: usize) -> OutputOptions {
    OutputOptions {
      extensions: vec![".nwk".to_owned(); k],
      imputed: true,
      auspice: true,
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
    let files = output_files(&run_trees(&[("ha", HA), ("na", NA)]), &all_files(2));
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
    let files = output_files(&run_trees(&[("ha", t), ("na", t), ("pb2", t)]), &all_files(3));
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
    let files = output_files(&run_trees(&[("ha", HA), ("na", NA)]), &options);
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
    let files = output_files(&run_trees(&[("ha", HA), ("na", NA)]), &options);
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
  fn output_files_hold_the_mccs_of_the_reference() {
    let files = output_files(&run_trees(&[("ha", HA), ("na", NA)]), &all_files(2));
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
    let files = output_files(
      &run_trees(&[("ha", "((A,B),(C,(D,P)));"), ("na", "((A,B),(C,D));")]),
      &all_files(2),
    );
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
    let files = output_files(&run_trees(&[("ha", HA), ("na", NA)]), &all_files(2));
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
      "treeknit --request treeknit_results/treeknit_request.json --impute --auspice-view",
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
    let entries: Vec<(String, String, Option<DateTime>, CompressionMethod)> = (0..archive.len())
      .map(|i| {
        let mut entry = archive.by_index(i).unwrap();
        let mut text = String::new();
        std::io::Read::read_to_string(&mut entry, &mut text).unwrap();
        (
          entry.name().to_owned(),
          text,
          entry.last_modified(),
          entry.compression(),
        )
      })
      .collect();
    let entry = |name: &str, text: &str| {
      (
        name.to_owned(),
        text.to_owned(),
        Some(DateTime::DEFAULT),
        CompressionMethod::Deflated,
      )
    };
    let expected = vec![
      entry("treeknit_results/MCCs.json", "{}\n"),
      entry("treeknit_results/ARG/arg.nwk", "(A,B);\n"),
      entry("treeknit_results/MCCs.dat", "A,B"),
    ];
    assert_eq!(expected, entries);
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
    assert!(matches!(zip_archive(&files), Err(ZipError::InvalidArchive(_))));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::json(  "MCCs.json",   "application/json")]
  #[case::upper( "A.JSON",      "application/json")]
  #[case::newick("ha.nwk",      "text/plain")]
  #[case::table( "ARG/nodes.dat", "text/plain")]
  #[case::none(  "ha_resolved", "text/plain")]
  #[trace]
  fn output_file_media_type_follows_the_extension(#[case] path: &str, #[case] expected: &str) {
    assert_eq!(expected, OutputFile::new(path.to_owned(), String::new()).media_type);
  }

  #[test]
  fn file_entry_serializes_camel_case_with_null_size() {
    let entry = FileEntry {
      path: "tanglegram_ha_na.svg".into(),
      media_type: "image/svg+xml".into(),
      size: None,
    };
    let expected = json!({"path": "tanglegram_ha_na.svg", "mediaType": "image/svg+xml", "size": null});
    assert_eq!(expected, serde_json::to_value(&entry).unwrap());
  }
}
