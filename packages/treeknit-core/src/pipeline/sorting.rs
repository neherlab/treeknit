//! The polytomy sorting of the pairs of a run, and its replay for the display of a pair.

#![expect(
  clippy::expect_used,
  reason = "findings from before the strict lint set; kb/issues/N-lint-baseline.md tracks their removal"
)]

use crate::mcc_map::{leaf_order, sort_by_leaf_order, sort_polytomies_by_mccs};
use crate::naive::Mcc;
use crate::options::{Options, Resolution};
use crate::pipeline::{pipeline_pairs, restricted_to_shared, round_mode, schedule, shared_leaves};
use crate::resolve::{resolve_with_mccs_quiet, warn_skipped};
use crate::tree::Tree;
use std::borrow::Cow;

/// Whether the run on `k` trees sorts the polytomies of its output trees through strictly
/// resolved copies: in its last round as resolution in that round (or `opts.sort_strict`), and
/// never after `Matched` resolution, whose trees already agree within MCCs.
pub fn sort_strictness(opts: &Options, k: usize) -> bool {
  if opts.resolution == Resolution::Matched {
    return false;
  }
  let (_, extra_round) = schedule(opts, k);
  let (_, strict) = round_mode(opts, extra_round);
  opts.sort_strict.unwrap_or(strict)
}

/// The last pair whose sort in the run reordered tree `tree`, or `None` if no pair sorted it.
/// `trees` are the trees of the run (their leaf sets decide which pairs are skipped).
///
/// The run sorts every pair that shares at least two leaves, in pipeline order, once: in the
/// last round, or after `Matched` resolution. A pair `(i, j)` reorders tree `j`, and tree `i`
/// when it ladderizes it (`i == 0`) or sorts strictly (see [`sort_strictness`]).
///
/// # Panics
///
/// If `tree` is not an index of `trees`.
pub fn last_sorting_pair(trees: &[Tree], opts: &Options, n: usize, tree: usize) -> Option<(usize, usize)> {
  let k = trees.len();
  assert!(tree < k, "tree {tree} is not one of the {k} trees");
  let (rounds, _) = schedule(opts, k);
  if rounds == 0 && opts.resolution != Resolution::Matched {
    return None;
  }
  let strict = sort_strictness(opts, k);
  pipeline_pairs(k).into_iter().rfind(|&(i, j)| {
    (j == tree || (i == tree && (i == 0 || strict))) && shared_leaves(&trees[i], &trees[j], n).is_some()
  })
}

/// Whether the run left trees `i` and `j` in the order of its sort of the pair `(i, j)`: the
/// run sorted the pair, and no later sort changed either tree. The view of that pair then uses
/// the run's trees as they are; other pairs sort copies with [`sort_for_pair`].
///
/// # Panics
///
/// Unless `i < j < trees.len()`.
pub fn keeps_run_order(trees: &[Tree], opts: &Options, n: usize, i: usize, j: usize) -> bool {
  assert!(
    i < j && j < trees.len(),
    "({i}, {j}) is not a pair i < j of the {} trees",
    trees.len()
  );
  shared_leaves(&trees[i], &trees[j], n).is_some()
    && [i, j]
      .iter()
      .all(|&t| last_sorting_pair(trees, opts, n, t).is_some_and(|last| last <= (i, j)))
}

/// Sort `left` and `right` for display as a pair: ladderize `left`, then order the polytomies
/// of both so that MCCs face each other with the run's sort of a pair. The run ladderizes only
/// tree 0; this display sort always ladderizes `left`, so that every pair view has the same
/// layout rule, and its order can differ from the run's for a pair `(i, j)` with `i > 0`.
/// `mccs` are the MCCs over the leaves the two trees share (see [`PairResult::shared_mccs`](crate::pipeline::PairResult::shared_mccs)),
/// and `strict` is the strictness of the run's sort (see [`sort_strictness`]). Logs nothing. A
/// pair that shares fewer than two leaves is left unchanged.
///
/// # Panics
///
/// If a leaf of `mccs`, `left`, or `right` is not one of the `n` taxa.
pub fn sort_for_pair(left: &mut Tree, right: &mut Tree, mccs: &[Mcc], n: usize, strict: bool) {
  assert!(
    mccs.iter().flatten().all(|&x| x < n),
    "an MCC holds a leaf that is not one of the {n} taxa"
  );
  assert!(
    left.has_taxa_below(n) && right.has_taxa_below(n),
    "a tree of the pair has a leaf that is not one of the {n} taxa"
  );
  sort_two(left, right, true, mccs, n, strict);
}

/// Sort trees `i < j` as the run's last round does: ladderize the first tree of the run and
/// order polytomies so that MCCs face each other, logging the splits that resolving the copies
/// of a strict sort skips. A pair that shares fewer than two leaves is left unchanged.
pub(crate) fn sort_pair(trees: &mut [Tree], i: usize, j: usize, mccs: &[Mcc], n: usize, strict: bool) {
  let [ti, tj] = trees.get_disjoint_mut([i, j]).expect("the trees of a pair differ");
  let skipped = sort_two(ti, tj, i == 0, mccs, n, strict);
  warn_skipped(ti, skipped[0]);
  warn_skipped(tj, skipped[1]);
}

/// Ladderize `left` if `ladderize_left`, then order polytomies so that MCCs face each other:
/// strictly through liberally resolved copies of both trees, whose order is applied to both,
/// or by sorting the polytomies of `right` along `left`. A pair that shares fewer than two
/// leaves is left unchanged. Returns the number of splits that resolving the copies skipped as
/// incompatible, per tree.
fn sort_two(
  left: &mut Tree,
  right: &mut Tree,
  ladderize_left: bool,
  mccs: &[Mcc],
  n: usize,
  strict: bool,
) -> [usize; 2] {
  let Some(shared) = shared_leaves(left, right, n) else {
    return [0, 0];
  };
  if ladderize_left {
    left.ladderize();
  }
  let ti = restricted_to_shared(left, &shared, n);
  let tj = restricted_to_shared(right, &shared, n);
  if strict {
    // Order leaves using liberally resolved copies, then apply that order.
    let (mut ti, mut tj) = (ti.into_owned(), tj.into_owned());
    let (_, skipped) = resolve_with_mccs_quiet(&mut ti, &mut tj, mccs, n, false);
    ti.ladderize();
    sort_polytomies_by_mccs(&ti, &mut tj, mccs, n);
    let (oi, oj) = (leaf_order(&ti), leaf_order(&tj));
    sort_by_leaf_order(left, &oi);
    sort_by_leaf_order(right, &oj);
    skipped
  } else {
    if matches!((&ti, &tj), (Cow::Borrowed(_), Cow::Borrowed(_))) {
      sort_polytomies_by_mccs(left, right, mccs, n);
    } else {
      let mut tj = tj.into_owned();
      sort_polytomies_by_mccs(&ti, &mut tj, mccs, n);
      sort_by_leaf_order(right, &leaf_order(&tj));
    }
    [0, 0]
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::pipeline::tests::{ONE_TREE_LEAVES, PAIR_INPUTS, POLYTOMY_PAIR, TWO_TREE_EXAMPLE, ids, logged, same_leaves};
  use crate::pipeline::{Inference, infer, match_topologies, resolve_pair, run};
  use crate::tree::Taxa;
  use crate::tree::test_util::trees;
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  /// Each node in preorder with its name and the names of its children in order: the topology,
  /// the internal node names, and the leaf order of `t`.
  fn layout(t: &Tree) -> Vec<(String, Vec<String>)> {
    t.preorder()
      .into_iter()
      .map(|v| {
        let children = t.children(v).iter().map(|&c| t.nodes[c].name.clone()).collect();
        (t.nodes[v].name.clone(), children)
      })
      .collect()
  }

  /// The trees of a run of the pair `nwks` with `resolution`, sorted by the run, and the same
  /// trees rebuilt from the input with the run's own steps before its sort: unchanged without
  /// resolution, resolved with the MCCs in strict mode, and in matched mode also matched within
  /// MCCs. The MCCs are those of the result restricted to the shared leaves; in matched mode no
  /// MCC is split, so they are also the MCCs before matching.
  fn run_and_rebuilt(nwks: [&str; 2], resolution: Resolution) -> (Vec<Tree>, Vec<Tree>) {
    let o = Options {
      n_t: 10,
      resolution,
      ..Options::for_trees(2)
    };
    let (mut ran, taxa) = trees(&nwks);
    let n = taxa.len();
    let res = run(&mut ran, &taxa, &o, 3);
    let mccs = res[0].shared_mccs(&ran, n);
    let (mut copies, _) = trees(&nwks);
    if resolution != Resolution::None {
      resolve_pair(&mut copies, 0, 1, &mccs, n, true);
    }
    if resolution == Resolution::Matched {
      let mut matched = vec![mccs.clone()];
      let (_, split) = match_topologies(&mut copies, &[(0, 1)], &mut matched, n);
      assert_eq!((0, &mccs), (split, &matched[0]));
    }
    let rebuilt = copies.clone();
    let [left, right] = copies.get_disjoint_mut([0, 1]).unwrap();
    sort_for_pair(left, right, &mccs, n, sort_strictness(&o, 2));
    assert_eq!(
      (layout(&ran[0]), layout(&ran[1])),
      (layout(&copies[0]), layout(&copies[1]))
    );
    (ran, rebuilt)
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::two_tree_example(TWO_TREE_EXAMPLE)]
  #[case::polytomy(        POLYTOMY_PAIR)]
  #[case::one_tree_leaves( ONE_TREE_LEAVES)]
  #[trace]
  fn sort_for_pair_reproduces_the_run_order_of_two_trees(
    #[case] nwks: [&str; 2],
    #[values(Resolution::None, Resolution::Strict, Resolution::Matched)] resolution: Resolution,
  ) {
    run_and_rebuilt(nwks, resolution);
  }

  #[test]
  fn the_run_sort_reorders_some_pair_input() {
    // Without a reordered input, the order check of the sort would hold for an identity sort.
    let reordered: Vec<bool> = [Resolution::None, Resolution::Strict, Resolution::Matched]
      .into_iter()
      .flat_map(|resolution| PAIR_INPUTS.map(|nwks| (nwks, resolution)))
      .map(|(nwks, resolution)| {
        let (ran, rebuilt) = run_and_rebuilt(nwks, resolution);
        rebuilt.iter().zip(&ran).any(|(c, r)| c.leaf_names() != r.leaf_names())
      })
      .collect();
    assert!(reordered.contains(&true), "{reordered:?}");
  }

  /// Ladderizing the first tree would reorder it, so an unchanged order shows no sort.
  #[rustfmt::skip]
  #[rstest]
  #[case::no_shared_leaf( ["(A,((B,C),D));", "(P,((Q,R),S));"])]
  #[case::one_shared_leaf(["(A,((B,C),D));", "(A,((Q,R),S));"])]
  #[trace]
  fn sort_for_pair_leaves_a_skipped_pair_unchanged(#[case] nwks: [&str; 2], #[values(false, true)] strict: bool) {
    let (mut ts, taxa) = trees(&nwks);
    let before: Vec<_> = ts.iter().map(layout).collect();
    let [left, right] = ts.get_disjoint_mut([0, 1]).unwrap();
    sort_for_pair(left, right, &[], taxa.len(), strict);
    assert_eq!(before, ts.iter().map(layout).collect::<Vec<_>>());
  }

  #[test]
  fn sort_for_pair_logs_nothing_where_the_run_warns() {
    // The MCCs cut the clade (A,B), so resolving the copies of a strict sort skips one split
    // in each tree. The run's sort warns about each; the display sort sorts the same and logs
    // nothing.
    let nwks = ["((A,B),C,D,E);", "((A,B),C,D,E);"];
    let (mut ran, taxa) = trees(&nwks);
    let n = taxa.len();
    let mccs = ids(&taxa, &[&["A", "C", "E"], &["B", "D"]]);
    let warned = logged(|| sort_pair(&mut ran, 0, 1, &mccs, n, true));
    let warning = "WARN skipping split incompatible with tree t".to_owned();
    assert_eq!(vec![warning.clone(), warning], warned);
    let (mut copies, _) = trees(&nwks);
    let [left, right] = copies.get_disjoint_mut([0, 1]).unwrap();
    assert!(logged(|| sort_for_pair(left, right, &mccs, n, true)).is_empty());
    assert_eq!(
      ran.iter().map(layout).collect::<Vec<_>>(),
      copies.iter().map(layout).collect::<Vec<_>>()
    );
  }

  /// The extra round of strict resolution with three trees runs without resolution and sorts
  /// non-strictly; matched trees agree within MCCs, so their sort is never strict.
  #[rustfmt::skip]
  #[rstest]
  #[case::strict_two_trees(          (Resolution::Strict,  2, true,  None),        true)]
  #[case::strict_extra_round(        (Resolution::Strict,  3, true,  None),        false)]
  #[case::strict_without_extra_round((Resolution::Strict,  3, false, None),        true)]
  #[case::liberal(                   (Resolution::Liberal, 2, true,  None),        false)]
  #[case::none(                      (Resolution::None,    2, true,  None),        false)]
  #[case::none_forced_strict(        (Resolution::None,    2, true,  Some(true)),  true)]
  #[case::strict_forced_lax(         (Resolution::Strict,  2, true,  Some(false)), false)]
  #[case::matched(                   (Resolution::Matched, 2, true,  None),        false)]
  #[case::matched_forced_strict(     (Resolution::Matched, 2, true,  Some(true)),  false)]
  #[trace]
  fn sort_strictness_follows_the_last_round_of_the_run(
    #[case] (resolution, k, final_unresolved_round, sort_strict): (Resolution, usize, bool, Option<bool>),
    #[case] strict: bool,
  ) {
    let opts = Options {
      resolution,
      final_unresolved_round,
      sort_strict,
      ..Options::for_trees(k)
    };
    assert_eq!(strict, sort_strictness(&opts, k));
  }

  /// The last sorting pair of each tree, and the pairs whose trees keep the run's order.
  fn sorting(ts: &[Tree], taxa: &Taxa, opts: &Options) -> (Vec<Option<(usize, usize)>>, Vec<(usize, usize)>) {
    let n = taxa.len();
    let last = (0..ts.len()).map(|t| last_sorting_pair(ts, opts, n, t)).collect();
    let kept = pipeline_pairs(ts.len())
      .into_iter()
      .filter(|&(i, j)| keeps_run_order(ts, opts, n, i, j))
      .collect();
    (last, kept)
  }

  #[rstest]
  #[trace]
  fn last_sorting_pair_of_two_trees_is_their_pair(
    #[values(Resolution::None, Resolution::Strict, Resolution::Matched)] resolution: Resolution,
  ) {
    let (ts, taxa) = same_leaves(2);
    let o = Options {
      resolution,
      ..Options::for_trees(2)
    };
    let expected = (vec![Some((0, 1)), Some((0, 1))], vec![(0, 1)]);
    assert_eq!(expected, sorting(&ts, &taxa, &o));
  }

  #[test]
  fn last_sorting_pair_of_three_and_four_trees() {
    // A non-strict sort reorders the second tree of a pair, and the first tree only when it is
    // tree 0 (ladderized); a strict sort reorders both trees.
    let non_strict = Options::for_trees(3);
    let strict = Options {
      resolution: Resolution::Strict,
      final_unresolved_round: false,
      ..Options::for_trees(3)
    };
    let (ts, taxa) = same_leaves(3);
    assert_eq!(
      (vec![Some((0, 2)), Some((0, 1)), Some((1, 2))], vec![(1, 2)]),
      sorting(&ts, &taxa, &non_strict)
    );
    assert_eq!(
      (vec![Some((0, 2)), Some((1, 2)), Some((1, 2))], vec![(1, 2)]),
      sorting(&ts, &taxa, &strict)
    );
    let (ts, taxa) = same_leaves(4);
    let four = |o: &Options| Options {
      seq_lengths: vec![1.0; 4],
      ..o.clone()
    };
    assert_eq!(
      (
        vec![Some((0, 3)), Some((0, 1)), Some((1, 2)), Some((2, 3))],
        vec![(1, 2), (2, 3)]
      ),
      sorting(&ts, &taxa, &four(&non_strict))
    );
    assert_eq!(
      (
        vec![Some((0, 3)), Some((1, 3)), Some((2, 3)), Some((2, 3))],
        vec![(2, 3)]
      ),
      sorting(&ts, &taxa, &four(&strict))
    );
  }

  #[test]
  fn last_sorting_pair_skips_pairs_without_two_shared_leaves() {
    // Tree 2 shares only A with tree 1, so the pair (1, 2) is not sorted.
    let (ts, taxa) = trees(&["((A,B),(C,P));", "((A,B),(C,D));", "((A,P),(Q,R));"]);
    let o = Options::for_trees(3);
    assert_eq!(
      (vec![Some((0, 2)), Some((0, 1)), Some((0, 2))], vec![(0, 2)]),
      sorting(&ts, &taxa, &o)
    );
  }

  /// Replay a sequential run (`opts.parallel` off, no pre-resolution) of `trees` with seed 1
  /// step by step, and return for each pair a copy of its two trees right before the run sorts
  /// it (`None` for a pair without two shared leaves, which the sort leaves unchanged).
  fn replay_sorts(trees: &mut [Tree], opts: &Options, n: usize) -> Vec<Option<[Tree; 2]>> {
    assert!(!opts.parallel && !opts.pre_resolve, "replay of a sequential run only");
    let k = trees.len();
    let pairs = pipeline_pairs(k);
    let mut mccs = vec![Vec::new(); pairs.len()];
    let mut before = vec![None; pairs.len()];
    let matched = opts.resolution == Resolution::Matched;
    let (rounds, extra_round) = schedule(opts, k);
    let strict_sort = sort_strictness(opts, k);
    let mut sort = |trees: &mut [Tree], p: usize, mccs: &[Mcc]| {
      let (i, j) = pairs[p];
      if shared_leaves(&trees[i], &trees[j], n).is_some() {
        before[p] = Some([trees[i].clone(), trees[j].clone()]);
      }
      sort_pair(trees, i, j, mccs, n, strict_sort);
    };
    for round in 1..=rounds {
      let last = round == rounds;
      let (resolve, strict) = round_mode(opts, extra_round && last);
      let inference = Inference {
        opts,
        n,
        seed: 1,
        round,
        resolve,
      };
      for (p, &(i, j)) in pairs.iter().enumerate() {
        mccs[p] = infer(trees, i, j, &inference, &|_| {});
        if resolve {
          resolve_pair(trees, i, j, &mccs[p], n, strict);
        }
        if last && !matched {
          sort(trees, p, &mccs[p]);
        }
      }
    }
    if matched {
      match_topologies(trees, &pairs, &mut mccs, n);
      for (p, m) in mccs.iter().enumerate() {
        sort(trees, p, m);
      }
    }
    before
  }

  /// Inputs of three and four trees for the run-order model: trees on the same leaves with
  /// polytomies, and four trees in which the pair (2, 3) shares only the leaf A.
  const RUN_ORDER_INPUTS: [&[&str]; 3] = [
    &["((D,A),(B,E),C);", "(E,(B,C),(D,A));", "((B,C),D,(E,A));"],
    &[
      "(A,(E,C,D),B);",
      "(D,((B,E),A,C));",
      "((A,((E,C),B)),D);",
      "((E,A,(C,D)),B);",
    ],
    &[
      "((A,B),(C,(D,X)));",
      "((A,(B,X)),(C,D));",
      "((A,X),(C,D));",
      "((B,P),(Q,A));",
    ],
  ];

  #[test]
  fn keeps_run_order_matches_runs_of_three_and_four_trees() {
    // For each pair that the model calls kept, the run's final trees must equal the run's sort
    // of the pair applied to copies of the pair's trees taken right before the run sorted it.
    // The copies come from a replay of the run's steps, which must end with the run's trees.
    let mut kept_pairs = Vec::new();
    let mut mismatches = Vec::new();
    let mut reordered_later = 0;
    for nwks in RUN_ORDER_INPUTS {
      let k = nwks.len();
      let o = |resolution, final_unresolved_round| Options {
        resolution,
        final_unresolved_round,
        n_t: 10,
        parallel: false,
        ..Options::for_trees(k)
      };
      let configs = [
        o(Resolution::None, true),
        o(Resolution::Strict, true),
        o(Resolution::Strict, false),
        o(Resolution::Liberal, true),
        o(Resolution::Matched, true),
      ];
      for opts in configs {
        let case = format!("{:?}, {}, {nwks:?}", opts.resolution, opts.final_unresolved_round);
        let (mut ran, taxa) = trees(nwks);
        let n = taxa.len();
        let res = run(&mut ran, &taxa, &opts, 1);
        let (mut replayed, _) = trees(nwks);
        let before = replay_sorts(&mut replayed, &opts, n);
        if ran.iter().map(layout).ne(replayed.iter().map(layout)) {
          mismatches.push(format!("the replay ends with other trees than the run: {case}"));
        }
        for (r, copies) in res.iter().zip(before) {
          let kept = keeps_run_order(&ran, &opts, n, r.i, r.j);
          let Some([mut left, mut right]) = copies else {
            if kept {
              mismatches.push(format!("({}, {}) kept but not sorted: {case}", r.i, r.j));
            }
            continue;
          };
          let mccs = r.shared_mccs(&ran, n);
          sort_two(&mut left, &mut right, r.i == 0, &mccs, n, sort_strictness(&opts, k));
          let same = layout(&left) == layout(&ran[r.i]) && layout(&right) == layout(&ran[r.j]);
          if kept {
            kept_pairs.push((k, r.i, r.j));
            if !same {
              mismatches.push(format!("({}, {}) kept but reordered later: {case}", r.i, r.j));
            }
          }
          reordered_later += usize::from(!same);
        }
      }
    }
    assert_eq!(Vec::<String>::new(), mismatches);
    // The check is not vacuous: kept pairs with i > 0 exist for three and four trees, and later
    // sorts reorder the trees of some pairs, which a wrongly kept pair would show.
    assert!(kept_pairs.iter().any(|&(k, i, _)| k == 3 && i > 0), "{kept_pairs:?}");
    assert!(kept_pairs.iter().any(|&(k, i, _)| k == 4 && i > 0), "{kept_pairs:?}");
    assert!(reordered_later > 0);
  }

  #[test]
  fn last_sorting_pair_is_none_for_a_run_without_rounds() {
    let (ts, taxa) = same_leaves(2);
    let o = Options {
      rounds: 0,
      resolution: Resolution::None,
      ..Options::for_trees(2)
    };
    assert_eq!((vec![None, None], vec![]), sorting(&ts, &taxa, &o));
  }

  #[test]
  #[should_panic(expected = "tree 2 is not one of the 2 trees")]
  fn last_sorting_pair_rejects_a_tree_out_of_range() {
    let (ts, taxa) = same_leaves(2);
    last_sorting_pair(&ts, &Options::for_trees(2), taxa.len(), 2);
  }

  #[test]
  #[should_panic(expected = "(1, 0) is not a pair i < j of the 2 trees")]
  fn keeps_run_order_rejects_a_reversed_pair() {
    let (ts, taxa) = same_leaves(2);
    keeps_run_order(&ts, &Options::for_trees(2), taxa.len(), 1, 0);
  }

  #[test]
  #[should_panic(expected = "(0, 0) is not a pair i < j of the 2 trees")]
  fn keeps_run_order_rejects_a_tree_paired_with_itself() {
    let (ts, taxa) = same_leaves(2);
    keeps_run_order(&ts, &Options::for_trees(2), taxa.len(), 0, 0);
  }

  #[test]
  #[should_panic(expected = "(0, 2) is not a pair i < j of the 2 trees")]
  fn keeps_run_order_rejects_a_tree_out_of_range() {
    let (ts, taxa) = same_leaves(2);
    keeps_run_order(&ts, &Options::for_trees(2), taxa.len(), 0, 2);
  }

  #[test]
  #[should_panic(expected = "an MCC holds a leaf that is not one of the 4 taxa")]
  fn sort_for_pair_rejects_an_mcc_leaf_out_of_range() {
    let (mut ts, taxa) = same_leaves(2);
    let [left, right] = ts.get_disjoint_mut([0, 1]).unwrap();
    sort_for_pair(left, right, &[vec![0, 1, 2, 3, 4]], taxa.len(), false);
  }

  #[test]
  #[should_panic(expected = "a tree of the pair has a leaf that is not one of the 3 taxa")]
  fn sort_for_pair_rejects_a_tree_leaf_out_of_range() {
    let (mut ts, _) = same_leaves(2);
    let [left, right] = ts.get_disjoint_mut([0, 1]).unwrap();
    sort_for_pair(left, right, &[], 3, false);
  }

  #[test]
  #[should_panic(expected = "a tree of the pair has a leaf that is not one of the 4 taxa")]
  fn sort_for_pair_rejects_a_leaf_without_taxon() {
    let (mut ts, taxa) = same_leaves(2);
    let leaf = ts[1].leaves()[0];
    ts[1].nodes[leaf].taxon = None;
    let [left, right] = ts.get_disjoint_mut([0, 1]).unwrap();
    sort_for_pair(left, right, &[], taxa.len(), false);
  }
}
