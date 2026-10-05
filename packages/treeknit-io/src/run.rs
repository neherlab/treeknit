//! The run result: everything one run of TreeKnit produces, built once by the command line and
//! by the web app. Output files, display data, and figures read from it.

use crate::analysis::ParsedTrees;
use crate::summary::ArgOutcome;
use std::sync::OnceLock;
use treeknit_core::arg::{Arg, ArgError, arg_from_trees};
use treeknit_core::{Options, PairResult, Progress, Taxa, Tree};

/// The trees, MCCs, imputed trees, and ARG of one run, with the options it ran with. The fields
/// are read-only outside this crate, because the display data keeps values derived from them
/// (the color slots of each pair).
#[derive(Debug)]
pub struct RunResult {
  /// The parsed trees, before resolution: the `input` version of the display data, and the
  /// reference by which every version flags added nodes and imputed leaves.
  pub(crate) input_trees: Vec<Tree>,
  /// The final trees of the run: resolved and sorted.
  pub(crate) trees: Vec<Tree>,
  /// The leaves of all trees; the trees carry taxon ids of this table.
  pub(crate) taxa: Taxa,
  /// The MCCs of every pair, in pipeline order `(0,1), (0,2), ..., (1,2), ...`.
  pub(crate) pairs: Vec<PairResult>,
  /// The final trees with the leaves that only other trees have, placed by imputation: the
  /// `_imputed` output files and the `imputed` version of the display data. Every run computes
  /// them, because the file set of the web app always holds them.
  pub(crate) imputed: Vec<Tree>,
  /// The ARG of two trees whose pair has MCCs, or the reason it could not be built; `None`
  /// otherwise.
  pub(crate) arg: Option<Result<Arg, ArgError>>,
  /// The options of the run; the display data and the figures sort the trees of a pair as the
  /// run did.
  pub(crate) opts: Options,
  /// The color slots of the MCCs of each pair, computed on the first display of the pair (see
  /// `display`), because they need a full layout of its resolved trees.
  pub(crate) pair_slots: Vec<OnceLock<Vec<usize>>>,
}

impl RunResult {
  /// The parsed trees, before resolution.
  pub fn input_trees(&self) -> &[Tree] {
    &self.input_trees
  }

  /// The final trees of the run: resolved and sorted.
  pub fn trees(&self) -> &[Tree] {
    &self.trees
  }

  /// The leaves of all trees.
  pub fn taxa(&self) -> &Taxa {
    &self.taxa
  }

  /// The MCCs of every pair, in pipeline order.
  pub fn pairs(&self) -> &[PairResult] {
    &self.pairs
  }

  /// The final trees with the leaves that only other trees have, placed by imputation.
  pub fn imputed(&self) -> &[Tree] {
    &self.imputed
  }

  /// The options of the run.
  pub fn options(&self) -> &Options {
    &self.opts
  }

  /// Outcome of the ARG for the summary; `None` when no ARG was attempted.
  pub fn arg_outcome(&self) -> Option<ArgOutcome> {
    self.arg.as_ref().map(|a| match a {
      Ok(arg) => ArgOutcome::Built {
        reassortments: arg.n_hybrids(),
      },
      Err(e) => ArgOutcome::Failed { message: e.to_string() },
    })
  }

  /// The ARG, when it was built.
  pub fn built_arg(&self) -> Option<&Arg> {
    self.arg.as_ref().and_then(|a| a.as_ref().ok())
  }
}

/// Run TreeKnit on `parsed` with `opts` and `seed`, calling `observe` with the progress of the
/// run (see `treeknit_core::run_observed`), then impute the missing leaves of every tree and,
/// for two trees, build the ARG. `opts` must have passed the settings checks of
/// `analysis::check_settings` and `parsed` those of `analysis::parse_trees`, because the core
/// panics on inputs that fail them.
pub fn run(parsed: ParsedTrees, opts: &Options, seed: u64, observe: &dyn Fn(Progress)) -> RunResult {
  let ParsedTrees { mut trees, taxa } = parsed;
  let input_trees = trees.clone();
  let pairs = treeknit_core::run_observed(&mut trees, &taxa, opts, seed, observe);
  let imputed = treeknit_core::imputed_trees(&trees, &pairs, taxa.len());
  let arg = match pairs.as_slice() {
    [pair] if !pair.mccs.is_empty() => Some(build_arg(&trees, pair, &taxa)),
    _ => None,
  };
  let pair_slots = pairs.iter().map(|_| OnceLock::new()).collect();
  RunResult {
    input_trees,
    trees,
    taxa,
    pairs,
    imputed,
    arg,
    opts: opts.clone(),
    pair_slots,
  }
}

/// Log, for each tree, the leaves that it lacks and that imputation places.
pub fn report_overlap(trees: &[Tree], taxa: &Taxa) {
  let n = taxa.len();
  for t in trees {
    let missing = n - t.n_leaves();
    if missing > 0 {
      log::info!(
        "tree {}: {} of {n} leaves missing (placed by imputation)",
        t.label,
        missing
      );
    }
  }
}

/// The ARG of the two trees of `pair`, built from their liberally resolved trees.
fn build_arg(trees: &[Tree], pair: &PairResult, taxa: &Taxa) -> Result<Arg, ArgError> {
  log::debug!("building ARG from trees and MCCs");
  let (t1, t2, m) = treeknit_core::arg_inputs(trees, pair, taxa.len());
  let arg = arg_from_trees(&t1, &t2, &m, taxa.len());
  match &arg {
    Ok(a) => log::info!("found {} reassortments in the ARG", a.n_hybrids()),
    Err(e) => log::error!("{e}; no ARG written"),
  }
  arg
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::analysis::{self, Settings, TreeText};
  use crate::newick;
  use pretty_assertions::assert_eq;
  use std::cell::RefCell;
  use std::collections::BTreeSet;

  /// The two-tree example: X moved between the trees.
  const HA: &str = "((A,B),(C,(D,X)));";
  const NA: &str = "((A,(B,X)),(C,D));";

  fn run_trees(trees: &[(&str, &str)]) -> RunResult {
    let texts: Vec<TreeText> = trees
      .iter()
      .map(|(label, newick)| TreeText {
        label: (*label).to_owned(),
        newick: (*newick).to_owned(),
      })
      .collect();
    let s = Settings::default();
    let opts = analysis::options(&s, texts.len(), false).unwrap();
    run(analysis::parse_trees(&texts).unwrap(), &opts, s.seed, &|_| {})
  }

  fn mccs(r: &RunResult) -> Vec<Vec<Vec<String>>> {
    r.pairs
      .iter()
      .map(|p| p.mccs.iter().map(|m| r.taxa.names_of(m)).collect())
      .collect()
  }

  fn clades(t: &Tree) -> BTreeSet<BTreeSet<String>> {
    t.internals()
      .into_iter()
      .filter(|&n| n != t.root)
      .map(|n| t.leaves_below(n).into_iter().map(|l| t.name(l).to_owned()).collect())
      .collect()
  }

  fn names(v: &[&[&str]]) -> Vec<Vec<String>> {
    v.iter().map(|m| m.iter().map(|&x| x.to_owned()).collect()).collect()
  }

  #[test]
  fn run_identical_trees_form_one_mcc_without_reassortment() {
    let r = run_trees(&[("ha", "((A,B),(C,D));"), ("na", "((A,B),(C,D));")]);
    assert_eq!(vec![names(&[&["A", "B", "C", "D"]])], mccs(&r));
    assert_eq!(Some(ArgOutcome::Built { reassortments: 0 }), r.arg_outcome());
  }

  #[test]
  fn run_moved_leaf_matches_reference() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    // Oracle: fixtures/doc_mccs_1.json (TreeKnit.jl): MCCs [X] and [A,B,C,D], one reassortment.
    assert_eq!(vec![names(&[&["X"], &["A", "B", "C", "D"]])], mccs(&r));
    assert_eq!(Some(ArgOutcome::Built { reassortments: 1 }), r.arg_outcome());
    assert_eq!(1, r.built_arg().unwrap().n_hybrids());
  }

  #[test]
  fn run_three_trees_give_every_pair_and_no_arg() {
    let t = "((A,B),(C,D));";
    let r = run_trees(&[("ha", t), ("na", t), ("pb2", t)]);
    let all = names(&[&["A", "B", "C", "D"]]);
    assert_eq!(vec![all.clone(), all.clone(), all], mccs(&r));
    let order: Vec<(usize, usize)> = r.pairs.iter().map(|p| (p.i, p.j)).collect();
    assert_eq!(vec![(0, 1), (0, 2), (1, 2)], order);
    assert_eq!(None, r.arg_outcome());
    assert!(r.arg.is_none());
  }

  #[test]
  fn run_keeps_the_input_trees_unresolved() {
    // Matched resolution copies na's splits into ha's polytomy; the input tree keeps it.
    let r = run_trees(&[("ha", "(A,B,C,D);"), ("na", "((A,B),(C,D));")]);
    assert_eq!(BTreeSet::new(), clades(&r.input_trees[0]));
    assert_eq!(
      clades(&newick::parse("((A,B),(C,D));", "t").unwrap()),
      clades(&r.trees[0])
    );
  }

  #[test]
  fn run_imputes_a_leaf_missing_from_one_tree() {
    let r = run_trees(&[("ha", "((A,B),(C,(D,P)));"), ("na", "((A,B),(C,D));")]);
    // Oracle: P is the sister of D in ha, the only tree that has it.
    assert_eq!(
      clades(&newick::parse("((A,B),(C,(D,P)));", "t").unwrap()),
      clades(&r.imputed[1])
    );
    assert_eq!(
      clades(&newick::parse("((A,B),(C,D));", "t").unwrap()),
      clades(&r.trees[1])
    );
  }

  #[test]
  fn run_reports_progress_to_the_end() {
    let fractions = RefCell::new(Vec::new());
    let texts = vec![
      TreeText {
        label: "ha".to_owned(),
        newick: HA.to_owned(),
      },
      TreeText {
        label: "na".to_owned(),
        newick: NA.to_owned(),
      },
    ];
    let opts = analysis::options(&Settings::default(), 2, false).unwrap();
    run(analysis::parse_trees(&texts).unwrap(), &opts, 1, &|p| {
      fractions.borrow_mut().push(p.fraction);
    });
    // The core reports fraction 1 exactly once, at the end of the run.
    assert_eq!(Some(&1.0), fractions.borrow().last());
  }

  #[test]
  fn arg_outcome_of_a_failed_arg_carries_its_message() {
    let mut r = run_trees(&[("ha", HA), ("na", NA)]);
    r.arg = Some(Err(ArgError("trees do not match within MCC 1".to_owned())));
    let expected = ArgOutcome::Failed {
      message: "ARG construction failed: trees do not match within MCC 1".to_owned(),
    };
    assert_eq!(Some(expected), r.arg_outcome());
    assert!(r.built_arg().is_none());
  }
}
