//! Colors of the drawings, for the light and the dark theme. The CSS color tokens of the web app
//! equal them.
//!
//! Contrast: every MCC slot and the "no MCC" color reach 3:1 against the ground of its theme
//! (WCAG 2.2 relative luminance, the minimum for graphic marks). A color that fell short had only
//! its HSL lightness changed, keeping hue and saturation: darkened for the light theme, lightened
//! for the dark theme, until it reached 3:1. In the light theme, slot 1 (gold, `#c9a227` at
//! 2.21:1), slot 6 (orange, `#d07a1e` at 2.96:1), and "no MCC" (`#9aa5a2` at 2.32:1) were
//! darkened. In the dark theme, slots 0, 3, and 4 were lightened from the light hues.
//!
//! Color-vision check (measured when the colors were chosen; no test repeats it): the slots were
//! simulated for protanopia, deuteranopia, and tritanopia
//! with the matrices of Machado, Oliveira, and Fernandes (2009, severity 1), and compared by
//! CIEDE2000 color difference. With the contrast-fitted colors, gold and orange (slots 1 and 6)
//! differed by only 1.7 under deuteranopia and 2.5 under protanopia, blue and purple (slots 0 and
//! 3) by 3.6 in the dark theme. Lightness changes alone (at most 0.12 in HSL lightness, every
//! slot kept at 3:1 and a CIEDE2000 difference of at least 20 from the ink color) raised the smallest difference between
//! any two slots under any simulation to 4.9 in the light theme (slots 1 and 6, protanopia) and
//! 6.1 in the dark theme: slot 1 was darkened, slot 3 lightened, and slot 5 darkened in the light
//! theme; slots 0 and 5 were changed in the dark theme. Without simulation the smallest
//! difference is 14.9 (light) and 7.6 (dark).

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

/// The drawing colors of both themes.
pub fn palette() -> Palette {
  Palette {
    light: theme(
      [
        "#2f4b9a", "#93771c", "#2c8c83", "#7c4fac", "#7a5537", "#677630", "#ce791e", "#5b7c99",
      ],
      "#83908d",
      [
        "#f3f5f4", "#1f2b30", "#55656b", "#b0265e", "#2457c5", "#3e6a8a", "#8a6a3e",
      ],
    ),
    dark: theme(
      [
        "#617ecf", "#c9a227", "#2c8c83", "#8054b0", "#89603e", "#6f7f33", "#d07a1e", "#5b7c99",
      ],
      "#6e7c79",
      [
        "#162024", "#dce4e1", "#9aaaa6", "#e0619a", "#7da2f0", "#406e8f", "#8a6a3e",
      ],
    ),
  }
}

/// The colors of one theme: the MCC slots, "no MCC", and the interface colors ground, ink,
/// ink-muted, signal, focus, segment A, and segment B.
fn theme(
  mcc: [&str; 8],
  no_mcc: &str,
  [ground, ink, ink_muted, signal, focus, segment_a, segment_b]: [&str; 7],
) -> ThemeColors {
  ThemeColors {
    mcc: mcc.map(str::to_owned),
    no_mcc: no_mcc.to_owned(),
    ground: ground.to_owned(),
    ink: ink.to_owned(),
    ink_muted: ink_muted.to_owned(),
    signal: signal.to_owned(),
    focus: focus.to_owned(),
    segment_a: segment_a.to_owned(),
    segment_b: segment_b.to_owned(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use std::collections::BTreeMap;

  const INDEX_CSS: &str = include_str!("../../web/src/index.css");

  #[derive(Clone, Copy, Debug)]
  enum Theme {
    Light,
    Dark,
  }

  fn colors(theme: Theme) -> ThemeColors {
    let p = palette();
    match theme {
      Theme::Light => p.light,
      Theme::Dark => p.dark,
    }
  }

  #[rstest]
  #[case::light(Theme::Light)]
  #[case::dark(Theme::Dark)]
  fn mcc_and_no_mcc_colors_reach_three_to_one_against_the_ground(#[case] theme: Theme) {
    // Oracle: WCAG 2.2 non-text contrast, 3:1 for graphic marks.
    let c = colors(theme);
    let marks: Vec<String> = c.mcc.iter().chain([&c.no_mcc]).cloned().collect();
    let failing: Vec<(String, f64)> = marks
      .into_iter()
      .map(|m| {
        let ratio = contrast(&m, &c.ground);
        (m, ratio)
      })
      .filter(|&(_, ratio)| ratio < 3.0)
      .collect();
    assert_eq!(Vec::<(String, f64)>::new(), failing);
  }

  #[rstest]
  #[case::light(Theme::Light)]
  #[case::dark(Theme::Dark)]
  fn drawing_colors_equal_the_css_tokens(#[case] theme: Theme) {
    let c = colors(theme);
    let expected: BTreeMap<&str, String> = [
      ("ground", c.ground),
      ("ink", c.ink),
      ("ink-muted", c.ink_muted),
      ("signal", c.signal),
      ("focus", c.focus),
      ("segment-a", c.segment_a),
      ("segment-b", c.segment_b),
    ]
    .into_iter()
    .map(|(name, value)| (name, value.to_lowercase()))
    .collect();
    let tokens = css_tokens(match theme {
      Theme::Light => "@theme static {",
      Theme::Dark => ":root.dark {",
    });
    let actual: BTreeMap<&str, String> = expected.keys().map(|&name| (name, tokens[name].clone())).collect();
    assert_eq!(expected, actual);
  }

  #[rstest]
  #[case::light(Theme::Light)]
  #[case::dark(Theme::Dark)]
  fn mcc_tokens_equal_the_palette(#[case] theme: Theme) {
    // The web app writes the MCC tokens at start-up; the values in the CSS are the palette of
    // each theme, so the page draws with them before the palette arrives.
    let c = colors(theme);
    let tokens = css_tokens(match theme {
      Theme::Light => "@theme static {",
      Theme::Dark => ":root.dark {",
    });
    let mut expected: Vec<String> = c.mcc.iter().map(|m| m.to_lowercase()).collect();
    expected.push(c.no_mcc.to_lowercase());
    let mut actual: Vec<String> = (0..8).map(|i| tokens[format!("mcc-{i}").as_str()].clone()).collect();
    actual.push(tokens["mcc-none"].clone());
    assert_eq!(expected, actual);
  }

  #[rstest]
  #[case::black_on_white("#000000", "#ffffff", 21.0)]
  #[case::equal_colors("#777777", "#777777", 1.0)]
  fn contrast_matches_the_wcag_bounds(#[case] a: &str, #[case] b: &str, #[case] expected: f64) {
    // Oracle: WCAG 2.2 defines contrast ratios from 1:1 to 21:1 (black on white).
    assert!((contrast(a, b) - expected).abs() < 1e-9, "{}", contrast(a, b));
  }

  #[test]
  fn contrast_of_the_lightest_gray_with_4_5_to_1_on_white() {
    // Oracle: #767676 is the lightest gray with at least 4.5:1 on white, 4.54:1 (rounded), the
    // value of WebAIM's contrast checker; #777777 falls short.
    let actual = [contrast("#767676", "#ffffff"), contrast("#777777", "#ffffff")].map(|c| format!("{c:.2}"));
    assert_eq!(["4.54", "4.48"], actual);
  }

  /// The `--color-<name>: <value>;` tokens of the CSS block that starts with `opening`, by name
  /// without the `--color-` prefix, with lowercase values.
  fn css_tokens(opening: &str) -> BTreeMap<String, String> {
    let (_, rest) = INDEX_CSS.split_once(opening).unwrap();
    let (block, _) = rest.split_once('}').unwrap();
    block
      .lines()
      .filter_map(|line| line.trim().strip_prefix("--color-"))
      .filter_map(|decl| decl.strip_suffix(';')?.split_once(':'))
      .map(|(name, value)| (name.trim().to_owned(), value.trim().to_lowercase()))
      .collect()
  }

  /// WCAG 2.2 contrast ratio of two `#rrggbb` colors.
  fn contrast(a: &str, b: &str) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
  }

  /// WCAG 2.2 relative luminance of a `#rrggbb` color.
  fn luminance(hex: &str) -> f64 {
    let rgb = u32::from_str_radix(hex.strip_prefix('#').unwrap(), 16).unwrap();
    let channel = |shift: u32| {
      let c = f64::from((rgb >> shift) & 0xff) / 255.0;
      if c <= 0.04045 {
        c / 12.92
      } else {
        ((c + 0.055) / 1.055).powf(2.4)
      }
    };
    0.2126 * channel(16) + 0.7152 * channel(8) + 0.0722 * channel(0)
  }
}
