//! Simulated annealing over configurations of a split graph.

#![expect(
  clippy::as_conversions,
  reason = "findings from before the strict lint set; kb/issues/N-lint-baseline.md tracks their removal"
)]

use crate::bits::{self, Bits};
use crate::options::Cooling;
use crate::progress::ratio;
use crate::splitgraph::{EnergyState, Graph};
use indexmap::IndexSet;
use rand::Rng;

/// Temperatures from `t_max` down to `t_min`.
#[expect(
  clippy::float_cmp,
  reason = "the acos schedule is defined piecewise around exactly x = 0.5, as in TreeKnit.jl"
)]
pub fn schedule(cooling: Cooling, t_min: f64, t_max: f64, n_t: usize) -> Vec<f64> {
  assert!(t_min > 0.0 && t_max > t_min, "need 0 < t_min < t_max");
  let lin = |i: usize| if n_t > 1 { i as f64 / (n_t - 1) as f64 } else { 0.0 };
  match cooling {
    Cooling::Geometric => {
      let alpha = ((t_min.ln() - t_max.ln()) / n_t as f64).exp();
      let n = ((t_min.ln() - t_max.ln()) / alpha.ln()).ceil() as i32;
      (0..=n).map(|i| alpha.powi(i) * t_max).collect()
    },
    Cooling::Linear => (0..n_t).map(|i| t_max - (t_max - t_min) * lin(i)).collect(),
    #[expect(
      clippy::approx_constant,
      reason = "TreeKnit.jl uses 3.14, not π, in the acos schedule"
    )]
    Cooling::Acos => {
      let k: f64 = 1.5;
      let f = |x: f64| {
        let d = 2_f64.powf(k - 1.0) * ((2.0 * x - 1.0).acos() / 3.14 - 0.5).abs().powf(k);
        if x < 0.5 {
          0.5 + d
        } else if x == 0.5 {
          0.5
        } else {
          0.5 - d
        }
      };
      (0..n_t).map(|i| (t_max - t_min) * f(lin(i)) + t_min).collect()
    },
  }
}

/// Distinct configurations in the order they were first visited.
type ConfSet = IndexSet<Bits>;

struct Chain<'a, R: Rng> {
  g: &'a Graph,
  gamma: f64,
  resolve: bool,
  rng: &'a mut R,
}

impl<R: Rng> Chain<'_, R> {
  fn free_energy(&self, st: &EnergyState) -> f64 {
    st.energy() as f64 + self.gamma * (self.g.n - st.n_kept()) as f64
  }

  /// `m` Metropolis steps at temperature `t` from the current state.
  /// Returns the minimal-F configurations visited and the minimal F.
  #[expect(
    clippy::float_cmp,
    reason = "free energies are compared for exact ties, so every configuration of minimal energy is kept"
  )]
  fn mcmc(&mut self, st: &mut EnergyState, m: usize, t: f64) -> (ConfSet, f64) {
    let mut f = self.free_energy(st);
    let mut fmin = f;
    let mut best = ConfSet::from([st.conf().clone()]);
    // The current configuration is in `best` and unchanged since; a rejected step keeps it so.
    let mut listed = true;
    for _ in 0..m {
      let leaf = self.rng.gen_range(0..self.g.n);
      st.flip(leaf);
      let fnew = self.free_energy(st);
      if fnew < f || (-(fnew - f) / t).exp() > self.rng.r#gen::<f64>() {
        f = fnew;
        listed = false;
      } else {
        st.undo();
      }
      if f < fmin {
        fmin = f;
        best = ConfSet::from([st.conf().clone()]);
        listed = true;
      } else if f == fmin && !listed {
        best.insert(st.conf().clone());
        listed = true;
      }
    }
    (best, fmin)
  }

  /// One annealing run starting from all leaves kept. Calls `after_step` with the number of
  /// completed temperatures after each temperature.
  #[expect(
    clippy::float_cmp,
    reason = "free energies are compared for exact ties, so every configuration of minimal energy is kept"
  )]
  fn anneal(&mut self, trange: &[f64], m: usize, after_step: &dyn Fn(usize)) -> (ConfSet, f64) {
    let mut st = EnergyState::new(self.g, bits::full(self.g.n), self.resolve);
    let mut best = ConfSet::from([st.conf().clone()]);
    let mut fmin = f64::INFINITY;
    for (step, &t) in trange.iter().enumerate() {
      let (b, f) = self.mcmc(&mut st, m, t);
      after_step(step + 1);
      if f < fmin {
        best = b;
        fmin = f;
      } else if f == fmin {
        best.extend(b);
      }
    }
    (best, fmin)
  }
}

/// Run `reps` annealing runs; return all distinct configurations of minimal free energy.
/// Calls `on_step` with the completed fraction of all runs after each temperature.
#[expect(
  clippy::float_cmp,
  reason = "free energies are compared for exact ties, so every configuration of minimal energy is kept"
)]
pub fn optimize(
  g: &Graph,
  gamma: f64,
  trange: &[f64],
  m: usize,
  reps: usize,
  resolve: bool,
  rng: &mut impl Rng,
  on_step: &dyn Fn(f64),
) -> Vec<Bits> {
  let mut chain = Chain { g, gamma, resolve, rng };
  let mut best = ConfSet::default();
  let mut fmin = f64::INFINITY;
  let reps = reps.max(1);
  let steps = reps * trange.len();
  for rep in 0..reps {
    let (b, f) = chain.anneal(trange, m, &|step| on_step(ratio(rep * trange.len() + step, steps)));
    if f < fmin {
      best = b;
      fmin = f;
    } else if f == fmin {
      best.extend(b);
    }
  }
  best.into_iter().collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use approx::assert_ulps_eq;

  #[test]
  fn geometric_schedule_matches_julia() {
    let t = schedule(Cooling::Geometric, 0.05, 1.0, 100);
    assert_eq!(101, t.len());
    // Oracle: the temperatures of TreeKnit.jl, which the schedule reproduces exactly.
    assert_ulps_eq!(1.0, t[0], max_ulps = 0);
    assert_ulps_eq!(0.9704869503929601, t[1], max_ulps = 0);
    assert_ulps_eq!(0.05, t[100], max_ulps = 0);
  }
}
