//! An SVG document writer over `quick-xml`, which escapes attribute values and text, with the
//! pixel frame, path data, text width estimate, and legend shared by the figures.

use crate::display::{Bezier, DRAWING_RULES, Point};
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

/// Space around the drawing, in px; the margin of the interactive views.
pub(super) const MARGIN: f64 = 16.0;
/// Space between a tree column and its labels, in px.
pub(super) const LABEL_GAP: f64 = 6.0;
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
/// Space between a legend symbol and its text, and between legend entries, in px.
const SYMBOL_GAP: f64 = 6.0;
const ENTRY_GAP: f64 = 20.0;
/// Average advance of a character in em. SVG text has no measured width before a viewer lays it
/// out, so widths are estimated: 0.55 em covers IBM Plex Sans Condensed and the wider Helvetica
/// and Arial fallbacks, so labels do not overrun their column in either.
const CHAR_EM: f64 = 0.55;
/// Replaces the middle of a shortened label (U+2026, the character of the interactive views).
const ELLIPSIS: char = '\u{2026}';

/// Fonts of the figure text.
const FONT_FAMILY: &str = "IBM Plex Sans, Helvetica, Arial, sans-serif";
/// Fonts of the leaf labels: the condensed cut first, as in the interactive views.
pub(super) const LABEL_FONT_FAMILY: &str = "IBM Plex Sans Condensed, IBM Plex Sans, Helvetica, Arial, sans-serif";

/// Stroke widths and mark sizes in px, those of the interactive views.
pub(super) const BRANCH_WIDTH: f64 = 1.5;
pub(super) const REASSORTMENT_WIDTH: f64 = 2.0;
pub(super) const LINK_WIDTH: f64 = 1.0;
pub(super) const LEADER_WIDTH: f64 = 1.0;
pub(super) const LEADER_OPACITY: f64 = 0.5;
pub(super) const MARK_RADIUS: f64 = 3.5;
pub(super) const MARK_WIDTH: f64 = 1.5;
pub(super) const RIBBON_OPACITY: f64 = 0.55;
/// Dash patterns: dashed branches and reticulations, dotted leaders.
pub(super) const DASH: &str = "4 3";
pub(super) const DOT: &str = "1 3";

/// An SVG document under construction, written with indentation.
pub(super) struct Svg {
  writer: Writer<Vec<u8>>,
}

impl Svg {
  /// A document of `width` by `height` px on the color `ground`, with `title` as its accessible
  /// name and as the heading above the drawing.
  pub(super) fn new(width: f64, height: f64, title: &str, ground: &str) -> Svg {
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
  pub(super) fn finish(mut self) -> String {
    self.close("svg");
    #[expect(
      clippy::expect_used,
      reason = "the writer copies UTF-8 text and ASCII markup, so its output is UTF-8"
    )]
    let text = String::from_utf8(self.writer.into_inner()).expect("the SVG text is UTF-8");
    format!("{text}\n")
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
    self.write(Event::Text(BytesText::new(text)));
    self.close(name);
  }

  /// A path with `d` and `attributes`.
  pub(super) fn path(&mut self, d: &Path, attributes: &[(&str, String)]) {
    let mut all = vec![("d", d.0.clone())];
    all.extend_from_slice(attributes);
    self.empty("path", &all);
  }

  /// The title heading, left-aligned at the margin.
  pub(super) fn title(&mut self, title: &str, ink: &str) {
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
  pub(super) fn legend(&mut self, entries: &[LegendEntry], top: f64, width: f64, ink: &str) {
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
    let line = |dash: Option<&str>, color: &str, width: f64| {
      let mut attributes = vec![
        ("fill", "none".to_owned()),
        ("stroke", color.to_owned()),
        ("stroke-width", num(width)),
      ];
      attributes.extend(dash.map(|d| ("stroke-dasharray", d.to_owned())));
      attributes
    };
    match symbol {
      Symbol::Line { color, width, dash } => {
        let d = Path::new().move_to([x, mid]).h(x + SYMBOL_WIDTH);
        self.path(&d, &line(*dash, color, *width));
      },
      Symbol::Curve { color, dash } => {
        let curve = s_curve([x, mid - 4.0], [x + SYMBOL_WIDTH, mid + 4.0]);
        self.path(&Path::new().curve(&curve), &line(*dash, color, BRANCH_WIDTH));
      },
      Symbol::Ribbon { color } => {
        let top = s_curve([x, mid - 5.0], [x + SYMBOL_WIDTH, mid - 1.0]);
        let bottom = s_curve([x + SYMBOL_WIDTH, mid + 5.0], [x, mid + 1.0]);
        let d = Path::new().curve(&top).v(mid + 5.0).curve_to(&bottom).close();
        self.path(&d, &[("fill", color.clone()), ("fill-opacity", num(RIBBON_OPACITY))]);
      },
      Symbol::Ring { stroke, ground, at } => {
        self.ring([x + at * SYMBOL_WIDTH, mid], stroke, ground);
      },
    }
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

/// The height of a figure whose drawing is `rows` rows of `row_height` px, with the legend
/// `entries` wrapped at `width` px.
pub(super) fn figure_height(rows: usize, row_height: f64, entries: &[LegendEntry], width: f64) -> f64 {
  let legend = count(legend_lines(entries, width)) * LEGEND_LINE;
  drawing_top() + count(rows) * row_height + LEGEND_GAP + legend + MARGIN
}

/// The top of the drawing, below the title band.
pub(super) fn drawing_top() -> f64 {
  MARGIN + TITLE_BAND
}

/// The top of the legend under a drawing of `rows` rows of `row_height` px.
pub(super) fn legend_top(rows: usize, row_height: f64) -> f64 {
  drawing_top() + count(rows) * row_height + LEGEND_GAP
}

/// The baseline of text centered on `y`: about a third of the font size below it.
pub(super) fn baseline(y: f64) -> f64 {
  y + 0.35 * FONT_SIZE
}

/// The estimated width of `text` in px at the label font size.
pub(super) fn text_width(text: &str) -> f64 {
  count(text.chars().count()) * CHAR_EM * FONT_SIZE
}

/// The number of characters that fit into `width` px at the label font size.
pub(super) fn chars_fitting(width: f64) -> usize {
  let chars = (width / (CHAR_EM * FONT_SIZE)).floor();
  if chars > 0.0 {
    #[expect(
      clippy::as_conversions,
      clippy::cast_possible_truncation,
      clippy::cast_sign_loss,
      reason = "a positive whole number of characters; `as` saturates at usize::MAX"
    )]
    let chars = chars as usize;
    chars
  } else {
    0
  }
}

/// `name` shortened in the middle to at most `max` characters, with an ellipsis in place of the
/// removed characters: the first half of the kept characters (rounded up), the ellipsis, then the
/// rest from the end. The interactive views shorten labels the same way.
pub(super) fn shorten(name: &str, max: usize) -> String {
  let chars: Vec<char> = name.chars().collect();
  if chars.len() <= max {
    return name.to_owned();
  }
  let kept = max.saturating_sub(1);
  let head = kept.div_ceil(2);
  let tail = kept - head;
  let mut text: String = chars[..head].iter().collect();
  text.push(ELLIPSIS);
  text.extend(&chars[chars.len() - tail..]);
  text
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

/// The longest label of the drawing rules, in characters.
pub(super) fn label_max_chars() -> usize {
  usize::try_from(DRAWING_RULES.label_max_chars).unwrap_or(usize::MAX)
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
    dash: Option<&'static str>,
  },
  /// An S-curve across the symbol.
  Curve { color: String, dash: Option<&'static str> },
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

/// The S-curve from `from` to `to` with both control points at the middle x, the link form of
/// the drawing rules.
fn s_curve(from: Point, to: Point) -> Bezier {
  let mid = f64::midpoint(from[0], to[0]);
  Bezier {
    from,
    c1: [mid, from[1]],
    c2: [mid, to[1]],
    to,
  }
}

fn start<'a>(name: &'a str, attributes: &'a [(&'a str, String)]) -> BytesStart<'a> {
  BytesStart::new(name).with_attributes(attributes.iter().map(|(k, v)| (*k, v.as_str())))
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
  #[case::short(     "A/New York/392/2004", 40, "A/New York/392/2004")]
  #[case::at_max(    "abcdef",              6,  "abcdef")]
  // Oracle: 5 kept characters, 3 from the start and 2 from the end, as `shortenLabel` of the web
  // app.
  #[case::odd_kept(  "abcdefgh",            6,  "abc\u{2026}gh")]
  #[case::even_kept( "abcdefgh",            5,  "ab\u{2026}gh")]
  #[case::multibyte( "αβγδεζηθ",            4,  "αβ\u{2026}θ")]
  #[case::one(       "abc",                 1,  "\u{2026}")]
  #[trace]
  fn shorten_keeps_the_start_and_the_end(#[case] name: &str, #[case] max: usize, #[case] expected: &str) {
    assert_eq!(expected, shorten(name, max));
    assert!(shorten(name, max).chars().count() <= max);
  }

  #[test]
  fn shorten_of_a_long_label_has_the_rule_length() {
    let name = "x".repeat(50);
    assert_eq!(40, shorten(&name, 40).chars().count());
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::none(    0.0,  0)]
  #[case::partial( 13.0, 1)]
  #[case::exact(   13.2, 2)]
  #[case::negative(-5.0, 0)]
  #[trace]
  fn chars_fitting_counts_whole_characters(#[case] width: f64, #[case] expected: usize) {
    // Oracle: a character is 0.55 em of 12 px, 6.6 px.
    assert_eq!(expected, chars_fitting(width));
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
    // Each entry is 24 + 6 + 10 * 6.6 = 96 px wide, 116 px with its gap; a 300 px figure has
    // 268 px between its margins, room for two entries per line.
    let entries = vec![entry("0123456789"), entry("0123456789"), entry("0123456789")];
    let expected = vec![(0, 16.0), (0, 132.0), (1, 16.0)];
    let places = legend_places(&entries, 300.0);
    let rounded: Vec<(usize, f64)> = places.iter().map(|&(l, x)| (l, x.round())).collect();
    assert_eq!(expected, rounded);
    assert_eq!(2, legend_lines(&entries, 300.0));
    assert_eq!(0, legend_lines(&[], 300.0));
  }
}
