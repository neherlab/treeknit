//! The SVG figure of the ARG of two trees: one tree column with the labels at its right.

use crate::display::{ArgView, DRAWING_RULES, EdgePath};
use crate::figure::svg::{
  BRANCH_WIDTH, Column, DASH, LABEL_GAP, LegendEntry, Path, RING_AT_CURVE_END, Rows, Symbol, dash_array, drawing_top,
  figure, label_column, num,
};
use crate::figure::{FigureOptions, labels_shown};
use crate::palette::{ThemeColors, palette};

/// The SVG text of the ARG `view` of the trees labeled `segments` (A, then B) with valid
/// `options`.
pub(super) fn draw(view: &ArgView, segments: [&str; 2], options: &FigureOptions) -> String {
  let colors = palette().light;
  let leaves = || view.nodes.iter().filter(|n| n.leaf);
  let rows_count = leaves().count().max(1);
  let labels = label_column(
    leaves().map(|n| n.label.as_str()),
    labels_shown(options),
    DRAWING_RULES.arg_label_max_px(options.width),
  );
  let arg_column = DRAWING_RULES.arg_column(options.width, labels.width);
  let column = Column {
    start: arg_column.start,
    end: arg_column.end,
  };
  let rows = Rows {
    top: drawing_top(),
    row_height: options.row_height,
  };
  let [a, b] = segments;
  let title = format!("ARG of {a} and {b}");
  let legend = legend(view, segments, &colors);
  let mut svg = figure(
    &title,
    rows_count,
    options.row_height,
    options.width,
    &legend,
    [&colors.ground, &colors.ink],
  );

  if labels.shown() {
    let labeled = |n: usize| !labels.text(&view.nodes[n].label).is_empty();
    svg.leaders(&view.shapes.leaders, labeled, rows, column, &colors.ink_muted);
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

  if labels.shown() {
    let names = leaves().map(|n| (n.label.as_str(), n.y));
    svg.labels(names, rows, &labels, column.end + LABEL_GAP, None, &colors.ink);
  }

  svg.finish_figure(rows_count, options.row_height, options.width, &legend, &colors.ink)
}

/// The legend: the two segments, both, and reassortment when the ARG has a hybrid node, its curve
/// in the color of the first reticulation the figure draws.
fn legend(view: &ArgView, [a, b]: [&str; 2], colors: &ThemeColors<String>) -> Vec<LegendEntry> {
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
  // Each hybrid node has one reticulation edge.
  if let Some(reticulation) = view.shapes.edges.iter().find(|s| s.reticulation) {
    entries.push(LegendEntry {
      symbol: vec![
        Symbol::Curve {
          color: segment_color(&view.edges[reticulation.edge].segments, colors),
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
fn segment_color(segments: &[usize], colors: &ThemeColors<String>) -> String {
  match segments {
    [0] => colors.segment_a.clone(),
    [1] => colors.segment_b.clone(),
    _ => colors.ink.clone(),
  }
}
