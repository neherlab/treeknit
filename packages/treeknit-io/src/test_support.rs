//! Helpers shared by the unit tests of the crate.

use crate::analysis::{self, Settings, TreeText};
use crate::newick;
use crate::run::{self, RunResult};
use std::collections::BTreeSet;
use treeknit_core::Tree;

/// The run of the `(label, newick)` trees with the default settings.
pub(crate) fn run_trees(trees: &[(&str, &str)]) -> RunResult {
  run_with(trees, &Settings::default())
}

/// The run of the `(label, newick)` trees with the default settings, or `None` when the trees do
/// not validate.
pub(crate) fn try_run(trees: &[(&str, &str)]) -> Option<RunResult> {
  let texts = texts(trees);
  let settings = Settings::default();
  let parsed = analysis::parse_trees(&texts).ok()?;
  let opts = analysis::options(&settings, texts.len(), false).ok()?;
  Some(run::run(parsed, &opts, settings.seed, &|_| {}))
}

/// The run of the `(label, newick)` trees with `settings`.
pub(crate) fn run_with(trees: &[(&str, &str)], settings: &Settings) -> RunResult {
  let texts = texts(trees);
  let opts = analysis::options(settings, texts.len(), false).unwrap();
  run::run(analysis::parse_trees(&texts).unwrap(), &opts, settings.seed, &|_| {})
}

/// The tree texts of `(label, newick)` pairs.
pub(crate) fn texts(trees: &[(&str, &str)]) -> Vec<TreeText> {
  trees
    .iter()
    .map(|(label, newick)| TreeText::new(*label, *newick))
    .collect()
}

/// The clades of the Newick tree `newick`, as [`clades_of`] gives them.
pub(crate) fn clades(newick: &str) -> BTreeSet<BTreeSet<String>> {
  clades_of(&newick::parse(newick, "t").unwrap())
}

/// The clades of the internal nodes of `t` below its root, as sets of leaf names.
pub(crate) fn clades_of(t: &Tree) -> BTreeSet<BTreeSet<String>> {
  t.internals()
    .into_iter()
    .filter(|&n| n != t.root)
    .map(|n| t.leaves_below(n).into_iter().map(|l| t.name(l).to_owned()).collect())
    .collect()
}

/// Owned copies of `names`.
pub(crate) fn names(names: &[&str]) -> Vec<String> {
  names.iter().map(|&n| n.to_owned()).collect()
}
