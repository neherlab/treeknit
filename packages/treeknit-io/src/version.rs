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

impl AppVersion {
  /// The version `version`, as the build script of a surface derives it, with the repository of
  /// the workspace.
  pub fn new(version: &str) -> Self {
    AppVersion {
      version: version.to_owned(),
      repository: env!("CARGO_PKG_REPOSITORY").to_owned(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn app_version_takes_the_workspace_repository() {
    // Oracle: the `repository` field of the workspace `Cargo.toml`.
    let expected = AppVersion {
      version: "1.0.0-dev".into(),
      repository: "https://github.com/neherlab/treeknit-rs".into(),
    };
    assert_eq!(expected, AppVersion::new("1.0.0-dev"));
  }
}
