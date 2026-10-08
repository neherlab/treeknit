//! End-to-end runs of the `treeknit` binary.

mod errors;
mod options;
mod outputs;
mod sessions;

#[cfg(test)]
pub(crate) mod tests {
  use std::path::{Path, PathBuf};
  use std::process::Command;

  /// Write each `(name, newick)` of `files` to `<dir>/<name>.nwk`.
  pub(crate) fn write_trees(dir: &Path, files: &[(&str, &str)]) -> Vec<PathBuf> {
    files
      .iter()
      .map(|(name, newick)| {
        let p = dir.join(format!("{name}.nwk"));
        std::fs::write(&p, newick).unwrap();
        p
      })
      .collect()
  }

  pub(crate) fn run(args: &[&str], out: &Path) {
    let status = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .args(args)
      .arg("-o")
      .arg(out)
      .args(["--verbosity-level", "-1"])
      .status()
      .unwrap();
    assert!(status.success());
  }

  pub(crate) const HA: &str = "((A:1,B:1):1,(C:1,(D:1,X:1):1):1);";
  pub(crate) const NA: &str = "((A:1,(B:1,X:1):1):1,(C:1,D:1):1);";

  /// Write a session file with the trees `HA` and `NA`, labeled `ha` and `na`, and `settings`.
  pub(crate) fn write_session(dir: &Path, settings: &serde_json::Value) -> PathBuf {
    let request = serde_json::json!({
      "trees": [{"label": "ha", "newick": HA}, {"label": "na", "newick": NA}],
      "settings": settings,
    });
    let path = dir.join("treeknit_session.json");
    std::fs::write(&path, request.to_string()).unwrap();
    path
  }

  pub(crate) fn mccs(out: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(out.join("MCCs.json")).unwrap()).unwrap()
  }

  pub(crate) fn parameters(out: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(out.join("parameters.json")).unwrap()).unwrap()
  }

  pub(crate) fn text(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
  }
}
