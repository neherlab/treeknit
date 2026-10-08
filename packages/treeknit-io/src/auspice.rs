//! Auspice v2 JSON with one MCC colouring per other tree, for tanglegram views.

use serde_json::{Map, Value, json};
use treeknit_core::mcc_map::{leaf_mcc_map, map_mccs};
use treeknit_core::{NodeId, PairResult, Taxa, Tree};

fn key(a: &str, b: &str) -> String {
  let (x, y) = if a <= b { (a, b) } else { (b, a) };
  format!("mcc_{x}_{y}")
}

/// Auspice JSON text for tree `i` of `trees`, compact as TreeKnit.jl writes it.
///
/// The nodes nest as deep as the tree, and `serde_json` serializes, copies, and drops a nested
/// `Value` by recursion, which overflows the stack on deep trees. So each node is a flat object
/// and the nesting is written with an explicit stack. Compact text also keeps the size linear in
/// the depth, where indentation would grow with its square.
pub fn auspice_json(i: usize, trees: &[Tree], pairs: &[PairResult], taxa: &Taxa) -> String {
  let t = &trees[i];
  let mut maps = Vec::new();
  for p in pairs.iter().filter(|p| p.i == i || p.j == i) {
    let other = if p.i == i { p.j } else { p.i };
    maps.push((
      key(&t.label, &trees[other].label),
      map_mccs(t, &leaf_mcc_map(&p.mccs, taxa.len())),
    ));
  }
  let colorings: Vec<Value> = maps
    .iter()
    .map(|(k, _)| json!({"key": k, "title": k, "type": "ordinal"}))
    .collect();
  let mut div = vec![0.0; t.nodes.len()];
  for n in t.preorder() {
    if let Some(p) = t.parent(n) {
      div[n] = div[p] + t.node(n).branch_length.unwrap_or(0.0);
    }
  }
  let fields = |n: NodeId| {
    let mut attrs = Map::new();
    attrs.insert("div".into(), json!(div[n]));
    for (k, m) in &maps {
      let v = m[n].map_or_else(|| "null".to_owned(), |x| (x + 1).to_string());
      attrs.insert(k.clone(), json!({ "value": v }));
    }
    Map::from_iter([
      ("name".to_owned(), json!(t.name(n))),
      ("node_attrs".to_owned(), Value::Object(attrs)),
      ("branch_attrs".to_owned(), json!({})),
    ])
  };
  let header = Map::from_iter([
    ("version".to_owned(), json!("v2")),
    (
      "meta".to_owned(),
      json!({"updated": "", "colorings": colorings, "filters": [], "panels": ["tree"]}),
    ),
  ]);
  let mut out = String::from("{");
  push_members(&mut out, &header);
  out.push_str(",\"tree\":");
  push_tree(&mut out, t, fields);
  out.push('}');
  out
}

/// Append the subtree of `t` below its root as nested objects, each node with the members
/// `fields` gives it and, for an internal node, a last member `children`.
fn push_tree(out: &mut String, t: &Tree, fields: impl Fn(NodeId) -> Map<String, Value>) {
  let mut stack = vec![Step::Open(t.root)];
  while let Some(step) = stack.pop() {
    match step {
      Step::Open(n) => {
        out.push('{');
        push_members(out, &fields(n));
        if t.is_leaf(n) {
          out.push('}');
        } else {
          out.push_str(",\"children\":[");
          stack.push(Step::Close);
          for (k, &c) in t.children(n).iter().rev().enumerate() {
            if k > 0 {
              stack.push(Step::Separator);
            }
            stack.push(Step::Open(c));
          }
        }
      },
      Step::Separator => out.push(','),
      Step::Close => out.push_str("]}"),
    }
  }
}

/// Append `members` to an object, separated by commas.
fn push_members(out: &mut String, members: &Map<String, Value>) {
  for (k, (name, value)) in members.iter().enumerate() {
    if k > 0 {
      out.push(',');
    }
    out.push_str(&Value::from(name.as_str()).to_string());
    out.push(':');
    out.push_str(&value.to_string());
  }
}

/// A step of `push_tree`.
enum Step {
  /// Write node `n`.
  Open(NodeId),
  /// Write the comma between two children.
  Separator,
  /// Close the `children` array and the object of a node.
  Close,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::analysis::{self, ParsedTrees};
  use crate::test_support::{run_trees, texts};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use treeknit_testing::{DEEP, caterpillar_newick, numbered, on_small_stack};

  #[rustfmt::skip]
  #[rstest]
  #[case::two_trees(  &[("ha", "((A:1,B:2):0.5,(C:1,(D:1,X:1)));"), ("na", "((A,(B,X)),(C,D));")])]
  #[case::three_trees(&[("a", "((A,B),(C,D));"), ("b", "((A,C),(B,D));"), ("c", "((A,D),(B,C));")])]
  #[case::escaped(    &[("a", "(('q\"x',B),'tab\tC');"), ("b", "(('q\"x','tab\tC'),B);")])]
  #[trace]
  fn auspice_json_is_compact(#[case] trees: &[(&str, &str)]) {
    let run = run_trees(trees);
    let text = auspice_json(0, &run.trees, &run.pairs, &run.taxa);
    let parsed: Value = serde_json::from_str(&text).unwrap();
    // Oracle: the compact writer of `serde_json`, applied to the value the text holds.
    assert_eq!(parsed.to_string(), text);
  }

  #[test]
  fn auspice_json_writes_a_deep_tree() {
    let newick = format!("{};", caterpillar_newick(&numbered("L", DEEP)));
    let ParsedTrees { trees, taxa } = analysis::parse_trees(&texts(&[("a", &newick), ("b", &newick)])).unwrap();
    let text = on_small_stack(|| auspice_json(0, &trees, &[], &taxa));
    // Oracle: the caterpillar has DEEP - 1 internal nodes, one `children` member each, nested in
    // a chain, so the text ends by closing each of them and then the dataset.
    let end = format!("{}}}", "]}".repeat(DEEP - 1));
    assert_eq!(
      (DEEP - 1, true),
      (text.matches("\"children\"").count(), text.ends_with(&end))
    );
  }
}
