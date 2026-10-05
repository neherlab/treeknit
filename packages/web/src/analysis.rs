//! Analysis requests and results exchanged with the web page, independent of the JavaScript
//! bindings so that they can be tested natively.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use treeknit_core::arg::arg_from_trees;
use treeknit_core::{Options, PairResult, Resolution, Taxa, Tree};
use treeknit_io::{arg, mccs, newick};

/// Run TreeKnit on the request's trees.
pub fn analyze(request: &Request) -> Result<Analysis, String> {
    let k = request.trees.len();
    if k < 2 {
        return Err("need at least two trees".to_owned());
    }
    check_labels(&request.trees)?;
    let opts = options(&request.settings, k)?;
    let mut trees = request
        .trees
        .iter()
        .map(|t| newick::parse_first(&t.newick, &t.label).map_err(|e| format!("tree {}: {e}", t.label)))
        .collect::<Result<Vec<Tree>, String>>()?;
    let taxa = Taxa::from_trees(&trees);
    for t in &mut trees {
        t.assign_taxa(&taxa).map_err(|e| format!("tree {}: {e}", t.label))?;
    }

    let pairs = treeknit_core::run(&mut trees, &taxa, &opts, request.settings.seed);
    let arg = (k == 2 && !pairs[0].mccs.is_empty()).then(|| arg_outcome(&trees, &pairs[0], &taxa));
    Ok(Analysis {
        mccs: mccs::to_json(&pairs, &trees, &taxa),
        resolved: texts(&trees),
        imputed: texts(&treeknit_core::imputed_trees(&trees, &pairs, taxa.len())),
        arg,
    })
}

/// Input trees and settings of one analysis.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub trees: Vec<TreeText>,
    #[serde(default)]
    pub settings: Settings,
}

/// A labelled tree in Newick format.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TreeText {
    pub label: String,
    pub newick: String,
}

/// Settings of the `treeknit` command line; missing fields take its defaults.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    /// Cost γ of a reassortment (removing an MCC).
    pub gamma: f64,
    /// Sequence length of each segment, for the branch-length tie-break (default: all equal).
    pub seq_lengths: Option<Vec<f64>>,
    /// MCMC steps per leaf.
    pub n_mcmc_it: usize,
    pub resolve: ResolveMode,
    /// Before inference, add to each tree the splits of other trees compatible with all trees.
    pub pre_resolve: bool,
    /// Rounds of pair inference.
    pub rounds: usize,
    /// With strict or liberal resolution and more than two trees, re-infer MCCs without
    /// resolution in a final round.
    pub final_round: bool,
    /// Break ties between configurations with branch lengths.
    pub likelihood: bool,
    /// Naive MCCs (γ → ∞).
    pub naive: bool,
    pub seed: u64,
}

impl Default for Settings {
    fn default() -> Self {
        let o = Options::default();
        Settings {
            gamma: o.gamma,
            seq_lengths: None,
            n_mcmc_it: o.n_mcmc,
            resolve: ResolveMode::default(),
            pre_resolve: o.pre_resolve,
            rounds: o.rounds,
            final_round: o.final_unresolved_round,
            likelihood: o.likelihood_sort,
            naive: o.naive,
            seed: 1,
        }
    }
}

/// How trees are resolved (see `treeknit --help-resolve`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResolveMode {
    None,
    Strict,
    Liberal,
    #[default]
    Matched,
}

impl From<ResolveMode> for Resolution {
    fn from(m: ResolveMode) -> Resolution {
        match m {
            ResolveMode::None => Resolution::None,
            ResolveMode::Strict => Resolution::Strict,
            ResolveMode::Liberal => Resolution::Liberal,
            ResolveMode::Matched => Resolution::Matched,
        }
    }
}

/// Results of one analysis, matching the output files of the `treeknit` command line.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    /// MCCs of every tree pair, in the format of `MCCs.json`.
    pub mccs: Value,
    /// Input trees after resolution, in input order.
    pub resolved: Vec<TreeText>,
    /// Resolved trees with the leaves missing from them placed by imputation.
    pub imputed: Vec<TreeText>,
    /// For two trees with MCCs, the ancestral reassortment graph.
    pub arg: Option<ArgOutcome>,
}

/// Ancestral reassortment graph, or why it could not be built.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ArgOutcome {
    Built(ArgText),
    Failed { message: String },
}

/// Ancestral reassortment graph of two trees.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArgText {
    /// Extended Newick.
    pub newick: String,
    /// Node table (`nodes.dat`).
    pub nodes: String,
    pub reassortments: usize,
    /// The liberally resolved trees the graph was built from.
    pub trees: Vec<TreeText>,
}

fn check_labels(trees: &[TreeText]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for t in trees {
        if t.label.trim().is_empty() {
            return Err("every tree needs a label".to_owned());
        }
        if !seen.insert(t.label.as_str()) {
            return Err(format!("tree label {} is used twice", t.label));
        }
    }
    Ok(())
}

/// Core options for `k` trees. Rejects settings on which `treeknit_core::run` would panic, since
/// a panic aborts the WebAssembly instance.
fn options(s: &Settings, k: usize) -> Result<Options, String> {
    if !(s.gamma.is_finite() && s.gamma >= 0.0) {
        return Err(format!("gamma must be a non-negative number, got {}", s.gamma));
    }
    if s.rounds == 0 {
        return Err("rounds must be at least 1".to_owned());
    }
    let mut o = Options::for_trees(k);
    if let Some(v) = &s.seq_lengths {
        if v.len() != k {
            return Err(format!("got {} sequence lengths for {k} trees", v.len()));
        }
        if !v.iter().all(|x| x.is_finite() && *x > 0.0) {
            return Err("sequence lengths must be positive numbers".to_owned());
        }
        o.seq_lengths.clone_from(v);
    }
    o.gamma = s.gamma;
    o.n_mcmc = s.n_mcmc_it;
    o.resolution = s.resolve.into();
    o.pre_resolve = s.pre_resolve;
    o.rounds = s.rounds;
    o.final_unresolved_round = s.final_round;
    o.likelihood_sort = s.likelihood;
    o.naive = s.naive;
    // WebAssembly in the browser has no threads by default; rayon would fall back to the
    // calling thread anyway, the sequential path avoids it altogether.
    o.parallel = false;
    Ok(o)
}

fn arg_outcome(trees: &[Tree], pair: &PairResult, taxa: &Taxa) -> ArgOutcome {
    let (t1, t2, m) = treeknit_core::arg_inputs(trees, pair, taxa.len());
    match arg_from_trees(&t1, &t2, &m, taxa.len()) {
        Ok(a) => ArgOutcome::Built(ArgText {
            newick: arg::extended_newick(&a),
            nodes: arg::node_table(&a),
            reassortments: a.n_hybrids(),
            trees: texts(&a.trees),
        }),
        Err(e) => ArgOutcome::Failed { message: e.to_string() },
    }
}

fn texts(trees: &[Tree]) -> Vec<TreeText> {
    trees
        .iter()
        .map(|t| TreeText {
            label: t.label.clone(),
            newick: newick::write(t),
        })
        .collect()
}
