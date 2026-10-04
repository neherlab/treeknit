//! Run options.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cooling {
    Geometric,
    Linear,
    Acos,
}

/// How trees are resolved during and after MCC inference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resolution {
    /// No resolution with MCCs: MCCs require identical topologies.
    None,
    /// Resolve during pair inference and with the inferred MCCs, unambiguous splits only.
    Strict,
    /// As `Strict`, also adding ambiguous splits (placement of other MCCs chosen arbitrarily).
    Liberal,
    /// As `Strict`, then resolve all trees so that their topologies match within every MCC,
    /// earlier trees taking precedence (see `pipeline::match_topologies`).
    Matched,
}

/// The method presets of TreeKnit.jl (see [`Options::treeknit_jl`]).
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
    /// How trees are resolved during and after inference.
    pub resolution: Resolution,
    /// Sequence length of each tree (for the likelihood).
    pub seq_lengths: Vec<f64>,
    /// Before inference, add to each tree the splits of other trees that are compatible with
    /// all trees.
    pub pre_resolve: bool,
    /// Rounds of pair inference (resolving, unless `resolution` is `None`).
    pub rounds: usize,
    /// With `Strict`/`Liberal` and more than two trees, re-infer all MCCs without resolution in a
    /// final extra round, since resolving later pairs can invalidate earlier pairs' MCCs.
    pub final_unresolved_round: bool,
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
    /// Whether polytomies of the output trees are sorted using strictly or liberally resolved
    /// copies; by default as resolution in the final round. Only needed to reproduce the
    /// output order of TreeKnit.jl option combinations exactly.
    pub sort_strict: Option<bool>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            gamma: 2.0,
            itmax: 15,
            likelihood_sort: true,
            resolution: Resolution::Matched,
            seq_lengths: vec![1.0, 1.0],
            pre_resolve: false,
            rounds: 1,
            final_unresolved_round: true,
            n_mcmc: 50,
            sa_rep: 1,
            t_min: 0.05,
            t_max: 1.0,
            n_t: 100,
            cooling: Cooling::Geometric,
            naive: false,
            parallel: true,
            sort_strict: None,
        }
    }
}

impl Options {
    /// Defaults for `k` trees (the same for any number of trees).
    pub fn for_trees(k: usize) -> Options {
        Options {
            seq_lengths: vec![1.0; k],
            ..Options::default()
        }
    }

    /// The presets of TreeKnit.jl for `k` trees (default method: `BetterMccs` for k = 2,
    /// `BetterTrees` otherwise).
    pub fn treeknit_jl(k: usize, method: Option<Method>) -> Options {
        let method = method.unwrap_or(if k > 2 { Method::BetterTrees } else { Method::BetterMccs });
        Options {
            resolution: match method {
                Method::BetterTrees => Resolution::None,
                Method::BetterMccs => Resolution::Strict,
            },
            pre_resolve: true,
            ..Options::for_trees(k)
        }
    }

    /// Resolve during pair inference.
    pub fn resolves(&self) -> bool {
        self.resolution != Resolution::None
    }

    pub fn temperatures(&self) -> Vec<f64> {
        crate::anneal::schedule(self.cooling, self.t_min, self.t_max, self.n_t)
    }
}
