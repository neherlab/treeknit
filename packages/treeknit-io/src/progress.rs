//! Progress of a run at the WebAssembly boundary, built from `treeknit_core::Progress`.

use serde::Serialize;
use treeknit_core::progress;
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
  /// Round in progress, 1-based, in the phase `pairs`; the number of rounds in `matching` and
  /// `done`.
  pub round: usize,
  /// Number of rounds, including the final round without resolution.
  pub rounds: usize,
  /// Pair in progress, 1-based, in pipeline order, in the phase `pairs`; the number of pairs in
  /// `matching` and `done`.
  pub pair: usize,
  /// Number of pairs.
  pub pairs: usize,
}

impl From<progress::Phase> for Phase {
  fn from(p: progress::Phase) -> Phase {
    match p {
      progress::Phase::Pairs => Phase::Pairs,
      progress::Phase::Matching => Phase::Matching,
      progress::Phase::Done => Phase::Done,
    }
  }
}

impl From<progress::Progress> for Progress {
  fn from(p: progress::Progress) -> Progress {
    // Destructured without `..`, so a new field of the core type stops the build here.
    let progress::Progress {
      phase,
      fraction,
      round,
      rounds,
      pair,
      pairs,
    } = p;
    Progress {
      phase: phase.into(),
      fraction,
      round,
      rounds,
      pair,
      pairs,
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
    let p = progress::Progress {
      phase: progress::Phase::Matching,
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
