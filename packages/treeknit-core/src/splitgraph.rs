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
  colors: Vec<Color>,
}

/// The internal nodes of one tree.
struct Color {
  clade: Vec<Bits>,
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
    let (mut l, mut z) = (0.0_f64, 0.0_f64);
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
      leaf_children: vec![],
      tree_node: vec![],
      leaf_anc: vec![usize::MAX; n],
      leaf_node: vec![usize::MAX; n],
    };
    // A single-leaf tree gets an artificial root above the leaf.
    if t.is_leaf(t.root) {
      c.clade.push(bits::from_iter(n, [t.taxon(t.root)]));
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
      let index = c.clade.len();
      idx[v] = index;
      let parent = t.parent(v).map(|tp| idx[tp]);
      if let Some(parent) = parent {
        c.children[parent].push(index);
      }
      c.parent.push(parent);
      c.children.push(vec![]);
      c.leaf_children.push(vec![]);
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
  /// `count[k][a]`: kept leaves below internal node `a` of color `k`.
  count: Vec<Vec<u32>>,
  /// Mismatching tree pairs of each kept leaf (0 for removed leaves).
  term: Vec<u32>,
  energy: usize,
  /// Undo information for the last flip: the leaf and the previous terms.
  last: Option<(usize, Vec<(usize, u32)>)>,
}

impl<'g> EnergyState<'g> {
  pub fn new(g: &'g Graph, conf: Bits, resolve: bool) -> Self {
    let count = g
      .colors
      .iter()
      .map(|c| c.clade.iter().map(|x| x.intersection_count(&conf) as u32).collect())
      .collect();
    let mut s = EnergyState {
      g,
      resolve,
      conf,
      count,
      term: vec![0; g.n],
      energy: 0,
      last: None,
    };
    for i in 0..g.n {
      s.term[i] = s.leaf_term(i);
    }
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

  fn leaf_term(&self, i: usize) -> u32 {
    if !self.conf.contains(i) || self.g.n == 1 {
      return 0;
    }
    let k = self.g.k();
    let anc: Vec<usize> = (0..k).map(|kk| self.climb(kk, i)).collect();
    let mut e = 0;
    for k1 in 0..k {
      for k2 in k1 + 1..k {
        if !self.g.compatible(k1, anc[k1], k2, anc[k2], &self.conf, self.resolve) {
          e += 1;
        }
      }
    }
    e
  }

  /// Kept leaves whose term may depend on whether `j` is kept, in the current state.
  fn candidates(&self, j: usize, out: &mut Vec<usize>) {
    for (k, c) in self.g.colors.iter().enumerate() {
      let mut a = Some(c.leaf_anc[j]);
      while let Some(v) = a {
        out.extend(c.leaf_children[v].iter().copied().filter(|&x| self.conf.contains(x)));
        for &ch in &c.children[v] {
          if self.count[k][ch] == 1 {
            out.push(self.single_kept(k, ch));
          }
        }
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
    let mut cand = vec![j];
    self.candidates(j, &mut cand);
    let add = !self.conf.contains(j);
    self.conf.toggle(j);
    self.update_counts(j, add);
    self.candidates(j, &mut cand);
    cand.sort_unstable();
    cand.dedup();
    let mut old = Vec::with_capacity(cand.len());
    for &i in &cand {
      let t = self.leaf_term(i);
      old.push((i, self.term[i]));
      self.energy = self.energy + t as usize - self.term[i] as usize;
      self.term[i] = t;
    }
    self.last = Some((j, old));
    self.energy
  }

  /// Revert the last flip.
  pub fn undo(&mut self) {
    let (j, old) = self.last.take().expect("nothing to undo");
    for (i, t) in old {
      self.energy = self.energy + t as usize - self.term[i] as usize;
      self.term[i] = t;
    }
    let add = !self.conf.contains(j);
    self.conf.toggle(j);
    self.update_counts(j, add);
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

  /// Incremental energy equals the full recomputation along random flip sequences.
  #[test]
  fn incremental_energy_matches_full() {
    use rand::{Rng, SeedableRng};
    let nwk = [
      "(((A,B),(C,(D,E))),((F,G),(H,(I,J))),K,L);",
      "(((A,C),(B,(D,K))),((F,(G,L)),(H,I)),J,E);",
      "((A,(B,C,D)),(E,F,G),((H,I),(J,K,L)));",
    ];
    let (ts, taxa) = trees(&nwk);
    let refs: Vec<&Tree> = ts.iter().collect();
    let g = Graph::new(&refs, taxa.len());
    let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(3);
    for resolve in [false, true] {
      let mut st = EnergyState::new(&g, bits::full(g.n), resolve);
      assert_eq!(st.energy(), g.energy(st.conf(), resolve));
      for step in 0..3000 {
        let j = rng.gen_range(0..g.n);
        let e = st.flip(j);
        assert_eq!(e, g.energy(st.conf(), resolve), "step {step} resolve {resolve}");
        if rng.gen_bool(0.5) {
          st.undo();
          assert_eq!(st.energy(), g.energy(st.conf(), resolve), "undo {step}");
        }
      }
    }
  }

  #[test]
  fn identical_trees_have_zero_energy() {
    let (ts, _) = trees(&["((A,B),(C,D));", "((A,B),(C,D));"]);
    let g = Graph::new(&[&ts[0], &ts[1]], 4);
    assert_eq!(g.energy(&bits::full(4), false), 0);
  }
}
