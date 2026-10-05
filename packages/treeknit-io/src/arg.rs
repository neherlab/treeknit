//! ARG output: extended Newick (Cardona et al. 2008) and the ARG-node ↔ tree-node table.

#![expect(
  clippy::disallowed_types,
  reason = "findings from before the strict lint set; kb/issues/N-lint-baseline.md tracks their removal"
)]

use crate::newick::fmt_f64;
use std::collections::HashMap;
use treeknit_core::arg::{Anc, Arg};

/// Extended Newick string. Every branch is annotated with the segments it carries
/// (`[&segments={0,1}]`, 0-based); reassortment nodes appear once per parent, as `label#Hi`.
pub fn extended_newick(arg: &Arg) -> String {
  let mut w = Writer {
    arg,
    hybrids: HashMap::new(),
  };
  let [r1, r2] = arg.roots;
  if r1 == r2 {
    return format!("{};", w.node(r1, None));
  }
  let (s1, s2) = (arg.nodes[r1].is_shared(), arg.nodes[r2].is_shared());
  match (s1, s2) {
    (false, false) => format!(
      "({},{})GlobalRoot[&segments={{0,1}}]:0.0;",
      w.node(r1, None),
      w.node(r2, None)
    ),
    (true, _) => format!("{};", w.node(r2, None)),
    (false, true) => format!("{};", w.node(r1, None)),
  }
}

struct Writer<'a> {
  arg: &'a Arg,
  hybrids: HashMap<usize, usize>,
}

impl Writer<'_> {
  fn node(&mut self, n: usize, anc: Option<usize>) -> String {
    let node = &self.arg.nodes[n];
    let mut s = String::new();
    if !node.children.is_empty() && !self.hybrids.contains_key(&n) {
      let parts: Vec<String> = node
        .children
        .clone()
        .into_iter()
        .map(|c| self.node(c, Some(n)))
        .collect();
      s.push('(');
      s.push_str(&parts.join(","));
      s.push(')');
    }
    if node.hybrid && !self.hybrids.contains_key(&n) {
      let i = self.hybrids.len() + 1;
      self.hybrids.insert(n, i);
    }
    s.push_str(&node.label);
    if node.hybrid {
      s.push_str("#H");
      s.push_str(&self.hybrids[&n].to_string());
    }
    s.push_str(&self.data(n, anc));
    s
  }

  /// Segment annotation and length of the branch from `anc` to `n`.
  fn data(&self, n: usize, anc: Option<usize>) -> String {
    let node = &self.arg.nodes[n];
    let segs: Vec<usize> = match anc {
      Some(a) => self.arg.edge_segments(a, n),
      None => (0..2).filter(|&c| node.has(c) && !node.is_root(c)).collect(),
    };
    let segs = if segs.is_empty() {
      (0..2).filter(|&c| node.has(c)).collect()
    } else {
      segs
    };
    let tau = segs.iter().find_map(|&c| node.tau[c]);
    let list = segs.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(",");
    match tau {
      Some(t) if matches!(node.anc[segs[0]], Anc::Node(_)) => format!("[&segments={{{list}}}]:{}", fmt_f64(t)),
      _ => format!("[&segments={{{list}}}]"),
    }
  }
}

/// One line per ARG node: `arg_node,tree1_node,tree2_node`, with a space for "none".
pub fn node_table(arg: &Arg) -> String {
  arg
    .nodes
    .iter()
    .zip(&arg.tree_nodes)
    .map(|(n, [a, b])| {
      format!(
        "{},{},{}",
        n.label,
        a.as_deref().unwrap_or(" "),
        b.as_deref().unwrap_or(" ")
      )
    })
    .collect::<Vec<_>>()
    .join("\n")
}
