//! The SVG tanglegram of a pair: the columns of the drawing rules, then the shapes of the pair
//! view mapped to px.

use crate::display::{ColorRole, DRAWING_RULES, DrawTree, Elbow, MarkKind, PairView, TreeShapes};
use crate::figure::svg::{
  BRANCH_WIDTH, Column, LABEL_GAP, LINK_WIDTH, LabelColumn, Path, REASSORTMENT_WIDTH, RIBBON_OPACITY, Rows, Svg,
  drawing_top, figure, label_column, legend_entries, num,
};
use crate::figure::{FigureOptions, labels_shown};
use crate::palette::{ThemeColors, palette};
use crate::summary::pair_title;

/// The SVG text of the tanglegram of `view` with valid `options`.
pub(super) fn draw(view: &PairView, options: &FigureOptions) -> String {
  let colors = palette().light;
  let layout = Layout::new(view, options);
  let title = pair_title(&view.left.label, &view.right.label);
  let legend = legend_entries(&view.legend, &colors, layout.ribbons);
  let mut svg = figure(
    &title,
    layout.rows_count,
    options.row_height,
    options.width,
    &legend,
    [&colors.ground, &colors.ink],
  );
  if layout.ribbons {
    ribbons(&mut svg, view, &layout, &colors);
  } else {
    links(&mut svg, view, &layout, &colors);
  }
  for (tree, shapes, column, side) in [
    (&view.left, &view.shapes.left, layout.left, Side::Left),
    (&view.right, &view.shapes.right, layout.right, Side::Right),
  ] {
    let draw = TreeDrawing {
      tree,
      shapes,
      column,
      rows: layout.rows,
      colors: &colors,
    };
    if layout.labels.shown() {
      let labeled = |n: usize| !layout.labels.text(&tree.nodes[n].name).is_empty();
      svg.leaders(&shapes.leaders, labeled, layout.rows, column, &colors.ink_muted);
    }
    draw.branches(&mut svg);
    draw.marks(&mut svg);
    if layout.labels.shown() {
      draw.labels(&mut svg, side, &layout.labels);
    }
  }
  svg.finish_figure(
    layout.rows_count,
    options.row_height,
    options.width,
    &legend,
    &colors.ink,
  )
}

/// The columns and rows of a tanglegram in px.
struct Layout {
  left: Column,
  links: Column,
  right: Column,
  rows: Rows,
  rows_count: usize,
  /// Each block is a ribbon, instead of one S-curve per link.
  ribbons: bool,
  /// The two label columns.
  labels: LabelColumn,
}

impl Layout {
  /// The columns of `DrawingRules`; labels that do not fit their column are shortened.
  fn new(view: &PairView, options: &FigureOptions) -> Layout {
    let labels = label_column(
      view.left.leaves().chain(view.right.leaves()).map(|n| n.name.as_str()),
      labels_shown(options),
      DRAWING_RULES.tanglegram_label_max_px(options.width),
    );
    let columns = DRAWING_RULES.tanglegram_columns(options.width, labels.width);
    Layout {
      left: Column {
        start: columns.left.start,
        end: columns.left.end,
      },
      links: Column {
        start: columns.links.start,
        end: columns.links.end,
      },
      // The right tree is mirrored: its root is at the right edge.
      right: Column {
        start: columns.right.end,
        end: columns.right.start,
      },
      rows: Rows {
        top: drawing_top(),
        row_height: options.row_height,
      },
      rows_count: view.left.leaves().count().max(view.right.leaves().count()).max(1),
      ribbons: DRAWING_RULES.ribbons_shown(options.row_height),
      labels,
    }
  }
}

fn ribbons(svg: &mut Svg, view: &PairView, layout: &Layout, colors: &ThemeColors<String>) {
  svg.open("g", &[("fill-opacity", num(RIBBON_OPACITY))]);
  for ribbon in &view.shapes.ribbons {
    let d = ribbon
      .outline
      .iter()
      .enumerate()
      .fold(Path::new(), |d, (i, c)| {
        let c = layout.rows.bezier(layout.links, c);
        if i == 0 { d.curve(&c) } else { d.curve_to(&c) }
      })
      .close();
    svg.path(&d, &[("fill", colors.role(ColorRole::Mcc, Some(ribbon.slot)).clone())]);
  }
  svg.close("g");
}

fn links(svg: &mut Svg, view: &PairView, layout: &Layout, colors: &ThemeColors<String>) {
  svg.open("g", &[("fill", "none".to_owned()), ("stroke-width", num(LINK_WIDTH))]);
  for link in &view.shapes.links {
    let d = Path::new().curve(&layout.rows.bezier(layout.links, &link.curve));
    svg.path(&d, &[("stroke", colors.role(ColorRole::Mcc, Some(link.slot)).clone())]);
  }
  svg.close("g");
}

/// Which tree of the pair; its labels face the link zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
  Left,
  Right,
}

/// One tree of a tanglegram, drawn into its column.
struct TreeDrawing<'a> {
  tree: &'a DrawTree,
  shapes: &'a TreeShapes,
  column: Column,
  rows: Rows,
  colors: &'a ThemeColors<String>,
}

impl TreeDrawing<'_> {
  /// The elbows in their colors: plain ones, then the part across of added nodes in ink-muted
  /// (their part along the parent's x is drawn like a plain branch), then the wider reassortment
  /// branches, drawn last so they stay on top.
  fn branches(&self, svg: &mut Svg) {
    svg.open("g", &[("fill", "none".to_owned()), ("stroke-width", num(BRANCH_WIDTH))]);
    let stroke = |e: &Elbow| ("stroke", self.colors.role(e.color, e.slot).clone());
    let added = |e: &&Elbow| !e.mcc_break && e.added;
    for elbow in self.shapes.elbows.iter().filter(|e| !e.mcc_break) {
      let path = if elbow.added {
        self.rows.elbow_stem(self.column, &elbow.points)
      } else {
        self.rows.elbow(self.column, &elbow.points)
      };
      svg.path(&path, &[stroke(elbow)]);
    }
    for elbow in self.shapes.elbows.iter().filter(added) {
      svg.path(
        &self.rows.elbow_across(self.column, &elbow.points),
        &[("stroke", self.colors.role(ColorRole::InkMuted, elbow.slot).clone())],
      );
    }
    for elbow in self.shapes.elbows.iter().filter(|e| e.mcc_break) {
      svg.path(
        &self.rows.elbow(self.column, &elbow.points),
        &[stroke(elbow), ("stroke-width", num(REASSORTMENT_WIDTH))],
      );
    }
    svg.close("g");
  }

  /// Hollow circles at imputed tips in the color of their MCC, then signal rings at the
  /// midpoints of reassortment branches.
  fn marks(&self, svg: &mut Svg) {
    for kind in [MarkKind::Imputed, MarkKind::Reassortment] {
      for mark in self.shapes.marks.iter().filter(|m| m.kind == kind) {
        let stroke = self.colors.role(mark.color, mark.slot);
        svg.ring(self.rows.point(self.column, mark.at), stroke, &self.colors.ground);
      }
    }
  }

  /// The leaf labels in the label column next to the tree: left-aligned right of the left tree,
  /// right-aligned left of the right tree.
  fn labels(&self, svg: &mut Svg, side: Side, column: &LabelColumn) {
    let (x, anchor) = match side {
      Side::Left => (self.column.end + LABEL_GAP, "start"),
      Side::Right => (self.column.end - LABEL_GAP, "end"),
    };
    let labels = self.tree.leaves().map(|n| (n.name.as_str(), n.y));
    svg.labels(labels, self.rows, column, x, Some(anchor), &self.colors.ink);
  }
}
