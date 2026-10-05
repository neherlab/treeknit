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
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::json;

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
