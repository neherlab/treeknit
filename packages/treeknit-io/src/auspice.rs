//! Auspice v2 JSON with one MCC colouring per other tree, for tanglegram views.

use serde_json::{Map, Value, json};
use treeknit_core::mcc_map::{leaf_mcc_map, map_mccs};
use treeknit_core::{PairResult, Taxa, Tree};

fn key(a: &str, b: &str) -> String {
  let (x, y) = if a <= b { (a, b) } else { (b, a) };
  format!("mcc_{x}_{y}")
}

/// Auspice JSON for tree `i` of `trees`.
pub fn auspice_json(i: usize, trees: &[Tree], pairs: &[PairResult], taxa: &Taxa) -> Value {
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
  // Build nodes bottom-up so deep trees do not recurse.
  let mut div = vec![0.0; t.nodes.len()];
  for n in t.preorder() {
    if let Some(p) = t.parent(n) {
      div[n] = div[p] + t.node(n).branch_length.unwrap_or(0.0);
    }
  }
  let mut built: Vec<Option<Value>> = vec![None; t.nodes.len()];
  for n in t.postorder() {
    let mut attrs = Map::new();
    attrs.insert("div".into(), json!(div[n]));
    for (k, m) in &maps {
      let v = m[n].map_or("null".to_string(), |x| (x + 1).to_string());
      attrs.insert(k.clone(), json!({ "value": v }));
    }
    let mut node = json!({"name": t.name(n), "node_attrs": attrs, "branch_attrs": {}});
    if !t.is_leaf(n) {
      let children: Vec<Value> = t.children(n).iter().map(|&c| built[c].take().unwrap()).collect();
      node["children"] = Value::Array(children);
    }
    built[n] = Some(node);
  }
  json!({
      "version": "v2",
      "meta": {"updated": "", "colorings": colorings, "filters": [], "panels": ["tree"]},
      "tree": built[t.root].take().unwrap(),
  })
}
