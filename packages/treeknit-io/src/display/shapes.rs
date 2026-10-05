//! The shapes of the drawing rules in normalized units: x from 0 to 1 across the column of a
//! shape, y in leaf rows. The SVG figures and the interactive views draw only these shapes.

use super::{
  ArgEdge, ArgEdgeShape, ArgNodeView, ArgShapes, Bezier, Block, DrawNode, DrawTree, EdgePath, Elbow, Link, LinkCurve,
  Mark, MarkKind, PairShapes, Point, Ribbon, Scale, TreeShapes,
};

/// Half a leaf row: a ribbon extends each y range of its block by this much.
const HALF_ROW: f64 = 0.5;

/// The shapes of a tanglegram, with `slots[mcc]` the color slot of each MCC.
pub(super) fn pair_shapes(
  left: &DrawTree,
  right: &DrawTree,
  links: &[Link],
  blocks: &[Block],
  slots: &[usize],
  scale: Scale,
) -> PairShapes {
  PairShapes {
    left: tree_shapes(left, slots, scale),
    right: tree_shapes(right, slots, scale),
    links: links
      .iter()
      .enumerate()
      .map(|(i, l)| LinkCurve {
        link: i,
        slot: slots[l.mcc],
        curve: s_curve([0.0, left.nodes[l.left].y], [1.0, right.nodes[l.right].y]),
      })
      .collect(),
    ribbons: blocks
      .iter()
      .enumerate()
      .map(|(i, b)| Ribbon {
        block: i,
        slot: slots[b.mcc],
        outline: ribbon(b),
      })
      .collect(),
  }
}

/// Elbows and marks of one tree.
fn tree_shapes(tree: &DrawTree, slots: &[usize], scale: Scale) -> TreeShapes {
  let x = normalized(tree.nodes.iter().map(|n| scaled(n, scale)));
  let mut elbows = Vec::new();
  let mut marks = Vec::new();
  for (i, node) in tree.nodes.iter().enumerate() {
    let at = [x[i], node.y];
    if let Some(p) = node.parent {
      let from = [x[p], tree.nodes[p].y];
      elbows.push(Elbow {
        node: i,
        points: elbow(from, at),
        slot: node.mcc.map(|m| slots[m]),
        mcc_break: node.mcc_break,
        added: node.added,
      });
      if node.mcc_break {
        marks.push(Mark {
          kind: MarkKind::Reassortment,
          node: i,
          at: [f64::midpoint(from[0], at[0]), at[1]],
        });
      }
    }
    if node.imputed {
      marks.push(Mark {
        kind: MarkKind::Imputed,
        node: i,
        at,
      });
    }
  }
  TreeShapes { elbows, marks }
}

/// The edges and hybrid rings of an ARG: an elbow per edge, and an S-curve per reticulation
/// edge.
pub(super) fn arg_shapes(nodes: &[ArgNodeView], edges: &[ArgEdge], scale: Scale) -> ArgShapes {
  let x = normalized(nodes.iter().map(|n| match scale {
    Scale::Div => n.x_div,
    Scale::Depth => n.x_depth,
  }));
  let point = |n: usize| [x[n], nodes[n].y];
  ArgShapes {
    edges: edges
      .iter()
      .enumerate()
      .map(|(i, e)| ArgEdgeShape {
        edge: i,
        path: if e.reticulation {
          EdgePath::Curve {
            curve: s_curve(point(e.parent), point(e.child)),
          }
        } else {
          EdgePath::Elbow {
            points: elbow(point(e.parent), point(e.child)),
          }
        },
      })
      .collect(),
    marks: nodes
      .iter()
      .enumerate()
      .filter(|(_, n)| n.hybrid)
      .map(|(i, _)| Mark {
        kind: MarkKind::Hybrid,
        node: i,
        at: point(i),
      })
      .collect(),
  }
}

fn scaled(node: &DrawNode, scale: Scale) -> f64 {
  match scale {
    Scale::Div => node.x_div,
    Scale::Depth => node.x_depth,
  }
}

/// `values` divided by their largest value, so they run from 0 to 1; all 0 when the largest is
/// not positive. The values are finite and at least 0.
fn normalized(values: impl Iterator<Item = f64>) -> Vec<f64> {
  let values: Vec<f64> = values.collect();
  let max = values.iter().copied().fold(0.0, f64::max);
  if max > 0.0 {
    values.iter().map(|v| v / max).collect()
  } else {
    vec![0.0; values.len()]
  }
}

/// The rectangular branch from `parent` to `node`: down the parent's x, then across.
fn elbow(parent: Point, node: Point) -> [Point; 3] {
  [parent, [parent[0], node[1]], node]
}

/// The S-curve from `from` to `to`, with both control points at the middle x.
fn s_curve(from: Point, to: Point) -> Bezier {
  let mid = f64::midpoint(from[0], to[0]);
  Bezier {
    from,
    c1: [mid, from[1]],
    c2: [mid, to[1]],
    to,
  }
}

/// A straight segment as a cubic Bézier.
fn line(from: Point, to: Point) -> Bezier {
  let at = |t: f64| [from[0] + t * (to[0] - from[0]), from[1] + t * (to[1] - from[1])];
  Bezier {
    from,
    c1: at(1.0 / 3.0),
    c2: at(2.0 / 3.0),
    to,
  }
}

/// The closed outline of the ribbon of `block`, clockwise from its top left: the S-curve along
/// the top, the right side, the S-curve back along the bottom, and the left side. Each y range
/// is extended by half a row.
fn ribbon(block: &Block) -> Vec<Bezier> {
  let span = |r: [f64; 2]| (r[0].min(r[1]) - HALF_ROW, r[0].max(r[1]) + HALF_ROW);
  let (left_top, left_bottom) = span(block.left);
  let (right_top, right_bottom) = span(block.right);
  vec![
    s_curve([0.0, left_top], [1.0, right_top]),
    line([1.0, right_top], [1.0, right_bottom]),
    s_curve([1.0, right_bottom], [0.0, left_bottom]),
    line([0.0, left_bottom], [0.0, left_top]),
  ]
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  fn node(parent: Option<usize>, x_div: f64, y: f64) -> DrawNode {
    DrawNode {
      name: String::new(),
      parent,
      children: vec![],
      branch_length: None,
      x_div,
      x_depth: x_div,
      y,
      leaf: false,
      added: false,
      imputed: false,
      mcc: Some(0),
      mcc_break: false,
    }
  }

  #[test]
  fn tree_shapes_of_three_leaves_are_elbows_in_column_units() {
    // ((A:1,B:3):1,C:2): root at x 0, AB at 1, A at 2, B at 4, C at 2; normalized by 4.
    let tree = DrawTree {
      label: "t".to_owned(),
      nodes: vec![
        node(None, 0.0, 1.25),
        node(Some(0), 1.0, 0.5),
        node(Some(1), 2.0, 0.0),
        node(Some(1), 4.0, 1.0),
        node(Some(0), 2.0, 2.0),
      ],
    };
    let shapes = tree_shapes(&tree, &[3], Scale::Div);
    let points: Vec<(usize, [Point; 3])> = shapes.elbows.iter().map(|e| (e.node, e.points)).collect();
    let expected = vec![
      (1, [[0.0, 1.25], [0.0, 0.5], [0.25, 0.5]]),
      (2, [[0.25, 0.5], [0.25, 0.0], [0.5, 0.0]]),
      (3, [[0.25, 0.5], [0.25, 1.0], [1.0, 1.0]]),
      (4, [[0.0, 1.25], [0.0, 2.0], [0.5, 2.0]]),
    ];
    assert_eq!(expected, points);
    assert!(shapes.elbows.iter().all(|e| e.slot == Some(3)));
    assert_eq!(Vec::<Mark>::new(), shapes.marks);
  }

  #[test]
  fn tree_shapes_mark_reassortment_midpoints_and_imputed_tips() {
    let mut x = node(Some(0), 2.0, 1.0);
    x.mcc_break = true;
    x.imputed = true;
    x.added = true;
    let tree = DrawTree {
      label: "t".to_owned(),
      nodes: vec![node(None, 0.0, 0.5), node(Some(0), 1.0, 0.0), x],
    };
    let shapes = tree_shapes(&tree, &[0], Scale::Div);
    let expected = vec![
      Mark {
        kind: MarkKind::Reassortment,
        node: 2,
        at: [0.5, 1.0],
      },
      Mark {
        kind: MarkKind::Imputed,
        node: 2,
        at: [1.0, 1.0],
      },
    ];
    assert_eq!(expected, shapes.marks);
    assert!(shapes.elbows[1].mcc_break && shapes.elbows[1].added);
  }

  #[test]
  fn normalized_of_zero_lengths_is_zero() {
    assert_eq!(vec![0.0, 0.0], normalized([0.0, 0.0].into_iter()));
  }

  #[test]
  fn s_curve_has_its_control_points_at_the_middle_x() {
    let expected = Bezier {
      from: [0.0, 4.0],
      c1: [0.5, 4.0],
      c2: [0.5, 1.0],
      to: [1.0, 1.0],
    };
    assert_eq!(expected, s_curve([0.0, 4.0], [1.0, 1.0]));
  }

  #[test]
  fn ribbon_outline_is_closed_and_extends_each_range_by_half_a_row() {
    // A reversed block: left rows 0..1, right rows 3 down to 2.
    let block = Block {
      mcc: 0,
      left: [0.0, 1.0],
      right: [3.0, 2.0],
    };
    let outline = ribbon(&block);
    assert_eq!(s_curve([0.0, -0.5], [1.0, 1.5]), outline[0]);
    assert_eq!(s_curve([1.0, 3.5], [0.0, 1.5]), outline[2]);
    for (a, b) in outline.iter().zip(outline.iter().cycle().skip(1)) {
      assert_eq!(a.to.map(f64::to_bits), b.from.map(f64::to_bits));
    }
  }
}
