//! Display data: the drawable form of a pair of trees (tanglegram), of the ARG, and of the
//! leaf-by-pair MCC table.
//!
//! Node coordinates are in tree units: `x_div` in branch-length units from the root, `x_depth` in
//! branch steps (the height of the root minus the height of the node, so every leaf is at the
//! height of the root), and `y` in leaf rows, from 0 for the first leaf in display order.
//! The shapes (`PairShapes`, `ArgShapes`) are in normalized units: x runs from 0 to 1 across the
//! column of a shape (a tree column, or the link zone between the two label columns of a
//! tanglegram), and y counts leaf rows as the nodes do. The right tree of a tanglegram is not
//! mirrored in the data; the consumer mirrors its column. The SVG figures and the interactive
//! views draw the same shapes and only map these units to pixels.

mod arg_view;
mod auspice;
mod constellation;
mod legend;
mod lengths;
mod names;
mod pair;
mod rows;
mod rules;
mod shapes;
mod slots;
mod tree;

pub use arg_view::arg_view;
pub use auspice::auspice_view;
pub use constellation::constellation;
pub use legend::{LegendItem, LegendKind, LegendMark, Stroke};
pub(crate) use names::shorten;
pub(crate) use names::{grapheme_count, mcc_name};
pub use pair::pair_view;
pub use rows::RowSpan;
pub use rules::{ColumnPx, TanglegramColumns};
pub(crate) use shapes::s_curve;

use serde::{Deserialize, Serialize};
use strum::VariantArray;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// Number of MCC color slots, the length of `ThemeColors.mcc` in `palette`.
pub(crate) const MCC_SLOTS: usize = 8;

/// The drawing rules that depend on the drawn size, shared by the SVG figures and the
/// interactive views.
pub const DRAWING_RULES: DrawingRules = DrawingRules {
  label_auto_min_row_px: 10,
  link_min_row_px: 6,
  label_max_chars: 40,
  label_font_px: 12.0,
  legend_symbol_px: 24.0,
  margin_px: 16.0,
  label_gap_px: 6.0,
  link_zone_share: 0.2,
  link_zone_min_share: 0.15,
  tanglegram_label_column_max_share: 0.25,
  arg_label_column_max_share: 0.25,
  branch_width_px: 1.5,
  reassortment_width_px: 2.0,
  link_width_px: 1.0,
  leader_width_px: 1.0,
  leader_opacity: 0.5,
  mark_radius_px: 3.5,
  mark_line_px: 1.5,
  ribbon_opacity: 0.55,
  dash_px: [4.0, 3.0],
  dot_px: [1.0, 3.0],
};

/// The longest label of the drawing rules, in characters.
pub(crate) fn label_max_chars() -> usize {
  #[expect(clippy::expect_used, reason = "the rule is 40 characters, which every usize holds")]
  usize::try_from(DRAWING_RULES.label_max_chars).expect("the label length fits in usize")
}

/// The drawing rules that the consumer applies, because they depend on the drawn size: the
/// thresholds of the row height, the columns of a drawing, and the strokes and marks in px. The
/// label width that the columns take is measured by the consumer: the interactive views measure
/// the rendered text, the SVG figures estimate it.
///
/// A drawing has a margin of `margin_px` on each side; the inner width is the rest. A label
/// column is as wide as its longest label plus `label_gap_px` on each side, at most a share of
/// the width it labels. The tanglegram has, from left to right, the left tree, its labels, the
/// link zone, the right labels, and the mirrored right tree. Its link zone takes
/// `link_zone_share` of the inner width minus both label columns, at least `link_zone_min_share`
/// of the inner width, and the two trees share the rest equally. The ARG has its tree, then its
/// labels.
///
/// The methods of `DrawingRules` (`display/rules.rs`) are the reference of these formulas: the SVG
/// figures call them, and the web app, which applies them on every zoom frame and resize without
/// a round trip to its worker, checks its own copy against the table of
/// `examples/drawing_cases.rs`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct DrawingRules {
  /// In the label mode `auto`, leaf labels are drawn from this many px per row.
  pub(crate) label_auto_min_row_px: u32,
  /// From this many px per row, each link is an S-curve; below it, each block is a ribbon.
  pub(crate) link_min_row_px: u32,
  /// A longer leaf label is shortened in the middle to this many characters (grapheme clusters);
  /// `DrawNode.short_name` and `ArgNodeView.short_label` hold the shortened labels.
  pub(crate) label_max_chars: u32,
  /// Font size of the leaf labels, in px.
  pub(crate) label_font_px: f64,
  /// Width of a legend symbol, in px; a legend ring sits at a fraction of it (`LegendMark::Ring`).
  pub(crate) legend_symbol_px: f64,
  /// Space around a drawing, in px.
  pub(crate) margin_px: f64,
  /// Space on each side of a label column, between it and the tree and the link zone, in px.
  pub(crate) label_gap_px: f64,
  /// Share of the inner width that the link zone of a tanglegram takes, minus the label columns.
  pub(crate) link_zone_share: f64,
  /// Smallest share of the inner width that the link zone of a tanglegram takes.
  pub(crate) link_zone_min_share: f64,
  /// Largest share of half the inner width that each label column of a tanglegram takes.
  pub(crate) tanglegram_label_column_max_share: f64,
  /// Largest share of the inner width that the label column of an ARG takes.
  pub(crate) arg_label_column_max_share: f64,
  /// Stroke width of a branch, in px.
  pub(crate) branch_width_px: f64,
  /// Stroke width of a reassortment branch, in px.
  pub(crate) reassortment_width_px: f64,
  /// Stroke width of a link, in px.
  pub(crate) link_width_px: f64,
  /// Stroke width of a leader from a leaf tip to its label, in px.
  pub(crate) leader_width_px: f64,
  /// Opacity of a leader.
  pub(crate) leader_opacity: f64,
  /// Radius of a mark ring, in px.
  pub(crate) mark_radius_px: f64,
  /// Stroke width of a mark ring, in px.
  pub(crate) mark_line_px: f64,
  /// Opacity of the fill of a ribbon.
  pub(crate) ribbon_opacity: f64,
  /// Dash and gap of a dashed line (reticulations), in px.
  pub(crate) dash_px: [f64; 2],
  /// Dash and gap of a dotted line (leaders), in px.
  pub(crate) dot_px: [f64; 2],
}

/// Version of the trees of a pair: input, resolved, or imputed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize, VariantArray)]
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

/// A count of rows, steps, lines, or characters as a coordinate.
#[expect(
  clippy::as_conversions,
  reason = "these counts are far below 2^53, so the conversion is exact"
)]
pub(crate) fn coordinate(n: usize) -> f64 {
  n as f64
}

/// The tanglegram of one pair of trees in one version.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct PairView {
  pub(crate) left: DrawTree,
  pub(crate) right: DrawTree,
  /// One link per leaf in both drawn trees that has an MCC of the pair, in the left display
  /// order.
  pub(crate) links: Vec<Link>,
  /// Runs of consecutive links of one MCC, in the left display order.
  pub(crate) blocks: Vec<Block>,
  /// The MCCs of the pair; `DrawNode.mcc`, `Link.mcc`, and `Block.mcc` index this list.
  pub(crate) mccs: Vec<MccInfo>,
  /// The scale of `shapes`: the requested scale, or `depth` for `div` when a tree of the pair has
  /// no branch lengths, because `div` would draw all the nodes of that tree at the root.
  pub(crate) scale: Scale,
  /// The shapes of the drawing for `scale`.
  pub(crate) shapes: PairShapes,
  /// The legend: an entry for each kind of shape that the drawing contains.
  pub(crate) legend: Vec<LegendItem>,
}

/// A tree laid out for drawing, with the MCCs of one pair.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct DrawTree {
  pub(crate) label: String,
  /// One node per tree node, indexed from 0 in preorder; node 0 is the root.
  pub(crate) nodes: Vec<DrawNode>,
}

impl DrawTree {
  /// The leaves, in display order.
  pub(crate) fn leaves(&self) -> impl Iterator<Item = &DrawNode> {
    self.nodes.iter().filter(|n| n.leaf)
  }
}

/// A node of a `DrawTree`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct DrawNode {
  pub(crate) name: String,
  /// `name` as a label shows it: shortened in the middle to `DRAWING_RULES.label_max_chars`
  /// characters (grapheme clusters) with an ellipsis.
  pub(crate) short_name: String,
  /// Index of the parent; `None` for the root.
  pub(crate) parent: Option<usize>,
  /// Indices of the children, in display order.
  pub(crate) children: Vec<usize>,
  /// Branch length as parsed; `None` when the tree gives none or the length is not finite.
  pub(crate) branch_length: Option<f64>,
  /// The mean length of the branch over the trees of the run, weighted by their sequence
  /// lengths, at which the drawing shows a split that some trees lack (see
  /// `lengths::mean_lengths`); `None` for a branch drawn at `branch_length`.
  pub(crate) mean_length: Option<f64>,
  /// Divergence from the root; a missing or negative length counts as 0. A branch with a
  /// `mean_length` adds that length instead of its own, so the nodes below it move with it.
  pub(crate) x_div: f64,
  /// Cladogram position in branch steps: the height of the root minus the height of the node,
  /// where a height is the largest number of branches from a node down to a leaf. Every leaf is
  /// at the height of the root.
  pub(crate) x_depth: f64,
  /// Leaf rank 0 to n-1 in display order; an internal node sits at the midpoint of its first and
  /// last child.
  pub(crate) y: f64,
  pub(crate) leaf: bool,
  /// Number of leaves at or below the node: 1 for a leaf.
  pub(crate) clade_size: usize,
  /// The rows of the leaves at or below the node: the row of a leaf, for a leaf.
  pub(crate) rows: RowSpan,
  /// An internal node that the parsed input tree lacks, added by resolution or imputation.
  pub(crate) added: bool,
  /// A leaf that the input tree lacks, placed by imputation.
  pub(crate) imputed: bool,
  /// Index of the node's MCC in `PairView.mccs`; `None` for a node without an MCC.
  pub(crate) mcc: Option<usize>,
  /// The branch above the node is a reassortment branch: the node is not the root, has an MCC,
  /// and its parent's MCC is a different one or none.
  pub(crate) mcc_break: bool,
}

/// The two copies of one leaf.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Link {
  /// Node index of the leaf in the left tree.
  left: usize,
  /// Node index of the leaf in the right tree.
  right: usize,
  /// Index of the leaf's MCC in `PairView.mccs`.
  mcc: usize,
}

/// A set of leaves of one MCC, consecutive in both trees.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Block {
  /// Index of the MCC in `PairView.mccs`.
  mcc: usize,
  /// First and last leaf row of the block in the left tree.
  left: [f64; 2],
  /// First and last leaf row of the block in the right tree, in left order: the first value
  /// belongs to the first leaf of the block in the left tree.
  right: [f64; 2],
}

/// An MCC of a pair.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct MccInfo {
  /// Index of the MCC in `PairView.mccs`, the order of `MCCs.json`.
  pub(crate) index: usize,
  /// Number of leaves, attached leaves included.
  pub(crate) size: usize,
  /// Leaf names.
  pub(crate) leaves: Vec<String>,
  /// Names of the members that imputation attached (leaves in one tree of the pair only).
  pub(crate) imputed_leaves: Vec<String>,
  /// Names of the attached members whose attachment is ambiguous: their attachment point is in
  /// no MCC, so they joined the MCC whose root is closest to it.
  pub(crate) ambiguous_leaves: Vec<String>,
  /// Color slot, from 0 to `MCC_SLOTS` - 1, the same in every version of the pair.
  pub(crate) slot: usize,
  /// The name of the MCC: "MCC 1" for index 0.
  pub(crate) name: String,
  /// Rank of the MCC by size, largest first, then by index: 0 for the largest MCC. The color
  /// slots are given in this order.
  pub(crate) rank: usize,
  /// The rows of the leaves of the MCC in both drawn trees; `None` when neither drawn tree has a
  /// leaf of it.
  pub(crate) rows: Option<RowSpan>,
}

/// The shapes of a tanglegram.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct PairShapes {
  /// Branches and marks of the left tree, x across the left tree column.
  pub(crate) left: TreeShapes,
  /// Branches and marks of the right tree, x across the right tree column, unmirrored.
  pub(crate) right: TreeShapes,
  /// One S-curve per link, x across the link zone: from (0, left y) to (1, right y).
  pub(crate) links: Vec<LinkCurve>,
  /// One outline per block, x across the link zone.
  pub(crate) ribbons: Vec<Ribbon>,
}

/// Branches and marks of one tree.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct TreeShapes {
  /// One elbow per non-root node.
  pub(crate) elbows: Vec<Elbow>,
  pub(crate) marks: Vec<Mark>,
  /// One leader per leaf, in node order.
  pub(crate) leaders: Vec<Leader>,
}

/// The rectangular branch above a node: from (parent x, parent y) to (parent x, node y) to
/// (node x, node y).
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Elbow {
  /// Index of the node below the branch.
  pub(crate) node: usize,
  pub(crate) points: [Point; 3],
  /// The node's MCC, its `DrawNode.mcc`.
  pub(crate) mcc: Option<usize>,
  /// Color slot of the node's MCC; `None` for a node without an MCC.
  pub(crate) slot: Option<usize>,
  /// The branch is a reassortment branch.
  pub(crate) mcc_break: bool,
  /// The node is `added`: the part across, from `points[1]` to `points[2]`, is its new branch
  /// and is drawn in ink-muted; the part along the parent's x joins it to its parent like a plain
  /// branch.
  pub(crate) added: bool,
  /// The color of the branch: signal for a reassortment branch, else the color of `slot`, or "no
  /// MCC". The part across of an added node is ink-muted instead.
  pub(crate) color: ColorRole,
}

/// The S-curve of one link, with control points at x = 0.5.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct LinkCurve {
  /// Index of the link in `PairView.links`.
  pub(crate) link: usize,
  /// The link's MCC, its `Link.mcc`.
  pub(crate) mcc: usize,
  /// Color slot of the link's MCC.
  pub(crate) slot: usize,
  pub(crate) curve: Bezier,
}

/// The ribbon of one block: its left and right y ranges, each extended by half a row, joined by
/// two S-curves.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Ribbon {
  /// Index of the block in `PairView.blocks`.
  pub(crate) block: usize,
  /// The block's MCC, its `Block.mcc`.
  pub(crate) mcc: usize,
  /// Color slot of the block's MCC.
  pub(crate) slot: usize,
  /// Closed outline: each segment starts where the previous one ends, and the last ends where
  /// the first starts.
  pub(crate) outline: Vec<Bezier>,
}

/// The ARG of two trees laid out for drawing, in one tree column. Segment 0 (A) is the first
/// tree and segment 1 (B) the second.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ArgView {
  /// The ARG nodes, with the synthetic `GlobalRoot` last when `root_case` is `synthetic`.
  pub(crate) nodes: Vec<ArgNodeView>,
  pub(crate) edges: Vec<ArgEdge>,
  /// Index of the top root.
  pub(crate) root: usize,
  /// How the top root relates to the segment roots, as in `ARG/arg.nwk`.
  pub(crate) root_case: RootCase,
  /// The scale of `shapes`: the requested scale, or `depth` for `div` when the ARG has no branch
  /// lengths, because `div` would draw all its nodes at the root. The ARG has branch lengths when
  /// one of its trees has them, because it takes the length of a branch from the other tree where
  /// one tree lacks it.
  pub(crate) scale: Scale,
  /// The shapes of the drawing for `scale`.
  pub(crate) shapes: ArgShapes,
  /// The legend: the two segments, both, and reassortment when the ARG has a hybrid node.
  pub(crate) legend: Vec<LegendItem>,
  /// The labels of the trees of segment A and segment B, which `ArgNodeView.segments` and
  /// `ArgEdge.segments` index.
  pub(crate) segments: [String; 2],
}

/// A node of an `ArgView`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ArgNodeView {
  pub(crate) label: String,
  /// `label` as a label shows it, shortened as `DrawNode.short_name`.
  pub(crate) short_label: String,
  /// Parent index per segment; `None` for the top root, where the node lacks the segment, and
  /// where it is the root of the segment without the synthetic `GlobalRoot` above it.
  pub(crate) parents: [Option<usize>; 2],
  /// Indices of the children, in display order.
  pub(crate) children: Vec<usize>,
  /// Length of the branch to the parent, per segment.
  pub(crate) tau: [Option<f64>; 2],
  /// The node has different parents in the two segments.
  pub(crate) hybrid: bool,
  pub(crate) leaf: bool,
  /// The segments the node carries (0, 1, or both), ascending.
  pub(crate) segments: Vec<usize>,
  /// Distance from the top root along the parent chain that leads to it; a missing or negative
  /// length counts as 0.
  pub(crate) x_div: f64,
  /// Cladogram position in branch steps, as `DrawNode.x_depth`: the height of the top root minus
  /// the height of the node, where a height is the largest number of edges from a node down to a
  /// leaf over the children of both segments.
  pub(crate) x_depth: f64,
  /// Leaf rank in display order; an internal node sits at the midpoint of its children.
  pub(crate) y: f64,
  /// The rows of the leaves at or below the node, below either parent of a hybrid node: the row
  /// of a leaf, for a leaf.
  pub(crate) rows: RowSpan,
}

/// An edge of an `ArgView`, from parent to child.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ArgEdge {
  pub(crate) parent: usize,
  pub(crate) child: usize,
  /// The segments the edge carries, ascending.
  pub(crate) segments: Vec<usize>,
  /// The child is a hybrid node and the edge is not on the child's chain to the top root, along
  /// which the node's x is measured. Each hybrid node has one reticulation edge.
  pub(crate) reticulation: bool,
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
  pub(crate) edges: Vec<ArgEdgeShape>,
  /// Hybrid rings.
  pub(crate) marks: Vec<Mark>,
  /// One leader per leaf, in node order.
  pub(crate) leaders: Vec<Leader>,
}

/// The dotted line from a leaf tip to the label edge of its tree column, at x = 1, so that the
/// labels align when the tips do not. A leaf at the label edge has a leader of length 0.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Leader {
  /// Index of the leaf.
  pub(crate) node: usize,
  /// The tip of the leaf.
  pub(crate) from: Point,
  /// The label edge, at the leaf's row.
  pub(crate) to: Point,
}

/// A point mark on a drawing.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Mark {
  pub(crate) kind: MarkKind,
  /// Index of the node the mark belongs to.
  pub(crate) node: usize,
  /// MCC of the node in a tanglegram, its `DrawNode.mcc`; `None` for a node without an MCC and
  /// on the ARG.
  pub(crate) mcc: Option<usize>,
  /// Color slot of `mcc`.
  pub(crate) slot: Option<usize>,
  pub(crate) at: Point,
  /// The color of the ring: signal for reassortment and hybrid nodes, else the color of `slot`,
  /// or "no MCC".
  pub(crate) color: ColorRole,
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

/// The shape of one ARG edge.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ArgEdgeShape {
  /// Index of the edge in `ArgView.edges`.
  pub(crate) edge: usize,
  /// The segments of the edge, its `ArgEdge.segments`.
  pub(crate) segments: Vec<usize>,
  /// The edge is a reticulation edge, its `ArgEdge.reticulation`; its path is a curve.
  pub(crate) reticulation: bool,
  pub(crate) path: EdgePath,
  /// The color of the edge: its segment, or ink for an edge of both segments.
  pub(crate) color: ColorRole,
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
  /// Every taxon: first the leaves of the first tree in the leaf order of its final tree, the
  /// order of its `_resolved` output file, then the leaves it lacks, in the order of the first
  /// tree that has them. The pair views can order the first tree differently, because they sort
  /// both trees of a pair for that pair.
  leaves: Vec<String>,
  /// The title of each pair, in pipeline order, as `PairSummary.title`.
  pair_titles: Vec<String>,
  /// `cells[leaf][pair]`; `None` when the leaf is in neither tree of the pair, or the pair has no
  /// MCCs (its trees share fewer than two leaves).
  cells: Vec<Vec<Option<ConstellationCell>>>,
  /// The name of each MCC index of the cells, `mcc_names[cell.mcc]`, as `MccInfo.name`.
  mcc_names: Vec<String>,
}

/// The MCC of one leaf in one pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ConstellationCell {
  /// Index of the MCC in the pair's `PairView.mccs`.
  mcc: usize,
  /// Number of leaves of the MCC.
  size: usize,
  /// Color slot of the MCC.
  slot: usize,
}

/// The two trees of a pair as Auspice v2 datasets, for Auspice's tanglegram: `left` is the main
/// tree and `right` the second tree. Auspice joins the tips of the two trees by name.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct AuspicePair {
  /// The scale of the `div` of the nodes, as `PairView.scale`: `depth` for `div` when a tree of
  /// the pair has no branch lengths.
  pub(crate) scale: Scale,
  /// The name of the `div` values on the axis and in the hover panel: "Divergence" for the shown
  /// scale `div`, "Depth" for `depth`.
  pub(crate) axis_title: String,
  /// For each MCC, in the order of `MCCs.json`, the node where it starts in each tree.
  pub(crate) mcc_roots: Vec<AuspiceMccRoot>,
  /// The key of the coloring and the filter by MCC in the datasets.
  pub(crate) mcc_key: String,
  /// For each MCC, in the order of `MCCs.json`, its value in the coloring and the filter of
  /// `mcc_key`: `MccInfo.name`.
  pub(crate) mcc_values: Vec<String>,
  /// The trees that each choice of `AuspiceTrees` shows.
  pub(crate) shown: AuspiceShown,
  pub(crate) left: AuspiceDataset,
  pub(crate) right: AuspiceDataset,
}

/// The trees of a pair that each choice of `AuspiceTrees` shows, main tree first; each field is
/// named after its choice.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct AuspiceShown {
  both: Vec<AuspiceShownTree>,
  left: Vec<AuspiceShownTree>,
  right: Vec<AuspiceShownTree>,
}

/// A tree that the Auspice view shows: its index among the trees of the run, and its label.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct AuspiceShownTree {
  tree: usize,
  label: String,
}

/// The node where an MCC starts in each tree of a pair: the node whose branch above is the
/// MCC's reassortment branch, or the root when the MCC holds the root; `None` for a tree without
/// a node of the MCC.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct AuspiceMccRoot {
  left: Option<String>,
  right: Option<String>,
}

/// The trees of a pair that the Auspice view shows: both as a tanglegram, or one alone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "lowercase")]
pub enum AuspiceTrees {
  #[default]
  Both,
  /// The first tree of the pair (pipeline order).
  Left,
  /// The second tree of the pair.
  Right,
}

impl AuspiceTrees {
  /// Whether the view shows the left tree (`true`) or the right tree (`false`) of the pair.
  #[must_use]
  pub(crate) fn shows(self, left: bool) -> bool {
    match self {
      Self::Both => true,
      Self::Left => left,
      Self::Right => !left,
    }
  }
}

/// An Auspice v2 dataset of one tree, with the field names of Auspice's schema.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct AuspiceDataset {
  version: AuspiceSchema,
  meta: AuspiceMeta,
  tree: AuspiceNode,
}

/// Version of the Auspice dataset schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub enum AuspiceSchema {
  #[serde(rename = "v2")]
  V2,
}

/// The `meta` section of an Auspice dataset.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct AuspiceMeta {
  title: String,
  panels: Vec<AuspicePanel>,
  colorings: Vec<AuspiceColoring>,
  /// Keys of the colorings that Auspice offers as filters.
  filters: Vec<String>,
  display_defaults: AuspiceDisplayDefaults,
  /// What the download panel of Auspice offers.
  sharing: AuspiceSharing,
}

/// A panel of the Auspice view.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "lowercase")]
pub enum AuspicePanel {
  Tree,
}

/// A coloring of the Auspice view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct AuspiceColoring {
  /// Key of the node attribute.
  key: String,
  title: String,
  #[serde(rename = "type")]
  kind: AuspiceColoringKind,
  /// `[value, color]` per value, the color as `#rrggbb`.
  /// Absent for a continuous coloring, which Auspice colors with its own scale.
  #[serde(skip_serializing_if = "Option::is_none")]
  scale: Option<Vec<(String, String)>>,
}

/// Type of an Auspice coloring.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "lowercase")]
pub enum AuspiceColoringKind {
  Categorical,
  Continuous,
}

/// What the download panel of Auspice offers besides the trees.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct AuspiceSharing {
  /// Offer the genetic diversity data (TSV); always `false`, because the datasets have no
  /// sequences, so they have no entropy to download.
  entropy: bool,
}

/// The settings that Auspice starts with.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct AuspiceDisplayDefaults {
  /// Key of the coloring.
  color_by: String,
  /// Key of the branch label.
  branch_label: String,
}

/// A node of an Auspice tree.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct AuspiceNode {
  /// The node's `DrawNode.name`, unique within the tree.
  name: String,
  node_attrs: AuspiceNodeAttrs,
  branch_attrs: AuspiceBranchAttrs,
  /// The children in display order; absent for a leaf.
  #[serde(skip_serializing_if = "Option::is_none")]
  children: Option<Vec<AuspiceNode>>,
}

/// The attributes of an Auspice node.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct AuspiceNodeAttrs {
  /// Position from the root: `DrawNode.x_div` for the scale `div`, `DrawNode.x_depth` for
  /// `depth`.
  div: f64,
  /// The number of the node's MCC, its index in `MCCs.json` plus 1; absent for a node without
  /// an MCC.
  #[serde(skip_serializing_if = "Option::is_none")]
  mcc: Option<AuspiceValue>,
  /// The MCC's value of `mcc` when the MCC is one of the 8 largest of the pair, "Other" for
  /// another MCC; absent for a node without an MCC.
  #[serde(skip_serializing_if = "Option::is_none")]
  largest_mcc: Option<AuspiceValue>,
  /// The number of leaves of the node's MCC; absent for a node without an MCC.
  #[serde(skip_serializing_if = "Option::is_none")]
  mcc_size: Option<AuspiceNumber>,
  /// "Yes" when the branch above the node is a reassortment branch, "No" otherwise; absent for
  /// the root.
  #[serde(skip_serializing_if = "Option::is_none")]
  reassortment: Option<AuspiceValue>,
  /// For a leaf: "Yes" when imputation placed it into the tree.
  #[serde(skip_serializing_if = "Option::is_none")]
  imputed: Option<AuspiceValue>,
  /// For an internal node: "Yes" when resolution or imputation added it.
  #[serde(skip_serializing_if = "Option::is_none")]
  added: Option<AuspiceValue>,
  /// For a leaf: "Yes" when the other tree of the pair lacks it.
  #[serde(skip_serializing_if = "Option::is_none")]
  one_tree: Option<AuspiceValue>,
  /// For a leaf with an MCC: "Ambiguous" when its attachment to the MCC is ambiguous, otherwise
  /// "Unambiguous".
  #[serde(skip_serializing_if = "Option::is_none")]
  attachment: Option<AuspiceValue>,
}

/// The value of a categorical node attribute.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct AuspiceValue {
  value: String,
}

/// The value of a continuous node attribute.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct AuspiceNumber {
  value: usize,
}

/// The attributes of the branch above an Auspice node.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct AuspiceBranchAttrs {
  /// Absent for a branch without labels.
  #[serde(skip_serializing_if = "Option::is_none")]
  labels: Option<AuspiceBranchLabels>,
}

/// The labels of an Auspice branch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct AuspiceBranchLabels {
  /// The number of the MCC that starts at this branch, on a reassortment branch.
  #[serde(rename = "MCC")]
  mcc: String,
}

/// A cubic Bézier segment.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Bezier {
  pub(crate) from: Point,
  pub(crate) c1: Point,
  pub(crate) c2: Point,
  pub(crate) to: Point,
}

/// Branch scale of a drawing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize, VariantArray)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "lowercase")]
pub enum Scale {
  /// Divergence from the root, from the branch lengths.
  #[default]
  Div,
  /// Cladogram: all leaves at the largest leaf depth.
  Depth,
}

impl Scale {
  /// The scale that a drawing shows for this requested scale: `depth` instead of `div` when a
  /// drawn tree has no branch lengths (`flat`), because its divergence is 0 everywhere and `div`
  /// would draw all its nodes at the root. A tree with some branch lengths keeps `div`. The
  /// interactive views and the figures apply the same rule, so both show the same drawing.
  fn shown(self, flat: bool) -> Scale {
    match self {
      Scale::Div if flat => Scale::Depth,
      Scale::Div | Scale::Depth => self,
    }
  }

  /// The x coordinate of a node on this scale: its divergence `x_div` or its depth `x_depth`.
  fn x(self, x_div: f64, x_depth: f64) -> f64 {
    match self {
      Scale::Div => x_div,
      Scale::Depth => x_depth,
    }
  }
}

/// The palette color of a drawn shape by its role; `ThemeColors::role` gives the color.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub enum ColorRole {
  /// The color slot of the shape's MCC; in a legend, the first slot.
  Mcc,
  /// A node without an MCC.
  NoMcc,
  /// An ARG edge of both segments.
  Ink,
  /// An added node.
  InkMuted,
  /// Reassortment.
  Signal,
  /// An ARG edge of segment A only.
  SegmentA,
  /// An ARG edge of segment B only.
  SegmentB,
}

/// A point `[x, y]` in normalized units.
#[cfg_attr(feature = "tsify", tsify::declare)]
pub type Point = [f64; 2];

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
      short_name: "X".into(),
      parent: Some(3),
      children: vec![],
      branch_length: None,
      mean_length: Some(0.25),
      x_div: 0.5,
      x_depth: 1.0,
      y: 4.0,
      leaf: true,
      clade_size: 1,
      rows: RowSpan::row(4),
      added: false,
      imputed: false,
      mcc: Some(0),
      mcc_break: true,
    };
    let expected = json!({
      "name": "X", "shortName": "X", "parent": 3, "children": [], "branchLength": null, "meanLength": 0.25,
      "xDiv": 0.5, "xDepth": 1.0, "y": 4.0, "leaf": true, "cladeSize": 1, "rows": { "first": 4, "last": 4 },
      "added": false, "imputed": false, "mcc": 0, "mccBreak": true,
    });
    assert_eq!(expected, serde_json::to_value(&node).unwrap());
  }

  #[test]
  fn drawing_rules_serialize_camel_case() {
    let expected = json!({
      "labelAutoMinRowPx": 10, "linkMinRowPx": 6, "labelMaxChars": 40, "labelFontPx": 12.0, "legendSymbolPx": 24.0, "marginPx": 16.0, "labelGapPx": 6.0,
      "linkZoneShare": 0.2, "linkZoneMinShare": 0.15, "tanglegramLabelColumnMaxShare": 0.25,
      "argLabelColumnMaxShare": 0.25, "branchWidthPx": 1.5, "reassortmentWidthPx": 2.0, "linkWidthPx": 1.0,
      "leaderWidthPx": 1.0, "leaderOpacity": 0.5, "markRadiusPx": 3.5, "markLinePx": 1.5, "ribbonOpacity": 0.55,
      "dashPx": [4.0, 3.0], "dotPx": [1.0, 3.0],
    });
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
