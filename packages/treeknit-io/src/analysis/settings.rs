//! The settings of a request: their defaults, their bounds, and the core `Options` they give.

use crate::analysis::{Field, ValidationError};
use serde::{Deserialize, Serialize};
use strum::VariantArray;
use treeknit_core::{Options, Resolution};
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::analysis::tests::setting;
  use crate::wire::wire_name;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::json;
  use treeknit_testing::assert_err;

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
}
