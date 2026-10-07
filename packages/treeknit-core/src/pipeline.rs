//! TreeKnit on K ≥ 2 trees: pre-resolution, rounds of pair inference, polytomy sorting,
//! and placement of leaves missing from one tree of a pair.

#![expect(
  clippy::as_conversions,
  clippy::disallowed_types,
  clippy::expect_used,
  clippy::unwrap_used,
  reason = "findings from before the strict lint set; kb/issues/N-lint-baseline.md tracks their removal"
)]

use crate::bits::{self, Bits};
use crate::impute::{Attachment, attach_private, graft_attachments};
use crate::mcc_map::{leaf_order, sort_by_leaf_order, sort_polytomies_by_mccs};
use crate::naive::{Mcc, naive_mccs, sort_mccs};
use crate::options::{Options, Resolution};
use crate::pair::{PairParams, infer_pair};
use crate::progress::Progress;
use crate::resolve::{
  Insert, insert_all_on, insert_split, resolve_trees, resolve_with_mccs, resolve_with_mccs_quiet, warn_skipped,
};
use crate::tree::{Taxa, Tree};
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use rayon::prelude::*;
use std::borrow::Cow;

/// MCCs of one pair of trees.
#[derive(Clone, Debug)]
pub struct PairResult {
  pub i: usize,
  pub j: usize,
  /// MCCs over shared leaves, extended by attached private leaves; sorted.
  pub mccs: Vec<Mcc>,
  /// Leaves that are in exactly one tree of the pair, with their attachment.
  pub attached: Vec<Attachment>,
}

impl PairResult {
  /// Index in `mccs` of the MCC containing `leaf`.
  pub fn mcc_of(&self, leaf: usize) -> Option<usize> {
    self.mccs.iter().position(|m| m.binary_search(&leaf).is_ok())
  }

  /// MCCs restricted to the leaves that trees `i` and `j` of `trees` share; sorted. These are
  /// the MCCs the run sorted the pair by, before the leaves of one tree only were attached.
  /// After `Matched` resolution they hold the parts of every MCC that matching split, so they
  /// can differ from the MCCs inferred for the pair. Empty when the pair shares fewer than two
  /// leaves.
  ///
  /// # Panics
  ///
  /// If the trees `i` and `j` of the pair are not trees of `trees`, or have a leaf without a
  /// taxon of `0..n`.
  pub fn shared_mccs(&self, trees: &[Tree], n: usize) -> Vec<Mcc> {
    assert!(
      self.i < self.j && self.j < trees.len(),
      "({}, {}) is not a pair i < j of the {} trees",
      self.i,
      self.j,
      trees.len()
    );
    assert!(
      trees[self.i].has_taxa_below(n) && trees[self.j].has_taxa_below(n),
      "a tree of the pair has a leaf that is not one of the {n} taxa"
    );
    let Some(shared) = shared_leaves(&trees[self.i], &trees[self.j], n) else {
      return Vec::new();
    };
    let restricted = self
      .mccs
      .iter()
      .map(|m| m.iter().copied().filter(|&x| shared.contains(x)).collect::<Mcc>())
      .filter(|m| !m.is_empty())
      .collect();
    sort_mccs(restricted)
  }
}

/// Number of rounds of a run on `k` trees, and whether the last of them is the extra round
/// without resolution.
fn schedule(opts: &Options, k: usize) -> (usize, bool) {
  let extra_round = opts.extra_unresolved_round(k);
  (opts.rounds + extra_round as usize, extra_round)
}

/// Whether a round resolves the trees, and whether it adds only unambiguous splits with MCCs
/// (all splits in liberal mode). `unresolved_round` is the extra round without resolution.
fn round_mode(opts: &Options, unresolved_round: bool) -> (bool, bool) {
  let resolve = opts.resolves() && !unresolved_round;
  (resolve, opts.resolution != Resolution::Liberal && resolve)
}

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

/// Pairs of `k` trees in pipeline order (0,1), (0,2), …, (1,2), …
fn pipeline_pairs(k: usize) -> Vec<(usize, usize)> {
  (0..k).flat_map(|i| (i + 1..k).map(move |j| (i, j))).collect()
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
/// `mccs` are the MCCs over the leaves the two trees share (see [`PairResult::shared_mccs`]),
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

/// Run TreeKnit on `trees` (leaves must have taxa assigned from `taxa`).
/// Trees are resolved and sorted in place. Pairs are returned in order (0,1), (0,2), …
pub fn run(trees: &mut [Tree], taxa: &Taxa, opts: &Options, seed: u64) -> Vec<PairResult> {
  run_observed(trees, taxa, opts, seed, &|_| {})
}

/// As [`run`], calling `observe` with the progress of the run: at the start of each pair, after
/// each temperature step of its annealing, once when the matching of `Matched` resolution
/// starts, and once with fraction 1 at the end; every earlier event has a fraction of at most
/// [`PAIRS_SHARE`](crate::progress::PAIRS_SHARE). Pre-resolution reports nothing. In a parallel
/// round, `observe` is called on the calling thread only: before the round and after all its
/// pairs. The observer consumes no random numbers, so the result equals that of `run`.
pub fn run_observed(
  trees: &mut [Tree],
  taxa: &Taxa,
  opts: &Options,
  seed: u64,
  observe: &dyn Fn(Progress),
) -> Vec<PairResult> {
  let k = trees.len();
  let n = taxa.len();
  assert!(k >= 2, "need at least two trees");
  assert!(opts.seq_lengths.len() == k, "need one sequence length per tree");
  if opts.pre_resolve {
    log::info!("pre-resolving all trees with each other");
    let new = resolve_trees(trees, n);
    log::debug!(
      "new splits per tree: {:?}",
      new.iter().map(|s| s.len()).collect::<Vec<_>>()
    );
  }
  let pairs = pipeline_pairs(k);
  let mut mccs: Vec<Vec<Mcc>> = vec![Vec::new(); pairs.len()];
  let matched = opts.resolution == Resolution::Matched;
  let (rounds, extra_round) = schedule(opts, k);
  let sort_strict = sort_strictness(opts, k);
  let reached = std::cell::Cell::new(0.0);
  let report = |p: Progress| {
    reached.set(p.fraction);
    observe(p);
  };
  for round in 1..=rounds {
    let last = round == rounds;
    // Splits added with MCCs: unambiguous ones only, except in liberal mode.
    let (resolve, strict) = round_mode(opts, extra_round && last);
    log::info!("round {round}/{rounds}{}", if resolve { " (resolving)" } else { "" });
    let at = |pair: usize, within: f64| report(Progress::at(round - 1, rounds, pair, pairs.len(), within));
    let inference = Inference {
      opts,
      n,
      seed,
      round,
      resolve,
    };
    if resolve || !opts.parallel {
      for (p, &(i, j)) in pairs.iter().enumerate() {
        at(p, 0.0);
        mccs[p] = infer(trees, i, j, &inference, &|within| at(p, within));
        if resolve {
          resolve_pair(trees, i, j, &mccs[p], n, strict);
        }
        if last && !matched {
          sort_pair(trees, i, j, &mccs[p], n, sort_strict);
        }
      }
    } else {
      let shared: &[Tree] = trees;
      at(0, 0.0);
      mccs = pairs
        .par_iter()
        .map(|&(i, j)| infer(shared, i, j, &inference, &|_| {}))
        .collect();
      at(pairs.len() - 1, 1.0);
      if last && !matched {
        for (p, &(i, j)) in pairs.iter().enumerate() {
          sort_pair(trees, i, j, &mccs[p], n, sort_strict);
        }
      }
    }
  }
  if matched {
    report(Progress::matching(reached.get(), rounds, pairs.len()));
    match_topologies(trees, &pairs, &mut mccs, n);
    for (p, &(i, j)) in pairs.iter().enumerate() {
      sort_pair(trees, i, j, &mccs[p], n, sort_strict);
    }
  }
  let results = pairs
    .iter()
    .zip(mccs)
    .map(|(&(i, j), m)| attach_pair(trees, i, j, m, n))
    .collect();
  report(Progress::done(rounds, pairs.len()));
  results
}

/// Resolve `trees` so that, within every MCC of every pair, the two trees restricted to the
/// MCC's leaves have the same topology. Earlier trees take precedence.
///
/// Pairs are visited in order (0,1), (0,2), …, (1,2), …; for each, the splits that either tree
/// has inside their shared MCCs are inserted into the other. Passes are
/// repeated until nothing changes, so splits propagate through chains of shared regions. A
/// split is only inserted if compatible with what a tree already has, so splits of earlier
/// trees win conflicts; no split is ever removed. MCCs whose topologies still differ after
/// propagation (conflicting splits from different trees) are replaced by the maximal clades
/// on which the two trees agree, i.e. additional reassortments are inferred there.
///
/// Returns the number of splits added and the number of MCCs that had to be split.
pub fn match_topologies(
  trees: &mut [Tree],
  pairs: &[(usize, usize)],
  mccs: &mut [Vec<Mcc>],
  n: usize,
) -> (usize, usize) {
  let mut added = 0;
  for pass in 1..=20 {
    let before = added;
    for (p, &(i, j)) in pairs.iter().enumerate() {
      added += propagate_splits(trees, i, j, &mccs[p], n);
      added += propagate_splits(trees, j, i, &mccs[p], n);
    }
    log::debug!("matching topologies, pass {pass}: {} splits added", added - before);
    if added == before {
      break;
    }
  }
  let mut split = 0;
  for (p, &(i, j)) in pairs.iter().enumerate() {
    let mut out: Vec<Mcc> = Vec::with_capacity(mccs[p].len());
    for m in std::mem::take(&mut mccs[p]) {
      match mismatched_restrictions(&trees[i], &trees[j], &m, n) {
        None => out.push(m),
        Some((ri, rj)) => {
          let parts = naive_mccs(&[&ri, &rj], n);
          log::info!(
            "MCC of {} leaves has conflicting topologies in {} and {}: split into {} MCCs",
            m.len(),
            trees[i].label,
            trees[j].label,
            parts.len()
          );
          split += 1;
          out.extend(parts);
        },
      }
    }
    mccs[p] = sort_mccs(out);
  }
  log::info!("matched topologies within MCCs: {added} splits added, {split} MCCs split");
  (added, split)
}

/// Insert into tree `dst` the splits tree `src` has inside each MCC, restricted to the MCC's
/// leaves (where the placement of other branches is free). Splits that conflict with `dst` are
/// skipped. Returns the number inserted.
fn propagate_splits(trees: &mut [Tree], src: usize, dst: usize, mccs: &[Mcc], n: usize) -> usize {
  let [s, d] = trees
    .get_disjoint_mut([src, dst])
    .expect("the source and destination trees differ");
  let s = &*s;
  let clades = s.clades(n);
  let leaf_of = s.leaf_of(n);
  let mut label = d.fresh_index("RESOLVED");
  let mut added = 0;
  for m in mccs.iter().filter(|m| m.len() >= 3) {
    let mask = bits::from_iter(n, m.iter().copied());
    let Some(r) = s.lca_of(m.iter().filter_map(|&x| leaf_of[x])) else {
      continue;
    };
    for v in s.postorder_from(r).into_iter().filter(|&v| !s.is_leaf(v)) {
      let split = &clades[v] & &mask;
      if split.count_ones(..) < 2 || split == mask {
        continue;
      }
      if insert_split(d, &split, &mask, n, &format!("RESOLVED_{label}")) == Insert::Added {
        label += 1;
        added += 1;
      }
    }
  }
  added
}

/// MCCs whose two trees, restricted to the MCC's leaves present in both, have different
/// topologies: `(pair index, MCC index)`. Empty after `match_topologies`.
pub fn unmatched_mccs(trees: &[Tree], results: &[PairResult], n: usize) -> Vec<(usize, usize)> {
  let mut out = Vec::new();
  for (p, r) in results.iter().enumerate() {
    let shared = &trees[r.i].leaf_set(n) & &trees[r.j].leaf_set(n);
    for (k, m) in r.mccs.iter().enumerate() {
      let m: Mcc = m.iter().copied().filter(|&x| shared.contains(x)).collect();
      if mismatched_restrictions(&trees[r.i], &trees[r.j], &m, n).is_some() {
        out.push((p, k));
      }
    }
  }
  out
}

/// `t1` and `t2` restricted to the leaves of `m`, if their topologies differ there.
fn mismatched_restrictions(t1: &Tree, t2: &Tree, m: &Mcc, n: usize) -> Option<(Tree, Tree)> {
  if m.len() < 3 {
    return None;
  }
  let keep = bits::from_iter(n, m.iter().copied());
  let (r1, r2) = (t1.restricted(&keep)?, t2.restricted(&keep)?);
  if internal_clades(&r1, n) == internal_clades(&r2, n) {
    None
  } else {
    Some((r1, r2))
  }
}

/// Clades of the internal non-root nodes of `t`.
fn internal_clades(t: &Tree, n: usize) -> std::collections::HashSet<Bits> {
  let c = t.clades(n);
  t.internals()
    .into_iter()
    .filter(|&v| v != t.root)
    .map(|v| c[v].clone())
    .collect()
}

/// Leaves common to trees `a` and `b`, or `None` if they share fewer than two leaves.
///
/// This is the only check of the shared-leaf count: a pair without two shared leaves has no
/// MCCs, and resolution and sorting leave its trees unchanged.
fn shared_leaves(a: &Tree, b: &Tree, n: usize) -> Option<Bits> {
  let shared = &a.leaf_set(n) & &b.leaf_set(n);
  (shared.count_ones(..) >= 2).then_some(shared)
}

/// Trees `a` and `b` restricted to their `shared` leaves (borrowed if no restriction is
/// needed). `shared` comes from `shared_leaves`, so it holds at least two leaves.
fn restrict_pair<'a>(a: &'a Tree, b: &'a Tree, shared: &Bits, n: usize) -> (Cow<'a, Tree>, Cow<'a, Tree>) {
  let r = |t: &'a Tree| {
    if t.leaf_set(n) == *shared {
      Cow::Borrowed(t)
    } else {
      Cow::Owned(t.restricted(shared).expect("shared leaves are in both trees"))
    }
  };
  (r(a), r(b))
}

/// Settings that all pairs of one round infer their MCCs with.
struct Inference<'a> {
  opts: &'a Options,
  /// Number of taxa.
  n: usize,
  /// Seed of the run, mixed with the round and the pair into the seed of each pair.
  seed: u64,
  /// Round, 1-based.
  round: usize,
  /// Whether the round resolves the trees with the MCCs it infers.
  resolve: bool,
}

/// Infer the MCCs of trees `i` and `j` on their shared leaves; none if they share fewer than two.
fn infer(trees: &[Tree], i: usize, j: usize, inference: &Inference<'_>, on_progress: &dyn Fn(f64)) -> Vec<Mcc> {
  let &Inference {
    opts,
    n,
    seed,
    round,
    resolve,
  } = inference;
  let Some(shared) = shared_leaves(&trees[i], &trees[j], n) else {
    log::warn!(
      "trees {} and {} share fewer than two leaves: skipped",
      trees[i].label,
      trees[j].label
    );
    return Vec::new();
  };
  log::info!(
    "inferring MCCs for {} and {} ({} shared leaves)",
    trees[i].label,
    trees[j].label,
    shared.count_ones(..)
  );
  let (ti, tj) = restrict_pair(&trees[i], &trees[j], &shared, n);
  let m = if opts.naive {
    naive_mccs(&[&ti, &tj], n)
  } else {
    let p = PairParams {
      gamma: opts.gamma,
      itmax: opts.itmax,
      likelihood_sort: opts.likelihood_sort,
      resolve,
      seq_lengths: [opts.seq_lengths[i], opts.seq_lengths[j]],
      n_mcmc: opts.n_mcmc,
      sa_rep: opts.sa_rep,
      temperatures: opts.temperatures(),
    };
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(mix(seed, round, i, j));
    infer_pair(&ti, &tj, n, &p, &mut rng, on_progress)
  };
  log::info!("found {} MCCs for {} and {}", m.len(), trees[i].label, trees[j].label);
  m
}

/// Per-pair seed, independent of execution order.
fn mix(seed: u64, round: usize, i: usize, j: usize) -> u64 {
  let mut z = seed ^ (((round as u64) << 40) | ((i as u64) << 20) | j as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
  z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
  z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
  z ^ (z >> 31)
}

/// Resolve trees `i` and `j` with their MCCs (computed on shared leaves). A pair that shares
/// fewer than two leaves is left unchanged.
/// Returns the number of splits added to the two trees.
fn resolve_pair(trees: &mut [Tree], i: usize, j: usize, mccs: &[Mcc], n: usize, strict: bool) -> usize {
  let Some(shared) = shared_leaves(&trees[i], &trees[j], n) else {
    return 0;
  };
  let (ti, tj) = restrict_pair(&trees[i], &trees[j], &shared, n);
  let (mut ti, mut tj) = (ti.into_owned(), tj.into_owned());
  let [mut si, mut sj] = resolve_with_mccs(&mut ti, &mut tj, mccs, n, strict);
  log::debug!(
    "resolved {} new splits in {}, {} in {}",
    si.len(),
    trees[i].label,
    sj.len(),
    trees[j].label
  );
  insert_all_on(&mut trees[i], &mut si, &shared, n);
  insert_all_on(&mut trees[j], &mut sj, &shared, n);
  si.len() + sj.len()
}

/// Sort trees `i < j` as the run's last round does: ladderize the first tree of the run and
/// order polytomies so that MCCs face each other, logging the splits that resolving the copies
/// of a strict sort skips. A pair that shares fewer than two leaves is left unchanged.
fn sort_pair(trees: &mut [Tree], i: usize, j: usize, mccs: &[Mcc], n: usize, strict: bool) {
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
  let (ti, tj) = restrict_pair(left, right, &shared, n);
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

/// Attach leaves of either tree that the other lacks, and extend the MCCs accordingly.
fn attach_pair(trees: &[Tree], i: usize, j: usize, mut mccs: Vec<Mcc>, n: usize) -> PairResult {
  let shared = &trees[i].leaf_set(n) & &trees[j].leaf_set(n);
  let mut attached = Vec::new();
  if !mccs.is_empty() {
    attached.extend(attach_private(&trees[i], i, &shared, &mccs, n));
    attached.extend(attach_private(&trees[j], j, &shared, &mccs, n));
  }
  for a in &attached {
    mccs[a.mcc].extend(&a.leaves);
  }
  // Re-index attachments to the sorted MCC list by a leaf of their MCC.
  let firsts: Vec<usize> = attached
    .iter()
    .map(|a| mccs[a.mcc].iter().min().copied().unwrap())
    .collect();
  let mccs = sort_mccs(mccs);
  for (a, first) in attached.iter_mut().zip(firsts) {
    a.mcc = mccs.iter().position(|m| m.binary_search(&first).is_ok()).unwrap();
  }
  PairResult { i, j, mccs, attached }
}

/// Copies of `trees` with every missing leaf placed, using for each tree `j` and each missing
/// leaf the pair (i, j) whose MCC containing the leaf is largest (ties: lowest i).
pub fn imputed_trees(trees: &[Tree], pairs: &[PairResult], n: usize) -> Vec<Tree> {
  let k = trees.len();
  let leafsets: Vec<Bits> = trees.iter().map(|t| t.leaf_set(n)).collect();
  (0..k)
    .map(|j| {
      let mut t = trees[j].clone();
      // best[x] = (MCC size, -i, attachment)
      let mut best: Vec<Option<(usize, std::cmp::Reverse<usize>, &Attachment)>> = vec![None; n];
      for p in pairs.iter().filter(|p| p.i == j || p.j == j) {
        for a in p.attached.iter().filter(|a| a.source != j) {
          let size = p.mccs[a.mcc].len();
          for &x in &a.leaves {
            let cand = (size, std::cmp::Reverse(a.source), a);
            if best[x].as_ref().is_none_or(|b| (b.0, b.1) < (cand.0, cand.1)) {
              best[x] = Some(cand);
            }
          }
        }
      }
      // Graft each attachment whose leaves all chose it; otherwise graft single leaves.
      let mut grafts: Vec<Attachment> = Vec::new();
      for x in (0..n).filter(|&x| !leafsets[j].contains(x)) {
        let Some((_, _, a)) = best[x] else { continue };
        if a.leaves.iter().all(|y| best[*y].is_some_and(|b| std::ptr::eq(b.2, a))) {
          if a.leaves[0] == x {
            grafts.push(a.clone());
          }
        } else {
          let src = &trees[a.source];
          let node = src.leaf_of(n)[x].unwrap();
          grafts.push(Attachment {
            node,
            leaves: vec![x],
            ..a.clone()
          });
        }
      }
      let refs: Vec<&Attachment> = grafts.iter().collect();
      graft_attachments(&mut t, trees, &refs, n);
      t
    })
    .collect()
}

/// Trees and MCCs from which to build the ARG of a pair (K = 2): the imputed trees,
/// restricted to shared leaves plus unambiguously attached ones.
pub fn arg_inputs(trees: &[Tree], pair: &PairResult, n: usize) -> (Tree, Tree, Vec<Mcc>) {
  let imputed = imputed_trees(trees, std::slice::from_ref(pair), n);
  let mut keep = &trees[pair.i].leaf_set(n) & &trees[pair.j].leaf_set(n);
  for a in pair.attached.iter().filter(|a| !a.ambiguous) {
    keep.extend(a.leaves.iter().copied());
  }
  let r = |t: &Tree| t.restricted(&keep).expect("no leaves for ARG");
  let mccs: Vec<Mcc> = pair
    .mccs
    .iter()
    .map(|m| m.iter().copied().filter(|&x| keep.contains(x)).collect::<Mcc>())
    .filter(|m| !m.is_empty())
    .collect();
  (r(&imputed[pair.i]), r(&imputed[pair.j]), sort_mccs(mccs))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::progress::{PAIRS_SHARE, Phase, ratio};
  use crate::tree::test_util::{splits, trees};
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  fn ids(taxa: &Taxa, m: &[&[&str]]) -> Vec<Mcc> {
    sort_mccs(m.iter().map(|x| x.iter().map(|s| taxa.index[*s]).collect()).collect())
  }

  fn all_matching(trees: &[Tree], pairs: &[(usize, usize)], mccs: &[Vec<Mcc>], n: usize) -> bool {
    pairs.iter().zip(mccs).all(|(&(i, j), ms)| {
      ms.iter()
        .all(|m| mismatched_restrictions(&trees[i], &trees[j], m, n).is_none())
    })
  }

  #[test]
  fn matching_two_trees_resolves_ambiguous_polytomy() {
    // Strict resolution leaves t2 unresolved (D might be nested in the MCC); matching
    // topologies requires the MCC's internal splits in both trees. The MCC's own clade
    // (A,B,C) is not needed: restricted to the MCC it is the root.
    let (mut ts, taxa) = trees(&["((A,(B,C)),D);", "(A,B,C,D);"]);
    let pairs = [(0, 1)];
    let mut mccs = vec![ids(&taxa, &[&["D"], &["A", "B", "C"]])];
    let (added, split) = match_topologies(&mut ts, &pairs, &mut mccs, taxa.len());
    assert_eq!((added, split), (1, 0));
    assert_eq!(splits(&ts[1], &taxa), vec![vec!["B", "C"]]);
    assert!(all_matching(&ts, &pairs, &mccs, taxa.len()));
  }

  #[test]
  fn matching_precedence_and_conflicts() {
    // Non-transitive MCCs: t1 and t2 both share everything with t0 but group B differently.
    let (mut ts, taxa) = trees(&["(A,B,C,D);", "((A,B),C,D);", "((B,C),A,D);"]);
    let n = taxa.len();
    let pairs = [(0, 1), (0, 2), (1, 2)];
    let all: &[&str] = &["A", "B", "C", "D"];
    let mut mccs = vec![
      ids(&taxa, &[all]),
      ids(&taxa, &[all]),
      ids(&taxa, &[&["A"], &["B", "C", "D"]]),
    ];
    let (_, split) = match_topologies(&mut ts, &pairs, &mut mccs, n);
    // t1 comes first: its grouping (A,B) is adopted by t0, t2's conflicting (B,C) is not.
    // (A,B,C) is consistent with all trees: within t1/t2's MCC {B,C,D}, t2's (B,C) has to
    // exist in t1, and A, outside that MCC, goes along with B.
    let s0 = splits(&ts[0], &taxa);
    assert!(s0.contains(&vec!["A".to_owned(), "B".into()]));
    assert!(!s0.contains(&vec!["B".to_owned(), "C".into()]));
    // Only t0/t2 still conflict ((A,B) vs (B,C)); their MCC is split.
    assert_eq!(split, 1);
    assert_eq!(mccs[0], ids(&taxa, &[all]));
    assert_eq!(mccs[2], ids(&taxa, &[&["A"], &["B", "C", "D"]]));
    assert!(mccs[1].len() > 1);
    assert!(all_matching(&ts, &pairs, &mccs, n));
  }

  #[test]
  fn three_trees_better_trees() {
    let (mut ts, taxa) = trees(&["((A,(B,C)),(D,E));", "((A,B,C,D),E);", "((A,B),((C,D),E));"]);
    let o = Options::treeknit_jl(3, None);
    let res = run(&mut ts, &taxa, &o, 1);
    // Every pair's MCCs hold all five leaves.
    let totals: Vec<usize> = res.iter().map(|r| r.mccs.iter().map(Vec::len).sum()).collect();
    assert_eq!(vec![5, 5, 5], totals);
  }

  #[test]
  fn partial_overlap_matches_prepruned() {
    let full = ["((A,B),(C,(D,(E,X))));", "((A,(B,X)),(C,D,E));"];
    let partial = ["((A,B),(C,(D,(E,X))));", "((A,(B,X)),(C,(D,P),E));"];
    let o = Options::treeknit_jl(2, None);
    let (mut a, ta) = trees(&full);
    let (mut b, tb) = trees(&partial);
    let ra = run(&mut a, &ta, &o, 7);
    let rb = run(&mut b, &tb, &o, 7);
    let names = |r: &PairResult, t: &Taxa| {
      let p = t.index.get("P").copied();
      r.mccs
        .iter()
        .map(|m| t.names_of(&m.iter().copied().filter(|&x| Some(x) != p).collect::<Vec<_>>()))
        .collect::<Vec<_>>()
    };
    assert_eq!(names(&ra[0], &ta), names(&rb[0], &tb));
    assert_eq!(rb[0].attached.len(), 1);
    let imp = imputed_trees(&b, &rb, tb.len());
    assert_eq!(imp[0].n_leaves(), 7);
    assert!(imp[0].check());
    assert!(splits(&imp[0], &tb).contains(&vec!["D".to_owned(), "P".to_owned()]));
  }

  /// Run `f` in a pool of four threads when `parallel`, so that the pairs of a parallel run run
  /// concurrently although the global pool of the tests has one thread.
  fn in_pool<T: Send>(parallel: bool, f: impl FnOnce() -> T + Send) -> T {
    if parallel {
      rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap()
        .install(f)
    } else {
      f()
    }
  }

  /// Two trees that share fewer than two leaves get no MCCs and no attachments and stay
  /// unchanged, in every resolution mode, sequentially and in parallel. Ladderizing either tree
  /// changes its leaf order, so an unchanged order shows that the pair was not sorted.
  #[rustfmt::skip]
  #[rstest]
  #[case::no_shared_leaf( ["(A,((B,C),D));", "(P,((Q,R),S));"])]
  #[case::one_shared_leaf(["(A,((B,C),D));", "(A,((Q,R),S));"])]
  #[trace]
  fn pair_sharing_fewer_than_two_leaves_is_skipped(
    #[case] nwks: [&str; 2],
    #[values(Resolution::None, Resolution::Strict, Resolution::Liberal, Resolution::Matched)] resolution: Resolution,
    #[values(false, true)] parallel: bool,
  ) {
    let (mut ts, taxa) = trees(&nwks);
    let before: Vec<_> = ts.iter().map(|t| (t.leaf_names(), splits(t, &taxa))).collect();
    let o = Options {
      resolution,
      parallel,
      pre_resolve: true,
      ..Options::for_trees(2)
    };
    let res = in_pool(parallel, || run(&mut ts, &taxa, &o, 1));
    let after: Vec<_> = ts.iter().map(|t| (t.leaf_names(), splits(t, &taxa))).collect();
    let skipped = res.iter().map(|r| (r.mccs.len(), r.attached.len())).collect::<Vec<_>>();
    assert_eq!((vec![(0, 0)], before), (skipped, after));
  }

  #[test]
  fn skipped_pairs_leave_other_pairs_unaffected() {
    // Tree 2 shares one leaf (Y) with tree 0 and none with tree 1, so the run skips one pair of
    // each kind. The pair (0, 1) gets the MCCs that it gets without tree 2: the same per-pair
    // seed, and no resolution across pairs. Y is in tree 0 only, inside the clade (B,Y) of the
    // MCC {A,B,C,D}, so attachment adds it to that MCC.
    let pair = ["((A,(B,Y)),(C,(D,X)));", "((A,(B,X)),(C,D));"];
    let o = |k| Options {
      resolution: Resolution::None,
      ..Options::for_trees(k)
    };
    let names = |r: &PairResult, t: &Taxa| r.mccs.iter().map(|m| t.names_of(m)).collect::<Vec<_>>();
    let (mut ts2, taxa2) = trees(&pair);
    let expected = run(&mut ts2, &taxa2, &o(2), 1);
    let (mut ts3, taxa3) = trees(&[pair[0], pair[1], "(Y,(P,Q));"]);
    let res = run(&mut ts3, &taxa3, &o(3), 1);
    assert_eq!(
      res.iter().map(|r| (r.i, r.j)).collect::<Vec<_>>(),
      [(0, 1), (0, 2), (1, 2)]
    );
    assert_eq!(names(&res[0], &taxa3), names(&expected[0], &taxa2));
    assert_eq!(names(&res[0], &taxa3), [vec!["X"], vec!["A", "B", "C", "D", "Y"]]);
    assert!(res[1].mccs.is_empty());
    assert!(res[2].mccs.is_empty());
  }

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

  /// The two-tree example: X moved between the trees.
  const TWO_TREE_EXAMPLE: [&str; 2] = ["((A,B),(C,(D,X)));", "((A,(B,X)),(C,D));"];

  /// A polytomy in the first tree.
  const POLYTOMY_PAIR: [&str; 2] = ["((A,B,C),(D,X),E);", "((A,(B,X)),(C,D),E);"];

  /// Leaves in one tree only: P in the first tree, Q in the second.
  const ONE_TREE_LEAVES: [&str; 2] = ["((A,B),((C,P),(D,X)));", "((A,(B,X)),(C,(D,Q)));"];

  /// The two-tree inputs of the display sort.
  const PAIR_INPUTS: [[&str; 2]; 3] = [TWO_TREE_EXAMPLE, POLYTOMY_PAIR, ONE_TREE_LEAVES];

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

  #[test]
  fn shared_mccs_drop_the_leaves_of_one_tree_only() {
    let (mut ts, taxa) = trees(&ONE_TREE_LEAVES);
    let o = Options {
      resolution: Resolution::None,
      ..Options::for_trees(2)
    };
    let res = run(&mut ts, &taxa, &o, 1);
    assert_eq!(2, res[0].attached.len());
    let shared = res[0].shared_mccs(&ts, taxa.len());
    assert_eq!(ids(&taxa, &[&["X"], &["A", "B", "C", "D"]]), shared);
    let attached: Vec<usize> = res[0].mccs.iter().map(Vec::len).collect();
    assert_eq!(vec![1, 6], attached);
  }

  /// Number in the log line of `lines` that starts with `before` and has `after` right after
  /// the number.
  fn logged_count(lines: &[String], before: &str, after: &str) -> Vec<usize> {
    lines
      .iter()
      .filter_map(|l| l.strip_prefix(before)?.strip_suffix(after)?.parse().ok())
      .collect()
  }

  #[test]
  fn shared_mccs_are_the_parts_after_matching_splits_an_mcc() {
    // With seed 1, matching finds conflicting topologies inside an MCC of this run and splits
    // it, and the run sorts the pair by the parts. The run's log gives the expected number of
    // MCCs per pair: the MCCs it inferred, plus one per part beyond the first of each split.
    let (mut ts, taxa) = trees(&["((D,A),(B,E),C);", "(E,(B,C),(D,A));", "((B,C),D,(E,A));"]);
    for (t, label) in ts.iter_mut().zip(["t0", "t1", "t2"]) {
      label.clone_into(&mut t.label);
    }
    let n = taxa.len();
    let o = Options {
      n_t: 10,
      ..Options::for_trees(3)
    };
    assert_eq!(Resolution::Matched, o.resolution);
    let mut res = Vec::new();
    let lines = logged(|| res = run(&mut ts, &taxa, &o, 1));
    let counts = |r: &PairResult| {
      let pair = format!("{} and {}", ts[r.i].label, ts[r.j].label);
      let inferred = logged_count(&lines, "INFO found ", &format!(" MCCs for {pair}"));
      let conflict = format!("conflicting topologies in {pair}: split into ");
      let parts: Vec<usize> = lines
        .iter()
        .filter_map(|l| l.split_once(&conflict)?.1.strip_suffix(" MCCs")?.parse().ok())
        .collect();
      (inferred, parts)
    };
    let logged_counts: Vec<_> = res.iter().map(counts).collect();
    assert!(
      logged_counts.iter().any(|(_, parts)| !parts.is_empty()),
      "no MCC was split: {lines:?}"
    );
    let expected: Vec<usize> = logged_counts
      .iter()
      .map(|(inferred, parts)| inferred.iter().sum::<usize>() + parts.iter().map(|p| p - 1).sum::<usize>())
      .collect();
    let shared: Vec<Vec<Mcc>> = res.iter().map(|r| r.shared_mccs(&ts, n)).collect();
    assert_eq!(expected, shared.iter().map(Vec::len).collect::<Vec<_>>());
    // The MCC that matching split still has conflicting topologies in the final trees, so only
    // its parts agree within every MCC.
    let conflicting: Vec<(usize, usize, Mcc)> = res
      .iter()
      .zip(&shared)
      .flat_map(|(r, ms)| {
        ms.iter()
          .filter(|m| mismatched_restrictions(&ts[r.i], &ts[r.j], m, n).is_some())
          .map(|m| (r.i, r.j, m.clone()))
      })
      .collect();
    assert_eq!(Vec::<(usize, usize, Mcc)>::new(), conflicting);
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

  /// Log lines, each with the thread that logged it.
  type Records = Vec<(std::thread::ThreadId, String)>;

  /// Logger that keeps every record, with the thread that logged it, since other tests may log
  /// from other threads of the same process.
  struct Capture(std::sync::Mutex<Records>);

  impl log::Log for Capture {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
      true
    }

    fn log(&self, record: &log::Record<'_>) {
      let line = format!("{} {}", record.level(), record.args());
      self.0.lock().unwrap().push((std::thread::current().id(), line));
    }

    fn flush(&self) {}
  }

  static CAPTURE: Capture = Capture(std::sync::Mutex::new(Vec::new()));

  /// Lines that `f` logs on this thread.
  fn logged(f: impl FnOnce()) -> Vec<String> {
    static INSTALL: std::sync::Once = std::sync::Once::new();
    INSTALL.call_once(|| {
      log::set_logger(&CAPTURE).unwrap();
      log::set_max_level(log::LevelFilter::Trace);
    });
    let me = std::thread::current().id();
    CAPTURE.0.lock().unwrap().retain(|(t, _)| *t != me);
    f();
    let records = CAPTURE.0.lock().unwrap();
    records
      .iter()
      .filter(|(t, _)| *t == me)
      .map(|(_, l)| l.clone())
      .collect()
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

  /// `k` trees on the same leaves.
  fn same_leaves(k: usize) -> (Vec<Tree>, Taxa) {
    trees(&vec!["((A,B),(C,D));"; k])
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
  #[should_panic(expected = "(0, 2) is not a pair i < j of the 2 trees")]
  fn shared_mccs_reject_a_pair_out_of_range() {
    let (ts, taxa) = same_leaves(2);
    let pair = PairResult {
      i: 0,
      j: 2,
      mccs: Vec::new(),
      attached: Vec::new(),
    };
    pair.shared_mccs(&ts, taxa.len());
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

  #[test]
  #[should_panic(expected = "a tree of the pair has a leaf that is not one of the 3 taxa")]
  fn shared_mccs_reject_a_tree_leaf_out_of_range() {
    let (ts, _) = same_leaves(2);
    let pair = PairResult {
      i: 0,
      j: 1,
      mccs: Vec::new(),
      attached: Vec::new(),
    };
    pair.shared_mccs(&ts, 3);
  }

  /// Progress events of a run of `nwks` with `opts` and seed 1.
  fn observed(nwks: &[&str], opts: &Options) -> Vec<Progress> {
    let (mut ts, taxa) = trees(nwks);
    let events = std::cell::RefCell::new(Vec::new());
    run_observed(&mut ts, &taxa, opts, 1, &|p| events.borrow_mut().push(p));
    events.into_inner()
  }

  fn never_decreasing(events: &[Progress]) -> bool {
    events
      .iter()
      .zip(events.iter().skip(1))
      .all(|(a, b)| a.fraction <= b.fraction)
  }

  /// Ten leaves whose MCC inference with seed 1 needs two prune iterations: B, D, G, and I
  /// swap places between the two trees.
  const TWO_ITERATIONS: [&str; 2] = [
    "(((((A,B),C),D),E),((((F,G),H),I),J));",
    "(((((A,G),C),I),E),((((F,B),H),D),J));",
  ];

  /// Options for one pair with `itmax` and ten temperatures, so that a test run stays short.
  fn one_pair(itmax: usize) -> Options {
    Options {
      itmax,
      n_t: 10,
      resolution: Resolution::Strict,
      parallel: false,
      ..Options::for_trees(2)
    }
  }

  /// Number of iterations that MCC inference completed in a run of one pair with `itmax`,
  /// counted from its progress events: the last temperature step of iteration `k` reports the
  /// completed fraction `k / (itmax + 1)` of the pair, and no other step reports a multiple of
  /// `1 / (itmax + 1)`.
  fn iterations(events: &[Progress], itmax: usize) -> usize {
    let max = itmax + 1;
    (1..=max)
      .filter(|&k| {
        let end = Progress::at(0, 1, 0, 1, ratio(k, max));
        events
          .iter()
          .any(|p| p.phase == Phase::Pairs && p.fraction.to_bits() == end.fraction.to_bits())
      })
      .count()
  }

  #[test]
  fn progress_of_a_pair_that_runs_all_iterations_rises_to_the_pairs_share() {
    // With itmax = 1, pair inference runs both of its itmax + 1 = 2 iterations.
    let events = observed(&TWO_ITERATIONS, &one_pair(1));
    assert_eq!(2, iterations(&events, 1), "{events:?}");
    assert_eq!(Some(&Progress::at(0, 1, 0, 1, 0.0)), events.first());
    assert_eq!(Some(&Progress::done(1, 1)), events.last());
    assert!(never_decreasing(&events), "{events:?}");
    // With one round and one pair, the fraction is the pairs' share of the completed fraction of
    // the pair: the last temperature step of iteration 1 completes 1/2, that of iteration 2
    // completes all of it.
    let before_end = &events[..events.len() - 1];
    assert!(before_end.iter().all(|p| p.fraction <= PAIRS_SHARE), "{events:?}");
    let half = PAIRS_SHARE * 0.5;
    assert!(
      before_end.iter().any(|p| p.fraction.to_bits() == half.to_bits()),
      "{events:?}"
    );
    assert_eq!(Some(PAIRS_SHARE), before_end.last().map(|p| p.fraction), "{events:?}");
  }

  #[test]
  fn progress_of_a_pair_that_stops_early_jumps_to_done_at_the_end() {
    // With itmax = 2 the same pair stops by itself after two of its three iterations, at 2/3.
    let events = observed(&TWO_ITERATIONS, &one_pair(2));
    assert_eq!(2, iterations(&events, 2), "{events:?}");
    let before_end = &events[..events.len() - 1];
    assert!(never_decreasing(&events), "{events:?}");
    assert_eq!(
      Some(PAIRS_SHARE * (2.0 / 3.0)),
      before_end.last().map(|p| p.fraction),
      "{events:?}"
    );
    assert_eq!(Some(&Progress::done(1, 1)), events.last());
  }

  #[test]
  fn progress_of_sequential_rounds_visits_every_pair_in_order() {
    // Strict resolution with three trees adds a final round without resolution: two rounds.
    let o = Options {
      itmax: 2,
      n_t: 5,
      resolution: Resolution::Strict,
      parallel: false,
      ..Options::for_trees(3)
    };
    let events = observed(&["((A,B),(C,(D,X)));", "((A,(B,X)),(C,D));", "((A,X),(B,(C,D)));"], &o);
    let mut visited: Vec<(usize, usize, usize, usize)> =
      events.iter().map(|p| (p.round, p.rounds, p.pair, p.pairs)).collect();
    visited.dedup();
    let expected = [
      (1, 2, 1, 3),
      (1, 2, 2, 3),
      (1, 2, 3, 3),
      (2, 2, 1, 3),
      (2, 2, 2, 3),
      (2, 2, 3, 3),
    ];
    assert_eq!(expected.as_slice(), visited);
    assert!(never_decreasing(&events), "{events:?}");
    let before_end = &events[..events.len() - 1];
    assert!(before_end.iter().all(|p| p.fraction <= PAIRS_SHARE), "{events:?}");
    assert_eq!(Some(&Progress::done(2, 3)), events.last());
  }

  #[test]
  fn progress_of_parallel_rounds_is_reported_before_and_after_each_batch() {
    let o = Options {
      rounds: 2,
      resolution: Resolution::None,
      parallel: true,
      ..Options::for_trees(3)
    };
    let nwks = ["((A,B),(C,(D,X)));", "((A,(B,X)),(C,D));", "((A,X),(B,(C,D)));"];
    let events = in_pool(true, || observed(&nwks, &o));
    // Fractions PAIRS_SHARE * (round + (pair + within) / 3) / 2: 0, (0 + 3 / 3) / 2 = 0.5,
    // (1 + 0) / 2 = 0.5, (1 + 3 / 3) / 2 = 1 of the share, and 1 at the end.
    let expected = [
      Progress {
        phase: Phase::Pairs,
        fraction: 0.0,
        round: 1,
        rounds: 2,
        pair: 1,
        pairs: 3,
      },
      Progress {
        phase: Phase::Pairs,
        fraction: PAIRS_SHARE * 0.5,
        round: 1,
        rounds: 2,
        pair: 3,
        pairs: 3,
      },
      Progress {
        phase: Phase::Pairs,
        fraction: PAIRS_SHARE * 0.5,
        round: 2,
        rounds: 2,
        pair: 1,
        pairs: 3,
      },
      Progress {
        phase: Phase::Pairs,
        fraction: PAIRS_SHARE,
        round: 2,
        rounds: 2,
        pair: 3,
        pairs: 3,
      },
      Progress {
        phase: Phase::Done,
        fraction: 1.0,
        round: 2,
        rounds: 2,
        pair: 3,
        pairs: 3,
      },
    ];
    assert_eq!(expected.as_slice(), events);
  }

  /// Three trees with polytomies: pre-resolution adds the split (C,D), which the other two trees
  /// have, to the first tree, and the placements of X differ, so the pairs have several MCCs.
  const POLYTOMIES: [&str; 3] = ["((A,B),(C,D,X));", "((A,(B,X)),(C,D));", "((A,X),B,(C,D));"];

  #[test]
  fn pre_resolution_changes_the_trees_of_the_progress_bounds() {
    // Oracle: (C,D) is a split of the second and third tree and fits the polytomy (C,D,X) of the
    // first; no other split is compatible with all three trees and missing from one.
    let (mut ts, taxa) = trees(&POLYTOMIES);
    let new = resolve_trees(&mut ts, taxa.len());
    let added: Vec<usize> = new.iter().map(Vec::len).collect();
    assert_eq!(vec![1, 0, 0], added);
  }

  /// A run of `POLYTOMIES` in `resolution`, `parallel`, and `pre_resolve` reports
  /// fractions that never decrease and stay at most `PAIRS_SHARE` before the last event, which is
  /// `Done` with fraction 1, and the phases `Pairs`, then `Matching` for `Matched` resolution, then
  /// `Done`.
  ///
  /// Matched resolution resolves in every round, so its rounds run sequentially with or without
  /// `parallel` and have no parallel cases.
  #[rustfmt::skip]
  #[rstest]
  #[case::none_sequential(                (Resolution::None,    false, false))]
  #[case::none_sequential_pre_resolved(   (Resolution::None,    false, true))]
  #[case::none_parallel(                  (Resolution::None,    true,  false))]
  #[case::none_parallel_pre_resolved(     (Resolution::None,    true,  true))]
  #[case::strict_sequential(              (Resolution::Strict,  false, false))]
  #[case::strict_sequential_pre_resolved( (Resolution::Strict,  false, true))]
  #[case::strict_parallel(                (Resolution::Strict,  true,  false))]
  #[case::strict_parallel_pre_resolved(   (Resolution::Strict,  true,  true))]
  #[case::liberal_sequential(             (Resolution::Liberal, false, false))]
  #[case::liberal_sequential_pre_resolved((Resolution::Liberal, false, true))]
  #[case::liberal_parallel(               (Resolution::Liberal, true,  false))]
  #[case::liberal_parallel_pre_resolved(  (Resolution::Liberal, true,  true))]
  #[case::matched(                        (Resolution::Matched, false, false))]
  #[case::matched_pre_resolved(           (Resolution::Matched, false, true))]
  #[trace]
  fn progress_bounds(#[case] (resolution, parallel, pre_resolve): (Resolution, bool, bool)) {
    let o = Options {
      itmax: 2,
      n_t: 5,
      resolution,
      parallel,
      pre_resolve,
      ..Options::for_trees(3)
    };
    let events = in_pool(parallel, || observed(&POLYTOMIES, &o));
    assert!(never_decreasing(&events), "{events:?}");
    let (last, before_end) = events.split_last().unwrap();
    assert!(
      before_end
        .iter()
        .all(|p| p.phase != Phase::Done && p.fraction <= PAIRS_SHARE),
      "{events:?}"
    );
    assert_eq!((Phase::Done, 1.0_f64.to_bits()), (last.phase, last.fraction.to_bits()));
    let mut phases: Vec<Phase> = events.iter().map(|p| p.phase).collect();
    phases.dedup();
    let expected = if resolution == Resolution::Matched {
      vec![Phase::Pairs, Phase::Matching, Phase::Done]
    } else {
      vec![Phase::Pairs, Phase::Done]
    };
    assert_eq!(expected, phases, "{events:?}");
  }

  /// Events of a run with `Matched` resolution (whose rounds resolve, so they run sequentially):
  /// the pair events, the start of matching, and the end.
  fn matched_events() -> (Vec<Progress>, Progress, Progress) {
    let o = Options {
      n_t: 5,
      ..Options::for_trees(3)
    };
    assert_eq!(Resolution::Matched, o.resolution);
    let mut events = observed(&["((A,B),(C,(D,X)));", "((A,(B,X)),(C,D));", "((A,X),(B,(C,D)));"], &o);
    let done = events.pop().unwrap();
    let matching = events.pop().unwrap();
    (events, matching, done)
  }

  #[test]
  fn progress_of_matched_resolution_reports_matching_at_the_fraction_reached() {
    let (pairs, matching, done) = matched_events();
    assert!(pairs.iter().all(|p| p.phase == Phase::Pairs), "{pairs:?}");
    assert!(never_decreasing(&pairs), "{pairs:?}");
    let reached = pairs.last().unwrap().fraction;
    assert_eq!(Progress::matching(reached, 1, 3), matching);
    assert_eq!(Progress::done(1, 3), done);
  }
}
