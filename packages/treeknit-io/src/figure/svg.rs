//! An SVG document writer over `quick-xml`, which escapes attribute values and text, with the
//! pixel frame, path data, text width estimate, and legend shared by the figures.

use crate::display::{Bezier, DRAWING_RULES, Leader, Point, grapheme_count, label_max_chars, s_curve, shorten};
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use std::borrow::Cow;

/// Space around the drawing, in px.
pub(super) const MARGIN: f64 = DRAWING_RULES.margin_px;
/// Space on each side of a label column, in px.
pub(super) const LABEL_GAP: f64 = DRAWING_RULES.label_gap_px;
/// Font size of the leaf labels and the legend, in px.
pub(super) const FONT_SIZE: f64 = 12.0;
/// Font size of the title, in px.
const TITLE_SIZE: f64 = 16.0;
/// Height of the title band above the drawing, in px.
const TITLE_BAND: f64 = 36.0;
/// Space between the drawing and the legend, in px.
const LEGEND_GAP: f64 = 16.0;
/// Height of a legend line, in px.
const LEGEND_LINE: f64 = 20.0;
/// Width of a legend symbol, in px.
const SYMBOL_WIDTH: f64 = 24.0;
/// Space between a legend symbol and its text, in px.
const SYMBOL_GAP: f64 = 6.0;
/// Space between two legend entries on a line, in px.
const ENTRY_GAP: f64 = 20.0;
/// Advance widths of the printable ASCII characters, U+0020 to U+007E, in 1/1000 em: those of
/// Helvetica (Adobe font metrics of the standard PostScript fonts), which Arial shares. SVG text
/// has no width before a viewer lays it out, so the figures estimate widths from these metrics.
/// They are an estimate, not a bound: a viewer that falls back to a wider font can draw a label
/// past its column.
#[rustfmt::skip]
const ASCII_ADVANCE: [u32; 95] = [
  // space to /
  278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278,
  // 0 to ?
  556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556,
  // @ to O
  1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778,
  // P to _
  667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556,
  // ` to o
  333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556,
  // p to ~
  556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584,
];
/// First character of `LATIN_ADVANCE`.
const LATIN_FIRST: u32 = 0xA0;
/// Advance widths of Latin-1 Supplement and Latin Extended-A, U+00A0 to U+017F, in 1/1000 em:
/// those of the Helvetica glyphs of the characters (Adobe Glyph List names). A letter that
/// Helvetica lacks takes the width of its base letter (`Ĉ` that of `C`), and the ligatures `Ĳ`,
/// `ĳ`, and `ŉ` the sum of their parts.
#[rustfmt::skip]
const LATIN_ADVANCE: [u32; 224] = [
  // U+00A0 to U+00AF
  278, 333, 556, 556, 556, 556, 260, 556, 333, 737, 370, 556, 584, 333, 737, 333,
  // U+00B0 to U+00BF
  400, 584, 333, 333, 333, 556, 537, 278, 333, 333, 365, 556, 834, 834, 834, 611,
  // U+00C0 to U+00CF
  667, 667, 667, 667, 667, 667, 1000, 722, 667, 667, 667, 667, 278, 278, 278, 278,
  // U+00D0 to U+00DF
  722, 722, 778, 778, 778, 778, 778, 584, 778, 722, 722, 722, 722, 667, 667, 611,
  // U+00E0 to U+00EF
  556, 556, 556, 556, 556, 556, 889, 500, 556, 556, 556, 556, 278, 278, 278, 278,
  // U+00F0 to U+00FF
  556, 556, 556, 556, 556, 556, 556, 584, 611, 556, 556, 556, 556, 500, 556, 500,
  // U+0100 to U+010F
  667, 556, 667, 556, 667, 556, 722, 500, 722, 500, 722, 500, 722, 500, 722, 643,
  // U+0110 to U+011F
  722, 556, 667, 556, 667, 556, 667, 556, 667, 556, 667, 556, 778, 556, 778, 556,
  // U+0120 to U+012F
  778, 556, 778, 556, 722, 556, 722, 556, 278, 222, 278, 278, 278, 222, 278, 222,
  // U+0130 to U+013F
  278, 278, 778, 444, 500, 222, 667, 500, 500, 556, 222, 556, 222, 556, 299, 556,
  // U+0140 to U+014F
  222, 556, 222, 722, 556, 722, 556, 722, 556, 778, 667, 556, 778, 556, 778, 556,
  // U+0150 to U+015F
  778, 556, 1000, 944, 722, 333, 722, 333, 722, 333, 667, 500, 667, 500, 667, 500,
  // U+0160 to U+016F
  667, 500, 611, 278, 611, 317, 611, 278, 722, 556, 722, 556, 722, 556, 722, 556,
  // U+0170 to U+017F
  722, 556, 722, 556, 944, 722, 667, 500, 667, 611, 500, 611, 500, 611, 500, 278,
];
/// Characters that take no width: combining marks (U+0300 to U+036F, U+1AB0 to U+1AFF, U+1DC0
/// to U+1DFF, U+20D0 to U+20FF, U+FE20 to U+FE2F), zero-width spaces and joiners and the
/// direction marks (U+200B to U+200F), and variation selectors (U+FE00 to U+FE0F, U+E0100 to
/// U+E01EF).
const ZERO_WIDTH: [(char, char); 8] = [
  ('\u{300}', '\u{36f}'),
  ('\u{1ab0}', '\u{1aff}'),
  ('\u{1dc0}', '\u{1dff}'),
  ('\u{200b}', '\u{200f}'),
  ('\u{20d0}', '\u{20ff}'),
  ('\u{fe00}', '\u{fe0f}'),
  ('\u{fe20}', '\u{fe2f}'),
  ('\u{e0100}', '\u{e01ef}'),
];
/// Typical advance of any other character in 1/1000 em: the full em of a CJK ideograph. The
/// letters of most other scripts are narrower, and a few symbols are wider (U+2E3B is 3 em).
const OTHER_ADVANCE: u32 = 1000;

/// Fonts of the figure text.
const FONT_FAMILY: &str = "Lato, Noto Sans, Helvetica, Arial, sans-serif";

/// Strokes and marks of the drawing rules.
pub(super) const BRANCH_WIDTH: f64 = DRAWING_RULES.branch_width_px;
pub(super) const REASSORTMENT_WIDTH: f64 = DRAWING_RULES.reassortment_width_px;
pub(super) const LINK_WIDTH: f64 = DRAWING_RULES.link_width_px;
const LEADER_WIDTH: f64 = DRAWING_RULES.leader_width_px;
const LEADER_OPACITY: f64 = DRAWING_RULES.leader_opacity;
pub(super) const MARK_RADIUS: f64 = DRAWING_RULES.mark_radius_px;
pub(super) const MARK_WIDTH: f64 = DRAWING_RULES.mark_line_px;
pub(super) const RIBBON_OPACITY: f64 = DRAWING_RULES.ribbon_opacity;
/// Dash patterns: dashed branches and reticulations, dotted leaders.
pub(super) const DASH: [f64; 2] = DRAWING_RULES.dash_px;
const DOT: [f64; 2] = DRAWING_RULES.dot_px;

/// The position of the ring in a legend symbol, as a fraction of the symbol width: a
/// reassortment ring at the middle of its branch, an imputed ring at the tip of a leaf branch with
/// the label gap after it, and a hybrid ring at the end of its reticulation curve. The legend of
/// the interactive views places the first two alike.
pub(super) const RING_AT_BRANCH_MIDDLE: f64 = 0.5;
pub(super) const RING_AT_LEAF_TIP: f64 = 0.75;
pub(super) const RING_AT_CURVE_END: f64 = 1.0;
/// Vertical travel of the S-curve of the link symbol across a legend symbol, in px.
const SYMBOL_CURVE_RISE: f64 = 8.0;
/// Height of the ribbon symbol, and its vertical travel across a legend symbol, in px. The curve
/// and the ribbon have the size of those in the legend of the interactive views.
const SYMBOL_RIBBON_HEIGHT: f64 = 6.0;
const SYMBOL_RIBBON_RISE: f64 = 4.0;
/// Shift from the middle of a text line to its baseline, in em: half the cap height of the label
/// fonts (about 0.7 em in Lato, Helvetica, and Arial), so capitals are centered on their row.
const BASELINE_SHIFT_EM: f64 = 0.35;

/// An SVG document under construction, written with indentation.
pub(super) struct Svg {
  writer: Writer<Vec<u8>>,
}

impl Svg {
  /// A document of `width` by `height` px on the color `ground`, with `title` as its accessible
  /// name and as the heading above the drawing.
  fn new(width: f64, height: f64, title: &str, ground: &str) -> Svg {
    let mut svg = Svg {
      writer: Writer::new_with_indent(Vec::new(), b' ', 2),
    };
    svg.open(
      "svg",
      &[
        ("xmlns", "http://www.w3.org/2000/svg".to_owned()),
        ("width", num(width)),
        ("height", num(height)),
        ("viewBox", format!("0 0 {} {}", num(width), num(height))),
        ("font-family", FONT_FAMILY.to_owned()),
        ("font-size", num(FONT_SIZE)),
      ],
    );
    svg.text("title", &[], title);
    svg.empty(
      "rect",
      &[
        ("width", num(width)),
        ("height", num(height)),
        ("fill", ground.to_owned()),
      ],
    );
    svg
  }

  /// The SVG text, closing the root element.
  fn finish(mut self) -> String {
    self.close("svg");
    #[expect(
      clippy::expect_used,
      reason = "the writer copies UTF-8 text and ASCII markup, so its output is UTF-8"
    )]
    let text = String::from_utf8(self.writer.into_inner()).expect("the SVG text is UTF-8");
    format!("{text}\n")
  }

  /// The SVG text of a figure begun by `figure`, with the legend `entries` under its `rows` rows of
  /// `row_height` px, wrapped at `width` px.
  pub(super) fn finish_figure(
    mut self,
    rows: usize,
    row_height: f64,
    width: f64,
    entries: &[LegendEntry],
    ink: &str,
  ) -> String {
    self.legend(entries, legend_top(rows, row_height), width, ink);
    self.finish()
  }

  /// Open element `name` with `attributes`.
  pub(super) fn open(&mut self, name: &str, attributes: &[(&str, String)]) {
    self.write(Event::Start(start(name, attributes)));
  }

  pub(super) fn close(&mut self, name: &str) {
    self.write(Event::End(BytesEnd::new(name)));
  }

  /// An element without content.
  pub(super) fn empty(&mut self, name: &str, attributes: &[(&str, String)]) {
    self.write(Event::Empty(start(name, attributes)));
  }

  /// An element with text content.
  pub(super) fn text(&mut self, name: &str, attributes: &[(&str, String)], text: &str) {
    self.open(name, attributes);
    self.write(Event::Text(BytesText::new(&xml_chars(text))));
    self.close(name);
  }

  /// A path with `d` and `attributes`.
  pub(super) fn path(&mut self, d: &Path, attributes: &[(&str, String)]) {
    let mut all = vec![("d", d.0.clone())];
    all.extend_from_slice(attributes);
    self.empty("path", &all);
  }

  /// The title heading, left-aligned at the margin.
  fn title(&mut self, title: &str, ink: &str) {
    self.text(
      "text",
      &[
        ("x", num(MARGIN)),
        ("y", num(MARGIN + TITLE_SIZE)),
        ("font-size", num(TITLE_SIZE)),
        ("font-weight", "600".to_owned()),
        ("fill", ink.to_owned()),
      ],
      title,
    );
  }

  /// The legend `entries` from `top`, in lines that wrap at `width` px.
  fn legend(&mut self, entries: &[LegendEntry], top: f64, width: f64, ink: &str) {
    if entries.is_empty() {
      return;
    }
    self.open("g", &[("fill", ink.to_owned())]);
    for (entry, (line, x)) in entries.iter().zip(legend_places(entries, width)) {
      let mid = top + (count(line) + 0.5) * LEGEND_LINE;
      for symbol in &entry.symbol {
        self.legend_symbol(symbol, x, mid);
      }
      self.text(
        "text",
        &[("x", num(x + SYMBOL_WIDTH + SYMBOL_GAP)), ("y", num(baseline(mid)))],
        &entry.label,
      );
    }
    self.close("g");
  }

  fn legend_symbol(&mut self, symbol: &Symbol, x: f64, mid: f64) {
    let line = |dash: Option<[f64; 2]>, color: &str, width: f64| {
      let mut attributes = vec![
        ("fill", "none".to_owned()),
        ("stroke", color.to_owned()),
        ("stroke-width", num(width)),
      ];
      attributes.extend(dash.map(|d| ("stroke-dasharray", dash_array(d))));
      attributes
    };
    match symbol {
      Symbol::Line { color, width, dash } => {
        let d = Path::new().move_to([x, mid]).h(x + SYMBOL_WIDTH);
        self.path(&d, &line(*dash, color, *width));
      },
      Symbol::Curve { color, dash } => {
        let rise = SYMBOL_CURVE_RISE / 2.0;
        let curve = s_curve([x, mid - rise], [x + SYMBOL_WIDTH, mid + rise]);
        self.path(&Path::new().curve(&curve), &line(*dash, color, BRANCH_WIDTH));
      },
      Symbol::Ribbon { color } => {
        let (rise, half) = (SYMBOL_RIBBON_RISE / 2.0, SYMBOL_RIBBON_HEIGHT / 2.0);
        let (left, right) = (mid - rise, mid + rise);
        let top = s_curve([x, left - half], [x + SYMBOL_WIDTH, right - half]);
        let bottom = s_curve([x + SYMBOL_WIDTH, right + half], [x, left + half]);
        let d = Path::new().curve(&top).v(right + half).curve_to(&bottom).close();
        self.path(&d, &[("fill", color.clone()), ("fill-opacity", num(RIBBON_OPACITY))]);
      },
      Symbol::Ring { stroke, ground, at } => {
        self.ring([x + at * SYMBOL_WIDTH, mid], stroke, ground);
      },
    }
  }

  /// The dotted `leaders` from the leaf tips to the label edge, in `column` of `rows`, for the
  /// leaves whose node index `labeled` accepts (the leaves with a label text); none for a tip at
  /// the edge.
  pub(super) fn leaders(
    &mut self,
    leaders: &[Leader],
    labeled: impl Fn(usize) -> bool,
    rows: Rows,
    column: Column,
    ink_muted: &str,
  ) {
    let leaders: Vec<_> = leaders
      .iter()
      .filter(|l| l.from[0] < l.to[0] && labeled(l.node))
      .collect();
    if leaders.is_empty() {
      return;
    }
    self.open(
      "g",
      &[
        ("fill", "none".to_owned()),
        ("stroke", ink_muted.to_owned()),
        ("stroke-opacity", num(LEADER_OPACITY)),
        ("stroke-width", num(LEADER_WIDTH)),
        ("stroke-dasharray", dash_array(DOT)),
      ],
    );
    for leader in leaders {
      let from = rows.point(column, leader.from);
      let to = rows.point(column, leader.to);
      self.path(&Path::new().move_to(from).h(to[0]), &[]);
    }
    self.close("g");
  }

  /// The leaf `labels`, each a name and its row, in `column` at `x` with the text `anchor`
  /// (`start` when `None`).
  pub(super) fn labels<'a>(
    &mut self,
    labels: impl Iterator<Item = (&'a str, f64)>,
    rows: Rows,
    column: &LabelColumn,
    x: f64,
    anchor: Option<&str>,
    ink: &str,
  ) {
    let mut attributes = vec![("fill", ink.to_owned())];
    attributes.extend(anchor.map(|a| ("text-anchor", a.to_owned())));
    self.open("g", &attributes);
    for (name, row) in labels {
      let text = column.text(name);
      if !text.is_empty() {
        self.text("text", &[("x", num(x)), ("y", num(baseline(rows.y(row))))], &text);
      }
    }
    self.close("g");
  }

  /// A mark ring at `at`, filled with the ground color.
  pub(super) fn ring(&mut self, at: Point, stroke: &str, ground: &str) {
    self.empty(
      "circle",
      &[
        ("cx", num(at[0])),
        ("cy", num(at[1])),
        ("r", num(MARK_RADIUS)),
        ("fill", ground.to_owned()),
        ("stroke", stroke.to_owned()),
        ("stroke-width", num(MARK_WIDTH)),
      ],
    );
  }

  fn write(&mut self, event: Event<'_>) {
    #[expect(clippy::expect_used, reason = "writing into a Vec<u8> cannot fail")]
    self.writer.write_event(event).expect("writing into memory");
  }
}

/// The width between the margins of a figure `width` px wide.
pub(super) fn inner_width(width: f64) -> f64 {
  (width - 2.0 * MARGIN).max(0.0)
}

/// A figure document of `width` px on the color `ground`, high enough for `rows` rows of
/// `row_height` px and the legend `entries`, with `title` above the drawing in `ink`. The figure
/// ends with `Svg::finish_figure`.
pub(super) fn figure(
  title: &str,
  rows: usize,
  row_height: f64,
  width: f64,
  entries: &[LegendEntry],
  [ground, ink]: [&str; 2],
) -> Svg {
  let height = figure_height(rows, row_height, entries, width);
  let mut svg = Svg::new(width, height, title, ground);
  svg.title(title, ink);
  svg
}

/// The height of a figure whose drawing is `rows` rows of `row_height` px, with the legend
/// `entries` wrapped at `width` px.
fn figure_height(rows: usize, row_height: f64, entries: &[LegendEntry], width: f64) -> f64 {
  let legend = count(legend_lines(entries, width)) * LEGEND_LINE;
  drawing_top() + count(rows) * row_height + LEGEND_GAP + legend + MARGIN
}

/// The top of the drawing, below the title band.
pub(super) fn drawing_top() -> f64 {
  MARGIN + TITLE_BAND
}

/// The top of the legend under a drawing of `rows` rows of `row_height` px.
fn legend_top(rows: usize, row_height: f64) -> f64 {
  drawing_top() + count(rows) * row_height + LEGEND_GAP
}

/// The baseline of text centered on `y`.
pub(super) fn baseline(y: f64) -> f64 {
  y + BASELINE_SHIFT_EM * FONT_SIZE
}

/// The `stroke-dasharray` value of the dash and gap `pattern`.
pub(super) fn dash_array([dash, gap]: [f64; 2]) -> String {
  format!("{} {}", num(dash), num(gap))
}

/// The estimated width of `text` in px at the label font size.
fn text_width(text: &str) -> f64 {
  f64::from(advance(text)) * FONT_SIZE / 1000.0
}

/// The estimated advance of `text` in 1/1000 em.
fn advance(text: &str) -> u32 {
  text.chars().map(char_advance).fold(0, u32::saturating_add)
}

/// The estimated advance of `c` in 1/1000 em: from the Helvetica tables for ASCII and Latin
/// letters, 0 for a character without width, and `OTHER_ADVANCE` otherwise.
fn char_advance(c: char) -> u32 {
  let at = |table: &[u32], first: u32| {
    let i = usize::try_from(u32::from(c).checked_sub(first)?).ok()?;
    table.get(i).copied()
  };
  if ZERO_WIDTH.iter().any(|&(low, high)| (low..=high).contains(&c)) {
    return 0;
  }
  at(&ASCII_ADVANCE, 0x20)
    .or_else(|| at(&LATIN_ADVANCE, LATIN_FIRST))
    .unwrap_or(OTHER_ADVANCE)
}

/// The label column of a drawing in px, and the room its labels have.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct LabelColumn {
  /// Width in px; 0 without labels.
  pub(super) width: f64,
  /// Room for the text of a label, in 1/1000 em.
  room: u32,
}

impl LabelColumn {
  /// No label column.
  const NONE: LabelColumn = LabelColumn { width: 0.0, room: 0 };

  /// The column has labels.
  pub(super) fn shown(&self) -> bool {
    self.width > 0.0
  }

  /// The label of `name`: shortened to the length of the drawing rules, then further until its
  /// estimated width fits the column, keeping at least one character of the name besides the
  /// ellipsis; empty when that does not fit.
  pub(super) fn text(&self, name: &str) -> String {
    let max = label_max_chars().min(grapheme_count(name));
    (shortest(name)..=max)
      .rev()
      .map(|k| shorten(name, k))
      .find(|t| advance(t) <= self.room)
      .unwrap_or_else(String::new)
  }
}

/// The fewest characters a shortened label of `name` keeps: a character of the name and the
/// ellipsis, or the whole name when it has fewer than two characters. A bare ellipsis shows
/// nothing of the name.
fn shortest(name: &str) -> usize {
  grapheme_count(name).min(2)
}

/// The label column for leaf `names`: as wide as the longest label (shortened to the length of
/// the drawing rules) and a gap on each side, at most `max_width` px. Labels that do not fit are
/// shortened further. No column when `shown` is false, there are no names, or no label fits with
/// one character of its name.
pub(super) fn label_column<'a>(names: impl Iterator<Item = &'a str>, shown: bool, max_width: f64) -> LabelColumn {
  if !shown {
    return LabelColumn::NONE;
  }
  let names: Vec<&str> = names.filter(|n| !n.is_empty()).collect();
  let Some(longest) = names.iter().map(|n| advance(&shorten(n, label_max_chars()))).max() else {
    return LabelColumn::NONE;
  };
  let wanted = em_px(longest) + 2.0 * LABEL_GAP;
  if wanted <= max_width {
    return LabelColumn {
      width: wanted,
      room: longest,
    };
  }
  let room = px_em(max_width - 2.0 * LABEL_GAP);
  let narrowest = names
    .iter()
    .map(|n| advance(&shorten(n, shortest(n))))
    .min()
    .unwrap_or(u32::MAX);
  if narrowest <= room {
    LabelColumn { width: max_width, room }
  } else {
    LabelColumn::NONE
  }
}

/// `units` of 1/1000 em in px at the label font size.
fn em_px(units: u32) -> f64 {
  f64::from(units) * FONT_SIZE / 1000.0
}

/// The whole 1/1000 em in `px` at the label font size; 0 for no room.
fn px_em(px: f64) -> u32 {
  let units = (px * 1000.0 / FONT_SIZE).floor();
  if units > 0.0 {
    #[expect(
      clippy::as_conversions,
      clippy::cast_possible_truncation,
      clippy::cast_sign_loss,
      reason = "a positive whole number; `as` saturates at u32::MAX"
    )]
    let units = units as u32;
    units
  } else {
    0
  }
}

/// `v` with at most two decimals and without trailing zeros, so that equal figures have equal
/// text: `12`, `0.5`, `-3.25`.
pub(super) fn num(v: f64) -> String {
  let text = format!("{v:.2}");
  let text = text.trim_end_matches('0').trim_end_matches('.');
  if text == "-0" { "0".to_owned() } else { text.to_owned() }
}

/// A count of rows, lines, or characters as a coordinate.
#[expect(
  clippy::as_conversions,
  reason = "these counts are far below 2^53, so the conversion is exact"
)]
pub(super) fn count(n: usize) -> f64 {
  n as f64
}

/// SVG path data.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Path(String);

impl Path {
  pub(super) fn new() -> Path {
    Path::default()
  }

  pub(super) fn move_to(self, p: Point) -> Path {
    self.push(&format!("M{} {}", num(p[0]), num(p[1])))
  }

  /// A vertical line to `y`.
  pub(super) fn v(self, y: f64) -> Path {
    self.push(&format!("V{}", num(y)))
  }

  /// A horizontal line to `x`.
  pub(super) fn h(self, x: f64) -> Path {
    self.push(&format!("H{}", num(x)))
  }

  /// A move to the start of `c`, then `c`.
  pub(super) fn curve(self, c: &Bezier) -> Path {
    self.move_to(c.from).curve_to(c)
  }

  /// The cubic Bézier `c` from the current point, which is its start.
  pub(super) fn curve_to(self, c: &Bezier) -> Path {
    self.push(&format!(
      "C{} {} {} {} {} {}",
      num(c.c1[0]),
      num(c.c1[1]),
      num(c.c2[0]),
      num(c.c2[1]),
      num(c.to[0]),
      num(c.to[1])
    ))
  }

  pub(super) fn close(self) -> Path {
    self.push("Z")
  }

  fn push(mut self, command: &str) -> Path {
    if !self.0.is_empty() {
      self.0.push(' ');
    }
    self.0.push_str(command);
    self
  }
}

/// A column of the figure in px: normalized x 0 maps to `start` and 1 to `end`, so a column with
/// `end` left of `start` is mirrored.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Column {
  pub(super) start: f64,
  pub(super) end: f64,
}

/// The pixel frame of a drawing: leaf rows of `row_height` px from `top`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Rows {
  pub(super) top: f64,
  pub(super) row_height: f64,
}

impl Rows {
  /// The y in px of the middle of row `y`.
  pub(super) fn y(&self, y: f64) -> f64 {
    self.top + (y + 0.5) * self.row_height
  }

  /// The point `p` in normalized units of `column`, in px.
  pub(super) fn point(&self, column: Column, p: Point) -> Point {
    [column.start + p[0] * (column.end - column.start), self.y(p[1])]
  }

  /// The Bézier `c` in normalized units of `column`, in px.
  pub(super) fn bezier(&self, column: Column, c: &Bezier) -> Bezier {
    Bezier {
      from: self.point(column, c.from),
      c1: self.point(column, c.c1),
      c2: self.point(column, c.c2),
      to: self.point(column, c.to),
    }
  }

  /// The elbow `points` in normalized units of `column` as path data: down or up the parent's x,
  /// then across to the node.
  pub(super) fn elbow(&self, column: Column, points: &[Point; 3]) -> Path {
    let [from, corner, to] = points.map(|p| self.point(column, p));
    Path::new().move_to(from).v(corner[1]).h(to[0])
  }

  /// The part of the elbow `points` along the parent's x, as path data.
  pub(super) fn elbow_stem(&self, column: Column, points: &[Point; 3]) -> Path {
    let [from, corner, _] = points.map(|p| self.point(column, p));
    Path::new().move_to(from).v(corner[1])
  }

  /// The part of the elbow `points` across to the node, as path data.
  pub(super) fn elbow_across(&self, column: Column, points: &[Point; 3]) -> Path {
    let [_, corner, to] = points.map(|p| self.point(column, p));
    Path::new().move_to(corner).h(to[0])
  }
}

/// An entry of a legend: a symbol drawn from its parts, and a label.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct LegendEntry {
  pub(super) symbol: Vec<Symbol>,
  pub(super) label: String,
}

/// A part of a legend symbol.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Symbol {
  /// A horizontal line across the symbol.
  Line {
    color: String,
    width: f64,
    dash: Option<[f64; 2]>,
  },
  /// An S-curve across the symbol.
  Curve { color: String, dash: Option<[f64; 2]> },
  /// A ribbon across the symbol.
  Ribbon { color: String },
  /// A mark ring at the fraction `at` of the symbol width.
  Ring { stroke: String, ground: String, at: f64 },
}

/// The line and the x of each legend entry: entries flow left to right from the margin and wrap
/// to a new line when the next would end beyond `width - MARGIN`.
fn legend_places(entries: &[LegendEntry], width: f64) -> Vec<(usize, f64)> {
  let right = width - MARGIN;
  let mut places = Vec::with_capacity(entries.len());
  let (mut line, mut x) = (0, MARGIN);
  for entry in entries {
    let w = entry_width(entry);
    if x > MARGIN && x + w > right {
      line += 1;
      x = MARGIN;
    }
    places.push((line, x));
    x += w + ENTRY_GAP;
  }
  places
}

fn legend_lines(entries: &[LegendEntry], width: f64) -> usize {
  legend_places(entries, width).last().map_or(0, |(line, _)| line + 1)
}

fn entry_width(entry: &LegendEntry) -> f64 {
  SYMBOL_WIDTH + SYMBOL_GAP + text_width(&entry.label)
}

fn start<'a>(name: &'a str, attributes: &'a [(&'a str, String)]) -> BytesStart<'a> {
  let mut element = BytesStart::new(name);
  for (k, v) in attributes {
    element.push_attribute((*k, xml_chars(v).as_ref()));
  }
  element
}

/// `text` with every character that XML 1.0 does not allow (section 2.2: the C0 controls other
/// than tab, line feed, and carriage return, and U+FFFE and U+FFFF) replaced by U+FFFD, the
/// replacement character. Newick names can hold such characters, and a viewer refuses an SVG file
/// that contains them.
fn xml_chars(text: &str) -> Cow<'_, str> {
  let allowed =
    |c: char| !matches!(c, '\u{0}'..='\u{8}' | '\u{b}' | '\u{c}' | '\u{e}'..='\u{1f}' | '\u{fffe}' | '\u{ffff}');
  if text.chars().all(allowed) {
    Cow::Borrowed(text)
  } else {
    Cow::Owned(text.chars().map(|c| if allowed(c) { c } else { '\u{fffd}' }).collect())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  #[rustfmt::skip]
  #[rstest]
  #[case::whole(        12.0,      "12")]
  #[case::half(         0.5,       "0.5")]
  #[case::rounded(      1.0 / 3.0, "0.33")]
  #[case::round_up(     2.005_1,   "2.01")]
  #[case::negative(     -3.25,     "-3.25")]
  #[case::negative_zero(-0.001,    "0")]
  #[case::zero(         0.0,       "0")]
  #[trace]
  fn num_writes_at_most_two_decimals_without_trailing_zeros(#[case] v: f64, #[case] expected: &str) {
    assert_eq!(expected, num(v));
  }

  #[rustfmt::skip]
  #[rstest]
  // Oracle: the Helvetica widths of the Adobe font metrics (Helvetica.afm), in 1/1000 em: Z 611,
  // udieresis 556, r 333, i 222, c 500, h 556, and Ccircumflex, which Helvetica lacks, as C 722;
  // 0 for a combining mark and a zero-width joiner, and 1000 for other characters.
  #[case::capitals( "AW",          667 + 944)]
  #[case::narrow(   "il",          222 + 222)]
  #[case::latin_1(  "Zürich",      611 + 556 + 333 + 222 + 500 + 556)]
  #[case::extended( "Ĉ",           722)]
  #[case::greek(    "αβ",          1000 + 1000)]
  #[case::combining("a\u{301}",    556)]
  #[case::joiner(   "a\u{200d}b",  556 + 556)]
  #[case::cjk(      "流感",        1000 + 1000)]
  #[case::empty(    "",            0)]
  #[trace]
  fn advance_adds_the_widths_of_the_characters(#[case] text: &str, #[case] expected: u32) {
    assert_eq!(expected, advance(text));
  }

  #[rustfmt::skip]
  #[rstest]
  // Oracle: a width of u units is u * 12 / 1000 px, and the column adds a 6 px gap on each side.
  // AW is 1611 units, 19.332 px, so its column is 31.332 px.
  #[case::fits(     &["AW"],         100.0, ("31.33", vec!["AW"]))]
  #[case::narrower( &["il", "AW"],   100.0, ("31.33", vec!["il", "AW"]))]
  // 40 px leave 28 px, 2333 units: "M…M" needs 833 + 1000 + 833 = 2666 units, "M…" 1833.
  #[case::shortened(&["MMMMMM"],     40.0,  ("40",    vec!["M\u{2026}"]))]
  // "a" alone needs 6.672 + 12 = 18.672 px.
  #[case::one_char( &["a"],          19.0,  ("18.67", vec!["a"]))]
  // 18 px leave 6 px, 500 units, less than the 556 of "a" and the 1000 of "…".
  #[case::no_room(  &["a", "abc"],   18.0,  ("0",     vec!["", ""]))]
  // 25 px leave 13 px, 1083 units: "…" (1000) fits, "a…" (1556) does not, and a bare ellipsis
  // shows nothing of a name.
  #[case::ellipsis( &["abc"],        25.0,  ("0",     vec![""]))]
  #[case::one_name( &["a", "abc"],   25.0,  ("25",    vec!["a", ""]))]
  #[case::no_names( &[],             100.0, ("0",     vec![]))]
  #[trace]
  fn label_column_fits_the_longest_label(
    #[case] names: &[&str],
    #[case] max_width: f64,
    #[case] (width, texts): (&str, Vec<&str>),
  ) {
    let column = label_column(names.iter().copied(), true, max_width);
    let actual: Vec<String> = names.iter().map(|n| column.text(n)).collect();
    assert_eq!((width.to_owned(), texts), (num(column.width), actual.iter().map(String::as_str).collect()));
  }

  #[test]
  fn label_column_without_labels_is_empty() {
    let column = label_column(["abc"].into_iter(), false, 100.0);
    assert_eq!(("0".to_owned(), false), (num(column.width), column.shown()));
  }

  #[test]
  fn path_writes_commands_with_spaces() {
    let c = Bezier {
      from: [0.0, 1.0],
      c1: [0.5, 1.0],
      c2: [0.5, 2.0],
      to: [1.0, 2.0],
    };
    let d = Path::new().move_to([1.0, 2.5]).v(3.0).h(4.25).curve(&c).close();
    assert_eq!("M1 2.5 V3 H4.25 M0 1 C0.5 1 0.5 2 1 2 Z", d.0);
  }

  #[test]
  fn rows_map_a_mirrored_column() {
    let rows = Rows {
      top: 10.0,
      row_height: 4.0,
    };
    let mirrored = Column {
      start: 100.0,
      end: 60.0,
    };
    // Oracle: x 0.25 of a 40 px column from 100 leftwards is 90; row 1 is centered at
    // 10 + 1.5 * 4.
    assert_eq!(
      [90.0, 16.0].map(f64::to_bits),
      rows.point(mirrored, [0.25, 1.0]).map(f64::to_bits)
    );
  }

  #[test]
  fn legend_wraps_entries_that_do_not_fit() {
    let entry = |label: &str| LegendEntry {
      symbol: vec![],
      label: label.to_owned(),
    };
    // Each entry is 24 + 6 + 10 * 556 * 12 / 1000 = 96.72 px wide, 116.72 px with its gap; a
    // 300 px figure has 268 px between its margins, room for two entries per line.
    let entries = vec![entry("0123456789"), entry("0123456789"), entry("0123456789")];
    let expected = vec![(0, "16".to_owned()), (0, "132.72".to_owned()), (1, "16".to_owned())];
    let places: Vec<(usize, String)> = legend_places(&entries, 300.0)
      .into_iter()
      .map(|(l, x)| (l, num(x)))
      .collect();
    assert_eq!(expected, places);
    assert_eq!(2, legend_lines(&entries, 300.0));
    assert_eq!(0, legend_lines(&[], 300.0));
  }
}
