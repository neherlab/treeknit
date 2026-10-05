//! Settings schema: the defaults, ranges, applicability, and help texts of every setting, so
//! the web app shows the rules of the command line without a copy of them.

use crate::analysis::ResolveMode;
use serde::Serialize;
#[cfg(feature = "tsify")]
use tsify::Tsify;

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
  /// Smallest accepted value; validation states whether the bound itself is accepted.
  pub min: f64,
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
