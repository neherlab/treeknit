//! The analysis request of the command line and the web app: the trees, the settings, their
//! validation, and the conversion of the settings to core `Options`.
//!
//! Both surfaces validate with the same functions, so a request that one accepts the other
//! accepts too. The checks reject the inputs known to make the core panic, because a panic
//! traps the WebAssembly instance.

mod labels;
mod settings;

pub use labels::{label_key, tree_labels};
pub use settings::{MAX_SEED, ResolveMode, SettingKey, Settings, options};

use crate::figure::FigureOptionKey;
use crate::newick;
use crate::output::{self, OutputOptions};
use crate::wire::wire_name;
pub(crate) use labels::pair_stem;
use labels::{check_labels, check_pair_stems};
use serde::{Deserialize, Serialize};
#[cfg(test)]
pub(crate) use settings::check_settings;
pub(crate) use settings::{MAX_MCMC_IT, MAX_ROUNDS};
use treeknit_core::{Options, Taxa, Tree};
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// Fewest leaves a pair of trees must share. With fewer, the pair has no MCCs to infer.
pub(crate) const MIN_SHARED_LEAVES: usize = 2;

/// Least number of trees of a run, because TreeKnit infers the MCCs of pairs of trees.
pub const MIN_TREES: usize = 2;

/// Number of trees of a run that builds an ARG: the ARG joins the two segments of one pair.
pub const ARG_TREE_COUNT: usize = 2;

/// Check the trees and the settings of `request` for a run of the web app: every error of
/// `prepare`. An empty list means the request runs.
pub fn validate(request: &AnalysisRequest) -> Vec<ValidationError> {
  let k = request.trees.len();
  let opts = options(&request.settings, k, false);
  prepare(&request.trees, &OutputOptions::web(k), opts)
    .err()
    .unwrap_or_default()
}

/// The parsed trees and the options of a run that writes the files of `output`, or every error:
/// those of `parse_trees`, then those of `output::check_output_paths` when the trees parse, then
/// those of the options `opts` (see `options`). The command line, `validate`, and the session of
/// the web app check a run with this function, so they report the same errors in one order.
pub fn prepare(
  trees: &[TreeText],
  output: &OutputOptions,
  opts: Result<Options, Vec<ValidationError>>,
) -> Result<(ParsedTrees, Options), Vec<ValidationError>> {
  let parsed = parse_trees(trees).and_then(|p| {
    let errors = output::check_output_paths(&labels(trees), output);
    if errors.is_empty() { Ok(p) } else { Err(errors) }
  });
  match (parsed, opts) {
    (Ok(p), Ok(o)) => Ok((p, o)),
    (parsed, opts) => {
      let mut errors = parsed.err().unwrap_or_default();
      errors.extend(opts.err().unwrap_or_default());
      Err(errors)
    },
  }
}

/// The labels of `trees`, in their order.
pub(crate) fn labels(trees: &[TreeText]) -> Vec<String> {
  trees.iter().map(|t| t.label.clone()).collect()
}

/// Parse the trees of a request after checking their number, their labels, their Newick text,
/// the output file names of their pairs, and the leaves each pair shares, or return every error
/// those checks find.
pub fn parse_trees(trees: &[TreeText]) -> Result<ParsedTrees, Vec<ValidationError>> {
  let mut errors = Vec::new();
  if trees.len() < MIN_TREES {
    errors.push(ValidationError::at(Field::Trees, "need at least two trees"));
  }
  let label_errors = check_labels(trees);
  // A pair with an invalid or repeated label is left out of the pair-name check: its label
  // error already covers it, and two equal labels would repeat it as equal pair names.
  let valid: Vec<bool> = label_errors.iter().map(Option::is_none).collect();
  errors.extend(label_errors.into_iter().flatten());
  errors.extend(check_pair_stems(trees, &valid));
  let mut parsed = Vec::with_capacity(trees.len());
  for (i, t) in trees.iter().enumerate() {
    match newick::parse_first(&t.newick, &t.label) {
      Ok(p) => {
        p.log_warnings();
        parsed.push(p.tree);
      },
      Err(e) => {
        for w in &e.warnings {
          w.log(&t.label);
        }
        errors.push(parse_error(i, t, &e));
      },
    }
  }
  if parsed.len() < trees.len() {
    return Err(errors);
  }
  let ParsedTrees { trees: parsed, taxa } = number_leaves(parsed);
  errors.extend(
    shared_leaf_counts(&parsed, taxa.len())
      .into_iter()
      .filter(|p| p.shared < MIN_SHARED_LEAVES)
      .map(|p| {
        ValidationError::at(
          Field::Trees,
          format!(
            "trees {:?} and {:?} share fewer than {MIN_SHARED_LEAVES} leaves",
            trees[p.i].label, trees[p.j].label
          ),
        )
      }),
  );
  if errors.is_empty() {
    Ok(ParsedTrees { trees: parsed, taxa })
  } else {
    Err(errors)
  }
}

/// Number the leaves of `trees` with one taxon table built from all their leaves.
pub fn number_leaves(mut trees: Vec<Tree>) -> ParsedTrees {
  let taxa = Taxa::from_trees(&trees);
  for t in &mut trees {
    #[expect(
      clippy::expect_used,
      reason = "the table is built from these trees, so it holds every leaf"
    )]
    t.assign_taxa(&taxa)
      .expect("the taxon table built from the trees holds every leaf of every tree");
  }
  ParsedTrees { trees, taxa }
}

/// The request of a session file (`treeknit_session.json`), or the error of its JSON structure:
/// wrong types, missing required or unknown fields, and a seed above [`MAX_SEED`], which no
/// JavaScript number holds exactly. The other rules of [`validate`] are not applied, so a session
/// file with a broken tree or an out-of-range setting still loads and can be fixed.
pub fn read_session(text: &str) -> Result<AnalysisRequest, Vec<ValidationError>> {
  let request: AnalysisRequest =
    serde_json::from_str(text).map_err(|e| vec![ValidationError::new(format!("not a TreeKnit session file: {e}"))])?;
  let seed = request.settings.seed;
  if seed > MAX_SEED {
    return Err(vec![ValidationError::at(
      Field::Setting { key: SettingKey::Seed },
      format!(
        "the seed {seed} of the session file is above {MAX_SEED}, the largest integer a JavaScript number holds exactly"
      ),
    )]);
  }
  Ok(request)
}

/// Shared leaf count of every pair of `trees`, in the pair order of `treeknit_core::run`:
/// `(0,1), (0,2), ..., (1,2), ...`. The trees carry taxa from a table of `n_taxa` leaves.
pub(crate) fn shared_leaf_counts(trees: &[Tree], n_taxa: usize) -> Vec<PairShared> {
  let sets: Vec<_> = trees.iter().map(|t| t.leaf_set(n_taxa)).collect();
  let mut counts = Vec::new();
  for i in 0..sets.len() {
    for j in i + 1..sets.len() {
      counts.push(PairShared {
        i,
        j,
        shared: sets[i].intersection_count(&sets[j]),
      });
    }
  }
  counts
}

fn parse_error(i: usize, t: &TreeText, e: &newick::ParseError) -> ValidationError {
  ValidationError {
    field: Some(Field::TreeNewick { index: i }),
    message: format!("tree {:?}: {e}", t.label),
    line: e.location.map(|l| l.line),
    column: e.location.map(|l| l.column),
  }
}

/// Trees to compare, and the settings of the analysis.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnalysisRequest {
  pub trees: Vec<TreeText>,
  #[serde(default)]
  pub settings: Settings,
}

/// A labeled tree in Newick format.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(deny_unknown_fields)]
pub struct TreeText {
  pub(crate) label: String,
  pub(crate) newick: String,
}

impl TreeText {
  /// The tree `newick` labeled `label`.
  pub fn new(label: impl Into<String>, newick: impl Into<String>) -> Self {
    TreeText {
      label: label.into(),
      newick: newick.into(),
    }
  }
}

/// A problem with a request, at the field it concerns.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, thiserror::Error)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
#[error("{message}")]
pub struct ValidationError {
  /// The field the error concerns; `None` for the request as a whole.
  pub field: Option<Field>,
  pub message: String,
  /// 1-based line of a parse error in the Newick text.
  pub line: Option<usize>,
  /// 1-based column of a parse error, in Unicode characters.
  pub column: Option<usize>,
}

impl ValidationError {
  /// An error of the request as a whole.
  pub fn new(message: impl Into<String>) -> Self {
    ValidationError {
      field: None,
      message: message.into(),
      line: None,
      column: None,
    }
  }

  /// An error at the field `field`.
  pub fn at(field: Field, message: impl Into<String>) -> Self {
    ValidationError {
      field: Some(field),
      ..ValidationError::new(message)
    }
  }
}

/// The input field that a [`ValidationError`] concerns.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Field {
  /// The list of trees as a whole, such as the number of trees or a pair of trees.
  Trees,
  /// Tree `index` as a whole, such as a tree file that cannot be read.
  Tree { index: usize },
  /// The label of tree `index`.
  TreeLabel { index: usize },
  /// The Newick text of tree `index`.
  TreeNewick { index: usize },
  /// A setting.
  Setting { key: SettingKey },
  /// Sequence length `index` of `settings.seqLengths`.
  SeqLength { index: usize },
  /// An option of a figure.
  FigureOption { key: FigureOptionKey },
  /// The key `key` of a link, as the link writes it.
  LinkKey { key: String },
  /// The `tree` value `index` of a link.
  LinkTree { index: usize },
}

impl Field {
  /// The path of the field as the command line names it, such as `trees[0].newick`,
  /// `settings.seqLengths[1]`, `rowHeight`, or the key of a link.
  pub fn path(&self) -> String {
    match self {
      Field::Trees => "trees".to_owned(),
      Field::Tree { index } => format!("trees[{index}]"),
      Field::TreeLabel { index } => format!("trees[{index}].label"),
      Field::TreeNewick { index } => format!("trees[{index}].newick"),
      Field::Setting { key } => format!("settings.{}", wire_name(key)),
      Field::SeqLength { index } => format!("settings.{}[{index}]", wire_name(&SettingKey::SeqLengths)),
      Field::FigureOption { key } => wire_name(key),
      Field::LinkKey { key } => key.clone(),
      Field::LinkTree { index } => format!("tree[{index}]"),
    }
  }

  /// The index of the tree of the request that the field belongs to; `None` for a field that
  /// belongs to no tree of the request.
  pub fn tree_index(&self) -> Option<usize> {
    match self {
      Field::Tree { index } | Field::TreeLabel { index } | Field::TreeNewick { index } => Some(*index),
      Field::Trees
      | Field::Setting { .. }
      | Field::SeqLength { .. }
      | Field::FigureOption { .. }
      | Field::LinkKey { .. }
      | Field::LinkTree { .. } => None,
    }
  }
}

/// The parsed trees of a request, with their leaves numbered by one taxon table.
#[derive(Clone, Debug)]
pub struct ParsedTrees {
  pub trees: Vec<Tree>,
  pub taxa: Taxa,
}

/// Shared leaf count of the trees `i` and `j`, with `i < j`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PairShared {
  pub(crate) i: usize,
  pub(crate) j: usize,
  pub(crate) shared: usize,
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::test_support::{clades, texts};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::json;
  use std::collections::BTreeSet;

  const T: &str = "((A,B),(C,D));";

  /// The validation errors of `trees`, none when they parse.
  pub(crate) fn tree_errors(trees: &[TreeText]) -> Vec<ValidationError> {
    match parse_trees(trees) {
      Ok(_) => Vec::new(),
      Err(errors) => errors,
    }
  }

  pub(crate) fn labeled(labels: &[&str]) -> Vec<TreeText> {
    texts(&labels.iter().map(|&l| (l, T)).collect::<Vec<_>>())
  }

  pub(crate) fn label(index: usize) -> Field {
    Field::TreeLabel { index }
  }

  pub(crate) fn setting(key: SettingKey) -> Field {
    Field::Setting { key }
  }

  #[test]
  fn request_without_settings_takes_the_defaults() {
    let r: AnalysisRequest = serde_json::from_value(json!({"trees": [{"label": "ha", "newick": T}]})).unwrap();
    assert_eq!(Settings::default(), r.settings);
  }

  #[test]
  fn request_round_trips_through_json() {
    let r = AnalysisRequest {
      trees: labeled(&["ha", "na"]),
      settings: Settings {
        seq_lengths: Some(vec![1700.0, 1400.0]),
        resolve: ResolveMode::Strict,
        seed: MAX_SEED,
        ..Settings::default()
      },
    };
    let text = serde_json::to_string(&r).unwrap();
    assert_eq!(r, serde_json::from_str::<AnalysisRequest>(&text).unwrap());
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::trees(        Field::Trees,                                                 ("trees",                  json!({"kind": "trees"})))]
  #[case::tree(         Field::Tree { index: 2 },                                     ("trees[2]",               json!({"kind": "tree", "index": 2})))]
  #[case::tree_label(   Field::TreeLabel { index: 0 },                                ("trees[0].label",         json!({"kind": "treeLabel", "index": 0})))]
  #[case::tree_newick(  Field::TreeNewick { index: 1 },                               ("trees[1].newick",        json!({"kind": "treeNewick", "index": 1})))]
  #[case::setting(      Field::Setting { key: SettingKey::NMcmcIt },                  ("settings.nMcmcIt",       json!({"kind": "setting", "key": "nMcmcIt"})))]
  #[case::seq_lengths(  Field::Setting { key: SettingKey::SeqLengths },               ("settings.seqLengths",    json!({"kind": "setting", "key": "seqLengths"})))]
  #[case::seq_length(   Field::SeqLength { index: 1 },                                ("settings.seqLengths[1]", json!({"kind": "seqLength", "index": 1})))]
  #[case::figure_option(Field::FigureOption { key: FigureOptionKey::RowHeight },      ("rowHeight",              json!({"kind": "figureOption", "key": "rowHeight"})))]
  #[case::link_key(     Field::LinkKey { key: "seq-lengths".to_owned() },             ("seq-lengths",            json!({"kind": "linkKey", "key": "seq-lengths"})))]
  #[case::link_tree(    Field::LinkTree { index: 3 },                                 ("tree[3]",                json!({"kind": "linkTree", "index": 3})))]
  #[trace]
  fn fields_have_a_command_line_path_and_a_kind(#[case] field: Field, #[case] expected: (&str, serde_json::Value)) {
    let actual = (field.path(), serde_json::to_value(&field).unwrap());
    assert_eq!((expected.0.to_owned(), expected.1), actual);
  }

  #[test]
  fn validation_error_serializes_camel_case_fields() {
    let e = ValidationError {
      field: Some(Field::TreeNewick { index: 0 }),
      message: "m".to_owned(),
      line: Some(2),
      column: Some(5),
    };
    assert_eq!(
      json!({"field": {"kind": "treeNewick", "index": 0}, "message": "m", "line": 2, "column": 5}),
      serde_json::to_value(&e).unwrap()
    );
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::matched(ResolveMode::Matched, clades("((A,B),(C,D));"))]
  #[case::none(   ResolveMode::None, BTreeSet::new())]
  #[trace]
  fn resolve_mode_controls_resolution(#[case] resolve: ResolveMode, #[case] expected: BTreeSet<BTreeSet<String>>) {
    // Matched resolution copies na's splits into ha's polytomy.
    let s = Settings {
      resolve,
      ..Settings::default()
    };
    let ParsedTrees { mut trees, taxa } = parse_trees(&texts(&[("ha", "(A,B,C,D);"), ("na", T)])).unwrap();
    treeknit_core::run(&mut trees, &taxa, &options(&s, 2, false).unwrap(), s.seed);
    assert_eq!(expected, clades(&newick::write(&trees[0]).unwrap()));
  }

  #[test]
  fn valid_request_has_no_errors() {
    let r = AnalysisRequest {
      trees: texts(&[("ha", "((A,B),(C,(D,X)));"), ("na", "((A,(B,X)),(C,D));")]),
      settings: Settings::default(),
    };
    assert_eq!(Vec::<ValidationError>::new(), validate(&r));
  }

  #[test]
  fn validate_reports_tree_and_settings_errors() {
    let r = AnalysisRequest {
      trees: labeled(&["ha"]),
      settings: Settings {
        rounds: 0,
        ..Settings::default()
      },
    };
    let expected = vec![
      ValidationError::at(Field::Trees, "need at least two trees"),
      ValidationError::at(setting(SettingKey::Rounds), "rounds must be at least 1"),
    ];
    assert_eq!(expected, validate(&r));
  }

  #[test]
  fn parse_error_has_field_line_and_column() {
    let errors = tree_errors(&texts(&[("ha", T), ("na", "((A,B),\n(C,D)x y);")]));
    let expected = vec![ValidationError {
      field: Some(Field::TreeNewick { index: 1 }),
      message: "tree \"na\": Newick parse error: expected comment, ')', ',', ':', ';' at byte 15".to_owned(),
      line: Some(2),
      column: Some(8),
    }];
    assert_eq!(expected, errors);
  }

  #[test]
  fn parse_error_column_counts_characters() {
    let errors = tree_errors(&texts(&[("ha", T), ("na", "((Ä,B),(C,D)x y);")]));
    assert_eq!((Some(1), Some(15)), (errors[0].line, errors[0].column));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::duplicate_leaf("((A,A),(C,D));", "tree \"na\": Newick parse error: duplicate leaf name \"A\"")]
  #[trace]
  fn parse_error_without_position(#[case] newick: &str, #[case] message: &str) {
    let errors = tree_errors(&texts(&[("ha", T), ("na", newick)]));
    let expected = vec![ValidationError::at(Field::TreeNewick { index: 1 }, message)];
    assert_eq!(expected, errors);
  }

  #[test]
  fn empty_newick_text_is_a_parse_error_of_its_field() {
    // The command line gives a file that it cannot read an empty text and reports the read error
    // instead of the error of this field.
    let errors = tree_errors(&texts(&[("ha", T), ("na", "")]));
    let fields: Vec<Option<Field>> = errors.iter().map(|e| e.field.clone()).collect();
    assert_eq!(vec![Some(Field::TreeNewick { index: 1 })], fields);
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::disjoint(  "((P,Q),R);")]
  #[case::one_shared("((A,Q),R);")]
  #[trace]
  fn pairs_sharing_fewer_than_two_leaves_are_rejected(#[case] newick: &str) {
    let errors = tree_errors(&texts(&[("ha", T), ("na", T), ("pb2", newick)]));
    let expected = vec![
      ValidationError::at(Field::Trees, "trees \"ha\" and \"pb2\" share fewer than 2 leaves"),
      ValidationError::at(Field::Trees, "trees \"na\" and \"pb2\" share fewer than 2 leaves"),
    ];
    assert_eq!(expected, errors);
  }

  #[test]
  fn pairs_sharing_two_leaves_are_accepted() {
    assert_eq!(
      Vec::<ValidationError>::new(),
      tree_errors(&texts(&[("ha", T), ("na", "((A,B),(P,Q));")]))
    );
  }

  #[test]
  fn all_errors_are_reported_together() {
    let errors = tree_errors(&texts(&[("a/b", T), ("na", "(A,B"), ("", T)]));
    let fields: Vec<Option<Field>> = errors.iter().map(|e| e.field.clone()).collect();
    assert_eq!(
      vec![Some(label(0)), Some(label(2)), Some(Field::TreeNewick { index: 1 })],
      fields
    );
  }

  #[test]
  fn parse_trees_numbers_leaves_with_one_taxon_table() {
    let p = parse_trees(&texts(&[("ha", "((A,B),(C,D));"), ("na", "((A,B),(C,P));")])).unwrap();
    assert_eq!(5, p.taxa.len());
    assert_eq!(
      vec!["ha", "na"],
      p.trees.iter().map(|t| t.label.as_str()).collect::<Vec<_>>()
    );
  }

  #[test]
  fn read_session_loads_counts_beyond_32_bits_and_validation_reports_them() {
    // Oracle: 2^32 is one above the largest usize of wasm32, so it must load as a number and fail
    // only the range check, on every target.
    let text = r#"{"trees": [], "settings": {"nMcmcIt": 4294967296, "rounds": 4294967296}}"#;
    let request = read_session(text).unwrap();
    let expected = vec![
      ValidationError::at(
        setting(SettingKey::Rounds),
        "rounds must be at most 4294967294, got 4294967296",
      ),
      ValidationError::at(
        Field::Setting {
          key: SettingKey::NMcmcIt,
        },
        "MCMC steps per leaf must be at most 4294967295, got 4294967296",
      ),
    ];
    assert_eq!(expected, check_settings(&request.settings, 2));
  }

  #[test]
  fn shared_leaf_counts_follow_the_pipeline_pair_order() {
    let p = parse_trees(&texts(&[
      ("a", "((A,B),(C,D));"),
      ("b", "((A,B),C);"),
      ("c", "((A,D),(C,P));"),
    ]))
    .unwrap();
    let expected = vec![
      PairShared { i: 0, j: 1, shared: 3 },
      PairShared { i: 0, j: 2, shared: 3 },
      PairShared { i: 1, j: 2, shared: 2 },
    ];
    assert_eq!(expected, shared_leaf_counts(&p.trees, p.taxa.len()));
  }

  #[test]
  fn read_session_loads_a_request_that_fails_validation() {
    // Structure only: the broken tree and the negative gamma stay for the user to fix.
    let text = r#"{"trees": [{"label": "ha", "newick": "((A,B"}], "settings": {"gamma": -1}}"#;
    let expected = AnalysisRequest {
      trees: texts(&[("ha", "((A,B")]),
      settings: Settings {
        gamma: -1.0,
        ..Settings::default()
      },
    };
    assert_eq!(Ok(expected), read_session(text));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::not_json(     "ha.nwk",                                          "not a TreeKnit session file: expected value at line 1 column 1")]
  #[case::no_trees(     r#"{"settings": {}}"#,                             "not a TreeKnit session file: missing field `trees` at line 1 column 16")]
  #[case::unknown_field(r#"{"trees": [], "tree": []}"#,                    "not a TreeKnit session file: unknown field `tree`, expected `trees` or `settings` at line 1 column 20")]
  #[case::wrong_type(   r#"{"trees": [{"label": 1, "newick": "(A,B);"}]}"#, "not a TreeKnit session file: invalid type: integer `1`, expected a string at line 1 column 22")]
  #[trace]
  fn read_session_rejects_a_wrong_structure(#[case] text: &str, #[case] message: &str) {
    let expected = vec![ValidationError::new(message)];
    assert_eq!(Err(expected), read_session(text));
  }

  #[test]
  fn read_session_rejects_a_seed_above_the_javascript_limit() {
    let text = r#"{"trees": [], "settings": {"seed": 9007199254740992}}"#;
    let expected = vec![ValidationError::at(
      Field::Setting { key: SettingKey::Seed },
      "the seed 9007199254740992 of the session file is above 9007199254740991, the largest integer a JavaScript number holds exactly",
    )];
    assert_eq!(Err(expected), read_session(text));
  }

  #[test]
  fn read_session_accepts_the_largest_seed() {
    let text = r#"{"trees": [], "settings": {"seed": 9007199254740991}}"#;
    assert_eq!(Ok(MAX_SEED), read_session(text).map(|r| r.settings.seed));
  }
}
