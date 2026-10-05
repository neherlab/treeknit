//! Layout of one tree: node coordinates, MCCs, and the flags of resolution and imputation.

use super::names::{shorten, unique_labels};
use super::{DrawNode, DrawTree, label_max_chars};
use std::collections::BTreeSet;
use treeknit_core::Tree;
use treeknit_core::mcc_map::map_mccs;

/// `tree` laid out for drawing, with `leaf_mcc[taxon]` the MCC of each leaf of the pair.
/// `input` is the parsed tree of the same index: an internal node whose name it lacks is
/// `added`, and a leaf it lacks is `imputed`. Node names are unique (see `unique_labels`):
/// imputation can graft a leaf next to an internal node of the same name.
pub(super) fn draw_tree(tree: &Tree, input: &Tree, leaf_mcc: &[Option<usize>]) -> DrawTree {
  let order = tree.preorder();
  let mut index = vec![0; tree.nodes.len()];
  for (i, &n) in order.iter().enumerate() {
    index[n] = i;
  }
  let mcc = map_mccs(tree, leaf_mcc);
  let input_names: BTreeSet<&str> = input.preorder().into_iter().map(|n| input.name(n)).collect();
  let input_taxa: BTreeSet<usize> = input.leaves().into_iter().filter_map(|n| input.node(n).taxon).collect();
  let names = unique_labels(
    order.iter().map(|&n| tree.name(n).to_owned()).collect(),
    &order.iter().map(|&n| tree.is_leaf(n)).collect::<Vec<_>>(),
  );
  let mut nodes: Vec<DrawNode> = order
    .iter()
    .zip(names)
    .map(|(&n, name)| {
      let node = tree.node(n);
      let leaf = tree.is_leaf(n);
      DrawNode {
        short_name: shorten(&name, label_max_chars()),
        name,
        parent: node.parent.map(|p| index[p]),
        children: node.children.iter().map(|&c| index[c]).collect(),
        branch_length: node.branch_length.filter(|b| b.is_finite()),
        x_div: 0.0,
        x_depth: 0.0,
        y: 0.0,
        leaf,
        clade_size: 1,
        added: !leaf && !input_names.contains(node.name.as_str()),
        imputed: leaf && node.taxon.is_some_and(|x| !input_taxa.contains(&x)),
        mcc: mcc[n],
        mcc_break: false,
      }
    })
    .collect();
  place(&mut nodes);
  DrawTree {
    label: tree.label.clone(),
    nodes,
  }
}

/// Fill `x_div`, `x_depth`, `y`, `clade_size`, and `mcc_break` of `nodes`, which are in preorder.
fn place(nodes: &mut [DrawNode]) {
  let mut rank = 0;
  for i in 0..nodes.len() {
    if nodes[i].leaf {
      nodes[i].y = row(rank);
      rank += 1;
    }
    if let Some(p) = nodes[i].parent {
      nodes[i].x_div = add_length(nodes[p].x_div, nodes[i].branch_length);
      nodes[i].mcc_break = nodes[i].mcc.is_some() && nodes[p].mcc != nodes[i].mcc;
    }
  }
  // Height: the largest number of branches down to a leaf.
  let mut height = vec![0_usize; nodes.len()];
  for i in (0..nodes.len()).rev() {
    let children = &nodes[i].children;
    if let (Some(&first), Some(&last)) = (children.first(), children.last()) {
      nodes[i].y = f64::midpoint(nodes[first].y, nodes[last].y);
      height[i] = 1 + children.iter().map(|&c| height[c]).max().unwrap_or(0);
      nodes[i].clade_size = children.iter().map(|&c| nodes[c].clade_size).sum();
    }
  }
  let top = height.first().copied().unwrap_or(0);
  for (node, h) in nodes.iter_mut().zip(height) {
    // The root is the highest node; the check holds in the shipped build, which does not check
    // for overflow.
    #[expect(clippy::expect_used, reason = "every node is below the root")]
    let below = top.checked_sub(h).expect("no node is higher than the root");
    node.x_depth = row(below);
  }
}

/// `x` plus the branch `length`; a missing, negative, or non-finite length counts as 0, and the
/// sum stays finite.
pub(super) fn add_length(x: f64, length: Option<f64>) -> f64 {
  let length = length.filter(|l| l.is_finite() && *l > 0.0).unwrap_or(0.0);
  (x + length).min(f64::MAX)
}

/// A row or step count as a coordinate.
#[expect(
  clippy::as_conversions,
  reason = "row and step counts are far below 2^53, so the conversion is exact"
)]
pub(super) fn row(n: usize) -> f64 {
  n as f64
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::newick;
  use pretty_assertions::assert_eq;
  use treeknit_core::Taxa;
  use treeknit_core::mcc_map::leaf_mcc_map;

  /// `newick` parsed with taxa assigned from its own leaves, and its taxa.
  fn tree(newick: &str) -> (Tree, Taxa) {
    let mut t = newick::parse(newick, "t").unwrap();
    let taxa = Taxa::from_trees(std::slice::from_ref(&t));
    t.assign_taxa(&taxa).unwrap();
    (t, taxa)
  }

  fn column<'a, T>(t: &'a DrawTree, f: impl Fn(&'a DrawNode) -> T) -> Vec<T> {
    t.nodes.iter().map(f).collect()
  }

  #[test]
  fn draw_tree_places_nodes_in_preorder() {
    let (t, taxa) = tree("((A:1,B:2)ab:0.5,C:3)r;");
    let d = draw_tree(&t, &t, &vec![None; taxa.len()]);
    assert_eq!(vec!["r", "ab", "A", "B", "C"], column(&d, |n| n.name.as_str()));
    assert_eq!(vec![None, Some(0), Some(1), Some(1), Some(0)], column(&d, |n| n.parent));
    assert_eq!(
      vec![vec![1, 4], vec![2, 3], vec![], vec![], vec![]],
      column(&d, |n| n.children.clone())
    );
    // Oracle: leaves ranked 0, 1, 2; ab at the midpoint of A and B, the root of ab and C.
    assert_eq!(vec![1.25, 0.5, 0.0, 1.0, 2.0], column(&d, |n| n.y));
    assert_eq!(vec![0.0, 0.5, 1.5, 2.5, 3.0], column(&d, |n| n.x_div));
    // Height 2 at the root: leaves at 2, ab one step left of its children, C at 2 too.
    assert_eq!(vec![0.0, 1.0, 2.0, 2.0, 2.0], column(&d, |n| n.x_depth));
    // Oracle: three leaves below the root, two below ab.
    assert_eq!(vec![3, 2, 1, 1, 1], column(&d, |n| n.clade_size));
  }

  #[test]
  fn draw_tree_shortens_long_names_to_the_label_length_of_the_drawing_rules() {
    let long = "A/New York/392/2004/H3N2/segment-4/hemagglutinin";
    let (t, taxa) = tree(&format!("('{long}',B)r;"));
    let d = draw_tree(&t, &t, &vec![None; taxa.len()]);
    // Oracle: 40 characters: the first 20, the ellipsis, the last 19.
    let expected = "A/New York/392/2004/\u{2026}ent-4/hemagglutinin";
    assert_eq!(
      (long, expected),
      (d.nodes[1].name.as_str(), d.nodes[1].short_name.as_str())
    );
    assert_eq!(40, d.nodes[1].short_name.chars().count());
    assert_eq!("B", d.nodes[2].short_name);
  }

  #[test]
  fn draw_tree_counts_missing_negative_and_non_finite_lengths_as_zero() {
    let (t, taxa) = tree("((A:-1,B:inf)ab,C:NaN)r;");
    let d = draw_tree(&t, &t, &vec![None; taxa.len()]);
    assert_eq!(vec![0.0; 5], column(&d, |n| n.x_div));
    assert_eq!(
      vec![None, None, Some(-1.0), None, None],
      column(&d, |n| n.branch_length)
    );
  }

  #[test]
  fn draw_tree_keeps_a_huge_divergence_finite() {
    let (t, taxa) = tree("((A:1e308)a:1e308,B:1)r;");
    let d = draw_tree(&t, &t, &vec![None; taxa.len()]);
    assert!(d.nodes.iter().all(|n| n.x_div.is_finite()));
  }

  #[test]
  fn draw_tree_marks_the_branch_above_x_as_a_reassortment() {
    let (t, taxa) = tree("((A,B)ab,(C,(D,X)dx)cdx)r;");
    let ids = |v: &[&str]| v.iter().map(|s| taxa.index[*s]).collect::<Vec<_>>();
    let mccs = vec![ids(&["X"]), ids(&["A", "B", "C", "D"])];
    let d = draw_tree(&t, &t, &leaf_mcc_map(&mccs, taxa.len()));
    let breaks: Vec<&str> = d
      .nodes
      .iter()
      .filter(|n| n.mcc_break)
      .map(|n| n.name.as_str())
      .collect();
    assert_eq!(vec!["X"], breaks);
  }

  #[test]
  fn draw_tree_breaks_each_child_branch_below_a_root_without_mcc() {
    // Oracle: map_mccs gives the root no MCC when its children's MCCs differ.
    let (t, taxa) = tree("((A,B)ab,(C,D)cd)r;");
    let ids = |v: &[&str]| v.iter().map(|s| taxa.index[*s]).collect::<Vec<_>>();
    let mccs = vec![ids(&["A", "B"]), ids(&["C", "D"])];
    let d = draw_tree(&t, &t, &leaf_mcc_map(&mccs, taxa.len()));
    assert_eq!(None, d.nodes[0].mcc);
    let breaks: Vec<&str> = d
      .nodes
      .iter()
      .filter(|n| n.mcc_break)
      .map(|n| n.name.as_str())
      .collect();
    assert_eq!(vec!["ab", "cd"], breaks);
  }

  #[test]
  fn draw_tree_flags_added_nodes_and_imputed_leaves_against_the_input() {
    let (t, taxa) = tree("((A,B)RESOLVED_1,(C,P)x)r;");
    let mut input = newick::parse("(A,B,C)r;", "t").unwrap();
    input.assign_taxa(&taxa).unwrap();
    let d = draw_tree(&t, &input, &vec![None; taxa.len()]);
    let added: Vec<&str> = d.nodes.iter().filter(|n| n.added).map(|n| n.name.as_str()).collect();
    let imputed: Vec<&str> = d.nodes.iter().filter(|n| n.imputed).map(|n| n.name.as_str()).collect();
    assert_eq!(vec!["RESOLVED_1", "x"], added);
    assert_eq!(vec!["P"], imputed);
  }

  #[test]
  fn draw_tree_compares_names_so_an_input_label_like_a_resolved_node_is_not_added() {
    let (t, taxa) = tree("((A,B)RESOLVED_3,C)r;");
    let d = draw_tree(&t, &t, &vec![None; taxa.len()]);
    assert!(d.nodes.iter().all(|n| !n.added));
  }
}
