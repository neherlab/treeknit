//! Progress of a run at the WebAssembly boundary, built from `treeknit_core::Progress`.

use serde::Serialize;
use treeknit_core::progress as core;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// Part of a run that is in progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "lowercase")]
pub enum Phase {
  /// Pair inference and resolution, in rounds over all pairs.
  Pairs,
  /// Topology matching of `matched` resolution; the fraction does not change during it.
  Matching,
  /// The run is complete.
  Done,
}

/// Progress of a run: the completed fraction, and the round and pair in progress.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Progress {
  pub phase: Phase,
  /// Completed fraction of the run, from 0 to 1, never decreasing during a run.
  pub fraction: f64,
  /// Round in progress, 1-based.
  pub round: usize,
  /// Number of rounds, including the final round without resolution.
  pub rounds: usize,
  /// Pair in progress, 1-based, in pipeline order.
  pub pair: usize,
  /// Number of pairs.
  pub pairs: usize,
}

impl From<core::Phase> for Phase {
  fn from(p: core::Phase) -> Phase {
    match p {
      core::Phase::Pairs => Phase::Pairs,
      core::Phase::Matching => Phase::Matching,
      core::Phase::Done => Phase::Done,
    }
  }
}

impl From<core::Progress> for Progress {
  fn from(p: core::Progress) -> Progress {
    Progress {
      phase: p.phase.into(),
      fraction: p.fraction,
      round: p.round,
      rounds: p.rounds,
      pair: p.pair,
      pairs: p.pairs,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use serde_json::json;

  #[test]
  fn progress_from_core_serializes_phase_as_lowercase_string() {
    let p = core::Progress {
      phase: core::Phase::Matching,
      fraction: 0.5,
      round: 1,
      rounds: 1,
      pair: 1,
      pairs: 1,
    };
    let expected = json!({"phase": "matching", "fraction": 0.5, "round": 1, "rounds": 1, "pair": 1, "pairs": 1});
    assert_eq!(expected, serde_json::to_value(Progress::from(p)).unwrap());
  }
}
