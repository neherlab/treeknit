//! Naive MCCs: maximal clades whose subtrees are identical in all trees.

#![expect(
  clippy::expect_used,
  clippy::unwrap_used,
  reason = "findings from before the strict lint set; kb/issues/N-lint-baseline.md tracks their removal"
)]

use crate::bits::Bits;
use crate::tree::{NodeId, Tree};

pub type Mcc = Vec<usize>;

/// Sort MCC members ascending and MCCs by (size, first member).
pub fn sort_mccs(mut mccs: Vec<Mcc>) -> Vec<Mcc> {
  for m in &mut mccs {
    m.sort_unstable();
  }
  mccs.sort_by(|a, b| a.len().cmp(&b.len()).then(a[0].cmp(&b[0])));
  mccs
}

/// Naive MCCs of `trees`, which must all have the same leaf set (taxa in `0..n_taxa`).
pub fn naive_mccs(trees: &[&Tree], n_taxa: usize) -> Vec<Mcc> {
  let clades: Vec<Vec<Bits>> = trees.iter().map(|t| t.clades(n_taxa)).collect();
  let leaf_of: Vec<Vec<Option<NodeId>>> = trees.iter().map(|t| t.leaf_of(n_taxa)).collect();
  let leaves: Vec<usize> = (0..n_taxa).filter(|&x| leaf_of[0][x].is_some()).collect();
  let mut coherent = vec![false; trees[0].nodes.len()];
  let mut visited = vec![false; n_taxa];
  let mut out = Vec::new();

  for &x in &leaves {
    if visited[x] {
      continue;
    }
    let mut croot: Vec<NodeId> = leaf_of
      .iter()
      .map(|l| l[x].expect("trees do not share leaves"))
      .collect();
    let mut members: Vec<usize> = vec![x];
    while croot.iter().zip(trees).all(|(&n, t)| t.parent(n).is_some()) {
      let nroot: Vec<NodeId> = croot.iter().zip(trees).map(|(&n, t)| t.parent(n).unwrap()).collect();
      let same = (1..trees.len()).all(|k| clades[0][nroot[0]] == clades[k][nroot[k]]);
      if !same || !is_coherent(trees, &clades, &nroot, &mut coherent) {
        break;
      }
      members = clades[0][nroot[0]].ones().collect();
      croot = nroot;
    }
    for &m in &members {
      visited[m] = true;
    }
    out.push(members);
  }
  sort_mccs(out)
}

/// Are the subtrees below `roots` (one node per tree) identical?
/// `memo` caches tree-0 nodes already known to be coherent.
///
/// Depth-first over the matched nodes with an explicit stack, because identical subtrees can be
/// deeper than the call stack allows. Each frame holds one node per tree and the position of the
/// next child of its tree-0 node; a node enters `memo` when all its children are checked.
fn is_coherent(trees: &[&Tree], clades: &[Vec<Bits>], roots: &[NodeId], memo: &mut [bool]) -> bool {
  let t0 = trees[0];
  let mut stack: Vec<(Vec<NodeId>, usize)> = Vec::new();
  if !memo[roots[0]] {
    if !same_child_count(trees, roots) {
      return false;
    }
    stack.push((roots.to_vec(), 0));
  }
  while let Some((nodes, next)) = stack.last_mut() {
    let Some(&c) = t0.children(nodes[0]).get(*next) else {
      memo[nodes[0]] = true;
      stack.pop();
      continue;
    };
    *next += 1;
    if t0.is_leaf(c) {
      let x = t0.taxon(c);
      for k in 1..nodes.len() {
        let t = trees[k];
        if !t.children(nodes[k]).iter().any(|&d| t.is_leaf(d) && t.taxon(d) == x) {
          return false;
        }
      }
    } else {
      let mut matched = vec![c];
      for k in 1..nodes.len() {
        let t = trees[k];
        match t
          .children(nodes[k])
          .iter()
          .find(|&&d| !t.is_leaf(d) && clades[k][d] == clades[0][c])
        {
          Some(&d) => matched.push(d),
          None => return false,
        }
      }
      if memo[c] {
        continue;
      }
      if !same_child_count(trees, &matched) {
        return false;
      }
      stack.push((matched, 0));
    }
  }
  true
}

/// Do the nodes `nodes` (one per tree) have the same number of children?
fn same_child_count(trees: &[&Tree], nodes: &[NodeId]) -> bool {
  let nc = trees[0].children(nodes[0]).len();
  (1..nodes.len()).all(|k| trees[k].children(nodes[k]).len() == nc)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::tree::test_util::{DEEP, caterpillar, numbered, on_small_stack, trees, with_taxa};

  fn names(m: &[Mcc], taxa: &crate::tree::Taxa) -> Vec<Vec<String>> {
    m.iter().map(|x| taxa.names_of(x)).collect()
  }

  #[test]
  fn doc_example() {
    let (ts, taxa) = trees(&["(((A1,A2),(B1,B2)),(C1,C2));", "(((A1,A2),(C1,C2)),(B1,B2));"]);
    let m = naive_mccs(&[&ts[0], &ts[1]], taxa.len());
    assert_eq!(
      names(&m, &taxa),
      vec![vec!["A1", "A2"], vec!["B1", "B2"], vec!["C1", "C2"]]
    );
  }

  #[test]
  fn identical_trees_give_one_mcc() {
    let (ts, taxa) = trees(&["((A,B),(C,D));", "((C,D),(B,A));"]);
    let m = naive_mccs(&[&ts[0], &ts[1]], taxa.len());
    assert_eq!(m.len(), 1);
  }

  #[test]
  fn identical_deep_trees_give_one_mcc() {
    let names = numbered("L", DEEP);
    let (ts, taxa) = with_taxa(vec![caterpillar(&names), caterpillar(&names)]);
    let m = on_small_stack(|| naive_mccs(&[&ts[0], &ts[1]], taxa.len()));
    // Oracle: the module doc; identical trees are one subtree identical in all trees.
    let expected: Vec<Mcc> = vec![(0..taxa.len()).collect()];
    assert_eq!(expected, m);
  }
}
