//! Settings schema: the defaults, ranges, applicability, and help texts of every setting, so
//! the web app shows the rules of the command line without a copy of them.

use crate::analysis::{MAX_SEED, ResolveMode, Settings};
use serde::Serialize;
use treeknit_core::Options;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// Help on the order of the trees.
pub const TREE_ORDER_HELP: &str = "With matched resolution, splits of earlier trees win conflicts.";

/// Help on the final round without resolution (`--no-final-round` skips it).
pub const FINAL_ROUND_HELP: &str = "With strict or liberal resolution and more than two trees, re-infer the MCCs without \
   resolution in a final extra round, because resolving later pairs can invalidate the MCCs of earlier pairs.";

/// Help on pre-resolution (`--pre-resolve`).
pub const PRE_RESOLVE_HELP: &str = "Before inference, add to each tree the splits of other trees that are compatible \
   with all trees, which is mostly useful without resolution.";

/// Why a setting of the pair inference does not apply in naive mode: naive MCCs skip the
/// inference (`treeknit_core::pipeline`, `infer`).
const NAIVE_REASON: &str = "Naive MCCs skip the inference that uses this setting.";

/// Rules and help of the settings for a request with a given number of trees and settings.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SettingsSchema {
  /// One entry per setting, named as the field of `Settings`.
  pub settings: SettingFields,
  /// The resolution modes, in the order the web app lists them.
  pub modes: Vec<ModeInfo>,
  /// Help on why the order of the trees matters.
  pub tree_order_help: String,
}

/// The schema of each field of `Settings`, except `resolve`, which `SettingsSchema.modes`
/// describes.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SettingFields {
  pub gamma: NumberSetting,
  /// One sequence length per tree; `default` is the length a newly added tree gets.
  pub seq_lengths: NumberSetting,
  pub n_mcmc_it: NumberSetting,
  pub rounds: NumberSetting,
  pub seed: NumberSetting,
  pub pre_resolve: ToggleSetting,
  pub final_round: ToggleSetting,
  pub likelihood: ToggleSetting,
  pub naive: ToggleSetting,
}

/// A numeric setting.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct NumberSetting {
  pub default: f64,
  /// Smallest accepted value, or, with `min_exclusive`, the bound that values must exceed.
  pub min: f64,
  /// Values must be greater than `min`, not equal to it.
  pub min_exclusive: bool,
  /// Largest accepted value; `None` without an upper bound.
  pub max: Option<f64>,
  /// Step of the input control.
  pub step: f64,
  /// The setting changes the result of a run with the current settings.
  pub applies: bool,
  /// Why the setting does not apply; `None` when it applies.
  pub reason: Option<String>,
  /// One-sentence help text.
  pub help: String,
}

/// An on-off setting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ToggleSetting {
  pub default: bool,
  /// The setting changes the result of a run with the current settings.
  pub applies: bool,
  /// Why the setting does not apply; `None` when it applies.
  pub reason: Option<String>,
  /// One-sentence help text.
  pub help: String,
}

/// A resolution mode with its display name and its effect.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ModeInfo {
  pub mode: ResolveMode,
  /// Display name, such as "Matched".
  pub name: String,
  /// One-line effect of the mode on the trees.
  pub effect: String,
}

/// The resolution modes with their names and effects, in the order the web app lists them; the
/// default mode comes first.
pub fn modes() -> Vec<ModeInfo> {
  let mode = |mode, name: &str, effect: &str| ModeInfo {
    mode,
    name: name.to_owned(),
    effect: effect.to_owned(),
  };
  vec![
    mode(
      ResolveMode::Matched,
      "Matched",
      "Resolve during inference and with the inferred MCCs, then resolve all trees so that their topologies match \
       within every MCC; where splits of different trees conflict, earlier trees win, and MCCs that cannot be \
       matched are split.",
    ),
    mode(
      ResolveMode::Strict,
      "Strict",
      "Resolve during inference and with the inferred MCCs, adding unambiguous splits only.",
    ),
    mode(
      ResolveMode::Liberal,
      "Liberal",
      "As strict, also adding ambiguous splits.",
    ),
    mode(
      ResolveMode::None,
      "None",
      "Do not resolve with MCCs, so MCCs require identical topologies.",
    ),
  ]
}

/// The schema of the settings of a request with `k` trees and the settings `s`. Applicability
/// follows `treeknit_core`: naive mode skips the pair inference, the only user of γ, the MCMC
/// steps, the likelihood tie-break with its sequence lengths, and the random numbers; the final
/// round runs only with strict or liberal resolution and more than two trees. Bounds are those
/// of `analysis::check_settings`.
pub fn settings_schema(k: usize, s: &Settings) -> SettingsSchema {
  let d = Settings::default();
  let unless = |skip: Option<&str>| (skip.is_none(), skip.map(str::to_owned));
  let naive = s.naive.then_some(NAIVE_REASON);
  let number = |default, min, min_exclusive, max, step, skip: Option<&str>, help: &str| {
    let (applies, reason) = unless(skip);
    NumberSetting {
      default,
      min,
      min_exclusive,
      max,
      step,
      applies,
      reason,
      help: help.to_owned(),
    }
  };
  let toggle = |default, skip: Option<&str>, help: &str| {
    let (applies, reason) = unless(skip);
    ToggleSetting {
      default,
      applies,
      reason,
      help: help.to_owned(),
    }
  };
  let final_round_skip = if !matches!(s.resolve, ResolveMode::Strict | ResolveMode::Liberal) {
    Some("Only strict and liberal resolution run a final round.")
  } else if k <= 2 {
    Some("A final round runs only with more than two trees.")
  } else {
    None
  };
  let seq_lengths_skip =
    naive.or_else(|| (!s.likelihood).then_some("Only the likelihood tie-break uses sequence lengths."));
  let seed_skip = s.naive.then_some("Naive MCCs use no random numbers.");
  SettingsSchema {
    settings: SettingFields {
      gamma: number(
        d.gamma,
        0.0,
        false,
        None,
        0.1,
        naive,
        "Cost γ of a reassortment, that is of removing an MCC.",
      ),
      seq_lengths: number(
        default_seq_length(),
        0.0,
        true,
        None,
        1.0,
        seq_lengths_skip,
        "Sequence length of each segment, in the order of the trees, used by the likelihood tie-break.",
      ),
      n_mcmc_it: number(
        exact_usize(d.n_mcmc_it),
        1.0,
        false,
        None,
        1.0,
        naive,
        "MCMC steps per leaf of the simulated annealing.",
      ),
      rounds: number(
        exact_usize(d.rounds),
        1.0,
        false,
        None,
        1.0,
        None,
        "Rounds of pair inference.",
      ),
      seed: number(
        exact_u64(d.seed),
        0.0,
        false,
        Some(exact_u64(MAX_SEED)),
        1.0,
        seed_skip,
        "Seed of the random number generator, so that a run can be repeated.",
      ),
      pre_resolve: toggle(d.pre_resolve, None, PRE_RESOLVE_HELP),
      final_round: toggle(d.final_round, final_round_skip, FINAL_ROUND_HELP),
      likelihood: toggle(
        d.likelihood,
        naive,
        "Break ties between equally good configurations with branch lengths.",
      ),
      naive: toggle(
        d.naive,
        None,
        "Return naive MCCs (γ → ∞): the maximal clades whose subtrees are identical in both trees, without inference.",
      ),
    },
    modes: modes(),
    tree_order_help: TREE_ORDER_HELP.to_owned(),
  }
}

/// The sequence length of a tree without one: the core default.
fn default_seq_length() -> f64 {
  Options::for_trees(1).seq_lengths.first().copied().unwrap_or(1.0)
}

#[expect(
  clippy::as_conversions,
  reason = "the counts of the settings are far below 2^53, so the conversion is exact"
)]
fn exact_usize(n: usize) -> f64 {
  n as f64
}

#[expect(
  clippy::as_conversions,
  reason = "seeds are at most MAX_SEED = 2^53 - 1, which f64 holds exactly"
)]
fn exact_u64(n: u64) -> f64 {
  n as f64
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::analysis::check_settings;
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  fn applies(schema: &SettingsSchema) -> [(&'static str, bool); 9] {
    let f = &schema.settings;
    [
      ("gamma", f.gamma.applies),
      ("seqLengths", f.seq_lengths.applies),
      ("nMcmcIt", f.n_mcmc_it.applies),
      ("rounds", f.rounds.applies),
      ("seed", f.seed.applies),
      ("preResolve", f.pre_resolve.applies),
      ("finalRound", f.final_round.applies),
      ("likelihood", f.likelihood.applies),
      ("naive", f.naive.applies),
    ]
  }

  #[test]
  fn schema_defaults_are_the_settings_defaults() {
    // Oracle: `Settings::default()` is the command-line default, and a new tree gets the core
    // sequence length 1.0 (`Options::for_trees`).
    let d = Settings::default();
    let f = settings_schema(2, &d).settings;
    let expected = (2.0, 1.0, 50.0, 1.0, 1.0, false, true, true, false);
    let actual = (
      f.gamma.default,
      f.seq_lengths.default,
      f.n_mcmc_it.default,
      f.rounds.default,
      f.seed.default,
      f.pre_resolve.default,
      f.final_round.default,
      f.likelihood.default,
      f.naive.default,
    );
    assert_eq!(expected, actual);
  }

  #[test]
  fn schema_bounds_are_the_validation_bounds() {
    // Oracle: the checks of `analysis::check_settings`.
    let f = settings_schema(2, &Settings::default()).settings;
    let bound = |n: &NumberSetting| (n.min, n.min_exclusive, n.max);
    let expected = [
      (0.0, false, None),
      (0.0, true, None),
      (1.0, false, None),
      (1.0, false, None),
      (0.0, false, Some(9_007_199_254_740_991.0)),
    ];
    let actual = [
      bound(&f.gamma),
      bound(&f.seq_lengths),
      bound(&f.n_mcmc_it),
      bound(&f.rounds),
      bound(&f.seed),
    ];
    assert_eq!(expected, actual);
  }

  #[rstest]
  #[case::gamma_at_min(Settings { gamma: 0.0, ..Settings::default() }, true)]
  #[case::seq_length_at_exclusive_min(Settings { seq_lengths: Some(vec![0.0, 1.0]), ..Settings::default() }, false)]
  #[case::mcmc_at_min(Settings { n_mcmc_it: 1, ..Settings::default() }, true)]
  #[case::rounds_at_min(Settings { rounds: 1, ..Settings::default() }, true)]
  #[case::seed_at_max(Settings { seed: MAX_SEED, ..Settings::default() }, true)]
  #[case::seed_above_max(Settings { seed: MAX_SEED + 1, ..Settings::default() }, false)]
  fn schema_bounds_agree_with_validation_at_the_bound(#[case] s: Settings, #[case] valid: bool) {
    assert_eq!(valid, check_settings(&s, 2).is_empty());
  }

  #[test]
  fn every_setting_applies_with_the_defaults_except_the_final_round_of_two_trees() {
    let schema = settings_schema(2, &Settings::default());
    let expected = [
      ("gamma", true),
      ("seqLengths", true),
      ("nMcmcIt", true),
      ("rounds", true),
      ("seed", true),
      ("preResolve", true),
      ("finalRound", false),
      ("likelihood", true),
      ("naive", true),
    ];
    assert_eq!(expected, applies(&schema));
    assert_eq!(
      Some("Only strict and liberal resolution run a final round.".to_owned()),
      schema.settings.final_round.reason
    );
  }

  #[rstest]
  #[case::matched_three_trees(
    ResolveMode::Matched,
    3,
    Some("Only strict and liberal resolution run a final round.")
  )]
  #[case::none_three_trees(ResolveMode::None, 3, Some("Only strict and liberal resolution run a final round."))]
  #[case::strict_two_trees(ResolveMode::Strict, 2, Some("A final round runs only with more than two trees."))]
  #[case::liberal_two_trees(ResolveMode::Liberal, 2, Some("A final round runs only with more than two trees."))]
  #[case::strict_three_trees(ResolveMode::Strict, 3, None)]
  #[case::liberal_three_trees(ResolveMode::Liberal, 3, None)]
  fn final_round_applies_to_strict_and_liberal_with_more_than_two_trees(
    #[case] resolve: ResolveMode,
    #[case] k: usize,
    #[case] reason: Option<&str>,
  ) {
    // Oracle: `schedule` in `treeknit_core::pipeline` adds the extra round only for these.
    let s = Settings {
      resolve,
      ..Settings::default()
    };
    let f = settings_schema(k, &s).settings.final_round;
    assert_eq!((reason.is_none(), reason.map(str::to_owned)), (f.applies, f.reason));
  }

  #[test]
  fn naive_mode_disables_the_settings_of_the_pair_inference() {
    // Oracle: `infer` in `treeknit_core::pipeline` returns naive MCCs without `infer_pair`, the
    // only user of γ, the MCMC steps, the likelihood tie-break, and the random generator.
    let s = Settings {
      naive: true,
      ..Settings::default()
    };
    let schema = settings_schema(3, &s);
    let expected = [
      ("gamma", false),
      ("seqLengths", false),
      ("nMcmcIt", false),
      ("rounds", true),
      ("seed", false),
      ("preResolve", true),
      ("finalRound", false),
      ("likelihood", false),
      ("naive", true),
    ];
    assert_eq!(expected, applies(&schema));
    let f = &schema.settings;
    let reasons = [
      &f.gamma.reason,
      &f.seq_lengths.reason,
      &f.n_mcmc_it.reason,
      &f.likelihood.reason,
    ];
    assert_eq!([&Some(NAIVE_REASON.to_owned()); 4], reasons);
    assert_eq!(Some("Naive MCCs use no random numbers.".to_owned()), f.seed.reason);
  }

  #[test]
  fn likelihood_off_disables_the_sequence_lengths() {
    // Oracle: `choose_conf` in `treeknit_core::pair` reads the sequence lengths only with
    // `likelihood_sort`.
    let s = Settings {
      likelihood: false,
      ..Settings::default()
    };
    let f = settings_schema(2, &s).settings;
    let expected = (
      false,
      Some("Only the likelihood tie-break uses sequence lengths.".to_owned()),
      true,
      true,
    );
    assert_eq!(
      expected,
      (
        f.seq_lengths.applies,
        f.seq_lengths.reason,
        f.gamma.applies,
        f.likelihood.applies
      )
    );
  }

  #[test]
  fn schema_lists_every_mode_once_with_the_default_first() {
    let modes: Vec<ResolveMode> = settings_schema(2, &Settings::default())
      .modes
      .into_iter()
      .map(|m| m.mode)
      .collect();
    let expected = vec![
      ResolveMode::Matched,
      ResolveMode::Strict,
      ResolveMode::Liberal,
      ResolveMode::None,
    ];
    assert_eq!(expected, modes);
    assert_eq!(ResolveMode::default(), expected[0]);
  }

  #[test]
  fn schema_serializes_camel_case_fields() {
    let json = serde_json::to_value(settings_schema(2, &Settings::default())).unwrap();
    let expected = serde_json::json!({
      "default": 1.0, "min": 0.0, "minExclusive": true, "max": null, "step": 1.0, "applies": true,
      "reason": null,
      "help": "Sequence length of each segment, in the order of the trees, used by the likelihood tie-break.",
    });
    assert_eq!(expected, json["settings"]["seqLengths"]);
    assert_eq!(serde_json::json!(TREE_ORDER_HELP), json["treeOrderHelp"]);
    assert_eq!(serde_json::json!("matched"), json["modes"][0]["mode"]);
  }
}
