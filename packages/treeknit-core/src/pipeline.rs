//! TreeKnit on K ≥ 2 trees: pre-resolution, rounds of pair inference, polytomy sorting,
//! and placement of leaves missing from one tree of a pair.

use crate::bits::{self, Bits};
use crate::impute::{Attachment, attach_private, graft_attachments};
use crate::mcc_map::{leaf_order, sort_by_leaf_order, sort_polytomies_by_mccs};
use crate::naive::{Mcc, naive_mccs, sort_mccs};
use crate::options::{Options, Resolution};
use crate::pair::{PairParams, infer_pair};
use crate::resolve::{Insert, insert_all_on, insert_split, resolve_trees, resolve_with_mccs};
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
}

/// Run TreeKnit on `trees` (leaves must have taxa assigned from `taxa`).
/// Trees are resolved and sorted in place. Pairs are returned in order (0,1), (0,2), …
pub fn run(trees: &mut [Tree], taxa: &Taxa, opts: &Options, seed: u64) -> Vec<PairResult> {
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
  let pairs: Vec<(usize, usize)> = (0..k).flat_map(|i| (i + 1..k).map(move |j| (i, j))).collect();
  let mut mccs: Vec<Vec<Mcc>> = vec![Vec::new(); pairs.len()];
  let matched = opts.resolution == Resolution::Matched;
  let extra_round =
    matches!(opts.resolution, Resolution::Strict | Resolution::Liberal) && k > 2 && opts.final_unresolved_round;
  let rounds = opts.rounds + extra_round as usize;
  for round in 1..=rounds {
    let last = round == rounds;
    let unresolved_round = extra_round && last;
    let resolve = opts.resolves() && !unresolved_round;
    // Splits added with MCCs: unambiguous ones only, except in liberal mode.
    let strict = opts.resolution != Resolution::Liberal && resolve;
    log::info!("round {round}/{rounds}{}", if resolve { " (resolving)" } else { "" });
    if resolve || !opts.parallel {
      for (p, &(i, j)) in pairs.iter().enumerate() {
        mccs[p] = infer(trees, i, j, n, opts, resolve, seed, round);
        if resolve {
          resolve_pair(trees, i, j, &mccs[p], n, strict);
        }
        if last && !matched {
          sort_pair(trees, i, j, &mccs[p], n, opts.sort_strict.unwrap_or(strict));
        }
      }
    } else {
      let shared: &[Tree] = trees;
      mccs = pairs
        .par_iter()
        .map(|&(i, j)| infer(shared, i, j, n, opts, false, seed, round))
        .collect();
      if last && !matched {
        for (p, &(i, j)) in pairs.iter().enumerate() {
          sort_pair(trees, i, j, &mccs[p], n, opts.sort_strict.unwrap_or(strict));
        }
      }
    }
  }
  if matched {
    match_topologies(trees, &pairs, &mut mccs, n);
    for (p, &(i, j)) in pairs.iter().enumerate() {
      sort_pair(trees, i, j, &mccs[p], n, false);
    }
  }
  pairs
    .iter()
    .zip(mccs)
    .map(|(&(i, j), m)| attach_pair(trees, i, j, m, n))
    .collect()
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
  let (s, d) = if src < dst {
    let (a, b) = trees.split_at_mut(dst);
    (&a[src], &mut b[0])
  } else {
    let (a, b) = trees.split_at_mut(src);
    (&b[0], &mut a[dst])
  };
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
      let split = bits::and(&clades[v], &mask);
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
    let shared = bits::and(&trees[r.i].leaf_set(n), &trees[r.j].leaf_set(n));
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
pub fn internal_clades(t: &Tree, n: usize) -> std::collections::HashSet<Bits> {
  let c = t.clades(n);
  t.internals()
    .into_iter()
    .filter(|&v| v != t.root)
    .map(|v| c[v].clone())
    .collect()
}

/// Leaves common to trees `i` and `j`, and both trees restricted to them (borrowed if no
/// restriction is needed).
fn restrict_pair<'a>(trees: &'a [Tree], i: usize, j: usize, n: usize) -> (Bits, Cow<'a, Tree>, Cow<'a, Tree>) {
  let (li, lj) = (trees[i].leaf_set(n), trees[j].leaf_set(n));
  let shared = bits::and(&li, &lj);
  let r = |t: &'a Tree, l: &Bits| {
    if *l == shared {
      Cow::Borrowed(t)
    } else {
      Cow::Owned(t.restricted(&shared).expect("no shared leaves"))
    }
  };
  let (ri, rj) = (r(&trees[i], &li), r(&trees[j], &lj));
  (shared, ri, rj)
}

#[allow(clippy::too_many_arguments)]
fn infer(
  trees: &[Tree],
  i: usize,
  j: usize,
  n: usize,
  opts: &Options,
  resolve: bool,
  seed: u64,
  round: usize,
) -> Vec<Mcc> {
  let (shared, ti, tj) = restrict_pair(trees, i, j, n);
  let n_shared = shared.count_ones(..);
  log::info!(
    "inferring MCCs for {} and {} ({n_shared} shared leaves)",
    trees[i].label,
    trees[j].label
  );
  if n_shared < 2 {
    log::warn!(
      "trees {} and {} share fewer than two leaves: skipped",
      trees[i].label,
      trees[j].label
    );
    return if n_shared == 1 {
      vec![shared.ones().collect()]
    } else {
      vec![]
    };
  }
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
    infer_pair(&ti, &tj, n, &p, &mut rng)
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

/// Resolve trees `i` and `j` with their MCCs (computed on shared leaves).
/// Returns the number of splits added to the two trees.
fn resolve_pair(trees: &mut [Tree], i: usize, j: usize, mccs: &[Mcc], n: usize, strict: bool) -> usize {
  let (shared, ti, tj) = restrict_pair(trees, i, j, n);
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

/// Ladderize the first tree and order polytomies so that MCCs face each other.
fn sort_pair(trees: &mut [Tree], i: usize, j: usize, mccs: &[Mcc], n: usize, strict: bool) {
  if i == 0 {
    trees[0].ladderize();
  }
  let (_, ti, tj) = restrict_pair(trees, i, j, n);
  let full = matches!((&ti, &tj), (Cow::Borrowed(_), Cow::Borrowed(_)));
  let (mut ti, mut tj) = (ti.into_owned(), tj.into_owned());
  if strict {
    // Order leaves using liberally resolved copies, then apply that order.
    resolve_with_mccs(&mut ti, &mut tj, mccs, n, false);
    ti.ladderize();
    sort_polytomies_by_mccs(&ti, &mut tj, mccs, n);
    let (oi, oj) = (leaf_order(&ti), leaf_order(&tj));
    sort_by_leaf_order(&mut trees[i], &oi);
    sort_by_leaf_order(&mut trees[j], &oj);
  } else if full {
    let ti = trees[i].clone();
    sort_polytomies_by_mccs(&ti, &mut trees[j], mccs, n);
  } else {
    sort_polytomies_by_mccs(&ti, &mut tj, mccs, n);
    sort_by_leaf_order(&mut trees[j], &leaf_order(&tj));
  }
}

/// Attach leaves of either tree that the other lacks, and extend the MCCs accordingly.
fn attach_pair(trees: &[Tree], i: usize, j: usize, mut mccs: Vec<Mcc>, n: usize) -> PairResult {
  let shared = bits::and(&trees[i].leaf_set(n), &trees[j].leaf_set(n));
  let mut attached = Vec::new();
  if !mccs.is_empty() {
    attached.extend(attach_private(&trees[i], i, &shared, &mccs, n));
    attached.extend(attach_private(&trees[j], j, &shared, &mccs, n));
  }
  for a in &attached {
    mccs[a.mcc].extend(&a.leaves);
  }
  let base = mccs.clone();
  let mccs = sort_mccs(mccs);
  // Re-index attachments to the sorted MCC list.
  for a in &mut attached {
    let first = base[a.mcc].iter().min().copied().unwrap();
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
  let mut keep = bits::and(&trees[pair.i].leaf_set(n), &trees[pair.j].leaf_set(n));
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
  use crate::tree::test_util::{splits, trees};

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
    assert_eq!(res.len(), 3);
    for r in &res {
      let total: usize = r.mccs.iter().map(|m| m.len()).sum();
      assert_eq!(total, 5);
    }
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
}
