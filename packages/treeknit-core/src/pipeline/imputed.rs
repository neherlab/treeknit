//! Leaves missing from one tree of a pair: their attachment to the MCCs, the imputed trees, and
//! the trees of the ARG.

#![expect(
  clippy::expect_used,
  clippy::unwrap_used,
  reason = "findings from before the strict lint set; kb/issues/N-lint-baseline.md tracks their removal"
)]

use crate::bits::Bits;
use crate::impute::{Attachment, attach_private, graft_attachments};
use crate::naive::{Mcc, sort_mccs};
use crate::pipeline::PairResult;
use crate::tree::Tree;

/// Attach leaves of either tree that the other lacks, and extend the MCCs accordingly.
pub(crate) fn attach_pair(trees: &[Tree], i: usize, j: usize, mut mccs: Vec<Mcc>, n: usize) -> PairResult {
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::options::Options;
  use crate::pipeline::run;
  use crate::tree::Taxa;
  use crate::tree::test_util::{splits, trees};
  use pretty_assertions::assert_eq;

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
