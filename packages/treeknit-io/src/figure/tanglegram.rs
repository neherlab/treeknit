//! The SVG tanglegram of a pair: the columns of the drawing rules, then the shapes of the pair
//! view mapped to px.

use super::svg::{
  BRANCH_WIDTH, Column, DASH, DOT, LABEL_FONT_FAMILY, LABEL_GAP, LEADER_OPACITY, LEADER_WIDTH, LINK_WIDTH, LegendEntry,
  MARGIN, Path, REASSORTMENT_WIDTH, RIBBON_OPACITY, Rows, Svg, Symbol, baseline, drawing_top, figure_height,
  label_column, legend_top, num,
};
use super::{FigureOptions, labels_shown};
use crate::display::{DRAWING_RULES, DrawTree, Elbow, MarkKind, PairView, TreeShapes, shorten};
use crate::palette::{ThemeColors, palette};

/// The label column takes at most this share of half the inner width.
const LABEL_COLUMN_MAX_SHARE: f64 = 0.25;
/// The link zone takes this share of the inner width, at least the 15% of the drawing rules.
const LINK_ZONE_SHARE: f64 = 0.2;

/// The SVG text of the tanglegram of `view` with valid `options`.
pub(super) fn draw(view: &PairView, options: &FigureOptions) -> String {
  let colors = palette().light;
  let layout = Layout::new(view, options);
  let title = format!("{} and {}", view.left.label, view.right.label);
  let legend = legend(view, &layout, &colors);
  let height = figure_height(layout.rows_count, options.row_height, &legend, options.width);
  let mut svg = Svg::new(options.width, height, &title, &colors.ground);
  svg.title(&title, &colors.ink);
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
    if layout.max_label_chars > 0 {
      draw.leaders(&mut svg);
    }
    draw.branches(&mut svg);
    draw.marks(&mut svg);
    if layout.max_label_chars > 0 {
      draw.labels(&mut svg, side, layout.max_label_chars);
    }
  }
  svg.legend(
    &legend,
    legend_top(layout.rows_count, options.row_height),
    options.width,
    &colors.ink,
  );
  svg.finish()
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
  /// Labels are shortened to this many characters; 0 when labels are not drawn.
  max_label_chars: usize,
}

impl Layout {
  /// From left to right: the left tree, its labels, the link zone, the right labels, and the
  /// right tree, mirrored. A label column is as wide as its longest label, at most a quarter of
  /// half the inner width; labels that do not fit are shortened.
  fn new(view: &PairView, options: &FigureOptions) -> Layout {
    let inner = (options.width - 2.0 * MARGIN).max(0.0);
    let labels = label_column(
      leaf_names(&view.left).chain(leaf_names(&view.right)),
      labels_shown(options),
      LABEL_COLUMN_MAX_SHARE * inner / 2.0,
    );
    let label = labels.width;
    let links = LINK_ZONE_SHARE * inner;
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
      max_label_chars: labels.max_chars,
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
  /// The dotted lines from the leaf tips to the label edge; none for a tip at the edge.
  fn leaders(&self, svg: &mut Svg) {
    let leaders: Vec<_> = self.shapes.leaders.iter().filter(|l| l.from[0] < l.to[0]).collect();
    if leaders.is_empty() {
      return;
    }
    svg.open(
      "g",
      &[
        ("fill", "none".to_owned()),
        ("stroke", self.colors.ink_muted.clone()),
        ("stroke-opacity", num(LEADER_OPACITY)),
        ("stroke-width", num(LEADER_WIDTH)),
        ("stroke-dasharray", DOT.to_owned()),
      ],
    );
    for leader in leaders {
      let from = self.rows.point(self.column, leader.from);
      let to = self.rows.point(self.column, leader.to);
      svg.path(&Path::new().move_to(from).h(to[0]), &[]);
    }
    svg.close("g");
  }

  /// The elbows: in the color of their MCC, dashed ink-muted for added nodes, and signal and
  /// wider for reassortment branches, drawn last so they stay on top.
  fn branches(&self, svg: &mut Svg) {
    svg.open("g", &[("fill", "none".to_owned()), ("stroke-width", num(BRANCH_WIDTH))]);
    let plain = |e: &&Elbow| !e.mcc_break && !e.added;
    let added = |e: &&Elbow| !e.mcc_break && e.added;
    for elbow in self.shapes.elbows.iter().filter(plain) {
      svg.path(
        &self.rows.elbow(self.column, &elbow.points),
        &[("stroke", slot_color(self.colors, elbow.slot))],
      );
    }
    for elbow in self.shapes.elbows.iter().filter(added) {
      svg.path(
        &self.rows.elbow(self.column, &elbow.points),
        &[
          ("stroke", self.colors.ink_muted.clone()),
          ("stroke-dasharray", DASH.to_owned()),
        ],
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
  fn labels(&self, svg: &mut Svg, side: Side, max_chars: usize) {
    let (x, anchor) = match side {
      Side::Left => (self.column.end + LABEL_GAP, "start"),
      Side::Right => (self.column.end - LABEL_GAP, "end"),
    };
    svg.open(
      "g",
      &[
        ("font-family", LABEL_FONT_FAMILY.to_owned()),
        ("fill", self.colors.ink.clone()),
        ("text-anchor", anchor.to_owned()),
      ],
    );
    for node in self.tree.nodes.iter().filter(|n| n.leaf) {
      svg.text(
        "text",
        &[("x", num(x)), ("y", num(baseline(self.rows.y(node.y))))],
        &shorten(&node.name, max_chars),
      );
    }
    svg.close("g");
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
  let line = |color: &str, width: f64, dash: Option<&'static str>| Symbol::Line {
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
        ring(&colors.signal, 0.5),
      ],
      label: "Reassortment branch".to_owned(),
    });
  }
  if elbows().any(|e| e.added && !e.mcc_break) {
    entries.push(LegendEntry {
      symbol: vec![line(&colors.ink_muted, BRANCH_WIDTH, Some(DASH))],
      label: "Node added by resolution".to_owned(),
    });
  }
  if marks().any(|m| m.kind == MarkKind::Imputed) {
    entries.push(LegendEntry {
      symbol: vec![line(&mcc_color, BRANCH_WIDTH, None), ring(&mcc_color, 0.75)],
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
  if elbows().any(|e| e.slot.is_none() && !e.added && !e.mcc_break) {
    entries.push(LegendEntry {
      symbol: vec![line(&colors.no_mcc, BRANCH_WIDTH, None)],
      label: "No MCC".to_owned(),
    });
  }
  entries
}

/// The color of MCC slot `slot`, or the "no MCC" color.
fn slot_color(colors: &ThemeColors, slot: Option<usize>) -> String {
  slot.and_then(|s| colors.mcc.get(s)).unwrap_or(&colors.no_mcc).clone()
}
