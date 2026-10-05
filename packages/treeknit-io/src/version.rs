//! Version of TreeKnit, as the web app shows it.

use serde::Serialize;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// The TreeKnit version and the source repository.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct AppVersion {
  /// Version as the command line reports it, such as `0.5.0` or `0.5.0-dev`.
  pub version: String,
  /// URL of the source repository, from the workspace `repository` field.
  pub repository: String,
}
