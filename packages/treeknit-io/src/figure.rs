//! SVG figures of a tanglegram and of the ARG, drawn from the shapes of `display` with the
//! light colors of `palette`: the same geometry as the interactive views, mapped to px.

mod arg;
mod svg;
mod tanglegram;

use crate::analysis::ValidationError;
use crate::display::{ArgView, DRAWING_RULES, PairView, Scale};
use serde::{Deserialize, Serialize};
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// The SVG tanglegram of `view`, titled with the labels of its two trees, with a legend under
/// the drawing; the errors of `check_figure_options` when `options` are invalid. The view must be
/// laid out with `options.scale`.
pub fn tanglegram_svg(view: &PairView, options: &FigureOptions) -> Result<String, Vec<ValidationError>> {
  checked(options)?;
  Ok(tanglegram::draw(view, options))
}

/// The SVG figure of the ARG `view` of the trees labeled `segments` (segment A, then B), titled
/// with them, with a legend under the drawing; the errors of `check_figure_options` when
/// `options` are invalid. The view must be laid out with `options.scale`.
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

/// Check `options`: the width and the row height must be finite and positive. Each error names
/// the field of `FigureOptions` as it is serialized, such as `rowHeight`.
pub fn check_figure_options(options: &FigureOptions) -> Vec<ValidationError> {
  let positive = |field: &str, name: &str, value: f64| {
    (!(value.is_finite() && value > 0.0)).then(|| ValidationError {
      field: Some(field.to_owned()),
      message: format!("{name} must be a positive number, got {value}"),
      line: None,
      column: None,
    })
  };
  [
    positive("width", "figure width", options.width),
    positive("rowHeight", "row height", options.row_height),
  ]
  .into_iter()
  .flatten()
  .collect()
}

fn checked(options: &FigureOptions) -> Result<(), Vec<ValidationError>> {
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "lowercase")]
pub enum LabelMode {
  /// From 10 px per row.
  #[default]
  Auto,
  On,
  Off,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::analysis::{self, Settings, TreeText};
  use crate::display::{self, TreeVersion};
  use crate::palette;
  use crate::run::{self, RunResult};
  use pretty_assertions::assert_eq;
  use quick_xml::events::BytesStart;
  use quick_xml::events::Event;
  use quick_xml::{Reader, XmlVersion};
  use rstest::rstest;
  use serde_json::json;
  use treeknit_core::Options;

  /// The two-tree example: X moved between the trees. Its MCCs are `[X]` and `[A,B,C,D]`
  /// (TreeKnit.jl fixture `fixtures/doc_mccs_1.json`).
  const HA: &str = "((A,B),(C,(D,X)));";
  const NA: &str = "((A,(B,X)),(C,D));";

  fn run_trees(trees: &[(&str, &str)]) -> (RunResult, Options) {
    let texts: Vec<TreeText> = trees
      .iter()
      .map(|(label, newick)| TreeText {
        label: (*label).to_owned(),
        newick: (*newick).to_owned(),
      })
      .collect();
    let s = Settings::default();
    let opts = analysis::options(&s, texts.len(), false).unwrap();
    let r = run::run(analysis::parse_trees(&texts).unwrap(), &opts, s.seed, &|_| {});
    (r, opts)
  }

  /// The resolved pair view of the two-tree example, laid out with `scale`.
  fn example_view(scale: Scale) -> PairView {
    let (r, opts) = run_trees(&[("ha", HA), ("na", NA)]);
    display::pair_view(&r, &opts, 0, TreeVersion::Resolved, scale).unwrap()
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
    // 6.75 + 2 * 6 = 18.75 px (one character); link zone 0.2 * 568 = 113.6 px; tree columns
    // (568 - 113.6 - 2 * 18.75) / 2 = 208.45 px, so the left tree spans 16 to 224.45, the link
    // zone 243.2 to 356.8, and the mirrored right tree 584 down to 375.55. Rows are centered at
    // 52 + 12 * (row + 0.5). Cladogram depths: ha has its leaves at 3, (A,B) and (D,X) at 2,
    // (C,(D,X)) at 1, so x = 16 + 208.45 * depth / 3. X is its own MCC (slot 1), and the branch
    // above X in each tree is a reassortment branch with a ring at its midpoint.
    let expected = r##"<svg xmlns="http://www.w3.org/2000/svg" width="600" height="164" viewBox="0 0 600 164" font-family="IBM Plex Sans, Helvetica, Arial, sans-serif" font-size="12">
  <title>ha and na</title>
  <rect width="600" height="164" fill="#f3f5f4"/>
  <text x="16" y="32" font-size="16" font-weight="600" fill="#1f2b30">ha and na</text>
  <g fill="none" stroke-width="1">
    <path d="M243.2 58 C300 58 300 58 356.8 58" stroke="#2f4b9a"/>
    <path d="M243.2 70 C300 70 300 70 356.8 70" stroke="#2f4b9a"/>
    <path d="M243.2 82 C300 82 300 94 356.8 94" stroke="#2f4b9a"/>
    <path d="M243.2 94 C300 94 300 106 356.8 106" stroke="#2f4b9a"/>
    <path d="M243.2 106 C300 106 300 82 356.8 82" stroke="#93771c"/>
  </g>
  <g fill="none" stroke-width="1.5">
    <path d="M16 77.5 V64 H154.97" stroke="#2f4b9a"/>
    <path d="M154.97 64 V58 H224.45" stroke="#2f4b9a"/>
    <path d="M154.97 64 V70 H224.45" stroke="#2f4b9a"/>
    <path d="M16 77.5 V91 H85.48" stroke="#2f4b9a"/>
    <path d="M85.48 91 V82 H224.45" stroke="#2f4b9a"/>
    <path d="M85.48 91 V100 H154.97" stroke="#2f4b9a"/>
    <path d="M154.97 100 V94 H224.45" stroke="#2f4b9a"/>
    <path d="M154.97 100 V106 H224.45" stroke="#b0265e" stroke-width="2"/>
  </g>
  <circle cx="189.71" cy="106" r="3.5" fill="#f3f5f4" stroke="#b0265e" stroke-width="1.5"/>
  <g font-family="IBM Plex Sans Condensed, IBM Plex Sans, Helvetica, Arial, sans-serif" fill="#1f2b30" text-anchor="start">
    <text x="230.45" y="62.2">A</text>
    <text x="230.45" y="74.2">B</text>
    <text x="230.45" y="86.2">C</text>
    <text x="230.45" y="98.2">D</text>
    <text x="230.45" y="110.2">X</text>
  </g>
  <g fill="none" stroke-width="1.5">
    <path d="M584 83.5 V67 H514.52" stroke="#2f4b9a"/>
    <path d="M514.52 67 V58 H375.55" stroke="#2f4b9a"/>
    <path d="M514.52 67 V76 H445.03" stroke="#2f4b9a"/>
    <path d="M445.03 76 V70 H375.55" stroke="#2f4b9a"/>
    <path d="M584 83.5 V100 H445.03" stroke="#2f4b9a"/>
    <path d="M445.03 100 V94 H375.55" stroke="#2f4b9a"/>
    <path d="M445.03 100 V106 H375.55" stroke="#2f4b9a"/>
    <path d="M445.03 76 V82 H375.55" stroke="#b0265e" stroke-width="2"/>
  </g>
  <circle cx="410.29" cy="82" r="3.5" fill="#f3f5f4" stroke="#b0265e" stroke-width="1.5"/>
  <g font-family="IBM Plex Sans Condensed, IBM Plex Sans, Helvetica, Arial, sans-serif" fill="#1f2b30" text-anchor="end">
    <text x="369.55" y="62.2">A</text>
    <text x="369.55" y="74.2">B</text>
    <text x="369.55" y="86.2">X</text>
    <text x="369.55" y="98.2">C</text>
    <text x="369.55" y="110.2">D</text>
  </g>
  <g fill="#1f2b30">
    <path d="M16 138 H40" fill="none" stroke="#b0265e" stroke-width="2"/>
    <circle cx="28" cy="138" r="3.5" fill="#f3f5f4" stroke="#b0265e" stroke-width="1.5"/>
    <text x="46" y="142.2">Reassortment branch</text>
    <path d="M194.25 134 C206.25 134 206.25 142 218.25 142" fill="none" stroke="#2f4b9a" stroke-width="1.5"/>
    <text x="224.25" y="142.2">Leaves of one MCC</text>
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
    let (r, opts) = run_trees(&[("ha", "((A,B),(C,(D,P)));"), ("na", "((A,B,C),D);")]);
    let view = display::pair_view(&r, &opts, 0, TreeVersion::Imputed, Scale::Depth).unwrap();
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
    assert!(imputed > 0 && added > 0, "imputed {imputed}, added {added}");
    let dashed = elements(&svg)
      .iter()
      .filter(|e| {
        e.attribute("stroke") == Some(colors.ink_muted.as_str()) && e.attribute("stroke-dasharray") == Some("4 3")
      })
      .count();
    // The legend adds one dashed line.
    assert_eq!(added + 1, dashed);
    let legend: Vec<String> = texts(&svg).into_iter().filter(|t| t.len() > 1).collect();
    assert!(
      legend.contains(&"Imputed leaf".to_owned()) && legend.contains(&"Node added by resolution".to_owned()),
      "{legend:?}"
    );
  }

  #[test]
  fn tanglegram_svg_rejects_invalid_options() {
    let errors = tanglegram_svg(
      &example_view(Scale::Div),
      &options(0.0, -1.0, Scale::Div, LabelMode::Auto),
    )
    .unwrap_err();
    let expected = vec![
      error("width", "figure width must be a positive number, got 0"),
      error("rowHeight", "row height must be a positive number, got -1"),
    ];
    assert_eq!(expected, errors);
  }

  #[test]
  fn arg_svg_of_the_two_tree_example() {
    let (r, _) = run_trees(&[("ha", HA), ("na", NA)]);
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
    let (r, _) = run_trees(&[("ha", HA), ("na", NA)]);
    let view = display::arg_view(&r, Scale::Div).unwrap();
    let errors = arg_svg(
      &view,
      ["ha", "na"],
      &options(f64::NAN, 12.0, Scale::Div, LabelMode::Auto),
    )
    .unwrap_err();
    assert_eq!(
      vec![error("width", "figure width must be a positive number, got NaN")],
      errors
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

  /// The error for `field` with `message`.
  fn error(field: &str, message: &str) -> ValidationError {
    ValidationError {
      field: Some(field.to_owned()),
      message: message.to_owned(),
      line: None,
      column: None,
    }
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
  #[case::width_smallest_positive((f64::MIN_POSITIVE, 12.0),     vec![])]
  #[case::width_zero(             (0.0,               12.0),     vec![error("width", "figure width must be a positive number, got 0")])]
  #[case::width_negative(         (-1.0,              12.0),     vec![error("width", "figure width must be a positive number, got -1")])]
  #[case::width_nan(              (f64::NAN,          12.0),     vec![error("width", "figure width must be a positive number, got NaN")])]
  #[case::width_infinite(         (f64::INFINITY,     12.0),     vec![error("width", "figure width must be a positive number, got inf")])]
  #[case::row_height_zero(        (1200.0,            0.0),      vec![error("rowHeight", "row height must be a positive number, got 0")])]
  #[case::row_height_infinite(    (1200.0,            f64::NEG_INFINITY), vec![error("rowHeight", "row height must be a positive number, got -inf")])]
  #[case::both_invalid(           (0.0,               f64::NAN), vec![
    error("width", "figure width must be a positive number, got 0"),
    error("rowHeight", "row height must be a positive number, got NaN"),
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
