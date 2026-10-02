//! TreeKnit on K ≥ 2 trees: pre-resolution, rounds of pair inference, polytomy sorting,
//! and placement of leaves missing from one tree of a pair.

use crate::bits::{self, Bits};
use crate::impute::{attach_private, graft_attachments, Attachment};
use crate::mcc_map::{leaf_order, sort_by_leaf_order, sort_polytomies_by_mccs};
use crate::naive::{naive_mccs, sort_mccs, Mcc};
use crate::options::Options;
use crate::pair::{infer_pair, PairParams};
use crate::resolve::{insert_all_on, resolve_trees, resolve_with_mccs};
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
    for round in 1..=opts.rounds {
        let last = round == opts.rounds;
        let resolve = opts.resolve && !(opts.final_no_resolve && last);
        let strict = opts.strict && !(opts.final_no_resolve && last);
        log::info!(
            "round {round}/{}{}",
            opts.rounds,
            if resolve { " (resolving)" } else { "" }
        );
        if resolve || !opts.parallel {
            for (p, &(i, j)) in pairs.iter().enumerate() {
                mccs[p] = infer(trees, i, j, n, opts, resolve, seed, round);
                if resolve {
                    resolve_pair(trees, i, j, &mccs[p], n, strict);
                }
                if last {
                    sort_pair(trees, i, j, &mccs[p], n, strict);
                }
            }
        } else {
            let shared: &[Tree] = trees;
            mccs = pairs
                .par_iter()
                .map(|&(i, j)| infer(shared, i, j, n, opts, false, seed, round))
                .collect();
            if last {
                for (p, &(i, j)) in pairs.iter().enumerate() {
                    sort_pair(trees, i, j, &mccs[p], n, strict);
                }
            }
        }
    }
    pairs
        .iter()
        .zip(mccs)
        .map(|(&(i, j), m)| attach_pair(trees, i, j, m, n))
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
    let mut z = seed ^ ((round as u64) << 40 | (i as u64) << 20 | j as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Resolve trees `i` and `j` with their MCCs (computed on shared leaves).
fn resolve_pair(trees: &mut [Tree], i: usize, j: usize, mccs: &[Mcc], n: usize, strict: bool) {
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
    for a in attached.iter_mut() {
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

    #[test]
    fn three_trees_better_trees() {
        let (mut ts, taxa) = trees(&["((A,(B,C)),(D,E));", "((A,B,C,D),E);", "((A,B),((C,D),E));"]);
        let o = Options::for_trees(3, None);
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
        let o = Options::for_trees(2, None);
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
        assert!(splits(&imp[0], &tb).contains(&vec!["D".to_string(), "P".to_string()]));
    }
}
