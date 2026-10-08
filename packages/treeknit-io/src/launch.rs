//! Links of the web app that name trees, settings, and a run: the grammar of their keys, read
//! by the web app and the command line, and the canonical link of a request, written by both.
//!
//! A link carries key-value pairs in its query and, for inline data, in its fragment:
//!
//! ```text
//! ?example=h3n2-2017&gamma=3&run
//! ?tree=ha=https://example.org/ha.nwk&tree=na=https://example.org/na.nwk&resolve=strict
//! ?run#session=data:application/gzip;base64,H4sIAAAA...
//! ```
//!
//! The input keys (`example`, `tree`, `session`, `from`) name the trees, the setting keys of
//! [`SETTING_KEYS`] change the settings, `run` starts the run, and `v` is the version of the
//! format. The web app owns the keys of its display (`view`, `pair`, ...), which this module
//! leaves alone.

mod link;
mod location;
mod read;

pub use link::{
  INLINE_SESSION_PREFIX, LONG_LINK_CHARS, LinkEntry, LinkLimits, LinkSource, MAX_LINK_CHARS, TreeAddress,
  inline_session, launch_pairs, link_limits, link_pairs, link_query, query_pairs, web_link,
};
pub use location::{
  DecodeError, FETCH_TIMEOUT_SECONDS, LinkLocation, LocationError, MAX_DOWNLOAD_BYTES, MAX_TEXT_BYTES, UrlPlace,
  decode_tree_bytes, parse_location, request_url, url_place,
};
pub use read::parse_launch;

use crate::analysis::{ResolveMode, Settings, ValidationError};
use crate::output;
use crate::schema::{KeyValue, SETTING_KEYS};
use serde::{Deserialize, Serialize};
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// The version of the link format that this build reads and writes. A link without `v` is
/// version 1, now and after later versions, so that published links keep their meaning.
pub const FORMAT_VERSION: u64 = 1;

/// The label and the location of a tree token `[<label>=]<location>`: the text before the first
/// `=` is a label when it is not empty and holds no `:`, `/`, or `\`, which a label cannot hold
/// and every location before its first `=` does (`https://`, `data:`, a path).
pub fn split_label(token: &str) -> LabeledToken<'_> {
  match token.split_once('=') {
    Some((label, rest)) if !label.is_empty() && !label.contains([':', '/', '\\']) => LabeledToken {
      label: Some(label),
      rest,
    },
    _ => LabeledToken {
      label: None,
      rest: token,
    },
  }
}

/// A token split by [`split_label`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LabeledToken<'a> {
  pub label: Option<&'a str>,
  /// The location or path after the label.
  pub rest: &'a str,
}

/// The settings of `base` with the values of `patch`: `Settings::default()` for trees and
/// examples, and the settings of the session file for `session`.
pub fn apply(base: &Settings, patch: &SettingsPatch) -> Settings {
  Settings {
    gamma: patch.gamma.unwrap_or(base.gamma),
    seq_lengths: patch.seq_lengths.clone().or_else(|| base.seq_lengths.clone()),
    n_mcmc_it: patch.n_mcmc_it.unwrap_or(base.n_mcmc_it),
    resolve: patch.resolve.unwrap_or(base.resolve),
    pre_resolve: patch.pre_resolve.unwrap_or(base.pre_resolve),
    rounds: patch.rounds.unwrap_or(base.rounds),
    final_round: patch.final_round.unwrap_or(base.final_round),
    likelihood: patch.likelihood.unwrap_or(base.likelihood),
    naive: patch.naive.unwrap_or(base.naive),
    seed: patch.seed.unwrap_or(base.seed),
  }
}

/// Every key of links that is not a key of the display, with its value and a one-line
/// description, for the help of the web app: the inputs, the settings, `run`, and `v`.
pub fn launch_keys() -> Vec<LaunchKeyInfo> {
  let key = |key: &str, value: Option<&str>, description: String| LaunchKeyInfo {
    key: key.to_owned(),
    opposite: None,
    value: value.map(str::to_owned),
    description,
  };
  let mut keys = vec![
    key(
      "example",
      Some("<id>"),
      "A built-in example, such as h3n2-2017: the id of the Examples menu.".to_owned(),
    ),
    key(
      "tree",
      Some("[<label>=]<location>"),
      "A tree at an https: address or as a data: text, with an optional label; give at least two.".to_owned(),
    ),
    key(
      "session",
      Some("<location>"),
      format!(
        "A session file ({}) at an https: address or as a data: text.",
        output::SESSION_FILE
      ),
    ),
    key(
      "from",
      Some("opener|parent"),
      "Receive a session file from the page that opened TreeKnit (opener) or that embeds it (parent).".to_owned(),
    ),
  ];
  keys.extend(SETTING_KEYS.iter().map(|s| LaunchKeyInfo {
    key: s.key.to_owned(),
    opposite: s.opposite().map(str::to_owned),
    value: value_placeholder(s.value).map(str::to_owned),
    description: s.description.to_owned(),
  }));
  keys.push(key(
    "run",
    None,
    "Run the analysis once the inputs have loaded.".to_owned(),
  ));
  keys.push(key(
    "v",
    Some("<number>"),
    format!("Version of the link format; without it, version {FORMAT_VERSION}."),
  ));
  keys
}

fn value_placeholder(value: KeyValue) -> Option<&'static str> {
  match value {
    KeyValue::Number => Some("<number>"),
    KeyValue::Numbers => Some("<number>,<number>,..."),
    KeyValue::Integer => Some("<whole number>"),
    KeyValue::Mode => Some("matched|strict|liberal|none"),
    KeyValue::Flag { .. } => None,
  }
}

/// What [`parse_launch`] read from a link.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct LaunchParse {
  /// The launch; `None` when the link has no launch keys or its input cannot load.
  pub launch: Option<Launch>,
  /// Every problem, at the key it concerns.
  pub errors: Vec<ValidationError>,
  /// Keys that are neither launch keys nor keys of the display, in their order.
  pub ignored: Vec<IgnoredKey>,
}

/// The input, the settings, and the run of a link.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Launch {
  pub input: LaunchInput,
  /// The settings of the link, applied by [`apply`] to the defaults or to the settings of the
  /// session file.
  pub settings: SettingsPatch,
  /// The link asks to run the analysis once the input has loaded.
  pub run: bool,
}

/// Where the trees of a launch come from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LaunchInput {
  /// A built-in example (`example=<id>`).
  Example { id: String },
  /// Trees one by one (`tree=...`), at least two.
  Trees { trees: Vec<LaunchTree> },
  /// A session file (`session=<location>`).
  Session { location: LinkLocation },
  /// A session file posted by another window (`from=opener` or `from=parent`).
  Message { source: MessageSource },
}

/// A tree of a link with its label: given in the link, or from the file name of its location.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct LaunchTree {
  pub label: String,
  pub location: LinkLocation,
}

/// The window that posts the session file of `from=`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub enum MessageSource {
  /// The window that opened TreeKnit (`window.opener`).
  Opener,
  /// The window that embeds TreeKnit in a frame (`window.parent`).
  Parent,
}

/// A setting value of a link for each field of `Settings`; `None` keeps the value of the base.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
  pub gamma: Option<f64>,
  pub seq_lengths: Option<Vec<f64>>,
  pub n_mcmc_it: Option<u64>,
  pub resolve: Option<ResolveMode>,
  pub pre_resolve: Option<bool>,
  pub rounds: Option<u64>,
  pub final_round: Option<bool>,
  pub likelihood: Option<bool>,
  pub naive: Option<bool>,
  pub seed: Option<u64>,
}

/// A key of a link that TreeKnit does not read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct IgnoredKey {
  pub key: String,
  /// The launch or display key it most likely misspells, within edit distance 2; none for keys
  /// shorter than 4 characters.
  pub suggestion: Option<String>,
  /// The key follows a tree or session location with a `?`, so it is most likely part of that
  /// location's query: `&` inside a location must be written `%26`.
  pub after_location_query: bool,
}

/// A key of links with its value and description.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct LaunchKeyInfo {
  pub key: String,
  /// For a flag, the key that sets the other value.
  pub opposite: Option<String>,
  /// The form of the value, such as `<number>`; `None` for a key without a value.
  pub value: Option<String>,
  pub description: String,
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  #[rustfmt::skip]
  #[rstest]
  #[case::label_before_first_equals("a=b=https://x/y",  (Some("a"), "b=https://x/y"))]
  #[case::query_with_equals(        "https://x/y?a=b",  (None,      "https://x/y?a=b"))]
  #[case::data_location(            "data:,(A=B,C);",   (None,      "data:,(A=B,C);"))]
  #[case::labeled_data(             "ha=data:,(A,B);",  (Some("ha"), "data:,(A,B);"))]
  #[case::empty_label(              "=https://x/y",     (None,      "=https://x/y"))]
  #[case::path_with_equals(         "./a=b.nwk",        (None,      "./a=b.nwk"))]
  #[case::backslash(                "a\\b=c.nwk",       (None,      "a\\b=c.nwk"))]
  #[trace]
  fn labels_are_split_at_the_first_equals(#[case] token: &str, #[case] (label, rest): (Option<&str>, &str)) {
    assert_eq!(LabeledToken { label, rest }, split_label(token));
  }

  #[test]
  fn apply_replaces_the_values_of_the_patch() {
    let base = Settings {
      seed: 9,
      pre_resolve: true,
      ..Settings::default()
    };
    let patch = SettingsPatch {
      gamma: Some(3.0),
      pre_resolve: Some(false),
      ..SettingsPatch::default()
    };
    let expected = Settings {
      gamma: 3.0,
      pre_resolve: false,
      ..base.clone()
    };
    assert_eq!(expected, apply(&base, &patch));
  }

  #[test]
  fn launch_keys_list_inputs_settings_run_and_version() {
    let keys: Vec<String> = launch_keys().into_iter().map(|k| k.key).collect();
    let expected = [
      "example",
      "tree",
      "session",
      "from",
      "gamma",
      "seq-lengths",
      "n-mcmc-it",
      "resolve",
      "pre-resolve",
      "rounds",
      "no-final-round",
      "no-likelihood",
      "naive",
      "seed",
      "run",
      "v",
    ];
    assert_eq!(expected.to_vec(), keys);
  }
}
