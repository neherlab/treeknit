//! The lengths at which a drawing shows the branches of splits that some trees of the run lack:
//! the mean over the trees, weighted by their sequence lengths.

use crate::run::RunResult;
use std::collections::{BTreeMap, BTreeSet};
use treeknit_core::Tree;
use treeknit_core::bits::{self, Bits};

/// The length at which each node of `tree`, a drawn version of tree `i` of `run`, has its branch
/// drawn, by node id; `None` for a branch drawn at its own length.
///
/// The split of a node is compared with each other tree `k` of the run on the leaves of one MCC of
/// the pair of `i` and `k`: the MCC that holds the most leaves of the node's clade, restricted to
/// the leaves that `tree` and the input tree `k` share. Tree `k` has the split when one of its
/// nodes has the same clade on these leaves; the length of the split is the summed length of all
/// such nodes, because leaves outside the MCC can divide one branch of the MCC into several. Two
/// trees agree on the topology of an MCC, so a split that tree `k` lacks lies in a polytomy of
/// tree `k`. A split of fewer than two of these leaves, or of all of them, tells nothing about
/// tree `k`, which then takes no part.
///
/// A split that some tree lacks gets the mean $\sum_k L_k l_k / \sum_k L_k$ over `tree` and the
/// input trees of the other trees, with $l_k$ the length of the split in tree `k`, 0 when the tree
/// lacks it, and $L_k$ the sequence length of tree `k` (`Options.seq_lengths`). A tree lacks it
/// when it has a polytomy there, or, for `tree`, when resolution inserted the node with length 0.
/// This is the mean edge length of consensus trees with an absent edge counted as 0 (phytools
/// `consensus.edges` with `if.absent = "zero"`), weighted by sequence length. A tree without a
/// length for the split takes no part. A split that every tree has keeps its own length.
pub(super) fn mean_lengths(run: &RunResult, i: usize, tree: &Tree) -> Vec<Option<f64>> {
  let n_taxa = run.taxa.len();
  let input = &run.input_trees[i];
  let input_names: BTreeSet<&str> = input.preorder().into_iter().map(|n| input.name(n)).collect();
  let clades = tree.clades(n_taxa);
  let leaves = tree.leaf_set(n_taxa);
  let others: Vec<Other> = (0..run.input_trees.len())
    .filter(|&k| k != i)
    .filter_map(|k| Other::new(run, i, k, &leaves))
    .collect();
  let weight = |k: usize| run.opts.seq_lengths[k];
  (0..tree.nodes.len())
    .map(|n| {
      if tree.is_leaf(n) || tree.parent(n).is_none() {
        return None;
      }
      let own = tree.node(n).branch_length.filter(|l| l.is_finite()).map(|l| l.max(0.0));
      let mut missing = !input_names.contains(tree.name(n)) && own.is_some_and(|l| l <= 0.0);
      let mut mean = Mean::default();
      if let Some(l) = own {
        mean.add(weight(i), l);
      }
      for other in &others {
        match other.split(&clades[n]) {
          Split::Uninformative | Split::Present(None) => {},
          Split::Absent => {
            missing = true;
            mean.add(weight(other.k), 0.0);
          },
          Split::Present(Some(l)) => mean.add(weight(other.k), l),
        }
      }
      if missing { mean.value() } else { None }
    })
    .collect()
}

/// What another tree shows of a split.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Split {
  /// The split has fewer than two leaves of its MCC, or all of them.
  Uninformative,
  /// The tree lacks the split: a polytomy holds it.
  Absent,
  /// The tree has the split, with its length; `None` when a node of it has no length.
  Present(Option<f64>),
}

/// Another tree `k` of the run, prepared for comparisons with the drawn tree.
struct Other {
  k: usize,
  /// The leaves of each MCC of the pair of the drawn tree and tree `k` that both trees have.
  masks: Vec<Bits>,
  /// For each MCC, the informative splits of the input tree `k` on its mask, with their lengths.
  splits: Vec<BTreeMap<Bits, Option<f64>>>,
}

impl Other {
  /// Tree `k` compared with tree `i`, whose drawn version has the leaves `leaves`; `None` when the
  /// run has no MCCs of the pair, or the input tree `k` has no branch lengths.
  fn new(run: &RunResult, i: usize, k: usize, leaves: &Bits) -> Option<Other> {
    let n_taxa = run.taxa.len();
    let pair = run.pairs.iter().find(|p| (p.i, p.j) == (i.min(k), i.max(k)))?;
    let input = &run.input_trees[k];
    let branches: Vec<usize> = input
      .preorder()
      .into_iter()
      .filter(|&n| input.parent(n).is_some())
      .collect();
    if !branches
      .iter()
      .any(|&n| input.node(n).branch_length.is_some_and(f64::is_finite))
    {
      return None;
    }
    let shared = bits::and(leaves, &input.leaf_set(n_taxa));
    let masks: Vec<Bits> = pair
      .mccs
      .iter()
      .map(|m| bits::and(&bits::from_iter(n_taxa, m.iter().copied()), &shared))
      .collect();
    let clades = input.clades(n_taxa);
    let splits = masks
      .iter()
      .map(|mask| {
        let mut splits: BTreeMap<Bits, Option<f64>> = BTreeMap::new();
        for &n in &branches {
          let split = bits::and(&clades[n], mask);
          if informative(&split, mask) {
            let length = input
              .node(n)
              .branch_length
              .filter(|l| l.is_finite())
              .map(|l| l.max(0.0));
            let total = splits.entry(split).or_insert(Some(0.0));
            *total = total.zip(length).map(|(a, b)| a + b);
          }
        }
        splits
      })
      .collect();
    Some(Other { k, masks, splits })
  }

  /// What tree `k` shows of the split with clade `clade` in the drawn tree.
  fn split(&self, clade: &Bits) -> Split {
    let overlap = |mask: &Bits| clade.intersection(mask).count();
    let Some((m, mask)) = self
      .masks
      .iter()
      .enumerate()
      .rev()
      .max_by_key(|(_, mask)| overlap(mask))
    else {
      return Split::Uninformative;
    };
    let split = bits::and(clade, mask);
    if !informative(&split, mask) {
      return Split::Uninformative;
    }
    self.splits[m].get(&split).map_or(Split::Absent, |&l| Split::Present(l))
  }
}

/// `split` divides `mask`: it has at least two of its leaves, and not all of them.
fn informative(split: &Bits, mask: &Bits) -> bool {
  let n = split.count_ones(..);
  n >= 2 && n < mask.count_ones(..)
}

/// A weighted mean, built one value at a time.
#[derive(Default)]
struct Mean {
  sum: f64,
  weight: f64,
}

impl Mean {
  fn add(&mut self, weight: f64, value: f64) {
    self.sum += weight * value;
    self.weight += weight;
  }

  /// The mean; `None` before any value.
  fn value(&self) -> Option<f64> {
    (self.weight > 0.0).then(|| self.sum / self.weight)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::analysis::{self, Settings, TreeText};
  use crate::run;
  use pretty_assertions::assert_eq;

  const HA_POLYTOMY: &str = "((A:1,B:1,C:1):1,(D:1,E:1):1);";
  const NA_HALF: &str = "((A:1,(B:1,C:1):0.5):1,(D:1,E:1):1);";

  fn run_with(trees: &[&str], seq_lengths: Option<Vec<f64>>) -> RunResult {
    let texts: Vec<TreeText> = trees
      .iter()
      .enumerate()
      .map(|(i, newick)| TreeText {
        label: format!("t{i}"),
        newick: (*newick).to_owned(),
      })
      .collect();
    let settings = Settings {
      seq_lengths,
      ..Settings::default()
    };
    let opts = analysis::options(&settings, texts.len(), false).unwrap();
    run::run(analysis::parse_trees(&texts).unwrap(), &opts, settings.seed, &|_| {})
  }

  /// The names of the nodes of final tree `i` that have a mean length, with that length.
  fn means(run: &RunResult, i: usize) -> Vec<(String, f64)> {
    let tree = &run.trees[i];
    mean_lengths(run, i, tree)
      .into_iter()
      .enumerate()
      .filter_map(|(n, mean)| Some((leaves_below(tree, n), mean?)))
      .collect()
  }

  /// The leaf names below node `n`, sorted and joined, as a name that does not depend on labels.
  fn leaves_below(tree: &Tree, n: usize) -> String {
    let mut names: Vec<&str> = tree.leaves_below(n).into_iter().map(|l| tree.name(l)).collect();
    names.sort_unstable();
    names.concat()
  }

  #[test]
  fn mean_lengths_give_a_resolved_split_and_its_source_the_mean_of_both_trees() {
    // Oracle: (0 + 0.5) / 2 in t0, where resolution inserted (B,C), and in t1, which has it.
    let r = run_with(&[HA_POLYTOMY, NA_HALF], None);
    let expected = vec![("BC".to_owned(), 0.25)];
    assert_eq!((expected.clone(), expected), (means(&r, 0), means(&r, 1)));
  }

  #[test]
  fn mean_lengths_weight_each_tree_by_its_sequence_length() {
    // Oracle: (3 * 0 + 1 * 0.5) / (3 + 1).
    let r = run_with(&[HA_POLYTOMY, NA_HALF], Some(vec![3.0, 1.0]));
    assert_eq!(vec![("BC".to_owned(), 0.125)], means(&r, 0));
  }

  #[test]
  fn mean_lengths_average_over_every_tree_of_the_run() {
    // Oracle: (0.5 + 0.25 + 0) / 3 in each of the three trees.
    let r = run_with(&[NA_HALF, "((A:1,(B:1,C:1):0.25):1,(D:1,E:1):1);", HA_POLYTOMY], None);
    let expected = vec![("BC".to_owned(), 0.25)];
    assert_eq!(
      vec![expected.clone(), expected.clone(), expected],
      (0..3).map(|i| means(&r, i)).collect::<Vec<_>>()
    );
  }

  #[test]
  fn mean_lengths_keep_the_length_of_a_split_that_every_tree_has() {
    let r = run_with(&[NA_HALF, "((A:1,(B:1,C:1):0.25):1,(D:1,E:1):1);"], None);
    assert_eq!(Vec::<(String, f64)>::new(), means(&r, 0));
  }

  #[test]
  fn mean_lengths_leave_out_a_tree_without_branch_lengths() {
    // t1 has no lengths, so the resolved split of t0 has only its own length 0.
    let r = run_with(&[HA_POLYTOMY, "((A,(B,C)),(D,E));"], None);
    assert_eq!(vec![("BC".to_owned(), 0.0)], means(&r, 0));
  }

  #[test]
  fn split_is_uninformative_on_a_single_leaf_or_on_the_whole_mask() {
    let mask = bits::from_iter(4, [0, 1, 2]);
    let cases = [vec![0], vec![0, 1], vec![0, 1, 2], vec![0, 3]];
    let informative: Vec<bool> = cases
      .iter()
      .map(|c| informative(&bits::and(&bits::from_iter(4, c.iter().copied()), &mask), &mask))
      .collect();
    assert_eq!(vec![false, true, false, false], informative);
  }
}
