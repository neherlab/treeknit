//! Display data: the drawable form of a pair of trees (tanglegram), of the ARG, and of the
//! leaf-by-pair MCC table.
//!
//! Coordinates are in normalized units. x runs from 0 to 1 across the column of a shape: a tree
//! column, or the link zone between the two label columns of a tanglegram. y counts leaf rows,
//! from 0 for the first leaf in display order. The right tree of a tanglegram is not mirrored in
//! the data; the consumer mirrors its column. The SVG figures and the interactive views draw the
//! same shapes and only map these units to pixels.

use serde::{Deserialize, Serialize};
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// Number of MCC color slots, the length of `ThemeColors.mcc` in `palette`.
pub const MCC_SLOTS: usize = 8;

/// The drawing rules that depend on the drawn size, shared by the SVG figures and the
/// interactive views.
pub const DRAWING_RULES: DrawingRules = DrawingRules {
  label_auto_min_row_px: 10,
  link_min_row_px: 6,
  label_max_chars: 40,
};

/// Thresholds of the drawing rules that the consumer applies, because they depend on the
/// height of a drawn leaf row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct DrawingRules {
  /// In the label mode `auto`, leaf labels are drawn from this many px per row.
  pub label_auto_min_row_px: u32,
  /// From this many px per row, each link is an S-curve; below it, each block is a ribbon.
  pub link_min_row_px: u32,
  /// A longer leaf label is shortened in the middle to this many characters.
  pub label_max_chars: u32,
}

/// A point `[x, y]` in normalized units.
#[cfg_attr(feature = "tsify", tsify::declare)]
pub type Point = [f64; 2];

/// Version of the trees of a pair: input, resolved, or imputed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "lowercase")]
pub enum TreeVersion {
  /// The parsed input tree.
  Input,
  /// The final tree of the run.
  #[default]
  Resolved,
  /// The resolved tree with the leaves that only other trees have placed into it.
  Imputed,
}

/// Branch scale of a drawing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "lowercase")]
pub enum Scale {
  /// Divergence from the root, from the branch lengths.
  #[default]
  Div,
  /// Cladogram: all leaves at the largest leaf depth.
  Depth,
}

/// A tree laid out for drawing, with the MCCs of one pair.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct DrawTree {
  pub label: String,
  /// One node per tree node, indexed from 0 in preorder; node 0 is the root.
  pub nodes: Vec<DrawNode>,
}

/// A node of a `DrawTree`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct DrawNode {
  pub name: String,
  /// Index of the parent; `None` for the root.
  pub parent: Option<usize>,
  /// Indices of the children, in display order.
  pub children: Vec<usize>,
  /// Branch length as parsed; `None` when the tree gives none.
  pub branch_length: Option<f64>,
  /// Divergence from the root; a missing or negative length counts as 0.
  pub x_div: f64,
  /// Cladogram position: all leaves at the largest leaf depth, each internal node one step left
  /// of its closest child.
  pub x_depth: f64,
  /// Leaf rank 0 to n-1 in display order; an internal node sits at the midpoint of its first and
  /// last child.
  pub y: f64,
  pub leaf: bool,
  /// An internal node that the parsed input tree lacks, added by resolution or imputation.
  pub added: bool,
  /// A leaf that the input tree lacks, placed by imputation.
  pub imputed: bool,
  /// Index of the node's MCC in `PairView.mccs`; `None` for a node without an MCC.
  pub mcc: Option<usize>,
  /// The branch above the node is a reassortment branch: the node is not the root, has an MCC,
  /// and its parent's MCC is a different one or none.
  pub mcc_break: bool,
}

/// The tanglegram of one pair of trees in one version.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct PairView {
  pub left: DrawTree,
  pub right: DrawTree,
  /// One link per leaf in both drawn trees, in the left display order.
  pub links: Vec<Link>,
  /// Runs of consecutive links of one MCC, in the left display order.
  pub blocks: Vec<Block>,
  /// The MCCs of the pair; `DrawNode.mcc`, `Link.mcc`, and `Block.mcc` index this list.
  pub mccs: Vec<MccInfo>,
  /// The shapes of the drawing for the requested scale.
  pub shapes: PairShapes,
}

/// The two copies of one leaf.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Link {
  /// Node index of the leaf in the left tree.
  pub left: usize,
  /// Node index of the leaf in the right tree.
  pub right: usize,
  /// Index of the leaf's MCC in `PairView.mccs`.
  pub mcc: usize,
}

/// A set of leaves of one MCC, consecutive in both trees.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Block {
  /// Index of the MCC in `PairView.mccs`.
  pub mcc: usize,
  /// First and last leaf row of the block in the left tree.
  pub left: [f64; 2],
  /// First and last leaf row of the block in the right tree, in left order: the first value
  /// belongs to the first leaf of the block in the left tree.
  pub right: [f64; 2],
}

/// An MCC of a pair.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct MccInfo {
  /// Index of the MCC in `PairView.mccs`, the order of `MCCs.json`.
  pub index: usize,
  /// Number of leaves, attached leaves included.
  pub size: usize,
  /// Leaf names.
  pub leaves: Vec<String>,
  /// Names of the members that imputation attached (leaves in one tree of the pair only).
  pub imputed_leaves: Vec<String>,
  /// An attachment of a member is ambiguous.
  pub ambiguous: bool,
  /// Color slot, 0 to 7, the same in every version of the pair.
  pub slot: usize,
}

/// The shapes of a tanglegram.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct PairShapes {
  /// Branches and marks of the left tree, x across the left tree column.
  pub left: TreeShapes,
  /// Branches and marks of the right tree, x across the right tree column, unmirrored.
  pub right: TreeShapes,
  /// One S-curve per link, x across the link zone: from (0, left y) to (1, right y).
  pub links: Vec<LinkCurve>,
  /// One outline per block, x across the link zone.
  pub ribbons: Vec<Ribbon>,
}

/// Branches and marks of one tree.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct TreeShapes {
  /// One elbow per non-root node.
  pub elbows: Vec<Elbow>,
  pub marks: Vec<Mark>,
}

/// The rectangular branch above a node: from (parent x, parent y) to (parent x, node y) to
/// (node x, node y).
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Elbow {
  /// Index of the node below the branch.
  pub node: usize,
  pub points: [Point; 3],
  /// Color slot of the node's MCC; `None` for a node without an MCC.
  pub slot: Option<usize>,
  /// The branch is a reassortment branch.
  pub mcc_break: bool,
  /// The node is `added`; its branch is dashed.
  pub added: bool,
}

/// A point mark on a drawing.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Mark {
  pub kind: MarkKind,
  /// Index of the node the mark belongs to.
  pub node: usize,
  pub at: Point,
}

/// Meaning of a mark.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "lowercase")]
pub enum MarkKind {
  /// Signal ring at the midpoint of a reassortment branch.
  Reassortment,
  /// Hollow circle at the tip of an imputed leaf.
  Imputed,
  /// Signal ring on a hybrid node of the ARG.
  Hybrid,
}

/// A cubic Bézier segment.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Bezier {
  pub from: Point,
  pub c1: Point,
  pub c2: Point,
  pub to: Point,
}

/// The S-curve of one link, with control points at x = 0.5.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct LinkCurve {
  /// Index of the link in `PairView.links`.
  pub link: usize,
  /// Color slot of the link's MCC.
  pub slot: usize,
  pub curve: Bezier,
}

/// The ribbon of one block: its left and right y ranges, each extended by half a row, joined by
/// two S-curves.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Ribbon {
  /// Index of the block in `PairView.blocks`.
  pub block: usize,
  /// Color slot of the block's MCC.
  pub slot: usize,
  /// Closed outline: each segment starts where the previous one ends, and the last ends where
  /// the first starts.
  pub outline: Vec<Bezier>,
}

/// The ARG of two trees laid out for drawing, in one tree column. Segment 0 (A) is the first
/// tree and segment 1 (B) the second.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ArgView {
  /// The ARG nodes, with the synthetic `GlobalRoot` last when `root_case` is `synthetic`.
  pub nodes: Vec<ArgNodeView>,
  pub edges: Vec<ArgEdge>,
  /// Index of the top root.
  pub root: usize,
  /// How the top root relates to the segment roots, as in `ARG/arg.nwk`.
  pub root_case: RootCase,
  /// The shapes of the drawing for the requested scale.
  pub shapes: ArgShapes,
}

/// A node of an `ArgView`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ArgNodeView {
  pub label: String,
  /// Parent index per segment; `None` where the node is a root of the segment or lacks it.
  pub parents: [Option<usize>; 2],
  /// Indices of the children, in display order.
  pub children: Vec<usize>,
  /// Length of the branch to the parent, per segment.
  pub tau: [Option<f64>; 2],
  /// The node has different parents in the two segments.
  pub hybrid: bool,
  pub leaf: bool,
  /// The segments the node carries (0, 1, or both), ascending.
  pub segments: Vec<usize>,
  /// Distance from the top root along the parent chain that leads to it; a missing or negative
  /// length counts as 0.
  pub x_div: f64,
  /// Cladogram position along the same chain, with all leaves at the largest leaf depth.
  pub x_depth: f64,
  /// Leaf rank in display order; an internal node sits at the midpoint of its children.
  pub y: f64,
}

/// An edge of an `ArgView`, from parent to child.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ArgEdge {
  pub parent: usize,
  pub child: usize,
  /// The segments the edge carries, ascending.
  pub segments: Vec<usize>,
  /// The child is a hybrid node.
  pub reticulation: bool,
}

/// The three top-root cases of the extended Newick writer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub enum RootCase {
  /// The two segment roots are one node, the top root.
  Shared,
  /// Neither root is shared; a synthetic `GlobalRoot` sits above both with zero-length edges.
  Synthetic,
  /// Exactly one segment root is shared; the unshared root is the top root.
  OneShared,
}

/// The shapes of an ARG drawing, x across the tree column.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ArgShapes {
  /// One shape per edge, in the order of `ArgView.edges`.
  pub edges: Vec<ArgEdgeShape>,
  /// Hybrid rings.
  pub marks: Vec<Mark>,
}

/// The shape of one ARG edge.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ArgEdgeShape {
  /// Index of the edge in `ArgView.edges`.
  pub edge: usize,
  pub path: EdgePath,
}

/// Path of an ARG edge: an elbow, or a dashed S-curve for a reticulation edge.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum EdgePath {
  /// From (parent x, parent y) to (parent x, child y) to (child x, child y).
  Elbow { points: [Point; 3] },
  /// From the parent to the hybrid child.
  Curve { curve: Bezier },
}

/// The MCC of every leaf in every pair.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ConstellationTable {
  /// Every taxon: first the leaves of the first tree in the display order of its resolved
  /// version, then the leaves it lacks, in the order of the first tree that has them.
  pub leaves: Vec<String>,
  /// Labels of the two trees of each pair, in pipeline order.
  pub pairs: Vec<[String; 2]>,
  /// `cells[leaf][pair]`; `None` when the leaf is in neither tree of the pair.
  pub cells: Vec<Vec<Option<ConstellationCell>>>,
}

/// The MCC of one leaf in one pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ConstellationCell {
  /// Index of the MCC in the pair's `PairView.mccs`.
  pub mcc: usize,
  /// Number of leaves of the MCC.
  pub size: usize,
  /// Color slot of the MCC.
  pub slot: usize,
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use serde_json::json;

  #[test]
  fn tree_version_and_scale_are_lowercase_strings() {
    assert_eq!(
      json!(["input", "resolved", "imputed"]),
      json!([TreeVersion::Input, TreeVersion::Resolved, TreeVersion::Imputed])
    );
    assert_eq!(json!(["div", "depth"]), json!([Scale::Div, Scale::Depth]));
    assert_eq!(TreeVersion::Imputed, serde_json::from_value(json!("imputed")).unwrap());
  }

  #[test]
  fn draw_node_serializes_camel_case() {
    let node = DrawNode {
      name: "X".into(),
      parent: Some(3),
      children: vec![],
      branch_length: None,
      x_div: 0.5,
      x_depth: 1.0,
      y: 4.0,
      leaf: true,
      added: false,
      imputed: false,
      mcc: Some(0),
      mcc_break: true,
    };
    let expected = json!({
      "name": "X", "parent": 3, "children": [], "branchLength": null, "xDiv": 0.5, "xDepth": 1.0,
      "y": 4.0, "leaf": true, "added": false, "imputed": false, "mcc": 0, "mccBreak": true,
    });
    assert_eq!(expected, serde_json::to_value(&node).unwrap());
  }

  #[test]
  fn drawing_rules_serialize_camel_case() {
    let expected = json!({"labelAutoMinRowPx": 10, "linkMinRowPx": 6, "labelMaxChars": 40});
    assert_eq!(expected, serde_json::to_value(DRAWING_RULES).unwrap());
  }

  #[test]
  fn edge_path_is_tagged_by_kind() {
    let elbow = EdgePath::Elbow {
      points: [[0.0, 0.5], [0.0, 1.0], [0.25, 1.0]],
    };
    let expected = json!({"kind": "elbow", "points": [[0.0, 0.5], [0.0, 1.0], [0.25, 1.0]]});
    assert_eq!(expected, serde_json::to_value(elbow).unwrap());
  }

  #[test]
  fn root_case_and_mark_kind_strings() {
    assert_eq!(
      json!(["shared", "synthetic", "oneShared"]),
      json!([RootCase::Shared, RootCase::Synthetic, RootCase::OneShared])
    );
    assert_eq!(
      json!(["reassortment", "imputed", "hybrid"]),
      json!([MarkKind::Reassortment, MarkKind::Imputed, MarkKind::Hybrid])
    );
  }
}
