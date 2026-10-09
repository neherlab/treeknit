//! The kinds of output files: their order in the file set of a run, their paths, their texts, and
//! their descriptions.

use crate::analysis::{self, ARG_TREE_COUNT, Field, ValidationError};
use crate::output::{
  ARG_FIGURE, ARG_NEWICK, ARG_NODES, Figure, FileError, LOG_FILE, MCCS_JSON, OutputOptions, PARAMETERS_FILE,
  SESSION_FILE, figure_text,
};
use crate::run::RunResult;
use crate::{arg, auspice, mccs, newick};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use treeknit_core::Tree;

/// Labels of the trees of `file_reference`: a placeholder for any tree, and placeholders for the
/// trees of a pair, with a third tree for the names that depend on the number of trees.
const TREE_PLACEHOLDER: [&str; 1] = ["<tree>"];

const PAIR_PLACEHOLDERS: [&str; 3] = ["<a>", "<b>", "<c>"];

/// The files of `run` with `options`, each at its path with the kind of file it is.
pub(crate) fn run_files(run: &RunResult, options: &OutputOptions) -> Vec<(FileKind, String)> {
  let labels = run_labels(run);
  file_kinds(labels.len(), options, run.built_arg().is_some())
    .into_iter()
    .map(|kind| (kind, kind.path(&labels, options)))
    .collect()
}

/// The labels of the trees of `run`.
fn run_labels(run: &RunResult) -> Vec<&str> {
  run.trees.iter().map(|t| t.label.as_str()).collect()
}
/// The output files of the command line for its documentation: one entry per kind of file, in
/// the order of `output_files`, then `parameters.json`, `log.txt`, and the session file. Paths
/// hold the placeholders `<tree>` for the label of a tree and `<a>`, `<b>` for the labels of the
/// trees of a pair; tree files have the extension `.nwk`.
pub fn file_reference() -> Vec<FileReference> {
  let options = OutputOptions::web(PAIR_PLACEHOLDERS.len());
  let run_files = [
    (PARAMETERS_FILE, "The options and the seed of the run"),
    (LOG_FILE, "The log of the run"),
    (
      SESSION_FILE,
      "With `--session`, `--example`, `--link`, or `--print-link`: the trees and settings that ran, \
       which the web app opens and `--session` runs",
    ),
  ];
  reference_kinds()
    .into_iter()
    .map(|(kind, label_sets)| FileReference {
      paths: label_sets
        .into_iter()
        .map(|labels| kind.path(labels, &options))
        .collect(),
      description: kind.description(),
    })
    .chain(run_files.map(|(path, description)| FileReference {
      paths: vec![path.to_owned()],
      description,
    }))
    .collect()
}

/// One file of each kind for `file_reference`, with the label sets of its paths.
fn reference_kinds() -> Vec<(FileKind, Vec<&'static [&'static str]>)> {
  let two: &[&str] = &PAIR_PLACEHOLDERS[..ARG_TREE_COUNT];
  let more: &[&str] = &PAIR_PLACEHOLDERS;
  let tree: &[&str] = &TREE_PLACEHOLDER;
  let pair = Figure::Pair { pair: 0 };
  vec![
    (FileKind::Mccs, vec![two]),
    (FileKind::MccLines { pair: 0, i: 0, j: 1 }, vec![two, more]),
    (FileKind::Resolved { tree: 0 }, vec![tree]),
    (FileKind::Imputed { tree: 0 }, vec![tree]),
    (FileKind::Auspice { tree: 0 }, vec![tree]),
    (FileKind::ArgNewick, vec![two]),
    (FileKind::ArgNodes, vec![two]),
    (FileKind::ArgTree { tree: 0 }, vec![tree]),
    (
      FileKind::Figure {
        figure: pair,
        i: 0,
        j: 1,
      },
      vec![two],
    ),
    (
      FileKind::Figure {
        figure: Figure::Arg,
        i: 0,
        j: 1,
      },
      vec![two],
    ),
  ]
}

/// An entry of `file_reference`: the paths of one kind of output file and what it holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileReference {
  /// The paths of the file, with placeholders for the tree labels; a file whose name depends on
  /// the number of trees has a path for two trees and one for more.
  pub paths: Vec<String>,
  /// What the file holds, and the runs and options that write it.
  pub description: &'static str,
}

/// Longest file name of the file systems of the release targets, in bytes: 255 bytes in ext4,
/// APFS, and NTFS (where it is 255 UTF-16 units, which UTF-8 bytes bound from above).
const MAX_FILE_NAME_BYTES: usize = 255;

/// Output file names that a file system cannot hold: names longer than `MAX_FILE_NAME_BYTES`,
/// such as `tanglegram_<a>_<b>.svg` of two long labels, and files of different trees or kinds
/// that would get the same name, ignoring case as the label check does: the resolved tree of
/// `MCCs_a` from `MCCs_a.dat` and the MCCs of the pair (`a`, `resolved`) are both
/// `MCCs_a_resolved.dat`. The command line would fail after the inference or overwrite one file
/// with the other, and the ZIP archive of the web app cannot hold both. A clash needs a tree
/// extension of the command line, such as `.dat`: the file set of the web app gives each kind of
/// file its own prefix, suffix, extension, or folder. `labels` must have passed the checks of
/// `analysis::parse_trees`, which reports repeated labels and pair names.
pub(crate) fn check_output_paths(labels: &[String], options: &OutputOptions) -> Vec<ValidationError> {
  let error = |message: String| ValidationError::at(Field::Trees, message);
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

/// Every path that a run of the trees labeled `labels` can write with `options`, in the order of
/// `output_files`, followed by `parameters.json`, `log.txt`, and the session file. For two trees
/// the ARG files are listed, although a run writes them only when the ARG is built.
fn output_paths(labels: &[String], options: &OutputOptions) -> Vec<String> {
  let labels: Vec<&str> = labels.iter().map(String::as_str).collect();
  let mut paths: Vec<String> = file_kinds(labels.len(), options, labels.len() == ARG_TREE_COUNT)
    .into_iter()
    .map(|kind| kind.path(&labels, options))
    .collect();
  paths.extend([PARAMETERS_FILE, LOG_FILE, SESSION_FILE].map(str::to_owned));
  paths
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

/// A file of the output set, by what it holds; `file_kinds` lists them in the order of the set,
/// and `FileKind::path` and `FileKind::text` make their paths and texts, so the paths of a run
/// that has not happened yet and the files of a finished run follow one list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FileKind {
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

  /// What the file holds, and the runs and options that write it, for `file_reference`.
  fn description(self) -> &'static str {
    match self {
      FileKind::Mccs => {
        "The MCCs of every pair, in the `MCC_dict` format of TreeKnit.jl. When leaves are missing \
         from one tree of a pair, the list `imputed` gives each such leaf, its source tree, the \
         index of the MCC it joined, and whether the placement was ambiguous"
      },
      FileKind::MccLines { .. } => {
        "The MCCs of a pair in the text format of TreeKnit.jl before 0.5: one MCC per line, leaves \
         separated by commas. `MCCs.dat` for two trees, one file per pair for more"
      },
      FileKind::Resolved { .. } => {
        "The tree after resolution, with polytomies sorted for tanglegrams. Tree files keep the \
         extension of their input file"
      },
      FileKind::Imputed { .. } => {
        "With `--impute`: the resolved tree with the leaves that only other trees have, placed by \
         imputation"
      },
      FileKind::Auspice { .. } => {
        "With `--auspice-view`: the resolved tree in the JSON format of Auspice, colored by MCC, \
         for tanglegrams in Auspice"
      },
      FileKind::ArgNewick => {
        "Two trees: the ARG in extended Newick, with `[&segments={0,1}]` annotations and the \
         hybrid nodes `#H<i>`"
      },
      FileKind::ArgNodes => "Two trees: the table of the ARG nodes and the tree nodes they stand for",
      FileKind::ArgTree { .. } => "Two trees: the liberally resolved trees that the ARG was built from",
      FileKind::Figure {
        figure: Figure::Pair { .. },
        ..
      } => {
        "With `--plot`: the tanglegram of the resolved trees of a pair, with MCC colors and the \
         reassortment branches marked"
      },
      FileKind::Figure {
        figure: Figure::Arg, ..
      } => {
        "With `--plot`, two trees with a built ARG: the ARG, colored by segment, with \
         reassortments as dashed curves"
      },
    }
  }

  /// The text of the file of `run` at `path`, with the bytes the command line writes; an error
  /// when `run` lacks the pair or the ARG of the file, or when Newick cannot hold one of its trees.
  pub(crate) fn text(self, run: &RunResult, path: &str) -> Result<String, FileError> {
    let RunResult {
      trees,
      taxa,
      pairs,
      imputed,
      ..
    } = run;
    let missing = || FileError::Missing { path: path.to_owned() };
    let newick_error = |error| FileError::Newick {
      path: path.to_owned(),
      error,
    };
    let newick = |t: Option<&Tree>| {
      let text = newick::write(t.ok_or_else(missing)?).map_err(newick_error)?;
      Ok(format!("{text}\n"))
    };
    let arg = || run.built_arg().ok_or_else(missing);
    Ok(match self {
      FileKind::Mccs => format!("{:#}\n", mccs::to_json(pairs, trees, taxa)),
      FileKind::MccLines { pair, .. } => {
        let pair = pairs.get(pair).ok_or_else(missing)?;
        let names: Vec<Vec<String>> = pair.mccs.iter().map(|m| taxa.names_of(m)).collect();
        mccs::to_lines(&names)
      },
      FileKind::Resolved { tree } => newick(trees.get(tree))?,
      FileKind::Imputed { tree } => newick(imputed.get(tree))?,
      FileKind::Auspice { tree } => auspice::auspice_json(tree, trees, pairs, taxa),
      FileKind::ArgNewick => format!("{}\n", arg::extended_newick(arg()?).map_err(newick_error)?),
      FileKind::ArgNodes => format!("{}\n", arg::node_table(arg()?)),
      FileKind::ArgTree { tree } => newick(arg()?.trees.get(tree))?,
      FileKind::Figure { figure, .. } => figure_text(run, figure).ok_or_else(missing)?,
    })
  }
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
pub(crate) fn auspice_path(label: &str) -> String {
  format!("auspice_{label}.json")
}

/// Path of the tanglegram figure of the pair of trees labeled `a` and `b`.
pub(crate) fn tanglegram_path(a: &str, b: &str) -> String {
  format!("tanglegram_{}.svg", analysis::pair_stem(a, b))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::output::output_files;
  use crate::test_support::{HA, NA, run_trees};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use std::collections::BTreeSet;

  #[test]
  fn file_reference_lists_one_entry_per_kind_of_file_with_placeholder_names() {
    let actual: Vec<Vec<String>> = file_reference().into_iter().map(|r| r.paths).collect();
    let expected = vec![
      vec!["MCCs.json"],
      vec!["MCCs.dat", "MCCs_<a>_<b>.dat"],
      vec!["<tree>_resolved.nwk"],
      vec!["<tree>_imputed.nwk"],
      vec!["auspice_<tree>.json"],
      vec!["ARG/arg.nwk"],
      vec!["ARG/nodes.dat"],
      vec!["ARG/<tree>_liberal_resolved.nwk"],
      vec!["tanglegram_<a>_<b>.svg"],
      vec!["ARG/arg.svg"],
      vec!["parameters.json"],
      vec!["log.txt"],
      vec!["treeknit_session.json"],
    ];
    assert_eq!(expected, actual);
  }

  #[rstest]
  #[case::two_trees(2)]
  #[case::three_trees(3)]
  #[trace]
  fn file_reference_describes_every_file_that_a_run_writes(#[case] k: usize) {
    let described: BTreeSet<&str> = reference_kinds()
      .into_iter()
      .map(|(kind, _)| kind.description())
      .collect();
    let written: BTreeSet<&str> = file_kinds(k, &OutputOptions::web(k), true)
      .into_iter()
      .map(FileKind::description)
      .collect();
    assert!(
      written.is_subset(&described),
      "undescribed: {:?}",
      written.difference(&described)
    );
  }

  fn labels(v: &[&str]) -> Vec<String> {
    v.iter().map(|&l| l.to_owned()).collect()
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::two_trees(  &[("ha", HA), ("na", NA)])]
  #[case::three_trees(&[("ha", "((A,B),(C,D));"), ("na", "((A,B),(C,D));"), ("pb2", "((A,B),(C,D));")])]
  #[trace]
  fn output_paths_list_every_file_that_a_run_writes(#[case] trees: &[(&str, &str)]) {
    let fixed = [PARAMETERS_FILE, LOG_FILE, SESSION_FILE];
    let options = OutputOptions {
      extensions: vec![".tree".to_owned(); trees.len()],
      ..OutputOptions::web(trees.len())
    };
    let written = output_files(&run_trees(trees), &options).unwrap();
    let expected: Vec<&str> = written.iter().map(|f| f.path.as_str()).chain(fixed).collect();
    let names: Vec<String> = trees.iter().map(|(l, _)| (*l).to_owned()).collect();
    assert_eq!(expected, output_paths(&names, &options));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::pair_and_tree(   &["a", "resolved", "MCCs_a"], &[".nwk", ".nwk", ".dat"], "two output files are named \"MCCs_a_resolved.dat\"; rename a tree")]
  #[case::auspice_and_tree(&["auspice_x", "x_resolved"], &[".json", ".nwk"],       "two output files are named \"auspice_x_resolved.json\"; rename a tree")]
  #[case::case_only(       &["a", "Resolved", "mccs_a"], &[".nwk", ".nwk", ".dat"], "the output files \"MCCs_a_Resolved.dat\" and \"mccs_a_resolved.dat\" differ only in case; rename a tree")]
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
    let expected = vec![ValidationError::at(Field::Trees, message)];
    assert_eq!(expected, check_output_paths(&labels(names), &options));
  }

  #[test]
  fn check_output_paths_reports_file_names_longer_than_a_file_system_holds() {
    let (a, b) = ("a".repeat(125), "b".repeat(125));
    let error = |message: String| ValidationError::at(Field::Trees, message);
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
}
