//! The analysis request of the command line and the web app: the trees, the settings, their
//! validation, and the conversion of the settings to core `Options`.
//!
//! Both surfaces validate with the same functions, so a request that one accepts the other
//! accepts too. The checks reject the inputs known to make the core panic, because a panic
//! traps the WebAssembly instance.

use crate::newick;
use serde::{Deserialize, Serialize};
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use treeknit_core::{Options, Resolution, Taxa, Tree, bits};
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// Largest seed: JavaScript numbers hold integers exactly only up to 2^53 - 1, and a session
/// file must pass through the web app unchanged.
pub const MAX_SEED: u64 = (1 << 53) - 1;

/// Fewest leaves a pair of trees must share. With fewer, the pair has no MCCs to infer.
pub const MIN_SHARED_LEAVES: usize = 2;

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
  pub label: String,
  pub newick: String,
}

/// Settings of the command line; a missing field takes the command-line default.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
  /// Cost γ of a reassortment, removing an MCC (`--gamma`).
  pub gamma: f64,
  /// Sequence lengths of the segments, in the order of the trees, used by the likelihood
  /// tie-break (`--seq-lengths`).
  pub seq_lengths: Option<Vec<f64>>,
  /// MCMC steps per leaf (`--n-mcmc-it`).
  pub n_mcmc_it: usize,
  /// How trees are resolved (`--resolve`).
  pub resolve: ResolveMode,
  /// Before inference, add to each tree the splits of other trees compatible with all trees
  /// (`--pre-resolve`).
  pub pre_resolve: bool,
  /// Rounds of pair inference (`--rounds`).
  pub rounds: usize,
  /// With strict or liberal resolution and more than two trees, run a final round that
  /// re-infers MCCs without resolution (the opposite of `--no-final-round`).
  pub final_round: bool,
  /// Break ties between configurations with branch lengths (the opposite of
  /// `--no-likelihood`).
  pub likelihood: bool,
  /// Naive MCCs, γ → ∞ (`--naive`).
  pub naive: bool,
  /// Seed of the random number generator (`--seed`), at most 2^53 - 1.
  pub seed: u64,
}

impl Default for Settings {
  fn default() -> Self {
    let o = Options::default();
    Settings {
      gamma: o.gamma,
      seq_lengths: None,
      n_mcmc_it: o.n_mcmc,
      resolve: ResolveMode::default(),
      pre_resolve: o.pre_resolve,
      rounds: o.rounds,
      final_round: o.final_unresolved_round,
      likelihood: o.likelihood_sort,
      naive: o.naive,
      seed: 1,
    }
  }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "lowercase")]
pub enum ResolveMode {
  None,
  Strict,
  Liberal,
  #[default]
  Matched,
}

impl From<ResolveMode> for Resolution {
  fn from(m: ResolveMode) -> Resolution {
    match m {
      ResolveMode::None => Resolution::None,
      ResolveMode::Strict => Resolution::Strict,
      ResolveMode::Liberal => Resolution::Liberal,
      ResolveMode::Matched => Resolution::Matched,
    }
  }
}

/// A problem with a request, at the field it concerns.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ValidationError {
  /// Path of the field in the request, such as `settings.gamma`, `settings.seqLengths[1]`, or
  /// `trees[0].newick`; `None` for the request as a whole.
  pub field: Option<String>,
  pub message: String,
  /// 1-based line of a parse error in the Newick text.
  pub line: Option<usize>,
  /// 1-based column of a parse error, in Unicode characters.
  pub column: Option<usize>,
}

impl ValidationError {
  fn at(field: impl Into<String>, message: impl Into<String>) -> Self {
    ValidationError {
      field: Some(field.into()),
      message: message.into(),
      line: None,
      column: None,
    }
  }
}

impl std::fmt::Display for ValidationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.message)
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
  pub i: usize,
  pub j: usize,
  pub shared: usize,
}

/// Check the trees and the settings of `request`: every error of `check_trees` and of
/// `check_settings`. An empty list means the request runs.
pub fn validate(request: &AnalysisRequest) -> Vec<ValidationError> {
  let mut errors = check_trees(&request.trees);
  errors.extend(check_settings(&request.settings, request.trees.len()));
  errors
}

/// Check the trees of a request: their number, their labels, their Newick text, the output file
/// names of their pairs, and the leaves each pair shares.
pub fn check_trees(trees: &[TreeText]) -> Vec<ValidationError> {
  parse_trees(trees).err().unwrap_or_default()
}

/// Parse the trees of a request after the checks of `check_trees`, or return every error those
/// checks find.
pub fn parse_trees(trees: &[TreeText]) -> Result<ParsedTrees, Vec<ValidationError>> {
  let mut errors = Vec::new();
  if trees.len() < 2 {
    errors.push(ValidationError::at("trees", "need at least two trees"));
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
      Ok(tree) => parsed.push(tree),
      Err(e) => errors.push(parse_error(i, t, &e)),
    }
  }
  if parsed.len() < trees.len() {
    return Err(errors);
  }
  let taxa = Taxa::from_trees(&parsed);
  for t in &mut parsed {
    #[expect(clippy::expect_used, reason = "the table is built from these trees, so it holds every leaf")]
    t.assign_taxa(&taxa)
      .expect("the taxon table built from the trees holds every leaf of every tree");
  }
  errors.extend(
    shared_leaf_counts(&parsed, taxa.len())
      .into_iter()
      .filter(|p| p.shared < MIN_SHARED_LEAVES)
      .map(|p| {
        ValidationError::at(
          "trees",
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

/// Check the settings of a request with `k` trees.
pub fn check_settings(s: &Settings, k: usize) -> Vec<ValidationError> {
  let mut errors = Vec::new();
  if !(s.gamma.is_finite() && s.gamma >= 0.0) {
    errors.push(ValidationError::at(
      "settings.gamma",
      format!("gamma must be a non-negative number, got {}", s.gamma),
    ));
  }
  if let Some(v) = &s.seq_lengths {
    if v.len() != k {
      errors.push(ValidationError::at(
        "settings.seqLengths",
        format!("got {} sequence lengths for {k} trees", v.len()),
      ));
    }
    for (i, x) in v.iter().enumerate() {
      if !(x.is_finite() && *x > 0.0) {
        errors.push(ValidationError::at(
          format!("settings.seqLengths[{i}]"),
          format!("sequence length {} must be a positive number, got {x}", i + 1),
        ));
      }
    }
  }
  if s.rounds == 0 {
    errors.push(ValidationError::at("settings.rounds", "rounds must be at least 1"));
  } else if s.rounds.checked_add(1).is_none() {
    // The core counts the final round without resolution on top of `rounds`.
    errors.push(ValidationError::at(
      "settings.rounds",
      format!("rounds must be less than {}, got {}", usize::MAX, s.rounds),
    ));
  }
  if s.n_mcmc_it == 0 {
    errors.push(ValidationError::at(
      "settings.nMcmcIt",
      "MCMC steps per leaf must be at least 1",
    ));
  }
  if s.seed > MAX_SEED {
    errors.push(ValidationError::at(
      "settings.seed",
      format!("seed must be at most {MAX_SEED}, got {}", s.seed),
    ));
  }
  errors
}

/// Core options of the settings for `k` trees. The settings must pass `check_settings`; the
/// core panics on settings that fail it.
///
/// `Options::parallel` keeps the core default: each surface decides about threads.
pub fn options(s: &Settings, k: usize) -> Options {
  let mut o = Options::for_trees(k);
  if let Some(v) = &s.seq_lengths {
    o.seq_lengths.clone_from(v);
  }
  o.gamma = s.gamma;
  o.n_mcmc = s.n_mcmc_it;
  o.resolution = s.resolve.into();
  o.pre_resolve = s.pre_resolve;
  o.rounds = s.rounds;
  o.final_unresolved_round = s.final_round;
  o.likelihood_sort = s.likelihood;
  o.naive = s.naive;
  o
}

/// File-name stem of the pair of trees labeled `a` and `b`, as in `MCCs_<a>_<b>.dat`.
pub fn pair_stem(a: &str, b: &str) -> String {
  format!("{a}_{b}")
}

/// Shared leaf count of every pair of `trees`, in the pair order of `treeknit_core::run`:
/// `(0,1), (0,2), ..., (1,2), ...`. The trees carry taxa from a table of `n_taxa` leaves.
pub fn shared_leaf_counts(trees: &[Tree], n_taxa: usize) -> Vec<PairShared> {
  let sets: Vec<_> = trees.iter().map(|t| t.leaf_set(n_taxa)).collect();
  let mut counts = Vec::new();
  for i in 0..sets.len() {
    for j in i + 1..sets.len() {
      counts.push(PairShared {
        i,
        j,
        shared: bits::and(&sets[i], &sets[j]).count_ones(..),
      });
    }
  }
  counts
}

/// Characters that Windows does not allow in file names, besides the path separators.
const RESERVED_CHARS: [char; 7] = ['<', '>', ':', '"', '|', '?', '*'];

/// A label is also a file-name stem of the outputs, so it must name a file in the results
/// directory on every release target: no path separator, no character reserved on Windows, no
/// control character, and neither `.` nor `..`. Labels that differ only in case are repeated
/// labels, because the file systems of macOS and Windows ignore case.
///
/// Return the error of each label, in the order of `trees`.
fn check_labels(trees: &[TreeText]) -> Vec<Option<ValidationError>> {
  let mut errors = Vec::new();
  let mut seen: BTreeMap<String, &str> = BTreeMap::new();
  for (i, t) in trees.iter().enumerate() {
    let field = format!("trees[{i}].label");
    let l = t.label.as_str();
    let message = if l.trim().is_empty() {
      Some(format!("tree {} needs a label", i + 1))
    } else if l.contains(['/', '\\']) {
      Some(format!("tree label {l:?} must not contain / or \\"))
    } else if l.contains(RESERVED_CHARS) {
      Some(format!("tree label {l:?} must not contain any of <>:\"|?*"))
    } else if l.chars().any(char::is_control) {
      Some(format!("tree label {l:?} must not contain control characters"))
    } else if l == "." || l == ".." {
      Some(format!("tree label {l:?} is not a file name"))
    } else {
      match seen.entry(l.to_lowercase()) {
        Entry::Vacant(e) => {
          e.insert(l);
          None
        },
        Entry::Occupied(e) if *e.get() == l => Some(format!("tree label {l:?} is used twice")),
        Entry::Occupied(e) => Some(format!(
          "tree label {l:?} differs from {:?} only in case, so their output files get the same name",
          e.get()
        )),
      }
    };
    errors.push(message.map(|m| ValidationError::at(field, m)));
  }
  errors
}

/// Pairs whose output files would get the same name, ignoring case as `check_labels` does:
/// `MCCs_<a>_<b>.dat` joins two labels with `_`, so the labels `a_b`, `c`, `a`, `b_c` give the
/// pairs (0,1) and (2,3) one name. Only pairs of two `valid` labels are checked.
fn check_pair_stems(trees: &[TreeText], valid: &[bool]) -> Vec<ValidationError> {
  let mut first: BTreeMap<String, (usize, usize)> = BTreeMap::new();
  let mut errors = Vec::new();
  for i in 0..trees.len() {
    for j in i + 1..trees.len() {
      if !(valid[i] && valid[j]) {
        continue;
      }
      let stem = pair_stem(&trees[i].label, &trees[j].label);
      match first.entry(stem.to_lowercase()) {
        Entry::Occupied(e) => {
          let &(a, b) = e.get();
          errors.push(ValidationError::at(
            "trees",
            format!(
              "tree pairs ({:?}, {:?}) and ({:?}, {:?}) give the same output file names ({stem:?}); rename a tree",
              trees[a].label, trees[b].label, trees[i].label, trees[j].label
            ),
          ));
        },
        Entry::Vacant(e) => {
          e.insert((i, j));
        },
      }
    }
  }
  errors
}

fn parse_error(i: usize, t: &TreeText, e: &newick::ParseError) -> ValidationError {
  let position = e.offset.map(|o| newick::line_column(&t.newick, o));
  ValidationError {
    field: Some(format!("trees[{i}].newick")),
    message: format!("tree {:?}: {e}", t.label),
    line: position.map(|(l, _)| l),
    column: position.map(|(_, c)| c),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::json;
  use std::collections::BTreeSet;

  const T: &str = "((A,B),(C,D));";

  fn trees(trees: &[(&str, &str)]) -> Vec<TreeText> {
    trees
      .iter()
      .map(|(label, newick)| TreeText {
        label: (*label).to_owned(),
        newick: (*newick).to_owned(),
      })
      .collect()
  }

  fn labeled(labels: &[&str]) -> Vec<TreeText> {
    trees(&labels.iter().map(|&l| (l, T)).collect::<Vec<_>>())
  }

  fn error(field: &str, message: &str) -> ValidationError {
    ValidationError::at(field, message)
  }

  fn clades(newick: &str) -> BTreeSet<BTreeSet<String>> {
    let t = newick::parse(newick, "t").unwrap();
    t.internals()
      .into_iter()
      .filter(|&n| n != t.root)
      .map(|n| t.leaves_below(n).into_iter().map(|l| t.name(l).to_owned()).collect())
      .collect()
  }

  #[test]
  fn settings_default_to_command_line_defaults() {
    let expected = Settings {
      gamma: 2.0,
      seq_lengths: None,
      n_mcmc_it: 50,
      resolve: ResolveMode::Matched,
      pre_resolve: false,
      rounds: 1,
      final_round: true,
      likelihood: true,
      naive: false,
      seed: 1,
    };
    assert_eq!(expected, serde_json::from_value::<Settings>(json!({})).unwrap());
    assert_eq!(expected, Settings::default());
  }

  #[test]
  fn settings_read_camel_case_fields() {
    let s: Settings = serde_json::from_value(json!({
        "gamma": 3.5, "seqLengths": [1700.0, 1400.0], "nMcmcIt": 10, "resolve": "liberal",
        "preResolve": true, "rounds": 2, "finalRound": false, "likelihood": false,
        "naive": true, "seed": 7,
    }))
    .unwrap();
    let expected = Settings {
      gamma: 3.5,
      seq_lengths: Some(vec![1700.0, 1400.0]),
      n_mcmc_it: 10,
      resolve: ResolveMode::Liberal,
      pre_resolve: true,
      rounds: 2,
      final_round: false,
      likelihood: false,
      naive: true,
      seed: 7,
    };
    assert_eq!(expected, s);
  }

  #[test]
  fn settings_reject_unknown_fields() {
    let e = serde_json::from_value::<Settings>(json!({"gama": 1.0})).unwrap_err();
    assert_eq!(
      "unknown field `gama`, expected one of `gamma`, `seqLengths`, `nMcmcIt`, `resolve`, \
       `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`, `seed`",
      e.to_string()
    );
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

  #[test]
  fn validation_error_serializes_camel_case_fields() {
    let e = ValidationError {
      field: Some("trees[0].newick".to_owned()),
      message: "m".to_owned(),
      line: Some(2),
      column: Some(5),
    };
    assert_eq!(
      json!({"field": "trees[0].newick", "message": "m", "line": 2, "column": 5}),
      serde_json::to_value(&e).unwrap()
    );
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::none(   ResolveMode::None,    Resolution::None)]
  #[case::strict( ResolveMode::Strict,  Resolution::Strict)]
  #[case::liberal(ResolveMode::Liberal, Resolution::Liberal)]
  #[case::matched(ResolveMode::Matched, Resolution::Matched)]
  #[trace]
  fn options_take_the_resolution_mode(#[case] resolve: ResolveMode, #[case] expected: Resolution) {
    let s = Settings { resolve, ..Settings::default() };
    assert_eq!(expected, options(&s, 2).resolution);
  }

  #[test]
  #[expect(clippy::float_cmp, reason = "the conversion copies the values unchanged")]
  fn options_take_every_setting() {
    let s = Settings {
      gamma: 3.5,
      seq_lengths: Some(vec![1700.0, 1400.0, 900.0]),
      n_mcmc_it: 10,
      resolve: ResolveMode::Liberal,
      pre_resolve: true,
      rounds: 2,
      final_round: false,
      likelihood: false,
      naive: true,
      seed: 7,
    };
    let o = options(&s, 3);
    assert_eq!(3.5, o.gamma);
    assert_eq!(vec![1700.0, 1400.0, 900.0], o.seq_lengths);
    assert_eq!(10, o.n_mcmc);
    assert_eq!(Resolution::Liberal, o.resolution);
    assert!(o.pre_resolve);
    assert_eq!(2, o.rounds);
    assert!(!o.final_unresolved_round);
    assert!(!o.likelihood_sort);
    assert!(o.naive);
  }

  #[test]
  fn options_without_lengths_give_one_length_per_tree() {
    assert_eq!(vec![1.0; 3], options(&Settings::default(), 3).seq_lengths);
  }

  #[test]
  fn options_keep_the_core_thread_default() {
    assert_eq!(Options::default().parallel, options(&Settings::default(), 2).parallel);
  }

  #[rstest]
  #[case::matched(ResolveMode::Matched, clades("((A,B),(C,D));"))]
  #[case::none(ResolveMode::None, BTreeSet::new())]
  #[trace]
  fn resolve_mode_controls_resolution(#[case] resolve: ResolveMode, #[case] expected: BTreeSet<BTreeSet<String>>) {
    // Matched resolution copies na's splits into ha's polytomy.
    let s = Settings {
      resolve,
      ..Settings::default()
    };
    let ParsedTrees { mut trees, taxa } = parse_trees(&trees(&[("ha", "(A,B,C,D);"), ("na", T)])).unwrap();
    treeknit_core::run(&mut trees, &taxa, &options(&s, 2), s.seed);
    assert_eq!(expected, clades(&newick::write(&trees[0])));
  }

  #[test]
  fn valid_request_has_no_errors() {
    let r = AnalysisRequest {
      trees: trees(&[("ha", "((A,B),(C,(D,X)));"), ("na", "((A,(B,X)),(C,D));")]),
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
      error("trees", "need at least two trees"),
      error("settings.rounds", "rounds must be at least 1"),
    ];
    assert_eq!(expected, validate(&r));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::no_tree(        &[],                         "trees",           "need at least two trees")]
  #[case::one_tree(       &["ha"],                     "trees",           "need at least two trees")]
  #[case::empty_label(    &["ha", ""],                 "trees[1].label",  "tree 2 needs a label")]
  #[case::blank_label(    &["ha", " "],                "trees[1].label",  "tree 2 needs a label")]
  #[case::duplicate_label(&["ha", "na", "ha"],         "trees[2].label",  "tree label \"ha\" is used twice")]
  #[case::parent_path(    &["ha", "../x"],             "trees[1].label",  "tree label \"../x\" must not contain / or \\")]
  #[case::slash(          &["a/b", "na"],              "trees[0].label",  "tree label \"a/b\" must not contain / or \\")]
  #[case::backslash(      &["ha", "a\\b"],             "trees[1].label",  "tree label \"a\\\\b\" must not contain / or \\")]
  #[case::control(        &["ha", "a\tb"],             "trees[1].label",  "tree label \"a\\tb\" must not contain control characters")]
  #[case::dot(            &["ha", "."],                "trees[1].label",  "tree label \".\" is not a file name")]
  #[case::dot_dot(        &["..", "na"],               "trees[0].label",  "tree label \"..\" is not a file name")]
  #[case::pair_stems(     &["a_b", "c", "a", "b_c"],   "trees",           "tree pairs (\"a_b\", \"c\") and (\"a\", \"b_c\") give the same output file names (\"a_b_c\"); rename a tree")]
  #[case::case_only(      &["HA", "na", "ha"],         "trees[2].label",  "tree label \"ha\" differs from \"HA\" only in case, so their output files get the same name")]
  #[case::pair_stems_case(&["a_b", "c", "A", "B_c"],   "trees",           "tree pairs (\"a_b\", \"c\") and (\"A\", \"B_c\") give the same output file names (\"A_B_c\"); rename a tree")]
  #[case::less_than(      &["ha", "a<b"],              "trees[1].label",  "tree label \"a<b\" must not contain any of <>:\"|?*")]
  #[case::greater_than(   &["ha", "a>b"],              "trees[1].label",  "tree label \"a>b\" must not contain any of <>:\"|?*")]
  #[case::colon(          &["ha", "a:b"],              "trees[1].label",  "tree label \"a:b\" must not contain any of <>:\"|?*")]
  #[case::quote(          &["ha", "a\"b"],             "trees[1].label",  "tree label \"a\\\"b\" must not contain any of <>:\"|?*")]
  #[case::pipe(           &["ha", "a|b"],              "trees[1].label",  "tree label \"a|b\" must not contain any of <>:\"|?*")]
  #[case::question(       &["ha", "a?b"],              "trees[1].label",  "tree label \"a?b\" must not contain any of <>:\"|?*")]
  #[case::star(           &["ha", "a*b"],              "trees[1].label",  "tree label \"a*b\" must not contain any of <>:\"|?*")]
  #[trace]
  fn invalid_labels_are_rejected(#[case] labels: &[&str], #[case] field: &str, #[case] message: &str) {
    assert_eq!(vec![error(field, message)], check_trees(&labeled(labels)));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::dots_inside(&["ha", "a.b", "..c"])]
  #[case::unicode(    &["A-H3N2 HA", "NA ü"])]
  #[case::underscores(&["a_b", "c", "a"])]
  #[case::case_distinct(&["HA", "ha2"])]
  #[trace]
  fn usable_labels_are_accepted(#[case] labels: &[&str]) {
    assert_eq!(Vec::<ValidationError>::new(), check_trees(&labeled(labels)));
  }

  #[test]
  fn invalid_label_does_not_hide_pair_name_collisions() {
    let expected = vec![
      error("trees[4].label", "tree label \"bad/label\" must not contain / or \\"),
      error(
        "trees",
        "tree pairs (\"a_b\", \"c\") and (\"a\", \"b_c\") give the same output file names (\"a_b_c\"); rename a tree",
      ),
    ];
    assert_eq!(expected, check_trees(&labeled(&["a_b", "c", "a", "b_c", "bad/label"])));
  }

  #[test]
  fn repeated_label_gives_no_pair_name_error() {
    assert_eq!(
      vec![error("trees[2].label", "tree label \"ha\" is used twice")],
      check_trees(&labeled(&["ha", "na", "ha"]))
    );
  }

  #[test]
  fn parse_error_has_field_line_and_column() {
    let errors = check_trees(&trees(&[("ha", T), ("na", "((A,B),\n(C,D)x y);")]));
    let expected = vec![ValidationError {
      field: Some("trees[1].newick".to_owned()),
      message: "tree \"na\": Newick parse error: expected ',' or ')' at byte 15".to_owned(),
      line: Some(2),
      column: Some(8),
    }];
    assert_eq!(expected, errors);
  }

  #[test]
  fn parse_error_column_counts_characters() {
    let errors = check_trees(&trees(&[("ha", T), ("na", "((Ä,B),(C,D)x y);")]));
    assert_eq!((Some(1), Some(15)), (errors[0].line, errors[0].column));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::no_semicolon(   "((A,B),(C,D))", "tree \"na\": Newick parse error: no ';' found")]
  #[case::duplicate_leaf( "((A,A),(C,D));", "tree \"na\": Newick parse error: duplicate leaf name A")]
  #[trace]
  fn parse_error_without_position(#[case] newick: &str, #[case] message: &str) {
    let errors = check_trees(&trees(&[("ha", T), ("na", newick)]));
    let expected = vec![ValidationError { field: Some("trees[1].newick".to_owned()), message: message.to_owned(), line: None, column: None }];
    assert_eq!(expected, errors);
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::disjoint(  "((P,Q),R);")]
  #[case::one_shared("((A,Q),R);")]
  #[trace]
  fn pairs_sharing_fewer_than_two_leaves_are_rejected(#[case] newick: &str) {
    let errors = check_trees(&trees(&[("ha", T), ("na", T), ("pb2", newick)]));
    let expected = vec![
      error("trees", "trees \"ha\" and \"pb2\" share fewer than 2 leaves"),
      error("trees", "trees \"na\" and \"pb2\" share fewer than 2 leaves"),
    ];
    assert_eq!(expected, errors);
  }

  #[test]
  fn pairs_sharing_two_leaves_are_accepted() {
    assert_eq!(
      Vec::<ValidationError>::new(),
      check_trees(&trees(&[("ha", T), ("na", "((A,B),(P,Q));")]))
    );
  }

  #[test]
  fn all_errors_are_reported_together() {
    let errors = check_trees(&trees(&[("a/b", T), ("na", "(A,B"), ("", T)]));
    let fields: Vec<_> = errors.iter().map(|e| e.field.as_deref().unwrap()).collect();
    assert_eq!(vec!["trees[0].label", "trees[2].label", "trees[1].newick"], fields);
  }

  #[test]
  fn parse_trees_numbers_leaves_with_one_taxon_table() {
    let p = parse_trees(&trees(&[("ha", "((A,B),(C,D));"), ("na", "((A,B),(C,P));")])).unwrap();
    assert_eq!(5, p.taxa.len());
    assert_eq!(
      vec!["ha", "na"],
      p.trees.iter().map(|t| t.label.as_str()).collect::<Vec<_>>()
    );
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::negative_gamma( Settings { gamma: -1.0, ..Settings::default() },                       "settings.gamma",         "gamma must be a non-negative number, got -1")]
  #[case::nan_gamma(      Settings { gamma: f64::NAN, ..Settings::default() },                   "settings.gamma",         "gamma must be a non-negative number, got NaN")]
  #[case::infinite_gamma( Settings { gamma: f64::INFINITY, ..Settings::default() },              "settings.gamma",         "gamma must be a non-negative number, got inf")]
  #[case::length_count(   Settings { seq_lengths: Some(vec![1.0]), ..Settings::default() },      "settings.seqLengths",    "got 1 sequence lengths for 2 trees")]
  #[case::zero_length(    Settings { seq_lengths: Some(vec![1.0, 0.0]), ..Settings::default() }, "settings.seqLengths[1]", "sequence length 2 must be a positive number, got 0")]
  #[case::nan_length(     Settings { seq_lengths: Some(vec![f64::NAN, 1.0]), ..Settings::default() }, "settings.seqLengths[0]", "sequence length 1 must be a positive number, got NaN")]
  #[case::zero_rounds(    Settings { rounds: 0, ..Settings::default() },                         "settings.rounds",        "rounds must be at least 1")]
  #[case::zero_mcmc(      Settings { n_mcmc_it: 0, ..Settings::default() },                      "settings.nMcmcIt",       "MCMC steps per leaf must be at least 1")]
  #[case::large_seed(     Settings { seed: 1 << 53, ..Settings::default() },                     "settings.seed",          "seed must be at most 9007199254740991, got 9007199254740992")]
  #[trace]
  fn invalid_settings_are_rejected(#[case] s: Settings, #[case] field: &str, #[case] message: &str) {
    assert_eq!(vec![error(field, message)], check_settings(&s, 2));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::defaults(    Settings::default())]
  #[case::zero_gamma(  Settings { gamma: 0.0, ..Settings::default() })]
  #[case::lengths(     Settings { seq_lengths: Some(vec![1700.0, 0.5]), ..Settings::default() })]
  #[case::largest_seed(Settings { seed: MAX_SEED, ..Settings::default() })]
  #[trace]
  fn valid_settings_are_accepted(#[case] s: Settings) {
    assert_eq!(Vec::<ValidationError>::new(), check_settings(&s, 2));
  }

  #[test]
  fn rounds_leave_room_for_the_final_round() {
    // Strict resolution of three trees adds a final round on top of `rounds`.
    let s = |rounds| Settings { rounds, resolve: ResolveMode::Strict, ..Settings::default() };
    let expected = vec![error(
      "settings.rounds",
      &format!("rounds must be less than {0}, got {0}", usize::MAX),
    )];
    assert_eq!(expected, check_settings(&s(usize::MAX), 3));
    assert_eq!(Vec::<ValidationError>::new(), check_settings(&s(usize::MAX - 1), 3));
  }

  #[test]
  fn every_invalid_length_is_reported() {
    let s = Settings {
      seq_lengths: Some(vec![0.0, -1.0, 2.0]),
      ..Settings::default()
    };
    let expected = vec![
      error("settings.seqLengths", "got 3 sequence lengths for 2 trees"),
      error(
        "settings.seqLengths[0]",
        "sequence length 1 must be a positive number, got 0",
      ),
      error(
        "settings.seqLengths[1]",
        "sequence length 2 must be a positive number, got -1",
      ),
    ];
    assert_eq!(expected, check_settings(&s, 2));
  }

  #[test]
  fn shared_leaf_counts_follow_the_pipeline_pair_order() {
    let p = parse_trees(&trees(&[
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
  fn pair_stem_joins_labels() {
    assert_eq!("ha_na", pair_stem("ha", "na"));
  }
}
