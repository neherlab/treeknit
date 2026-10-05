//! SVG figures of a tanglegram and of the ARG, drawn from the shapes of `display`.

use crate::display::Scale;
use serde::{Deserialize, Serialize};
#[cfg(feature = "tsify")]
use tsify::Tsify;

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

  #[test]
  fn figure_options_default_serializes_camel_case() {
    let expected = json!({"width": 1200.0, "rowHeight": 12.0, "scale": "div", "labels": "auto"});
    assert_eq!(expected, serde_json::to_value(FigureOptions::default()).unwrap());
  }
}
