//! Runs of session files.

#[cfg(test)]
mod tests {
  use crate::tests::{HA, NA, mccs, parameters, run, text, write_session, write_trees};
  use indoc::formatdoc;
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  use std::path::PathBuf;
  use std::process::Command;
  use tempfile::tempdir;

  #[test]
  fn session_gives_the_mccs_of_the_tree_files_with_the_same_settings() {
    let dir = tempdir().unwrap();
    let request = write_session(
      dir.path(),
      &serde_json::json!({"gamma": 3, "resolve": "strict", "seed": 5}),
    );
    let from_session = dir.path().join("from-session");
    run(&["--session", request.to_str().unwrap()], &from_session);
    let paths = write_trees(dir.path(), &[("ha", HA), ("na", NA)]);
    let from_files = dir.path().join("from-files");
    let (a, b) = (paths[0].to_str().unwrap(), paths[1].to_str().unwrap());
    run(
      &[a, b, "--gamma", "3", "--resolve", "strict", "--seed", "5"],
      &from_files,
    );
    assert_eq!(mccs(&from_files), mccs(&from_session));
  }

  #[test]
  fn session_writes_the_session_file_and_trees_named_by_label() {
    let dir = tempdir().unwrap();
    let request = write_session(dir.path(), &serde_json::json!({}));
    let out = dir.path().join("out");
    run(&["--session", request.to_str().unwrap(), "--impute"], &out);
    let written: serde_json::Value =
      serde_json::from_str(&std::fs::read_to_string(out.join("treeknit_session.json")).unwrap()).unwrap();
    let expected = serde_json::json!({
      "trees": [{"label": "ha", "newick": HA}, {"label": "na", "newick": NA}],
      "settings": {
        "gamma": 2.0, "seqLengths": null, "nMcmcIt": 50, "resolve": "matched", "preResolve": false, "rounds": 1,
        "finalRound": true, "likelihood": true, "naive": false, "seed": 1,
      },
    });
    assert_eq!(expected, written);
    let present: Vec<bool> = ["ha_resolved.nwk", "na_imputed.nwk", "ARG/na_liberal_resolved.nwk"]
      .iter()
      .map(|f| out.join(f).exists())
      .collect();
    assert_eq!(vec![true, true, true], present);
  }

  /// Run `treeknit` with `args` in the directory of a session file, returning the path of the
  /// session file, the exit code, and the error output.
  fn run_session(settings: &serde_json::Value, args: &[&str]) -> (PathBuf, Option<i32>, String) {
    let dir = tempdir().unwrap();
    let request = write_session(dir.path(), settings);
    let output = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .arg("--session")
      .arg(&request)
      .args(args)
      .arg("-o")
      .arg(dir.path().join("out"))
      .args(["--verbosity-level", "-1"])
      .current_dir(dir.path())
      .output()
      .unwrap();
    (request, output.status.code(), String::from_utf8(output.stderr).unwrap())
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::tree_file(&["ha.nwk"], "the argument '--session <FILE>' cannot be used with '[TREE]...'")]
  #[case::example(  &["--example", "5-leaves"], "the argument '--session <FILE>' cannot be used with '--example <ID>'")]
  #[case::former(   &["--better-MCCs"], "the argument '--session <FILE>' cannot be used with '--better-MCCs'")]
  #[trace]
  fn session_with_tree_files_or_former_options_is_a_usage_error(
    #[case] args: &[&str],
    #[case] message: &str,
  ) {
    let (_, code, stderr) = run_session(&serde_json::json!({}), args);
    // clap shows the usage of the arguments given, `-o` and `--verbosity-level` of `run_session`.
    let expected = formatdoc! {"
      error: {message}

      Usage: treeknit --session <FILE> --outdir <OUTDIR> --verbosity-level <VERBOSITY_LEVEL> [TREE]...

      For more information, try '--help'.
    "};
    assert_eq!((Some(2), expected), (code, stderr));
  }

  #[test]
  fn session_with_invalid_settings_exits_with_their_message_and_field() {
    let (request, code, stderr) = run_session(&serde_json::json!({"gamma": -1}), &[]);
    let expected = format!(
      "Error: {}: settings.gamma: gamma must be a non-negative number, got -1\n",
      request.display()
    );
    assert_eq!((Some(1), expected), (code, stderr));
  }

  #[test]
  fn session_with_a_seed_above_the_largest_exact_integer_names_the_field() {
    let seed = treeknit_io::analysis::MAX_SEED + 1;
    let (request, code, stderr) = run_session(&serde_json::json!({ "seed": seed }), &[]);
    let expected = format!(
      "Error: {}: settings.seed: the seed {seed} of the session file is above 9007199254740991, the \
       largest integer a JavaScript number holds exactly\n",
      request.display()
    );
    assert_eq!((Some(1), expected), (code, stderr));
  }

  #[test]
  fn session_with_an_invalid_tree_names_the_file_the_field_and_the_position() {
    let dir = tempdir().unwrap();
    let request = serde_json::json!({
      "trees": [{"label": "ha", "newick": HA}, {"label": "na", "newick": "((A,B),\n(C,D)x y);"}],
    });
    let path = dir.path().join("treeknit_session.json");
    std::fs::write(&path, request.to_string()).unwrap();
    let out = dir.path().join("out");
    let output = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .arg("--session")
      .arg(&path)
      .arg("-o")
      .arg(&out)
      .args(["--verbosity-level", "-1"])
      .output()
      .unwrap();
    let expected = format!(
      "Error: {}: trees[1].newick: line 2, column 8: tree \"na\": Newick parse error: expected comment, ')', ',', ':', ';' at byte 15\n",
      path.display()
    );
    assert_eq!(
      (Some(1), expected, false),
      (
        output.status.code(),
        String::from_utf8(output.stderr).unwrap(),
        out.exists()
      )
    );
  }

  #[test]
  fn session_with_a_wrong_structure_names_the_file() {
    let (request, code, stderr) = run_session(&serde_json::json!({"gama": 1}), &[]);
    let expected = format!(
      "Error: {}: not a TreeKnit session file: unknown field `gama`, expected one of `gamma`, `seqLengths`, \
       `nMcmcIt`, `resolve`, `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`, `seed` at line 1 column 151\n",
      request.display()
    );
    assert_eq!((Some(1), expected), (code, stderr));
  }

  #[test]
  fn analysis_options_change_the_settings_of_a_session_file() {
    let dir = tempdir().unwrap();
    let request = write_session(
      dir.path(),
      &serde_json::json!({"gamma": 3, "resolve": "strict", "seed": 5}),
    );
    let out = dir.path().join("out");
    run(
      &["--session", request.to_str().unwrap(), "--seed", "7", "--no-likelihood"],
      &out,
    );
    let p = parameters(&out);
    let written: serde_json::Value = serde_json::from_str(&text(&out.join("treeknit_session.json"))).unwrap();
    let log = text(&out.join("log.txt"));
    let expected = (
      serde_json::json!([3.0, "strict", 7, false]),
      serde_json::json!([7, false]),
      (true, true),
    );
    let actual = (
      serde_json::json!([p["gamma"], p["resolution"], p["seed"], p["likelihood_sort"]]),
      serde_json::json!([written["settings"]["seed"], written["settings"]["likelihood"]]),
      (
        log.contains("--seed changes the setting of the input"),
        log.contains("--no-likelihood changes the setting of the input"),
      ),
    );
    assert_eq!(expected, actual);
  }
}
