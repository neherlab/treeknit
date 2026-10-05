//! Summary of a run: the MCCs of each pair, the ARG outcome, and the diagnostics.

use crate::run::RunResult;
use serde::Serialize;
use std::fmt;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// Results of a run at a glance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Summary {
  /// Pairs in pipeline order (0,1), (0,2), ..., (1,2), ...
  pub pairs: Vec<PairSummary>,
  /// Outcome of the ARG; `None` for more than two trees.
  pub arg: Option<ArgOutcome>,
  /// Warnings and errors of the run, in the order they occurred.
  pub diagnostics: Vec<Diagnostic>,
}

impl Summary {
  /// The summary of `run`, with the `diagnostics` of the run.
  pub fn new(run: &RunResult, diagnostics: Vec<Diagnostic>) -> Summary {
    let RunResult { trees, taxa, pairs, .. } = run;
    let pairs = pairs
      .iter()
      .enumerate()
      .map(|(index, p)| PairSummary {
        index,
        labels: [trees[p.i].label.clone(), trees[p.j].label.clone()],
        mcc_count: p.mccs.len(),
        mccs: p.mccs.iter().map(|m| taxa.names_of(m)).collect(),
        // Counted per leaf, as the `imputed` entries of `MCCs.json`.
        imputed_count: p.attached.iter().map(|a| a.leaves.len()).sum(),
        ambiguous_count: p.attached.iter().filter(|a| a.ambiguous).map(|a| a.leaves.len()).sum(),
      })
      .collect();
    Summary {
      pairs,
      arg: run.arg_outcome(),
      diagnostics,
    }
  }
}

/// MCCs of one pair of trees.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct PairSummary {
  /// Index of the pair in pipeline order.
  pub index: usize,
  /// Labels of the two trees.
  pub labels: [String; 2],
  /// Number of MCCs.
  pub mcc_count: usize,
  /// The MCCs as leaf-name lists, as in `MCCs.json`.
  pub mccs: Vec<Vec<String>>,
  /// Number of leaves in one tree of the pair only, attached to an MCC of the pair.
  pub imputed_count: usize,
  /// Number of these attached leaves whose attachment is ambiguous.
  pub ambiguous_count: usize,
}

/// Outcome of building the ARG of two trees.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ArgOutcome {
  Built {
    /// Number of reassortment (hybrid) nodes.
    reassortments: usize,
  },
  Failed {
    message: String,
  },
}

/// A log record of a run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
  pub level: Level,
  pub message: String,
  /// RFC 3339 time of the record.
  pub time: String,
}

/// Severity of a diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "lowercase")]
pub enum Level {
  Error,
  Warn,
  Info,
  Debug,
}

impl fmt::Display for Level {
  /// The upper-case name of the level, as `log::Level` writes it in the command-line log.
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(match self {
      Level::Error => "ERROR",
      Level::Warn => "WARN",
      Level::Info => "INFO",
      Level::Debug => "DEBUG",
    })
  }
}

impl From<log::Level> for Level {
  /// Trace records count as debug; the log capture keeps debug and above.
  fn from(l: log::Level) -> Level {
    match l {
      log::Level::Error => Level::Error,
      log::Level::Warn => Level::Warn,
      log::Level::Info => Level::Info,
      log::Level::Debug | log::Level::Trace => Level::Debug,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::analysis::{self, Settings, TreeText};
  use crate::run;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::json;

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

  fn names(v: &[&str]) -> Vec<String> {
    v.iter().map(|&x| x.to_owned()).collect()
  }

  fn warning() -> Diagnostic {
    Diagnostic {
      level: Level::Warn,
      message: "m".into(),
      time: "2026-01-01T00:00:00Z".into(),
    }
  }

  #[test]
  fn summary_new_of_the_two_tree_example_matches_the_reference() {
    let r = run_trees(&[("ha", "((A,B),(C,(D,X)));"), ("na", "((A,(B,X)),(C,D));")]);
    // Oracle: fixtures/doc_mccs_1.json (TreeKnit.jl): MCCs [X] and [A,B,C,D], one reassortment.
    let expected = Summary {
      pairs: vec![PairSummary {
        index: 0,
        labels: ["ha".into(), "na".into()],
        mcc_count: 2,
        mccs: vec![names(&["X"]), names(&["A", "B", "C", "D"])],
        imputed_count: 0,
        ambiguous_count: 0,
      }],
      arg: Some(ArgOutcome::Built { reassortments: 1 }),
      diagnostics: vec![warning()],
    };
    assert_eq!(expected, Summary::new(&r, vec![warning()]));
  }

  #[test]
  fn summary_new_of_three_trees_lists_every_pair_without_arg() {
    let t = "((A,B),(C,D));";
    let s = Summary::new(&run_trees(&[("ha", t), ("na", t), ("pb2", t)]), Vec::new());
    let pairs: Vec<(usize, [String; 2])> = s.pairs.iter().map(|p| (p.index, p.labels.clone())).collect();
    let expected = vec![
      (0, ["ha".into(), "na".into()]),
      (1, ["ha".into(), "pb2".into()]),
      (2, ["na".into(), "pb2".into()]),
    ];
    assert_eq!(expected, pairs);
    assert_eq!(None, s.arg);
  }

  #[test]
  fn summary_new_counts_imputed_and_ambiguous_leaves() {
    let mut r = run_trees(&[("ha", "((A,B),(C,(D,P)));"), ("na", "((A,B),(C,D));")]);
    // Oracle: P is in ha only and attaches unambiguously to the MCC [A,B,C,D] (see the
    // `imputed` entry of `output_files_place_a_leaf_missing_from_one_tree`).
    let mut extra = r.pairs[0].attached[0].clone();
    extra.leaves = vec![0, 1];
    extra.ambiguous = true;
    r.pairs[0].attached.push(extra);
    let pair = &Summary::new(&r, Vec::new()).pairs[0];
    assert_eq!((3, 2), (pair.imputed_count, pair.ambiguous_count));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::error(Level::Error, log::Level::Error)]
  #[case::warn( Level::Warn,  log::Level::Warn)]
  #[case::info( Level::Info,  log::Level::Info)]
  #[case::debug(Level::Debug, log::Level::Debug)]
  #[trace]
  fn summary_level_displays_as_the_log_crate(#[case] level: Level, #[case] log_level: log::Level) {
    // Oracle: the Display of `log::Level`, which the command-line log writes in brackets.
    assert_eq!(log_level.to_string(), level.to_string());
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::error(log::Level::Error, Level::Error)]
  #[case::warn( log::Level::Warn,  Level::Warn)]
  #[case::info( log::Level::Info,  Level::Info)]
  #[case::debug(log::Level::Debug, Level::Debug)]
  #[case::trace(log::Level::Trace, Level::Debug)]
  #[trace]
  fn summary_level_from_log_level_counts_trace_as_debug(#[case] level: log::Level, #[case] expected: Level) {
    // Oracle: the doc of `From<log::Level> for Level`: each level maps to its namesake, and
    // trace records count as debug.
    assert_eq!(expected, Level::from(level));
  }

  #[test]
  fn summary_serializes_arg_outcome_tagged_by_status() {
    let summary = Summary {
      pairs: vec![PairSummary {
        index: 0,
        labels: ["ha".into(), "na".into()],
        mcc_count: 2,
        mccs: vec![vec!["X".into()], vec!["A".into(), "B".into(), "C".into(), "D".into()]],
        imputed_count: 0,
        ambiguous_count: 0,
      }],
      arg: Some(ArgOutcome::Built { reassortments: 1 }),
      diagnostics: vec![Diagnostic {
        level: Level::Warn,
        message: "m".into(),
        time: "2026-01-01T00:00:00Z".into(),
      }],
    };
    let expected = json!({
      "pairs": [{
        "index": 0, "labels": ["ha", "na"], "mccCount": 2, "mccs": [["X"], ["A", "B", "C", "D"]],
        "imputedCount": 0, "ambiguousCount": 0,
      }],
      "arg": {"status": "built", "reassortments": 1},
      "diagnostics": [{"level": "warn", "message": "m", "time": "2026-01-01T00:00:00Z"}],
    });
    assert_eq!(expected, serde_json::to_value(&summary).unwrap());
  }

  #[test]
  fn arg_outcome_failed_carries_its_message() {
    let expected = json!({"status": "failed", "message": "no ARG"});
    let failed = ArgOutcome::Failed {
      message: "no ARG".into(),
    };
    assert_eq!(expected, serde_json::to_value(&failed).unwrap());
  }
}
