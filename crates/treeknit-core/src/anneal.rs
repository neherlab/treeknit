//! Simulated annealing over configurations of a split graph.

use crate::bits::{self, Bits};
use crate::options::Cooling;
use crate::splitgraph::{EnergyState, Graph};
use rand::Rng;
use std::collections::HashSet;

/// Temperatures from `t_max` down to `t_min`.
pub fn schedule(cooling: Cooling, t_min: f64, t_max: f64, n_t: usize) -> Vec<f64> {
    assert!(t_min > 0.0 && t_max > t_min, "need 0 < t_min < t_max");
    let lin = |i: usize| if n_t > 1 { i as f64 / (n_t - 1) as f64 } else { 0.0 };
    match cooling {
        Cooling::Geometric => {
            let alpha = ((t_min.ln() - t_max.ln()) / n_t as f64).exp();
            let n = ((t_min.ln() - t_max.ln()) / alpha.ln()).ceil() as i32;
            (0..=n).map(|i| alpha.powi(i) * t_max).collect()
        }
        Cooling::Linear => (0..n_t).map(|i| t_max - (t_max - t_min) * lin(i)).collect(),
        // 3.14 (not π) as in the Julia implementation.
        #[allow(clippy::approx_constant)]
        Cooling::Acos => {
            let k: f64 = 1.5;
            let f = |x: f64| {
                let d = 2f64.powf(k - 1.0) * ((2.0 * x - 1.0).acos() / 3.14 - 0.5).abs().powf(k);
                if x < 0.5 {
                    0.5 + d
                } else if x == 0.5 {
                    0.5
                } else {
                    0.5 - d
                }
            };
            (0..n_t).map(|i| (t_max - t_min) * f(lin(i)) + t_min).collect()
        }
    }
}

/// Set of distinct configurations preserving insertion order.
#[derive(Default)]
struct ConfSet {
    list: Vec<Bits>,
    seen: HashSet<Bits>,
}

impl ConfSet {
    fn single(c: Bits) -> Self {
        let mut s = ConfSet::default();
        s.push(c);
        s
    }
    fn push(&mut self, c: Bits) {
        if self.seen.insert(c.clone()) {
            self.list.push(c);
        }
    }
    fn extend(&mut self, other: ConfSet) {
        for c in other.list {
            self.push(c);
        }
    }
}

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
    fn mcmc(&mut self, st: &mut EnergyState, m: usize, t: f64) -> (ConfSet, f64) {
        let n = self.g.n;
        let mut f = self.free_energy(st);
        let mut fmin = f;
        let mut best = ConfSet::single(st.conf().clone());
        for _ in 0..m {
            let i = self.rng.gen_range(0..n);
            st.flip(i);
            let fnew = self.free_energy(st);
            if fnew < f || (-(fnew - f) / t).exp() > self.rng.gen::<f64>() {
                f = fnew;
            } else {
                st.undo();
            }
            if f < fmin {
                fmin = f;
                best = ConfSet::single(st.conf().clone());
            } else if f == fmin {
                best.push(st.conf().clone());
            }
        }
        (best, fmin)
    }

    /// One annealing run starting from all leaves kept.
    fn anneal(&mut self, trange: &[f64], m: usize) -> (ConfSet, f64) {
        let mut st = EnergyState::new(self.g, bits::full(self.g.n), self.resolve);
        let mut best = ConfSet::single(st.conf().clone());
        let mut fmin = f64::INFINITY;
        for &t in trange {
            let (b, f) = self.mcmc(&mut st, m, t);
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
pub fn optimize(
    g: &Graph,
    gamma: f64,
    trange: &[f64],
    m: usize,
    reps: usize,
    resolve: bool,
    rng: &mut impl Rng,
) -> Vec<Bits> {
    let mut chain = Chain { g, gamma, resolve, rng };
    let mut best = ConfSet::default();
    let mut fmin = f64::INFINITY;
    for _ in 0..reps.max(1) {
        let (b, f) = chain.anneal(trange, m);
        if f < fmin {
            best = b;
            fmin = f;
        } else if f == fmin {
            best.extend(b);
        }
    }
    best.list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometric_schedule_matches_julia() {
        let t = schedule(Cooling::Geometric, 0.05, 1.0, 100);
        assert_eq!(t.len(), 101);
        assert!((t[0] - 1.0).abs() < 1e-12);
        assert!((t[1] - 0.9704869503929601).abs() < 1e-12);
        assert!((t[100] - 0.05).abs() < 1e-12);
    }
}
