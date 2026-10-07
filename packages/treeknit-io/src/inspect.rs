//! Checks of input trees before a run: the shape of one tree, and the leaf overlap of all trees.

use crate::analysis::{self, MIN_SHARED_LEAVES, TreeText};
use crate::newick;
use serde::Serialize;
use treeknit_core::Tree;
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

/// Inspect the first tree of the Newick `text` of the tree labeled `label`: its counts, branch
/// lengths, and parser warnings, or its parse error with the line and column. A text with
/// several trees has the warning about them also when its first tree does not parse.
pub fn inspect_tree(label: &str, text: &str) -> TreeInspection {
  match newick::parse_first(text, label) {
    Ok(parsed) => {
      let t = &parsed.tree;
      let internals = t.internals();
      TreeInspection {
        label: label.to_owned(),
        leaves: t.n_leaves(),
        internal_nodes: internals.len(),
        polytomies: internals.iter().filter(|&&n| t.children(n).len() > 2).count(),
        branch_lengths: branch_lengths(t),
        warnings: parsed.warnings.iter().map(ToString::to_string).collect(),
        error: None,
      }
    },
    Err(e) => {
      let position = e.offset.map(|o| newick::line_column(text, o));
      TreeInspection {
        label: label.to_owned(),
        leaves: 0,
        internal_nodes: 0,
        polytomies: 0,
        branch_lengths: BranchLengths::None,
        warnings: e.warnings.iter().map(ToString::to_string).collect(),
        error: Some(TreeError {
          message: e.message,
          line: position.map(|(l, _)| l),
          column: position.map(|(_, c)| c),
        }),
      }
    },
  }
}

/// Leaf overlap of `trees` and of each pair. Each tree is parsed on its own, so a tree that
/// does not parse is left out and listed in `failed` instead of hiding the overlap of the others.
pub fn overlap(trees: &[TreeText]) -> Overlap {
  let mut failed = Vec::new();
  let mut indices = Vec::new();
  let mut parsed = Vec::new();
  for (i, t) in trees.iter().enumerate() {
    match newick::parse_first(&t.newick, &t.label) {
      Ok(p) => {
        indices.push(i);
        parsed.push(p.tree);
      },
      Err(_) => failed.push(i),
    }
  }
  let numbered = analysis::number_leaves(parsed);
  let total_leaves = numbered.taxa.len();
  let tree_overlaps = numbered
    .trees
    .iter()
    .zip(&indices)
    .map(|(t, &i)| TreeOverlap {
      index: i,
      label: trees[i].label.clone(),
      leaves: t.n_leaves(),
      missing: total_leaves - t.n_leaves(),
    })
    .collect();
  let pairs = analysis::shared_leaf_counts(&numbered.trees, total_leaves)
    .into_iter()
    .map(|p| PairOverlap {
      i: indices[p.i],
      j: indices[p.j],
      shared: p.shared,
      blocked: p.shared < MIN_SHARED_LEAVES,
    })
    .collect();
  Overlap {
    total_leaves,
    trees: tree_overlaps,
    pairs,
    failed,
  }
}

fn branch_lengths(t: &Tree) -> BranchLengths {
  let (with, without) = t
    .preorder()
    .into_iter()
    .filter(|&n| n != t.root)
    .fold((0, 0), |(w, wo), n| {
      if t.node(n).branch_length.is_some() {
        (w + 1, wo)
      } else {
        (w, wo + 1)
      }
    });
  match (with, without) {
    (0, _) => BranchLengths::None,
    (_, 0) => BranchLengths::All,
    _ => BranchLengths::Some,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::json;

  #[test]
  fn inspect_tree_counts_leaves_internal_nodes_and_polytomies() {
    // Five leaves under three internal nodes; the root has three children.
    let expected = TreeInspection {
      label: "ha".into(),
      leaves: 5,
      internal_nodes: 3,
      polytomies: 1,
      branch_lengths: BranchLengths::All,
      warnings: vec![],
      error: None,
    };
    assert_eq!(expected, inspect_tree("ha", "((A:1,B:1):1,C:1,(D:1,X:1):1);\n"));
  }

  #[rstest]
  #[case::all("((A:1,B:2):3,C:4);", BranchLengths::All)]
  #[case::root_length_is_ignored("((A:1,B:2):3,C:4):5;", BranchLengths::All)]
  #[case::some("((A:1,B):3,C);", BranchLengths::Some)]
  #[case::none("((A,B),C);", BranchLengths::None)]
  #[case::invalid_length_counts_as_missing("((A:1,B:x):3,C:4);", BranchLengths::Some)]
  fn inspect_tree_classifies_branch_lengths(#[case] newick: &str, #[case] expected: BranchLengths) {
    assert_eq!(expected, inspect_tree("t", newick).branch_lengths);
  }

  #[test]
  fn inspect_tree_reports_parser_warnings() {
    let inspection = inspect_tree("t", "((A,B):0.R,C);\n(A,B,C);\n");
    let expected = vec![
      "ignoring invalid branch length '0.R'".to_owned(),
      "more than one tree in file, using the first".to_owned(),
    ];
    assert_eq!(expected, inspection.warnings);
    assert_eq!(None, inspection.error);
  }

  #[test]
  fn inspect_tree_reports_a_parse_error_with_line_and_column() {
    // The ';' at line 3 column 1 ends the text where a ',' or ')' is expected.
    let expected = TreeInspection {
      label: "t".into(),
      leaves: 0,
      internal_nodes: 0,
      polytomies: 0,
      branch_lengths: BranchLengths::None,
      warnings: vec![],
      error: Some(TreeError {
        message: "expected ',' or ')'".into(),
        line: Some(3),
        column: Some(1),
      }),
    };
    assert_eq!(expected, inspect_tree("t", "(A,\n(B,C)D\n;"));
  }

  #[test]
  fn inspect_tree_keeps_the_several_trees_warning_with_a_parse_error() {
    let inspection = inspect_tree("t", "(A,A);\n(A,B);\n");
    let expected = (
      vec!["more than one tree in file, using the first".to_owned()],
      Some("duplicate leaf name \"A\"".to_owned()),
    );
    assert_eq!(expected, (inspection.warnings, inspection.error.map(|e| e.message)));
  }

  #[test]
  fn inspect_tree_column_counts_characters() {
    // "é" is two bytes; the error is at the fifth character, the missing ')' before ';'.
    let error = inspect_tree("t", "(é,B;").error.unwrap();
    assert_eq!((Some(1), Some(5)), (error.line, error.column));
  }

  #[test]
  fn inspect_tree_reports_an_error_without_position() {
    let expected = Some(TreeError {
      message: "duplicate leaf name \"A\"".into(),
      line: None,
      column: None,
    });
    assert_eq!(expected, inspect_tree("t", "(A,A);").error);
  }

  #[test]
  fn overlap_counts_missing_and_shared_leaves_and_blocks_a_pair() {
    // a and b share A, B, C; c shares only A with each of them.
    let trees = [
      TreeText::new("a", "((A,B),(C,D));"),
      TreeText::new("b", "((A,B),C);"),
      TreeText::new("c", "(A,(E,F));"),
    ];
    let expected = Overlap {
      total_leaves: 6,
      trees: vec![
        TreeOverlap {
          index: 0,
          label: "a".into(),
          leaves: 4,
          missing: 2,
        },
        TreeOverlap {
          index: 1,
          label: "b".into(),
          leaves: 3,
          missing: 3,
        },
        TreeOverlap {
          index: 2,
          label: "c".into(),
          leaves: 3,
          missing: 3,
        },
      ],
      pairs: vec![
        PairOverlap {
          i: 0,
          j: 1,
          shared: 3,
          blocked: false,
        },
        PairOverlap {
          i: 0,
          j: 2,
          shared: 1,
          blocked: true,
        },
        PairOverlap {
          i: 1,
          j: 2,
          shared: 1,
          blocked: true,
        },
      ],
      failed: vec![],
    };
    assert_eq!(expected, overlap(&trees));
  }

  #[test]
  fn overlap_leaves_out_trees_that_do_not_parse() {
    let trees = [
      TreeText::new("a", "((A,B),C);"),
      TreeText::new("broken", "((A,B),C"),
      TreeText::new("c", "((A,C),(B,D));"),
    ];
    let expected = Overlap {
      total_leaves: 4,
      trees: vec![
        TreeOverlap {
          index: 0,
          label: "a".into(),
          leaves: 3,
          missing: 1,
        },
        TreeOverlap {
          index: 2,
          label: "c".into(),
          leaves: 4,
          missing: 0,
        },
      ],
      pairs: vec![PairOverlap {
        i: 0,
        j: 2,
        shared: 3,
        blocked: false,
      }],
      failed: vec![1],
    };
    assert_eq!(expected, overlap(&trees));
  }

  #[test]
  fn overlap_of_no_trees_is_empty() {
    let expected = Overlap {
      total_leaves: 0,
      trees: vec![],
      pairs: vec![],
      failed: vec![],
    };
    assert_eq!(expected, overlap(&[]));
  }

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
