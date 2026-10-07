//! The legends of the drawings: an entry for each kind of shape that a drawing contains, with the
//! parts of its symbol, which the SVG figures and the web app draw at their own size.

use crate::display::{ArgShapes, ColorRole, MarkKind, PairShapes, TreeShapes};
use serde::Serialize;
use strum::VariantArray;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// The place of the ring of a reassortment branch, as a fraction of the symbol width: the middle
/// of the branch, as on the drawn branch.
const RING_AT_BRANCH_MIDDLE: f64 = 0.5;

/// The place of the ring of an imputed leaf: the tip of a leaf branch that ends before the end of
/// the symbol.
const RING_AT_LEAF_TIP: f64 = 0.75;

/// The place of the ring of a hybrid node: the end of the reticulation curve.
const RING_AT_CURVE_END: f64 = 1.0;

/// The legend of a tanglegram with the shapes `shapes`, from the reassortment branches to the
/// branches without an MCC. The links entry shows one MCC, in the first color slot.
pub(super) fn pair_legend(shapes: &PairShapes) -> Vec<LegendItem> {
  let trees = [&shapes.left, &shapes.right];
  let elbows = || trees.into_iter().flat_map(|t: &TreeShapes| &t.elbows);
  let marks = || trees.into_iter().flat_map(|t: &TreeShapes| &t.marks);
  let branch = |color, stroke| LegendMark::Branch {
    color,
    stroke,
    dashed: false,
  };
  [
    elbows().any(|e| e.mcc_break).then(|| {
      item(
        LegendKind::ReassortmentBranch,
        vec![
          branch(ColorRole::Signal, Stroke::Reassortment),
          LegendMark::Ring {
            color: ColorRole::Signal,
            at: RING_AT_BRANCH_MIDDLE,
          },
        ],
      )
    }),
    elbows().any(|e| e.added && !e.mcc_break).then(|| {
      item(
        LegendKind::AddedNode,
        vec![LegendMark::Branch {
          color: ColorRole::InkMuted,
          stroke: Stroke::Branch,
          dashed: false,
        }],
      )
    }),
    marks().any(|m| m.kind == MarkKind::Imputed).then(|| {
      item(
        LegendKind::ImputedLeaf,
        vec![
          branch(ColorRole::Mcc, Stroke::Branch),
          LegendMark::Ring {
            color: ColorRole::Mcc,
            at: RING_AT_LEAF_TIP,
          },
        ],
      )
    }),
    (!shapes.links.is_empty()).then(|| {
      item(
        LegendKind::Links,
        vec![
          LegendMark::LinkRibbon { color: ColorRole::Mcc },
          LegendMark::LinkCurve { color: ColorRole::Mcc },
        ],
      )
    }),
    elbows()
      .any(|e| e.color == ColorRole::NoMcc)
      .then(|| item(LegendKind::NoMcc, vec![branch(ColorRole::NoMcc, Stroke::Branch)])),
  ]
  .into_iter()
  .flatten()
  .collect()
}

/// The legend of an ARG with the shapes `shapes` whose segments are the trees labeled `a` and
/// `b`: both segments, the edges of both, and reassortment when the ARG has a hybrid node, its
/// curve in the color of the first reticulation edge.
pub(super) fn arg_legend(shapes: &ArgShapes, [a, b]: [&str; 2]) -> Vec<LegendItem> {
  let line = |color| LegendMark::Branch {
    color,
    stroke: Stroke::Branch,
    dashed: false,
  };
  let segment = |kind: LegendKind, label: &str, color| LegendItem {
    kind,
    label: format!("{} {label}", kind.label()),
    marks: vec![line(color)],
  };
  let reassortment = shapes.edges.iter().find(|e| e.reticulation).map(|edge| {
    item(
      LegendKind::Reassortment,
      vec![
        LegendMark::Reticulation { color: edge.color },
        LegendMark::Ring {
          color: ColorRole::Signal,
          at: RING_AT_CURVE_END,
        },
      ],
    )
  });
  [
    Some(segment(LegendKind::SegmentA, a, ColorRole::SegmentA)),
    Some(segment(LegendKind::SegmentB, b, ColorRole::SegmentB)),
    Some(item(LegendKind::BothSegments, vec![line(ColorRole::Ink)])),
    reassortment,
  ]
  .into_iter()
  .flatten()
  .collect()
}

/// The entry of `kind` with its label and the symbol `marks`.
fn item(kind: LegendKind, marks: Vec<LegendMark>) -> LegendItem {
  LegendItem {
    kind,
    label: kind.label().to_owned(),
    marks,
  }
}

/// An entry of the legend of a drawing: its symbol and its label.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct LegendItem {
  pub kind: LegendKind,
  /// `kind.label()`, and for a segment the label of its tree after it.
  pub label: String,
  /// The parts of the symbol, drawn in order.
  pub marks: Vec<LegendMark>,
}

/// What a legend entry stands for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, VariantArray)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub enum LegendKind {
  ReassortmentBranch,
  AddedNode,
  ImputedLeaf,
  Links,
  NoMcc,
  SegmentA,
  SegmentB,
  BothSegments,
  Reassortment,
}

impl LegendKind {
  /// The label of an entry of this kind; an entry of a segment adds the label of its tree.
  pub fn label(self) -> &'static str {
    match self {
      LegendKind::ReassortmentBranch => "Reassortment branch",
      LegendKind::AddedNode => "Node added by resolution or imputation",
      LegendKind::ImputedLeaf => "Imputed leaf",
      LegendKind::Links => "Leaves of one MCC",
      LegendKind::NoMcc => "No MCC",
      LegendKind::SegmentA | LegendKind::SegmentB => "Segment",
      LegendKind::BothSegments => "Both segments",
      LegendKind::Reassortment => "Reassortment",
    }
  }
}

/// A part of a legend symbol, `DrawingRules.legend_symbol_px` wide; each consumer draws it with
/// the widths of the drawing rules.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(tag = "kind", rename_all = "camelCase")]
#[expect(
  variant_size_differences,
  reason = "a legend has a handful of marks, built once per view, so the size of a mark does not matter"
)]
pub enum LegendMark {
  /// A branch across the symbol, `dashed` with `DrawingRules.dash_px`.
  Branch {
    color: ColorRole,
    stroke: Stroke,
    dashed: bool,
  },
  /// The links of one MCC as an S-curve, drawn where the drawing shows each link as a curve.
  LinkCurve { color: ColorRole },
  /// The links of one MCC as a ribbon, drawn where the drawing shows ribbons
  /// (`DrawingRules::ribbons_shown`); a symbol draws either this or `LinkCurve`.
  LinkRibbon { color: ColorRole },
  /// A reticulation edge: a dashed S-curve into the hybrid node.
  Reticulation { color: ColorRole },
  /// A mark ring at the fraction `at` of the symbol width.
  Ring { color: ColorRole, at: f64 },
}

/// The stroke width of a legend branch: `DrawingRules.branch_width_px` or
/// `reassortment_width_px`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub enum Stroke {
  Branch,
  Reassortment,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::display::{ArgEdgeShape, Bezier, EdgePath, Elbow, LinkCurve, Mark};
  use pretty_assertions::assert_eq;

  fn elbow(color: ColorRole, mcc_break: bool, added: bool) -> Elbow {
    Elbow {
      node: 1,
      points: [[0.0, 0.0]; 3],
      mcc: None,
      slot: None,
      mcc_break,
      added,
      color,
    }
  }

  fn kinds(items: &[LegendItem]) -> Vec<(LegendKind, &str)> {
    items.iter().map(|i| (i.kind, i.label.as_str())).collect()
  }

  #[test]
  fn pair_legend_has_an_entry_for_each_kind_of_shape_in_the_drawing() {
    let left = TreeShapes {
      elbows: vec![
        elbow(ColorRole::Signal, true, false),
        elbow(ColorRole::InkMuted, false, true),
        elbow(ColorRole::NoMcc, false, false),
      ],
      marks: vec![Mark {
        kind: MarkKind::Imputed,
        node: 1,
        mcc: Some(0),
        slot: Some(0),
        at: [1.0, 0.0],
        color: ColorRole::Mcc,
      }],
      leaders: Vec::new(),
    };
    let shapes = PairShapes {
      left,
      right: TreeShapes {
        elbows: Vec::new(),
        marks: Vec::new(),
        leaders: Vec::new(),
      },
      links: vec![LinkCurve {
        link: 0,
        mcc: 0,
        slot: 0,
        curve: Bezier {
          from: [0.0, 0.0],
          c1: [0.5, 0.0],
          c2: [0.5, 1.0],
          to: [1.0, 1.0],
        },
      }],
      ribbons: Vec::new(),
    };
    // Oracle: the order and the labels of the legend of the SVG tanglegram.
    let expected = vec![
      (LegendKind::ReassortmentBranch, "Reassortment branch"),
      (LegendKind::AddedNode, "Node added by resolution or imputation"),
      (LegendKind::ImputedLeaf, "Imputed leaf"),
      (LegendKind::Links, "Leaves of one MCC"),
      (LegendKind::NoMcc, "No MCC"),
    ];
    assert_eq!(expected, kinds(&pair_legend(&shapes)));
  }

  #[test]
  fn pair_legend_leaves_out_shapes_that_the_drawing_lacks() {
    // A reassortment branch of an added node is drawn as a reassortment branch only.
    let shapes = PairShapes {
      left: TreeShapes {
        elbows: vec![elbow(ColorRole::Signal, true, true)],
        marks: Vec::new(),
        leaders: Vec::new(),
      },
      right: TreeShapes {
        elbows: Vec::new(),
        marks: Vec::new(),
        leaders: Vec::new(),
      },
      links: Vec::new(),
      ribbons: Vec::new(),
    };
    assert_eq!(
      vec![(LegendKind::ReassortmentBranch, "Reassortment branch")],
      kinds(&pair_legend(&shapes))
    );
  }

  #[test]
  fn arg_legend_names_the_segments_and_draws_reassortment_in_the_reticulation_color() {
    let reticulation = ArgEdgeShape {
      edge: 0,
      segments: vec![1],
      reticulation: true,
      path: EdgePath::Elbow {
        points: [[0.0, 0.0]; 3],
      },
      color: ColorRole::SegmentB,
    };
    let shapes = ArgShapes {
      edges: vec![reticulation],
      marks: Vec::new(),
      leaders: Vec::new(),
    };
    let legend = arg_legend(&shapes, ["ha", "na"]);
    let expected = (
      vec![
        (LegendKind::SegmentA, "Segment ha"),
        (LegendKind::SegmentB, "Segment na"),
        (LegendKind::BothSegments, "Both segments"),
        (LegendKind::Reassortment, "Reassortment"),
      ],
      Some(vec![
        LegendMark::Reticulation {
          color: ColorRole::SegmentB,
        },
        LegendMark::Ring {
          color: ColorRole::Signal,
          at: RING_AT_CURVE_END,
        },
      ]),
    );
    assert_eq!(expected, (kinds(&legend), legend.last().map(|i| i.marks.clone())));
  }
}
