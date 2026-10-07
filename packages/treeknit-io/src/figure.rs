//! SVG figures of a tanglegram and of the ARG, drawn from the shapes of `display` with the
//! light colors of `palette` and the columns of `DrawingRules`, as the interactive views draw
//! them. The width of a label is estimated, because SVG text has no width before a viewer lays it
//! out, so the label columns can differ from those of the interactive views.

mod arg;
mod svg;
mod tanglegram;

use crate::analysis::{Field, ValidationError};
use crate::display::{ArgView, DRAWING_RULES, PairView, Scale};
use serde::{Deserialize, Serialize};
use strum::VariantArray;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// The SVG tanglegram of `view`, titled with the labels of its two trees, with a legend under
/// the drawing; the errors of `check_figure_options` when `options` are invalid. The view must be
/// laid out with `options.scale` by `display::pair_view`; the shapes have the scale `view.scale`.
pub fn tanglegram_svg(view: &PairView, options: &FigureOptions) -> Result<String, Vec<ValidationError>> {
  checked(options)?;
  Ok(tanglegram::draw(view, options))
}

/// The SVG figure of the ARG `view` of the trees labeled `segments` (segment A, then B), titled
/// with them, with a legend under the drawing; the errors of `check_figure_options` when
/// `options` are invalid. The view is laid out with `options.scale` by `display::arg_view`; the
/// shapes have the scale `view.scale`.
pub fn arg_svg(view: &ArgView, segments: [&str; 2], options: &FigureOptions) -> Result<String, Vec<ValidationError>> {
  checked(options)?;
  Ok(arg::draw(view, segments, options))
}

/// Size and content of a figure; a missing field takes its default.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct FigureOptions {
  /// Width in px.
  pub width: f64,
  /// Height of a leaf row in px.
  pub row_height: f64,
  /// Branch scale.
  pub scale: Scale,
  /// When leaf labels are drawn.
  pub labels: LabelMode,
}

impl Default for FigureOptions {
  fn default() -> Self {
    FigureOptions {
      width: 1200.0,
      row_height: 12.0,
      scale: Scale::default(),
      labels: LabelMode::default(),
    }
  }
}

/// An option of [`FigureOptions`] that [`check_figure_options`] reports, named as its field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, VariantArray)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub enum FigureOptionKey {
  Width,
  RowHeight,
}

/// Smallest figure width, in px: the two margins and one px between them, so that the drawing
/// has a width.
pub const MIN_FIGURE_WIDTH: f64 = 2.0 * DRAWING_RULES.margin_px + 1.0;
/// Largest figure width, in px: a bound against mistyped values, a hundred times the default.
pub const MAX_FIGURE_WIDTH: f64 = 100_000.0;
/// Smallest row height, in px: the figures write coordinates with two decimals, so rows closer
/// than 0.01 px would fall on one y; one px keeps every row apart.
pub const MIN_ROW_HEIGHT: f64 = 1.0;
/// Largest row height, in px: a bound against mistyped values, far above the 12 px of a label.
pub const MAX_ROW_HEIGHT: f64 = 1_000.0;

/// Check `options`: the width and the row height must be finite numbers from `MIN_FIGURE_WIDTH`
/// to `MAX_FIGURE_WIDTH` and from `MIN_ROW_HEIGHT` to `MAX_ROW_HEIGHT` px. Each error names its
/// option as a [`FigureOptionKey`].
pub fn check_figure_options(options: &FigureOptions) -> Vec<ValidationError> {
  let bounded = |key: FigureOptionKey, name: &str, value: f64, [min, max]: [f64; 2]| {
    let message = if !(value.is_finite() && value > 0.0) {
      format!("{name} must be a positive number, got {value}")
    } else if value < min {
      format!("{name} must be at least {min} px, got {value}")
    } else if value > max {
      format!("{name} must be at most {max} px, got {value}")
    } else {
      return None;
    };
    Some(ValidationError {
      field: Some(Field::FigureOption { key }),
      message,
      line: None,
      column: None,
    })
  };
  [
    bounded(
      FigureOptionKey::Width,
      "figure width",
      options.width,
      [MIN_FIGURE_WIDTH, MAX_FIGURE_WIDTH],
    ),
    bounded(
      FigureOptionKey::RowHeight,
      "row height",
      options.row_height,
      [MIN_ROW_HEIGHT, MAX_ROW_HEIGHT],
    ),
  ]
  .into_iter()
  .flatten()
  .collect()
}

pub(crate) fn checked(options: &FigureOptions) -> Result<(), Vec<ValidationError>> {
  let errors = check_figure_options(options);
  if errors.is_empty() { Ok(()) } else { Err(errors) }
}

/// Leaf labels are drawn: always with `on`, never with `off`, and with `auto` from the row height
/// of the drawing rules.
fn labels_shown(options: &FigureOptions) -> bool {
  match options.labels {
    LabelMode::On => true,
    LabelMode::Off => false,
    LabelMode::Auto => options.row_height >= f64::from(DRAWING_RULES.label_auto_min_row_px),
  }
}

/// When leaf labels are drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize, VariantArray)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "lowercase")]
pub enum LabelMode {
  /// From `DrawingRules.label_auto_min_row_px` px per row.
  #[default]
  Auto,
  On,
  Off,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::display::{self, TreeVersion};
  use crate::palette;
  use crate::test_support::run_trees;
  use crate::wire::wire_name;
  use pretty_assertions::assert_eq;
  use quick_xml::events::BytesStart;
  use quick_xml::events::Event;
  use quick_xml::{Reader, XmlVersion};
  use rstest::rstest;
  use serde_json::json;

  /// The two-tree example: X moved between the trees. Its MCCs are `[X]` and `[A,B,C,D]`
  /// (TreeKnit.jl fixture `fixtures/doc_mccs_1.json`).
  const HA: &str = "((A,B),(C,(D,X)));";
  const NA: &str = "((A,(B,X)),(C,D));";

  /// The resolved pair view of the two-tree example, laid out with `scale`.
  fn example_view(scale: Scale) -> PairView {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    display::pair_view(&r, 0, TreeVersion::Resolved, scale).unwrap()
  }

  /// The elements of `svg` in document order, with their attributes and the text of `text` and
  /// `title` elements, unescaped; panics when `svg` is not well-formed XML.
  fn elements(svg: &str) -> Vec<Element> {
    let mut reader = Reader::from_str(svg);
    let mut elements: Vec<Element> = Vec::new();
    let mut open: Vec<usize> = Vec::new();
    loop {
      match reader.read_event().unwrap() {
        Event::Start(e) => {
          elements.push(element(&e));
          open.push(elements.len() - 1);
        },
        Event::Empty(e) => elements.push(element(&e)),
        Event::End(_) => {
          open.pop().unwrap();
        },
        Event::Text(t) => {
          if let Some(&i) = open.last() {
            elements[i].text.push_str(&t.xml10_content());
          }
        },
        Event::GeneralRef(r) => {
          // Oracle: the predefined entities of XML 1.0, section 4.6.
          let text = match r.xml10_content().as_ref() {
            "lt" => "<",
            "gt" => ">",
            "amp" => "&",
            "quot" => "\"",
            "apos" => "'",
            other => panic!("unexpected entity {other}"),
          };
          elements[*open.last().unwrap()].text.push_str(text);
        },
        Event::Eof => break,
        _ => {},
      }
    }
    assert!(open.is_empty());
    for e in &mut elements {
      if !matches!(e.name.as_str(), "text" | "title") {
        e.text.clear();
      }
    }
    elements
  }

  fn element(e: &BytesStart<'_>) -> Element {
    Element {
      name: e.name().as_ref().to_owned(),
      attributes: e
        .attributes()
        .map(|a| {
          let a = a.unwrap();
          (
            a.key.as_ref().to_owned(),
            a.normalized_value(XmlVersion::Implicit1_0).unwrap().into_owned(),
          )
        })
        .collect(),
      text: String::new(),
    }
  }

  /// An element of an SVG text.
  #[derive(Clone, Debug, PartialEq, Eq)]
  struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    text: String,
  }

  impl Element {
    fn attribute(&self, key: &str) -> Option<&str> {
      self.attributes.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }
  }

  /// The texts of the `text` elements of `svg`.
  fn texts(svg: &str) -> Vec<String> {
    elements(svg)
      .into_iter()
      .filter(|e| e.name == "text")
      .map(|e| e.text)
      .collect()
  }

  /// The paths of `svg` stroked with `color`.
  fn stroked(svg: &str, color: &str) -> usize {
    elements(svg)
      .iter()
      .filter(|e| e.name == "path" && e.attribute("stroke") == Some(color))
      .count()
  }

  fn options(width: f64, row_height: f64, scale: Scale, labels: LabelMode) -> FigureOptions {
    FigureOptions {
      width,
      row_height,
      scale,
      labels,
    }
  }

  #[test]
  fn tanglegram_svg_of_the_two_tree_example_with_labels_on() {
    let svg = tanglegram_svg(
      &example_view(Scale::Depth),
      &options(600.0, 12.0, Scale::Depth, LabelMode::On),
    )
    .unwrap();
    // Oracle, by hand from the drawing rules: inner width 600 - 2 * 16 = 568; label columns
    // 8.664 + 2 * 6 = 20.664 px, from the widest label, C or D, 722/1000 em of 12 px in the
    // Helvetica metrics; link zone 0.2 * 568 - 2 * 20.664 = 72.272 px, less than
    // 0.15 * 568 = 85.2 px, so 85.2 px; tree columns (568 - 85.2 - 2 * 20.664) / 2 = 220.736 px,
    // so the left tree spans 16 to 236.736, the link zone 257.4 to 342.6, and the mirrored right
    // tree 584 down to 363.264. Rows are centered at 52 + 12 * (row + 0.5). Cladogram depths: ha
    // has its leaves at 3, (A,B) and (D,X) at 2, (C,(D,X)) at 1, so x = 16 + 220.736 * depth / 3.
    // X is its own MCC (slot 1), and the branch above X in each tree is a reassortment branch
    // with a ring at its midpoint. The second legend entry starts 24 + 6 + 20 px after the
    // 114.708 px of "Reassortment branch" (9559/1000 em).
    let expected = r##"<svg xmlns="http://www.w3.org/2000/svg" width="600" height="164" viewBox="0 0 600 164" font-family="Lato, Noto Sans, Helvetica, Arial, sans-serif" font-size="12">
  <title>ha and na</title>
  <rect width="600" height="164" fill="#f3f5f4"/>
  <text x="16" y="32" font-size="16" font-weight="600" fill="#1f2b30">ha and na</text>
  <g fill="none" stroke-width="1">
    <path d="M257.4 58 C300 58 300 58 342.6 58" stroke="#2f4b9a"/>
    <path d="M257.4 70 C300 70 300 70 342.6 70" stroke="#2f4b9a"/>
    <path d="M257.4 82 C300 82 300 94 342.6 94" stroke="#2f4b9a"/>
    <path d="M257.4 94 C300 94 300 106 342.6 106" stroke="#2f4b9a"/>
    <path d="M257.4 106 C300 106 300 82 342.6 82" stroke="#93771c"/>
  </g>
  <g fill="none" stroke-width="1.5">
    <path d="M16 77.5 V64 H163.16" stroke="#2f4b9a"/>
    <path d="M163.16 64 V58 H236.74" stroke="#2f4b9a"/>
    <path d="M163.16 64 V70 H236.74" stroke="#2f4b9a"/>
    <path d="M16 77.5 V91 H89.58" stroke="#2f4b9a"/>
    <path d="M89.58 91 V82 H236.74" stroke="#2f4b9a"/>
    <path d="M89.58 91 V100 H163.16" stroke="#2f4b9a"/>
    <path d="M163.16 100 V94 H236.74" stroke="#2f4b9a"/>
    <path d="M163.16 100 V106 H236.74" stroke="#b0265e" stroke-width="2"/>
  </g>
  <circle cx="199.95" cy="106" r="3.5" fill="#f3f5f4" stroke="#b0265e" stroke-width="1.5"/>
  <g fill="#1f2b30" text-anchor="start">
    <text x="242.74" y="62.2">A</text>
    <text x="242.74" y="74.2">B</text>
    <text x="242.74" y="86.2">C</text>
    <text x="242.74" y="98.2">D</text>
    <text x="242.74" y="110.2">X</text>
  </g>
  <g fill="none" stroke-width="1.5">
    <path d="M584 83.5 V67 H510.42" stroke="#2f4b9a"/>
    <path d="M510.42 67 V58 H363.26" stroke="#2f4b9a"/>
    <path d="M510.42 67 V76 H436.84" stroke="#2f4b9a"/>
    <path d="M436.84 76 V70 H363.26" stroke="#2f4b9a"/>
    <path d="M584 83.5 V100 H436.84" stroke="#2f4b9a"/>
    <path d="M436.84 100 V94 H363.26" stroke="#2f4b9a"/>
    <path d="M436.84 100 V106 H363.26" stroke="#2f4b9a"/>
    <path d="M436.84 76 V82 H363.26" stroke="#b0265e" stroke-width="2"/>
  </g>
  <circle cx="400.05" cy="82" r="3.5" fill="#f3f5f4" stroke="#b0265e" stroke-width="1.5"/>
  <g fill="#1f2b30" text-anchor="end">
    <text x="357.26" y="62.2">A</text>
    <text x="357.26" y="74.2">B</text>
    <text x="357.26" y="86.2">X</text>
    <text x="357.26" y="98.2">C</text>
    <text x="357.26" y="110.2">D</text>
  </g>
  <g fill="#1f2b30">
    <path d="M16 138 H40" fill="none" stroke="#b0265e" stroke-width="2"/>
    <circle cx="28" cy="138" r="3.5" fill="#f3f5f4" stroke="#b0265e" stroke-width="1.5"/>
    <text x="46" y="142.2">Reassortment branch</text>
    <path d="M180.71 134 C192.71 134 192.71 142 204.71 142" fill="none" stroke="#2f4b9a" stroke-width="1.5"/>
    <text x="210.71" y="142.2">Leaves of one MCC</text>
  </g>
</svg>
"##;
    assert_eq!(expected, svg);
  }

  #[test]
  fn tanglegram_svg_escapes_labels() {
    let mut view = example_view(Scale::Depth);
    let label = "A <b> & \"c\" 'd'";
    let leaf = view.left.nodes.iter().position(|n| n.name == "A").unwrap();
    view.left.nodes[leaf].name = label.to_owned();
    view.left.label = "h&a <1>".to_owned();
    let svg = tanglegram_svg(&view, &options(1200.0, 12.0, Scale::Depth, LabelMode::On)).unwrap();
    // Oracle: XML 1.0 escapes `<` and `&` in text, and quick-xml also escapes `>` and both
    // quotes.
    assert!(
      svg.contains(">A &lt;b&gt; &amp; &quot;c&quot; &apos;d&apos;</text>"),
      "{svg}"
    );
    let parsed = elements(&svg);
    let title = parsed.iter().find(|e| e.name == "title").unwrap();
    assert_eq!("h&a <1> and na", title.text);
    assert!(texts(&svg).contains(&label.to_owned()));
  }

  #[test]
  fn tanglegram_svg_replaces_characters_that_xml_does_not_allow() {
    let mut view = example_view(Scale::Depth);
    let leaf = view.left.nodes.iter().position(|n| n.name == "A").unwrap();
    view.left.nodes[leaf].name = "a\u{1}b\tc\u{ffff}".to_owned();
    view.left.label = "h\u{8}a".to_owned();
    let svg = tanglegram_svg(&view, &options(1200.0, 12.0, Scale::Depth, LabelMode::On)).unwrap();
    // Oracle: the production `Char` of XML 1.0, section 2.2, allows tab but not U+0001, U+0008,
    // or U+FFFF.
    let xml_char =
      |c: char| matches!(c, '\t' | '\n' | '\r' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..);
    assert!(svg.chars().all(xml_char), "{svg:?}");
    assert!(texts(&svg).contains(&"a\u{fffd}b\tc\u{fffd}".to_owned()), "{svg}");
    let title = elements(&svg).into_iter().find(|e| e.name == "title").unwrap();
    assert_eq!("h\u{fffd}a and na", title.text);
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::auto_below(LabelMode::Auto, 9.99, false)]
  #[case::auto_at(   LabelMode::Auto, 10.0, true)]
  #[case::on_small(  LabelMode::On,   2.0,  true)]
  #[case::off_large( LabelMode::Off,  40.0, false)]
  #[trace]
  fn tanglegram_svg_draws_labels_by_mode_and_row_height(
    #[case] labels: LabelMode,
    #[case] row_height: f64,
    #[case] shown: bool,
  ) {
    let svg = tanglegram_svg(&example_view(Scale::Depth), &options(1200.0, row_height, Scale::Depth, labels)).unwrap();
    // Oracle: the 10 px rule of the drawing rules; the title and the legend are text too.
    let leaves: Vec<String> = texts(&svg).into_iter().filter(|t| t.len() == 1).collect();
    let expected: Vec<String> = if shown {
      ["A", "B", "C", "D", "X", "A", "B", "X", "C", "D"].map(str::to_owned).to_vec()
    } else {
      vec![]
    };
    assert_eq!(expected, leaves);
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::ribbons_below(5.99, (4, 0))]
  #[case::links_at(     6.0,  (0, 6))]
  #[trace]
  fn tanglegram_svg_draws_ribbons_below_six_px_per_row(#[case] row_height: f64, #[case] expected: (usize, usize)) {
    let svg = tanglegram_svg(&example_view(Scale::Depth), &options(1200.0, row_height, Scale::Depth, LabelMode::Off)).unwrap();
    let curves: Vec<String> = elements(&svg)
      .iter()
      .filter(|e| e.name == "path")
      .filter_map(|e| e.attribute("d").filter(|d| d.contains('C')).map(str::to_owned))
      .collect();
    let closed = curves.iter().filter(|d| d.ends_with('Z')).count();
    // Oracle: by the block rule, left order A B C D X and right order A B X C D give the blocks
    // [A,B], [C,D], and [X]; there are five links, one per leaf; the legend adds one ribbon or
    // one link symbol.
    assert_eq!(expected, (closed, curves.len() - closed));
  }

  #[test]
  fn tanglegram_svg_marks_reassortment_branches_in_signal() {
    let view = example_view(Scale::Depth);
    let svg = tanglegram_svg(&view, &options(1200.0, 12.0, Scale::Depth, LabelMode::On)).unwrap();
    let signal = palette::palette().light.signal;
    let breaks = view
      .shapes
      .left
      .elbows
      .iter()
      .chain(&view.shapes.right.elbows)
      .filter(|e| e.mcc_break)
      .count();
    // Oracle: X is its own MCC in both trees, so the branch above X is the one reassortment
    // branch of each tree; the legend adds one signal line.
    assert_eq!(2, breaks);
    assert_eq!(breaks + 1, stroked(&svg, &signal));
    let rings = elements(&svg)
      .iter()
      .filter(|e| e.name == "circle" && e.attribute("stroke") == Some(signal.as_str()))
      .count();
    assert_eq!(breaks + 1, rings);
  }

  #[test]
  fn tanglegram_svg_shortens_long_labels_in_the_middle() {
    let mut view = example_view(Scale::Depth);
    let leaf = view.left.nodes.iter().position(|n| n.name == "A").unwrap();
    view.left.nodes[leaf].name = format!("A/{}/2004", "x".repeat(60));
    let svg = tanglegram_svg(&view, &options(4000.0, 12.0, Scale::Depth, LabelMode::On)).unwrap();
    // Oracle: a 4000 px figure has room for 40 characters, the limit of the drawing rules: 20
    // from the start, the ellipsis, and 19 from the end.
    let expected = format!("A/{}\u{2026}{}/2004", "x".repeat(18), "x".repeat(14));
    assert!(texts(&svg).contains(&expected), "{:?}", texts(&svg));
  }

  #[test]
  fn tanglegram_svg_draws_imputed_and_added_marks() {
    // P is only in ha, so imputation places it into na; the polytomy of na is resolved.
    let r = run_trees(&[("ha", "((A,B),(C,(D,P)));"), ("na", "((A,B,C),D);")]);
    let view = display::pair_view(&r, 0, TreeVersion::Imputed, Scale::Depth).unwrap();
    let svg = tanglegram_svg(&view, &options(1200.0, 12.0, Scale::Depth, LabelMode::On)).unwrap();
    let colors = palette::palette().light;
    let imputed = view
      .shapes
      .right
      .marks
      .iter()
      .filter(|m| m.kind == display::MarkKind::Imputed)
      .count();
    let added = view
      .shapes
      .left
      .elbows
      .iter()
      .chain(&view.shapes.right.elbows)
      .filter(|e| e.added && !e.mcc_break)
      .count();
    // Oracle: P is the one leaf that na lacks, so it is the one imputed leaf.
    assert!(imputed == 1 && added > 0, "imputed {imputed}, added {added}");
    let p = view.right.nodes.iter().find(|n| n.name == "P").unwrap();
    // Oracle: the trees share A, B, C, and D, which form the one MCC of the pair, so P joins it
    // and has its slot 0.
    assert_eq!(Some(0), p.mcc.map(|m| view.mccs[m].slot));
    let p_color = colors.mcc[0].clone();
    let mut rings: Vec<String> = elements(&svg)
      .iter()
      .filter(|e| e.name == "circle")
      .filter_map(|e| e.attribute("stroke").map(str::to_owned))
      .filter(|stroke| colors.mcc.contains(stroke))
      .collect();
    rings.sort();
    // The ring of P in the color of its MCC, and the legend ring in the color of slot 0.
    let mut expected = vec![p_color, colors.mcc[0].clone()];
    expected.sort();
    assert_eq!(expected, rings);
    let muted = elements(&svg)
      .iter()
      .filter(|e| e.name == "path" && e.attribute("stroke") == Some(colors.ink_muted.as_str()))
      .count();
    // The legend adds one ink-muted line.
    assert_eq!(added + 1, muted);
    let legend: Vec<String> = texts(&svg).into_iter().filter(|t| t.len() > 1).collect();
    assert!(
      legend.contains(&"Imputed leaf".to_owned())
        && legend.contains(&"Node added by resolution or imputation".to_owned()),
      "{legend:?}"
    );
  }

  #[test]
  fn tanglegram_svg_draws_only_the_part_across_of_an_added_branch_in_ink_muted() {
    // Resolution copies na's split (B,C) into the polytomy of ha with branch length 0.
    let r = run_trees(&[
      ("ha", "((A:1,B:1,C:1):1,(D:1,E:1):1);"),
      ("na", "((A:1,(B:1,C:1):1):1,(D:1,E:1):1);"),
    ]);
    let view = display::pair_view(&r, 0, TreeVersion::Resolved, Scale::Div).unwrap();
    let svg = tanglegram_svg(&view, &options(1200.0, 12.0, Scale::Div, LabelMode::On)).unwrap();
    let colors = palette::palette().light;
    let elements = elements(&svg);
    let paths: Vec<(String, String)> = elements
      .iter()
      .filter(|e| e.name == "path")
      .filter_map(|e| Some((e.attribute("stroke")?.to_owned(), e.attribute("d")?.to_owned())))
      .collect();
    let commands = |d: &str| -> String { d.chars().filter(char::is_ascii_alphabetic).collect() };
    let numbers = |d: &str| -> Vec<f64> { d.split([' ', 'M', 'H', 'V']).filter_map(|v| v.parse().ok()).collect() };
    let muted: Vec<&str> = elements
      .iter()
      .filter(|e| e.attribute("stroke") == Some(colors.ink_muted.as_str()))
      .filter(|e| e.attribute("stroke-dasharray").is_none())
      .filter_map(|e| e.attribute("d"))
      .collect();
    // Oracle: the one added node of the left tree, and the legend line; each runs across, from
    // left to right, and is longer than 0.
    let shapes: Vec<(String, bool)> = muted
      .iter()
      .map(|d| (commands(d), numbers(d)[0] < numbers(d)[2]))
      .collect();
    assert_eq!(vec![("MH".to_owned(), true); 2], shapes, "{muted:?}");
    // The part along the parent's x stays in the color of the MCC.
    let stems: Vec<&(String, String)> = paths
      .iter()
      .filter(|(stroke, d)| colors.mcc.contains(stroke) && commands(d) == "MV")
      .collect();
    assert_eq!(1, stems.len(), "{paths:?}");
  }

  #[test]
  fn tanglegram_svg_rejects_invalid_options() {
    let svg = tanglegram_svg(
      &example_view(Scale::Div),
      &options(0.0, -1.0, Scale::Div, LabelMode::Auto),
    );
    let expected = vec![
      error(FigureOptionKey::Width, "figure width must be a positive number, got 0"),
      error(
        FigureOptionKey::RowHeight,
        "row height must be a positive number, got -1",
      ),
    ];
    assert_eq!(Err(expected), svg);
  }

  #[test]
  fn tanglegram_svg_rejects_options_above_the_bounds() {
    let svg = tanglegram_svg(
      &example_view(Scale::Div),
      &options(1e308, 1000.5, Scale::Div, LabelMode::Auto),
    );
    let expected = vec![
      error(
        FigureOptionKey::Width,
        &format!("figure width must be at most 100000 px, got {}", 1e308),
      ),
      error(
        FigureOptionKey::RowHeight,
        "row height must be at most 1000 px, got 1000.5",
      ),
    ];
    assert_eq!(Err(expected), svg);
  }

  #[test]
  fn tanglegram_svg_at_the_bounds_has_finite_numbers() {
    let svg = tanglegram_svg(
      &example_view(Scale::Div),
      &options(MAX_FIGURE_WIDTH, MAX_ROW_HEIGHT, Scale::Div, LabelMode::On),
    )
    .unwrap();
    let root = elements(&svg).into_iter().find(|e| e.name == "svg").unwrap();
    // Oracle: 52 px above the rows, 5 rows of 1000 px, then the legend band of 16 + 20 px and
    // the 16 px margin.
    assert_eq!(
      (Some("100000"), Some("5104")),
      (root.attribute("width"), root.attribute("height"))
    );
    assert!(!svg.contains("inf") && !svg.contains("NaN"), "{svg}");
  }

  #[test]
  fn tanglegram_svg_too_narrow_for_one_character_has_no_labels_or_leaders() {
    let svg = tanglegram_svg(
      &example_view(Scale::Depth),
      &options(100.0, 12.0, Scale::Depth, LabelMode::On),
    )
    .unwrap();
    // Oracle: a label column takes at most 0.25 of half the inner width, 0.25 * 68 / 2 = 8.5 px,
    // less than its two 6 px gaps.
    let parsed = elements(&svg);
    let labels: Vec<&str> = parsed
      .iter()
      .filter(|e| e.name == "text")
      .map(|e| e.text.as_str())
      .filter(|t| ["A", "B", "C", "D", "X"].contains(t))
      .collect();
    let leaders = parsed
      .iter()
      .filter(|e| e.attribute("stroke-dasharray") == Some("1 3"))
      .count();
    assert_eq!((Vec::<&str>::new(), 0), (labels, leaders));
  }

  #[test]
  fn arg_svg_draws_no_leader_for_a_leaf_without_label_text() {
    // Oracle: a 132 px figure has an inner width of 100 px and an ARG label column of at most
    // 25 px, which leaves 13 px (1083 units) for a label: "A" (667) fits, "L…" (1556) does not.
    let (ha, na) = (
      "((A:1,B:1):1,(Long:1,(C:1,D:1):1):1);",
      "((A:1,B:1):1,(Long:1,(C:1,D:1):1):1);",
    );
    let r = run_trees(&[("ha", ha), ("na", na)]);
    let view = display::arg_view(&r, Scale::Div).unwrap();
    let svg = arg_svg(&view, ["ha", "na"], &options(132.0, 12.0, Scale::Div, LabelMode::On)).unwrap();
    let away = |l: &&display::Leader| l.from[0] < l.to[0];
    let long = view.nodes.iter().position(|n| n.label == "Long").unwrap();
    assert!(view.shapes.leaders.iter().filter(away).any(|l| l.node == long));
    let labeled = view
      .shapes
      .leaders
      .iter()
      .filter(away)
      .filter(|l| l.node != long)
      .count();
    let parsed = elements(&svg);
    // The leaders are the paths of the group with the dotted stroke.
    let group = parsed
      .iter()
      .position(|e| e.name == "g" && e.attribute("stroke-dasharray") == Some("1 3"))
      .unwrap();
    let leaders = parsed[group + 1..]
      .iter()
      .take_while(|e| e.name == "path" && e.attribute("stroke").is_none())
      .count();
    let labels = parsed
      .iter()
      .filter(|e| e.name == "text" && e.text.contains('\u{2026}'))
      .count();
    assert_eq!((labeled, 0), (leaders, labels));
  }

  #[test]
  fn arg_svg_without_reassortment_has_no_rings_or_reassortment_entry() {
    let r = run_trees(&[("ha", HA), ("na", HA)]);
    let view = display::arg_view(&r, Scale::Depth).unwrap();
    let svg = arg_svg(&view, ["ha", "na"], &options(800.0, 12.0, Scale::Depth, LabelMode::Off)).unwrap();
    let parsed = elements(&svg);
    let texts: Vec<&str> = parsed
      .iter()
      .filter(|e| e.name == "text")
      .map(|e| e.text.as_str())
      .collect();
    let circles = parsed.iter().filter(|e| e.name == "circle").count();
    assert_eq!(
      (vec!["ARG of ha and na", "Segment ha", "Segment na", "Both segments"], 0),
      (texts, circles)
    );
  }

  #[test]
  fn arg_svg_of_the_two_tree_example() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    let view = display::arg_view(&r, Scale::Depth).unwrap();
    let svg = arg_svg(&view, ["ha", "na"], &options(800.0, 12.0, Scale::Depth, LabelMode::On)).unwrap();
    let colors = palette::palette().light;
    let parsed = elements(&svg);
    assert_eq!(
      "ARG of ha and na",
      parsed.iter().find(|e| e.name == "title").unwrap().text
    );
    let reticulations = view.edges.iter().filter(|e| e.reticulation).count();
    let hybrids = view.nodes.iter().filter(|n| n.hybrid).count();
    // Oracle: one reassortment in the two-tree example (`fixtures/doc_mccs_1.json`): X is the one
    // hybrid node, with one reticulation edge.
    assert_eq!((1, 1), (reticulations, hybrids));
    let dashed = parsed
      .iter()
      .filter(|e| e.name == "path" && e.attribute("stroke-dasharray") == Some("4 3"))
      .count();
    let rings = parsed
      .iter()
      .filter(|e| e.name == "circle" && e.attribute("stroke") == Some(colors.signal.as_str()))
      .count();
    // The legend adds one dashed curve and one ring.
    assert_eq!((reticulations + 1, hybrids + 1), (dashed, rings));
    // The legend curve has the color of the reticulation it explains.
    let strokes: std::collections::BTreeSet<&str> = parsed
      .iter()
      .filter(|e| e.name == "path" && e.attribute("stroke-dasharray") == Some("4 3"))
      .filter_map(|e| e.attribute("stroke"))
      .collect();
    assert_eq!(1, strokes.len(), "{strokes:?}");
    let expected: Vec<String> = [
      "ARG of ha and na",
      "A",
      "B",
      "X",
      "C",
      "D",
      "Segment ha",
      "Segment na",
      "Both segments",
      "Reassortment",
    ]
    .map(str::to_owned)
    .to_vec();
    let mut actual = texts(&svg);
    actual.sort_by_key(|t| expected.iter().position(|e| e == t));
    assert_eq!(expected, actual);
  }

  #[test]
  fn arg_svg_rejects_invalid_options() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    let view = display::arg_view(&r, Scale::Div).unwrap();
    let svg = arg_svg(
      &view,
      ["ha", "na"],
      &options(f64::NAN, 12.0, Scale::Div, LabelMode::Auto),
    );
    assert_eq!(
      Err(vec![error(
        FigureOptionKey::Width,
        "figure width must be a positive number, got NaN"
      )]),
      svg
    );
  }

  #[test]
  fn figure_options_missing_fields_take_defaults() {
    let options: FigureOptions = serde_json::from_value(json!({"labels": "on"})).unwrap();
    let expected = FigureOptions {
      labels: LabelMode::On,
      ..FigureOptions::default()
    };
    assert_eq!(expected, options);
  }

  #[test]
  fn figure_option_keys_are_fields_of_the_options() {
    // Oracle: the serialized `FigureOptions`, the options that the web app sends.
    let options = serde_json::to_value(FigureOptions::default()).unwrap();
    let not_in_options: Vec<String> = FigureOptionKey::VARIANTS
      .iter()
      .map(wire_name)
      .filter(|n| options.get(n).is_none())
      .collect();
    assert_eq!(Vec::<String>::new(), not_in_options);
  }

  /// The error of the option `key` with `message`.
  fn error(key: FigureOptionKey, message: &str) -> ValidationError {
    ValidationError::at(Field::FigureOption { key }, message)
  }

  #[test]
  fn figure_options_default_is_valid() {
    assert_eq!(
      Vec::<ValidationError>::new(),
      check_figure_options(&FigureOptions::default())
    );
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::width_smallest(     (33.0,              1.0),      vec![])]
  #[case::width_below_margins((32.5,              12.0),     vec![error(FigureOptionKey::Width, "figure width must be at least 33 px, got 32.5")])]
  #[case::width_tiny(         (f64::MIN_POSITIVE, 12.0),     vec![error(FigureOptionKey::Width, &format!("figure width must be at least 33 px, got {}", f64::MIN_POSITIVE))])]
  #[case::row_height_below_px((1200.0,            0.004),    vec![error(FigureOptionKey::RowHeight, "row height must be at least 1 px, got 0.004")])]
  #[case::width_zero(         (0.0,               12.0),     vec![error(FigureOptionKey::Width, "figure width must be a positive number, got 0")])]
  #[case::width_negative(     (-1.0,              12.0),     vec![error(FigureOptionKey::Width, "figure width must be a positive number, got -1")])]
  #[case::width_nan(          (f64::NAN,          12.0),     vec![error(FigureOptionKey::Width, "figure width must be a positive number, got NaN")])]
  #[case::width_infinite(     (f64::INFINITY,     12.0),     vec![error(FigureOptionKey::Width, "figure width must be a positive number, got inf")])]
  #[case::row_height_zero(    (1200.0,            0.0),      vec![error(FigureOptionKey::RowHeight, "row height must be a positive number, got 0")])]
  #[case::row_height_infinite((1200.0,            f64::NEG_INFINITY), vec![error(FigureOptionKey::RowHeight, "row height must be a positive number, got -inf")])]
  #[case::both_invalid(       (0.0,               f64::NAN), vec![
    error(FigureOptionKey::Width, "figure width must be a positive number, got 0"),
    error(FigureOptionKey::RowHeight, "row height must be a positive number, got NaN"),
  ])]
  #[trace]
  fn figure_options_width_and_row_height_must_be_finite_and_positive(
    #[case] (width, row_height): (f64, f64),
    #[case] expected: Vec<ValidationError>,
  ) {
    let options = FigureOptions {
      width,
      row_height,
      ..FigureOptions::default()
    };
    assert_eq!(expected, check_figure_options(&options));
  }

  #[test]
  fn figure_options_default_serializes_camel_case() {
    let expected = json!({"width": 1200.0, "rowHeight": 12.0, "scale": "div", "labels": "auto"});
    assert_eq!(expected, serde_json::to_value(FigureOptions::default()).unwrap());
  }
}
