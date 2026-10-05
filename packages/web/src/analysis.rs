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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use serde_json::json;

    macro_rules! assert_err {
        ($result:expr, $expected:expr) => {
            match $result {
                Ok(_) => panic!("expected error {:?}, got Ok", $expected),
                Err(e) => assert_eq!($expected, e.to_string()),
            }
        };
    }

    fn request(trees: &[(&str, &str)], settings: Settings) -> Request {
        Request {
            trees: trees
                .iter()
                .map(|(label, newick)| TreeText {
                    label: (*label).to_owned(),
                    newick: (*newick).to_owned(),
                })
                .collect(),
            settings,
        }
    }

    /// Non-root clades of a Newick tree, as sets of leaf names.
    fn clades(newick: &str) -> BTreeSet<BTreeSet<String>> {
        let t = newick::parse(newick, "t").unwrap();
        t.internals()
            .into_iter()
            .filter(|&n| n != t.root)
            .map(|n| t.leaves_below(n).into_iter().map(|l| t.name(l).to_owned()).collect())
            .collect()
    }

    fn built(arg: Option<ArgOutcome>) -> ArgText {
        match arg {
            Some(ArgOutcome::Built(a)) => a,
            other => panic!("expected an ARG, got {other:?}"),
        }
    }

    #[test]
    fn identical_trees_form_one_mcc_without_reassortment() {
        let r = request(
            &[("ha", "((A,B),(C,D));"), ("na", "((A,B),(C,D));")],
            Settings::default(),
        );
        let a = analyze(&r).unwrap();
        assert_eq!(
            json!({"MCC_dict": {"1": {"trees": ["ha", "na"], "mccs": [["A", "B", "C", "D"]]}}}),
            a.mccs
        );
        assert_eq!(0, built(a.arg).reassortments);
    }

    #[test]
    fn moved_leaf_matches_reference() {
        // `fixtures/doc_mccs_1.json`: TreeKnit.jl finds X in its own MCC in all seeded runs,
        // with one reassortment in the ARG.
        let r = request(
            &[("ha", "((A,B),(C,(D,X)));"), ("na", "((A,(B,X)),(C,D));")],
            Settings::default(),
        );
        let a = analyze(&r).unwrap();
        assert_eq!(
            json!({"MCC_dict": {"1": {"trees": ["ha", "na"], "mccs": [["X"], ["A", "B", "C", "D"]]}}}),
            a.mccs
        );
        let arg = built(a.arg);
        assert_eq!(1, arg.reassortments);
        assert_eq!(
            vec!["ha", "na"],
            arg.trees.iter().map(|t| t.label.as_str()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn three_trees_give_all_pairs_and_no_arg() {
        let t = "((A,B),(C,D));";
        let r = request(&[("ha", t), ("na", t), ("pb2", t)], Settings::default());
        let a = analyze(&r).unwrap();
        let all = json!([["A", "B", "C", "D"]]);
        assert_eq!(
            json!({"MCC_dict": {
                "1": {"trees": ["ha", "na"], "mccs": all},
                "2": {"trees": ["ha", "pb2"], "mccs": all},
                "3": {"trees": ["na", "pb2"], "mccs": all},
            }}),
            a.mccs
        );
        assert!(a.arg.is_none());
    }

    #[test]
    fn leaf_missing_from_one_tree_is_imputed() {
        let r = request(
            &[("ha", "((A,B),(C,(D,P)));"), ("na", "((A,B),(C,D));")],
            Settings::default(),
        );
        let a = analyze(&r).unwrap();
        assert_eq!(
            json!({"MCC_dict": {"1": {
                "trees": ["ha", "na"],
                "mccs": [["A", "B", "C", "D", "P"]],
                "imputed": [{"leaf": "P", "tree": "ha", "mcc": 0, "ambiguous": false}],
            }}}),
            a.mccs
        );
        // P joins na as sister of D, where ha has it.
        assert_eq!(clades("((A,B),(C,(D,P)));"), clades(&a.imputed[1].newick));
        assert_eq!(clades("((A,B),(C,D));"), clades(&a.resolved[1].newick));
    }

    #[rstest]
    #[case::matched(ResolveMode::Matched, clades("((A,B),(C,D));"))]
    #[case::none(ResolveMode::None, BTreeSet::new())]
    #[trace]
    fn resolve_mode_controls_resolution(#[case] resolve: ResolveMode, #[case] expected: BTreeSet<BTreeSet<String>>) {
        // The polytomy of ha is compatible with na: one MCC, within which matched resolution
        // copies na's splits into ha.
        let r = request(
            &[("ha", "(A,B,C,D);"), ("na", "((A,B),(C,D));")],
            Settings {
                resolve,
                ..Settings::default()
            },
        );
        let a = analyze(&r).unwrap();
        assert_eq!(expected, clades(&a.resolved[0].newick));
    }

    #[test]
    fn settings_default_to_command_line_defaults() {
        let expected = Settings {
            gamma: 2.0,
            seq_lengths: None,
            n_mcmc_it: 50,
            resolve: ResolveMode::Matched,
            pre_resolve: false,
            rounds: 1,
            final_round: true,
            likelihood: true,
            naive: false,
            seed: 1,
        };
        assert_eq!(expected, serde_json::from_value::<Settings>(json!({})).unwrap());
    }

    #[test]
    fn settings_read_camel_case_fields() {
        let s: Settings = serde_json::from_value(json!({
            "gamma": 3.5, "seqLengths": [1700.0, 1400.0], "nMcmcIt": 10, "resolve": "liberal",
            "preResolve": true, "rounds": 2, "finalRound": false, "likelihood": false,
            "naive": true, "seed": 7,
        }))
        .unwrap();
        let expected = Settings {
            gamma: 3.5,
            seq_lengths: Some(vec![1700.0, 1400.0]),
            n_mcmc_it: 10,
            resolve: ResolveMode::Liberal,
            pre_resolve: true,
            rounds: 2,
            final_round: false,
            likelihood: false,
            naive: true,
            seed: 7,
        };
        assert_eq!(expected, s);
    }

    #[test]
    fn settings_reject_unknown_fields() {
        assert_err!(
            serde_json::from_value::<Settings>(json!({"gama": 1.0})),
            "unknown field `gama`, expected one of `gamma`, `seqLengths`, `nMcmcIt`, `resolve`, \
             `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`, `seed`"
        );
    }

    #[rustfmt::skip]
    #[rstest]
    #[case::one_tree(        &[("ha", "(A,B);")],                     Settings::default(),                                                         "need at least two trees")]
    #[case::empty_label(     &[("ha", "(A,B);"), (" ", "(A,B);")],    Settings::default(),                                                         "every tree needs a label")]
    #[case::duplicate_label( &[("ha", "(A,B);"), ("ha", "(A,B);")],   Settings::default(),                                                         "tree label ha is used twice")]
    #[case::bad_newick(      &[("ha", "(A,B);"), ("na", "(A,B")],     Settings::default(),                                                         "tree na: Newick parse error: no ';' found")]
    #[case::negative_gamma(  &[("ha", "(A,B);"), ("na", "(A,B);")],   Settings { gamma: -1.0, ..Settings::default() },                             "gamma must be a non-negative number, got -1")]
    #[case::nan_gamma(       &[("ha", "(A,B);"), ("na", "(A,B);")],   Settings { gamma: f64::NAN, ..Settings::default() },                         "gamma must be a non-negative number, got NaN")]
    #[case::zero_rounds(     &[("ha", "(A,B);"), ("na", "(A,B);")],   Settings { rounds: 0, ..Settings::default() },                               "rounds must be at least 1")]
    #[case::length_count(    &[("ha", "(A,B);"), ("na", "(A,B);")],   Settings { seq_lengths: Some(vec![1.0]), ..Settings::default() },            "got 1 sequence lengths for 2 trees")]
    #[case::zero_length(     &[("ha", "(A,B);"), ("na", "(A,B);")],   Settings { seq_lengths: Some(vec![1.0, 0.0]), ..Settings::default() },       "sequence lengths must be positive numbers")]
    #[trace]
    fn invalid_requests_are_rejected(#[case] trees: &[(&str, &str)], #[case] settings: Settings, #[case] expected: &str) {
        assert_err!(analyze(&request(trees, settings)), expected);
    }
}
