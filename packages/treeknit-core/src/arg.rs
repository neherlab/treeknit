//! Ancestral reassortment graph (ARG) of two segment trees and their MCCs.
//!
//! Each node stores one parent link per segment, so a reassortment (hybrid) node is a node
//! whose segments have different parents. [`Arg::parents`] gives the grouped view: a list
//! of parents, each carrying a set of segments.

#![expect(
  clippy::disallowed_types,
  clippy::iter_over_hash_type,
  clippy::unwrap_used,
  reason = "findings from before the strict lint set; kb/issues/N-lint-baseline.md tracks their removal"
)]

use crate::bits;
use crate::naive::Mcc;
use crate::resolve::resolve_with_mccs;
use crate::tree::{NodeId, Tree};
use std::collections::HashMap;

/// Parent of a node for one segment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anc {
  /// The node does not carry this segment.
  Absent,
  /// The node is the root of this segment's tree.
  Root,
  Node(usize),
}

#[derive(Clone, Debug)]
pub struct ArgNode {
  pub label: String,
  /// Parent per segment.
  pub anc: [Anc; 2],
  /// Length of the branch to the parent, per segment.
  pub tau: [Option<f64>; 2],
  pub children: Vec<usize>,
  pub hybrid: bool,
  pub is_leaf: bool,
}

#[derive(Clone, Debug)]
pub struct Arg {
  pub nodes: Vec<ArgNode>,
  /// Root of each segment's tree.
  pub roots: [usize; 2],
  /// Tree node (by name) of each ARG node in the two trees.
  pub tree_nodes: Vec<[Option<String>; 2]>,
  /// The two trees the ARG was built from (liberally resolved, with shared singletons).
  pub trees: [Tree; 2],
}

impl ArgNode {
  pub fn has(&self, c: usize) -> bool {
    self.anc[c] != Anc::Absent
  }
  pub fn is_shared(&self) -> bool {
    self.has(0) && self.has(1)
  }
  pub fn is_root(&self, c: usize) -> bool {
    self.anc[c] == Anc::Root
  }
  pub fn is_partial_root(&self) -> bool {
    (self.is_root(0) || self.is_root(1)) && !(self.is_root(0) && self.is_root(1))
  }
}

impl Arg {
  pub fn n_hybrids(&self) -> usize {
    self.nodes.iter().filter(|n| n.hybrid).count()
  }

  /// Distinct parents of node `n` with the segments (0-based) each carries.
  pub fn parents(&self, n: usize) -> Vec<(Anc, Vec<usize>)> {
    let mut out: Vec<(Anc, Vec<usize>)> = Vec::new();
    for c in 0..2 {
      let a = self.nodes[n].anc[c];
      if a == Anc::Absent {
        continue;
      }
      match out.iter_mut().find(|(x, _)| *x == a) {
        Some((_, v)) => v.push(c),
        None => out.push((a, vec![c])),
      }
    }
    out
  }

  /// Segments carried by the edge from `a` to `n`.
  pub fn edge_segments(&self, a: usize, n: usize) -> Vec<usize> {
    (0..2).filter(|&c| self.nodes[n].anc[c] == Anc::Node(a)).collect()
  }

  /// The tree of segment `c` embedded in the ARG, as (node, parent) pairs reachable from its root.
  pub fn segment_tree(&self, c: usize) -> Tree {
    let mut t = Tree::new(format!("segment_{c}"));
    let mut stack = vec![(self.roots[c], t.root)];
    t.nodes[t.root].name = self.nodes[self.roots[c]].label.clone();
    while let Some((a, tn)) = stack.pop() {
      for &ch in &self.nodes[a].children {
        if self.nodes[ch].anc[c] == Anc::Node(a) {
          let x = t.add_node(self.nodes[ch].label.clone(), self.nodes[ch].tau[c]);
          t.attach(tn, x);
          stack.push((ch, x));
        }
      }
    }
    t
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
  NonShared,
  Shared,
  MccRoot,
  SharedSingleton,
}

#[derive(Clone, Copy, Debug)]
struct Status {
  kind: Kind,
  partner: Option<NodeId>,
  partner_is_root: bool,
  mcc: Option<usize>,
}

type StatusMap = HashMap<NodeId, Status>;

#[derive(Debug, thiserror::Error)]
#[error("ARG construction failed: {0}")]
pub struct ArgError(pub String);

/// Build the ARG of `t1` and `t2` (same leaf set, taxa in `0..n_taxa`) given their MCCs.
pub fn arg_from_trees(t1: &Tree, t2: &Tree, mccs: &[Mcc], n_taxa: usize) -> Result<Arg, ArgError> {
  if t1.node(t1.root).branch_length.is_some() || t2.node(t2.root).branch_length.is_some() {
    log::warn!("root of an input tree has a branch length; ARG branch lengths may be off");
  }
  let (mut t1, mut t2) = (t1.clone(), t2.clone());
  resolve_with_mccs(&mut t1, &mut t2, mccs, n_taxa, false);
  let (mut x1, mut x2) = shared_nodes(&t1, &t2, mccs, n_taxa)?;
  let mut singleton = 0;
  for m in mccs {
    fix_shared_singletons(&mut t1, &mut t2, &mut x1, &mut x2, m, n_taxa, &mut singleton)?;
  }
  let mut b = Builder { nodes: Vec::new() };
  let (root1, lm1) = b.first_tree(&t1, &x1);
  let (root2, lm2) = b.add_tree(&t2, &x2, &lm1)?;
  let mut tree_nodes = vec![[None, None]; b.nodes.len()];
  for (tn, &a) in &lm1 {
    tree_nodes[a][0] = Some(t1.name(*tn).to_owned());
  }
  for (tn, &a) in &lm2 {
    tree_nodes[a][1] = Some(t2.name(*tn).to_owned());
  }
  let mut arg = Arg {
    nodes: b.nodes,
    roots: [root1, root2],
    tree_nodes,
    trees: [t1, t2],
  };
  set_branch_lengths(&mut arg, &lm1, &lm2);
  Ok(arg)
}

/// Status of every node of both trees with respect to the MCCs.
fn shared_nodes(t1: &Tree, t2: &Tree, mccs: &[Mcc], n: usize) -> Result<(StatusMap, StatusMap), ArgError> {
  let (c1, c2) = (t1.clades(n), t2.clades(n));
  let (l1, l2) = (t1.leaf_of(n), t2.leaf_of(n));
  let mut x1 = StatusMap::new();
  let mut x2 = StatusMap::new();
  let st = |kind, partner: Option<NodeId>, root: bool, mcc| Status {
    kind,
    partner,
    partner_is_root: root,
    mcc: Some(mcc),
  };
  for (id, m) in mccs.iter().enumerate() {
    let mask = bits::from_iter(n, m.iter().copied());
    let a1 = t1.lca_of(m.iter().map(|&x| l1[x].unwrap())).unwrap();
    let a2 = t2.lca_of(m.iter().map(|&x| l2[x].unwrap())).unwrap();
    x1.insert(a1, st(Kind::MccRoot, Some(a2), a2 == t2.root, id));
    x2.insert(a2, st(Kind::MccRoot, Some(a1), a1 == t1.root, id));
    // `a` is a shared singleton above child `c` if it adds no MCC leaf to `c`.
    let singleton = |t: &Tree, c: &[bits::Bits], a: NodeId, ch: NodeId| {
      if t.is_leaf(ch) {
        bits::trivial_on(&c[a], &mask)
      } else {
        bits::eq_on(&c[a], &c[ch], &mask)
      }
    };
    for &x in m {
      let (mut n1, mut n2) = (l1[x].unwrap(), l2[x].unwrap());
      x1.entry(n1)
        .or_insert_with(|| st(Kind::Shared, Some(n2), n2 == t2.root, id));
      x2.entry(n2)
        .or_insert_with(|| st(Kind::Shared, Some(n1), n1 == t1.root, id));
      while n1 != a1 || n2 != a2 {
        let (p1, p2) = (t1.parent(n1), t2.parent(n2));
        if p1.is_some_and(|p| x1.contains_key(&p)) && p2.is_some_and(|p| x2.contains_key(&p)) {
          break;
        }
        let s1 = (n1 != a1).then(|| singleton(t1, &c1, p1.unwrap(), n1));
        let s2 = (n2 != a2).then(|| singleton(t2, &c2, p2.unwrap(), n2));
        match (s1, s2) {
          (Some(false), Some(false)) => {
            let (p1, p2) = (p1.unwrap(), p2.unwrap());
            x1.entry(p1)
              .or_insert_with(|| st(Kind::Shared, Some(p2), p2 == t2.root, id));
            x2.entry(p2)
              .or_insert_with(|| st(Kind::Shared, Some(p1), p1 == t1.root, id));
            n1 = p1;
            n2 = p2;
            continue;
          },
          (None | Some(false), None | Some(false)) => {
            return Err(ArgError(format!("trees do not match within MCC {id}")));
          },
          _ => {},
        }
        if s1 == Some(true) {
          let p = p1.unwrap();
          x1.insert(
            p,
            Status {
              kind: Kind::SharedSingleton,
              partner: None,
              partner_is_root: false,
              mcc: Some(id),
            },
          );
          n1 = p;
        }
        if s2 == Some(true) {
          let p = p2.unwrap();
          x2.insert(
            p,
            Status {
              kind: Kind::SharedSingleton,
              partner: None,
              partner_is_root: false,
              mcc: Some(id),
            },
          );
          n2 = p;
        }
      }
    }
  }
  let non_shared = Status {
    kind: Kind::NonShared,
    partner: None,
    partner_is_root: false,
    mcc: None,
  };
  for n in t1.internals() {
    x1.entry(n).or_insert(non_shared);
  }
  for n in t2.internals() {
    x2.entry(n).or_insert(non_shared);
  }
  for s in x1.values().filter(|s| s.kind == Kind::Shared) {
    let back = x2.get(&s.partner.unwrap());
    if !back.is_some_and(|b| b.kind == Kind::Shared) {
      return Err(ArgError("inconsistent shared nodes".into()));
    }
  }
  Ok((x1, x2))
}

/// Make both trees contain the same shared nodes by inserting the shared singletons of
/// one tree into the other.
#[allow(clippy::too_many_arguments)]
fn fix_shared_singletons(
  t1: &mut Tree,
  t2: &mut Tree,
  x1: &mut StatusMap,
  x2: &mut StatusMap,
  m: &Mcc,
  n: usize,
  counter: &mut usize,
) -> Result<(), ArgError> {
  let (l1, l2) = (t1.leaf_of(n), t2.leaf_of(n));
  let a1 = t1.lca_of(m.iter().map(|&x| l1[x].unwrap())).unwrap();
  let a2 = t2.lca_of(m.iter().map(|&x| l2[x].unwrap())).unwrap();
  let mut v1 = std::collections::HashSet::new();
  let mut v2 = std::collections::HashSet::new();
  for &x in m {
    let (mut n1, mut n2) = (l1[x].unwrap(), l2[x].unwrap());
    while n1 != a1 && n2 != a2 && !v1.contains(&n1) && !v2.contains(&n2) {
      let (p1, p2) = (t1.parent(n1).unwrap(), t2.parent(n2).unwrap());
      let f1 = x1[&p1].kind == Kind::SharedSingleton;
      let f2 = x2[&p2].kind == Kind::SharedSingleton;
      let (b1, b2) = (t1.node(n1).branch_length, t2.node(n2).branch_length);
      // Which tree receives a copy of the other's singleton.
      let into_t2 = match (f1, f2) {
        (true, true) => !matches!((b1, b2), (Some(x), Some(y)) if x >= y),
        (true, false) => true,
        (false, true) => false,
        (false, false) => {
          v1.insert(n1);
          v2.insert(n2);
          n1 = p1;
          n2 = p2;
          continue;
        },
      };
      *counter += 1;
      let name = format!("Singleton_{counter}");
      if into_t2 {
        introduce_singleton(t2, n2, p1, b1, x2, x1, name);
      } else {
        introduce_singleton(t1, n1, p2, b2, x1, x2, name);
      }
      v1.insert(n1);
      v2.insert(n2);
      n1 = t1.parent(n1).unwrap();
      n2 = t2.parent(n2).unwrap();
      if x1[&n1].partner != Some(n2) || x2[&n2].partner != Some(n1) {
        return Err(ArgError("shared singletons could not be matched".into()));
      }
    }
  }
  Ok(())
}

/// Insert a node `s` above `node` in `t` (partner of `sref` in the other tree) at height `tau`.
fn introduce_singleton(
  t: &mut Tree,
  node: NodeId,
  sref: NodeId,
  tau: Option<f64>,
  x: &mut StatusMap,
  xref: &mut StatusMap,
  name: String,
) {
  let at = match (t.node(node).branch_length, tau) {
    (Some(b), Some(tau)) if b >= tau => Some(tau),
    (Some(b), Some(_)) => Some(b),
    _ => None,
  };
  let s = t.insert_above(node, name, at);
  let id = x[&node].mcc;
  x.insert(
    s,
    Status {
      kind: Kind::Shared,
      partner: Some(sref),
      partner_is_root: false,
      mcc: id,
    },
  );
  let rid = xref[&sref].mcc;
  xref.insert(
    sref,
    Status {
      kind: Kind::Shared,
      partner: Some(s),
      partner_is_root: false,
      mcc: rid,
    },
  );
}

struct Builder {
  nodes: Vec<ArgNode>,
}

impl Builder {
  fn new_node(&mut self, hybrid: bool) -> usize {
    let i = self.nodes.len();
    self.nodes.push(ArgNode {
      label: format!("ARGNode_{}", i + 1),
      anc: [Anc::Absent; 2],
      tau: [None; 2],
      children: vec![],
      hybrid,
      is_leaf: false,
    });
    i
  }

  fn graft(&mut self, a: usize, n: usize, c: usize) {
    self.nodes[n].anc[c] = Anc::Node(a);
    if !self.nodes[a].children.contains(&n) {
      self.nodes[a].children.push(n);
    }
  }

  fn label_leaf(&mut self, a: usize, t: &Tree, tn: NodeId) {
    if t.is_leaf(tn) {
      self.nodes[a].label = t.name(tn).to_owned();
      self.nodes[a].is_leaf = true;
    }
  }

  /// ARG of the first tree, with a hybrid above every MCC root that is not the other root.
  fn first_tree(&mut self, t: &Tree, x: &StatusMap) -> (usize, HashMap<NodeId, usize>) {
    let mut lm = HashMap::new();
    let r = self.new_node(false);
    self.nodes[r].anc[0] = Anc::Root;
    self.label_leaf(r, t, t.root);
    lm.insert(t.root, r);
    let mut stack: Vec<(usize, NodeId)> = t.children(t.root).iter().rev().map(|&c| (r, c)).collect();
    while let Some((a, tn)) = stack.pop() {
      let status = x[&tn];
      let an = if status.kind == Kind::MccRoot && !status.partner_is_root {
        let hybrid = self.new_node(true);
        self.graft(a, hybrid, 0);
        let an = self.new_node(false);
        self.graft(hybrid, an, 0);
        an
      } else {
        let an = self.new_node(false);
        self.graft(a, an, 0);
        an
      };
      self.label_leaf(an, t, tn);
      lm.insert(tn, an);
      stack.extend(t.children(tn).iter().rev().map(|&c| (an, c)));
    }
    (r, lm)
  }

  /// Add the second tree to the ARG.
  fn add_tree(
    &mut self,
    t: &Tree,
    x: &StatusMap,
    lmref: &HashMap<NodeId, usize>,
  ) -> Result<(usize, HashMap<NodeId, usize>), ArgError> {
    let mut lm = HashMap::new();
    let r = match x[&t.root].kind {
      Kind::NonShared => self.new_node(false),
      _ => lmref[&x[&t.root].partner.unwrap()],
    };
    self.nodes[r].anc[1] = Anc::Root;
    lm.insert(t.root, r);
    let mut stack: Vec<(usize, NodeId)> = t.children(t.root).iter().rev().map(|&c| (r, c)).collect();
    while let Some((a, tn)) = stack.pop() {
      let status = x[&tn];
      let an = match status.kind {
        Kind::NonShared => {
          let an = self.new_node(false);
          self.graft(a, an, 1);
          an
        },
        Kind::MccRoot => {
          let an = lmref[&status.partner.unwrap()];
          if status.partner_is_root {
            self.graft(a, an, 1);
          } else {
            let Anc::Node(hybrid) = self.nodes[an].anc[0] else {
              return Err(ArgError("MCC root without hybrid parent".into()));
            };
            if !self.nodes[hybrid].hybrid {
              return Err(ArgError("MCC root without hybrid parent".into()));
            }
            self.graft(a, hybrid, 1);
            self.graft(hybrid, an, 1);
          }
          an
        },
        Kind::Shared => {
          let an = lmref[&status.partner.unwrap()];
          if self.nodes[an].anc[0] != Anc::Node(a) {
            return Err(ArgError(format!("shared node {} has different parents", t.name(tn))));
          }
          self.graft(a, an, 1);
          an
        },
        Kind::SharedSingleton => return Err(ArgError("unresolved shared singleton".into())),
      };
      if t.is_leaf(tn) {
        self.nodes[an].is_leaf = true;
      }
      lm.insert(tn, an);
      stack.extend(t.children(tn).iter().rev().map(|&c| (an, c)));
    }
    Ok((r, lm))
  }
}

/// Branch lengths: averages on fully shared branches, tree values on single-segment
/// branches, and a reassortment placed halfway along the shorter branch above an MCC root.
fn set_branch_lengths(arg: &mut Arg, lm1: &HashMap<NodeId, usize>, lm2: &HashMap<NodeId, usize>) {
  let mut rev: Vec<[Option<NodeId>; 2]> = vec![[None, None]; arg.nodes.len()];
  for (&tn, &a) in lm1 {
    rev[a][0] = Some(tn);
  }
  for (&tn, &a) in lm2 {
    rev[a][1] = Some(tn);
  }
  let [t1, t2] = &arg.trees;
  let bl = |t: &Tree, n: NodeId| t.node(n).branch_length;
  let mut set: Vec<(usize, usize, Option<f64>)> = Vec::new();
  for (a, node) in arg.nodes.iter().enumerate() {
    if node.hybrid {
      continue;
    }
    match rev[a] {
      [None, Some(n2)] => set.push((a, 1, bl(t2, n2))),
      [Some(n1), None] => set.push((a, 0, bl(t1, n1))),
      [Some(n1), Some(n2)] => {
        let (b1, b2) = (bl(t1, n1), bl(t2, n2));
        if let Anc::Node(h) = node.anc[0] {
          if arg.nodes[h].hybrid {
            let tau = match (b1, b2) {
              (None, b) | (b, None) => b,
              (Some(x), Some(y)) => Some(x.min(y)),
            };
            let at = tau.map(|x| x / 2.0);
            let sub = |b: Option<f64>| Some(b? - at?);
            set.extend([(a, 0, at), (a, 1, at), (h, 0, sub(b1)), (h, 1, sub(b2))]);
            continue;
          }
        }
        let (x, y) = match (b1, b2) {
          (None, _) if n1 == t1.root => (None, b2),
          (None, _) => (b2, b2),
          (_, None) if n2 == t2.root => (b1, None),
          (_, None) => (b1, b1),
          (Some(x), Some(y)) => (Some(f64::midpoint(x, y)), Some(f64::midpoint(x, y))),
        };
        set.extend([(a, 0, x), (a, 1, y)]);
      },
      [None, None] => {},
    }
  }
  for (a, c, tau) in set {
    if matches!(arg.nodes[a].anc[c], Anc::Node(_)) {
      arg.nodes[a].tau[c] = tau;
    }
  }
  arg.tree_nodes.resize(arg.nodes.len(), [None, None]);
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::tree::test_util::{splits, trees};

  #[test]
  #[allow(clippy::needless_range_loop)]
  fn one_reassortment() {
    let (ts, taxa) = trees(&[
      "((A:1,B:1):1,(C:1,(D:1,X:1):1):1);",
      "((A:1,(B:1,X:1):1):1,(C:1,D:1):1);",
    ]);
    let id = |s: &str| taxa.index[s];
    let mccs = vec![vec![id("X")], vec![id("A"), id("B"), id("C"), id("D")]];
    let arg = arg_from_trees(&ts[0], &ts[1], &mccs, taxa.len()).unwrap();
    assert_eq!(arg.n_hybrids(), 1);
    for c in 0..2 {
      let mut seg = arg.segment_tree(c);
      seg.assign_taxa(&taxa).unwrap();
      let mut orig = ts[c].clone();
      orig.remove_unary();
      seg.remove_unary();
      assert_eq!(splits(&seg, &taxa), splits(&orig, &taxa), "segment {c}");
    }
    let x = arg.nodes.iter().position(|n| n.label == "X").unwrap();
    let Anc::Node(h) = arg.nodes[x].anc[0] else { panic!() };
    assert!(arg.nodes[h].hybrid);
    assert_eq!(arg.parents(h).len(), 2);
  }
}
