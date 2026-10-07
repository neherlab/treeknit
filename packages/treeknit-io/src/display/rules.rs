//! The formulas of the drawing rules: the columns of a drawing and the row thresholds. The SVG
//! figures call them; the web app applies its own copy on every zoom frame and resize, which its
//! tests check against the table of `examples/drawing_cases.rs`.

use crate::display::DrawingRules;
use serde::Serialize;

impl DrawingRules {
  /// The width between the margins of a drawing `width_px` wide.
  pub fn inner_width_px(&self, width_px: f64) -> f64 {
    (width_px - 2.0 * self.margin_px).max(0.0)
  }

  /// The width of a label column whose longest label is `longest_px` wide: the label and a gap on
  /// each side, at most `max_px`; 0 without labels.
  pub fn label_column_px(&self, longest_px: f64, max_px: f64) -> f64 {
    if longest_px <= 0.0 {
      0.0
    } else {
      (longest_px + 2.0 * self.label_gap_px).min(max_px)
    }
  }

  /// The widest label column of a tanglegram `width_px` wide.
  pub fn tanglegram_label_max_px(&self, width_px: f64) -> f64 {
    self.tanglegram_label_column_max_share * self.inner_width_px(width_px) / 2.0
  }

  /// The widest label column of an ARG `width_px` wide.
  pub fn arg_label_max_px(&self, width_px: f64) -> f64 {
    self.arg_label_column_max_share * self.inner_width_px(width_px)
  }

  /// The columns of a tanglegram `width_px` wide whose two label columns are `label_px` wide.
  pub fn tanglegram_columns(&self, width_px: f64, label_px: f64) -> TanglegramColumns {
    let inner = self.inner_width_px(width_px);
    let links = (self.link_zone_share * inner - 2.0 * label_px).max(self.link_zone_min_share * inner);
    let tree = ((inner - links - 2.0 * label_px) / 2.0).max(0.0);
    let left_end = self.margin_px + tree;
    let links_start = left_end + label_px;
    let links_end = links_start + links;
    let right_start = links_end + label_px;
    TanglegramColumns {
      left: ColumnPx {
        start: self.margin_px,
        end: left_end,
      },
      left_labels: ColumnPx {
        start: left_end,
        end: links_start,
      },
      links: ColumnPx {
        start: links_start,
        end: links_end,
      },
      right_labels: ColumnPx {
        start: links_end,
        end: right_start,
      },
      right: ColumnPx {
        start: right_start,
        end: right_start + tree,
      },
    }
  }

  /// The tree column of an ARG `width_px` wide whose label column is `label_px` wide.
  pub fn arg_column(&self, width_px: f64, label_px: f64) -> ColumnPx {
    ColumnPx {
      start: self.margin_px,
      end: (width_px - self.margin_px - label_px).max(self.margin_px),
    }
  }

  /// Whether leaf labels in the label mode `auto` are drawn at `row_px` px per row.
  pub fn auto_labels_shown(&self, row_px: f64) -> bool {
    row_px >= f64::from(self.label_auto_min_row_px)
  }

  /// Whether the links of a tanglegram are drawn as ribbons of blocks, instead of one S-curve per
  /// link, at `row_px` px per row.
  pub fn ribbons_shown(&self, row_px: f64) -> bool {
    row_px < f64::from(self.link_min_row_px)
  }
}

/// The columns of a tanglegram from left to right: the left tree, its labels, the link zone, the
/// right labels, and the right tree, which the drawing mirrors.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TanglegramColumns {
  pub left: ColumnPx,
  pub left_labels: ColumnPx,
  pub links: ColumnPx,
  pub right_labels: ColumnPx,
  pub right: ColumnPx,
}

/// A column of a drawing, from its left edge `start` to its right edge `end`, in px.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ColumnPx {
  pub start: f64,
  pub end: f64,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::display::DRAWING_RULES;
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  /// Columns of a tanglegram whose left tree, label column, and link zone are `tree`, `label`, and
  /// `links` px wide, with the 16 px margin of the rules.
  fn tiled(tree: f64, label: f64, links: f64) -> TanglegramColumns {
    let column = |start: f64, width: f64| ColumnPx {
      start,
      end: start + width,
    };
    let left = column(16.0, tree);
    let left_labels = column(left.end, label);
    let links = column(left_labels.end, links);
    let right_labels = column(links.end, label);
    TanglegramColumns {
      left,
      left_labels,
      links,
      right_labels,
      right: column(right_labels.end, tree),
    }
  }

  // Oracle: the rules of `DrawingRules` at 1232 px, an inner width of 1200 px: the link zone is
  // 20% of 1200 px minus both label columns, at least 15% (180 px), and the trees share the rest.
  #[rustfmt::skip]
  #[rstest]
  #[case::no_labels(         0.0, tiled(480.0,   0.0, 240.0))]
  #[case::short_labels(     20.0, tiled(480.0,  20.0, 200.0))]
  #[case::link_zone_floor(  30.0, tiled(480.0,  30.0, 180.0))]
  #[case::wide_labels(     100.0, tiled(410.0, 100.0, 180.0))]
  #[trace]
  fn tanglegram_columns_follow_the_rules(#[case] label_px: f64, #[case] expected: TanglegramColumns) {
    assert_eq!(expected, DRAWING_RULES.tanglegram_columns(1232.0, label_px));
  }

  // Oracle: a label column is the label plus 6 px on each side, at most a quarter of half the
  // inner width in a tanglegram (150 px at 1232 px) and a quarter of it in an ARG (300 px).
  #[rustfmt::skip]
  #[rstest]
  #[case::fits(      80.0, ( 92.0,  92.0))]
  #[case::capped(   500.0, (150.0, 300.0))]
  #[case::no_labels(  0.0, (  0.0,   0.0))]
  #[trace]
  fn label_columns_fit_the_longest_label_up_to_their_share(#[case] longest_px: f64, #[case] expected: (f64, f64)) {
    let rules = DRAWING_RULES;
    let widths = (
      rules.label_column_px(longest_px, rules.tanglegram_label_max_px(1232.0)),
      rules.label_column_px(longest_px, rules.arg_label_max_px(1232.0)),
    );
    assert_eq!(expected, widths);
  }

  #[test]
  fn arg_column_ends_at_its_label_column() {
    // Oracle: the 16 px margin on each side, then the 100 px label column at the right; a drawing
    // narrower than both margins has an empty column at the left margin.
    let columns = [
      DRAWING_RULES.arg_column(1232.0, 100.0),
      DRAWING_RULES.arg_column(20.0, 0.0),
    ];
    let expected = [
      ColumnPx {
        start: 16.0,
        end: 1116.0,
      },
      ColumnPx { start: 16.0, end: 16.0 },
    ];
    assert_eq!(expected, columns);
  }

  #[test]
  fn row_thresholds_switch_labels_at_10_px_and_links_at_6_px() {
    // Oracle: labels from 10 px per row, S-curve links from 6 px per row.
    let rules = DRAWING_RULES;
    let shown = [9.5, 10.0, 5.5, 6.0].map(|row| (rules.auto_labels_shown(row), rules.ribbons_shown(row)));
    assert_eq!([(false, false), (true, false), (false, true), (false, false)], shown);
  }
}
