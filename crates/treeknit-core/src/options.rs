//! Run options and the two method presets.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cooling {
    Geometric,
    Linear,
    Acos,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// Pre-resolve, then one round of pair inference without resolving (default for K > 2).
    BetterTrees,
    /// Pre-resolve, resolve during pair inference; for K > 2 a final round without resolving
    /// (default for K = 2).
    BetterMccs,
}

#[derive(Clone, Debug)]
pub struct Options {
    /// Cost of removing one MCC (enforcing one reassortment).
    pub gamma: f64,
    /// Maximal number of prune iterations per pair.
    pub itmax: usize,
    /// Break ties between equally good configurations with branch lengths.
    pub likelihood_sort: bool,
    /// Resolve trees during pair inference, and with inferred MCCs.
    pub resolve: bool,
    /// Only introduce unambiguous splits when resolving with MCCs.
    pub strict: bool,
    /// Sequence length of each tree (for the likelihood).
    pub seq_lengths: Vec<f64>,
    /// Resolve all trees with each other before inference.
    pub pre_resolve: bool,
    /// Rounds of pair inference.
    pub rounds: usize,
    /// Do not resolve in the final round.
    pub final_no_resolve: bool,
    /// Total MCMC steps per leaf.
    pub n_mcmc: usize,
    /// Independent annealing runs per iteration.
    pub sa_rep: usize,
    pub t_min: f64,
    pub t_max: f64,
    pub n_t: usize,
    pub cooling: Cooling,
    /// Return naive MCCs (γ → ∞).
    pub naive: bool,
    /// Run independent pairs in parallel.
    pub parallel: bool,
    /// After inference, resolve trees so that their topologies match within every MCC, earlier
    /// trees taking precedence (see `pipeline::match_topologies`).
    pub match_topologies: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            gamma: 2.0,
            itmax: 15,
            likelihood_sort: true,
            resolve: true,
            strict: true,
            seq_lengths: vec![1.0, 1.0],
            pre_resolve: true,
            rounds: 1,
            final_no_resolve: false,
            n_mcmc: 50,
            sa_rep: 1,
            t_min: 0.05,
            t_max: 1.0,
            n_t: 100,
            cooling: Cooling::Geometric,
            naive: false,
            parallel: true,
            match_topologies: false,
        }
    }
}

impl Options {
    /// Defaults for `k` trees with the given method (default: `BetterMccs` for k = 2,
    /// `BetterTrees` otherwise).
    pub fn for_trees(k: usize, method: Option<Method>) -> Options {
        let method = method.unwrap_or(if k > 2 { Method::BetterTrees } else { Method::BetterMccs });
        let mut o = Options {
            seq_lengths: vec![1.0; k],
            ..Options::default()
        };
        match method {
            Method::BetterTrees => {
                o.resolve = false;
                o.final_no_resolve = true;
                o.rounds = 1;
            }
            Method::BetterMccs if k > 2 => {
                o.resolve = true;
                o.final_no_resolve = true;
                o.rounds = 2;
            }
            Method::BetterMccs => {
                o.resolve = true;
                o.final_no_resolve = false;
                o.rounds = 1;
            }
        }
        o.normalize();
        o
    }

    /// A single round that should not resolve is simply a round without resolution.
    pub fn normalize(&mut self) {
        if self.final_no_resolve && self.rounds == 1 && self.resolve {
            self.resolve = false;
            self.final_no_resolve = false;
        }
    }

    pub fn temperatures(&self) -> Vec<f64> {
        crate::anneal::schedule(self.cooling, self.t_min, self.t_max, self.n_t)
    }
}
