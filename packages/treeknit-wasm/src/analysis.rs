use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;
use treeknit_core::arg::arg_from_trees;
use treeknit_core::{Options, PairResult, Resolution, Taxa, Tree};
use treeknit_io::{arg, mccs, newick};
use tsify::Tsify;

pub fn analyze(request: &AnalysisRequest) -> Result<Analysis, String> {
  let k = request.trees.len();
  if k < 2 {
    return Err("need at least two trees".to_owned());
  }
  check_labels(&request.trees)?;
  let opts = options(&request.settings, k)?;
  let mut trees = request
    .trees
    .iter()
    .map(|t| newick::parse_first(&t.newick, &t.label).map_err(|e| format!("tree {}: {e}", t.label)))
    .collect::<Result<Vec<Tree>, String>>()?;
  let taxa = Taxa::from_trees(&trees);
  for t in &mut trees {
    t.assign_taxa(&taxa).map_err(|e| format!("tree {}: {e}", t.label))?;
  }

  let pairs = treeknit_core::run(&mut trees, &taxa, &opts, request.settings.seed);
  let mut files = mcc_files(&pairs, &trees, &taxa)?;
  files.extend(tree_files(&trees, "_resolved"));
  files.extend(tree_files(
    &treeknit_core::imputed_trees(&trees, &pairs, taxa.len()),
    "_imputed",
  ));
  let (arg, arg_files) = (k == 2 && !pairs[0].mccs.is_empty())
    .then(|| arg_outcome(&trees, &pairs[0], &taxa))
    .map_or((None, vec![]), |(outcome, files)| (Some(outcome), files));
  files.extend(arg_files);
  Ok(Analysis {
    pairs: pairs
      .iter()
      .map(|p| PairMccs {
        trees: [trees[p.i].label.clone(), trees[p.j].label.clone()],
        mccs: p.mccs.iter().map(|m| taxa.names_of(m)).collect(),
      })
      .collect(),
    arg,
    files,
  })
}

/// Trees to compare, and the settings of the analysis.
#[derive(Clone, Debug, Deserialize, Serialize, Tsify)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnalysisRequest {
  pub trees: Vec<TreeText>,
  #[serde(default)]
  pub settings: Settings,
}

/// A labeled tree in Newick format.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Tsify)]
#[serde(deny_unknown_fields)]
pub struct TreeText {
  pub label: String,
  pub newick: String,
}

/// Settings of the command line; a missing field takes the command-line default.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Tsify)]
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
  /// Seed of the random number generator (`--seed`).
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize, Tsify)]
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

/// MCCs of every tree pair, the ARG of two trees, and the output files of the command line.
#[derive(Clone, Debug, Serialize, Tsify)]
pub struct Analysis {
  pub pairs: Vec<PairMccs>,
  /// `None` for more than two trees, or when the two trees share no MCC.
  pub arg: Option<ArgOutcome>,
  pub files: Vec<OutputFile>,
}

/// MCCs of one tree pair, as leaf names.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Tsify)]
pub struct PairMccs {
  pub trees: [String; 2],
  pub mccs: Vec<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Tsify)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ArgOutcome {
  Built { reassortments: usize },
  Failed { message: String },
}

/// A result file, named as the command line names it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct OutputFile {
  pub name: String,
  pub media_type: String,
  pub text: String,
}

impl OutputFile {
  fn new(name: String, text: &str) -> Self {
    let json = Path::new(&name).extension().is_some_and(|e| e == "json");
    let media_type = if json { "application/json" } else { "text/plain" };
    OutputFile {
      name,
      media_type: media_type.to_owned(),
      text: format!("{text}\n"),
    }
  }
}

fn check_labels(trees: &[TreeText]) -> Result<(), String> {
  let mut seen = BTreeSet::new();
  for t in trees {
    if t.label.trim().is_empty() {
      return Err("every tree needs a label".to_owned());
    }
    if !seen.insert(t.label.as_str()) {
      return Err(format!("tree label {} is used twice", t.label));
    }
  }
  Ok(())
}

/// Rejects settings on which the core panics: a panic aborts the WebAssembly instance.
fn options(s: &Settings, k: usize) -> Result<Options, String> {
  if !(s.gamma.is_finite() && s.gamma >= 0.0) {
    return Err(format!("gamma must be a non-negative number, got {}", s.gamma));
  }
  if s.rounds == 0 {
    return Err("rounds must be at least 1".to_owned());
  }
  let mut o = Options::for_trees(k);
  if let Some(v) = &s.seq_lengths {
    if v.len() != k {
      return Err(format!("got {} sequence lengths for {k} trees", v.len()));
    }
    if !v.iter().all(|x| x.is_finite() && *x > 0.0) {
      return Err("sequence lengths must be positive numbers".to_owned());
    }
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
  // No threads in the browser.
  o.parallel = false;
  Ok(o)
}

fn mcc_files(pairs: &[PairResult], trees: &[Tree], taxa: &Taxa) -> Result<Vec<OutputFile>, String> {
  let json = serde_json::to_string_pretty(&mccs::to_json(pairs, trees, taxa)).map_err(|e| e.to_string())?;
  let mut files = vec![OutputFile::new("MCCs.json".to_owned(), &json)];
  // Legacy text format of TreeKnit.jl < 0.5, as the command line writes it.
  files.extend(pairs.iter().map(|p| {
    let name = if trees.len() == 2 {
      "MCCs.dat".to_owned()
    } else {
      format!("MCCs_{}_{}.dat", trees[p.i].label, trees[p.j].label)
    };
    let names: Vec<Vec<String>> = p.mccs.iter().map(|m| taxa.names_of(m)).collect();
    OutputFile::new(name, &mccs::to_lines(&names))
  }));
  Ok(files)
}

fn tree_files<'a>(trees: &'a [Tree], suffix: &'a str) -> impl Iterator<Item = OutputFile> + 'a {
  trees
    .iter()
    .map(move |t| OutputFile::new(format!("{}{suffix}.nwk", t.label), &newick::write(t)))
}

/// The ARG of two trees, with its files: the extended Newick, the node table, and the
/// liberally resolved trees it was built from.
fn arg_outcome(trees: &[Tree], pair: &PairResult, taxa: &Taxa) -> (ArgOutcome, Vec<OutputFile>) {
  let (t1, t2, m) = treeknit_core::arg_inputs(trees, pair, taxa.len());
  match arg_from_trees(&t1, &t2, &m, taxa.len()) {
    Ok(a) => {
      let mut files = vec![
        OutputFile::new("arg.nwk".to_owned(), &arg::extended_newick(&a)),
        OutputFile::new("nodes.dat".to_owned(), &arg::node_table(&a)),
      ];
      files.extend(tree_files(&a.trees, "_liberal_resolved"));
      let outcome = ArgOutcome::Built {
        reassortments: a.n_hybrids(),
      };
      (outcome, files)
    },
    Err(e) => (ArgOutcome::Failed { message: e.to_string() }, vec![]),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::{Value, json};

  macro_rules! assert_err {
    ($result:expr, $expected:expr) => {
      match $result {
        Ok(_) => panic!("expected error {:?}, got Ok", $expected),
        Err(e) => assert_eq!($expected, e.to_string()),
      }
    };
  }

  fn request(trees: &[(&str, &str)], settings: Settings) -> AnalysisRequest {
    AnalysisRequest {
      trees: trees
        .iter()
        .map(|(label, newick)| TreeText {
          label: (*label).to_owned(),
          newick: (*newick).to_owned(),
        })
        .collect(),
      settings,
    }
  }

  fn clades(newick: &str) -> BTreeSet<BTreeSet<String>> {
    let t = newick::parse(newick, "t").unwrap();
    t.internals()
      .into_iter()
      .filter(|&n| n != t.root)
      .map(|n| t.leaves_below(n).into_iter().map(|l| t.name(l).to_owned()).collect())
      .collect()
  }

  fn pair(trees: [&str; 2], mccs: &[&[&str]]) -> PairMccs {
    PairMccs {
      trees: trees.map(str::to_owned),
      mccs: mccs.iter().map(|m| m.iter().map(|&x| x.to_owned()).collect()).collect(),
    }
  }

  fn file<'a>(a: &'a Analysis, name: &str) -> &'a str {
    &a.files.iter().find(|f| f.name == name).unwrap().text
  }

  fn file_names(a: &Analysis) -> Vec<&str> {
    a.files.iter().map(|f| f.name.as_str()).collect()
  }

  #[test]
  fn identical_trees_form_one_mcc_without_reassortment() {
    let r = request(
      &[("ha", "((A,B),(C,D));"), ("na", "((A,B),(C,D));")],
      Settings::default(),
    );
    let a = analyze(&r).unwrap();
    assert_eq!(vec![pair(["ha", "na"], &[&["A", "B", "C", "D"]])], a.pairs);
    assert_eq!(Some(ArgOutcome::Built { reassortments: 0 }), a.arg);
  }

  #[test]
  fn moved_leaf_matches_reference() {
    // Oracle: fixtures/doc_mccs_1.json (TreeKnit.jl)
    let r = request(
      &[("ha", "((A,B),(C,(D,X)));"), ("na", "((A,(B,X)),(C,D));")],
      Settings::default(),
    );
    let a = analyze(&r).unwrap();
    assert_eq!(vec![pair(["ha", "na"], &[&["X"], &["A", "B", "C", "D"]])], a.pairs);
    assert_eq!(Some(ArgOutcome::Built { reassortments: 1 }), a.arg);
    assert_eq!(
      json!({"MCC_dict": {"1": {"trees": ["ha", "na"], "mccs": [["X"], ["A", "B", "C", "D"]]}}}),
      serde_json::from_str::<Value>(file(&a, "MCCs.json")).unwrap()
    );
    assert_eq!("X\nA,B,C,D\n", file(&a, "MCCs.dat"));
  }

  #[test]
  fn two_trees_give_the_files_of_the_command_line() {
    let r = request(
      &[("ha", "((A,B),(C,(D,X)));"), ("na", "((A,(B,X)),(C,D));")],
      Settings::default(),
    );
    let a = analyze(&r).unwrap();
    let expected = vec![
      "MCCs.json",
      "MCCs.dat",
      "ha_resolved.nwk",
      "na_resolved.nwk",
      "ha_imputed.nwk",
      "na_imputed.nwk",
      "arg.nwk",
      "nodes.dat",
      "ha_liberal_resolved.nwk",
      "na_liberal_resolved.nwk",
    ];
    assert_eq!(expected, file_names(&a));
  }

  #[test]
  fn three_trees_give_all_pairs_and_no_arg() {
    let t = "((A,B),(C,D));";
    let r = request(&[("ha", t), ("na", t), ("pb2", t)], Settings::default());
    let a = analyze(&r).unwrap();
    let all: &[&[&str]] = &[&["A", "B", "C", "D"]];
    let expected_pairs = vec![
      pair(["ha", "na"], all),
      pair(["ha", "pb2"], all),
      pair(["na", "pb2"], all),
    ];
    assert_eq!(expected_pairs, a.pairs);
    assert_eq!(None, a.arg);
    let expected_files = vec![
      "MCCs.json",
      "MCCs_ha_na.dat",
      "MCCs_ha_pb2.dat",
      "MCCs_na_pb2.dat",
      "ha_resolved.nwk",
      "na_resolved.nwk",
      "pb2_resolved.nwk",
      "ha_imputed.nwk",
      "na_imputed.nwk",
      "pb2_imputed.nwk",
    ];
    assert_eq!(expected_files, file_names(&a));
  }

  #[test]
  fn leaf_missing_from_one_tree_is_imputed() {
    let r = request(
      &[("ha", "((A,B),(C,(D,P)));"), ("na", "((A,B),(C,D));")],
      Settings::default(),
    );
    let a = analyze(&r).unwrap();
    assert_eq!(
      json!({"MCC_dict": {"1": {
          "trees": ["ha", "na"],
          "mccs": [["A", "B", "C", "D", "P"]],
          "imputed": [{"leaf": "P", "tree": "ha", "mcc": 0, "ambiguous": false}],
      }}}),
      serde_json::from_str::<Value>(file(&a, "MCCs.json")).unwrap()
    );
    // Oracle: P is sister of D in ha.
    assert_eq!(clades("((A,B),(C,(D,P)));"), clades(file(&a, "na_imputed.nwk")));
    assert_eq!(clades("((A,B),(C,D));"), clades(file(&a, "na_resolved.nwk")));
  }

  #[rstest]
  #[case::matched(ResolveMode::Matched, clades("((A,B),(C,D));"))]
  #[case::none(ResolveMode::None, BTreeSet::new())]
  #[trace]
  fn resolve_mode_controls_resolution(#[case] resolve: ResolveMode, #[case] expected: BTreeSet<BTreeSet<String>>) {
    // Matched resolution copies na's splits into ha's polytomy.
    let r = request(
      &[("ha", "(A,B,C,D);"), ("na", "((A,B),(C,D));")],
      Settings {
        resolve,
        ..Settings::default()
      },
    );
    let a = analyze(&r).unwrap();
    assert_eq!(expected, clades(file(&a, "ha_resolved.nwk")));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::json(  "MCCs.json", "application/json")]
  #[case::newick("ha.nwk",    "text/plain")]
  #[case::table( "nodes.dat", "text/plain")]
  #[trace]
  fn output_file_media_type_follows_the_extension(#[case] name: &str, #[case] expected: &str) {
    assert_eq!(expected, OutputFile::new(name.to_owned(), "").media_type);
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

  #[rustfmt::skip]
    #[rstest]
    #[case::one_tree(        &[("ha", "(A,B);")],                     Settings::default(),                                                         "need at least two trees")]
    #[case::empty_label(     &[("ha", "(A,B);"), (" ", "(A,B);")],    Settings::default(),                                                         "every tree needs a label")]
    #[case::duplicate_label( &[("ha", "(A,B);"), ("ha", "(A,B);")],   Settings::default(),                                                         "tree label ha is used twice")]
    #[case::bad_newick(      &[("ha", "(A,B);"), ("na", "(A,B")],     Settings::default(),                                                         "tree na: Newick parse error: no ';' found")]
    #[case::negative_gamma(  &[("ha", "(A,B);"), ("na", "(A,B);")],   Settings { gamma: -1.0, ..Settings::default() },                             "gamma must be a non-negative number, got -1")]
    #[case::nan_gamma(       &[("ha", "(A,B);"), ("na", "(A,B);")],   Settings { gamma: f64::NAN, ..Settings::default() },                         "gamma must be a non-negative number, got NaN")]
    #[case::zero_rounds(     &[("ha", "(A,B);"), ("na", "(A,B);")],   Settings { rounds: 0, ..Settings::default() },                               "rounds must be at least 1")]
    #[case::length_count(    &[("ha", "(A,B);"), ("na", "(A,B);")],   Settings { seq_lengths: Some(vec![1.0]), ..Settings::default() },            "got 1 sequence lengths for 2 trees")]
    #[case::zero_length(     &[("ha", "(A,B);"), ("na", "(A,B);")],   Settings { seq_lengths: Some(vec![1.0, 0.0]), ..Settings::default() },       "sequence lengths must be positive numbers")]
    #[trace]
    fn invalid_requests_are_rejected(#[case] trees: &[(&str, &str)], #[case] settings: Settings, #[case] expected: &str) {
        assert_err!(analyze(&request(trees, settings)), expected);
    }
}
