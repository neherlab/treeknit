//! Placing leaves that are missing from one tree of a pair.
//!
//! Inference uses only the leaves both trees share. Afterwards, each maximal subtree of
//! `Tᵢ` whose leaves are all absent from `Tⱼ` joins the MCC of its attachment point `p`
//! (its parent), using the node→MCC map of `Tᵢ` with absent leaves as wildcards. If `p`
//! is not inside an MCC, the subtree joins the MCC below `p` whose root is closest to `p`
//! and is flagged ambiguous. Its imputed position in `Tⱼ` is a new child of the LCA of
//! the leaves of that MCC below `p` (or a new sister of that leaf if there is only one).

#![expect(
  clippy::expect_used,
  clippy::unwrap_used,
  reason = "findings from before the strict lint set; kb/issues/N-lint-baseline.md tracks their removal"
)]

use crate::bits::Bits;
use crate::mcc_map::{leaf_mcc_map, map_mccs};
use crate::naive::Mcc;
use crate::tree::{NodeId, Tree};
use ordered_float::OrderedFloat;
use std::cmp::Reverse;

/// A subtree of the source tree attached to an MCC of the pair.
#[derive(Clone, Debug)]
pub struct Attachment {
  /// Index of the tree the subtree comes from.
  pub source: usize,
  /// Root of the subtree in the source tree.
  pub node: NodeId,
  /// Leaves of the subtree.
  pub leaves: Vec<usize>,
  /// Index into the pair's MCC list (before attached leaves were added).
  pub mcc: usize,
  /// Shared leaves of the MCC below the attachment point; the subtree is placed at their LCA.
  pub anchor: Vec<usize>,
  pub ambiguous: bool,
}

/// Attachments for the leaves of `t` (tree index `source`) that are not in `shared`.
pub fn attach_private(t: &Tree, source: usize, shared: &Bits, mccs: &[Mcc], n_taxa: usize) -> Vec<Attachment> {
  let clades = t.clades(n_taxa);
  let leaf_mcc = leaf_mcc_map(mccs, n_taxa);
  let map = map_mccs(t, &leaf_mcc);
  let is_private = |n: NodeId| clades[n].is_disjoint(shared);
  let leaf_of = t.leaf_of(n_taxa);
  let mut out = Vec::new();
  for n in t.preorder() {
    let Some(p) = t.parent(n) else { continue };
    if !is_private(n) || is_private(p) {
      continue;
    }
    let below: Vec<usize> = clades[p].intersection(shared).collect();
    let (mcc, ambiguous) = match map[p] {
      Some(m) => (m, false),
      None => (closest_mcc(t, p, &below, &leaf_mcc, mccs, &leaf_of), true),
    };
    out.push(Attachment {
      source,
      node: n,
      leaves: clades[n].ones().collect(),
      mcc,
      anchor: below.into_iter().filter(|&x| leaf_mcc[x] == Some(mcc)).collect(),
      ambiguous,
    });
  }
  out
}

/// Among MCCs with leaves in `below`, the one whose root is fewest edges from `p`; ties by
/// branch-length distance, then larger MCC, then lower index. A distance that is missing,
/// negative, or not finite counts as infinite, as in the branch likelihood: the rule applies to
/// the summed path length, so a path with a negative edge and a positive sum keeps its sum.
fn closest_mcc(
  t: &Tree,
  p: NodeId,
  below: &[usize],
  leaf_mcc: &[Option<usize>],
  mccs: &[Mcc],
  leaf_of: &[Option<NodeId>],
) -> usize {
  let mut cand: Vec<usize> = below.iter().filter_map(|&x| leaf_mcc[x]).collect();
  cand.sort_unstable();
  cand.dedup();
  let key = |m: usize| {
    let mcc_root = t.lca_of(mccs[m].iter().filter_map(|&x| leaf_of[x])).unwrap();
    let ancestor = t.lca(p, mcc_root);
    let edges = t.depth(p) + t.depth(mcc_root) - 2 * t.depth(ancestor);
    let dist = t
      .divtime(p, ancestor)
      .zip(t.divtime(mcc_root, ancestor))
      .map(|(up, down)| up + down)
      .filter(|d| d.is_finite() && *d >= 0.0)
      .unwrap_or(f64::INFINITY);
    // OrderedFloat orders -0.0 and +0.0 as equal, so the tie goes to the next key.
    (edges, OrderedFloat(dist), Reverse(mccs[m].len()), m)
  };
  cand
    .into_iter()
    .min_by_key(|&m| key(m))
    .expect("no MCC below attachment point")
}

/// Graft copies of attached subtrees from `source` trees into `target`.
pub fn graft_attachments(target: &mut Tree, trees: &[Tree], atts: &[&Attachment], n_taxa: usize) {
  let leaf_of = target.leaf_of(n_taxa);
  let mut label = target.fresh_index("IMPUTED");
  for a in atts {
    let mut q = target
      .lca_of(a.anchor.iter().filter_map(|&x| leaf_of[x]))
      .expect("anchor not in target tree");
    if target.is_leaf(q) {
      // A single anchor leaf: the subtree becomes its sister.
      // The new parent sits right above the anchor leaf; the leaf keeps its length.
      let t = target.nodes[q].branch_length;
      q = target.insert_above(q, format!("IMPUTED_{label}"), t);
      label += 1;
    }
    let src = &trees[a.source];
    let root = copy_subtree(target, src, a.node, &mut label);
    target.nodes[root].branch_length = None;
    target.attach(q, root);
  }
}

fn copy_subtree(dst: &mut Tree, src: &Tree, n: NodeId, label: &mut usize) -> NodeId {
  let s = &src.nodes[n];
  let name = if src.is_leaf(n) {
    s.name.clone()
  } else {
    *label += 1;
    format!("IMPUTED_{}", *label - 1)
  };
  let c = dst.add_node(name, s.branch_length);
  dst.nodes[c].taxon = s.taxon;
  for &ch in src.children(n) {
    let x = copy_subtree(dst, src, ch, label);
    dst.attach(c, x);
  }
  c
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::tree::test_util::trees;

  #[test]
  fn private_leaf_joins_mcc_of_neighbours() {
    // P is missing from tree 2. In tree 1 it sits with (C,D), inside the large MCC.
    let (ts, taxa) = trees(&["((A,B),(X,(C,(D,P))));", "((A,(B,X)),(C,D));"]);
    let id = |s: &str| taxa.index[s];
    let shared = ts[1].leaf_set(taxa.len());
    let mccs = vec![vec![id("X")], vec![id("A"), id("B"), id("C"), id("D")]];
    let a = attach_private(&ts[0], 0, &shared, &mccs, taxa.len());
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].mcc, 1);
    assert!(!a[0].ambiguous);
    assert_eq!(a[0].anchor, vec![id("D")]);

    let mut t2 = ts[1].clone();
    let all = ts;
    graft_attachments(&mut t2, &all, &[&a[0]], taxa.len());
    assert!(t2.check());
    assert!(t2.leaf_names().contains(&"P".to_owned()));
  }

  #[test]
  fn private_leaf_at_boundary_is_ambiguous() {
    // P hangs at the root, whose children belong to different MCCs.
    let (ts, taxa) = trees(&["(P,((A,B),(C,D)),X);", "((A,(B,X)),(C,D));"]);
    let id = |s: &str| taxa.index[s];
    let shared = ts[1].leaf_set(taxa.len());
    let mccs = vec![vec![id("X")], vec![id("A"), id("B"), id("C"), id("D")]];
    let a = attach_private(&ts[0], 0, &shared, &mccs, taxa.len());
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].mcc, 1);
    assert!(a[0].ambiguous);
  }

  /// The index of the MCC that [`closest_mcc`] picks for the root of `newick`, with the MCCs
  /// `mccs` named by their leaves.
  fn closest_to_root(newick: &str, mccs: &[&[&str]]) -> usize {
    let (ts, taxa) = trees(&[newick]);
    let n = taxa.len();
    let mccs: Vec<Mcc> = mccs
      .iter()
      .map(|m| m.iter().map(|x| taxa.index[*x]).collect())
      .collect();
    let below: Vec<usize> = (0..n).collect();
    let t = &ts[0];
    closest_mcc(t, t.root, &below, &leaf_mcc_map(&mccs, n), &mccs, &t.leaf_of(n))
  }

  #[test]
  fn closest_mcc_counts_a_nan_distance_as_missing() {
    // The NaN distance of (A,B) used to panic in the comparison; as a missing distance it loses
    // to the 1 of (C,D).
    let picked = closest_to_root("((A,B):NaN,(C,D):1);", &[&["A", "B"], &["C", "D"]]);
    assert_eq!(1, picked);
  }

  #[test]
  fn closest_mcc_counts_a_negative_distance_as_missing() {
    let picked = closest_to_root("((A,B):-1,(C,D):2);", &[&["A", "B"], &["C", "D"]]);
    assert_eq!(1, picked);
  }

  #[test]
  fn closest_mcc_ties_signed_zero_distances() {
    // Both MCCs have size 2 and distance zero, so the lower index wins whatever the sign.
    let picked = [
      closest_to_root("((A,B):-0,(C,D):0);", &[&["A", "B"], &["C", "D"]]),
      closest_to_root("((A,B):0,(C,D):-0);", &[&["A", "B"], &["C", "D"]]),
    ];
    assert_eq!([0, 0], picked);
  }

  #[test]
  fn closest_mcc_keeps_a_positive_sum_with_a_negative_edge() {
    // Both MCC roots are two edges below the root: (A,B) at -1 + 3 = 2, (C,D) at 1.5 + 1 = 2.5.
    // The single leaves E and F are further away.
    let picked = closest_to_root(
      "(((A,B):-1,E:10):3,((C,D):1.5,F:10):1);",
      &[&["A", "B"], &["C", "D"], &["E"], &["F"]],
    );
    assert_eq!(0, picked);
  }

  #[test]
  fn private_cherry_is_one_unit() {
    let (ts, taxa) = trees(&["((A,B),(C,(P,Q)));", "((A,B),C);"]);
    let shared = ts[1].leaf_set(taxa.len());
    let mccs = vec![(0..taxa.len()).filter(|&x| shared.contains(x)).collect()];
    let a = attach_private(&ts[0], 0, &shared, &mccs, taxa.len());
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].leaves.len(), 2);
  }
}
