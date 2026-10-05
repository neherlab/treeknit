//! Version of TreeKnit, as the web app shows it.

use serde::Serialize;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// The TreeKnit version, the source repository, and its release page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct AppVersion {
  /// Version as the command line reports it, such as `0.5.0` or `0.5.0-dev`.
  pub version: String,
  /// URL of the source repository, from the workspace `repository` field.
  pub repository: String,
  /// URL of the release page of the repository, where the command-line binaries are published.
  pub releases: String,
}

impl AppVersion {
  /// The version `version`, as the build script of a surface derives it, with the repository of
  /// the workspace and its release page.
  pub fn new(version: &str) -> Self {
    let repository = env!("CARGO_PKG_REPOSITORY").trim_end_matches('/');
    AppVersion {
      version: version.to_owned(),
      repository: repository.to_owned(),
      releases: format!("{repository}/releases"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn app_version_takes_the_workspace_repository_and_its_releases() {
    // Oracle: the `repository` field of the workspace `Cargo.toml`, and the GitHub release page
    // under it.
    let expected = AppVersion {
      version: "1.0.0-dev".into(),
      repository: "https://github.com/neherlab/treeknit-rs".into(),
      releases: "https://github.com/neherlab/treeknit-rs/releases".into(),
    };
    assert_eq!(expected, AppVersion::new("1.0.0-dev"));
  }
}
