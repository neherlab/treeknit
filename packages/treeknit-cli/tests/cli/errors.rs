//! Runs that stop with an input error, and the warnings of a run.

#[cfg(test)]
mod tests {
  use crate::tests::{HA, NA, run, write_trees};
  use indoc::{formatdoc, indoc};
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  use std::path::PathBuf;
  use std::process::Command;
  use tempfile::{TempDir, tempdir};

  /// A failed run: its directory with the input files and the output directory `out`, the exit
  /// code, the error output, and whether the output directory was created.
  struct Failure {
    dir: TempDir,
    code: Option<i32>,
    stderr: String,
    created: bool,
  }

  /// Run `treeknit` on Newick `trees` written to files `t0.nwk`, `t1.nwk`, ..., expecting a
  /// failure.
  fn fail(trees: &[&str], args: &[&str]) -> Failure {
    let names: Vec<String> = (0..trees.len()).map(|i| format!("t{i}")).collect();
    let files: Vec<(&str, &str)> = names.iter().map(String::as_str).zip(trees.iter().copied()).collect();
    fail_named(&files, args)
  }

  /// Run `treeknit` on the `(name, newick)` trees of `files`, expecting a failure.
  fn fail_named(files: &[(&str, &str)], args: &[&str]) -> Failure {
    let dir = tempdir().unwrap();
    let paths = write_trees(dir.path(), files);
    let out = dir.path().join("out");
    let output = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .args(&paths)
      .args(args)
      .arg("-o")
      .arg(&out)
      .args(["--verbosity-level", "-1"])
      .output()
      .unwrap();
    Failure {
      dir,
      code: output.status.code(),
      stderr: String::from_utf8(output.stderr).unwrap(),
      created: out.exists(),
    }
  }

  /// Assert that `f` exited with 1, printed exactly `message` as the error, and created no output
  /// directory.
  fn assert_failed(f: &Failure, message: &str) {
    assert_eq!(
      (Some(1), format!("Error: {message}\n").as_str(), false),
      (f.code, f.stderr.as_str(), f.created)
    );
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::gamma_negative(&["--gamma=-1"],                            "gamma must be a non-negative number, got -1")]
  #[case::gamma_nan(     &["--gamma=nan"],                           "gamma must be a non-negative number, got NaN")]
  #[case::lengths_zero(  &["--seq-lengths", "0 0"],                  "sequence length 1 must be a positive number, got 0\nsequence length 2 must be a positive number, got 0")]
  #[case::lengths_former(&["--better-MCCs", "--seq-lengths", "0 0"], "sequence length 1 must be a positive number, got 0\nsequence length 2 must be a positive number, got 0")]
  #[case::seed_large(    &["--seed", "9007199254740992"],            "seed must be at most 9007199254740991, got 9007199254740992")]
  #[case::mcmc_zero(     &["--n-mcmc-it", "0"],                      "MCMC steps per leaf must be at least 1")]
  #[case::mcmc_former(   &["--better-MCCs", "--n-mcmc-it", "0"],     "MCMC steps per leaf must be at least 1")]
  #[case::rounds_former( &["--better-MCCs", "--rounds", "0"],        "rounds must be at least 1")]
  #[trace]
  fn invalid_settings_exit_with_their_message(#[case] args: &[&str], #[case] message: &str) {
    assert_failed(&fail(&[HA, NA], args), message);
  }

  #[test]
  fn every_flag_error_is_reported_with_the_tree_errors() {
    let args = [
      "--better-MCCs",
      "--resolve",
      "none",
      "--seq-lengths",
      "x 1",
      "--gamma=-1",
    ];
    let f = fail(&[HA, "(A,B"], &args);
    let expected = formatdoc! {r#"
      {}:1:5: tree "t1": Newick parse error: The tree does not end with ';' at byte 4
      --seq-lengths should look like 1500,2000, got "x 1": invalid float literal
      former method options (--better-trees, --better-MCCs, --no-resolve, --liberal-resolve, --resolve-all-rounds, --match-topologies) cannot be combined with --resolve, --pre-resolve, --no-final-round or --final-round; see --help-resolve
      gamma must be a non-negative number, got -1"#,
      f.dir.path().join("t1.nwk").display()
    };
    assert_failed(&f, &expected);
  }

  #[test]
  fn too_few_tree_files_are_reported_with_the_flag_errors() {
    let f = fail(&[HA], &["--gamma=-1"]);
    let expected = indoc! {"
      need at least two trees
      gamma must be a non-negative number, got -1"};
    assert_failed(&f, expected);
  }

  #[test]
  fn unreadable_tree_files_are_reported_with_the_tree_and_flag_errors() {
    // The relative path names no file in the directory of the test.
    let f = fail(&[HA, "(A,B"], &["missing-tree-file.nwk", "--gamma=-1"]);
    // The text of the operating system error differs between systems.
    let Err(os_error) = std::fs::read("missing-tree-file.nwk") else {
      panic!("missing-tree-file.nwk exists");
    };
    let expected = formatdoc! {r#"
      missing-tree-file.nwk: cannot read the file: {os_error}
      {}:1:5: tree "t1": Newick parse error: The tree does not end with ';' at byte 4
      gamma must be a non-negative number, got -1"#,
      f.dir.path().join("t1.nwk").display()
    };
    assert_failed(&f, &expected);
  }

  #[test]
  fn an_unexpected_failure_prints_its_chain_of_causes() {
    // `-o` names a regular file, so creating the results directory fails after validation.
    let dir = tempdir().unwrap();
    let paths = write_trees(dir.path(), &[("ha", HA), ("na", NA)]);
    let out = dir.path().join("out");
    std::fs::write(&out, "").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .args(&paths)
      .arg("-o")
      .arg(&out)
      .args(["--verbosity-level", "-1"])
      .env_remove("RUST_BACKTRACE")
      .env_remove("RUST_LIB_BACKTRACE")
      .output()
      .unwrap();
    // The text of the operating system error differs between systems.
    let Err(os_error) = std::fs::create_dir_all(&out) else {
      panic!("creating a directory over a file succeeded");
    };
    let expected = format!("Error: \n   0: creating {}\n   1: {os_error}\n", out.display());
    assert_eq!(
      (Some(1), expected),
      (output.status.code(), String::from_utf8(output.stderr).unwrap())
    );
  }

  #[test]
  fn a_missing_session_file_is_an_input_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("treeknit_session.json");
    let output = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .arg("--session")
      .arg(&path)
      .arg("-o")
      .arg(dir.path().join("out"))
      .args(["--verbosity-level", "-1"])
      .output()
      .unwrap();
    let Err(os_error) = std::fs::read(&path) else {
      panic!("the session file exists");
    };
    let expected = format!("Error: reading {}: {os_error}\n", path.display());
    assert_eq!(
      (Some(1), expected),
      (output.status.code(), String::from_utf8(output.stderr).unwrap())
    );
  }

  #[test]
  fn output_files_of_different_kinds_with_one_name_exit_before_writing_results() {
    // The resolved tree of `MCCs_a.dat` and the MCCs of the pair (`a`, `resolved`) are both
    // `MCCs_a_resolved.dat`.
    let dir = tempdir().unwrap();
    let t = "((A,B),(C,D));";
    let mut paths = write_trees(dir.path(), &[("a", t), ("resolved", t)]);
    let dat = dir.path().join("MCCs_a.dat");
    std::fs::write(&dat, t).unwrap();
    paths.push(dat);
    let out = dir.path().join("out");
    let output = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .args(&paths)
      .arg("-o")
      .arg(&out)
      .args(["--verbosity-level", "-1"])
      .output()
      .unwrap();
    assert_eq!(
      (
        Some(1),
        "Error: two output files are named \"MCCs_a_resolved.dat\"; rename a tree\n",
        false
      ),
      (
        output.status.code(),
        String::from_utf8(output.stderr).unwrap().as_str(),
        out.exists()
      )
    );
  }

  #[test]
  fn labels_that_stay_equal_with_their_directory_are_reported_with_the_flag_errors() {
    // `a/x/ha.nwk` and `b/x/ha.nwk` both get the label `ha_x`.
    let dir = tempdir().unwrap();
    let paths: Vec<PathBuf> = [("a/x", HA), ("b/x", NA)]
      .iter()
      .map(|(sub, newick)| {
        std::fs::create_dir_all(dir.path().join(sub)).unwrap();
        write_trees(&dir.path().join(sub), &[("ha", newick)]).remove(0)
      })
      .collect();
    let output = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .args(&paths)
      .args(["--gamma=-1", "-o"])
      .arg(dir.path().join("out"))
      .args(["--verbosity-level", "-1"])
      .output()
      .unwrap();
    let expected = formatdoc! {r#"
      Error: {}: tree label "ha_x" is used twice
      gamma must be a non-negative number, got -1
      "#,
      paths[1].display()
    };
    assert_eq!(
      (Some(1), expected),
      (output.status.code(), String::from_utf8(output.stderr).unwrap())
    );
  }

  #[test]
  fn tree_errors_name_the_input_file_and_position() {
    let f = fail(&[HA, "((A,B),\n(C,D)x y);", "((A,A),(C,D));"], &[]);
    let expected = formatdoc! {r#"
      {}:2:8: tree "t1": Newick parse error: expected comment, ')', ',', ':', ';' at byte 15
      {}: tree "t2": Newick parse error: duplicate leaf name "A""#,
      f.dir.path().join("t1.nwk").display(),
      f.dir.path().join("t2.nwk").display(),
    };
    assert_failed(&f, &expected);
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::disjoint_resolve(  "(P,(Q,R));", &["--resolve", "strict"])]
  #[case::one_shared_resolve("(A,(Q,R));", &["--resolve", "matched"])]
  #[case::disjoint_former(   "(P,(Q,R));", &["--better-trees"])]
  #[case::one_shared_former( "(A,(Q,R));", &["--better-MCCs"])]
  #[trace]
  fn pairs_sharing_fewer_than_two_leaves_exit_with_their_message(#[case] other: &str, #[case] args: &[&str]) {
    assert_failed(&fail(&[HA, other], args), "trees \"t0\" and \"t1\" share fewer than 2 leaves");
  }

  #[test]
  fn colliding_pair_names_exit_before_writing_results() {
    let t = "((A,B),(C,D));";
    let f = fail_named(&[("a_b", t), ("c", t), ("a", t), ("b_c", t)], &[]);
    assert_failed(
      &f,
      "tree pairs (\"a_b\", \"c\") and (\"a\", \"b_c\") give the same output file names (\"a_b_c\"); rename a tree",
    );
  }

  #[test]
  fn parse_warnings_are_logged() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("in");
    std::fs::create_dir_all(&input).unwrap();
    let ha = input.join("ha.nwk");
    let na = input.join("na.nwk");
    std::fs::write(&ha, format!("{HA}\n{NA}\n")).unwrap();
    std::fs::write(&na, "((A:1,(B:1,X:1):1):1,(C:1,D:1):1);\n(A,B);\n").unwrap();
    let out = dir.path().join("out");
    run(&[ha.to_str().unwrap(), na.to_str().unwrap()], &out);
    let log = std::fs::read_to_string(out.join("log.txt")).unwrap();
    assert!(
      log.contains("[WARN] ha: more than one tree in file, using the first\n"),
      "{log}"
    );
    assert!(
      log.contains("[WARN] na: more than one tree in file, using the first\n"),
      "{log}"
    );
  }

  #[test]
  fn several_trees_warning_is_logged_for_a_tree_with_a_duplicate_leaf() {
    let dir = tempdir().unwrap();
    let paths = write_trees(dir.path(), &[("ha", HA), ("na", "((A,A),B);\n(C,D);")]);
    let out = dir.path().join("out");
    let output = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .args(&paths)
      .arg("-o")
      .arg(&out)
      .env("NO_COLOR", "1")
      .output()
      .unwrap();
    // The log lines on stderr without their time; the error line has none.
    let lines: Vec<String> = String::from_utf8(output.stderr)
      .unwrap()
      .lines()
      .map(|l| {
        l.split_once(' ')
          .filter(|_| !l.starts_with("Error: "))
          .map_or(l, |(_, rest)| rest)
          .to_owned()
      })
      .collect();
    let expected = vec![
      format!("[INFO] TreeKnit {}", env!("TREEKNIT_LONG_VERSION")),
      format!("[INFO] input trees: {} {}", paths[0].display(), paths[1].display()),
      format!("[INFO] results directory: {}", out.display()),
      "[WARN] na: more than one tree in file, using the first".to_owned(),
      format!(
        "Error: {}: tree \"na\": Newick parse error: duplicate leaf name \"A\"",
        paths[1].display()
      ),
    ];
    assert_eq!((Some(1), expected), (output.status.code(), lines));
  }
}
