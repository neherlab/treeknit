//! The two trees of a pair as Auspice datasets, for the Auspice view of the web app.

use crate::display::pair::{mcc_infos, pair_layout};
use crate::display::{
  AuspiceBranchAttrs, AuspiceBranchLabels, AuspiceColoring, AuspiceColoringKind, AuspiceDataset,
  AuspiceDisplayDefaults, AuspiceMccRoot, AuspiceMeta, AuspiceNode, AuspiceNodeAttrs, AuspiceNumber, AuspicePair,
  AuspicePanel, AuspiceSchema, AuspiceSharing, AuspiceShown, AuspiceShownTree, AuspiceTrees, AuspiceValue, DrawTree,
  MCC_SLOTS, MccInfo, Scale, TreeVersion,
};
use crate::palette::palette;
use crate::run::RunResult;
use std::collections::BTreeSet;

/// Key of the MCC coloring and node attribute.
const MCC_KEY: &str = "mcc";
/// Key of the MCC branch label.
const MCC_LABEL: &str = "MCC";
/// Key of the coloring of the largest MCCs.
const LARGEST_MCC_KEY: &str = "largest_mcc";
/// Key of the continuous coloring by MCC size.
const MCC_SIZE_KEY: &str = "mcc_size";
/// Key of the reassortment branch coloring.
const REASSORTMENT_KEY: &str = "reassortment";
/// Key of the imputed leaf coloring.
const IMPUTED_KEY: &str = "imputed";
/// Key of the added node coloring.
const ADDED_KEY: &str = "added";
/// Key of the coloring of leaves that one tree of the pair lacks.
const ONE_TREE_KEY: &str = "one_tree";
/// Key of the attachment coloring.
const ATTACHMENT_KEY: &str = "attachment";
/// Value of `largest_mcc` for the MCCs outside the largest ones.
const OTHER_MCC: &str = "Other";
const YES: &str = "Yes";
const NO: &str = "No";
const AMBIGUOUS: &str = "Ambiguous";
const UNAMBIGUOUS: &str = "Unambiguous";

/// The trees of pair `pair` (pipeline order) of `run` in `version` as Auspice datasets, with
/// `div` from the scale that `pair_view` shows for `scale`; `None` when the run has no such pair.
///
/// The trees are those of `pair_view`: the same node names, display order, MCCs, and color
/// slots. Each node with an MCC has the attribute `mcc` with the MCC's name ("MCC 1" for the first
/// MCC of `MCCs.json`), and each reassortment branch the label `MCC` with its number. The colorings
/// also show the 8 largest MCCs, the MCC size, the reassortment branches, the imputed leaves, the
/// added nodes, the leaves of one tree only, and the attachment of the leaves; each categorical
/// coloring is a filter. Both datasets hold every coloring in the light theme's colors, because
/// Auspice has no dark theme. They turn off the genetic diversity download of Auspice, because
/// they have no sequences. The `auspice_<label>.json` files of the command line keep their own
/// fields.
pub fn auspice_view(run: &RunResult, pair: usize, version: TreeVersion, scale: Scale) -> Option<AuspicePair> {
  let (layout, slots) = pair_layout(run, pair, version)?;
  let mccs = mcc_infos(run, &run.pairs[pair], slots, [&layout.left, &layout.right]);
  let scale = layout.shown_scale(scale);
  let largest = largest_mccs(&mccs);
  let meta = AuspiceMeta {
    title: format!("TreeKnit: {} and {}", layout.left.label, layout.right.label),
    panels: vec![AuspicePanel::Tree],
    colorings: colorings(&mccs, &largest, &layout.left.label, &layout.right.label),
    filters: [
      MCC_KEY,
      LARGEST_MCC_KEY,
      REASSORTMENT_KEY,
      IMPUTED_KEY,
      ADDED_KEY,
      ONE_TREE_KEY,
      ATTACHMENT_KEY,
    ]
    .map(str::to_owned)
    .to_vec(),
    display_defaults: AuspiceDisplayDefaults {
      color_by: MCC_KEY.to_owned(),
      branch_label: MCC_LABEL.to_owned(),
    },
    sharing: AuspiceSharing { entropy: false },
  };
  let names: [BTreeSet<&str>; 2] =
    [&layout.left, &layout.right].map(|tree| tree.leaves().map(|n| n.name.as_str()).collect());
  let context = |other: usize| TreeContext {
    scale,
    mccs: &mccs,
    largest: &largest,
    other_leaves: &names[other],
  };
  let dataset = |tree: &DrawTree, other: usize| AuspiceDataset {
    version: AuspiceSchema::V2,
    meta: meta.clone(),
    tree: auspice_tree(tree, &context(other)),
  };
  Some(AuspicePair {
    scale,
    axis_title: match scale {
      Scale::Div => "Divergence",
      Scale::Depth => "Depth",
    }
    .to_owned(),
    mcc_roots: (0..mccs.len())
      .map(|mcc| AuspiceMccRoot {
        left: mcc_root(&layout.left, mcc),
        right: mcc_root(&layout.right, mcc),
      })
      .collect(),
    mcc_key: MCC_KEY.to_owned(),
    mcc_values: mccs.iter().map(|m| m.name.clone()).collect(),
    shown: shown(run, pair),
    left: dataset(&layout.left, 1),
    right: dataset(&layout.right, 0),
  })
}

/// The trees of pair `pair` that each choice of `AuspiceTrees` shows.
fn shown(run: &RunResult, pair: usize) -> AuspiceShown {
  let p = &run.pairs[pair];
  let trees = |choice: AuspiceTrees| {
    [(true, p.i), (false, p.j)]
      .into_iter()
      .filter(|&(left, _)| choice.shows(left))
      .map(|(_, tree)| AuspiceShownTree {
        tree,
        label: run.trees[tree].label.clone(),
      })
      .collect()
  };
  AuspiceShown {
    both: trees(AuspiceTrees::Both),
    left: trees(AuspiceTrees::Left),
    right: trees(AuspiceTrees::Right),
  }
}

/// The number of MCC `mcc` as its branch label shows it.
fn mcc_number(mcc: usize) -> String {
  (mcc + 1).to_string()
}

/// The indices of the largest MCCs, at most `MCC_SLOTS` of them, in the order of `MccInfo.rank`.
/// The rank of an MCC in the result picks its color slot.
fn largest_mccs(mccs: &[MccInfo]) -> Vec<usize> {
  let mut order: Vec<usize> = (0..mccs.len()).collect();
  order.sort_by_key(|&i| mccs[i].rank);
  order.truncate(MCC_SLOTS);
  order
}

/// The colorings of the datasets: the MCCs (titled with the pair, because one tree shown alone is
/// colored by the MCCs of a pair), the largest MCCs, the MCC size, and the categories of nodes.
fn colorings(mccs: &[MccInfo], largest: &[usize], left: &str, right: &str) -> Vec<AuspiceColoring> {
  let colors = &palette().light;
  let categorical = |key: &str, title: &str, scale: Vec<(String, String)>| AuspiceColoring {
    key: key.to_owned(),
    title: title.to_owned(),
    kind: AuspiceColoringKind::Categorical,
    scale: Some(scale),
  };
  let yes_no = |key: &str, title: &str| {
    categorical(
      key,
      title,
      vec![
        (YES.to_owned(), colors.signal.clone()),
        (NO.to_owned(), colors.no_mcc.clone()),
      ],
    )
  };
  let mut largest_scale: Vec<(String, String)> = largest
    .iter()
    .enumerate()
    .map(|(rank, &mcc)| (mccs[mcc].name.clone(), colors.mcc[rank].clone()))
    .collect();
  if mccs.len() > largest.len() {
    largest_scale.push((OTHER_MCC.to_owned(), colors.no_mcc.clone()));
  }
  vec![
    categorical(
      MCC_KEY,
      &format!("MCC ({left} and {right})"),
      mccs
        .iter()
        .map(|m| (m.name.clone(), colors.mcc[m.slot].clone()))
        .collect(),
    ),
    categorical(LARGEST_MCC_KEY, "Largest MCCs", largest_scale),
    AuspiceColoring {
      key: MCC_SIZE_KEY.to_owned(),
      title: "MCC size".to_owned(),
      kind: AuspiceColoringKind::Continuous,
      scale: None,
    },
    yes_no(REASSORTMENT_KEY, "Reassortment branch"),
    yes_no(IMPUTED_KEY, "Imputed leaf"),
    yes_no(ADDED_KEY, "Added node"),
    yes_no(ONE_TREE_KEY, "Leaf in one tree only"),
    categorical(
      ATTACHMENT_KEY,
      "Attachment",
      vec![
        (AMBIGUOUS.to_owned(), colors.signal.clone()),
        (UNAMBIGUOUS.to_owned(), colors.no_mcc.clone()),
      ],
    ),
  ]
}

/// The name of the node of `tree` where MCC `mcc` starts: the node below its reassortment branch,
/// or the root when the MCC holds the root; `None` when no node of `tree` has the MCC.
fn mcc_root(tree: &DrawTree, mcc: usize) -> Option<String> {
  tree
    .nodes
    .iter()
    .find(|n| n.mcc == Some(mcc) && (n.mcc_break || n.parent.is_none()))
    .map(|n| n.name.clone())
}

/// What the nodes of one tree need besides the tree: the shown scale, the MCCs, the largest MCCs,
/// and the leaves of the other tree of the pair.
struct TreeContext<'a> {
  scale: Scale,
  mccs: &'a [MccInfo],
  largest: &'a [usize],
  other_leaves: &'a BTreeSet<&'a str>,
}

/// The nested Auspice tree of `tree`, built bottom-up so deep trees do not recurse.
fn auspice_tree(tree: &DrawTree, context: &TreeContext) -> AuspiceNode {
  let value = |text: &str| Some(AuspiceValue { value: text.to_owned() });
  let yes_no = |yes: bool| value(if yes { YES } else { NO });
  let mut built: Vec<Option<AuspiceNode>> = vec![None; tree.nodes.len()];
  // Nodes are in preorder, so every child comes after its parent.
  for (i, n) in tree.nodes.iter().enumerate().rev() {
    let children = (!n.leaf).then(|| n.children.iter().filter_map(|&c| built[c].take()).collect());
    let mcc = n.mcc.and_then(|m| context.mccs.get(m));
    built[i] = Some(AuspiceNode {
      name: n.name.clone(),
      node_attrs: AuspiceNodeAttrs {
        div: context.scale.x(n.x_div, n.x_depth),
        mcc: mcc.and_then(|m| value(&m.name)),
        largest_mcc: mcc.and_then(|m| {
          if context.largest.contains(&m.index) {
            value(&m.name)
          } else {
            value(OTHER_MCC)
          }
        }),
        mcc_size: mcc.map(|m| AuspiceNumber { value: m.size }),
        reassortment: n.parent.and_then(|_| yes_no(n.mcc_break)),
        imputed: n.leaf.then(|| yes_no(n.imputed)).flatten(),
        added: (!n.leaf).then(|| yes_no(n.added)).flatten(),
        one_tree: n
          .leaf
          .then(|| yes_no(!context.other_leaves.contains(n.name.as_str())))
          .flatten(),
        attachment: mcc.filter(|_| n.leaf).and_then(|m| {
          value(if m.ambiguous_leaves.contains(&n.name) {
            AMBIGUOUS
          } else {
            UNAMBIGUOUS
          })
        }),
      },
      branch_attrs: AuspiceBranchAttrs {
        labels: n
          .mcc
          .filter(|_| n.mcc_break)
          .map(|m| AuspiceBranchLabels { mcc: mcc_number(m) }),
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
  use crate::test_support::run_trees;

  use crate::display::{DrawNode, pair_view};

  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::json;

  fn value(text: &str) -> Option<AuspiceValue> {
    Some(AuspiceValue { value: text.to_owned() })
  }

  /// The attributes of a node of `HA` or `NA` with `div` and the MCC number `mcc`: both MCCs of
  /// the example have 3 leaves and are among the 8 largest.
  fn attrs(div: f64, mcc: Option<&str>) -> AuspiceNodeAttrs {
    let name = mcc.map(|m| format!("MCC {m}"));
    AuspiceNodeAttrs {
      div,
      mcc: name.as_deref().and_then(value),
      largest_mcc: name.as_deref().and_then(value),
      mcc_size: mcc.map(|_| AuspiceNumber { value: 3 }),
      reassortment: None,
      imputed: None,
      added: None,
      one_tree: None,
      attachment: None,
    }
  }

  fn branch(label: Option<&str>) -> AuspiceBranchAttrs {
    AuspiceBranchAttrs {
      labels: label.map(|v| AuspiceBranchLabels { mcc: v.to_owned() }),
    }
  }

  /// A root with `div`, the MCC number `mcc`, and `children`: it has no branch above it.
  fn root(name: &str, div: f64, mcc: Option<&str>, children: Vec<AuspiceNode>) -> AuspiceNode {
    AuspiceNode {
      name: name.to_owned(),
      node_attrs: AuspiceNodeAttrs {
        added: value("No"),
        ..attrs(div, mcc)
      },
      branch_attrs: branch(None),
      children: Some(children),
    }
  }

  /// An internal node of the input trees, so never added; `label` marks a reassortment branch.
  fn node(name: &str, div: f64, mcc: Option<&str>, label: Option<&str>, children: Vec<AuspiceNode>) -> AuspiceNode {
    AuspiceNode {
      name: name.to_owned(),
      node_attrs: AuspiceNodeAttrs {
        reassortment: value(if label.is_some() { "Yes" } else { "No" }),
        added: value("No"),
        ..attrs(div, mcc)
      },
      branch_attrs: branch(label),
      children: Some(children),
    }
  }

  /// A leaf of the input trees, in both trees and attached unambiguously.
  fn leaf(name: &str, div: f64, mcc: Option<&str>, label: Option<&str>) -> AuspiceNode {
    AuspiceNode {
      name: name.to_owned(),
      node_attrs: AuspiceNodeAttrs {
        reassortment: value(if label.is_some() { "Yes" } else { "No" }),
        imputed: value("No"),
        one_tree: value("No"),
        attachment: mcc.and_then(|_| value("Unambiguous")),
        ..attrs(div, mcc)
      },
      branch_attrs: branch(label),
      children: None,
    }
  }

  /// `leaf` for a leaf that the other tree of the pair lacks.
  fn only_here(leaf: AuspiceNode) -> AuspiceNode {
    AuspiceNode {
      node_attrs: AuspiceNodeAttrs {
        one_tree: value("Yes"),
        ..leaf.node_attrs
      },
      ..leaf
    }
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
    let colors = palette().light;
    let yes_no = vec![
      ("Yes".to_owned(), colors.signal.clone()),
      ("No".to_owned(), colors.no_mcc.clone()),
    ];
    let categorical = |key: &str, title: &str, scale: Vec<(String, String)>| AuspiceColoring {
      key: key.to_owned(),
      title: title.to_owned(),
      kind: AuspiceColoringKind::Categorical,
      scale: Some(scale),
    };
    AuspiceMeta {
      title: "TreeKnit: ha and na".to_owned(),
      panels: vec![AuspicePanel::Tree],
      colorings: vec![
        // Oracle: the two MCCs of `MCCS` have the same size, so MCC 1 is colored first and takes
        // slot 0, and its neighbor MCC 2 takes slot 1.
        categorical(
          "mcc",
          "MCC (ha and na)",
          vec![
            ("MCC 1".to_owned(), colors.mcc[0].clone()),
            ("MCC 2".to_owned(), colors.mcc[1].clone()),
          ],
        ),
        // Oracle: both MCCs are among the 8 largest; equal sizes keep the order of `MCCs.json`.
        categorical(
          "largest_mcc",
          "Largest MCCs",
          vec![
            ("MCC 1".to_owned(), colors.mcc[0].clone()),
            ("MCC 2".to_owned(), colors.mcc[1].clone()),
          ],
        ),
        AuspiceColoring {
          key: "mcc_size".to_owned(),
          title: "MCC size".to_owned(),
          kind: AuspiceColoringKind::Continuous,
          scale: None,
        },
        categorical("reassortment", "Reassortment branch", yes_no.clone()),
        categorical("imputed", "Imputed leaf", yes_no.clone()),
        categorical("added", "Added node", yes_no.clone()),
        categorical("one_tree", "Leaf in one tree only", yes_no),
        categorical(
          "attachment",
          "Attachment",
          vec![
            ("Ambiguous".to_owned(), colors.signal.clone()),
            ("Unambiguous".to_owned(), colors.no_mcc),
          ],
        ),
      ],
      filters: [
        "mcc",
        "largest_mcc",
        "reassortment",
        "imputed",
        "added",
        "one_tree",
        "attachment",
      ]
      .map(str::to_owned)
      .to_vec(),
      display_defaults: AuspiceDisplayDefaults {
        color_by: "mcc".to_owned(),
        branch_label: "MCC".to_owned(),
      },
      sharing: AuspiceSharing { entropy: false },
    }
  }

  /// The MCCs of `HA` and `NA`, in the order of `MCCs.json`. TreeKnit explains the pair with one
  /// reassortment, which moves the clade of C and D; P attaches next to D, so it joins MCC 2.
  const MCCS: [&[&str]; 2] = [&["A", "B", "W"], &["C", "D", "P"]];

  /// The right tree with the `div` of its nodes in the order r2, cd, C, D, abw, W, ab, A, B.
  fn right_tree(div: [f64; 9]) -> AuspiceNode {
    let (one, two) = (Some("1"), Some("2"));
    root(
      "r2",
      div[0],
      None,
      vec![
        node(
          "cd",
          div[1],
          two,
          two,
          vec![leaf("C", div[2], two, None), leaf("D", div[3], two, None)],
        ),
        node(
          "abw",
          div[4],
          one,
          one,
          vec![
            leaf("W", div[5], one, None),
            node(
              "ab",
              div[6],
              one,
              None,
              vec![leaf("A", div[7], one, None), leaf("B", div[8], one, None)],
            ),
          ],
        ),
      ],
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
    // joins both MCCs and has none. A branch label, and the reassortment attribute, mark each
    // node with an MCC whose parent has another MCC or none. `div` sums the branch lengths. The
    // children are in the display order of `pair_view`. P is in ha only, so the right tree lacks
    // it; P hangs next to D inside MCC 2, so its attachment is unambiguous. The input trees have
    // no imputed leaves and no added nodes. MCC 1 starts at ha's root and at na's abw, MCC 2 at
    // ha's cdp and na's cd.
    let (one, two) = (Some("1"), Some("2"));
    let left = root(
      "r1",
      0.0,
      one,
      vec![
        leaf("W", 2.0, one, None),
        node(
          "abcd",
          1.0,
          one,
          None,
          vec![
            node(
              "ab",
              2.0,
              one,
              None,
              vec![leaf("A", 3.0, one, None), leaf("B", 3.0, one, None)],
            ),
            node(
              "cdp",
              2.0,
              two,
              two,
              vec![
                leaf("C", 3.0, two, None),
                node(
                  "dp",
                  3.0,
                  two,
                  None,
                  vec![leaf("D", 4.0, two, None), only_here(leaf("P", 4.0, two, None))],
                ),
              ],
            ),
          ],
        ),
      ],
    );
    let right = right_tree([0.0, 3.0, 4.0, 5.0, 1.0, 2.0, 2.0, 3.0, 3.0]);
    let expected = AuspicePair {
      scale: Scale::Div,
      axis_title: "Divergence".to_owned(),
      mcc_roots: vec![
        AuspiceMccRoot {
          left: Some("r1".to_owned()),
          right: Some("abw".to_owned()),
        },
        AuspiceMccRoot {
          left: Some("cdp".to_owned()),
          right: Some("cd".to_owned()),
        },
      ],
      mcc_key: "mcc".to_owned(),
      mcc_values: vec!["MCC 1".to_owned(), "MCC 2".to_owned()],
      // Oracle: ha is the first tree of the run and na the second; one tree alone is the main tree.
      shown: AuspiceShown {
        both: vec![shown_tree(0, "ha"), shown_tree(1, "na")],
        left: vec![shown_tree(0, "ha")],
        right: vec![shown_tree(1, "na")],
      },
      left: dataset(&meta(), left),
      right: dataset(&meta(), right),
    };
    assert_eq!(Some(expected), auspice_view(&r, 0, TreeVersion::Input, Scale::Div));
  }

  #[test]
  fn auspice_view_of_the_right_tree_as_a_cladogram() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    // Oracle: na has height 3 (r2, abw, ab, A), so its leaves are at 3; abw has height 2 (at 1),
    // ab and cd height 1 (at 2). The axis names the depth.
    let right = right_tree([0.0, 2.0, 3.0, 3.0, 1.0, 3.0, 2.0, 3.0, 3.0]);
    let view = auspice_view(&r, 0, TreeVersion::Input, Scale::Depth).unwrap();
    assert_eq!(
      (dataset(&meta(), right), "Depth"),
      (view.right, view.axis_title.as_str())
    );
  }

  fn shown_tree(tree: usize, label: &str) -> AuspiceShownTree {
    AuspiceShownTree {
      tree,
      label: label.to_owned(),
    }
  }

  /// A star of `n` cherries: cherry `i` holds the leaves `a{i}` and `b{i}` in one tree and is
  /// broken up in the other, which pairs `b{i}` with `a{i+1}`, so the pair has many small MCCs.
  fn cherries(n: usize, shift: bool) -> String {
    let cherry = |i: usize| {
      if shift {
        format!("(b{i},a{})", (i + 1) % n)
      } else {
        format!("(a{i},b{i})")
      }
    };
    format!("({});", (0..n).map(cherry).collect::<Vec<_>>().join(","))
  }

  #[test]
  fn auspice_view_keeps_the_eight_largest_mccs_and_groups_the_others() {
    let r = run_trees(&[("x", &cherries(12, false)), ("y", &cherries(12, true))]);
    let view = auspice_view(&r, 0, TreeVersion::Resolved, Scale::Div).unwrap();
    let (layout, _) = pair_layout(&r, 0, TreeVersion::Resolved).unwrap();
    let mccs = mcc_infos(
      &r,
      &r.pairs[0],
      &[0; 64][..r.pairs[0].mccs.len()],
      [&layout.left, &layout.right],
    );
    // Oracle: the 8 largest MCCs by leaf count, ties in the order of `MCCs.json`, keep their name
    // and take the 8 palette colors in rank order; the rest share "Other" in the no-MCC color.
    let mut by_size: Vec<&MccInfo> = mccs.iter().collect();
    by_size.sort_by(|a, b| b.size.cmp(&a.size).then(a.index.cmp(&b.index)));
    let colors = palette().light;
    let mut expected: Vec<(String, String)> = by_size
      .iter()
      .take(8)
      .enumerate()
      .map(|(rank, m)| (format!("MCC {}", m.index + 1), colors.mcc[rank].clone()))
      .collect();
    expected.push(("Other".to_owned(), colors.no_mcc));
    let largest = view
      .left
      .meta
      .colorings
      .iter()
      .find(|c| c.key == "largest_mcc")
      .unwrap();
    assert!(mccs.len() > 8, "the example has more than 8 MCCs");
    assert_eq!(Some(&expected), largest.scale.as_ref());
    // Every leaf outside the 8 largest MCCs has the value "Other".
    let kept: BTreeSet<String> = expected.iter().map(|(name, _)| name.clone()).collect();
    let mut stack = vec![&view.left.tree];
    while let Some(n) = stack.pop() {
      if let (Some(mcc), Some(largest)) = (&n.node_attrs.mcc, &n.node_attrs.largest_mcc) {
        let want = if kept.contains(&mcc.value) {
          mcc.value.as_str()
        } else {
          "Other"
        };
        assert_eq!(want, largest.value, "{}", n.name);
      }
      stack.extend(n.children.iter().flatten());
    }
  }

  #[test]
  fn auspice_view_serializes_with_the_auspice_field_names() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    let view = auspice_view(&r, 0, TreeVersion::Input, Scale::Div).unwrap();
    let value = serde_json::to_value(&view.left).unwrap();
    let colors = palette().light;
    assert_eq!(
      json!({"key": "mcc", "title": "MCC (ha and na)", "type": "categorical", "scale": [["MCC 1", colors.mcc[0]], ["MCC 2", colors.mcc[1]]]}),
      value["meta"]["colorings"][0]
    );
    assert_eq!(
      json!({"key": "mcc_size", "title": "MCC size", "type": "continuous"}),
      value["meta"]["colorings"][2]
    );
    assert_eq!(json!("v2"), value["version"]);
    // The right root has no MCC, no branch, and no label; the branch above cd starts MCC 2; a
    // leaf has no children.
    let right = serde_json::to_value(&view.right.tree).unwrap();
    assert_eq!(
      json!({"node_attrs": {"div": 0.0, "added": {"value": "No"}}, "branch_attrs": {}}),
      json!({"node_attrs": right["node_attrs"], "branch_attrs": right["branch_attrs"]})
    );
    assert_eq!(
      json!({"name": "C", "node_attrs": {
        "div": 4.0, "mcc": {"value": "MCC 2"}, "largest_mcc": {"value": "MCC 2"}, "mcc_size": {"value": 3},
        "reassortment": {"value": "No"}, "imputed": {"value": "No"}, "one_tree": {"value": "No"},
        "attachment": {"value": "Unambiguous"}
      }, "branch_attrs": {}}),
      right["children"][0]["children"][0]
    );
    assert_eq!(json!({"labels": {"MCC": "2"}}), right["children"][0]["branch_attrs"]);
    assert_eq!(
      json!({"axis_title": "Divergence", "mcc_roots": [{"left": "r1", "right": "abw"}, {"left": "cdp", "right": "cd"}]}),
      json!({"axis_title": view.axis_title, "mcc_roots": serde_json::to_value(&view.mcc_roots).unwrap()})
    );
  }

  /// The nodes of `tree` in preorder, each as its name and MCC number.
  fn preorder(tree: &AuspiceNode) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    let mut stack = vec![tree];
    while let Some(n) = stack.pop() {
      out.push((
        n.name.clone(),
        n.node_attrs
          .mcc
          .as_ref()
          .map(|m| m.value.trim_start_matches("MCC ").to_owned()),
      ));
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

  #[rstest]
  #[trace]
  fn auspice_view_of_the_two_tree_example_has_the_trees_of_the_pair_view(
    #[values(TreeVersion::Input, TreeVersion::Resolved, TreeVersion::Imputed)] version: TreeVersion,
  ) {
    let r = run_trees(&[("ha", "((A,B),(C,(D,X)));"), ("na", "((A,(B,X)),(C,D));")]);
    let pair = pair_view(&r, 0, version, Scale::Div).unwrap();
    let view = auspice_view(&r, 0, version, Scale::Div).unwrap();
    // The trees have no branch lengths, so both views show the cladogram.
    assert_eq!(
      (
        draw_preorder(&pair.left.nodes),
        draw_preorder(&pair.right.nodes),
        Scale::Depth,
        Scale::Depth
      ),
      (
        preorder(&view.left.tree),
        preorder(&view.right.tree),
        pair.scale,
        view.scale
      )
    );
  }

  #[test]
  fn auspice_view_of_an_unknown_pair_is_none() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    assert!(auspice_view(&r, 1, TreeVersion::Resolved, Scale::Div).is_none());
  }
}
