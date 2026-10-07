//! Ranges of leaf rows: the rows a node, a clade, or an MCC covers in a drawing.

use serde::Serialize;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// The first and the last leaf row of a range, both included; a row is the rank of a leaf in
/// display order, from 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct RowSpan {
  pub first: usize,
  pub last: usize,
}

impl RowSpan {
  /// The range of the one row `row`.
  pub fn row(row: usize) -> Self {
    Self { first: row, last: row }
  }

  /// The smallest range that covers `self` and `other`.
  #[must_use]
  pub fn union(self, other: Self) -> Self {
    Self {
      first: self.first.min(other.first),
      last: self.last.max(other.last),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  #[rustfmt::skip]
  #[rstest]
  #[case::disjoint(   (RowSpan { first: 0, last: 1 }, RowSpan { first: 4, last: 6 }), RowSpan { first: 0, last: 6 })]
  #[case::reversed(   (RowSpan { first: 4, last: 6 }, RowSpan { first: 0, last: 1 }), RowSpan { first: 0, last: 6 })]
  #[case::contained(  (RowSpan { first: 0, last: 9 }, RowSpan { first: 3, last: 4 }), RowSpan { first: 0, last: 9 })]
  #[case::same_row(   (RowSpan::row(2),               RowSpan::row(2)),               RowSpan::row(2))]
  #[trace]
  fn union_covers_both_ranges(#[case] (a, b): (RowSpan, RowSpan), #[case] expected: RowSpan) {
    assert_eq!(expected, a.union(b));
  }
}
