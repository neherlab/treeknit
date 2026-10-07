//! The SVG figure of the ARG of two trees: one tree column with the labels at its right.

use crate::display::{ArgView, DRAWING_RULES, EdgePath};
use crate::figure::svg::{
  BRANCH_WIDTH, Column, DASH, LABEL_GAP, Path, Rows, dash_array, drawing_top, figure, label_column, legend_entries, num,
};
use crate::figure::{FigureOptions, labels_shown};
use crate::palette::palette;
use crate::summary::pair_title;

/// The SVG text of the ARG `view` with valid `options`.
pub(super) fn draw(view: &ArgView, options: &FigureOptions) -> String {
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
  let [a, b] = &view.segments;
  let title = format!("ARG of {}", pair_title(a, b));
  let legend = legend_entries(&view.legend, &colors, false);
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
      svg.path(
        &rows.elbow(column, points),
        &[("stroke", colors.role(shape.color, None).clone())],
      );
    }
  }
  for shape in &view.shapes.edges {
    if let EdgePath::Curve { curve } = &shape.path {
      svg.path(
        &Path::new().curve(&rows.bezier(column, curve)),
        &[
          ("stroke", colors.role(shape.color, None).clone()),
          ("stroke-dasharray", dash_array(DASH)),
        ],
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
