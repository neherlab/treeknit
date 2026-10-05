//! The SVG figure of the ARG of two trees: one tree column with the labels at its right.

use super::svg::{
  BRANCH_WIDTH, Column, DASH, DOT, LABEL_FONT_FAMILY, LABEL_GAP, LEADER_OPACITY, LEADER_WIDTH, LegendEntry, MARGIN,
  Path, RING_AT_CURVE_END, Rows, Svg, Symbol, baseline, dash_array, drawing_top, figure_height, label_column,
  legend_top, num,
};
use super::{FigureOptions, labels_shown};
use crate::display::{ArgView, DRAWING_RULES, EdgePath, shorten};
use crate::palette::{ThemeColors, palette};

/// The SVG text of the ARG `view` of the trees labeled `segments` (A, then B) with valid
/// `options`.
pub(super) fn draw(view: &ArgView, segments: [&str; 2], options: &FigureOptions) -> String {
  let colors = palette().light;
  let leaves = || view.nodes.iter().filter(|n| n.leaf);
  let rows_count = leaves().count().max(1);
  let inner = (options.width - 2.0 * MARGIN).max(0.0);
  let labels = label_column(
    leaves().map(|n| n.label.as_str()),
    labels_shown(options),
    DRAWING_RULES.arg_label_column_max_share * inner,
  );
  let (label, max_label_chars) = (labels.width, labels.max_chars);
  let column = Column {
    start: MARGIN,
    end: MARGIN + inner - label,
  };
  let rows = Rows {
    top: drawing_top(),
    row_height: options.row_height,
  };
  let [a, b] = segments;
  let title = format!("ARG of {a} and {b}");
  let legend = legend(view, segments, &colors);
  let height = figure_height(rows_count, options.row_height, &legend, options.width);
  let mut svg = Svg::new(options.width, height, &title, &colors.ground);
  svg.title(&title, &colors.ink);

  let leaders: Vec<_> = view.shapes.leaders.iter().filter(|l| l.from[0] < l.to[0]).collect();
  if max_label_chars > 0 && !leaders.is_empty() {
    svg.open(
      "g",
      &[
        ("fill", "none".to_owned()),
        ("stroke", colors.ink_muted.clone()),
        ("stroke-opacity", num(LEADER_OPACITY)),
        ("stroke-width", num(LEADER_WIDTH)),
        ("stroke-dasharray", dash_array(DOT)),
      ],
    );
    for leader in leaders {
      let from = rows.point(column, leader.from);
      let to = rows.point(column, leader.to);
      svg.path(&Path::new().move_to(from).h(to[0]), &[]);
    }
    svg.close("g");
  }

  svg.open("g", &[("fill", "none".to_owned()), ("stroke-width", num(BRANCH_WIDTH))]);
  // Elbows first, then the dashed reticulations above them.
  for shape in &view.shapes.edges {
    if let EdgePath::Elbow { points } = &shape.path {
      let color = segment_color(&view.edges[shape.edge].segments, &colors);
      svg.path(&rows.elbow(column, points), &[("stroke", color)]);
    }
  }
  for shape in &view.shapes.edges {
    if let EdgePath::Curve { curve } = &shape.path {
      let color = segment_color(&view.edges[shape.edge].segments, &colors);
      svg.path(
        &Path::new().curve(&rows.bezier(column, curve)),
        &[("stroke", color), ("stroke-dasharray", dash_array(DASH))],
      );
    }
  }
  svg.close("g");

  for mark in &view.shapes.marks {
    svg.ring(rows.point(column, mark.at), &colors.signal, &colors.ground);
  }

  if max_label_chars > 0 {
    svg.open(
      "g",
      &[
        ("font-family", LABEL_FONT_FAMILY.to_owned()),
        ("fill", colors.ink.clone()),
      ],
    );
    let x = column.end + LABEL_GAP;
    for node in leaves() {
      svg.text(
        "text",
        &[("x", num(x)), ("y", num(baseline(rows.y(node.y))))],
        &shorten(&node.label, max_label_chars),
      );
    }
    svg.close("g");
  }

  svg.legend(
    &legend,
    legend_top(rows_count, options.row_height),
    options.width,
    &colors.ink,
  );
  svg.finish()
}

/// The legend: the two segments, both, and reassortment when the ARG has a hybrid node.
fn legend(view: &ArgView, [a, b]: [&str; 2], colors: &ThemeColors) -> Vec<LegendEntry> {
  let line = |color: &str| Symbol::Line {
    color: color.to_owned(),
    width: BRANCH_WIDTH,
    dash: None,
  };
  let mut entries = vec![
    LegendEntry {
      symbol: vec![line(&colors.segment_a)],
      label: format!("Segment {a}"),
    },
    LegendEntry {
      symbol: vec![line(&colors.segment_b)],
      label: format!("Segment {b}"),
    },
    LegendEntry {
      symbol: vec![line(&colors.ink)],
      label: "Both segments".to_owned(),
    },
  ];
  if !view.shapes.marks.is_empty() {
    entries.push(LegendEntry {
      symbol: vec![
        Symbol::Curve {
          color: colors.ink_muted.clone(),
          dash: Some(DASH),
        },
        Symbol::Ring {
          stroke: colors.signal.clone(),
          ground: colors.ground.clone(),
          at: RING_AT_CURVE_END,
        },
      ],
      label: "Reassortment".to_owned(),
    });
  }
  entries
}

/// Segment A or B for an edge of one segment, ink for an edge of both.
fn segment_color(segments: &[usize], colors: &ThemeColors) -> String {
  match segments {
    [0] => colors.segment_a.clone(),
    [1] => colors.segment_b.clone(),
    _ => colors.ink.clone(),
  }
}
