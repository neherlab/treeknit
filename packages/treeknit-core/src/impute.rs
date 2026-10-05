//! Placing leaves that are missing from one tree of a pair.
//!
//! Inference uses only the leaves both trees share. Afterwards, each maximal subtree of
//! `Tᵢ` whose leaves are all absent from `Tⱼ` joins the MCC of its attachment point `p`
//! (its parent), using the node→MCC map of `Tᵢ` with absent leaves as wildcards. If `p`
//! is not inside an MCC, the subtree joins the MCC below `p` whose root is closest to `p`
//! and is flagged ambiguous. Its imputed position in `Tⱼ` is a new child of the LCA of
//! the leaves of that MCC below `p` (or a new sister of that leaf if there is only one).

use crate::bits::Bits;
use crate::mcc_map::{leaf_mcc_map, map_mccs};
use crate::naive::Mcc;
use crate::tree::{NodeId, Tree};

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

/// Among MCCs with leaves in `below`, the one whose root is fewest edges from `p`;
/// ties by branch-length distance, then larger MCC, then lower index.
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
    let r = t.lca_of(mccs[m].iter().filter_map(|&x| leaf_of[x])).unwrap();
    let a = t.lca(p, r);
    let edges = t.depth(p) + t.depth(r) - 2 * t.depth(a);
    let dist = match (t.divtime(p, a), t.divtime(r, a)) {
      (Some(x), Some(y)) => x + y,
      _ => f64::INFINITY,
    };
    (edges, dist, std::cmp::Reverse(mccs[m].len()), m)
  };
  cand
    .into_iter()
    .min_by(|&a, &b| key(a).partial_cmp(&key(b)).unwrap())
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
