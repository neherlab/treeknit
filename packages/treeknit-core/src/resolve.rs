//! Resolving polytomies: split exchange between trees, and resolution using MCCs.

use crate::bits::{self, Bits};
use crate::mcc_map::{leaf_mcc_map, map_mccs};
use crate::naive::Mcc;
use crate::tree::{NodeId, Tree};

/// Outcome of [`insert_split`].
#[derive(Debug, PartialEq, Eq)]
pub enum Insert {
    Added,
    /// Already present, or trivial on the mask.
    Present,
    Incompatible,
}

/// Children of `lca(s)` whose clade (restricted to `mask`) intersects `s`, in order of
/// first appearance when scanning leaves of `s` by taxon id. Returns the lca too.
fn blca(t: &Tree, leaf_of: &[Option<NodeId>], s: &Bits, mask: &Bits) -> Option<(NodeId, Vec<NodeId>)> {
    let leaves: Vec<NodeId> = s
        .ones()
        .filter(|&x| mask.contains(x))
        .filter_map(|x| leaf_of[x])
        .collect();
    let r = t.lca_of(leaves.iter().copied())?;
    let mut roots: Vec<NodeId> = Vec::new();
    for &l in &leaves {
        let mut a = l;
        while a != r && t.parent(a) != Some(r) {
            a = t.parent(a).unwrap();
        }
        if a != r && !roots.contains(&a) {
            roots.push(a);
        }
    }
    Some((r, roots))
}

/// Insert split `s` into `t`, considering only leaves in `mask`.
///
/// Subtrees containing no leaf of `mask` are never moved.
pub fn insert_split(t: &mut Tree, s: &Bits, mask: &Bits, n_taxa: usize, name: &str) -> Insert {
    if bits::trivial_on(s, mask) {
        return Insert::Present;
    }
    let clades = t.clades(n_taxa);
    let leaf_of = t.leaf_of(n_taxa);
    let Some((r, roots)) = blca(t, &leaf_of, s, mask) else {
        return Insert::Incompatible;
    };
    if roots.iter().any(|&c| !bits::subset_on(&clades[c], s, mask)) {
        return Insert::Incompatible;
    }
    if bits::subset_on(&clades[r], s, mask) {
        return Insert::Present; // `s` is the clade of `r`
    }
    let _ = r;
    t.insert_parent(&roots, name.to_string(), Some(0.0));
    Insert::Added
}

/// Resolve `trees` using each other's splits. A split of one tree is added to another only
/// if it can be added to (or is already in) every other tree. Splits are compared on
/// the leaves the trees have in common, so leaf sets may differ.
///
/// Returns the new splits of each tree.
pub fn resolve_trees(trees: &mut [Tree], n_taxa: usize) -> Vec<Vec<Bits>> {
    let k = trees.len();
    let leafsets: Vec<Bits> = trees.iter().map(|t| t.leaf_set(n_taxa)).collect();
    let clades: Vec<Vec<Bits>> = trees.iter().map(|t| t.clades(n_taxa)).collect();
    let leaf_of: Vec<Vec<Option<NodeId>>> = trees.iter().map(|t| t.leaf_of(n_taxa)).collect();
    // Split lists: internal clades in postorder, extended by newly accepted splits.
    let mut splits: Vec<Vec<Bits>> = trees
        .iter()
        .zip(&clades)
        .map(|(t, c)| t.internals().into_iter().map(|n| c[n].clone()).collect())
        .collect();
    // New splits with the mask (leaves shared with the source tree) they were judged on.
    let mut new: Vec<Vec<(Bits, Bits)>> = vec![Vec::new(); k];

    let mut changed = true;
    let mut nit = 0;
    while changed && nit < 20 {
        changed = false;
        for src in 0..k {
            let src_splits = splits[src].clone();
            for sk in &src_splits {
                let mut ok = 0;
                let mut accept: Vec<(usize, Bits, Bits)> = Vec::new();
                for i in (0..k).filter(|&i| i != src) {
                    let s = bits::and(sk, &leafsets[i]);
                    if bits::trivial_on(&s, &leafsets[i]) {
                        ok += 1;
                        continue;
                    }
                    let common = &leafsets[src];
                    let t = &trees[i];
                    let leaves = s.ones().filter_map(|x| leaf_of[i][x]);
                    let ri = t.lca_of(leaves).unwrap();
                    if bits::eq_on(&clades[i][ri], &s, common) {
                        ok += 1;
                    } else if !new[i].iter().any(|(x, _)| *x == s) {
                        // `s` can be added iff it is a union of children of `ri`.
                        let mut joined = Bits::with_capacity(n_taxa);
                        for &c in t.children(ri) {
                            if bits::subset_on(&clades[i][c], &s, common) {
                                joined.union_with(&bits::and(&clades[i][c], common));
                            }
                        }
                        if joined == bits::and(&s, common) {
                            ok += 1;
                            accept.push((i, s, bits::and(common, &leafsets[i])));
                        } else {
                            break;
                        }
                    }
                }
                if ok == k - 1 {
                    for (i, s, mask) in accept {
                        splits[i].push(s.clone());
                        new[i].push((s, mask));
                        changed = true;
                    }
                }
            }
        }
        nit += 1;
    }
    if nit == 20 {
        log::warn!("resolve_trees: maximum number of iterations reached");
    }

    let mut out = Vec::with_capacity(k);
    for (i, t) in trees.iter_mut().enumerate() {
        let mut label = t.fresh_index("RESOLVED");
        new[i].retain(
            |(s, mask)| match insert_split(t, s, mask, n_taxa, &format!("RESOLVED_{label}")) {
                Insert::Added => {
                    label += 1;
                    true
                }
                Insert::Present => false,
                Insert::Incompatible => {
                    log::warn!("resolve_trees: skipping incompatible split in tree {}", t.label);
                    false
                }
            },
        );
        out.push(std::mem::take(&mut new[i]).into_iter().map(|(s, _)| s).collect());
    }
    out
}

/// New splits of `tref` implied by the shared topology of `t` within each MCC.
/// Both trees must have the same leaf set.
fn mcc_splits(tref: &Tree, t: &Tree, mccs: &[Mcc], n_taxa: usize, strict: bool) -> Vec<Bits> {
    let c_ref = tref.clades(n_taxa);
    let c_t = t.clades(n_taxa);
    let leaf_ref = tref.leaf_of(n_taxa);
    let leaf_t = t.leaf_of(n_taxa);
    let leaf_mcc = leaf_mcc_map(mccs, n_taxa);
    let map_ref = strict.then(|| map_mccs(tref, &leaf_mcc));
    let ref_splits: Vec<&Bits> = tref.internals().into_iter().map(|n| &c_ref[n]).collect();
    let all = bits::full(n_taxa);
    let mcc_sets: Vec<Bits> = mccs
        .iter()
        .map(|m| bits::from_iter(n_taxa, m.iter().copied()))
        .collect();

    let mut out: Vec<Bits> = Vec::new();
    for m in mccs {
        let mmask = bits::from_iter(n_taxa, m.iter().copied());
        let r = t.lca_of(m.iter().map(|&x| leaf_t[x].unwrap())).unwrap();
        for v in t.postorder_from(r).into_iter().filter(|&v| !t.is_leaf(v)) {
            let leaves = bits::and(&c_t[v], &mmask);
            if leaves.count_ones(..) < 2 {
                continue;
            }
            let (_, roots) = blca(tref, &leaf_ref, &leaves, &all).unwrap();
            if let Some(map) = &map_ref {
                if !strict_ok(tref, &c_ref, &leaf_ref, map, &mcc_sets, &leaves, &roots) {
                    continue;
                }
            }
            let mut ms = Bits::with_capacity(n_taxa);
            for &x in &roots {
                ms.union_with(&c_ref[x]);
            }
            if ms.is_clear() || ref_splits.iter().any(|s| **s == ms) || out.contains(&ms) {
                continue;
            }
            out.push(ms);
        }
    }
    out
}

/// Strict resolution: a split may only be introduced if the placement of every sister
/// branch in the polytomy is determined by the MCCs.
///
/// A sister is placed outside the new clade with certainty if
/// - it continues the MCC of the polytomy node (node→MCC map), or
/// - it holds leaves of an MCC that also has leaves outside the polytomy: MCC regions are
///   connected, so that MCC passes through the polytomy node and the sister attaches there.
///
/// It belongs inside if it holds leaves of the MCC of the new clade. A sister made only of
/// MCCs lying entirely within the polytomy may or may not be nested in the new clade, and
/// the split is then ambiguous.
fn strict_ok(
    t: &Tree,
    clades: &[Bits],
    leaf_of: &[Option<NodeId>],
    map: &[Option<usize>],
    mcc_sets: &[Bits],
    leaves: &Bits,
    roots: &[NodeId],
) -> bool {
    let all = bits::full(leaves.len());
    let (r, _) = blca(t, leaf_of, leaves, &all).unwrap();
    let leaf_mccs = |n: NodeId| t.leaves_below(n).into_iter().map(|l| map[l]).collect::<Vec<_>>();
    let mut root_mccs: Vec<usize> = Vec::new();
    for m in roots.iter().filter_map(|&x| map[x]) {
        if !root_mccs.contains(&m) {
            root_mccs.push(m);
        }
    }
    // In the Julia implementation, the fallback when no root maps to an MCC yields a set,
    // which is then never found among the MCCs of a sister: the second condition is false.
    let target = root_mccs.first().copied();
    for &s in t.children(r).iter().filter(|c| !roots.contains(c)) {
        let shares_parent = map[s].is_some() && map[t.parent(s).unwrap()] == map[s];
        let mccs_of_s = leaf_mccs(s);
        let contains_target = target.is_some_and(|m| mccs_of_s.contains(&Some(m)));
        let continues_above = mccs_of_s.iter().flatten().any(|&m| !mcc_sets[m].is_subset(&clades[r]));
        if !(shares_parent || contains_target || continues_above) {
            return false;
        }
    }
    true
}

/// Resolve `t1` and `t2` (same leaf set) using MCCs; returns the new splits of each.
pub fn resolve_with_mccs(t1: &mut Tree, t2: &mut Tree, mccs: &[Mcc], n_taxa: usize, strict: bool) -> [Vec<Bits>; 2] {
    let s1 = mcc_splits(t1, t2, mccs, n_taxa, strict);
    let s2 = mcc_splits(t2, t1, mccs, n_taxa, strict);
    [insert_all(t1, s1, n_taxa), insert_all(t2, s2, n_taxa)]
}

/// Insert `splits` into `t` (on all its leaves); returns those actually inserted.
pub fn insert_all(t: &mut Tree, mut splits: Vec<Bits>, n_taxa: usize) -> Vec<Bits> {
    insert_all_on(t, &mut splits, &t.leaf_set(n_taxa), n_taxa);
    splits
}

/// Insert `splits` into `t` considering only leaves in `mask`; keeps the inserted ones.
pub fn insert_all_on(t: &mut Tree, splits: &mut Vec<Bits>, mask: &Bits, n_taxa: usize) {
    let mut label = t.fresh_index("RESOLVED");
    splits.retain(
        |s| match insert_split(t, s, mask, n_taxa, &format!("RESOLVED_{label}")) {
            Insert::Added => {
                label += 1;
                true
            }
            Insert::Present => false,
            Insert::Incompatible => {
                log::warn!("skipping split incompatible with tree {}", t.label);
                false
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::test_util::{splits, trees};

    fn names(v: &[Bits], taxa: &crate::tree::Taxa) -> Vec<Vec<String>> {
        let mut x: Vec<Vec<String>> = v
            .iter()
            .map(|s| s.ones().map(|i| taxa.names[i].clone()).collect())
            .collect();
        x.sort();
        x
    }

    #[test]
    fn basic_pair() {
        let (mut ts, taxa) = trees(&["(((A,B),C),D);", "(B,C,(A,D));"]);
        let ns = resolve_trees(&mut ts, taxa.len());
        assert!(ns[0].is_empty() && ns[1].is_empty());

        let (mut ts, taxa) = trees(&["(A,(B,C));", "(A,B,C);"]);
        let ns = resolve_trees(&mut ts, taxa.len());
        assert!(ns[0].is_empty());
        assert_eq!(names(&ns[1], &taxa), vec![vec!["B", "C"]]);
        assert_eq!(splits(&ts[1], &taxa), vec![vec!["B", "C"]]);
    }

    #[test]
    fn three_trees() {
        let (mut ts, taxa) = trees(&["(A,B,C,D);", "(A,(B,C,D));", "(A,B,(C,D));"]);
        let ns = resolve_trees(&mut ts, taxa.len());
        assert_eq!(names(&ns[1], &taxa), vec![vec!["C", "D"]]);
        assert_eq!(names(&ns[2], &taxa), vec![vec!["B", "C", "D"]]);
        assert_eq!(names(&ns[0], &taxa), vec![vec!["B", "C", "D"], vec!["C", "D"]]);

        let (mut ts, taxa) = trees(&["(A,B,C,D);", "(A,(B,C,D));", "((A,B),(C,D));"]);
        let ns = resolve_trees(&mut ts, taxa.len());
        assert_eq!(names(&ns[0], &taxa), vec![vec!["C", "D"]]);
        assert_eq!(names(&ns[1], &taxa), vec![vec!["C", "D"]]);
        assert!(ns[2].is_empty());
    }

    #[test]
    fn four_trees_incompatible() {
        let (mut ts, taxa) = trees(&["(A,(B,C));", "(A,B,C);", "(A,B,C);", "((A,B),C);"]);
        let ns = resolve_trees(&mut ts, taxa.len());
        assert!(ns[1].is_empty() && ns[2].is_empty());
    }

    #[test]
    fn with_mccs_strict_and_liberal() {
        let mk = || trees(&["((A,(B,C)),D);", "(A,B,C,D);"]);
        let (mut ts, taxa) = mk();
        let id = |s: &str| taxa.index[s];
        let mccs = vec![vec![id("D")], vec![id("A"), id("B"), id("C")]];
        let (a, b) = ts.split_at_mut(1);
        let ns = resolve_with_mccs(&mut a[0], &mut b[0], &mccs, taxa.len(), true);
        assert!(ns[1].is_empty());
        let (mut ts, _) = mk();
        let (a, b) = ts.split_at_mut(1);
        let ns = resolve_with_mccs(&mut a[0], &mut b[0], &mccs, taxa.len(), false);
        assert_eq!(names(&ns[1], &taxa), vec![vec!["A", "B", "C"], vec!["B", "C"]]);
    }

    #[test]
    fn strict_resolves_when_sister_mcc_continues_above_polytomy() {
        // Tree 2 has a polytomy holding MCC {A,B,C} and Y, whose MCC {X,Y} has X outside the
        // polytomy. {X,Y} must pass through the polytomy node, so (A,B,C) is a clade. The
        // node→MCC map assigns no MCC to the polytomy here (the root's children disagree).
        let (mut ts, taxa) = trees(&["((((A,B),C),W),(X,Y),V);", "(((A,B,C,Y),W),X,V);"]);
        let id = |s: &str| taxa.index[s];
        let mccs = vec![
            vec![id("V")],
            vec![id("W")],
            vec![id("X"), id("Y")],
            vec![id("A"), id("B"), id("C")],
        ];
        let map = map_mccs(&ts[1], &leaf_mcc_map(&mccs, taxa.len()));
        let p = ts[1].parent(ts[1].leaves()[0]).unwrap();
        assert_eq!(map[p], None);
        let (a, b) = ts.split_at_mut(1);
        let ns = resolve_with_mccs(&mut a[0], &mut b[0], &mccs, taxa.len(), true);
        assert!(names(&ns[1], &taxa).contains(&vec!["A".to_string(), "B".into(), "C".into()]));

        // If Y's MCC lies entirely inside the polytomy, Y may be nested in {A,B,C}: ambiguous.
        let (mut ts, taxa) = trees(&["(((A,B),C),(X,Y));", "((A,B,C,Y),X);"]);
        let id = |s: &str| taxa.index[s];
        let mccs = vec![vec![id("X")], vec![id("Y")], vec![id("A"), id("B"), id("C")]];
        let (a, b) = ts.split_at_mut(1);
        let ns = resolve_with_mccs(&mut a[0], &mut b[0], &mccs, taxa.len(), true);
        assert!(!names(&ns[1], &taxa).contains(&vec!["A".to_string(), "B".into(), "C".into()]));
    }

    #[test]
    fn with_mccs_reassorted_leaf() {
        let (mut ts, taxa) = trees(&["((A,B),(C,(D,(E,X))));", "((A,(B,X)),(C,D,E));"]);
        let id = |s: &str| taxa.index[s];
        let mccs = vec![vec![id("X")], vec![id("A"), id("B"), id("C"), id("D"), id("E")]];
        let (a, b) = ts.split_at_mut(1);
        resolve_with_mccs(&mut a[0], &mut b[0], &mccs, taxa.len(), true);
        assert!(splits(&ts[1], &taxa).contains(&vec!["D".to_string(), "E".to_string()]));
    }

    #[test]
    fn partial_overlap_keeps_private_leaves_outside() {
        // P is only in tree 2; split (B,C) from tree 1 should group B and C but leave P.
        let (mut ts, taxa) = trees(&["(A,(B,C));", "(A,B,C,P);"]);
        resolve_trees(&mut ts, taxa.len());
        assert_eq!(splits(&ts[1], &taxa), vec![vec!["B", "C"]]);
    }
}
