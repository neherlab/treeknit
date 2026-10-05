//! Colors of the drawings, for the light and the dark theme. The SVG figures and the
//! interactive views draw with these values, and the CSS tokens of the web app equal them.

use serde::Serialize;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// Drawing colors of both themes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Palette {
  pub light: ThemeColors,
  pub dark: ThemeColors,
}

/// Drawing colors of one theme, each as `#rrggbb`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ThemeColors {
  /// The eight MCC color slots.
  pub mcc: [String; 8],
  /// Branches of nodes without an MCC.
  pub no_mcc: String,
  pub ground: String,
  pub ink: String,
  pub ink_muted: String,
  /// Reassortment, and nothing else.
  pub signal: String,
  pub focus: String,
  /// Segment A (the first tree) of the ARG.
  pub segment_a: String,
  /// Segment B (the second tree) of the ARG.
  pub segment_b: String,
}
