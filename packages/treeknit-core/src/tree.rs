//! Rooted trees stored in an arena, plus the taxon table shared by all trees.
//!
//! Nodes are never removed from the arena; detached nodes simply become unreachable
//! from the root. Traversals only visit nodes reachable from the root. Use
//! [`Tree::compacted`] to drop unreachable nodes.

use crate::bits::Bits;
use std::collections::HashMap;

pub type NodeId = usize;

#[derive(Clone, Debug)]
pub struct Node {
    pub name: String,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub branch_length: Option<f64>,
    /// Index of the leaf in the current taxon table (leaves only).
    pub taxon: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct Tree {
    pub label: String,
    pub nodes: Vec<Node>,
    pub root: NodeId,
}

/// Sorted, unique leaf names. Taxon ids follow name order, so sorting ids sorts names.
#[derive(Clone, Debug, Default)]
pub struct Taxa {
    pub names: Vec<String>,
    pub index: HashMap<String, usize>,
}

impl Taxa {
    pub fn new(mut names: Vec<String>) -> Self {
        names.sort();
        names.dedup();
        let index = names.iter().enumerate().map(|(i, n)| (n.clone(), i)).collect();
        Taxa { names, index }
    }
    pub fn from_trees(trees: &[Tree]) -> Self {
        Taxa::new(trees.iter().flat_map(|t| t.leaf_names()).collect())
    }
    pub fn len(&self) -> usize {
        self.names.len()
    }
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
    pub fn names_of(&self, ids: &[usize]) -> Vec<String> {
        ids.iter().map(|&i| self.names[i].clone()).collect()
    }
}

impl Tree {
    pub fn new(label: impl Into<String>) -> Self {
        Tree {
            label: label.into(),
            nodes: vec![Node {
                name: String::new(),
                parent: None,
                children: vec![],
                branch_length: None,
                taxon: None,
            }],
            root: 0,
        }
    }

    pub fn add_node(&mut self, name: impl Into<String>, branch_length: Option<f64>) -> NodeId {
        self.nodes.push(Node {
            name: name.into(),
            parent: None,
            children: vec![],
            branch_length,
            taxon: None,
        });
        self.nodes.len() - 1
    }

    #[inline]
    pub fn node(&self, n: NodeId) -> &Node {
        &self.nodes[n]
    }
    #[inline]
    pub fn children(&self, n: NodeId) -> &[NodeId] {
        &self.nodes[n].children
    }
    #[inline]
    pub fn parent(&self, n: NodeId) -> Option<NodeId> {
        self.nodes[n].parent
    }
    #[inline]
    pub fn is_leaf(&self, n: NodeId) -> bool {
        self.nodes[n].children.is_empty()
    }
    #[inline]
    pub fn name(&self, n: NodeId) -> &str {
        &self.nodes[n].name
    }

    /// Children before parents; children in their stored order.
    pub fn postorder(&self) -> Vec<NodeId> {
        self.postorder_from(self.root)
    }
    pub fn postorder_from(&self, r: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack = vec![(r, false)];
        while let Some((n, expanded)) = stack.pop() {
            if expanded || self.is_leaf(n) {
                out.push(n);
            } else {
                stack.push((n, true));
                for &c in self.children(n).iter().rev() {
                    stack.push((c, false));
                }
            }
        }
        out
    }
    pub fn preorder(&self) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack = vec![self.root];
        while let Some(n) = stack.pop() {
            out.push(n);
            stack.extend(self.children(n).iter().rev());
        }
        out
    }
    /// Leaves in left-to-right order.
    pub fn leaves(&self) -> Vec<NodeId> {
        self.leaves_below(self.root)
    }
    pub fn leaves_below(&self, r: NodeId) -> Vec<NodeId> {
        self.postorder_from(r)
            .into_iter()
            .filter(|&n| self.is_leaf(n))
            .collect()
    }
    pub fn leaf_names(&self) -> Vec<String> {
        self.leaves().into_iter().map(|n| self.nodes[n].name.clone()).collect()
    }
    pub fn n_leaves(&self) -> usize {
        self.leaves().len()
    }
    pub fn internals(&self) -> Vec<NodeId> {
        self.postorder().into_iter().filter(|&n| !self.is_leaf(n)).collect()
    }
    /// Map from node name to id for reachable nodes.
    pub fn name_map(&self) -> HashMap<String, NodeId> {
        self.postorder()
            .into_iter()
            .map(|n| (self.nodes[n].name.clone(), n))
            .collect()
    }

    /// Set `taxon` on all leaves; fails on unknown leaf names.
    pub fn assign_taxa(&mut self, taxa: &Taxa) -> Result<(), String> {
        for n in self.leaves() {
            let id = taxa.index.get(&self.nodes[n].name).ok_or_else(|| {
                format!(
                    "leaf {} of tree {} is not in the taxon table",
                    self.nodes[n].name, self.label
                )
            })?;
            self.nodes[n].taxon = Some(*id);
        }
        Ok(())
    }

    /// `leaf_of[taxon] = node` for leaves of this tree.
    pub fn leaf_of(&self, n_taxa: usize) -> Vec<Option<NodeId>> {
        let mut v = vec![None; n_taxa];
        for n in self.leaves() {
            v[self.taxon(n)] = Some(n);
        }
        v
    }
    #[inline]
    pub fn taxon(&self, n: NodeId) -> usize {
        self.nodes[n].taxon.expect("leaf without taxon")
    }

    /// Leaf set of the tree as a bitset over `n_taxa`.
    pub fn leaf_set(&self, n_taxa: usize) -> Bits {
        crate::bits::from_iter(n_taxa, self.leaves().into_iter().map(|n| self.taxon(n)))
    }

    /// Clade (set of taxa below) of every node, indexed by node id.
    pub fn clades(&self, n_taxa: usize) -> Vec<Bits> {
        let mut c = vec![Bits::with_capacity(n_taxa); self.nodes.len()];
        for n in self.postorder() {
            if self.is_leaf(n) {
                c[n].insert(self.taxon(n));
            } else {
                let mut s = Bits::with_capacity(n_taxa);
                for &ch in self.children(n) {
                    s.union_with(&c[ch]);
                }
                c[n] = s;
            }
        }
        c
    }

    pub fn depth(&self, mut n: NodeId) -> usize {
        let mut d = 0;
        while let Some(p) = self.parent(n) {
            n = p;
            d += 1;
        }
        d
    }

    pub fn lca(&self, a: NodeId, b: NodeId) -> NodeId {
        let (mut a, mut b) = (a, b);
        let (mut da, mut db) = (self.depth(a), self.depth(b));
        while da > db {
            a = self.parent(a).unwrap();
            da -= 1;
        }
        while db > da {
            b = self.parent(b).unwrap();
            db -= 1;
        }
        while a != b {
            a = self.parent(a).unwrap();
            b = self.parent(b).unwrap();
        }
        a
    }

    pub fn lca_of(&self, nodes: impl IntoIterator<Item = NodeId>) -> Option<NodeId> {
        nodes.into_iter().reduce(|a, b| self.lca(a, b))
    }

    pub fn is_ancestor(&self, a: NodeId, mut n: NodeId) -> bool {
        loop {
            if n == a {
                return true;
            }
            match self.parent(n) {
                Some(p) => n = p,
                None => return false,
            }
        }
    }

    /// Sum of branch lengths from `n` up to its ancestor `a`; `None` if any is missing.
    pub fn divtime(&self, mut n: NodeId, a: NodeId) -> Option<f64> {
        let mut d = 0.0;
        while n != a {
            d += self.nodes[n].branch_length?;
            n = self.parent(n).expect("divtime: not an ancestor");
        }
        Some(d)
    }

    /// Detach `n` from its parent.
    pub fn detach(&mut self, n: NodeId) {
        if let Some(p) = self.nodes[n].parent.take() {
            self.nodes[p].children.retain(|&c| c != n);
        }
    }

    /// Append `c` (which must be detached) to the children of `p`.
    pub fn attach(&mut self, p: NodeId, c: NodeId) {
        debug_assert!(self.nodes[c].parent.is_none());
        self.nodes[c].parent = Some(p);
        self.nodes[p].children.push(c);
    }

    /// Remove internal nodes with a single child, adding branch lengths. If the root
    /// has a single internal child, that child becomes the root.
    pub fn remove_unary(&mut self) {
        for n in self.postorder() {
            if n == self.root || self.children(n).len() != 1 {
                continue;
            }
            let c = self.children(n)[0];
            let p = self.parent(n).unwrap();
            let bl = match (self.nodes[c].branch_length, self.nodes[n].branch_length) {
                (Some(x), Some(y)) => Some(x + y),
                _ => None,
            };
            let pos = self.children(p).iter().position(|&x| x == n).unwrap();
            self.nodes[p].children[pos] = c;
            self.nodes[c].parent = Some(p);
            self.nodes[c].branch_length = bl;
            self.nodes[n].parent = None;
            self.nodes[n].children.clear();
        }
        let r = self.root;
        if self.children(r).len() == 1 && !self.is_leaf(self.children(r)[0]) {
            let c = self.children(r)[0];
            self.nodes[r].children.clear();
            self.nodes[c].parent = None;
            self.nodes[c].branch_length = None;
            self.root = c;
        }
    }

    /// Remove the subtree rooted at `n`; parents left without children are removed too.
    /// Unary nodes are spliced out afterwards.
    pub fn prune(&mut self, n: NodeId) {
        assert!(n != self.root, "cannot prune the root");
        let mut n = n;
        loop {
            let p = self.parent(n).unwrap();
            self.detach(n);
            if !self.children(p).is_empty() || p == self.root {
                break;
            }
            n = p;
        }
        self.remove_unary();
    }

    /// Insert a new node `name` as parent of `nodes`, which must share a parent.
    pub fn insert_parent(&mut self, nodes: &[NodeId], name: String, branch_length: Option<f64>) -> NodeId {
        let p = self.parent(nodes[0]).expect("insert_parent: node without parent");
        let s = self.add_node(name, branch_length);
        for &c in nodes {
            debug_assert_eq!(self.parent(c), Some(p));
            self.detach(c);
            self.attach(s, c);
        }
        self.attach(p, s);
        s
    }

    /// Insert node `name` on the branch above `n`, at distance `t` above `n`.
    /// The branch length above the new node is the remainder.
    pub fn insert_above(&mut self, n: NodeId, name: String, t: Option<f64>) -> NodeId {
        let p = self.parent(n).expect("insert_above: root");
        let rest = match (self.nodes[n].branch_length, t) {
            (Some(b), Some(t)) => Some(b - t),
            _ => None,
        };
        let s = self.add_node(name, rest);
        let pos = self.children(p).iter().position(|&x| x == n).unwrap();
        self.nodes[p].children[pos] = s;
        self.nodes[s].parent = Some(p);
        self.nodes[n].parent = Some(s);
        self.nodes[n].branch_length = t;
        self.nodes[s].children.push(n);
        s
    }

    /// First label `prefix_i` (i ≥ 1) such that no `prefix_j` with j ≥ i exists.
    pub fn fresh_label(&self, prefix: &str) -> String {
        format!("{prefix}_{}", self.fresh_index(prefix))
    }
    pub fn fresh_index(&self, prefix: &str) -> usize {
        let pre = format!("{prefix}_");
        self.postorder()
            .into_iter()
            .filter_map(|n| self.nodes[n].name.strip_prefix(&pre)?.parse::<usize>().ok())
            .max()
            .map_or(1, |m| m + 1)
    }

    /// Copy of the tree containing only leaves whose taxon is in `keep`.
    /// Unary nodes are spliced out with branch lengths added. Returns `None` if no leaf is kept.
    pub fn restricted(&self, keep: &Bits) -> Option<Tree> {
        let mut t = self.clone();
        for n in t.leaves() {
            if !keep.contains(t.taxon(n)) && n != t.root {
                let mut x = n;
                loop {
                    let p = t.parent(x).unwrap();
                    t.detach(x);
                    if !t.children(p).is_empty() || p == t.root {
                        break;
                    }
                    x = p;
                }
            }
        }
        if t.is_leaf(t.root) && !t.nodes[t.root].taxon.is_some_and(|x| keep.contains(x)) {
            return None;
        }
        t.remove_unary();
        Some(t.compacted())
    }

    /// Copy without unreachable nodes; node ids are renumbered in preorder.
    pub fn compacted(&self) -> Tree {
        let order = self.preorder();
        let mut map = vec![usize::MAX; self.nodes.len()];
        for (i, &n) in order.iter().enumerate() {
            map[n] = i;
        }
        let nodes = order
            .iter()
            .map(|&n| {
                let o = &self.nodes[n];
                Node {
                    name: o.name.clone(),
                    parent: o.parent.map(|p| map[p]),
                    children: o.children.iter().map(|&c| map[c]).collect(),
                    branch_length: o.branch_length,
                    taxon: o.taxon,
                }
            })
            .collect();
        Tree {
            label: self.label.clone(),
            nodes,
            root: 0,
        }
    }

    /// Sort children by (number of leaves, name), smallest first, recursively.
    pub fn ladderize(&mut self) {
        let mut size = vec![0usize; self.nodes.len()];
        for n in self.postorder() {
            if self.is_leaf(n) {
                size[n] = 1;
            } else {
                let mut ch = self.nodes[n].children.clone();
                ch.sort_by(|&a, &b| {
                    size[a]
                        .cmp(&size[b])
                        .then_with(|| self.nodes[a].name.cmp(&self.nodes[b].name))
                });
                size[n] = ch.iter().map(|&c| size[c]).sum();
                self.nodes[n].children = ch;
            }
        }
    }

    /// Check structural invariants: parent/child links agree and leaves have taxa.
    pub fn check(&self) -> bool {
        self.nodes[self.root].parent.is_none()
            && self.postorder().into_iter().all(|n| {
                self.children(n).iter().all(|&c| self.parent(c) == Some(n))
                    && (!self.is_leaf(n) || self.nodes[n].taxon.is_some())
            })
    }
}

#[cfg(test)]
pub(crate) mod test_util {
    use super::*;

    /// Minimal Newick reader for tests (no quoting, no comments).
    pub fn nwk(s: &str) -> Tree {
        fn rec(t: &mut Tree, s: &[u8], i: &mut usize, parent: Option<NodeId>) -> NodeId {
            let n = if parent.is_none() { 0 } else { t.add_node("", None) };
            if s[*i] == b'(' {
                *i += 1;
                loop {
                    let c = rec(t, s, i, Some(n));
                    t.attach(n, c);
                    let ch = s[*i];
                    *i += 1;
                    if ch == b')' {
                        break;
                    }
                }
            }
            let start = *i;
            while *i < s.len() && !b",():;".contains(&s[*i]) {
                *i += 1;
            }
            t.nodes[n].name = String::from_utf8(s[start..*i].to_vec()).unwrap();
            if *i < s.len() && s[*i] == b':' {
                *i += 1;
                let st = *i;
                while !b",();".contains(&s[*i]) {
                    *i += 1;
                }
                t.nodes[n].branch_length = std::str::from_utf8(&s[st..*i]).unwrap().parse().ok();
            }
            n
        }
        let mut t = Tree::new("t");
        let mut i = 0;
        rec(&mut t, s.as_bytes(), &mut i, None);
        let mut k = 0;
        for n in t.internals() {
            if t.nodes[n].name.is_empty() {
                k += 1;
                t.nodes[n].name = format!("NODE_{k}");
            }
        }
        t
    }

    /// Parse trees and assign a shared taxon table.
    pub fn trees(s: &[&str]) -> (Vec<Tree>, Taxa) {
        let mut ts: Vec<Tree> = s.iter().map(|x| nwk(x)).collect();
        let taxa = Taxa::from_trees(&ts);
        for t in ts.iter_mut() {
            t.assign_taxa(&taxa).unwrap();
        }
        (ts, taxa)
    }

    /// Sorted list of non-root internal clades as sorted name lists.
    pub fn splits(t: &Tree, taxa: &Taxa) -> Vec<Vec<String>> {
        let c = t.clades(taxa.len());
        let mut v: Vec<Vec<String>> = t
            .internals()
            .into_iter()
            .filter(|&n| n != t.root)
            .map(|n| c[n].ones().map(|i| taxa.names[i].clone()).collect())
            .collect();
        v.sort();
        v
    }
}

#[cfg(test)]
mod tests {
    use super::test_util::*;

    #[test]
    fn restrict_and_prune() {
        let (ts, taxa) = trees(&["((A:1,B:1):1,(C:1,(D:1,E:1):2):1);"]);
        let keep = crate::bits::from_iter(taxa.len(), [0, 2, 3]);
        let r = ts[0].restricted(&keep).unwrap();
        assert!(r.check());
        assert_eq!(r.leaf_names(), vec!["A", "C", "D"]);
        let d = r.leaves()[2];
        assert_eq!(r.node(d).branch_length, Some(3.0));
        let a = r.leaves()[0];
        assert_eq!(r.node(a).branch_length, Some(2.0));

        let mut t = ts[0].clone();
        let de = t.parent(t.leaves()[3]).unwrap();
        t.prune(de);
        assert!(t.check());
        assert_eq!(t.leaf_names(), vec!["A", "B", "C"]);
        assert_eq!(t.node(t.leaves()[2]).branch_length, Some(2.0));
    }

    #[test]
    fn ladderize_sorts_by_size_then_name() {
        let (mut ts, _) = trees(&["((C,(A,B)),D);"]);
        ts[0].ladderize();
        assert_eq!(ts[0].leaf_names(), vec!["D", "C", "A", "B"]);
    }
}
