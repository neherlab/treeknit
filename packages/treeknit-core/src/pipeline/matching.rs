//! Matching the topologies of the trees of a pair within their MCCs.

#![expect(
  clippy::disallowed_types,
  clippy::expect_used,
  reason = "findings from before the strict lint set; kb/issues/N-lint-baseline.md tracks their removal"
)]

use crate::bits::{self, Bits};
use crate::naive::{Mcc, naive_mccs, sort_mccs};
use crate::pipeline::PairResult;
use crate::resolve::{Insert, insert_split};
use crate::tree::Tree;

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
pub(crate) fn mismatched_restrictions(t1: &Tree, t2: &Tree, m: &Mcc, n: usize) -> Option<(Tree, Tree)> {
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::pipeline::tests::ids;
  use crate::tree::test_util::{splits, trees};
  use pretty_assertions::assert_eq;

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
}
