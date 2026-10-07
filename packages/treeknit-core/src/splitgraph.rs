//! Graph of splits of MCC-reduced trees, with the topological energy of a configuration
//! and the branch-length likelihood used to break ties.
//!
//! A configuration is a bitset over graph leaves (naive MCCs): set = kept, unset = removed.

#![expect(
  clippy::as_conversions,
  clippy::expect_used,
  clippy::unwrap_used,
  reason = "findings from before the strict lint set; kb/issues/N-lint-baseline.md tracks their removal"
)]

use crate::bits::{self, Bits};
use crate::tree::{NodeId, Tree};

pub struct Graph {
  /// Number of leaves.
  pub(crate) n: usize,
  /// `bit[i]`: position of leaf `i` in the leaf sets of the internal nodes. Leaves follow their
  /// left-to-right order in the first tree, so that every clade of that tree, and most clades of
  /// similar trees, cover a short range of words.
  bit: Vec<usize>,
  colors: Vec<Color>,
}

/// The internal nodes of one tree.
struct Color {
  clade: Clades,
  parent: Vec<Option<usize>>,
  /// Internal children of each internal node.
  children: Vec<Vec<usize>>,
  /// Leaf children of each internal node.
  leaf_children: Vec<Vec<usize>>,
  tree_node: Vec<NodeId>,
  /// Parent (internal index) of each leaf.
  leaf_anc: Vec<usize>,
  leaf_node: Vec<NodeId>,
}

impl Graph {
  /// Build from trees whose leaves have taxa `0..n` (the same set in every tree).
  pub fn new(trees: &[&Tree], n: usize) -> Graph {
    let mut bit = vec![usize::MAX; n];
    for (pos, v) in trees[0].leaves().into_iter().enumerate() {
      bit[trees[0].taxon(v)] = pos;
    }
    let colors = trees.iter().map(|t| Color::new(t, n, &bit)).collect();
    Graph { n, bit, colors }
  }

  /// `conf` in the bit positions of the leaf sets.
  fn mask(&self, conf: &Bits) -> Bits {
    bits::from_iter(self.n, conf.ones().map(|i| self.bit[i]))
  }

  fn k(&self) -> usize {
    self.colors.len()
  }

  /// First ancestor of `leaf` in color `k` that has at least two leaves of `mask` below it
  /// (or the root).
  #[inline]
  fn climb(&self, k: usize, leaf: usize, mask: &Bits) -> usize {
    let c = &self.colors[k];
    let mut a = c.leaf_anc[leaf];
    while c.clade.get(a).trivial_on(mask.as_slice()) {
      match c.parent[a] {
        Some(p) => a = p,
        None => break,
      }
    }
    a
  }

  /// Are the clades of node `a1` of color `k1` and node `a2` of color `k2` equal on the leaves of
  /// `mask`, or with `resolve`, is one inside the other and the larger node refinable?
  fn compatible(&self, k1: usize, a1: usize, k2: usize, a2: usize, mask: &Bits, resolve: bool) -> bool {
    let m = mask.as_slice();
    let (c1, c2) = (self.colors[k1].clade.get(a1), self.colors[k2].clade.get(a2));
    if c1.eq_on(c2, m) {
      return true;
    }
    if !resolve {
      return false;
    }
    (c1.subset_on(c2, m) && self.refinable(c1, k2, a2, m)) || (c2.subset_on(c1, m) && self.refinable(c2, k1, a1, m))
  }

  /// Can node `a` of color `k` be resolved to contain split `s`? True if every internal
  /// child of `a` is either inside `s` or disjoint from it.
  fn refinable(&self, s: Clade<'_>, k: usize, a: usize, mask: &[usize]) -> bool {
    let color = &self.colors[k];
    color.children[a].iter().all(|&ch| {
      let child = color.clade.get(ch);
      child.subset_on(s, mask) || child.disjoint_on(s, mask)
    })
  }

  /// Number of (kept leaf, tree pair) for which the first non-trivial ancestors differ.
  pub fn energy(&self, conf: &Bits, resolve: bool) -> usize {
    if self.n == 1 {
      return 0;
    }
    let k = self.k();
    let mask = self.mask(conf);
    let mut anc = vec![0; k];
    let mut e = 0;
    for i in conf.ones() {
      for (kk, a) in anc.iter_mut().enumerate() {
        *a = self.climb(kk, i, &mask);
      }
      for k1 in 0..k {
        for k2 in k1 + 1..k {
          if !self.compatible(k1, anc[k1], k2, anc[k2], &mask, resolve) {
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
    let mask = self.mask(conf);
    let (mut l, mut z) = (0.0_f64, 0.0_f64);
    for i in 0..self.n {
      if conf.contains(i) {
        let anc: Vec<usize> = (0..k).map(|kk| self.climb(kk, i, &mask)).collect();
        for k1 in 0..k {
          for k2 in k1 + 1..k {
            if self.compatible(k1, anc[k1], k2, anc[k2], &mask, resolve) {
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
  /// The internal nodes of `t`, with leaf `i` at position `bit[i]` of the leaf sets.
  fn new(t: &Tree, n: usize, bit: &[usize]) -> Color {
    let mut idx = vec![usize::MAX; t.nodes.len()];
    let mut c = Color {
      clade: Clades::default(),
      parent: vec![],
      children: vec![],
      leaf_children: vec![],
      tree_node: vec![],
      leaf_anc: vec![usize::MAX; n],
      leaf_node: vec![usize::MAX; n],
    };
    // A single-leaf tree gets an artificial root above the leaf.
    if t.is_leaf(t.root) {
      c.clade = Clades::new(vec![bits::from_iter(n, [bit[t.taxon(t.root)]])]);
      c.parent.push(None);
      c.children.push(vec![]);
      c.leaf_children.push(vec![t.taxon(t.root)]);
      c.tree_node.push(t.root);
      c.leaf_anc[t.taxon(t.root)] = 0;
      c.leaf_node[t.taxon(t.root)] = t.root;
      return c;
    }
    for v in t.preorder() {
      if t.is_leaf(v) {
        let taxon = t.taxon(v);
        c.leaf_anc[taxon] = idx[t.parent(v).unwrap()];
        c.leaf_children[c.leaf_anc[taxon]].push(taxon);
        c.leaf_node[taxon] = v;
        continue;
      }
      let index = c.parent.len();
      idx[v] = index;
      let parent = t.parent(v).map(|tp| idx[tp]);
      if let Some(parent) = parent {
        c.children[parent].push(index);
      }
      c.parent.push(parent);
      c.children.push(vec![]);
      c.leaf_children.push(vec![]);
      c.tree_node.push(v);
    }
    let clades = t.clades(n);
    c.clade = Clades::new(
      c.tree_node
        .iter()
        .map(|&v| bits::from_iter(n, clades[v].ones().map(|i| bit[i])))
        .collect(),
    );
    c
  }
}

/// Leaf sets of the internal nodes of one tree, with the range of the non-zero words of each.
#[derive(Default)]
struct Clades {
  sets: Vec<Bits>,
  /// `lo[a]..hi[a]`: the words of set `a` outside which all words are 0.
  lo: Vec<usize>,
  hi: Vec<usize>,
}

impl Clades {
  fn new(sets: Vec<Bits>) -> Self {
    let (mut lo, mut hi) = (vec![], vec![]);
    for s in &sets {
      let w = s.as_slice();
      let l = w.iter().position(|&x| x != 0).unwrap_or(0);
      lo.push(l);
      hi.push(w.iter().rposition(|&x| x != 0).map_or(l, |h| h + 1));
    }
    Clades { sets, lo, hi }
  }

  #[inline]
  fn get(&self, a: usize) -> Clade<'_> {
    Clade {
      words: self.sets[a].as_slice(),
      lo: self.lo[a],
      hi: self.hi[a],
    }
  }
}

/// A leaf set whose words outside `lo..hi` are 0. The masks `m` of the methods are bit sets over
/// all leaves.
#[derive(Clone, Copy)]
struct Clade<'a> {
  words: &'a [usize],
  lo: usize,
  hi: usize,
}

impl Clade<'_> {
  /// `|self ∩ m|`
  #[inline]
  fn count_on(self, m: &[usize]) -> u32 {
    let r = self.lo..self.hi;
    self.words[r.clone()]
      .iter()
      .zip(&m[r])
      .map(|(x, z)| (x & z).count_ones())
      .sum()
  }

  /// `|self ∩ m| < 2`
  #[inline]
  fn trivial_on(self, m: &[usize]) -> bool {
    let r = self.lo..self.hi;
    let mut n = 0;
    for (x, z) in self.words[r.clone()].iter().zip(&m[r]) {
      n += (x & z).count_ones();
      if n > 1 {
        return false;
      }
    }
    true
  }

  /// `self ∩ m ⊆ other`
  #[inline]
  fn subset_on(self, other: Clade<'_>, m: &[usize]) -> bool {
    let r = self.lo..self.hi;
    self.words[r.clone()]
      .iter()
      .zip(&other.words[r.clone()])
      .zip(&m[r])
      .all(|((x, y), z)| x & !y & z == 0)
  }

  /// `self ∩ other ∩ m == ∅`
  #[inline]
  fn disjoint_on(self, other: Clade<'_>, m: &[usize]) -> bool {
    let r = self.lo.max(other.lo)..self.hi.min(other.hi);
    r.is_empty()
      || self.words[r.clone()]
        .iter()
        .zip(&other.words[r.clone()])
        .zip(&m[r])
        .all(|((x, y), z)| x & y & z == 0)
  }

  /// `self ∩ m == other ∩ m`
  #[inline]
  fn eq_on(self, other: Clade<'_>, m: &[usize]) -> bool {
    let r = self.lo.min(other.lo)..self.hi.max(other.hi);
    self.words[r.clone()]
      .iter()
      .zip(&other.words[r.clone()])
      .zip(&m[r])
      .all(|((x, y), z)| (x ^ y) & z == 0)
  }
}

/// Energy of a configuration maintained under single-leaf flips.
///
/// Flipping leaf `j` can only change the terms of `j` itself and of kept leaves whose first
/// non-trivial ancestor is an ancestor of `j` in some tree. Those leaves are the unique
/// kept leaf below each child (with exactly one kept leaf) of each ancestor of `j`. The
/// number of kept leaves below every node is maintained along `j`'s root paths.
pub struct EnergyState<'g> {
  g: &'g Graph,
  resolve: bool,
  conf: Bits,
  /// `conf` in the bit positions of the leaf sets.
  mask: Bits,
  /// `count[k][a]`: kept leaves below internal node `a` of color `k`.
  count: Vec<Vec<u32>>,
  /// Mismatching tree pairs of each kept leaf (0 for removed leaves).
  term: Vec<u32>,
  energy: usize,
  /// The leaf of the last flip, for [`EnergyState::undo`].
  flipped: Option<usize>,
  /// The terms before the last flip of the leaves it changed.
  old: Vec<(usize, u32)>,
  /// Buffer of the leaves whose terms a flip recomputes.
  cand: Vec<usize>,
  /// Buffer of the first non-trivial ancestor of a leaf in each color.
  anc: Vec<usize>,
  /// Number of flips so far.
  flips: u64,
  /// `stamp[i]`: the last flip that recomputed the term of leaf `i`.
  stamp: Vec<u64>,
}

impl<'g> EnergyState<'g> {
  pub fn new(g: &'g Graph, conf: Bits, resolve: bool) -> Self {
    let mask = g.mask(&conf);
    let count = g
      .colors
      .iter()
      .map(|c| {
        (0..c.parent.len())
          .map(|a| c.clade.get(a).count_on(mask.as_slice()))
          .collect()
      })
      .collect();
    let mut s = EnergyState {
      g,
      resolve,
      conf,
      mask,
      count,
      term: vec![0; g.n],
      energy: 0,
      flipped: None,
      old: vec![],
      cand: vec![],
      anc: vec![0; g.k()],
      flips: 0,
      stamp: vec![0; g.n],
    };
    let mut anc = std::mem::take(&mut s.anc);
    for i in 0..g.n {
      s.term[i] = s.leaf_term(i, &mut anc);
    }
    s.anc = anc;
    s.energy = s.term.iter().map(|&x| x as usize).sum();
    s
  }

  pub fn conf(&self) -> &Bits {
    &self.conf
  }
  pub fn energy(&self) -> usize {
    self.energy
  }
  pub fn n_kept(&self) -> usize {
    self.conf.count_ones(..)
  }

  fn climb(&self, k: usize, leaf: usize) -> usize {
    let c = &self.g.colors[k];
    let mut a = c.leaf_anc[leaf];
    while self.count[k][a] < 2 {
      match c.parent[a] {
        Some(p) => a = p,
        None => break,
      }
    }
    a
  }

  /// Term of leaf `i`; `anc` is a buffer with one entry per color.
  fn leaf_term(&self, i: usize, anc: &mut [usize]) -> u32 {
    if !self.conf.contains(i) || self.g.n == 1 {
      return 0;
    }
    let k = self.g.k();
    for (kk, a) in anc.iter_mut().enumerate() {
      *a = self.climb(kk, i);
    }
    let mut e = 0;
    for k1 in 0..k {
      for k2 in k1 + 1..k {
        if !self.g.compatible(k1, anc[k1], k2, anc[k2], &self.mask, self.resolve) {
          e += 1;
        }
      }
    }
    e
  }

  /// Update the counts of the ancestors of `j`, whose state `j` just changed, and add to `out`
  /// the kept leaves other than `j` whose term may depend on whether `j` is kept.
  ///
  /// Those are the kept leaf children of the ancestors of `j` and the only kept leaf below each
  /// child with exactly one, before or after the flip. Only the child on the path to `j` changes
  /// its count, and it needs no check: a kept leaf other than `j` alone below it is a leaf child
  /// of a lower ancestor or alone below one of its other children, which the walk visits.
  fn flip_ancestors(&mut self, j: usize, add: bool, out: &mut Vec<usize>) {
    let g = self.g;
    for (k, c) in g.colors.iter().enumerate() {
      let mut below = None;
      let mut a = Some(c.leaf_anc[j]);
      while let Some(v) = a {
        if add {
          self.count[k][v] += 1;
        } else {
          self.count[k][v] -= 1;
        }
        out.extend(
          c.leaf_children[v]
            .iter()
            .copied()
            .filter(|&x| x != j && self.conf.contains(x)),
        );
        for &ch in &c.children[v] {
          if Some(ch) != below && self.count[k][ch] == 1 {
            out.push(self.single_kept(k, ch));
          }
        }
        below = Some(v);
        a = c.parent[v];
      }
    }
  }

  /// The only kept leaf below `a` (which has exactly one).
  fn single_kept(&self, k: usize, mut a: usize) -> usize {
    let c = &self.g.colors[k];
    loop {
      if let Some(&x) = c.leaf_children[a].iter().find(|&&x| self.conf.contains(x)) {
        return x;
      }
      a = *c.children[a].iter().find(|&&ch| self.count[k][ch] == 1).unwrap();
    }
  }

  fn update_counts(&mut self, j: usize, add: bool) {
    for (k, c) in self.g.colors.iter().enumerate() {
      let mut a = Some(c.leaf_anc[j]);
      while let Some(v) = a {
        if add {
          self.count[k][v] += 1;
        } else {
          self.count[k][v] -= 1;
        }
        a = c.parent[v];
      }
    }
  }

  /// Flip leaf `j` and return the new energy. [`EnergyState::undo`] reverts it.
  pub fn flip(&mut self, j: usize) -> usize {
    let mut cand = std::mem::take(&mut self.cand);
    let mut anc = std::mem::take(&mut self.anc);
    cand.clear();
    cand.push(j);
    let add = !self.conf.contains(j);
    self.conf.toggle(j);
    self.mask.toggle(self.g.bit[j]);
    self.flip_ancestors(j, add, &mut cand);
    self.flips += 1;
    let (flips, stamp) = (self.flips, &mut self.stamp);
    cand.retain(|&i| std::mem::replace(&mut stamp[i], flips) != flips);
    self.old.clear();
    for &i in &cand {
      let t = self.leaf_term(i, &mut anc);
      self.old.push((i, self.term[i]));
      self.energy = self.energy + t as usize - self.term[i] as usize;
      self.term[i] = t;
    }
    self.cand = cand;
    self.anc = anc;
    self.flipped = Some(j);
    self.energy
  }

  /// Revert the last flip.
  pub fn undo(&mut self) {
    let j = self.flipped.take().expect("nothing to undo");
    for &(i, t) in &self.old {
      self.energy = self.energy + t as usize - self.term[i] as usize;
      self.term[i] = t;
    }
    let add = !self.conf.contains(j);
    self.conf.toggle(j);
    self.mask.toggle(self.g.bit[j]);
    self.update_counts(j, add);
  }
}

/// Log-ratio of the Poisson likelihoods of branches `t1`, `t2` (segment lengths `l1`, `l2`)
/// being equal versus independent. A missing, negative, or non-finite length gives 0: the model
/// has no likelihood for it.
fn branch_likelihood(t1: Option<f64>, t2: Option<f64>, l1: f64, l2: f64) -> f64 {
  let valid = |t: &f64| t.is_finite() && *t >= 0.0;
  let (Some(t1), Some(t2)) = (t1.filter(valid), t2.filter(valid)) else {
    return 0.0;
  };
  let mean = (t1 * l1 + t2 * l2) / (l1 + l2);
  let term = |n: f64, ns: f64| if n != 0.0 { n - ns + n * (ns / n).ln() } else { -ns };
  term(t1 * l1, mean * l1) + term(t2 * l2, mean * l2)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::tree::test_util::trees;
  use pretty_assertions::assert_eq;
  use rand::{Rng, SeedableRng};
  use rand_xoshiro::Xoshiro256PlusPlus;
  use rstest::rstest;
  use std::iter;

  /// The graph of the basic case of the Julia test suite (test/splitgraph/basic), whose leaves
  /// are A to E in this order.
  fn basic_graph() -> Graph {
    let (ts, _) = trees(&["((A,B),((C,D),E));", "((A,(B,C)),(D,E));"]);
    Graph::new(&[&ts[0], &ts[1]], 5)
  }

  /// `bits::full(5)` without the leaves `removed`.
  fn without(removed: &[usize]) -> Bits {
    let mut c = bits::full(5);
    for &i in removed {
      c.set(i, false);
    }
    c
  }

  #[test]
  fn basic_energy() {
    let g = basic_graph();
    // Removing C, the leaf that reassorts, leaves no incompatibility.
    assert_eq!(
      (5, 0),
      (g.energy(&bits::full(5), false), g.energy(&without(&[2]), false))
    );
  }

  #[rstest]
  #[trace]
  fn basic_energy_without_c_and_another_leaf_is_zero(#[values(0, 1, 3, 4)] leaf: usize) {
    assert_eq!(0, basic_graph().energy(&without(&[2, leaf]), false));
  }

  #[rstest]
  #[trace]
  fn basic_energy_without_a_leaf_other_than_c_is_four(#[values(0, 1, 3, 4)] leaf: usize) {
    assert_eq!(4, basic_graph().energy(&without(&[leaf]), false));
  }

  /// Incremental energy equals the full recomputation along random flip sequences.
  #[rstest]
  #[trace]
  fn incremental_energy_matches_full(#[values(false, true)] resolve: bool) {
    let nwk = [
      "(((A,B),(C,(D,E))),((F,G),(H,(I,J))),K,L);",
      "(((A,C),(B,(D,K))),((F,(G,L)),(H,I)),J,E);",
      "((A,(B,C,D)),(E,F,G),((H,I),(J,K,L)));",
    ];
    let (ts, taxa) = trees(&nwk);
    let refs: Vec<&Tree> = ts.iter().collect();
    let g = Graph::new(&refs, taxa.len());
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(3);
    let mut st = EnergyState::new(&g, bits::full(g.n), resolve);
    assert_eq!(st.energy(), g.energy(st.conf(), resolve));
    // The step, the operation, and the incremental and full energies of each mismatch.
    let mut mismatches: Vec<(usize, &str, usize, usize)> = Vec::new();
    for step in 0..3000 {
      let j = rng.gen_range(0..g.n);
      let e = st.flip(j);
      let full = g.energy(st.conf(), resolve);
      if e != full {
        mismatches.push((step, "flip", e, full));
      }
      if rng.gen_bool(0.5) {
        st.undo();
        let full = g.energy(st.conf(), resolve);
        if st.energy() != full {
          mismatches.push((step, "undo", st.energy(), full));
        }
      }
    }
    assert_eq!(Vec::<(usize, &str, usize, usize)>::new(), mismatches);
  }

  /// The operations on word ranges agree with those on whole bit sets, for masks of density `p`.
  #[rstest]
  #[trace]
  fn clade_ranges_match_bit_sets(#[values(0.02, 0.3, 0.9)] p: f64) {
    let n = 300;
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(5);
    // Sets within random ranges of leaves, as the clades of a tree in its leaf order, including
    // empty and dense ones.
    let sets: Vec<Bits> = iter::repeat_with(|| {
      let lo = rng.gen_range(0..n);
      let hi = rng.gen_range(lo..=n);
      let density = rng.gen_range(0.0..1.0);
      bits::from_iter(n, (lo..hi).filter(|_| rng.gen_bool(density)))
    })
    .take(60)
    .collect();
    let clades = Clades::new(sets.clone());
    let mask = bits::from_iter(n, (0..n).filter(|_| rng.gen_bool(p)));
    let m = mask.as_slice();
    // The operation and the set of each disagreement on one set.
    let single: Vec<(&str, usize)> = sets
      .iter()
      .enumerate()
      .flat_map(|(a, sa)| {
        let ca = clades.get(a);
        [
          ("count", ca.count_on(m) as usize == sa.intersection_count(&mask)),
          ("trivial", ca.trivial_on(m) == bits::trivial_on(sa, &mask)),
        ]
        .into_iter()
        .filter(|&(_, agree)| !agree)
        .map(move |(op, _)| (op, a))
      })
      .collect();
    // The operation and the sets of each disagreement on a pair of sets.
    let pairs: Vec<(&str, usize, usize)> = (0..sets.len())
      .flat_map(|a| (0..sets.len()).map(move |b| (a, b)))
      .flat_map(|(a, b)| {
        let (ca, cb, sa, sb) = (clades.get(a), clades.get(b), &sets[a], &sets[b]);
        [
          ("subset", ca.subset_on(cb, m) == bits::subset_on(sa, sb, &mask)),
          ("disjoint", ca.disjoint_on(cb, m) == bits::disjoint_on(sa, sb, &mask)),
          ("eq", ca.eq_on(cb, m) == bits::eq_on(sa, sb, &mask)),
        ]
        .into_iter()
        .filter(|&(_, agree)| !agree)
        .map(move |(op, _)| (op, a, b))
      })
      .collect();
    assert_eq!((Vec::new(), Vec::new()), (single, pairs));
  }

  #[test]
  #[expect(clippy::float_cmp, reason = "a missing length contributes exactly 0")]
  fn branch_likelihood_treats_negative_and_non_finite_lengths_as_missing() {
    let lk = [
      branch_likelihood(Some(-0.1), Some(0.3), 1.0, 1.0),
      branch_likelihood(Some(0.2), Some(-0.2), 1.0, 1.0),
      branch_likelihood(Some(f64::INFINITY), Some(0.3), 1.0, 1.0),
      branch_likelihood(Some(0.3), Some(f64::NAN), 1.0, 1.0),
    ];
    assert_eq!(lk, [0.0; 4]);
  }

  #[test]
  fn identical_trees_have_zero_energy() {
    let (ts, _) = trees(&["((A,B),(C,D));", "((A,B),(C,D));"]);
    let g = Graph::new(&[&ts[0], &ts[1]], 4);
    assert_eq!(g.energy(&bits::full(4), false), 0);
  }
}
