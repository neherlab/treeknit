//! Graph of splits of MCC-reduced trees, with the topological energy of a configuration
//! and the branch-length likelihood used to break ties.
//!
//! A configuration is a bitset over graph leaves (naive MCCs): set = kept, unset = removed.

use crate::bits::{self, Bits};
use crate::tree::{NodeId, Tree};

pub struct Graph {
    /// Number of leaves.
    pub n: usize,
    colors: Vec<Color>,
}

/// The internal nodes of one tree.
struct Color {
    clade: Vec<Bits>,
    parent: Vec<Option<usize>>,
    /// Internal children of each internal node.
    children: Vec<Vec<usize>>,
    tree_node: Vec<NodeId>,
    /// Parent (internal index) of each leaf.
    leaf_anc: Vec<usize>,
    leaf_node: Vec<NodeId>,
}

impl Graph {
    /// Build from trees whose leaves have taxa `0..n` (the same set in every tree).
    pub fn new(trees: &[&Tree], n: usize) -> Graph {
        let colors = trees.iter().map(|t| Color::new(t, n)).collect();
        Graph { n, colors }
    }

    pub fn k(&self) -> usize {
        self.colors.len()
    }

    /// First ancestor of `leaf` in color `k` that has at least two kept leaves below it
    /// (or the root).
    #[inline]
    fn climb(&self, k: usize, leaf: usize, conf: &Bits) -> usize {
        let c = &self.colors[k];
        let mut a = c.leaf_anc[leaf];
        while bits::trivial_on(&c.clade[a], conf) {
            match c.parent[a] {
                Some(p) => a = p,
                None => break,
            }
        }
        a
    }

    fn compatible(&self, k1: usize, a1: usize, k2: usize, a2: usize, conf: &Bits, resolve: bool) -> bool {
        let (c1, c2) = (&self.colors[k1].clade[a1], &self.colors[k2].clade[a2]);
        if bits::eq_on(c1, c2, conf) {
            return true;
        }
        if !resolve {
            return false;
        }
        (bits::subset_on(c1, c2, conf) && self.refinable(c1, k2, a2, conf))
            || (bits::subset_on(c2, c1, conf) && self.refinable(c2, k1, a1, conf))
    }

    /// Can node `a` of color `k` be resolved to contain split `s`? True if every internal
    /// child of `a` is either inside `s` or disjoint from it.
    fn refinable(&self, s: &Bits, k: usize, a: usize, conf: &Bits) -> bool {
        let c = &self.colors[k];
        c.children[a]
            .iter()
            .all(|&ch| bits::subset_on(&c.clade[ch], s, conf) || bits::disjoint_on(&c.clade[ch], s, conf))
    }

    /// Number of (kept leaf, tree pair) for which the first non-trivial ancestors differ.
    pub fn energy(&self, conf: &Bits, resolve: bool) -> usize {
        if self.n == 1 {
            return 0;
        }
        let k = self.k();
        let mut anc = vec![0; k];
        let mut e = 0;
        for i in conf.ones() {
            for (kk, a) in anc.iter_mut().enumerate() {
                *a = self.climb(kk, i, conf);
            }
            for k1 in 0..k {
                for k2 in k1 + 1..k {
                    if !self.compatible(k1, anc[k1], k2, anc[k2], conf, resolve) {
                        e += 1;
                    }
                }
            }
        }
        e
    }

    /// Mean log-likelihood ratio of "shared" vs "not shared" for the branches implied by `conf`.
    /// Missing branch lengths contribute 0.
    pub fn likelihood(&self, conf: &Bits, resolve: bool, trees: &[&Tree], seq_lengths: &[f64]) -> f64 {
        if self.n == 1 {
            return 0.0;
        }
        let k = self.k();
        let (mut l, mut z) = (0.0f64, 0.0f64);
        for i in 0..self.n {
            if conf.contains(i) {
                let anc: Vec<usize> = (0..k).map(|kk| self.climb(kk, i, conf)).collect();
                for k1 in 0..k {
                    for k2 in k1 + 1..k {
                        if self.compatible(k1, anc[k1], k2, anc[k2], conf, resolve) {
                            let t1 = self.branch(k1, i, anc[k1], trees[k1]);
                            let t2 = self.branch(k2, i, anc[k2], trees[k2]);
                            l += branch_likelihood(t1, t2, seq_lengths[k1], seq_lengths[k2]);
                            z += 1.0;
                        }
                    }
                }
            } else {
                for k1 in 0..k {
                    for k2 in 0..k {
                        let t1 = self.leaf_branch(k1, i, trees[k1]);
                        let t2 = self.leaf_branch(k2, i, trees[k2]);
                        l -= branch_likelihood(t1, t2, seq_lengths[k1], seq_lengths[k2]);
                        z += 1.0;
                    }
                }
            }
        }
        l / z.max(1.0)
    }

    fn branch(&self, k: usize, leaf: usize, a: usize, t: &Tree) -> Option<f64> {
        let c = &self.colors[k];
        t.divtime(c.leaf_node[leaf], c.tree_node[a])
    }

    fn leaf_branch(&self, k: usize, leaf: usize, t: &Tree) -> Option<f64> {
        let c = &self.colors[k];
        t.divtime(c.leaf_node[leaf], c.tree_node[c.leaf_anc[leaf]])
    }
}

impl Color {
    fn new(t: &Tree, n: usize) -> Color {
        let mut idx = vec![usize::MAX; t.nodes.len()];
        let mut c = Color {
            clade: vec![],
            parent: vec![],
            children: vec![],
            tree_node: vec![],
            leaf_anc: vec![usize::MAX; n],
            leaf_node: vec![usize::MAX; n],
        };
        // A single-leaf tree gets an artificial root above the leaf.
        if t.is_leaf(t.root) {
            c.clade.push(bits::from_iter(n, [t.taxon(t.root)]));
            c.parent.push(None);
            c.children.push(vec![]);
            c.tree_node.push(t.root);
            c.leaf_anc[t.taxon(t.root)] = 0;
            c.leaf_node[t.taxon(t.root)] = t.root;
            return c;
        }
        for v in t.preorder() {
            if t.is_leaf(v) {
                let x = t.taxon(v);
                c.leaf_anc[x] = idx[t.parent(v).unwrap()];
                c.leaf_node[x] = v;
                continue;
            }
            let i = c.clade.len();
            idx[v] = i;
            let p = t.parent(v).map(|p| idx[p]);
            if let Some(p) = p {
                c.children[p].push(i);
            }
            c.parent.push(p);
            c.children.push(vec![]);
            c.tree_node.push(v);
            c.clade.push(Bits::with_capacity(n));
        }
        let clades = t.clades(n);
        for (i, &v) in c.tree_node.iter().enumerate() {
            c.clade[i] = clades[v].clone();
        }
        c
    }
}

/// Log-ratio of the Poisson likelihoods of branches `t1`, `t2` (segment lengths `l1`, `l2`)
/// being equal versus independent.
pub fn branch_likelihood(t1: Option<f64>, t2: Option<f64>, l1: f64, l2: f64) -> f64 {
    let (Some(t1), Some(t2)) = (t1, t2) else { return 0.0 };
    let mean = (t1 * l1 + t2 * l2) / (l1 + l2);
    let term = |n: f64, ns: f64| if n != 0.0 { n - ns + n * (ns / n).ln() } else { -ns };
    term(t1 * l1, mean * l1) + term(t2 * l2, mean * l2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::test_util::trees;

    #[test]
    fn basic_energy() {
        // From the Julia test suite (test/splitgraph/basic).
        let (ts, _) = trees(&["((A,B),((C,D),E));", "((A,(B,C)),(D,E));"]);
        let g = Graph::new(&[&ts[0], &ts[1]], 5);
        let mut conf = bits::full(5);
        assert_eq!(g.energy(&conf, false), 5);
        conf.set(2, false); // remove C
        assert_eq!(g.energy(&conf, false), 0);
        for i in [0, 1, 3, 4] {
            let mut c = conf.clone();
            c.set(i, false);
            assert_eq!(g.energy(&c, false), 0);
        }
        for i in [0, 1, 3, 4] {
            let mut c = bits::full(5);
            c.set(i, false);
            assert_eq!(g.energy(&c, false), 4);
        }
    }

    #[test]
    fn identical_trees_have_zero_energy() {
        let (ts, _) = trees(&["((A,B),(C,D));", "((A,B),(C,D));"]);
        let g = Graph::new(&[&ts[0], &ts[1]], 4);
        assert_eq!(g.energy(&bits::full(4), false), 0);
    }
}
