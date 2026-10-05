//! The tanglegram of a pair: both trees sorted for the pair, links, blocks, and MCCs.

use super::shapes::pair_shapes;
use super::slots::{block_neighbors, color_slots};
use super::tree::{draw_tree, row};
use super::{Block, DrawTree, Link, MccInfo, PairView, Scale, TreeVersion};
use crate::run::RunResult;
use std::borrow::Cow;
use std::collections::BTreeMap;
use treeknit_core::mcc_map::leaf_mcc_map;
use treeknit_core::pipeline::{keeps_run_order, sort_for_pair, sort_strictness};
use treeknit_core::{Options, PairResult, Tree};

/// The tanglegram of pair `pair` (pipeline order) of `run` in `version`, with the shapes for
/// `scale`; `None` when the run has no such pair. `opts` are the options of the run.
///
/// The trees of the pair are sorted for it on copies, with the sort of the run (see
/// `treeknit_core::pipeline::sort_for_pair`). Only the `resolved` version of a pair whose order
/// the run kept uses the final trees as they are, because sorting them again can reorder them.
/// The MCCs that each version is sorted by are the pair's MCCs, attached leaves included,
/// restricted to the leaves the two drawn trees share: the leaves of one tree only for `input`
/// and `resolved` are left out, as the run left them out of its sort; the `imputed` trees have
/// the attached leaves in both trees, so they take part.
///
/// MCC colors come from the `resolved` version, so they stay put across versions.
pub fn pair_view(run: &RunResult, opts: &Options, pair: usize, version: TreeVersion, scale: Scale) -> Option<PairView> {
  let p = run.pairs.get(pair)?;
  let resolved = Layout::new(run, opts, p, TreeVersion::Resolved);
  let slots = resolved.slots(p);
  let layout = if version == TreeVersion::Resolved {
    resolved
  } else {
    Layout::new(run, opts, p, version)
  };
  let mccs = mcc_infos(run, p, &slots);
  let shapes = pair_shapes(
    &layout.left,
    &layout.right,
    &layout.links,
    &layout.blocks,
    &slots,
    scale,
  );
  Some(PairView {
    left: layout.left,
    right: layout.right,
    links: layout.links,
    blocks: layout.blocks,
    mccs,
    shapes,
  })
}

/// The color slot of every MCC of pair `p`, computed on its `resolved` version.
pub(super) fn pair_slots(run: &RunResult, opts: &Options, p: &PairResult) -> Vec<usize> {
  Layout::new(run, opts, p, TreeVersion::Resolved).slots(p)
}

/// The two drawn trees of a pair in one version, with their links and blocks.
struct Layout {
  left: DrawTree,
  right: DrawTree,
  links: Vec<Link>,
  blocks: Vec<Block>,
}

impl Layout {
  fn new(run: &RunResult, opts: &Options, p: &PairResult, version: TreeVersion) -> Layout {
    let (left, right) = sorted_pair(run, opts, p, version);
    let leaf_mcc = leaf_mcc_map(&p.mccs, run.taxa.len());
    let left = draw_tree(&left, &run.input_trees[p.i], &leaf_mcc);
    let right = draw_tree(&right, &run.input_trees[p.j], &leaf_mcc);
    let links = links(&left, &right);
    let blocks = blocks(&left, &right, &links);
    Layout {
      left,
      right,
      links,
      blocks,
    }
  }

  fn slots(&self, p: &PairResult) -> Vec<usize> {
    let sizes: Vec<usize> = p.mccs.iter().map(Vec::len).collect();
    color_slots(&sizes, &block_neighbors(sizes.len(), &self.blocks))
  }
}

/// The trees `i` and `j` of pair `p` in `version`, sorted for the pair.
fn sorted_pair<'a>(
  run: &'a RunResult,
  opts: &Options,
  p: &PairResult,
  version: TreeVersion,
) -> (Cow<'a, Tree>, Cow<'a, Tree>) {
  let n = run.taxa.len();
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
  let left_rank = leaf_ranks(left);
  let right_rank = leaf_ranks(right);
  // Each block: its MCC, the ranks of its first and last leaf on both sides, and its direction.
  let mut runs: Vec<(usize, [usize; 2], [usize; 2], Option<bool>)> = Vec::new();
  for link in links {
    let (l, r) = (left_rank[link.left], right_rank[link.right]);
    if let Some((mcc, ls, rs, down)) = runs.last_mut() {
      let step_down = r.checked_sub(1) == Some(rs[1]);
      let step_up = rs[1].checked_sub(1) == Some(r);
      let extends = *mcc == link.mcc
        && l == ls[1] + 1
        && match *down {
          None => step_down || step_up,
          Some(true) => step_down,
          Some(false) => step_up,
        };
      if extends {
        ls[1] = l;
        rs[1] = r;
        *down = Some(step_down);
        continue;
      }
    }
    runs.push((link.mcc, [l, l], [r, r], None));
  }
  runs
    .into_iter()
    .map(|(mcc, ls, rs, _)| Block {
      mcc,
      left: [row(ls[0]), row(ls[1])],
      right: [row(rs[0]), row(rs[1])],
    })
    .collect()
}

/// The rank of each leaf in display order, by node index; 0 for an internal node.
fn leaf_ranks(tree: &DrawTree) -> Vec<usize> {
  let mut rank = 0;
  tree
    .nodes
    .iter()
    .map(|n| {
      if n.leaf {
        rank += 1;
        rank - 1
      } else {
        0
      }
    })
    .collect()
}

/// The MCCs of pair `p`, with their attached members and color slots.
fn mcc_infos(run: &RunResult, p: &PairResult, slots: &[usize]) -> Vec<MccInfo> {
  p.mccs
    .iter()
    .enumerate()
    .map(|(i, m)| {
      let attached = p.attached.iter().filter(|a| a.mcc == i);
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
        ambiguous: attached.clone().any(|a| a.ambiguous),
        slot: slots[i],
      }
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::analysis::{self, ResolveMode, Settings, TreeText};
  use crate::newick;
  use crate::output::{self, OutputOptions};
  use crate::run;
  use pretty_assertions::assert_eq;
  use rand::{Rng, SeedableRng};
  use rand_xoshiro::Xoshiro256PlusPlus;
  use std::collections::BTreeSet;

  /// The two-tree example: X moved between the trees.
  const HA: &str = "((A,B),(C,(D,X)));";
  const NA: &str = "((A,(B,X)),(C,D));";

  fn run_with(trees: &[(&str, &str)], settings: &Settings) -> (RunResult, Options) {
    let texts: Vec<TreeText> = trees
      .iter()
      .map(|(label, newick)| TreeText {
        label: (*label).to_owned(),
        newick: (*newick).to_owned(),
      })
      .collect();
    let opts = analysis::options(settings, texts.len(), false).unwrap();
    let r = run::run(analysis::parse_trees(&texts).unwrap(), &opts, settings.seed, &|_| {});
    (r, opts)
  }

  fn run_trees(trees: &[(&str, &str)]) -> (RunResult, Options) {
    run_with(trees, &Settings::default())
  }

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
    };
    let file = output::output_files(r, &options)
      .into_iter()
      .find(|f| f.path == path)
      .unwrap();
    newick::parse(file.text.trim_end(), "t").unwrap().leaf_names()
  }

  fn view(r: &RunResult, opts: &Options, pair: usize, version: TreeVersion) -> PairView {
    pair_view(r, opts, pair, version, Scale::Div).unwrap()
  }

  #[test]
  fn pair_view_of_the_two_tree_example() {
    let (r, opts) = run_trees(&[("ha", HA), ("na", NA)]);
    let v = view(&r, &opts, 0, TreeVersion::Resolved);
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
    let (r, opts) = run_trees(&[("ha", HA), ("na", NA)]);
    let v = view(&r, &opts, 0, TreeVersion::Resolved);
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
    let (r, opts) = run_trees(&[("ha", HA), ("na", NA)]);
    let slots = |version| -> Vec<usize> { view(&r, &opts, 0, version).mccs.iter().map(|m| m.slot).collect() };
    assert_eq!(slots(TreeVersion::Resolved), slots(TreeVersion::Input));
    assert_eq!(slots(TreeVersion::Resolved), slots(TreeVersion::Imputed));
  }

  #[test]
  fn pair_view_of_an_unknown_pair_is_none() {
    let (r, opts) = run_trees(&[("ha", HA), ("na", NA)]);
    assert!(pair_view(&r, &opts, 1, TreeVersion::Resolved, Scale::Div).is_none());
  }

  #[rstest::rstest]
  #[case::strict_polytomy(ResolveMode::Strict, "((A,B,C),(D,E,F));", "((A,(B,C)),((D,E),F));")]
  #[case::matched_polytomy(ResolveMode::Matched, "((A,B,C),(D,E,F));", "((A,(B,C)),((D,E),F));")]
  #[case::strict_one_tree_only(ResolveMode::Strict, "((A,B),(C,(D,(E,P))));", "((A,(B,E)),(C,D,Q));")]
  #[case::matched_one_tree_only(ResolveMode::Matched, "((A,B),(C,(D,(E,P))));", "((A,(B,E)),(C,D,Q));")]
  fn pair_view_of_two_trees_has_the_leaf_order_of_the_resolved_files(
    #[case] resolve: ResolveMode,
    #[case] ha: &str,
    #[case] na: &str,
  ) {
    let settings = Settings {
      resolve,
      ..Settings::default()
    };
    let (r, opts) = run_with(&[("ha", ha), ("na", na)], &settings);
    let v = view(&r, &opts, 0, TreeVersion::Resolved);
    assert_eq!(output_order(&r, "ha_resolved.nwk"), leaf_names(&v.left));
    assert_eq!(output_order(&r, "na_resolved.nwk"), leaf_names(&v.right));
  }

  #[rstest::rstest]
  #[case::strict(ResolveMode::Strict)]
  #[case::matched(ResolveMode::Matched)]
  #[case::liberal(ResolveMode::Liberal)]
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
    let (r, opts) = run_with(&trees, &settings);
    let n = r.taxa.len();
    let mut kept = 0;
    for (index, p) in r.pairs.iter().enumerate() {
      if !keeps_run_order(&r.trees, &opts, n, p.i, p.j) {
        continue;
      }
      kept += 1;
      let v = view(&r, &opts, index, TreeVersion::Resolved);
      assert_eq!(
        output_order(&r, &format!("{}_resolved.nwk", trees[p.i].0)),
        leaf_names(&v.left)
      );
      assert_eq!(
        output_order(&r, &format!("{}_resolved.nwk", trees[p.j].0)),
        leaf_names(&v.right)
      );
    }
    assert!(kept > 0, "no pair kept the run order");
    // Every tree's last sorting pair: its own order in that pair's view, when the run kept it.
    for t in 0..trees.len() {
      let Some((i, j)) = treeknit_core::pipeline::last_sorting_pair(&r.trees, &opts, n, t) else {
        continue;
      };
      if !keeps_run_order(&r.trees, &opts, n, i, j) {
        continue;
      }
      let index = r.pairs.iter().position(|p| (p.i, p.j) == (i, j)).unwrap();
      let v = view(&r, &opts, index, TreeVersion::Resolved);
      let drawn = if t == i { &v.left } else { &v.right };
      assert_eq!(
        output_order(&r, &format!("{}_resolved.nwk", trees[t].0)),
        leaf_names(drawn)
      );
    }
  }

  #[test]
  fn pair_view_flags_added_nodes_of_a_resolved_polytomy() {
    let (r, opts) = run_trees(&[("ha", "((A,B,C),(D,E));"), ("na", "((A,(B,C)),(D,E));")]);
    let v = view(&r, &opts, 0, TreeVersion::Resolved);
    // Matched resolution copies na's split (B,C) into ha's polytomy.
    let added = names_where(&v.left, |n| n.added);
    assert_eq!(1, added.len());
    let bc = v.left.nodes.iter().find(|n| n.added).unwrap();
    let below: Vec<&str> = bc.children.iter().map(|&c| v.left.nodes[c].name.as_str()).collect();
    assert_eq!(vec!["B", "C"], below);
    assert!(names_where(&v.right, |n| n.added).is_empty());
    let input = view(&r, &opts, 0, TreeVersion::Input);
    assert!(names_where(&input.left, |n| n.added).is_empty());
    assert!(input.left.nodes.iter().all(|n| !n.imputed));
  }

  #[test]
  fn pair_view_of_the_partial_overlap_input_flags_imputed_leaves() {
    // The input of the command-line test `three_trees_partial_overlap_imputed`.
    let (r, opts) = run_trees(&[
      ("seg0", "((A,B),(C,(D,(E,X))));"),
      ("seg1", "((A,(B,X)),(C,D,E,P));"),
      ("seg2", "((A,(B,P)),((C,D),(E,X)));"),
    ]);
    // Pair (0,1): P is only in seg1. In the resolved version it has no link; in the imputed
    // version seg0 holds it as an imputed leaf, and it is an imputed member of its MCC.
    let resolved = view(&r, &opts, 0, TreeVersion::Resolved);
    assert!(!leaf_names(&resolved.left).contains(&"P"));
    assert!(leaf_names(&resolved.right).contains(&"P"));
    assert_eq!(6, resolved.links.len());
    let p_mcc = resolved
      .mccs
      .iter()
      .find(|m| m.leaves.iter().any(|l| l == "P"))
      .unwrap();
    assert_eq!(vec!["P".to_owned()], p_mcc.imputed_leaves);
    let imputed = view(&r, &opts, 0, TreeVersion::Imputed);
    assert_eq!(vec!["P"], names_where(&imputed.left, |n| n.imputed));
    assert!(names_where(&imputed.right, |n| n.imputed).is_empty());
    assert_eq!(7, imputed.links.len());
    let marks: Vec<super::super::MarkKind> = imputed.shapes.left.marks.iter().map(|m| m.kind).collect();
    assert!(marks.contains(&super::super::MarkKind::Imputed));
    // Every pair and version lays out without non-finite values.
    for pair in 0..r.pairs.len() {
      for version in [TreeVersion::Input, TreeVersion::Resolved, TreeVersion::Imputed] {
        for scale in [Scale::Div, Scale::Depth] {
          let v = pair_view(&r, &opts, pair, version, scale).unwrap();
          assert!(numbers(&v).iter().all(|x| x.is_finite()));
        }
      }
    }
  }

  #[test]
  fn pair_view_names_ambiguously_attached_leaves() {
    // P hangs at ha's root, whose children belong to different MCCs: its attachment is
    // ambiguous.
    let (r, opts) = run_trees(&[("ha", "(P,((A,B),(C,D)),X);"), ("na", "((A,(B,X)),(C,D));")]);
    let v = view(&r, &opts, 0, TreeVersion::Resolved);
    let p_mcc = v.mccs.iter().find(|m| m.leaves.iter().any(|l| l == "P")).unwrap();
    assert_eq!(vec!["P".to_owned()], p_mcc.imputed_leaves);
    assert_eq!(vec!["P".to_owned()], p_mcc.ambiguous_leaves);
    assert!(p_mcc.ambiguous);
    assert!(
      v.mccs
        .iter()
        .filter(|m| m.index != p_mcc.index)
        .all(|m| m.ambiguous_leaves.is_empty())
    );
  }

  #[test]
  fn pair_view_node_names_are_unique_and_non_empty_in_every_version() {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(11);
    let mut checked = 0;
    for _ in 0..150 {
      let k = rng.gen_range(2..4);
      let texts: Vec<String> = std::iter::repeat_with(|| random_tree(&mut rng)).take(k).collect();
      let trees: Vec<(String, &str)> = texts
        .iter()
        .enumerate()
        .map(|(i, t)| (format!("t{i}"), t.as_str()))
        .collect();
      let trees: Vec<(&str, &str)> = trees.iter().map(|(l, t)| (l.as_str(), *t)).collect();
      let Some((r, opts)) = try_run(&trees) else { continue };
      for pair in 0..r.pairs.len() {
        for version in [TreeVersion::Input, TreeVersion::Resolved, TreeVersion::Imputed] {
          let v = view(&r, &opts, pair, version);
          for tree in [&v.left, &v.right] {
            let names: BTreeSet<&str> = tree.nodes.iter().map(|n| n.name.as_str()).collect();
            assert_eq!(tree.nodes.len(), names.len(), "{texts:?}");
            assert!(!names.contains(""), "{texts:?}");
          }
          checked += 1;
        }
      }
    }
    assert!(checked > 100);
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

  /// The run of `trees`, or `None` when the trees do not validate.
  fn try_run(trees: &[(&str, &str)]) -> Option<(RunResult, Options)> {
    let texts: Vec<TreeText> = trees
      .iter()
      .map(|(label, newick)| TreeText {
        label: (*label).to_owned(),
        newick: (*newick).to_owned(),
      })
      .collect();
    let settings = Settings::default();
    let parsed = analysis::parse_trees(&texts).ok()?;
    let opts = analysis::options(&settings, texts.len(), false).ok()?;
    Some((run::run(parsed, &opts, settings.seed, &|_| {}), opts))
  }

  #[test]
  fn pair_view_breaks_each_child_branch_below_a_root_without_mcc() {
    // W moves into the clade of A and B: MCCs [W] and [A,B,C,D], and ha's root has no MCC.
    let (r, opts) = run_trees(&[("ha", "(((A,B),(C,D)),W);"), ("na", "(((A,B),W),(C,D));")]);
    let v = view(&r, &opts, 0, TreeVersion::Resolved);
    let root = &v.left.nodes[0];
    assert_eq!(None, root.mcc);
    let child_mccs: BTreeSet<Option<usize>> = root.children.iter().map(|&c| v.left.nodes[c].mcc).collect();
    assert_eq!(2, child_mccs.len());
    assert!(root.children.iter().all(|&c| v.left.nodes[c].mcc_break));
    for tree in [&v.left, &v.right] {
      for node in &tree.nodes {
        let parent_mcc = node.parent.and_then(|p| tree.nodes[p].mcc);
        let expected = node.parent.is_some() && node.mcc.is_some() && parent_mcc != node.mcc;
        assert_eq!(expected, node.mcc_break, "node {}", node.name);
      }
    }
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
