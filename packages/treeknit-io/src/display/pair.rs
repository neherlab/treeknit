//! The tanglegram of a pair: both trees sorted for the pair, links, blocks, and MCCs.

use super::lengths::mean_lengths;
use super::shapes::pair_shapes;
use super::slots::{block_neighbors, color_slots};
use super::tree::draw_tree;
use super::{Block, DrawTree, Link, MccInfo, PairView, Scale, TreeVersion};
use crate::run::RunResult;
use std::borrow::Cow;
use std::collections::BTreeMap;
use treeknit_core::impute::Attachment;
use treeknit_core::mcc_map::leaf_mcc_map;
use treeknit_core::pipeline::{keeps_run_order, sort_for_pair, sort_strictness};
use treeknit_core::{PairResult, Tree};

/// The tanglegram of pair `pair` (pipeline order) of `run` in `version`, with the shapes for
/// `scale`, or for `depth` when `scale` is `div` and a tree of the pair has no branch lengths
/// (`PairView.scale`); `None` when the run has no such pair.
///
/// The trees of the pair are sorted for it on copies, with the sort of the run (see
/// `treeknit_core::pipeline::sort_for_pair`). Only the `resolved` version of a pair whose order
/// the run kept uses the final trees as they are, because sorting them again can reorder them.
/// The MCCs that each version is sorted by are the pair's MCCs, attached leaves included,
/// restricted to the leaves the two drawn trees share: the leaves of one tree only for `input`
/// and `resolved` are left out, as the run left them out of its sort; the `imputed` trees have
/// the attached leaves in both trees, so they take part.
///
/// MCC colors come from the `resolved` version, so they stay put across versions. The slots keep
/// neighboring MCCs apart only in that version: the `input` and `imputed` trees can order the
/// leaves differently, so two MCCs that are neighbors only there can share a color.
pub fn pair_view(run: &RunResult, pair: usize, version: TreeVersion, scale: Scale) -> Option<PairView> {
  let (layout, slots) = pair_layout(run, pair, version)?;
  let mccs = mcc_infos(run, &run.pairs[pair], slots);
  let scale = layout.shown_scale(scale);
  let shapes = pair_shapes(&layout.left, &layout.right, &layout.links, &layout.blocks, slots, scale);
  Some(PairView {
    left: layout.left,
    right: layout.right,
    links: layout.links,
    blocks: layout.blocks,
    mccs,
    scale,
    shapes,
  })
}

/// The drawn trees of pair `pair` (pipeline order) of `run` in `version`, with their links and
/// blocks, and the color slot of every MCC of the pair; `None` when the run has no such pair.
pub(super) fn pair_layout(run: &RunResult, pair: usize, version: TreeVersion) -> Option<(Layout, &[usize])> {
  let p = run.pairs.get(pair)?;
  let layout = Layout::new(run, p, version);
  let slots = if version == TreeVersion::Resolved {
    run.pair_slots[pair].get_or_init(|| layout.slots(p))
  } else {
    pair_slots(run, pair)
  };
  Some((layout, slots))
}

/// The color slot of every MCC of pair `pair`, computed on its `resolved` version once per run
/// and kept in `run`.
pub(super) fn pair_slots(run: &RunResult, pair: usize) -> &[usize] {
  run.pair_slots[pair].get_or_init(|| {
    let p = &run.pairs[pair];
    Layout::new(run, p, TreeVersion::Resolved).slots(p)
  })
}

/// The two drawn trees of a pair in one version, with their links and blocks.
pub(super) struct Layout {
  pub(super) left: DrawTree,
  pub(super) right: DrawTree,
  pub(super) links: Vec<Link>,
  pub(super) blocks: Vec<Block>,
}

impl Layout {
  fn new(run: &RunResult, p: &PairResult, version: TreeVersion) -> Layout {
    let (left, right) = sorted_pair(run, p, version);
    let leaf_mcc = leaf_mcc_map(&p.mccs, run.taxa.len());
    let means = |tree: &Tree, i: usize| match version {
      TreeVersion::Input => vec![None; tree.nodes.len()],
      TreeVersion::Resolved | TreeVersion::Imputed => mean_lengths(run, i, tree),
    };
    let left = draw_tree(&left, &run.input_trees[p.i], &leaf_mcc, &means(&left, p.i));
    let right = draw_tree(&right, &run.input_trees[p.j], &leaf_mcc, &means(&right, p.j));
    let links = links(&left, &right);
    let blocks = blocks(&left, &right, &links);
    Layout {
      left,
      right,
      links,
      blocks,
    }
  }

  /// The scale that a drawing of the layout shows for the requested `scale` (see `Scale::shown`):
  /// `depth` for `div` when a tree has no branch lengths.
  pub(super) fn shown_scale(&self, scale: Scale) -> Scale {
    let flat = |tree: &DrawTree| tree.nodes.iter().all(|n| n.x_div <= 0.0);
    scale.shown(flat(&self.left) || flat(&self.right))
  }

  fn slots(&self, p: &PairResult) -> Vec<usize> {
    let sizes: Vec<usize> = p.mccs.iter().map(Vec::len).collect();
    color_slots(&sizes, &block_neighbors(sizes.len(), &self.blocks))
  }
}

/// The trees `i` and `j` of pair `p` in `version`, sorted for the pair.
fn sorted_pair<'a>(run: &'a RunResult, p: &PairResult, version: TreeVersion) -> (Cow<'a, Tree>, Cow<'a, Tree>) {
  let (n, opts) = (run.taxa.len(), &run.opts);
  let trees = match version {
    TreeVersion::Input => &run.input_trees,
    TreeVersion::Resolved => &run.trees,
    TreeVersion::Imputed => &run.imputed,
  };
  if version == TreeVersion::Resolved && keeps_run_order(&run.trees, opts, n, p.i, p.j) {
    return (Cow::Borrowed(&run.trees[p.i]), Cow::Borrowed(&run.trees[p.j]));
  }
  let (mut left, mut right) = (trees[p.i].clone(), trees[p.j].clone());
  let mccs = p.shared_mccs(trees, n);
  sort_for_pair(&mut left, &mut right, &mccs, n, sort_strictness(opts, run.trees.len()));
  (Cow::Owned(left), Cow::Owned(right))
}

/// One link per leaf that has an MCC and is in both trees, in the left display order. A leaf of
/// one tree only, and a leaf without an MCC of the pair, have no link.
fn links(left: &DrawTree, right: &DrawTree) -> Vec<Link> {
  let right_leaves: BTreeMap<&str, usize> = right
    .nodes
    .iter()
    .enumerate()
    .filter(|(_, n)| n.leaf)
    .map(|(i, n)| (n.name.as_str(), i))
    .collect();
  left
    .nodes
    .iter()
    .enumerate()
    .filter(|(_, n)| n.leaf)
    .filter_map(|(i, n)| {
      Some(Link {
        left: i,
        right: *right_leaves.get(n.name.as_str())?,
        mcc: n.mcc?,
      })
    })
    .collect()
}

/// Runs of links of one MCC whose leaves are consecutive in both trees, in left order: each next
/// leaf follows the previous one in the left tree, and its right rank is the previous one plus
/// or minus 1, in the same direction throughout the block.
fn blocks(left: &DrawTree, right: &DrawTree, links: &[Link]) -> Vec<Block> {
  let (left_rank, right_rank) = (leaf_ranks(left), leaf_ranks(right));
  // Each block: its MCC, its first and last link, and its direction.
  let mut runs: Vec<(usize, usize, usize, Option<bool>)> = Vec::new();
  for (k, link) in links.iter().enumerate() {
    let (l, r) = (left_rank[link.left], right_rank[link.right]);
    if let Some((mcc, _, last, down)) = runs.last_mut() {
      let previous = &links[*last];
      let (pl, pr) = (left_rank[previous.left], right_rank[previous.right]);
      let step_down = r == pr + 1;
      let step_up = r + 1 == pr;
      let extends = *mcc == link.mcc
        && l == pl + 1
        && match *down {
          None => step_down || step_up,
          Some(true) => step_down,
          Some(false) => step_up,
        };
      if extends {
        *last = k;
        *down = Some(step_down);
        continue;
      }
    }
    runs.push((link.mcc, k, k, None));
  }
  runs
    .into_iter()
    .map(|(mcc, first, last, _)| {
      let (first, last) = (&links[first], &links[last]);
      Block {
        mcc,
        left: [left.nodes[first.left].y, left.nodes[last.left].y],
        right: [right.nodes[first.right].y, right.nodes[last.right].y],
      }
    })
    .collect()
}

/// The rank of each leaf of `tree` in display order, by node index; 0 for an internal node. The
/// nodes are in preorder with children in display order, so the leaves come in display order.
fn leaf_ranks(tree: &DrawTree) -> Vec<usize> {
  let mut next = 0;
  tree
    .nodes
    .iter()
    .map(|n| {
      if !n.leaf {
        return 0;
      }
      next += 1;
      next - 1
    })
    .collect()
}

/// The MCCs of pair `p`, with their attached members and color slots.
pub(super) fn mcc_infos(run: &RunResult, p: &PairResult, slots: &[usize]) -> Vec<MccInfo> {
  let mut by_mcc: Vec<Vec<&Attachment>> = vec![Vec::new(); p.mccs.len()];
  for a in &p.attached {
    by_mcc[a.mcc].push(a);
  }
  p.mccs
    .iter()
    .zip(by_mcc)
    .enumerate()
    .map(|(i, (m, attached))| {
      let attached = attached.into_iter();
      MccInfo {
        index: i,
        size: m.len(),
        leaves: run.taxa.names_of(m),
        imputed_leaves: attached.clone().flat_map(|a| run.taxa.names_of(&a.leaves)).collect(),
        ambiguous_leaves: attached
          .clone()
          .filter(|a| a.ambiguous)
          .flat_map(|a| run.taxa.names_of(&a.leaves))
          .collect(),
        slot: slots[i],
      }
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::analysis::{ResolveMode, Settings};
  use crate::newick;
  use crate::output::{self, OutputOptions};
  use crate::test_support::{run_trees, run_with, try_run};

  use pretty_assertions::assert_eq;
  use rand::{Rng, SeedableRng};
  use rand_xoshiro::Xoshiro256PlusPlus;
  use rstest::rstest;
  use std::collections::BTreeSet;

  /// The two-tree example: X moved between the trees.
  const HA: &str = "((A,B),(C,(D,X)));";
  const NA: &str = "((A,(B,X)),(C,D));";

  fn leaf_names(t: &DrawTree) -> Vec<&str> {
    t.nodes.iter().filter(|n| n.leaf).map(|n| n.name.as_str()).collect()
  }

  fn names_where(t: &DrawTree, f: impl Fn(&super::super::DrawNode) -> bool) -> Vec<&str> {
    t.nodes.iter().filter(|n| f(n)).map(|n| n.name.as_str()).collect()
  }

  /// Leaf order of the output file `path` of `run`.
  fn output_order(r: &RunResult, path: &str) -> Vec<String> {
    let options = OutputOptions {
      extensions: vec![".nwk".to_owned(); r.trees.len()],
      imputed: true,
      auspice: false,
      figures: false,
    };
    let file = output::output_files(r, &options)
      .unwrap()
      .into_iter()
      .find(|f| f.path == path)
      .unwrap();
    newick::parse(file.text.trim_end(), "t").unwrap().leaf_names()
  }

  fn view(r: &RunResult, pair: usize, version: TreeVersion) -> PairView {
    pair_view(r, pair, version, Scale::Div).unwrap()
  }

  /// A tree of one root above `leaves`, each a name and an MCC, in display order.
  fn flat(leaves: &[(&str, Option<usize>)]) -> DrawTree {
    let node = |name: &str, parent, y, leaf, mcc| super::super::DrawNode {
      name: name.to_owned(),
      short_name: name.to_owned(),
      parent,
      children: if leaf { vec![] } else { (1..=leaves.len()).collect() },
      branch_length: None,
      mean_length: None,
      x_div: 0.0,
      x_depth: 0.0,
      y,
      leaf,
      clade_size: if leaf { 1 } else { leaves.len() },
      added: false,
      imputed: false,
      mcc,
      mcc_break: false,
    };
    let mut nodes = vec![node("r", None, 0.0, false, None)];
    nodes.extend(
      leaves
        .iter()
        .zip(0_u32..)
        .map(|(&(name, mcc), y)| node(name, Some(0), f64::from(y), true, mcc)),
    );
    DrawTree {
      label: "t".to_owned(),
      nodes,
    }
  }

  fn block_of(mcc: usize, left: [f64; 2], right: [f64; 2]) -> Block {
    Block { mcc, left, right }
  }

  #[rustfmt::skip]
  #[rstest]
  // Oracle: a block holds leaves of one MCC that are consecutive in both trees, in left order,
  // with right rows that step by +1 throughout or by -1 throughout.
  #[case::ascending( &[("A", 0), ("B", 0), ("C", 0)], &["A", "B", "C"], vec![block_of(0, [0.0, 2.0], [0.0, 2.0])])]
  #[case::descending(&[("A", 0), ("B", 0), ("C", 0)], &["C", "B", "A"], vec![block_of(0, [0.0, 2.0], [2.0, 0.0])])]
  #[case::jump(      &[("A", 0), ("B", 0), ("C", 0)], &["A", "C", "B"], vec![block_of(0, [0.0, 0.0], [0.0, 0.0]), block_of(0, [1.0, 2.0], [2.0, 1.0])])]
  #[case::mcc_change(&[("A", 0), ("B", 1)],           &["A", "B"],      vec![block_of(0, [0.0, 0.0], [0.0, 0.0]), block_of(1, [1.0, 1.0], [1.0, 1.0])])]
  // X is in the left tree only, so it has no link and A and B are not consecutive there.
  #[case::skipped(&[("A", 0), ("X", 0), ("B", 0)], &["A", "B"],      vec![block_of(0, [0.0, 0.0], [0.0, 0.0]), block_of(0, [2.0, 2.0], [1.0, 1.0])])]
  #[trace]
  fn blocks_join_links_consecutive_in_both_trees(
    #[case] left: &[(&str, usize)],
    #[case] right: &[&str],
    #[case] expected: Vec<Block>,
  ) {
    let mcc_of = |name: &str| left.iter().find(|(n, _)| *n == name).map(|&(_, m)| m);
    let left = flat(&left.iter().map(|&(n, m)| (n, Some(m))).collect::<Vec<_>>());
    let right = flat(&right.iter().map(|&n| (n, mcc_of(n))).collect::<Vec<_>>());
    assert_eq!(expected, blocks(&left, &right, &links(&left, &right)));
  }

  #[test]
  fn pair_view_lays_out_each_version_and_scale_exactly() {
    // P is in ha only; the imputed version of na places it as the sister of D.
    let r = run_trees(&[
      ("ha", "((A:1,B:1):1,(C:1,(D:1,P:1):1):1);"),
      ("na", "((A:1,B:1):2,(C:1,D:3):1);"),
    ]);
    // The x of each named leaf and the sorted x of the internal nodes of the right tree.
    let xs = |version, scale| {
      let v = pair_view(&r, 0, version, scale).unwrap();
      let x = |n: &super::super::DrawNode| if scale == Scale::Div { n.x_div } else { n.x_depth };
      let mut leaves: Vec<(String, f64)> = v
        .right
        .nodes
        .iter()
        .filter(|n| n.leaf)
        .map(|n| (n.name.clone(), x(n)))
        .collect();
      leaves.sort_by(|a, b| a.0.cmp(&b.0));
      let mut internal: Vec<f64> = v.right.nodes.iter().filter(|n| !n.leaf).map(x).collect();
      internal.sort_by(f64::total_cmp);
      let rows: BTreeSet<u64> = v.right.nodes.iter().filter(|n| n.leaf).map(|n| n.y.to_bits()).collect();
      (leaves, internal, rows.len())
    };
    let named = |v: &[(&str, f64)]| v.iter().map(|&(n, x)| (n.to_owned(), x)).collect::<Vec<_>>();
    // Oracle: na is ((A:1,B:1):2,(C:1,D:3):1): divergence A, B 3, C 2, D 4; internal nodes at
    // 0, 1, and 2. As a cladogram the root has height 2, so the leaves are at 2 and the two
    // internal nodes at 1.
    let input_div = (
      named(&[("A", 3.0), ("B", 3.0), ("C", 2.0), ("D", 4.0)]),
      vec![0.0, 1.0, 2.0],
      4,
    );
    let input_depth = (
      named(&[("A", 2.0), ("B", 2.0), ("C", 2.0), ("D", 2.0)]),
      vec![0.0, 1.0, 1.0],
      4,
    );
    assert_eq!(input_div, xs(TreeVersion::Input, Scale::Div));
    assert_eq!(input_depth, xs(TreeVersion::Input, Scale::Depth));
    assert_eq!(input_div, xs(TreeVersion::Resolved, Scale::Div));
    assert_eq!(input_depth, xs(TreeVersion::Resolved, Scale::Depth));
    // Imputed: na becomes ((A,B),(C,(D,P))), so the cladogram root has height 3 and the leaves
    // are at 3; (C,(D,P)) has height 2 (at 1), and (A,B) and (D,P) have height 1 (at 2).
    let (leaves, internal, rows) = xs(TreeVersion::Imputed, Scale::Depth);
    let expected = named(&[("A", 3.0), ("B", 3.0), ("C", 3.0), ("D", 3.0), ("P", 3.0)]);
    assert_eq!((expected, vec![0.0, 1.0, 2.0, 2.0], 5), (leaves, internal, rows));
  }

  #[test]
  fn pair_view_of_the_two_tree_example() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    let v = view(&r, 0, TreeVersion::Resolved);
    // Oracle: the run's output order of each tree (sorted by the run for this pair).
    assert_eq!(output_order(&r, "ha_resolved.nwk"), leaf_names(&v.left));
    assert_eq!(output_order(&r, "na_resolved.nwk"), leaf_names(&v.right));
    assert_eq!(vec!["A", "B", "C", "D", "X"], leaf_names(&v.left));
    assert_eq!(vec!["A", "B", "X", "C", "D"], leaf_names(&v.right));
    let ys: Vec<f64> = v.left.nodes.iter().filter(|n| n.leaf).map(|n| n.y).collect();
    assert_eq!(vec![0.0, 1.0, 2.0, 3.0, 4.0], ys);
    // The branch above X is the only reassortment branch in both trees.
    assert_eq!(vec!["X"], names_where(&v.left, |n| n.mcc_break));
    assert_eq!(vec!["X"], names_where(&v.right, |n| n.mcc_break));
    // MCCs in the order of MCCs.json: [X], [A,B,C,D].
    let leaves: Vec<Vec<String>> = v.mccs.iter().map(|m| m.leaves.clone()).collect();
    assert_eq!(
      vec![
        vec!["X".to_owned()],
        vec!["A", "B", "C", "D"].into_iter().map(String::from).collect()
      ],
      leaves
    );
    // Blocks: A,B (rows 0-1 on both sides), C,D (left 2-3, right 3-4), X (left 4, right 2).
    let expected = vec![
      Block {
        mcc: 1,
        left: [0.0, 1.0],
        right: [0.0, 1.0],
      },
      Block {
        mcc: 1,
        left: [2.0, 3.0],
        right: [3.0, 4.0],
      },
      Block {
        mcc: 0,
        left: [4.0, 4.0],
        right: [2.0, 2.0],
      },
    ];
    assert_eq!(expected, v.blocks);
    // The larger MCC takes slot 0, its neighbor X slot 1.
    let slots: Vec<usize> = v.mccs.iter().map(|m| m.slot).collect();
    assert_eq!(vec![1, 0], slots);
    assert_eq!(5, v.links.len());
  }

  #[test]
  fn pair_view_shapes_of_the_two_tree_example() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    let v = view(&r, 0, TreeVersion::Resolved);
    let x_link = &v.shapes.links[4];
    assert_eq!(1, x_link.slot);
    let expected = super::super::Bezier {
      from: [0.0, 4.0],
      c1: [0.5, 4.0],
      c2: [0.5, 2.0],
      to: [1.0, 2.0],
    };
    assert_eq!(expected, x_link.curve);
    // The ribbon of X: left row 4 and right row 2, each extended by half a row.
    let top = super::super::Bezier {
      from: [0.0, 3.5],
      c1: [0.5, 3.5],
      c2: [0.5, 1.5],
      to: [1.0, 1.5],
    };
    assert_eq!(top, v.shapes.ribbons[2].outline[0]);
    let marks: Vec<usize> = v.shapes.left.marks.iter().map(|m| m.node).collect();
    let x = v.left.nodes.iter().position(|n| n.name == "X").unwrap();
    assert_eq!(vec![x], marks);
  }

  #[test]
  fn pair_view_keeps_slots_across_versions() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    let slots = |version| -> Vec<usize> { view(&r, 0, version).mccs.iter().map(|m| m.slot).collect() };
    assert_eq!(slots(TreeVersion::Resolved), slots(TreeVersion::Input));
    assert_eq!(slots(TreeVersion::Resolved), slots(TreeVersion::Imputed));
  }

  #[test]
  fn pair_view_of_an_unknown_pair_is_none() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    assert!(pair_view(&r, 1, TreeVersion::Resolved, Scale::Div).is_none());
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::strict_polytomy(      ResolveMode::Strict, "((A,B,C),(D,E,F));", "((A,(B,C)),((D,E),F));")]
  #[case::matched_polytomy(     ResolveMode::Matched, "((A,B,C),(D,E,F));", "((A,(B,C)),((D,E),F));")]
  #[case::strict_one_tree_only( ResolveMode::Strict, "((A,B),(C,(D,(E,P))));", "((A,(B,E)),(C,D,Q));")]
  #[case::matched_one_tree_only(ResolveMode::Matched, "((A,B),(C,(D,(E,P))));", "((A,(B,E)),(C,D,Q));")]
  #[trace]
  fn pair_view_of_two_trees_has_the_leaf_order_of_the_resolved_files(
    #[case] resolve: ResolveMode,
    #[case] ha: &str,
    #[case] na: &str,
  ) {
    let settings = Settings {
      resolve,
      ..Settings::default()
    };
    let r = run_with(&[("ha", ha), ("na", na)], &settings);
    let v = view(&r, 0, TreeVersion::Resolved);
    assert_eq!(output_order(&r, "ha_resolved.nwk"), leaf_names(&v.left));
    assert_eq!(output_order(&r, "na_resolved.nwk"), leaf_names(&v.right));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::strict( ResolveMode::Strict)]
  #[case::matched(ResolveMode::Matched)]
  #[case::liberal(ResolveMode::Liberal)]
  #[trace]
  fn pair_view_of_a_pair_the_run_kept_has_the_output_order_of_three_trees(#[case] resolve: ResolveMode) {
    let settings = Settings {
      resolve,
      ..Settings::default()
    };
    let trees = [
      ("seg0", "((A,B),(C,(D,(E,X))));"),
      ("seg1", "((A,(B,X)),(C,D,E));"),
      ("seg2", "((A,B),((C,D),(E,X)));"),
    ];
    let r = run_with(&trees, &settings);
    let n = r.taxa.len();
    let output = |t: usize| output_order(&r, &format!("{}_resolved.nwk", trees[t].0));
    let drawn_order = |tree: &DrawTree| -> Vec<String> { leaf_names(tree).into_iter().map(str::to_owned).collect() };
    // The pairs whose order the run kept: both trees in the order of their output files.
    let (expected, drawn): (Vec<_>, Vec<_>) = r
      .pairs
      .iter()
      .enumerate()
      .filter(|(_, p)| keeps_run_order(&r.trees, &r.opts, n, p.i, p.j))
      .map(|(index, p)| {
        let v = view(&r, index, TreeVersion::Resolved);
        ((output(p.i), output(p.j)), (drawn_order(&v.left), drawn_order(&v.right)))
      })
      .unzip();
    assert!(!expected.is_empty(), "no pair kept the run order");
    assert_eq!(expected, drawn);
    // Every tree's last sorting pair: its own order in that pair's view, when the run kept it.
    let (expected, drawn): (Vec<_>, Vec<_>) = (0..trees.len())
      .filter_map(|t| {
        let (i, j) = treeknit_core::pipeline::last_sorting_pair(&r.trees, &r.opts, n, t)?;
        keeps_run_order(&r.trees, &r.opts, n, i, j).then(|| {
          let index = r.pairs.iter().position(|p| (p.i, p.j) == (i, j)).unwrap();
          let v = view(&r, index, TreeVersion::Resolved);
          (output(t), drawn_order(if t == i { &v.left } else { &v.right }))
        })
      })
      .unzip();
    assert_eq!(expected, drawn);
  }

  #[test]
  fn pair_view_flags_added_nodes_of_a_resolved_polytomy() {
    let r = run_trees(&[("ha", "((A,B,C),(D,E));"), ("na", "((A,(B,C)),(D,E));")]);
    let v = view(&r, 0, TreeVersion::Resolved);
    // Matched resolution copies na's split (B,C) into ha's polytomy.
    let added = names_where(&v.left, |n| n.added);
    assert_eq!(1, added.len());
    let bc = v.left.nodes.iter().find(|n| n.added).unwrap();
    let below: Vec<&str> = bc.children.iter().map(|&c| v.left.nodes[c].name.as_str()).collect();
    assert_eq!(vec!["B", "C"], below);
    assert!(names_where(&v.right, |n| n.added).is_empty());
    let input = view(&r, 0, TreeVersion::Input);
    assert!(names_where(&input.left, |n| n.added).is_empty());
    assert!(input.left.nodes.iter().all(|n| !n.imputed));
  }

  #[test]
  fn pair_view_draws_a_resolved_split_at_the_mean_length_and_moves_its_leaves() {
    let r = run_trees(&[
      ("ha", "((A:1,B:1,C:1):1,(D:1,E:1):1);"),
      ("na", "((A:1,(B:1,C:1):0.5):1,(D:1,E:1):1);"),
    ]);
    let resolved = view(&r, 0, TreeVersion::Resolved);
    let input = view(&r, 0, TreeVersion::Input);
    let bc = |t: &DrawTree| {
      let below = |n: &super::super::DrawNode| -> BTreeSet<&str> {
        n.children.iter().map(|&c| t.nodes[c].name.as_str()).collect()
      };
      let n = t.nodes.iter().find(|n| below(n) == BTreeSet::from(["B", "C"])).unwrap();
      (n.mean_length, n.x_div)
    };
    // Oracle: (0 + 0.5) / 2 = 0.25 right of the node (A,B,C) at 1, in both trees.
    assert_eq!((Some(0.25), 1.25), bc(&resolved.left));
    assert_eq!((Some(0.25), 1.25), bc(&resolved.right));
    let leaves = |t: &DrawTree| -> BTreeMap<String, f64> {
      t.nodes
        .iter()
        .filter(|n| n.leaf)
        .map(|n| (n.name.clone(), n.x_div))
        .collect()
    };
    // Oracle: B and C lie 1 right of (B,C) in both trees, so both trees draw them at 2.25.
    let expected: BTreeMap<String, f64> = [("A", 2.0), ("B", 2.25), ("C", 2.25), ("D", 2.0), ("E", 2.0)]
      .into_iter()
      .map(|(name, x)| (name.to_owned(), x))
      .collect();
    assert_eq!(
      (&expected, &expected),
      (&leaves(&resolved.left), &leaves(&resolved.right))
    );
    assert!(
      input
        .left
        .nodes
        .iter()
        .chain(&input.right.nodes)
        .all(|n| n.mean_length.is_none())
    );
  }

  /// The run of the input of the command-line test `three_trees_partial_overlap_imputed`.
  fn partial_overlap_run() -> RunResult {
    run_trees(&[
      ("seg0", "((A,B),(C,(D,(E,X))));"),
      ("seg1", "((A,(B,X)),(C,D,E,P));"),
      ("seg2", "((A,(B,P)),((C,D),(E,X)));"),
    ])
  }

  #[test]
  fn pair_view_of_the_partial_overlap_input_flags_imputed_leaves() {
    let r = partial_overlap_run();
    // Pair (0,1): P is only in seg1. In the resolved version it has no link; in the imputed
    // version seg0 holds it as an imputed leaf, and it is an imputed member of its MCC.
    let resolved = view(&r, 0, TreeVersion::Resolved);
    assert!(!leaf_names(&resolved.left).contains(&"P"));
    assert!(leaf_names(&resolved.right).contains(&"P"));
    assert_eq!(6, resolved.links.len());
    let p_mcc = resolved
      .mccs
      .iter()
      .find(|m| m.leaves.iter().any(|l| l == "P"))
      .unwrap();
    assert_eq!(vec!["P".to_owned()], p_mcc.imputed_leaves);
    let imputed = view(&r, 0, TreeVersion::Imputed);
    assert_eq!(vec!["P"], names_where(&imputed.left, |n| n.imputed));
    assert!(names_where(&imputed.right, |n| n.imputed).is_empty());
    assert_eq!(7, imputed.links.len());
    let marks: Vec<super::super::MarkKind> = imputed.shapes.left.marks.iter().map(|m| m.kind).collect();
    assert!(marks.contains(&super::super::MarkKind::Imputed));
  }

  #[rstest]
  #[trace]
  fn pair_views_of_the_partial_overlap_input_have_finite_numbers(
    #[values(0, 1, 2)] pair: usize,
    #[values(TreeVersion::Input, TreeVersion::Resolved, TreeVersion::Imputed)] version: TreeVersion,
    #[values(Scale::Div, Scale::Depth)] scale: Scale,
  ) {
    let v = pair_view(&partial_overlap_run(), pair, version, scale).unwrap();
    assert!(numbers(&v).iter().all(|x| x.is_finite()));
  }

  #[test]
  fn pair_view_names_ambiguously_attached_leaves() {
    // P hangs at ha's root, whose children belong to different MCCs: its attachment is
    // ambiguous.
    let r = run_trees(&[("ha", "(P,((A,B),(C,D)),X);"), ("na", "((A,(B,X)),(C,D));")]);
    let v = view(&r, 0, TreeVersion::Resolved);
    let p_mcc = v.mccs.iter().find(|m| m.leaves.iter().any(|l| l == "P")).unwrap();
    assert_eq!(vec!["P".to_owned()], p_mcc.imputed_leaves);
    assert_eq!(vec!["P".to_owned()], p_mcc.ambiguous_leaves);
    assert!(
      v.mccs
        .iter()
        .filter(|m| m.index != p_mcc.index)
        .all(|m| m.ambiguous_leaves.is_empty())
    );
  }

  #[test]
  fn pair_view_shapes_carry_the_mcc_of_their_link_block_and_node() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    let v = view(&r, 0, TreeVersion::Resolved);
    let (expected, actual): (Vec<_>, Vec<_>) = v
      .shapes
      .links
      .iter()
      .enumerate()
      .map(|(i, c)| ((i, v.links[i].mcc, v.mccs[c.mcc].slot), (c.link, c.mcc, c.slot)))
      .unzip();
    assert_eq!(expected, actual);
    let (expected, actual): (Vec<_>, Vec<_>) = v
      .shapes
      .ribbons
      .iter()
      .enumerate()
      .map(|(i, ribbon)| {
        (
          (i, v.blocks[i].mcc, v.mccs[ribbon.mcc].slot),
          (ribbon.block, ribbon.mcc, ribbon.slot),
        )
      })
      .unzip();
    assert_eq!(expected, actual);
    let trees = [(&v.left, &v.shapes.left), (&v.right, &v.shapes.right)];
    let (expected, actual): (Vec<_>, Vec<_>) = trees
      .iter()
      .flat_map(|(tree, shapes)| shapes.elbows.iter().map(|e| (tree.nodes[e.node].mcc, e.mcc)))
      .unzip();
    assert_eq!(expected, actual);
    let (expected, actual): (Vec<_>, Vec<_>) = trees
      .iter()
      .flat_map(|(tree, shapes)| {
        shapes.marks.iter().map(|m| {
          let mcc = tree.nodes[m.node].mcc;
          ((mcc, mcc.map(|x| v.mccs[x].slot)), (m.mcc, m.slot))
        })
      })
      .unzip();
    assert_eq!(expected, actual);
    assert!(!v.shapes.left.marks.is_empty() || !v.shapes.right.marks.is_empty());
  }

  #[test]
  fn pair_view_renames_an_internal_node_named_like_an_imputed_leaf() {
    // Imputation grafts na's leaf P into ha, whose internal node above A and B is named P.
    let r = run_trees(&[("ha", "((A,B)P,(C,D));"), ("na", "((A,B),(C,(D,P)));")]);
    let v = view(&r, 0, TreeVersion::Imputed);
    assert_eq!(vec!["P"], names_where(&v.left, |n| n.imputed));
    let internal: Vec<&str> = names_where(&v.left, |n| !n.leaf && n.name.starts_with('P'));
    assert_eq!(vec!["P_2"], internal);
    let names: BTreeSet<&str> = v.left.nodes.iter().map(|n| n.name.as_str()).collect();
    assert_eq!(v.left.nodes.len(), names.len());
  }

  #[test]
  fn pair_view_node_names_are_unique_and_non_empty_in_every_version() {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(11);
    let mut checked = 0;
    let mut repeated_or_empty = Vec::new();
    for _ in 0..150 {
      let k = rng.gen_range(2..4);
      let texts: Vec<String> = std::iter::repeat_with(|| random_tree(&mut rng)).take(k).collect();
      let trees: Vec<(String, &str)> = texts
        .iter()
        .enumerate()
        .map(|(i, t)| (format!("t{i}"), t.as_str()))
        .collect();
      let trees: Vec<(&str, &str)> = trees.iter().map(|(l, t)| (l.as_str(), *t)).collect();
      let Some(r) = try_run(&trees) else { continue };
      for pair in 0..r.pairs.len() {
        for version in [TreeVersion::Input, TreeVersion::Resolved, TreeVersion::Imputed] {
          let v = view(&r, pair, version);
          let unique_and_named = [&v.left, &v.right].iter().all(|tree| {
            let names: BTreeSet<&str> = tree.nodes.iter().map(|n| n.name.as_str()).collect();
            tree.nodes.len() == names.len() && !names.contains("")
          });
          if !unique_and_named {
            repeated_or_empty.push((texts.clone(), pair, version));
          }
          checked += 1;
        }
      }
    }
    assert_eq!((Vec::new(), true), (repeated_or_empty, checked > 100));
  }

  /// A random tree on a random subset of the leaves `A` to `H`, with unnamed internal nodes of
  /// two or three children.
  fn random_tree(rng: &mut Xoshiro256PlusPlus) -> String {
    let mut parts: Vec<String> = (0..8_u8)
      .filter(|_| rng.gen_bool(0.85))
      .map(|i| char::from(b'A' + i).to_string())
      .collect();
    while parts.len() > 1 {
      let take = if parts.len() > 2 && rng.gen_bool(0.3) { 3 } else { 2 };
      let group: Vec<String> = std::iter::repeat_with(|| parts.remove(rng.gen_range(0..parts.len())))
        .take(take)
        .collect();
      parts.push(format!("({})", group.join(",")));
    }
    format!("{};", parts.first().cloned().unwrap_or_default())
  }

  #[test]
  fn pair_view_breaks_each_child_branch_below_a_root_without_mcc() {
    // W moves into the clade of A and B: MCCs [W] and [A,B,C,D], and ha's root has no MCC.
    let r = run_trees(&[("ha", "(((A,B),(C,D)),W);"), ("na", "(((A,B),W),(C,D));")]);
    let v = view(&r, 0, TreeVersion::Resolved);
    let root = &v.left.nodes[0];
    assert_eq!(None, root.mcc);
    let child_mccs: BTreeSet<Option<usize>> = root.children.iter().map(|&c| v.left.nodes[c].mcc).collect();
    assert_eq!(2, child_mccs.len());
    assert!(root.children.iter().all(|&c| v.left.nodes[c].mcc_break));
    let (expected, actual): (Vec<_>, Vec<_>) = [&v.left, &v.right]
      .iter()
      .flat_map(|tree| {
        tree.nodes.iter().map(|node| {
          let parent_mcc = node.parent.and_then(|p| tree.nodes[p].mcc);
          let breaks = node.parent.is_some() && node.mcc.is_some() && parent_mcc != node.mcc;
          ((node.name.as_str(), breaks), (node.name.as_str(), node.mcc_break))
        })
      })
      .unzip();
    assert_eq!(expected, actual);
  }

  /// Every coordinate of `v`.
  fn numbers(v: &PairView) -> Vec<f64> {
    let mut out = Vec::new();
    for t in [&v.left, &v.right] {
      out.extend(t.nodes.iter().flat_map(|n| [n.x_div, n.x_depth, n.y]));
    }
    out.extend(v.blocks.iter().flat_map(|b| b.left.into_iter().chain(b.right)));
    for t in [&v.shapes.left, &v.shapes.right] {
      out.extend(t.elbows.iter().flat_map(|e| e.points.into_iter().flatten()));
      out.extend(t.marks.iter().flat_map(|m| m.at));
    }
    let curves = v.shapes.links.iter().map(|l| &l.curve);
    for c in curves.chain(v.shapes.ribbons.iter().flat_map(|r| &r.outline)) {
      out.extend([c.from, c.c1, c.c2, c.to].into_iter().flatten());
    }
    out
  }
}
