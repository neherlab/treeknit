//! Summary of a run: the MCCs of each pair, the ARG outcome, and the diagnostics.

use serde::Serialize;
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
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::json;

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
