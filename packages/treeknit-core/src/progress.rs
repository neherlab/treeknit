//! Progress of a TreeKnit run, reported to an observer of `pipeline::run_observed`.
//!
//! The completed fraction of a run is `PAIRS_SHARE * (r + (p + within) / pairs) / rounds`, where
//! `r` is the number of completed rounds, `p` the number of completed pairs of the current round,
//! and `within` the completed fraction of the current pair (see `iterations_done`). In
//! [`Phase::Pairs`], the `round` and `pair` of a [`Progress`] are 1-based: they name the round
//! and pair in progress, so they are `r + 1` and `p + 1`; in [`Phase::Matching`] and
//! [`Phase::Done`] they are the numbers of rounds and pairs. The work after the last round
//! (topology matching, sorting, attachment) reports no intermediate progress, so it gets the rest
//! up to 1, which only the end of the run reports.
//!
//! [`PAIRS_SHARE`] is a display heuristic, not an estimate of time: the fraction is not
//! proportional to elapsed or remaining time. The work after the last round can take most of a
//! run (topology matching on large trees) or almost none of it, so the fraction can stay at its
//! value for a long time and then jump to 1. A consumer that shows the fraction should show an
//! indeterminate indicator during [`Phase::Matching`] instead.

/// Share of a run's fraction that pair inference and resolution cover: the fraction at the end
/// of the last pair. It is below 1 so that a fraction of 1 always means that the run is done.
pub const PAIRS_SHARE: f64 = 0.95;

/// Part of a run that is in progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
  /// Pair inference and resolution with MCCs, in rounds over all pairs.
  Pairs,
  /// Matching of topologies within MCCs (`Resolution::Matched`), after the last round. Its
  /// duration is not known in advance, so the fraction does not change during it.
  Matching,
  /// The run is complete.
  Done,
}

/// Progress of a run: the completed fraction of the work, and the round and pair in progress.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Progress {
  /// Part of the run in progress.
  pub phase: Phase,
  /// Completed fraction of the run, from 0 to 1, never decreasing during a run. It is at most
  /// [`PAIRS_SHARE`] before [`Phase::Done`], and 1 at `Done`.
  pub fraction: f64,
  /// Round in progress, 1-based.
  pub round: usize,
  /// Number of rounds, including the final round without resolution.
  pub rounds: usize,
  /// Pair in progress, 1-based, in pipeline order (0,1), (0,2), ...
  pub pair: usize,
  /// Number of pairs.
  pub pairs: usize,
}

impl Progress {
  /// Progress in pair `pair` of round `round` (both zero-based), of which `within` (0 to 1) is
  /// complete.
  #[expect(
    clippy::as_conversions,
    reason = "round and pair counts are far below 2^53, so their conversion to f64 is exact"
  )]
  pub(crate) fn at(round: usize, rounds: usize, pair: usize, pairs: usize, within: f64) -> Progress {
    let fraction = PAIRS_SHARE * ((round as f64 + (pair as f64 + within) / pairs as f64) / rounds as f64);
    Progress {
      phase: Phase::Pairs,
      fraction,
      round: round + 1,
      rounds,
      pair: pair + 1,
      pairs,
    }
  }

  /// Progress at the start of topology matching, after all rounds, with the `fraction` reached.
  pub(crate) fn matching(fraction: f64, rounds: usize, pairs: usize) -> Progress {
    Progress {
      phase: Phase::Matching,
      fraction,
      round: rounds,
      rounds,
      pair: pairs,
      pairs,
    }
  }

  /// Progress of a finished run.
  pub(crate) fn done(rounds: usize, pairs: usize) -> Progress {
    Progress {
      phase: Phase::Done,
      fraction: 1.0,
      round: rounds,
      rounds,
      pair: pairs,
      pairs,
    }
  }
}

/// Completed fraction of `total` units of work after `done` units.
#[expect(
  clippy::as_conversions,
  reason = "step counts are far below 2^53, so their conversion to f64 is exact"
)]
pub(crate) fn ratio(done: usize, total: usize) -> f64 {
  done as f64 / total as f64
}

/// Completed fraction of pair inference after `iteration` (zero-based) complete iterations and
/// `step` (0 to 1) of the next one, out of at most `iterations`. Inference that stops before its
/// last iteration never reaches 1; the bound keeps the fraction at most 1.
#[expect(
  clippy::as_conversions,
  reason = "iteration counts are far below 2^53, so their conversion to f64 is exact"
)]
pub(crate) fn iterations_done(iteration: usize, step: f64, iterations: usize) -> f64 {
  ((iteration as f64 + step) / iterations as f64).min(1.0)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn progress_at_counts_completed_rounds_pairs_and_work_within_the_pair() {
    // (round + (pair + within) / pairs) / rounds = (1 + (1 + 0.5) / 3) / 2 = 0.75 of the pairs.
    let expected = Progress {
      phase: Phase::Pairs,
      fraction: PAIRS_SHARE * 0.75,
      round: 2,
      rounds: 2,
      pair: 2,
      pairs: 3,
    };
    assert_eq!(expected, Progress::at(1, 2, 1, 3, 0.5));
  }

  #[test]
  fn progress_at_end_of_last_pair_is_the_pairs_share() {
    // (1 + (2 + 1) / 3) / 2 = 1, exactly in floating point.
    assert_eq!(PAIRS_SHARE.to_bits(), Progress::at(1, 2, 2, 3, 1.0).fraction.to_bits());
  }

  #[test]
  fn progress_pairs_share_leaves_room_below_done() {
    assert!(PAIRS_SHARE < Progress::done(2, 3).fraction);
    assert_eq!(1.0_f64.to_bits(), Progress::done(2, 3).fraction.to_bits());
  }

  #[test]
  fn progress_end_of_pair_equals_start_of_next_pair() {
    let end = Progress::at(0, 2, 0, 3, 1.0).fraction;
    let start = Progress::at(0, 2, 1, 3, 0.0).fraction;
    assert_eq!(end.to_bits(), start.to_bits());
  }

  #[test]
  fn progress_end_of_round_equals_start_of_next_round() {
    let end = Progress::at(0, 2, 2, 3, 1.0).fraction;
    let start = Progress::at(1, 2, 0, 3, 0.0).fraction;
    assert_eq!(end.to_bits(), start.to_bits());
  }

  #[test]
  fn progress_iterations_done_reaches_one_after_the_last_iteration() {
    // itmax = 2 allows 3 iterations; the last step of iteration index 2 completes the pair.
    assert_eq!(1.0_f64.to_bits(), iterations_done(2, 1.0, 3).to_bits());
  }

  #[test]
  fn progress_iterations_done_is_bounded_by_one() {
    assert_eq!(1.0_f64.to_bits(), iterations_done(3, 0.5, 3).to_bits());
  }

  #[test]
  fn progress_iterations_done_within_an_iteration() {
    // (1 + 0.5) / 3 = 0.5.
    assert_eq!(0.5_f64.to_bits(), iterations_done(1, 0.5, 3).to_bits());
  }
}
