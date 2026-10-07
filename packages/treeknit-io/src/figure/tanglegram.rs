//! The SVG tanglegram of a pair: the columns of the drawing rules, then the shapes of the pair
//! view mapped to px.

use super::svg::{
  BRANCH_WIDTH, Column, LABEL_GAP, LINK_WIDTH, LabelColumn, LegendEntry, MARGIN, Path, REASSORTMENT_WIDTH,
  RIBBON_OPACITY, RING_AT_BRANCH_MIDDLE, RING_AT_LEAF_TIP, Rows, Svg, Symbol, drawing_top, figure, inner_width,
  label_column, num,
};
use super::{FigureOptions, labels_shown};
use crate::display::{DRAWING_RULES, DrawTree, Elbow, MarkKind, PairView, TreeShapes};
use crate::palette::{ThemeColors, palette};

/// The SVG text of the tanglegram of `view` with valid `options`.
pub(super) fn draw(view: &PairView, options: &FigureOptions) -> String {
  let colors = palette().light;
  let layout = Layout::new(view, options);
  let title = format!("{} and {}", view.left.label, view.right.label);
  let legend = legend(view, &layout, &colors);
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
      view,
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
    let rules = DRAWING_RULES;
    let inner = inner_width(options.width);
    let labels = label_column(
      leaf_names(&view.left).chain(leaf_names(&view.right)),
      labels_shown(options),
      rules.tanglegram_label_column_max_share * inner / 2.0,
    );
    let label = labels.width;
    let links = (rules.link_zone_share * inner - 2.0 * label).max(rules.link_zone_min_share * inner);
    let tree = ((inner - links - 2.0 * label) / 2.0).max(0.0);
    let left_end = MARGIN + tree;
    let links_start = left_end + label;
    let links_end = links_start + links;
    let right_start = links_end + label;
    Layout {
      left: Column {
        start: MARGIN,
        end: left_end,
      },
      links: Column {
        start: links_start,
        end: links_end,
      },
      right: Column {
        start: right_start + tree,
        end: right_start,
      },
      rows: Rows {
        top: drawing_top(),
        row_height: options.row_height,
      },
      rows_count: leaf_names(&view.left)
        .count()
        .max(leaf_names(&view.right).count())
        .max(1),
      ribbons: options.row_height < f64::from(DRAWING_RULES.link_min_row_px),
      labels,
    }
  }
}

fn leaf_names(tree: &DrawTree) -> impl Iterator<Item = &str> {
  tree.nodes.iter().filter(|n| n.leaf).map(|n| n.name.as_str())
}

fn ribbons(svg: &mut Svg, view: &PairView, layout: &Layout, colors: &ThemeColors) {
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
    svg.path(&d, &[("fill", slot_color(colors, Some(ribbon.slot)))]);
  }
  svg.close("g");
}

fn links(svg: &mut Svg, view: &PairView, layout: &Layout, colors: &ThemeColors) {
  svg.open("g", &[("fill", "none".to_owned()), ("stroke-width", num(LINK_WIDTH))]);
  for link in &view.shapes.links {
    let d = Path::new().curve(&layout.rows.bezier(layout.links, &link.curve));
    svg.path(&d, &[("stroke", slot_color(colors, Some(link.slot)))]);
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
  view: &'a PairView,
  tree: &'a DrawTree,
  shapes: &'a TreeShapes,
  column: Column,
  rows: Rows,
  colors: &'a ThemeColors,
}

impl TreeDrawing<'_> {
  /// The elbows: in the color of their MCC, the part across of an added node in ink-muted,
  /// and signal and wider for reassortment branches, drawn last so they stay on top.
  fn branches(&self, svg: &mut Svg) {
    svg.open("g", &[("fill", "none".to_owned()), ("stroke-width", num(BRANCH_WIDTH))]);
    let added = |e: &&Elbow| !e.mcc_break && e.added;
    for elbow in self.shapes.elbows.iter().filter(|e| !e.mcc_break) {
      let path = if elbow.added {
        self.rows.elbow_stem(self.column, &elbow.points)
      } else {
        self.rows.elbow(self.column, &elbow.points)
      };
      svg.path(&path, &[("stroke", slot_color(self.colors, elbow.slot))]);
    }
    for elbow in self.shapes.elbows.iter().filter(added) {
      svg.path(
        &self.rows.elbow_across(self.column, &elbow.points),
        &[("stroke", self.colors.ink_muted.clone())],
      );
    }
    for elbow in self.shapes.elbows.iter().filter(|e| e.mcc_break) {
      svg.path(
        &self.rows.elbow(self.column, &elbow.points),
        &[
          ("stroke", self.colors.signal.clone()),
          ("stroke-width", num(REASSORTMENT_WIDTH)),
        ],
      );
    }
    svg.close("g");
  }

  /// Hollow circles at imputed tips in the color of their MCC, then signal rings at the
  /// midpoints of reassortment branches.
  fn marks(&self, svg: &mut Svg) {
    for kind in [MarkKind::Imputed, MarkKind::Reassortment] {
      for mark in self.shapes.marks.iter().filter(|m| m.kind == kind) {
        let stroke = if kind == MarkKind::Reassortment {
          self.colors.signal.clone()
        } else {
          let mcc = self.tree.nodes[mark.node].mcc;
          slot_color(self.colors, mcc.map(|m| self.view.mccs[m].slot))
        };
        svg.ring(self.rows.point(self.column, mark.at), &stroke, &self.colors.ground);
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
    let labels = self
      .tree
      .nodes
      .iter()
      .filter(|n| n.leaf)
      .map(|n| (n.name.as_str(), n.y));
    svg.labels(labels, self.rows, column, x, Some(anchor), &self.colors.ink);
  }
}

/// The legend: the symbols that the drawing contains.
fn legend(view: &PairView, layout: &Layout, colors: &ThemeColors) -> Vec<LegendEntry> {
  let elbows = || view.shapes.left.elbows.iter().chain(&view.shapes.right.elbows);
  let marks = || view.shapes.left.marks.iter().chain(&view.shapes.right.marks);
  let ring = |stroke: &str, at: f64| Symbol::Ring {
    stroke: stroke.to_owned(),
    ground: colors.ground.clone(),
    at,
  };
  let line = |color: &str, width: f64, dash: Option<[f64; 2]>| Symbol::Line {
    color: color.to_owned(),
    width,
    dash,
  };
  let mcc_color = slot_color(colors, Some(0));
  let mut entries = Vec::new();
  if elbows().any(|e| e.mcc_break) {
    entries.push(LegendEntry {
      symbol: vec![
        line(&colors.signal, REASSORTMENT_WIDTH, None),
        ring(&colors.signal, RING_AT_BRANCH_MIDDLE),
      ],
      label: "Reassortment branch".to_owned(),
    });
  }
  if elbows().any(|e| e.added && !e.mcc_break) {
    entries.push(LegendEntry {
      symbol: vec![line(&colors.ink_muted, BRANCH_WIDTH, None)],
      label: "Node added by resolution or imputation".to_owned(),
    });
  }
  if marks().any(|m| m.kind == MarkKind::Imputed) {
    entries.push(LegendEntry {
      symbol: vec![line(&mcc_color, BRANCH_WIDTH, None), ring(&mcc_color, RING_AT_LEAF_TIP)],
      label: "Imputed leaf".to_owned(),
    });
  }
  if !view.links.is_empty() {
    let symbol = if layout.ribbons {
      Symbol::Ribbon { color: mcc_color }
    } else {
      Symbol::Curve {
        color: mcc_color,
        dash: None,
      }
    };
    entries.push(LegendEntry {
      symbol: vec![symbol],
      label: "Leaves of one MCC".to_owned(),
    });
  }
  if elbows().any(|e| e.slot.is_none() && !e.mcc_break) {
    entries.push(LegendEntry {
      symbol: vec![line(&colors.no_mcc, BRANCH_WIDTH, None)],
      label: "No MCC".to_owned(),
    });
  }
  entries
}

/// The color of MCC slot `slot`, or the "no MCC" color for `None`. A slot is below `MCC_SLOTS`,
/// so a slot out of range is a broken invariant and panics instead of taking the "no MCC" color.
fn slot_color(colors: &ThemeColors, slot: Option<usize>) -> String {
  slot.map_or(&colors.no_mcc, |s| &colors.mcc[s]).clone()
}
