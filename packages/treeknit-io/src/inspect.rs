//! Checks of input trees before a run: the shape of one tree, and the leaf overlap of all trees.

use serde::Serialize;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// Shape of one input tree, or why it does not parse.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct TreeInspection {
  pub label: String,
  /// Number of leaves; 0 when the tree does not parse.
  pub leaves: usize,
  /// Number of internal nodes, the root included.
  pub internal_nodes: usize,
  /// Number of internal nodes with more than two children.
  pub polytomies: usize,
  /// Which branches have a length.
  pub branch_lengths: BranchLengths,
  /// Warnings of the Newick parser, such as "more than one tree in file, using the first".
  pub warnings: Vec<String>,
  /// Why the Newick text does not parse; `None` when it parses.
  pub error: Option<TreeError>,
}

/// Presence of branch lengths in a tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "lowercase")]
pub enum BranchLengths {
  /// Every non-root branch has a length.
  All,
  /// Some non-root branches have a length.
  Some,
  /// No branch has a length.
  None,
}

/// Parse error of a Newick text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct TreeError {
  pub message: String,
  /// 1-based line of the error; `None` for an error without a position.
  pub line: Option<usize>,
  /// 1-based column of the error, in Unicode characters.
  pub column: Option<usize>,
}

/// Leaf overlap of the input trees and of each pair, before a run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Overlap {
  /// Number of distinct leaves over the trees that parse.
  pub total_leaves: usize,
  /// The trees that parse, in input order.
  pub trees: Vec<TreeOverlap>,
  /// Pairs of trees that parse, in pipeline order (0,1), (0,2), ..., (1,2), ...
  pub pairs: Vec<PairOverlap>,
  /// Input indices of the trees that do not parse.
  pub failed: Vec<usize>,
}

/// Leaves of one tree against all leaves.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct TreeOverlap {
  /// Input index of the tree.
  pub index: usize,
  pub label: String,
  /// Number of leaves of the tree.
  pub leaves: usize,
  /// Number of leaves that other trees have and this tree lacks.
  pub missing: usize,
}

/// Leaves shared by the trees `i` and `j` (input indices, `i < j`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct PairOverlap {
  pub i: usize,
  pub j: usize,
  /// Number of shared leaves.
  pub shared: usize,
  /// The pair shares fewer than `analysis::MIN_SHARED_LEAVES` leaves, so the request does not
  /// validate.
  pub blocked: bool,
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use serde_json::json;

  #[test]
  fn tree_inspection_serializes_branch_lengths_and_error() {
    let inspection = TreeInspection {
      label: "ha".into(),
      leaves: 0,
      internal_nodes: 0,
      polytomies: 0,
      branch_lengths: BranchLengths::None,
      warnings: vec![],
      error: Some(TreeError {
        message: "expected ')'".into(),
        line: Some(1),
        column: Some(7),
      }),
    };
    let expected = json!({
      "label": "ha", "leaves": 0, "internalNodes": 0, "polytomies": 0, "branchLengths": "none",
      "warnings": [], "error": {"message": "expected ')'", "line": 1, "column": 7},
    });
    assert_eq!(expected, serde_json::to_value(&inspection).unwrap());
  }
}
