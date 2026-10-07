//! The analysis request of the command line and the web app: the trees, the settings, their
//! validation, and the conversion of the settings to core `Options`.
//!
//! Both surfaces validate with the same functions, so a request that one accepts the other
//! accepts too. The checks reject the inputs known to make the core panic, because a panic
//! traps the WebAssembly instance.

use crate::figure::FigureOptionKey;
use crate::newick;
use crate::output::{self, OutputOptions};
use crate::wire::wire_name;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use strum::VariantArray;
use treeknit_core::{Options, Resolution, Taxa, Tree};
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// Largest seed: JavaScript numbers hold integers exactly only up to 2^53 - 1, and a session
/// file must pass through the web app unchanged.
pub const MAX_SEED: u64 = (1 << 53) - 1;

/// Largest number of rounds: 2^32 - 2. The web app runs the core as 32-bit WebAssembly, whose
/// counts end at 2^32 - 1, and a run with more than two trees can add a final round on top of
/// `rounds`. The command line applies the same bound, so that a request one surface accepts the
/// other accepts too.
pub const MAX_ROUNDS: u64 = 0xFFFF_FFFE;

/// Largest number of MCMC steps per leaf: 2^32 - 1, the largest count of 32-bit WebAssembly,
/// for the reason of [`MAX_ROUNDS`].
pub const MAX_MCMC_IT: u64 = 0xFFFF_FFFF;

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

impl TreeText {
  /// The tree `newick` labeled `label`.
  pub fn new(label: impl Into<String>, newick: impl Into<String>) -> Self {
    TreeText {
      label: label.into(),
      newick: newick.into(),
    }
  }
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
  /// MCMC steps per leaf (`--n-mcmc-it`). A fixed-width integer, so that a session file reads
  /// the same on 32-bit WebAssembly and on 64-bit hosts, and the check against [`MAX_MCMC_IT`]
  /// reports a value that no `usize` of WebAssembly holds.
  pub n_mcmc_it: u64,
  /// How trees are resolved (`--resolve`).
  pub resolve: ResolveMode,
  /// Before inference, add to each tree the splits of other trees compatible with all trees
  /// (`--pre-resolve`).
  pub pre_resolve: bool,
  /// Rounds of pair inference (`--rounds`), a fixed-width integer as `n_mcmc_it`.
  pub rounds: u64,
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
      n_mcmc_it: count_u64(o.n_mcmc),
      resolve: ResolveMode::default(),
      pre_resolve: o.pre_resolve,
      rounds: count_u64(o.rounds),
      final_round: o.final_unresolved_round,
      likelihood: o.likelihood_sort,
      naive: o.naive,
      seed: 1,
    }
  }
}

/// How trees are resolved. The serde names of the variants are the values of `resolve` in session
/// files and links, and of `--resolve` on the command line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, VariantArray)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum), value(rename_all = "lower"))]
#[serde(rename_all = "lowercase")]
pub enum ResolveMode {
  None,
  Strict,
  Liberal,
  Matched,
}

impl Default for ResolveMode {
  /// The resolution of the core's default options.
  fn default() -> Self {
    Options::default().resolution.into()
  }
}

impl From<Resolution> for ResolveMode {
  fn from(r: Resolution) -> ResolveMode {
    match r {
      Resolution::None => ResolveMode::None,
      Resolution::Strict => ResolveMode::Strict,
      Resolution::Liberal => ResolveMode::Liberal,
      Resolution::Matched => ResolveMode::Matched,
    }
  }
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

/// A setting that a check of [`check_settings`] reports, named as the field of [`Settings`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, VariantArray)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub enum SettingKey {
  Gamma,
  SeqLengths,
  NMcmcIt,
  Rounds,
  Seed,
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
pub fn labels(trees: &[TreeText]) -> Vec<String> {
  trees.iter().map(|t| t.label.clone()).collect()
}

/// Least number of trees of a run, because TreeKnit infers the MCCs of pairs of trees.
pub const MIN_TREES: usize = 2;

/// Number of trees of a run that builds an ARG: the ARG joins the two segments of one pair.
pub const ARG_TREE_COUNT: usize = 2;

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

/// Check the settings of a request with `k` trees.
pub fn check_settings(s: &Settings, k: usize) -> Vec<ValidationError> {
  let mut errors = Vec::new();
  if !(s.gamma.is_finite() && s.gamma >= 0.0) {
    errors.push(ValidationError::at(
      Field::Setting { key: SettingKey::Gamma },
      format!("gamma must be a non-negative number, got {}", s.gamma),
    ));
  }
  if let Some(v) = &s.seq_lengths {
    if v.len() != k {
      errors.push(ValidationError::at(
        Field::Setting {
          key: SettingKey::SeqLengths,
        },
        format!("got {} sequence lengths for {k} trees", v.len()),
      ));
    }
    for (i, x) in v.iter().enumerate() {
      if !(x.is_finite() && *x > 0.0) {
        errors.push(ValidationError::at(
          Field::SeqLength { index: i },
          format!("sequence length {} must be a positive number, got {x}", i + 1),
        ));
      }
    }
  }
  if s.rounds == 0 {
    errors.push(ValidationError::at(
      Field::Setting {
        key: SettingKey::Rounds,
      },
      "rounds must be at least 1",
    ));
  } else if s.rounds > MAX_ROUNDS {
    errors.push(ValidationError::at(
      Field::Setting {
        key: SettingKey::Rounds,
      },
      format!("rounds must be at most {MAX_ROUNDS}, got {}", s.rounds),
    ));
  }
  if s.n_mcmc_it == 0 {
    errors.push(ValidationError::at(
      Field::Setting {
        key: SettingKey::NMcmcIt,
      },
      "MCMC steps per leaf must be at least 1",
    ));
  } else if s.n_mcmc_it > MAX_MCMC_IT {
    errors.push(ValidationError::at(
      Field::Setting {
        key: SettingKey::NMcmcIt,
      },
      format!("MCMC steps per leaf must be at most {MAX_MCMC_IT}, got {}", s.n_mcmc_it),
    ));
  }
  if s.seed > MAX_SEED {
    errors.push(ValidationError::at(
      Field::Setting { key: SettingKey::Seed },
      format!("seed must be at most {MAX_SEED}, got {}", s.seed),
    ));
  }
  errors
}

/// Core options of the settings for `k` trees, or every error of `check_settings`: the core
/// panics on settings that fail it. `parallel` runs independent pairs on several threads; each
/// surface states its choice, because browsers give WebAssembly no threads.
pub fn options(s: &Settings, k: usize, parallel: bool) -> Result<Options, Vec<ValidationError>> {
  let errors = check_settings(s, k);
  if !errors.is_empty() {
    return Err(errors);
  }
  let mut o = Options::for_trees(k);
  if let Some(v) = &s.seq_lengths {
    o.seq_lengths.clone_from(v);
  }
  o.gamma = s.gamma;
  o.n_mcmc = count_usize(s.n_mcmc_it);
  o.resolution = s.resolve.into();
  o.pre_resolve = s.pre_resolve;
  o.rounds = count_usize(s.rounds);
  o.final_unresolved_round = s.final_round;
  o.likelihood_sort = s.likelihood;
  o.naive = s.naive;
  o.parallel = parallel;
  Ok(o)
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

/// Labels of the web app for trees loaded from `file_names`: each file name without its last
/// extension (`ha.nwk` gives `ha`, `ha.tree.nwk` gives `ha.tree`), with every character that the
/// label check rejects (path separators, characters reserved on Windows, control characters)
/// replaced by `_`, or `tree` for a name that leaves no usable label (empty, blank, `.`, or
/// `..`). A label that equals one of `existing_labels` or an earlier new label, ignoring case as
/// the label check does, gets the first free suffix of `_2`, `_3`, ... Every label passes the
/// label check.
///
/// The command line labels its tree files by path instead (the file stem, and the parent
/// directory when stems collide), because it has directories to tell equal file names apart;
/// the web app has file names only.
pub fn tree_labels(file_names: &[String], existing_labels: &[String]) -> Vec<String> {
  let mut taken: std::collections::BTreeSet<String> = existing_labels.iter().map(|l| label_key(l)).collect();
  file_names
    .iter()
    .map(|name| {
      let stem = match name.rsplit_once('.') {
        Some((stem, _)) if !stem.is_empty() => stem,
        _ => name.as_str(),
      };
      let stem: String = stem
        .chars()
        .map(|c| if label_char_rejected(c) { '_' } else { c })
        .collect();
      let base = if stem.trim().is_empty() || stem == "." || stem == ".." {
        "tree"
      } else {
        stem.as_str()
      };
      let mut label = base.to_owned();
      let mut suffix = 1;
      while taken.contains(&label_key(&label)) {
        suffix += 1;
        label = format!("{base}_{suffix}");
      }
      taken.insert(label_key(&label));
      label
    })
    .collect()
}

/// Key under which labels name the same output files: labels with equal keys are repeated
/// labels, because the file systems of macOS and Windows ignore case. The key lowercases each
/// character on its own, so that a letter has one key wherever it stands in the label (`Σ` and
/// `σ` both give `σ`, while `str::to_lowercase` gives a final `ς`). Unicode normalization and
/// full case folding are not applied (`kb/issues/M-label-check-misses-file-system-name-rules.md`).
pub fn label_key(label: &str) -> String {
  label.chars().flat_map(char::to_lowercase).collect()
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
        shared: sets[i].intersection_count(&sets[j]),
      });
    }
  }
  counts
}

/// A count of the core as a setting.
fn count_u64(n: usize) -> u64 {
  #[expect(clippy::expect_used, reason = "no target has a usize wider than 64 bits")]
  u64::try_from(n).expect("a usize fits in u64")
}

/// A count of the settings for the core, after `check_settings` has bounded it by [`MAX_ROUNDS`]
/// or [`MAX_MCMC_IT`], which every `usize` holds.
fn count_usize(n: u64) -> usize {
  #[expect(
    clippy::expect_used,
    reason = "check_settings bounds the counts by 2^32 - 1, the largest usize of 32-bit targets"
  )]
  usize::try_from(n).expect("a checked count fits in usize")
}

/// Characters that Windows does not allow in file names, besides the path separators.
const RESERVED_CHARS: [char; 7] = ['<', '>', ':', '"', '|', '?', '*'];

/// Whether the label check rejects a label holding `c`: a path separator, a character reserved
/// on Windows, or a control character.
fn label_char_rejected(c: char) -> bool {
  c == '/' || c == '\\' || RESERVED_CHARS.contains(&c) || c.is_control()
}

/// A label is also a file-name stem of the outputs, so it must name a file in the results
/// directory on every release target: no path separator, no character reserved on Windows, no
/// control character, and neither `.` nor `..`. Labels with the same [`label_key`], such as
/// labels that differ only in case, are repeated labels.
///
/// Return the error of each label, in the order of `trees`.
fn check_labels(trees: &[TreeText]) -> Vec<Option<ValidationError>> {
  let mut errors = Vec::new();
  let mut seen: BTreeMap<String, &str> = BTreeMap::new();
  for (i, t) in trees.iter().enumerate() {
    let field = Field::TreeLabel { index: i };
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
      match seen.entry(label_key(l)) {
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

/// Pairs whose output files would get the same name, by the `label_key` of `check_labels`:
/// `MCCs_<a>_<b>.dat` joins two labels with `_`, so the labels `a_b`, `c`, `a`, `b_c` give the
/// pairs (0,1) and (2,3) one name. Only pairs of two `valid` labels are checked.
fn check_pair_stems(trees: &[TreeText], valid: &[bool]) -> Vec<ValidationError> {
  let mut first: BTreeMap<String, (usize, usize, String)> = BTreeMap::new();
  let mut errors = Vec::new();
  for i in 0..trees.len() {
    for j in i + 1..trees.len() {
      if !(valid[i] && valid[j]) {
        continue;
      }
      let stem = pair_stem(&trees[i].label, &trees[j].label);
      match first.entry(label_key(&stem)) {
        Entry::Occupied(e) => {
          let (a, b, first_stem) = e.get();
          let names = if *first_stem == stem {
            format!("the same output file names ({stem:?})")
          } else {
            format!("output file names ({first_stem:?} and {stem:?}) that differ only in case")
          };
          errors.push(ValidationError::at(
            Field::Trees,
            format!(
              "tree pairs ({:?}, {:?}) and ({:?}, {:?}) give {names}; rename a tree",
              trees[*a].label, trees[*b].label, trees[i].label, trees[j].label
            ),
          ));
        },
        Entry::Vacant(e) => {
          e.insert((i, j, stem));
        },
      }
    }
  }
  errors
}

fn parse_error(i: usize, t: &TreeText, e: &newick::ParseError) -> ValidationError {
  ValidationError {
    field: Some(Field::TreeNewick { index: i }),
    message: format!("tree {:?}: {e}", t.label),
    line: e.location.map(|l| l.line),
    column: e.location.map(|l| l.column),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_support::{clades, texts};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::json;
  use std::collections::BTreeSet;
  use treeknit_testing::assert_err;

  const T: &str = "((A,B),(C,D));";

  /// The validation errors of `trees`, none when they parse.
  fn tree_errors(trees: &[TreeText]) -> Vec<ValidationError> {
    match parse_trees(trees) {
      Ok(_) => Vec::new(),
      Err(errors) => errors,
    }
  }

  fn labeled(labels: &[&str]) -> Vec<TreeText> {
    texts(&labels.iter().map(|&l| (l, T)).collect::<Vec<_>>())
  }

  fn label(index: usize) -> Field {
    Field::TreeLabel { index }
  }

  fn setting(key: SettingKey) -> Field {
    Field::Setting { key }
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
    assert_err!(
      serde_json::from_value::<Settings>(json!({"gama": 1.0})),
      "unknown field `gama`, expected one of `gamma`, `seqLengths`, `nMcmcIt`, `resolve`, \
       `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`, `seed`"
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
  fn setting_keys_are_fields_of_the_settings() {
    // Oracle: the serialized `Settings`, whose field names the web app's form uses.
    let settings = serde_json::to_value(Settings::default()).unwrap();
    let not_in_settings: Vec<String> = SettingKey::VARIANTS
      .iter()
      .map(wire_name)
      .filter(|n| settings.get(n).is_none())
      .collect();
    assert_eq!(Vec::<String>::new(), not_in_settings);
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
  #[case::none(   ResolveMode::None,    Resolution::None)]
  #[case::strict( ResolveMode::Strict,  Resolution::Strict)]
  #[case::liberal(ResolveMode::Liberal, Resolution::Liberal)]
  #[case::matched(ResolveMode::Matched, Resolution::Matched)]
  #[trace]
  fn options_take_the_resolution_mode(#[case] resolve: ResolveMode, #[case] expected: Resolution) {
    let s = Settings { resolve, ..Settings::default() };
    assert_eq!(expected, options(&s, 2, false).unwrap().resolution);
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
    let o = options(&s, 3, false).unwrap();
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
    assert_eq!(
      vec![1.0; 3],
      options(&Settings::default(), 3, false).unwrap().seq_lengths
    );
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::parallel(  true)]
  #[case::sequential(false)]
  #[trace]
  fn options_take_the_thread_choice(#[case] parallel: bool) {
    assert_eq!(parallel, options(&Settings::default(), 2, parallel).unwrap().parallel);
  }

  #[test]
  fn options_reject_settings_that_fail_the_checks() {
    let s = Settings {
      gamma: -1.0,
      rounds: 0,
      ..Settings::default()
    };
    assert_eq!(Err(check_settings(&s, 2)), options(&s, 2, true).map(|_| ()));
    assert_eq!(2, check_settings(&s, 2).len());
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

  #[rustfmt::skip]
  #[rstest]
  #[case::no_tree(        &[],                         Field::Trees,      "need at least two trees")]
  #[case::one_tree(       &["ha"],                     Field::Trees,      "need at least two trees")]
  #[case::empty_label(    &["ha", ""],                 label(1),          "tree 2 needs a label")]
  #[case::blank_label(    &["ha", " "],                label(1),          "tree 2 needs a label")]
  #[case::duplicate_label(&["ha", "na", "ha"],         label(2),          "tree label \"ha\" is used twice")]
  #[case::parent_path(    &["ha", "../x"],             label(1),          "tree label \"../x\" must not contain / or \\")]
  #[case::slash(          &["a/b", "na"],              label(0),          "tree label \"a/b\" must not contain / or \\")]
  #[case::backslash(      &["ha", "a\\b"],             label(1),          "tree label \"a\\\\b\" must not contain / or \\")]
  #[case::control(        &["ha", "a\tb"],             label(1),          "tree label \"a\\tb\" must not contain control characters")]
  #[case::dot(            &["ha", "."],                label(1),          "tree label \".\" is not a file name")]
  #[case::dot_dot(        &["..", "na"],               label(0),          "tree label \"..\" is not a file name")]
  #[case::pair_stems(     &["a_b", "c", "a", "b_c"],   Field::Trees,      "tree pairs (\"a_b\", \"c\") and (\"a\", \"b_c\") give the same output file names (\"a_b_c\"); rename a tree")]
  #[case::case_only(      &["HA", "na", "ha"],         label(2),          "tree label \"ha\" differs from \"HA\" only in case, so their output files get the same name")]
  #[case::final_sigma(    &["ΑΣ", "na", "ασ"],         label(2),          "tree label \"ασ\" differs from \"ΑΣ\" only in case, so their output files get the same name")]
  #[case::pair_stems_case(&["a_b", "c", "A", "B_c"],   Field::Trees,      "tree pairs (\"a_b\", \"c\") and (\"A\", \"B_c\") give output file names (\"a_b_c\" and \"A_B_c\") that differ only in case; rename a tree")]
  #[case::less_than(      &["ha", "a<b"],              label(1),          "tree label \"a<b\" must not contain any of <>:\"|?*")]
  #[case::greater_than(   &["ha", "a>b"],              label(1),          "tree label \"a>b\" must not contain any of <>:\"|?*")]
  #[case::colon(          &["ha", "a:b"],              label(1),          "tree label \"a:b\" must not contain any of <>:\"|?*")]
  #[case::quote(          &["ha", "a\"b"],             label(1),          "tree label \"a\\\"b\" must not contain any of <>:\"|?*")]
  #[case::pipe(           &["ha", "a|b"],              label(1),          "tree label \"a|b\" must not contain any of <>:\"|?*")]
  #[case::question(       &["ha", "a?b"],              label(1),          "tree label \"a?b\" must not contain any of <>:\"|?*")]
  #[case::star(           &["ha", "a*b"],              label(1),          "tree label \"a*b\" must not contain any of <>:\"|?*")]
  #[trace]
  fn invalid_labels_are_rejected(#[case] labels: &[&str], #[case] field: Field, #[case] message: &str) {
    assert_eq!(vec![ValidationError::at(field, message)], tree_errors(&labeled(labels)));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::dots_inside(  &["ha", "a.b", "..c"])]
  #[case::unicode(      &["A-H3N2 HA", "NA ü"])]
  #[case::underscores(  &["a_b", "c", "a"])]
  #[case::case_distinct(&["HA", "ha2"])]
  #[trace]
  fn usable_labels_are_accepted(#[case] labels: &[&str]) {
    assert_eq!(Vec::<ValidationError>::new(), tree_errors(&labeled(labels)));
  }

  #[test]
  fn invalid_label_does_not_hide_pair_name_collisions() {
    let expected = vec![
      ValidationError::at(label(4), "tree label \"bad/label\" must not contain / or \\"),
      ValidationError::at(
        Field::Trees,
        "tree pairs (\"a_b\", \"c\") and (\"a\", \"b_c\") give the same output file names (\"a_b_c\"); rename a tree",
      ),
    ];
    assert_eq!(expected, tree_errors(&labeled(&["a_b", "c", "a", "b_c", "bad/label"])));
  }

  #[test]
  fn repeated_label_gives_no_pair_name_error() {
    assert_eq!(
      vec![ValidationError::at(label(2), "tree label \"ha\" is used twice")],
      tree_errors(&labeled(&["ha", "na", "ha"]))
    );
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

  #[rustfmt::skip]
  #[rstest]
  #[case::negative_gamma(Settings { gamma: -1.0, ..Settings::default() },                       setting(SettingKey::Gamma),      "gamma must be a non-negative number, got -1")]
  #[case::nan_gamma(     Settings { gamma: f64::NAN, ..Settings::default() },                   setting(SettingKey::Gamma),      "gamma must be a non-negative number, got NaN")]
  #[case::infinite_gamma(Settings { gamma: f64::INFINITY, ..Settings::default() },              setting(SettingKey::Gamma),      "gamma must be a non-negative number, got inf")]
  #[case::length_count(  Settings { seq_lengths: Some(vec![1.0]), ..Settings::default() },      setting(SettingKey::SeqLengths), "got 1 sequence lengths for 2 trees")]
  #[case::zero_length(   Settings { seq_lengths: Some(vec![1.0, 0.0]), ..Settings::default() }, Field::SeqLength { index: 1 }, "sequence length 2 must be a positive number, got 0")]
  #[case::nan_length(    Settings { seq_lengths: Some(vec![f64::NAN, 1.0]), ..Settings::default() }, Field::SeqLength { index: 0 }, "sequence length 1 must be a positive number, got NaN")]
  #[case::zero_rounds(   Settings { rounds: 0, ..Settings::default() },                         setting(SettingKey::Rounds),     "rounds must be at least 1")]
  #[case::zero_mcmc(     Settings { n_mcmc_it: 0, ..Settings::default() },                      setting(SettingKey::NMcmcIt),    "MCMC steps per leaf must be at least 1")]
  #[case::large_seed(    Settings { seed: 1 << 53, ..Settings::default() },                     setting(SettingKey::Seed),       "seed must be at most 9007199254740991, got 9007199254740992")]
  #[trace]
  fn invalid_settings_are_rejected(#[case] s: Settings, #[case] field: Field, #[case] message: &str) {
    assert_eq!(vec![ValidationError::at(field, message)], check_settings(&s, 2));
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
  fn count_limits_are_the_counts_of_32_bit_webassembly() {
    // Oracle: u32::MAX is the largest `usize` of wasm32; the final round of strict resolution
    // with three trees comes on top of `rounds`.
    assert_eq!(
      (u32::MAX - 1, u32::MAX),
      (u32::try_from(MAX_ROUNDS).unwrap(), u32::try_from(MAX_MCMC_IT).unwrap())
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

  #[rustfmt::skip]
  #[rstest]
  #[case::largest_rounds( Settings { rounds: 0xFFFF_FFFE, ..Settings::default() },    vec![])]
  #[case::too_many_rounds(Settings { rounds: 0xFFFF_FFFF, ..Settings::default() },    vec![ValidationError::at(setting(SettingKey::Rounds), "rounds must be at most 4294967294, got 4294967295")])]
  #[case::largest_mcmc(   Settings { n_mcmc_it: 0xFFFF_FFFF, ..Settings::default() }, vec![])]
  #[case::too_many_mcmc(  Settings { n_mcmc_it: 0x1_0000_0000, ..Settings::default() }, vec![ValidationError::at(setting(SettingKey::NMcmcIt), "MCMC steps per leaf must be at most 4294967295, got 4294967296")])]
  #[trace]
  fn counts_are_checked_at_their_maximum(#[case] s: Settings, #[case] expected: Vec<ValidationError>) {
    let s = Settings { resolve: ResolveMode::Strict, ..s };
    assert_eq!(expected, check_settings(&s, 3));
  }

  #[test]
  fn every_invalid_length_is_reported() {
    let s = Settings {
      seq_lengths: Some(vec![0.0, -1.0, 2.0]),
      ..Settings::default()
    };
    let expected = vec![
      ValidationError::at(setting(SettingKey::SeqLengths), "got 3 sequence lengths for 2 trees"),
      ValidationError::at(
        Field::SeqLength { index: 0 },
        "sequence length 1 must be a positive number, got 0",
      ),
      ValidationError::at(
        Field::SeqLength { index: 1 },
        "sequence length 2 must be a positive number, got -1",
      ),
    ];
    assert_eq!(expected, check_settings(&s, 2));
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

  fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|&s| s.to_owned()).collect()
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::extension(     &["ha.nwk", "na.tree"],          &[],             &["ha", "na"])]
  #[case::last_extension(&["ha.tree.nwk"],                &[],             &["ha.tree"])]
  #[case::no_extension(  &["ha"],                         &[],             &["ha"])]
  #[case::dot_file(      &[".nwk"],                       &[],             &[".nwk"])]
  #[case::empty_name(    &["", ""],                       &[],             &["tree", "tree_2"])]
  #[case::existing(      &["ha.nwk"],                     &["ha"],         &["ha_2"])]
  #[case::existing_case( &["ha.nwk"],                     &["HA"],         &["ha_2"])]
  #[case::new_repeated(  &["ha.nwk", "ha.tree", "ha.nwk"], &[],            &["ha", "ha_2", "ha_3"])]
  #[case::suffix_taken(  &["ha.nwk"],                     &["ha", "ha_2"], &["ha_3"])]
  #[case::no_files(      &[],                             &["ha"],         &[])]
  #[case::reserved(      &["ha:1.nwk", "a<b>|c?*\"d.nwk"],  &[],            &["ha_1", "a_b__c___d"])]
  #[case::separators(    &["a\\b.nwk"],                   &[],             &["a_b"])]
  #[case::control(       &["a\tb.nwk"],                    &[],             &["a_b"])]
  #[case::blank_stem(    &[" .nwk"],                      &[],             &["tree"])]
  #[case::dot_stem(      &["..nwk"],                      &[],             &["tree"])]
  #[case::dot_dot_stem(  &["...nwk", ".."],               &[],             &["tree", "tree_2"])]
  #[trace]
  fn tree_labels_follow_the_web_label_policy(#[case] files: &[&str], #[case] existing: &[&str], #[case] expected: &[&str]) {
    assert_eq!(strings(expected), tree_labels(&strings(files), &strings(existing)));
  }

  #[test]
  fn tree_labels_pass_the_label_check() {
    // Repeated file names and existing labels give labels that parse_trees accepts.
    let existing = strings(&["HA", "na"]);
    let files = [
      "ha.nwk", "ha.nwk", "NA.nwk", "ha:1.nwk", " .nwk", "..nwk", "a\tb", "...nwk",
    ];
    let new = tree_labels(&strings(&files), &existing);
    let all: Vec<&str> = existing.iter().chain(&new).map(String::as_str).collect();
    assert_eq!(Vec::<ValidationError>::new(), tree_errors(&labeled(&all)));
  }

  #[test]
  fn pair_stem_joins_labels() {
    assert_eq!("ha_na", pair_stem("ha", "na"));
  }
}
