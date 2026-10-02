//! Mapping tree nodes to MCCs (Fitch-like two-pass) and polytomy sorting for tanglegrams.

use crate::naive::Mcc;
use crate::tree::{NodeId, Tree};
use std::collections::{BTreeSet, HashMap};

/// `leaf_mcc[taxon] = index of the MCC containing taxon`, `None` if in no MCC.
pub fn leaf_mcc_map(mccs: &[Mcc], n_taxa: usize) -> Vec<Option<usize>> {
    let mut v = vec![None; n_taxa];
    for (i, m) in mccs.iter().enumerate() {
        for &x in m {
            v[x] = Some(i);
        }
    }
    v
}

/// MCC of every node of `tree` (`None` if the node is not inside an MCC).
///
/// Leaves without an MCC are wildcards: they constrain nothing. Internal nodes whose
/// leaves are all wildcards map to `None`.
pub fn map_mccs(tree: &Tree, leaf_mcc: &[Option<usize>]) -> Vec<Option<usize>> {
    // Upward pass: `None` is the wildcard set.
    let mut up: Vec<Option<BTreeSet<usize>>> = vec![None; tree.nodes.len()];
    for n in tree.postorder() {
        if tree.is_leaf(n) {
            up[n] = leaf_mcc[tree.taxon(n)].map(|m| BTreeSet::from([m]));
            continue;
        }
        let sets: Vec<&BTreeSet<usize>> = tree.children(n).iter().filter_map(|&c| up[c].as_ref()).collect();
        if sets.is_empty() {
            continue;
        }
        let inter: BTreeSet<usize> = sets[0]
            .iter()
            .filter(|m| sets[1..].iter().all(|s| s.contains(m)))
            .copied()
            .collect();
        up[n] = Some(if !inter.is_empty() || n == tree.root {
            inter
        } else {
            sets.iter().flat_map(|s| s.iter().copied()).collect()
        });
    }
    // Downward pass.
    let mut down = vec![None; tree.nodes.len()];
    for n in tree.preorder() {
        let Some(s) = &up[n] else { continue };
        down[n] = if s.len() == 1 {
            s.first().copied()
        } else if let Some(p) = tree.parent(n) {
            down[p].filter(|m| s.contains(m))
        } else {
            None
        };
    }
    down
}

/// For each MCC, the rank of each of its leaves in the left-to-right order of `tree`.
fn leaf_order_in_mccs(tree: &Tree, leaf_mcc: &[Option<usize>], n_mcc: usize) -> Vec<HashMap<usize, usize>> {
    let mut order = vec![HashMap::new(); n_mcc];
    for n in tree.leaves() {
        let x = tree.taxon(n);
        if let Some(m) = leaf_mcc[x] {
            let k = order[m].len() + 1;
            order[m].insert(x, k);
        }
    }
    order
}

/// Stable insertion sort with an arbitrary "less than" predicate.
///
/// The predicates used for polytomy sorting are not total orders, so the standard
/// library sorts (which may panic on inconsistent comparators) are avoided.
fn insertion_sort_by<T>(v: &mut [T], lt: impl Fn(&T, &T) -> bool) {
    for i in 1..v.len() {
        let mut j = i;
        while j > 0 && lt(&v[j], &v[j - 1]) {
            v.swap(j, j - 1);
            j -= 1;
        }
    }
}

/// Reorder children of `t2` so that leaves of the same MCC appear in the same order as in `t1`.
/// The order of `t1` is unchanged. Both trees must have the same leaves.
pub fn sort_polytomies_by_mccs(t1: &Tree, t2: &mut Tree, mccs: &[Mcc], n_taxa: usize) {
    let leaf_mcc = leaf_mcc_map(mccs, n_taxa);
    let order = leaf_order_in_mccs(t1, &leaf_mcc, mccs.len());
    let map = map_mccs(t2, &leaf_mcc);
    // rank[n] = (rank in MCC, clade size); rank in MCC is +∞ if no leaf of the node's MCC is below.
    let mut rank: Vec<(f64, usize)> = vec![(f64::INFINITY, 0); t2.nodes.len()];
    for n in t2.postorder() {
        if t2.is_leaf(n) {
            let x = t2.taxon(n);
            let r = leaf_mcc[x].map_or(f64::INFINITY, |m| order[m][&x] as f64);
            rank[n] = (r, 1);
            continue;
        }
        let mut ch: Vec<(NodeId, Option<f64>, usize)> = t2
            .children(n)
            .iter()
            .map(|&c| (c, (map[c] == map[n]).then_some(rank[c].0), rank[c].1))
            .collect();
        insertion_sort_by(&mut ch, |a, b| match (a.1, b.1) {
            (Some(x), Some(y)) => x < y,
            _ => a.2 < b.2,
        });
        let r = ch.iter().filter_map(|c| c.1).fold(f64::INFINITY, f64::min);
        rank[n] = (r, ch.iter().map(|c| c.2).sum());
        t2.nodes[n].children = ch.into_iter().map(|c| c.0).collect();
    }
}

/// Reorder children of `t` by the minimum position of their leaves in `order` (by taxon).
/// Taxa absent from `order` are placed last.
pub fn sort_by_leaf_order(t: &mut Tree, order: &HashMap<usize, usize>) {
    let mut rank = vec![usize::MAX; t.nodes.len()];
    for n in t.postorder() {
        if t.is_leaf(n) {
            rank[n] = order.get(&t.taxon(n)).copied().unwrap_or(usize::MAX);
        } else {
            let mut ch = t.nodes[n].children.clone();
            ch.sort_by_key(|&c| rank[c]);
            rank[n] = ch.iter().map(|&c| rank[c]).min().unwrap();
            t.nodes[n].children = ch;
        }
    }
}

pub fn leaf_order(t: &Tree) -> HashMap<usize, usize> {
    t.leaves()
        .into_iter()
        .enumerate()
        .map(|(i, n)| (t.taxon(n), i))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::test_util::trees;

    #[test]
    fn fitch_assigns_internal_nodes() {
        let (ts, taxa) = trees(&["((A,B),(C,(D,X)));"]);
        let ids = |v: &[&str]| v.iter().map(|s| taxa.index[*s]).collect::<Vec<_>>();
        let mccs = vec![ids(&["X"]), ids(&["A", "B", "C", "D"])];
        let m = map_mccs(&ts[0], &leaf_mcc_map(&mccs, taxa.len()));
        let t = &ts[0];
        let dx = t.parent(t.leaves()[3]).unwrap();
        assert_eq!(m[dx], Some(1));
        assert_eq!(m[t.root], Some(1));
    }

    #[test]
    fn sorted_tanglegram() {
        let (mut ts, taxa) = trees(&["((A,B),(C,D));", "((D,C),(B,A));"]);
        let all = vec![(0..4).collect::<Vec<_>>()];
        ts[0].ladderize();
        let t1 = ts[0].clone();
        sort_polytomies_by_mccs(&t1, &mut ts[1], &all, taxa.len());
        assert_eq!(ts[0].leaf_names(), ts[1].leaf_names());
    }
}
