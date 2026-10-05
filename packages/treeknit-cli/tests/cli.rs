//! End-to-end runs of the `treeknit` binary.

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use std::collections::BTreeMap;
  use std::path::{Path, PathBuf};
  use std::process::Command;

  /// Directory under the system temporary directory, removed when dropped.
  struct TempDir(PathBuf);

  impl TempDir {
    fn new(name: &str) -> Self {
      let path = std::env::temp_dir().join(format!("treeknit-cli-test-{name}-{}", std::process::id()));
      if path.exists() {
        std::fs::remove_dir_all(&path).unwrap();
      }
      std::fs::create_dir_all(&path).unwrap();
      TempDir(path)
    }

    fn path(&self) -> &Path {
      &self.0
    }
  }

  impl Drop for TempDir {
    fn drop(&mut self) {
      if let Err(e) = std::fs::remove_dir_all(&self.0) {
        eprintln!("removing {}: {e}", self.0.display());
      }
    }
  }

  /// Write each `(name, newick)` of `files` to `<dir>/<name>.nwk`.
  fn write_trees(dir: &Path, files: &[(&str, &str)]) -> Vec<PathBuf> {
    files
      .iter()
      .map(|(name, newick)| {
        let p = dir.join(format!("{name}.nwk"));
        std::fs::write(&p, newick).unwrap();
        p
      })
      .collect()
  }

  fn run(args: &[&str], out: &Path) {
    let status = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .args(args)
      .arg("-o")
      .arg(out)
      .args(["--verbosity-level", "-1"])
      .status()
      .unwrap();
    assert!(status.success());
  }

  /// A failed run: its directory with the input files and the output directory `out`, the exit
  /// code, the error output, and whether any result file was written.
  struct Failure {
    dir: TempDir,
    code: Option<i32>,
    stderr: String,
    results: bool,
  }

  /// Run `treeknit` on Newick `trees` written to files `t0.nwk`, `t1.nwk`, ..., expecting a
  /// failure.
  fn fail(name: &str, trees: &[&str], args: &[&str]) -> Failure {
    let names: Vec<String> = (0..trees.len()).map(|i| format!("t{i}")).collect();
    let files: Vec<(&str, &str)> = names.iter().map(String::as_str).zip(trees.iter().copied()).collect();
    fail_named(name, &files, args)
  }

  /// Run `treeknit` on the `(name, newick)` trees of `files`, expecting a failure.
  fn fail_named(name: &str, files: &[(&str, &str)], args: &[&str]) -> Failure {
    let dir = TempDir::new(name);
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
    // The log is written from the start; result files only after validation.
    let results = std::fs::read_dir(&out).is_ok_and(|d| d.filter_map(Result::ok).any(|e| e.file_name() != "log.txt"));
    Failure {
      dir,
      code: output.status.code(),
      stderr: String::from_utf8(output.stderr).unwrap(),
      results,
    }
  }

  /// Assert that `f` exited with 1, printed exactly `message` as the error, and wrote no result.
  fn assert_failed(f: &Failure, message: &str, case: &str) {
    assert_eq!(
      (Some(1), format!("Error: {message}\n").as_str(), false),
      (f.code, f.stderr.as_str(), f.results),
      "{case}"
    );
  }

  const HA: &str = "((A:1,B:1):1,(C:1,(D:1,X:1):1):1);";
  const NA: &str = "((A:1,(B:1,X:1):1):1,(C:1,D:1):1);";

  #[rustfmt::skip]
  #[test]
  fn invalid_settings_exit_with_their_message() {
    for (name, args, message) in [
      ("gamma-negative", &["--gamma=-1"][..],            "gamma must be a non-negative number, got -1"),
      ("gamma-nan",      &["--gamma=nan"],               "gamma must be a non-negative number, got NaN"),
      ("lengths-zero",   &["--seq-lengths", "0 0"],      "sequence length 1 must be a positive number, got 0\nsequence length 2 must be a positive number, got 0"),
      ("lengths-former", &["--better-MCCs", "--seq-lengths", "0 0"], "sequence length 1 must be a positive number, got 0\nsequence length 2 must be a positive number, got 0"),
      ("seed-large",     &["--seed", "9007199254740992"], "seed must be at most 9007199254740991, got 9007199254740992"),
      ("mcmc-zero",      &["--n-mcmc-it", "0"],          "MCMC steps per leaf must be at least 1"),
      ("mcmc-former",    &["--better-MCCs", "--n-mcmc-it", "0"], "MCMC steps per leaf must be at least 1"),
      ("rounds-former",  &["--better-MCCs", "--rounds", "0"],    "rounds must be at least 1"),
    ] {
      assert_failed(&fail(name, &[HA, NA], args), message, name);
    }
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
    let expected = "--seq-lengths should look like \"1500 2000\", got \"x 1\": invalid float literal\n\
      former method options (--better-trees, --better-MCCs, --no-resolve, --liberal-resolve, \
      --resolve-all-rounds, --no-pre-resolve, --match-topologies) cannot be combined with \
      --resolve, --pre-resolve or --no-final-round; see --help-resolve\n\
      gamma must be a non-negative number, got -1";
    assert_failed(&fail("all-errors", &[HA, NA], &args), expected, "all errors");
  }

  #[test]
  fn tree_errors_name_the_input_file_and_position() {
    let f = fail("parse", &[HA, "((A,B),\n(C,D)x y);", "((A,A),(C,D));"], &[]);
    let expected = format!(
      "{}:2:8: tree \"t1\": Newick parse error: expected ',' or ')' at byte 15\n\
       {}: tree \"t2\": Newick parse error: duplicate leaf name A",
      f.dir.path().join("t1.nwk").display(),
      f.dir.path().join("t2.nwk").display(),
    );
    assert_failed(&f, &expected, "parse");
  }

  #[rustfmt::skip]
  #[test]
  fn pairs_sharing_fewer_than_two_leaves_exit_with_their_message() {
    for (name, other, args) in [
      ("disjoint-resolve",   "(P,(Q,R));", &["--resolve", "strict"][..]),
      ("one-shared-resolve", "(A,(Q,R));", &["--resolve", "matched"]),
      ("disjoint-former",    "(P,(Q,R));", &["--better-trees"]),
      ("one-shared-former",  "(A,(Q,R));", &["--better-MCCs"]),
    ] {
      assert_failed(&fail(name, &[HA, other], args), "trees \"t0\" and \"t1\" share fewer than 2 leaves", name);
    }
  }

  #[test]
  fn colliding_pair_names_exit_before_writing_results() {
    let t = "((A,B),(C,D));";
    let f = fail_named("stems", &[("a_b", t), ("c", t), ("a", t), ("b_c", t)], &[]);
    assert_failed(
      &f,
      "tree pairs (\"a_b\", \"c\") and (\"a\", \"b_c\") give the same output file names (\"a_b_c\"); rename a tree",
      "stems",
    );
  }

  #[test]
  fn parse_warnings_are_logged() {
    let dir = TempDir::new("warnings");
    let input = dir.path().join("in");
    std::fs::create_dir_all(&input).unwrap();
    let ha = input.join("ha.nwk");
    let na = input.join("na.nwk");
    std::fs::write(&ha, format!("{HA}\n{NA}\n")).unwrap();
    std::fs::write(&na, "((A:1,(B:1,X:1):1):x,(C:1,D:1):1);").unwrap();
    let out = dir.path().join("out");
    run(&[ha.to_str().unwrap(), na.to_str().unwrap()], &out);
    let log = std::fs::read_to_string(out.join("log.txt")).unwrap();
    assert!(
      log.contains("[WARN] ha: more than one tree in file, using the first\n"),
      "{log}"
    );
    assert!(log.contains("[WARN] ignoring invalid branch length 'x'\n"), "{log}");
  }

  #[test]
  fn help_resolve_uses_the_texts_of_the_settings_schema() {
    let output = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .arg("--help-resolve")
      .output()
      .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    let mut texts: Vec<String> = treeknit_io::schema::modes().into_iter().map(|m| m.effect).collect();
    texts.push(treeknit_io::schema::FINAL_ROUND_HELP.to_owned());
    texts.push(treeknit_io::schema::PRE_RESOLVE_HELP.to_owned());
    texts.push("  matched  (default) ".to_owned());
    texts.push("Former options are still accepted".to_owned());
    let missing: Vec<&String> = texts.iter().filter(|t| !help.contains(t.as_str())).collect();
    assert_eq!(Vec::<&String>::new(), missing, "{help}");
  }

  /// Every file under `dir` except `log.txt`, by its path relative to `dir` with `/` separators.
  fn files_below(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
      for entry in std::fs::read_dir(&d).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
          stack.push(path);
          continue;
        }
        let relative = path.strip_prefix(dir).unwrap().to_string_lossy().replace('\\', "/");
        if relative != "log.txt" {
          files.insert(relative, std::fs::read(&path).unwrap());
        }
      }
    }
    files
  }

  /// The text of each file, so a failure shows a line diff.
  fn readable(files: BTreeMap<String, Vec<u8>>) -> BTreeMap<String, String> {
    files
      .into_iter()
      .map(|(k, v)| (k, String::from_utf8(v).unwrap()))
      .collect()
  }

  #[rstest]
  #[case::two_trees("two", &["ha.nwk", "na.nwk"], &[])]
  #[case::three_trees_imputed_auspice("three", &["seg0.nwk", "seg1.nwk", "seg2.newick"], &["--impute", "--auspice-view"])]
  #[trace]
  fn output_files_keep_their_bytes(#[case] case: &str, #[case] inputs: &[&str], #[case] flags: &[&str]) {
    // Oracle: the output directories that the command line wrote before its writers moved to
    // `treeknit_io`, captured from the same inputs and flags, without `log.txt` (its lines carry
    // times).
    let data = Path::new(env!("CARGO_MANIFEST_DIR"))
      .join("tests/data/outputs")
      .join(case);
    let dir = TempDir::new(&format!("bytes-{case}"));
    let out = dir.path().join("out");
    let paths: Vec<String> = inputs
      .iter()
      .map(|f| data.join("input").join(f).to_string_lossy().into_owned())
      .collect();
    let args: Vec<&str> = paths.iter().map(String::as_str).chain(flags.iter().copied()).collect();
    run(&args, &out);
    assert_eq!(
      readable(files_below(&data.join("expected"))),
      readable(files_below(&out))
    );
  }

  #[test]
  fn files_with_one_stem_give_resolved_trees_named_by_label() {
    // Both files are named `ha.nwk`, so their labels get the directory: `ha_a` and `ha_b`.
    let dir = TempDir::new("stems-dirs");
    for (sub, newick) in [("a", HA), ("b", NA)] {
      std::fs::create_dir_all(dir.path().join(sub)).unwrap();
      write_trees(&dir.path().join(sub), &[("ha", newick)]);
    }
    let out = dir.path().join("out");
    let a = dir.path().join("a/ha.nwk");
    let b = dir.path().join("b/ha.nwk");
    run(&[a.to_str().unwrap(), b.to_str().unwrap()], &out);
    let resolved: Vec<bool> = ["ha_a_resolved.nwk", "ha_b_resolved.nwk", "ha_resolved.nwk"]
      .iter()
      .map(|f| out.join(f).exists())
      .collect();
    assert_eq!(vec![true, true, false], resolved);
  }

  fn mccs(out: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(out.join("MCCs.json")).unwrap()).unwrap()
  }

  #[test]
  fn two_trees_with_arg() {
    let ex = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../legacy_julia_version/examples");
    if !Path::new(ex).exists() {
      return;
    }
    let dir = TempDir::new("two");
    let out = dir.path().join("out");
    run(
      &[
        &format!("{ex}/tree_h3n2_ha.nwk"),
        &format!("{ex}/tree_h3n2_na.nwk"),
        "--auspice-view",
      ],
      &out,
    );
    for f in [
      "MCCs.json",
      "parameters.json",
      "log.txt",
      "tree_h3n2_ha_resolved.nwk",
      "auspice_tree_h3n2_na.json",
      "ARG/arg.nwk",
      "ARG/nodes.dat",
      "ARG/tree_h3n2_na_liberal_resolved.nwk",
    ] {
      assert!(out.join(f).exists(), "missing {f}");
    }
    let json = mccs(&out)["MCC_dict"]["1"]["mccs"].clone();
    let n = json.as_array().unwrap().len();
    // Legacy text output holds the same MCCs, one per line.
    let dat = std::fs::read_to_string(out.join("MCCs.dat")).unwrap();
    let from_dat: Vec<Vec<String>> = dat.lines().map(|l| l.split(',').map(String::from).collect()).collect();
    assert_eq!(serde_json::to_value(from_dat).unwrap(), json);
    let arg = std::fs::read_to_string(out.join("ARG/arg.nwk")).unwrap();
    assert_eq!(arg.matches("#H").count(), 2 * (n - 1));
  }

  #[test]
  fn three_trees_partial_overlap_imputed() {
    let dir = TempDir::new("three");
    let paths: Vec<String> = write_trees(
      dir.path(),
      &[
        ("seg0", "((A,B),(C,(D,(E,X))));"),
        ("seg1", "((A,(B,X)),(C,D,E,P));"),
        ("seg2", "((A,(B,P)),((C,D),(E,X)));"),
      ],
    )
    .iter()
    .map(|p| p.to_string_lossy().into_owned())
    .collect();
    let out = dir.path().join("out");
    let mut args: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    args.push("--impute");
    run(&args, &out);
    let m = mccs(&out);
    let d = m["MCC_dict"].as_object().unwrap();
    assert_eq!(d.len(), 3);
    for pair in ["seg0_seg1", "seg0_seg2", "seg1_seg2"] {
      assert!(out.join(format!("MCCs_{pair}.dat")).exists());
    }
    // P is missing from seg0: pairs (0,1) and (0,2) place it.
    assert!(d["1"]["imputed"].as_array().unwrap().iter().any(|e| e["leaf"] == "P"));
    let imputed = std::fs::read_to_string(out.join("seg0_imputed.nwk")).unwrap();
    assert!(imputed.contains('P'));
  }
}
