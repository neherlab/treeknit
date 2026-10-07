//! The ARG of two trees laid out in one tree column.

use super::label_max_chars;
use super::names::{shorten, unique_labels};
use super::shapes::arg_shapes;
use super::tree::{add_length, row};
use super::{ArgEdge, ArgNodeView, ArgView, RootCase, Scale};
use crate::run::RunResult;
use std::collections::{BTreeMap, VecDeque};
use treeknit_core::arg::{Anc, Arg};

/// Label of the synthetic top root, as in `ARG/arg.nwk`.
const GLOBAL_ROOT: &str = "GlobalRoot";

/// The ARG of `run` laid out with the shapes for `scale`, or for `depth` when `scale` is `div` and
/// the ARG has no branch lengths (`ArgView.scale`); `None` when the run built no ARG (more
/// than two trees, or a failed construction).
pub fn arg_view(run: &RunResult, scale: Scale) -> Option<ArgView> {
  run.built_arg().map(|arg| layout(arg, scale))
}

fn layout(arg: &Arg, scale: Scale) -> ArgView {
  let (root_case, root) = top_root(arg);
  let g = Graph::new(arg, root_case);
  let order = topological_order(&g.parents, &g.children);
  let chain = chain_parents(&g, &order, root);
  let x_div = divergence(&g, &order, &chain);
  let x_depth = depth(&g.children, &order, root);
  let y = rows(arg, &g.children, &order);
  let mut children = g.children.clone();
  for c in &mut children {
    c.sort_by(|&a, &b| y[a].total_cmp(&y[b]));
  }
  let leaf: Vec<bool> = (0..g.len())
    .map(|n| arg.nodes.get(n).is_some_and(|a| a.is_leaf))
    .collect();
  let labels = unique_labels(
    (0..g.len())
      .map(|n| {
        arg
          .nodes
          .get(n)
          .map_or_else(|| GLOBAL_ROOT.to_owned(), |a| a.label.clone())
      })
      .collect(),
    &leaf,
  );
  let nodes: Vec<ArgNodeView> = labels
    .into_iter()
    .enumerate()
    .map(|(n, label)| {
      let node = arg.nodes.get(n);
      ArgNodeView {
        short_label: shorten(&label, label_max_chars()),
        label,
        parents: g.parents[n],
        children: children[n].clone(),
        tau: node.map_or([None, None], |a| a.tau.map(|t| t.filter(|x| x.is_finite()))),
        hybrid: node.is_some_and(|a| a.hybrid),
        leaf: leaf[n],
        segments: (0..2).filter(|&c| node.is_none_or(|a| a.has(c))).collect(),
        x_div: x_div[n],
        x_depth: x_depth[n],
        y: y[n],
      }
    })
    .collect();
  let edges = edges(&g, &chain, &nodes);
  let scale = scale.shown(nodes.iter().all(|n| n.x_div <= 0.0));
  let shapes = arg_shapes(&nodes, &edges, scale);
  ArgView {
    nodes,
    edges,
    root,
    root_case,
    scale,
    shapes,
  }
}

/// The root case and the top root, following the extended Newick writer of `crate::arg`: a
/// shared segment root is below the other segment's root, which is then the top root.
fn top_root(arg: &Arg) -> (RootCase, usize) {
  let [r0, r1] = arg.roots;
  if r0 == r1 {
    (RootCase::Shared, r0)
  } else if arg.nodes[r0].is_shared() {
    (RootCase::OneShared, r1)
  } else if arg.nodes[r1].is_shared() {
    (RootCase::OneShared, r0)
  } else {
    (RootCase::Synthetic, arg.nodes.len())
  }
}

/// Parents and children of the ARG nodes, with the synthetic top root appended as the last node
/// when the root case asks for it.
struct Graph<'a> {
  arg: &'a Arg,
  /// Parent per segment.
  parents: Vec<[Option<usize>; 2]>,
  children: Vec<Vec<usize>>,
}

impl<'a> Graph<'a> {
  fn new(arg: &'a Arg, root_case: RootCase) -> Graph<'a> {
    let mut parents: Vec<[Option<usize>; 2]> = arg
      .nodes
      .iter()
      .map(|n| {
        n.anc.map(|a| match a {
          Anc::Node(p) => Some(p),
          Anc::Absent | Anc::Root => None,
        })
      })
      .collect();
    let mut children: Vec<Vec<usize>> = arg.nodes.iter().map(|n| n.children.clone()).collect();
    if root_case == RootCase::Synthetic {
      let top = arg.nodes.len();
      let [r0, r1] = arg.roots;
      parents[r0][0] = Some(top);
      parents[r1][1] = Some(top);
      parents.push([None, None]);
      children.push(vec![r0, r1]);
    }
    Graph { arg, parents, children }
  }

  fn len(&self) -> usize {
    self.parents.len()
  }

  /// Length of the branch from node `n` to its parent of segment `c`; the edges of the
  /// synthetic top root have length 0.
  fn length(&self, n: usize, c: usize) -> Option<f64> {
    let synthetic = self.parents[n][c] == Some(self.arg.nodes.len());
    self.arg.nodes.get(n).and_then(|a| a.tau[c]).filter(|_| !synthetic)
  }
}

/// The nodes ordered parents before children (Kahn's algorithm over the distinct parents).
fn topological_order(parents: &[[Option<usize>; 2]], children: &[Vec<usize>]) -> Vec<usize> {
  let distinct = |p: &[Option<usize>; 2]| match p {
    [Some(a), Some(b)] if a == b => 1,
    _ => p.iter().flatten().count(),
  };
  let mut waiting: Vec<usize> = parents.iter().map(distinct).collect();
  let mut queue: VecDeque<usize> = (0..parents.len()).filter(|&n| waiting[n] == 0).collect();
  let mut order = Vec::with_capacity(parents.len());
  while let Some(n) = queue.pop_front() {
    order.push(n);
    for &c in &children[n] {
      waiting[c] -= 1;
      if waiting[c] == 0 {
        queue.push_back(c);
      }
    }
  }
  order
}

/// For each node, its parent along the chain to the top root `root` and that parent's segment:
/// the segment 0 parent when its chain reaches the top root, otherwise the segment 1 parent.
fn chain_parents(g: &Graph<'_>, order: &[usize], root: usize) -> Vec<Option<(usize, usize)>> {
  let mut reaches = vec![false; g.len()];
  let mut chain = vec![None; g.len()];
  for &n in order {
    chain[n] = (0..2).find_map(|c| g.parents[n][c].filter(|&p| reaches[p]).map(|p| (p, c)));
    reaches[n] = n == root || chain[n].is_some();
  }
  chain
}

/// Distance from the top root along the chain; a missing or negative length counts as 0.
fn divergence(g: &Graph<'_>, order: &[usize], chain: &[Option<(usize, usize)>]) -> Vec<f64> {
  let mut x = vec![0.0; g.len()];
  for &n in order {
    if let Some((p, c)) = chain[n] {
      x[n] = add_length(x[p], g.length(n, c));
    }
  }
  x
}

/// Cladogram position (see `ArgNodeView.x_depth`): the height of the top root minus the height of
/// each node, the largest number of edges down to a leaf.
fn depth(children: &[Vec<usize>], order: &[usize], root: usize) -> Vec<f64> {
  let mut height = vec![0_usize; children.len()];
  for &n in order.iter().rev() {
    if let Some(h) = children[n].iter().map(|&c| height[c]).max() {
      height[n] = h + 1;
    }
  }
  // Every node is below the top root, so no height exceeds the root's. The check holds in the
  // shipped build too, which does not check for overflow and would wrap a broken invariant.
  let top = height[root];
  #[expect(clippy::expect_used, reason = "every node is below the top root")]
  height
    .into_iter()
    .map(|h| row(top.checked_sub(h).expect("no node is higher than the top root")))
    .collect()
}

/// Leaf rank in the leaf order of the ARG's segment 0 tree, then of its segment 1 tree for the
/// leaves only that segment has; every other node at the midpoint of its first and last child.
fn rows(arg: &Arg, children: &[Vec<usize>], order: &[usize]) -> Vec<f64> {
  let mut rank: Vec<Option<usize>> = vec![None; children.len()];
  let mut next = 0;
  for c in 0..2 {
    let by_name: BTreeMap<&str, usize> = arg
      .tree_nodes
      .iter()
      .enumerate()
      .filter(|&(a, _)| arg.nodes[a].is_leaf)
      .filter_map(|(a, names)| Some((names[c].as_deref()?, a)))
      .collect();
    let tree = &arg.trees[c];
    for leaf in tree.leaves() {
      if let Some(&a) = by_name.get(tree.name(leaf)) {
        if rank[a].is_none() {
          rank[a] = Some(next);
          next += 1;
        }
      }
    }
  }
  // A node without children that neither tree places, in index order.
  for n in 0..children.len() {
    if children[n].is_empty() && rank[n].is_none() {
      rank[n] = Some(next);
      next += 1;
    }
  }
  let mut y: Vec<f64> = rank.iter().map(|r| r.map_or(0.0, row)).collect();
  for &n in order.iter().rev() {
    let ys = children[n].iter().map(|&c| y[c]);
    if let (Some(lo), Some(hi)) = (ys.clone().reduce(f64::min), ys.reduce(f64::max)) {
      y[n] = f64::midpoint(lo, hi);
    }
  }
  y
}

/// One edge per parent of each node, with the segments it carries. An edge into a hybrid node
/// that is not on the node's chain to the top root is a reticulation edge.
fn edges(g: &Graph<'_>, chain: &[Option<(usize, usize)>], nodes: &[ArgNodeView]) -> Vec<ArgEdge> {
  let mut out = Vec::new();
  for (child, parents) in g.parents.iter().enumerate() {
    let mut by_parent: Vec<(usize, Vec<usize>)> = Vec::new();
    for (c, p) in parents.iter().enumerate() {
      if let Some(p) = *p {
        match by_parent.iter_mut().find(|(q, _)| *q == p) {
          Some((_, segments)) => segments.push(c),
          None => by_parent.push((p, vec![c])),
        }
      }
    }
    for (parent, segments) in by_parent {
      let on_chain = chain[child].is_some_and(|(p, _)| p == parent);
      out.push(ArgEdge {
        parent,
        child,
        segments,
        reticulation: nodes[child].hybrid && !on_chain,
      });
    }
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_support::run_trees;

  use crate::display::EdgePath;

  use pretty_assertions::assert_eq;
  use rand::{Rng, SeedableRng};
  use rand_xoshiro::Xoshiro256PlusPlus;
  use rstest::rstest;
  use std::collections::BTreeSet;

  /// The invariants of every ARG view: one node per ARG node (plus the synthetic root), distinct
  /// leaf rows, every node reached once from the top root, edges consistent with the parents,
  /// x along the chain to the top root never left of the parent, and finite coordinates.
  fn check_invariants(r: &RunResult, v: &ArgView) {
    let arg = r.built_arg().unwrap();
    let synthetic = usize::from(v.root_case == RootCase::Synthetic);
    assert_eq!(arg.nodes.len() + synthetic, v.nodes.len());
    let leaf_rows: BTreeSet<u64> = v.nodes.iter().filter(|n| n.leaf).map(|n| n.y.to_bits()).collect();
    assert_eq!(v.nodes.iter().filter(|n| n.leaf).count(), leaf_rows.len());
    // Reached once: a walk over the edges from the top root visits each node exactly once.
    let order = topological_order(
      &v.nodes.iter().map(|n| n.parents).collect::<Vec<_>>(),
      &v.nodes.iter().map(|n| n.children.clone()).collect::<Vec<_>>(),
    );
    assert_eq!(v.nodes.len(), order.len());
    assert_eq!(Some(&v.root), order.first());
    assert_eq!([None, None], v.nodes[v.root].parents);
    // Each edge is the parent link of its child in its segments and a child link of its parent.
    let unlinked: Vec<usize> = (0..v.edges.len())
      .filter(|&i| {
        let e = &v.edges[i];
        !e.segments
          .iter()
          .all(|&c| v.nodes[e.child].parents[c] == Some(e.parent))
          || !v.nodes[e.parent].children.contains(&e.child)
      })
      .collect();
    assert_eq!(Vec::<usize>::new(), unlinked);
    // Each node but the top root has one edge on its chain, to a parent left of it, and finite
    // coordinates; the top root has none.
    let misplaced: Vec<&str> = v
      .nodes
      .iter()
      .enumerate()
      .filter(|&(i, n)| {
        let on_chain: Vec<&ArgEdge> = v.edges.iter().filter(|e| e.child == i && !e.reticulation).collect();
        match on_chain.as_slice() {
          [] => i != v.root,
          [e] => {
            let p = &v.nodes[e.parent];
            let finite = [n.x_div, n.x_depth, n.y].iter().all(|x| x.is_finite());
            i == v.root || n.x_div < p.x_div || n.x_depth <= p.x_depth || !finite
          },
          _ => true,
        }
      })
      .map(|(_, n)| n.label.as_str())
      .collect();
    assert_eq!(Vec::<&str>::new(), misplaced);
    // Each edge has its shape, a curve for a reticulation.
    let (expected, shapes): (Vec<_>, Vec<_>) = v
      .edges
      .iter()
      .enumerate()
      .zip(&v.shapes.edges)
      .map(|((i, e), s)| {
        let curve = matches!(s.path, EdgePath::Curve { .. });
        (
          (i, &e.segments, e.reticulation, e.reticulation),
          (s.edge, &s.segments, s.reticulation, curve),
        )
      })
      .unzip();
    assert_eq!((v.edges.len(), expected), (v.shapes.edges.len(), shapes));
    let hybrids = v.nodes.iter().filter(|n| n.hybrid).count();
    assert_eq!(hybrids, v.shapes.marks.len());
    assert_eq!(hybrids, v.edges.iter().filter(|e| e.reticulation).count());
  }

  #[test]
  fn arg_view_of_the_two_tree_example() {
    let r = run_trees(&[("ha", "((A,B),(C,(D,X)));"), ("na", "((A,(B,X)),(C,D));")]);
    let v = arg_view(&r, Scale::Div).unwrap();
    check_invariants(&r, &v);
    // Oracle: one reassortment, the hybrid above X.
    let hybrids: Vec<&ArgNodeView> = v.nodes.iter().filter(|n| n.hybrid).collect();
    assert_eq!(1, hybrids.len());
    let leaves: Vec<&str> = {
      let mut l: Vec<&ArgNodeView> = v.nodes.iter().filter(|n| n.leaf).collect();
      l.sort_by(|a, b| a.y.total_cmp(&b.y));
      l.into_iter().map(|n| n.label.as_str()).collect()
    };
    // Leaf order of the ARG's segment 0 tree.
    let expected = r.built_arg().unwrap().trees[0].leaf_names();
    assert_eq!(expected, leaves);
    // Labels of at most 40 characters are shown whole.
    assert!(v.nodes.iter().all(|n| n.short_label == n.label));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::shared_identical(      "((A,B),(C,D));", "((A,B),(C,D));", RootCase::Shared)]
  #[case::shared_moved_leaf(     "((A,B),(C,(D,X)));", "((A,(B,X)),(C,D));", RootCase::Shared)]
  #[case::shared_two_moves(      "((A,(B,C)),((D,E),(F,G)));", "(((A,F),C),((D,B),(E,G)));", RootCase::Shared)]
  #[case::shared_partial_overlap("((A,B),(C,(D,(E,P))));", "((A,(B,E)),(C,D));", RootCase::Shared)]
  #[case::shared_polytomy(       "(A,B,C,D);", "((A,B),(C,D));", RootCase::Shared)]
  #[case::one_shared_second(     "((A,B),(C,D));", "((A,C),(B,D));", RootCase::OneShared)]
  #[case::one_shared_first(      "(((A,B),C),D);", "(((C,D),A),B);", RootCase::OneShared)]
  #[case::synthetic(             "(A,((C,B),D));", "((D,(A,C)),B);", RootCase::Synthetic)]
  #[case::synthetic_singletons(  "((B,(E,A)),(C,D));", "((B,(D,A)),(E,C));", RootCase::Synthetic)]
  #[trace]
  fn arg_view_invariants_hold_for_each_root_case(
    #[case] ha: &str,
    #[case] na: &str,
    #[case] root_case: RootCase,
    #[values(Scale::Div, Scale::Depth)] scale: Scale,
  ) {
    let r = run_trees(&[("ha", ha), ("na", na)]);
    let v = arg_view(&r, scale).unwrap();
    assert_eq!(root_case, v.root_case);
    check_invariants(&r, &v);
  }

  /// Oracle: ARG/arg.nwk names the top root last; the synthetic one is GlobalRoot.
  #[rustfmt::skip]
  #[rstest]
  #[case::shared(           "((A,B),(C,(D,X)));", "((A,(B,X)),(C,D));")]
  #[case::one_shared_second("((A,B),(C,D));", "((A,C),(B,D));")]
  #[case::one_shared_first( "(((A,B),C),D);", "(((C,D),A),B);")]
  #[case::synthetic(        "(A,((C,B),D));", "((D,(A,C)),B);")]
  #[trace]
  fn arg_view_top_root_follows_the_extended_newick(#[case] ha: &str, #[case] na: &str) {
    let r = run_trees(&[("ha", ha), ("na", na)]);
    let v = arg_view(&r, Scale::Div).unwrap();
    let newick = crate::arg::extended_newick(r.built_arg().unwrap());
    let top = newick.trim_end_matches(';').rsplit(')').next().unwrap();
    let label = top.split(['[', '#', ':']).next().unwrap();
    assert_eq!(label, v.nodes[v.root].label);
  }

  #[rstest]
  #[trace]
  fn arg_view_invariants_hold_on_a_simulated_fixture(#[values(Scale::Div, Scale::Depth)] scale: Scale) {
    let fixture: serde_json::Value =
      serde_json::from_str(include_str!("../../../../fixtures/sim_k2_n50_r0.05.json")).unwrap();
    let trees: Vec<&str> = fixture["trees"]
      .as_array()
      .unwrap()
      .iter()
      .map(|t| t.as_str().unwrap())
      .collect();
    let r = run_trees(&[("a", trees[0]), ("b", trees[1])]);
    check_invariants(&r, &arg_view(&r, scale).unwrap());
  }

  #[test]
  fn arg_view_invariants_hold_on_random_trees() {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(3);
    let mut cases = BTreeSet::new();
    for _ in 0..300 {
      let n = rng.gen_range(4..9);
      let r = run_trees(&[("ha", &random_tree(&mut rng, n)), ("na", &random_tree(&mut rng, n))]);
      if let Some(v) = arg_view(&r, Scale::Div) {
        check_invariants(&r, &v);
        cases.insert(format!("{:?}", v.root_case));
      }
    }
    assert_eq!(3, cases.len(), "every root case occurs: {cases:?}");
  }

  /// A random binary tree on `n` leaves named `A`, `B`, ...
  fn random_tree(rng: &mut Xoshiro256PlusPlus, n: u8) -> String {
    let mut parts: Vec<String> = (0..n).map(|i| char::from(b'A' + i).to_string()).collect();
    while parts.len() > 1 {
      let a = parts.remove(rng.gen_range(0..parts.len()));
      let b = parts.remove(rng.gen_range(0..parts.len()));
      parts.push(format!("({a},{b})"));
    }
    format!("{};", parts[0])
  }

  #[test]
  fn arg_view_labels_are_unique_when_leaves_take_arg_node_labels() {
    // The synthetic root case, with leaves named like the synthetic root and an ARG node.
    let r = run_trees(&[
      ("ha", "(GlobalRoot,((C,ARGNode_1),D));"),
      ("na", "((D,(GlobalRoot,C)),ARGNode_1);"),
    ]);
    let v = arg_view(&r, Scale::Div).unwrap();
    let labels: BTreeSet<&str> = v.nodes.iter().map(|n| n.label.as_str()).collect();
    assert_eq!(v.nodes.len(), labels.len());
    assert!(labels.iter().all(|l| !l.is_empty()));
    let leaves: BTreeSet<&str> = v.nodes.iter().filter(|n| n.leaf).map(|n| n.label.as_str()).collect();
    assert_eq!(BTreeSet::from(["ARGNode_1", "C", "D", "GlobalRoot"]), leaves);
  }

  #[test]
  #[expect(clippy::float_cmp, reason = "rows are whole numbers, and their midpoint is exact")]
  fn arg_view_of_identical_trees_has_the_coordinates_of_the_tree() {
    let t = "((A:1,B:2):1,C:3);";
    let r = run_trees(&[("ha", t), ("na", t)]);
    let div = arg_view(&r, Scale::Div).unwrap();
    let depth = arg_view(&r, Scale::Depth).unwrap();
    assert_eq!(RootCase::Shared, div.root_case);
    // The coordinates of each leaf by label, and those of the internal nodes sorted.
    let leaves = |v: &ArgView, x: fn(&ArgNodeView) -> f64| {
      let mut l: Vec<(String, f64)> = v
        .nodes
        .iter()
        .filter(|n| n.leaf)
        .map(|n| (n.label.clone(), x(n)))
        .collect();
      l.sort_by(|a, b| a.0.cmp(&b.0));
      l
    };
    let internal = |v: &ArgView, x: fn(&ArgNodeView) -> f64| {
      let mut i: Vec<f64> = v.nodes.iter().filter(|n| !n.leaf).map(x).collect();
      i.sort_by(f64::total_cmp);
      i
    };
    let named = |v: &[(&str, f64)]| v.iter().map(|&(n, x)| (n.to_owned(), x)).collect::<Vec<_>>();
    // Oracle: the tree ((A:1,B:2):1,C:3): divergence A 2, B 3, C 3, (A,B) 1, root 0; as a
    // cladogram the root has height 2, so the leaves are at 2 and (A,B) at 1.
    assert_eq!(named(&[("A", 2.0), ("B", 3.0), ("C", 3.0)]), leaves(&div, |n| n.x_div));
    assert_eq!(vec![0.0, 1.0], internal(&div, |n| n.x_div));
    assert_eq!(
      named(&[("A", 2.0), ("B", 2.0), ("C", 2.0)]),
      leaves(&depth, |n| n.x_depth)
    );
    assert_eq!(vec![0.0, 1.0], internal(&depth, |n| n.x_depth));
    // Leaves in rows 0 to 2; (A,B) at the midpoint of A and B, the root at the midpoint of its
    // first and last child.
    let y = |label: &str| div.nodes.iter().find(|n| n.label == label).unwrap().y;
    let mut rows = vec![y("A"), y("B"), y("C")];
    rows.sort_by(f64::total_cmp);
    assert_eq!(vec![0.0, 1.0, 2.0], rows);
    let ab = div
      .nodes
      .iter()
      .find(|n| !n.leaf && n.children.len() == 2 && div.nodes[n.children[0]].leaf && div.nodes[n.children[1]].leaf)
      .unwrap();
    assert_eq!(f64::midpoint(y("A"), y("B")), ab.y);
  }

  #[test]
  fn arg_view_hangs_the_segment_roots_below_the_synthetic_root_at_its_x() {
    let r = run_trees(&[
      ("ha", "(A:1,((C:1,B:1):1,D:1):1);"),
      ("na", "((D:1,(A:1,C:1):1):1,B:1);"),
    ]);
    let v = arg_view(&r, Scale::Div).unwrap();
    assert_eq!(RootCase::Synthetic, v.root_case);
    let top = &v.nodes[v.root];
    assert_eq!(("GlobalRoot", 0.0), (top.label.as_str(), top.x_div));
    // Oracle: the edges of the synthetic root have length 0.
    let below: Vec<f64> = top.children.iter().map(|&c| v.nodes[c].x_div).collect();
    assert_eq!(vec![0.0, 0.0], below);
  }

  #[test]
  fn arg_view_takes_the_segment_0_edge_of_a_hybrid_on_its_chain() {
    let r = run_trees(&[("ha", "((A,B),(C,(D,X)));"), ("na", "((A,(B,X)),(C,D));")]);
    let v = arg_view(&r, Scale::Div).unwrap();
    let hybrid = v.nodes.iter().position(|n| n.hybrid).unwrap();
    let into: Vec<(Vec<usize>, bool)> = v
      .edges
      .iter()
      .filter(|e| e.child == hybrid)
      .map(|e| (e.segments.clone(), e.reticulation))
      .collect();
    // Oracle: the segment 0 parent reaches the shared top root, so its edge is on the chain and
    // the segment 1 edge is the reticulation edge.
    assert_eq!(vec![(vec![0], false), (vec![1], true)], into);
  }

  #[test]
  fn arg_view_top_root_of_one_shared_root_is_the_unshared_root_of_either_segment() {
    let (cases, tops): (Vec<RootCase>, BTreeSet<usize>) = [
      ("((A,B),(C,D));", "((A,C),(B,D));"),
      ("(((A,B),C),D);", "(((C,D),A),B);"),
    ]
    .into_iter()
    .map(|(ha, na)| {
      let r = run_trees(&[("ha", ha), ("na", na)]);
      let v = arg_view(&r, Scale::Div).unwrap();
      let roots = r.built_arg().unwrap().roots;
      (v.root_case, roots.iter().position(|&root| root == v.root).unwrap())
    })
    .unzip();
    // Oracle: the cases cover both branches, a shared root of segment 0 and of segment 1.
    assert_eq!(
      (vec![RootCase::OneShared, RootCase::OneShared], BTreeSet::from([0, 1])),
      (cases, tops)
    );
  }

  #[test]
  fn arg_view_of_more_than_two_trees_is_none() {
    let t = "((A,B),(C,D));";
    let r = run_trees(&[("a", t), ("b", t), ("c", t)]);
    assert!(arg_view(&r, Scale::Div).is_none());
  }
}
