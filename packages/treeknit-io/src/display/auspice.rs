//! The two trees of a pair as Auspice datasets, for the Auspice view of the web app.

use super::pair::pair_layout;
use super::{
  AuspiceBranchAttrs, AuspiceBranchLabels, AuspiceColoring, AuspiceColoringKind, AuspiceDataset,
  AuspiceDisplayDefaults, AuspiceMeta, AuspiceNode, AuspiceNodeAttrs, AuspicePair, AuspicePanel, AuspiceSchema,
  AuspiceValue, DrawTree, Scale, TreeVersion,
};
use crate::palette::palette;
use crate::run::RunResult;

/// Key of the MCC coloring and node attribute.
const MCC_KEY: &str = "mcc";
/// Key of the MCC branch label.
const MCC_LABEL: &str = "MCC";

/// The trees of pair `pair` (pipeline order) of `run` in `version` as Auspice datasets, with
/// `div` from the scale that `pair_view` shows for `scale`; `None` when the run has no such pair.
///
/// The trees are those of `pair_view`: the same node names, display order, MCCs, and color
/// slots. Each node with an MCC has the attribute `mcc` with the MCC's number (its index in
/// `MCCs.json` plus 1), and each reassortment branch the label `MCC` with that number. Both
/// datasets hold the whole MCC coloring, in the light theme's colors, because Auspice has no
/// dark theme.
pub fn auspice_view(run: &RunResult, pair: usize, version: TreeVersion, scale: Scale) -> Option<AuspicePair> {
  let (layout, slots) = pair_layout(run, pair, version)?;
  let scale = layout.shown_scale(scale);
  let colors = palette().light.mcc;
  let coloring = AuspiceColoring {
    key: MCC_KEY.to_owned(),
    title: MCC_LABEL.to_owned(),
    kind: AuspiceColoringKind::Categorical,
    scale: slots
      .iter()
      .enumerate()
      .map(|(mcc, &slot)| (mcc_number(mcc), colors[slot].clone()))
      .collect(),
  };
  let meta = AuspiceMeta {
    title: format!("TreeKnit: {} and {}", layout.left.label, layout.right.label),
    panels: vec![AuspicePanel::Tree],
    colorings: vec![coloring],
    filters: vec![MCC_KEY.to_owned()],
    display_defaults: AuspiceDisplayDefaults {
      color_by: MCC_KEY.to_owned(),
      branch_label: MCC_LABEL.to_owned(),
    },
  };
  let dataset = |tree: &DrawTree| AuspiceDataset {
    version: AuspiceSchema::V2,
    meta: meta.clone(),
    tree: auspice_tree(tree, scale),
  };
  Some(AuspicePair {
    scale,
    left: dataset(&layout.left),
    right: dataset(&layout.right),
  })
}

/// The number of MCC `mcc` (an index into the pair's MCCs) as Auspice shows it.
fn mcc_number(mcc: usize) -> String {
  (mcc + 1).to_string()
}

/// The nested Auspice tree of `tree`, built bottom-up so deep trees do not recurse.
fn auspice_tree(tree: &DrawTree, scale: Scale) -> AuspiceNode {
  let mut built: Vec<Option<AuspiceNode>> = vec![None; tree.nodes.len()];
  // Nodes are in preorder, so every child comes after its parent.
  for (i, n) in tree.nodes.iter().enumerate().rev() {
    let children = (!n.leaf).then(|| n.children.iter().filter_map(|&c| built[c].take()).collect());
    let number = n.mcc.map(mcc_number);
    built[i] = Some(AuspiceNode {
      name: n.name.clone(),
      node_attrs: AuspiceNodeAttrs {
        div: match scale {
          Scale::Div => n.x_div,
          Scale::Depth => n.x_depth,
        },
        mcc: number.clone().map(|value| AuspiceValue { value }),
      },
      branch_attrs: AuspiceBranchAttrs {
        labels: number.filter(|_| n.mcc_break).map(|mcc| AuspiceBranchLabels { mcc }),
      },
      children,
    });
  }
  #[expect(clippy::expect_used, reason = "a drawn tree has a root at index 0")]
  built.swap_remove(0).expect("the root")
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::analysis::{self, Settings, TreeText};
  use crate::display::{DrawNode, pair_view};
  use crate::run;
  use pretty_assertions::assert_eq;
  use serde_json::json;

  fn run_trees(trees: &[(&str, &str)]) -> RunResult {
    let texts: Vec<TreeText> = trees
      .iter()
      .map(|(label, newick)| TreeText {
        label: (*label).to_owned(),
        newick: (*newick).to_owned(),
      })
      .collect();
    let settings = Settings::default();
    let opts = analysis::options(&settings, texts.len(), false).unwrap();
    run::run(analysis::parse_trees(&texts).unwrap(), &opts, settings.seed, &|_| {})
  }

  /// A node with `div`, the MCC number `mcc`, the branch label `label`, and `children` (`None`
  /// for a leaf).
  fn node(
    name: &str,
    div: f64,
    mcc: Option<&str>,
    label: Option<&str>,
    children: Option<Vec<AuspiceNode>>,
  ) -> AuspiceNode {
    AuspiceNode {
      name: name.to_owned(),
      node_attrs: AuspiceNodeAttrs {
        div,
        mcc: mcc.map(|v| AuspiceValue { value: v.to_owned() }),
      },
      branch_attrs: AuspiceBranchAttrs {
        labels: label.map(|v| AuspiceBranchLabels { mcc: v.to_owned() }),
      },
      children,
    }
  }

  fn leaf(name: &str, div: f64, mcc: Option<&str>, label: Option<&str>) -> AuspiceNode {
    node(name, div, mcc, label, None)
  }

  fn dataset(meta: &AuspiceMeta, tree: AuspiceNode) -> AuspiceDataset {
    AuspiceDataset {
      version: AuspiceSchema::V2,
      meta: meta.clone(),
      tree,
    }
  }

  /// W moves into the clade of A and B, and P is in `ha` only, as the sister of D. The internal
  /// nodes are named.
  const HA: &str = "(((A:1,B:1)ab:1,(C:1,(D:1,P:1)dp:1)cdp:1)abcd:1,W:2)r1;";
  const NA: &str = "(((A:1,B:1)ab:1,W:1)abw:1,(C:1,D:2)cd:3)r2;";

  /// The `meta` of the datasets of `HA` and `NA`.
  fn meta() -> AuspiceMeta {
    let colors = palette().light.mcc;
    AuspiceMeta {
      title: "TreeKnit: ha and na".to_owned(),
      panels: vec![AuspicePanel::Tree],
      // Oracle: the two MCCs of `MCCS` have the same size, so MCC 1 is colored first and takes
      // slot 0, and its neighbor MCC 2 takes slot 1.
      colorings: vec![AuspiceColoring {
        key: "mcc".to_owned(),
        title: "MCC".to_owned(),
        kind: AuspiceColoringKind::Categorical,
        scale: vec![("1".to_owned(), colors[0].clone()), ("2".to_owned(), colors[1].clone())],
      }],
      filters: vec!["mcc".to_owned()],
      display_defaults: AuspiceDisplayDefaults {
        color_by: "mcc".to_owned(),
        branch_label: "MCC".to_owned(),
      },
    }
  }

  /// The MCCs of `HA` and `NA`, in the order of `MCCs.json`. TreeKnit explains the pair with one
  /// reassortment, which moves the clade of C and D; P attaches next to D, so it joins MCC 2.
  const MCCS: [&[&str]; 2] = [&["A", "B", "W"], &["C", "D", "P"]];

  /// The right tree with the `div` of its nodes in the order r2, cd, C, D, abw, W, ab, A, B.
  fn right_tree(div: [f64; 9]) -> AuspiceNode {
    let (one, two) = (Some("1"), Some("2"));
    node(
      "r2",
      div[0],
      None,
      None,
      Some(vec![
        node(
          "cd",
          div[1],
          two,
          two,
          Some(vec![leaf("C", div[2], two, None), leaf("D", div[3], two, None)]),
        ),
        node(
          "abw",
          div[4],
          one,
          one,
          Some(vec![
            leaf("W", div[5], one, None),
            node(
              "ab",
              div[6],
              one,
              None,
              Some(vec![leaf("A", div[7], one, None), leaf("B", div[8], one, None)]),
            ),
          ]),
        ),
      ]),
    )
  }

  #[test]
  fn auspice_view_of_the_input_version_with_divergence() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    let mccs: Vec<Vec<String>> = r.pairs[0].mccs.iter().map(|m| r.taxa.names_of(m)).collect();
    assert_eq!(
      MCCS.map(|m| m.join(",")).to_vec(),
      mccs.iter().map(|m| m.join(",")).collect::<Vec<_>>()
    );
    // Oracle: a node has the MCC that spans it and its leaves; ha's root is in MCC 1, na's root
    // joins both MCCs and has none. A branch label marks each node with an MCC whose parent has
    // another MCC or none. `div` sums the branch lengths. The children are in the display order
    // of `pair_view`. P is in ha only, so the right tree lacks it.
    let (one, two) = (Some("1"), Some("2"));
    let left = node(
      "r1",
      0.0,
      one,
      None,
      Some(vec![
        leaf("W", 2.0, one, None),
        node(
          "abcd",
          1.0,
          one,
          None,
          Some(vec![
            node(
              "ab",
              2.0,
              one,
              None,
              Some(vec![leaf("A", 3.0, one, None), leaf("B", 3.0, one, None)]),
            ),
            node(
              "cdp",
              2.0,
              two,
              two,
              Some(vec![
                leaf("C", 3.0, two, None),
                node(
                  "dp",
                  3.0,
                  two,
                  None,
                  Some(vec![leaf("D", 4.0, two, None), leaf("P", 4.0, two, None)]),
                ),
              ]),
            ),
          ]),
        ),
      ]),
    );
    let right = right_tree([0.0, 3.0, 4.0, 5.0, 1.0, 2.0, 2.0, 3.0, 3.0]);
    let expected = AuspicePair {
      scale: Scale::Div,
      left: dataset(&meta(), left),
      right: dataset(&meta(), right),
    };
    assert_eq!(Some(expected), auspice_view(&r, 0, TreeVersion::Input, Scale::Div));
  }

  #[test]
  fn auspice_view_of_the_right_tree_as_a_cladogram() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    // Oracle: na has height 3 (r2, abw, ab, A), so its leaves are at 3; abw has height 2 (at 1),
    // ab and cd height 1 (at 2).
    let right = right_tree([0.0, 2.0, 3.0, 3.0, 1.0, 3.0, 2.0, 3.0, 3.0]);
    let view = auspice_view(&r, 0, TreeVersion::Input, Scale::Depth).unwrap();
    assert_eq!(dataset(&meta(), right), view.right);
  }

  #[test]
  fn auspice_view_serializes_with_the_auspice_field_names() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    let view = auspice_view(&r, 0, TreeVersion::Input, Scale::Div).unwrap();
    let value = serde_json::to_value(&view.left).unwrap();
    let colors = palette().light.mcc;
    assert_eq!(
      json!({
        "title": "TreeKnit: ha and na",
        "panels": ["tree"],
        "colorings": [{"key": "mcc", "title": "MCC", "type": "categorical", "scale": [["1", colors[0]], ["2", colors[1]]]}],
        "filters": ["mcc"],
        "display_defaults": {"color_by": "mcc", "branch_label": "MCC"},
      }),
      value["meta"]
    );
    assert_eq!(json!("v2"), value["version"]);
    // The right root has no MCC and no label; the branch above cd starts MCC 2; a leaf has no
    // children.
    let right = serde_json::to_value(&view.right.tree).unwrap();
    assert_eq!(
      json!({"node_attrs": {"div": 0.0}, "branch_attrs": {}}),
      json!({"node_attrs": right["node_attrs"], "branch_attrs": right["branch_attrs"]})
    );
    assert_eq!(
      json!({"name": "C", "node_attrs": {"div": 4.0, "mcc": {"value": "2"}}, "branch_attrs": {}}),
      right["children"][0]["children"][0]
    );
    assert_eq!(
      json!({"div": 3.0, "mcc": {"value": "2"}}),
      right["children"][0]["node_attrs"]
    );
    assert_eq!(json!({"labels": {"MCC": "2"}}), right["children"][0]["branch_attrs"]);
  }

  /// The nodes of `tree` in preorder, each as its name and MCC number.
  fn preorder(tree: &AuspiceNode) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    let mut stack = vec![tree];
    while let Some(n) = stack.pop() {
      out.push((n.name.clone(), n.node_attrs.mcc.as_ref().map(|m| m.value.clone())));
      stack.extend(n.children.iter().flatten().rev());
    }
    out
  }

  fn draw_preorder(nodes: &[DrawNode]) -> Vec<(String, Option<String>)> {
    nodes
      .iter()
      .map(|n| (n.name.clone(), n.mcc.map(|m| (m + 1).to_string())))
      .collect()
  }

  #[test]
  fn auspice_view_of_the_two_tree_example_has_the_trees_of_the_pair_view() {
    let r = run_trees(&[("ha", "((A,B),(C,(D,X)));"), ("na", "((A,(B,X)),(C,D));")]);
    for version in [TreeVersion::Input, TreeVersion::Resolved, TreeVersion::Imputed] {
      let pair = pair_view(&r, 0, version, Scale::Div).unwrap();
      let view = auspice_view(&r, 0, version, Scale::Div).unwrap();
      assert_eq!(draw_preorder(&pair.left.nodes), preorder(&view.left.tree));
      assert_eq!(draw_preorder(&pair.right.nodes), preorder(&view.right.tree));
      // The trees have no branch lengths, so both views show the cladogram.
      assert_eq!((Scale::Depth, Scale::Depth), (pair.scale, view.scale));
    }
  }

  #[test]
  fn auspice_view_of_an_unknown_pair_is_none() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    assert!(auspice_view(&r, 1, TreeVersion::Resolved, Scale::Div).is_none());
  }
}
