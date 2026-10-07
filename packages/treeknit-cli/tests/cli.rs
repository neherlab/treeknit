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
      let path = Self::path_of(name);
      if path.exists() {
        std::fs::remove_dir_all(&path).unwrap();
      }
      std::fs::create_dir_all(&path).unwrap();
      TempDir(path)
    }

    fn path(&self) -> &Path {
      &self.0
    }

    /// The path of the directory of `new(name)`.
    fn path_of(name: &str) -> PathBuf {
      std::env::temp_dir().join(format!("treeknit-cli-test-{name}-{}", std::process::id()))
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
  /// code, the error output, and whether the output directory was created.
  struct Failure {
    dir: TempDir,
    code: Option<i32>,
    stderr: String,
    created: bool,
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
    Failure {
      dir,
      code: output.status.code(),
      stderr: String::from_utf8(output.stderr).unwrap(),
      created: out.exists(),
    }
  }

  /// Assert that `f` exited with 1, printed exactly `message` as the error, and created no output
  /// directory.
  fn assert_failed(f: &Failure, message: &str, case: &str) {
    assert_eq!(
      (Some(1), format!("Error: {message}\n").as_str(), false),
      (f.code, f.stderr.as_str(), f.created),
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
    let f = fail("all-errors", &[HA, "(A,B"], &args);
    let expected = format!(
      "{}:1:5: tree \"t1\": Newick parse error: expected ',' or ')' at byte 4\n\
      --seq-lengths should look like 1500,2000, got \"x 1\": invalid float literal\n\
      former method options (--better-trees, --better-MCCs, --no-resolve, --liberal-resolve, \
      --resolve-all-rounds, --match-topologies) cannot be combined with --resolve, --pre-resolve, \
      --no-final-round or --final-round; see --help-resolve\n\
      gamma must be a non-negative number, got -1",
      f.dir.path().join("t1.nwk").display()
    );
    assert_failed(&f, &expected, "all errors");
  }

  #[test]
  fn too_few_tree_files_are_reported_with_the_flag_errors() {
    let f = fail("one-file", &[HA], &["--gamma=-1"]);
    let expected = "need at least two trees\ngamma must be a non-negative number, got -1";
    assert_failed(&f, expected, "one file");
  }

  #[test]
  fn unreadable_tree_files_are_reported_with_the_tree_and_flag_errors() {
    // The relative path names no file in the directory of the test.
    let f = fail("unreadable", &[HA, "(A,B"], &["missing-tree-file.nwk", "--gamma=-1"]);
    // The text of the operating system error differs between systems.
    let Err(os_error) = std::fs::read("missing-tree-file.nwk") else {
      panic!("missing-tree-file.nwk exists");
    };
    let expected = format!(
      "missing-tree-file.nwk: cannot read the file: {os_error}\n\
       {}:1:5: tree \"t1\": Newick parse error: expected ',' or ')' at byte 4\n\
       gamma must be a non-negative number, got -1",
      f.dir.path().join("t1.nwk").display()
    );
    assert_failed(&f, &expected, "unreadable");
  }

  #[test]
  fn an_unexpected_failure_prints_its_chain_of_causes() {
    // `-o` names a regular file, so creating the results directory fails after validation.
    let dir = TempDir::new("outdir-is-file");
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
    let dir = TempDir::new("missing-session");
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
    let dir = TempDir::new("output-names");
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
    let dir = TempDir::new("labels-equal");
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
    let expected = format!(
      "Error: {}: tree label \"ha_x\" is used twice\ngamma must be a non-negative number, got -1\n",
      paths[1].display()
    );
    assert_eq!(
      (Some(1), expected),
      (output.status.code(), String::from_utf8(output.stderr).unwrap())
    );
  }

  #[test]
  fn tree_errors_name_the_input_file_and_position() {
    let f = fail("parse", &[HA, "((A,B),\n(C,D)x y);", "((A,A),(C,D));"], &[]);
    let expected = format!(
      "{}:2:8: tree \"t1\": Newick parse error: expected ',' or ')' at byte 15\n\
       {}: tree \"t2\": Newick parse error: duplicate leaf name \"A\"",
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
    assert!(log.contains("[WARN] na: ignoring invalid branch length 'x'\n"), "{log}");
  }

  #[test]
  fn several_trees_warning_is_logged_for_a_tree_that_fails_to_parse() {
    let dir = TempDir::new("warnings-error");
    let paths = write_trees(dir.path(), &[("ha", HA), ("na", "((A,B;\n(C,D);")]);
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
        "Error: {}:1:6: tree \"na\": Newick parse error: expected ',' or ')' at byte 5",
        paths[1].display()
      ),
    ];
    assert_eq!((Some(1), expected), (output.status.code(), lines));
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
    // Oracle: the values that `--resolve` accepts, each at the start of its line.
    texts.push("\n  matched  (default) ".to_owned());
    texts.extend(["\n  strict   ", "\n  liberal  ", "\n  none     "].map(str::to_owned));
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
    // `treeknit_io`, without `log.txt` (its lines carry times): the `treeknit` binary of commit
    // 6db59a4, run in `tests/data/outputs/<case>` as `treeknit <inputs> <flags> -o expected`. The
    // MCCs of the two-tree case are those of the TreeKnit.jl fixture `doc_mccs_1.json`.
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
  fn session_of_the_two_tree_case_keeps_the_bytes_and_adds_the_session_file() {
    // Oracle: the captured output directory of `output_files_keep_their_bytes`, whose trees carry
    // the labels of their file stems.
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/outputs/two");
    let dir = TempDir::new("bytes-request");
    let tree = |name: &str| serde_json::json!({"label": name, "newick": std::fs::read_to_string(data.join(format!("input/{name}.nwk"))).unwrap()});
    let request = serde_json::json!({"trees": [tree("ha"), tree("na")]});
    let path = dir.path().join("treeknit_session.json");
    std::fs::write(&path, request.to_string()).unwrap();
    let out = dir.path().join("out");
    run(&["--session", path.to_str().unwrap()], &out);
    let mut written = readable(files_below(&out));
    let session: serde_json::Value = serde_json::from_str(&written.remove("treeknit_session.json").unwrap()).unwrap();
    assert_eq!(request["trees"], session["trees"]);
    assert_eq!(readable(files_below(&data.join("expected"))), written);
  }

  #[test]
  fn log_of_the_two_tree_case_holds_the_steps_of_the_run_in_order() {
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/outputs/two");
    let dir = TempDir::new("log-lines");
    let out = dir.path().join("out");
    let (ha, na) = (data.join("input/ha.nwk"), data.join("input/na.nwk"));
    run(&[ha.to_str().unwrap(), na.to_str().unwrap()], &out);
    let log = std::fs::read_to_string(out.join("log.txt")).unwrap();
    // The lines without their time, the info lines only (the debug lines carry thread numbers),
    // with the version and the runtime replaced by `_`.
    let lines: Vec<String> = log
      .lines()
      .map(|l| l.split_once(' ').unwrap().1)
      .filter(|l| l.starts_with("[INFO]"))
      .map(
        |l| match (l.starts_with("[INFO] TreeKnit "), l.split_once(" (runtime ")) {
          (true, _) => "[INFO] TreeKnit _".to_owned(),
          (false, Some((head, _))) => format!("{head} (runtime _)"),
          (false, None) => l.to_owned(),
        },
      )
      .collect();
    // Oracle: the steps of a two-tree run; the counts are those of the TreeKnit.jl fixture
    // `doc_mccs_1.json`: five shared leaves, the MCCs [X] and [A,B,C,D], one reassortment.
    let expected = vec![
      "[INFO] TreeKnit _".to_owned(),
      format!("[INFO] input trees: {} {}", ha.display(), na.display()),
      format!("[INFO] results directory: {}", out.display()),
      "[INFO] γ = 2, resolution: matched, pre-resolve: false, 1 round(s)".to_owned(),
      "[INFO] round 1/1 (resolving)".to_owned(),
      "[INFO] inferring MCCs for ha and na (5 shared leaves)".to_owned(),
      "[INFO] found 2 MCCs for ha and na".to_owned(),
      "[INFO] matched topologies within MCCs: 0 splits added, 0 MCCs split".to_owned(),
      "[INFO] found 1 reassortments in the ARG".to_owned(),
      "[INFO] found [2] MCCs (runtime _)".to_owned(),
      format!("[INFO] writing results in {}", out.display()),
    ];
    assert_eq!(expected, lines);
  }

  #[test]
  fn command_line_of_the_web_app_agrees_with_the_web_file_set() {
    // The command that the web app shows, run on its session file, writes every file of the web
    // file set with the same bytes, except `log.txt`, whose lines carry times, into a directory
    // of its own, and keeps the extracted files.
    let dir = TempDir::new("web-file-set");
    let results = dir.path().join(treeknit_io::output::RESULTS_DIR);
    std::fs::create_dir_all(&results).unwrap();
    let request = write_session(&results, &serde_json::json!({"seed": 3}));
    let command = treeknit_io::output::command_line();
    let args: Vec<&str> = command.split(' ').skip(1).collect();
    let out = dir.path().join("treeknit_results_cli");
    let status = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .args(&args)
      .args(["--verbosity-level", "-1"])
      .current_dir(dir.path())
      .status()
      .unwrap();
    assert!(status.success());
    let extracted: Vec<_> = std::fs::read_dir(&results)
      .unwrap()
      .map(|e| e.unwrap().file_name())
      .collect();
    assert_eq!(vec![std::ffi::OsString::from("treeknit_session.json")], extracted);
    let parsed = treeknit_io::analysis::read_session(&std::fs::read_to_string(&request).unwrap()).unwrap();
    let opts = treeknit_io::analysis::options(&parsed.settings, 2, true).unwrap();
    let texts = treeknit_io::analysis::parse_trees(&parsed.trees).unwrap();
    let run = treeknit_io::run::run(texts, &opts, parsed.settings.seed, &|_| {});
    let expected: BTreeMap<String, String> = treeknit_io::output::web_files(&parsed, &run, parsed.settings.seed, &[])
      .unwrap()
      .into_iter()
      .map(|f| match f {
        treeknit_io::output::WebFile::Text(f) => (f.path, f.text),
        treeknit_io::output::WebFile::Figure(f) => {
          let text = treeknit_io::output::figure_text(&run, f.figure).unwrap();
          (f.path, text)
        },
      })
      .filter(|(path, _)| path != "log.txt")
      .collect();
    assert_eq!(expected, readable(files_below(&out)));
  }

  /// The file at `path` is an SVG figure; the command line writes the extension in lowercase.
  fn is_figure(path: &str) -> bool {
    Path::new(path).extension().is_some_and(|e| e == "svg")
  }

  #[test]
  fn plot_adds_the_figures_and_keeps_the_bytes_of_the_other_files() {
    // Oracle: the captured output directory of `output_files_keep_their_bytes`.
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/outputs/two");
    let dir = TempDir::new("plot-bytes");
    let out = dir.path().join("out");
    let ha = data.join("input/ha.nwk");
    let na = data.join("input/na.nwk");
    run(&[ha.to_str().unwrap(), na.to_str().unwrap(), "--plot"], &out);
    let mut written = readable(files_below(&out));
    let figures: Vec<String> = written.keys().filter(|p| is_figure(p)).cloned().collect();
    assert_eq!(
      vec!["ARG/arg.svg".to_owned(), "tanglegram_ha_na.svg".to_owned()],
      figures
    );
    for path in &figures {
      let svg = written.remove(path).unwrap();
      assert!(
        svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\"") && svg.ends_with("</svg>\n"),
        "{path}"
      );
    }
    assert_eq!(readable(files_below(&data.join("expected"))), written);
  }

  #[test]
  fn plot_writes_a_tanglegram_per_pair_of_three_trees_and_no_arg() {
    let dir = TempDir::new("plot-three");
    let t = "((A:1,B:1):1,(C:1,D:1):1);";
    let paths = write_trees(dir.path(), &[("ha", t), ("na", t), ("pb2", t)]);
    let out = dir.path().join("out");
    let args: Vec<&str> = paths.iter().map(|p| p.to_str().unwrap()).chain(["--plot"]).collect();
    run(&args, &out);
    let figures: Vec<String> = files_below(&out).into_keys().filter(|p| is_figure(p)).collect();
    let expected = vec!["tanglegram_ha_na.svg", "tanglegram_ha_pb2.svg", "tanglegram_na_pb2.svg"];
    assert_eq!(expected, figures);
  }

  #[test]
  fn session_with_plot_gives_the_figures_of_the_tree_files() {
    let dir = TempDir::new("plot-request");
    let request = write_session(dir.path(), &serde_json::json!({}));
    let from_session = dir.path().join("from-session");
    run(&["--session", request.to_str().unwrap(), "--plot"], &from_session);
    let paths = write_trees(dir.path(), &[("ha", HA), ("na", NA)]);
    let from_files = dir.path().join("from-files");
    run(
      &[paths[0].to_str().unwrap(), paths[1].to_str().unwrap(), "--plot"],
      &from_files,
    );
    let figures = |out: &Path| -> BTreeMap<String, String> {
      readable(files_below(out))
        .into_iter()
        .filter(|(p, _)| is_figure(p))
        .collect()
    };
    let expected = figures(&from_files);
    assert_eq!(2, expected.len());
    assert_eq!(expected, figures(&from_session));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::same_stem(      "stems-dirs", ("ha", "ha"), ["ha_a_resolved.nwk", "ha_b_resolved.nwk"], "ha_resolved.nwk")]
  #[case::stems_in_case(  "stems-case", ("HA", "ha"), ["HA_a_resolved.nwk", "ha_b_resolved.nwk"], "HA_resolved.nwk")]
  #[trace]
  fn files_with_one_stem_give_resolved_trees_named_by_label(
    #[case] name: &str,
    #[case] (stem_a, stem_b): (&str, &str),
    #[case] written: [&str; 2],
    #[case] absent: &str,
  ) {
    // Stems that are equal, or equal ignoring case as in the label check, give each label its
    // directory: `a/HA.nwk` and `b/ha.nwk` are `HA_a` and `ha_b`.
    let dir = TempDir::new(name);
    for (sub, stem, newick) in [("a", stem_a, HA), ("b", stem_b, NA)] {
      std::fs::create_dir_all(dir.path().join(sub)).unwrap();
      write_trees(&dir.path().join(sub), &[(stem, newick)]);
    }
    let out = dir.path().join("out");
    let a = dir.path().join(format!("a/{stem_a}.nwk"));
    let b = dir.path().join(format!("b/{stem_b}.nwk"));
    run(&[a.to_str().unwrap(), b.to_str().unwrap()], &out);
    let present: Vec<bool> = [written[0], written[1], absent].iter().map(|f| out.join(f).exists()).collect();
    assert_eq!(vec![true, true, false], present);
  }

  /// Write a session file with the trees `HA` and `NA`, labeled `ha` and `na`, and `settings`.
  fn write_session(dir: &Path, settings: &serde_json::Value) -> PathBuf {
    let request = serde_json::json!({
      "trees": [{"label": "ha", "newick": HA}, {"label": "na", "newick": NA}],
      "settings": settings,
    });
    let path = dir.join("treeknit_session.json");
    std::fs::write(&path, request.to_string()).unwrap();
    path
  }

  #[test]
  fn session_gives_the_mccs_of_the_tree_files_with_the_same_settings() {
    let dir = TempDir::new("session-same");
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
    let dir = TempDir::new("session-files");
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

  /// Path of the session file of `run_session` for the test `name`.
  fn session_path(name: &str) -> PathBuf {
    TempDir::path_of(name).join("treeknit_session.json")
  }

  /// Run `treeknit` with `args` in the directory of a session file, returning the exit code and
  /// the error output.
  fn run_session(name: &str, settings: &serde_json::Value, args: &[&str]) -> (Option<i32>, String) {
    let dir = TempDir::new(name);
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
    (output.status.code(), String::from_utf8(output.stderr).unwrap())
  }

  #[rstest]
  #[case::tree_file("session-tree", &["ha.nwk"], "the argument '--session <FILE>' cannot be used with '[TREE]...'")]
  #[case::example("session-example", &["--example", "5-leaves"], "the argument '--session <FILE>' cannot be used with '--example <ID>'")]
  #[case::former("session-former", &["--better-MCCs"], "the argument '--session <FILE>' cannot be used with '--better-MCCs'")]
  #[trace]
  fn session_with_tree_files_or_former_options_is_a_usage_error(
    #[case] name: &str,
    #[case] args: &[&str],
    #[case] message: &str,
  ) {
    let (code, stderr) = run_session(name, &serde_json::json!({}), args);
    // clap shows the usage of the arguments given, `-o` and `--verbosity-level` of `run_session`.
    let expected = format!(
      "error: {message}\n\n\
       Usage: treeknit --session <FILE> --outdir <OUTDIR> --verbosity-level <VERBOSITY_LEVEL> [TREE]...\n\n\
       For more information, try '--help'.\n"
    );
    assert_eq!((Some(2), expected), (code, stderr));
  }

  #[test]
  fn session_with_invalid_settings_exits_with_their_message_and_field() {
    let (code, stderr) = run_session("session-invalid", &serde_json::json!({"gamma": -1}), &[]);
    let request = session_path("session-invalid");
    let expected = format!(
      "Error: {}: settings.gamma: gamma must be a non-negative number, got -1\n",
      request.display()
    );
    assert_eq!((Some(1), expected), (code, stderr));
  }

  #[test]
  fn session_with_a_seed_above_the_largest_exact_integer_names_the_field() {
    let seed = treeknit_io::analysis::MAX_SEED + 1;
    let (code, stderr) = run_session("session-seed", &serde_json::json!({ "seed": seed }), &[]);
    let expected = format!(
      "Error: {}: settings.seed: the seed {seed} of the session file is above 9007199254740991, the \
       largest integer a JavaScript number holds exactly\n",
      session_path("session-seed").display()
    );
    assert_eq!((Some(1), expected), (code, stderr));
  }

  #[test]
  fn session_with_an_invalid_tree_names_the_file_the_field_and_the_position() {
    let name = "session-tree-error";
    let dir = TempDir::new(name);
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
      "Error: {}: trees[1].newick: line 2, column 8: tree \"na\": Newick parse error: expected ',' or ')' at byte 15\n",
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
    let (code, stderr) = run_session("session-structure", &serde_json::json!({"gama": 1}), &[]);
    let expected = format!(
      "Error: {}: not a TreeKnit session file: unknown field `gama`, expected one of `gamma`, `seqLengths`, \
       `nMcmcIt`, `resolve`, `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`, `seed` at line 1 column 151\n",
      session_path("session-structure").display()
    );
    assert_eq!((Some(1), expected), (code, stderr));
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

  /// Run `treeknit` with `args` and the output directory `out`, expecting success, and return its
  /// standard output.
  fn run_stdout(args: &[&str], out: &Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .args(args)
      .arg("-o")
      .arg(out)
      .args(["--verbosity-level", "-1"])
      .output()
      .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap()
  }

  fn parameters(out: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(out.join("parameters.json")).unwrap()).unwrap()
  }

  fn text(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
  }

  #[test]
  fn labeled_tree_arguments_name_the_output_trees() {
    let dir = TempDir::new("labeled");
    let paths = write_trees(dir.path(), &[("seg4", HA), ("a=b", NA)]);
    let out = dir.path().join("out");
    // The second path holds a `/` before its `=`, so it is a path without a label.
    run(
      &[&format!("HA={}", paths[0].display()), paths[1].to_str().unwrap()],
      &out,
    );
    let present = ["HA_resolved.nwk", "a=b_resolved.nwk"].map(|f| out.join(f).exists());
    assert_eq!([true, true], present);
  }

  #[rstest]
  #[case::commas("lengths-commas", "1701,1410")]
  #[case::spaces("lengths-spaces", "1701 1410")]
  #[trace]
  fn sequence_lengths_take_commas_or_spaces(#[case] name: &str, #[case] lengths: &str) {
    let dir = TempDir::new(name);
    let paths = write_trees(dir.path(), &[("ha", HA), ("na", NA)]);
    let out = dir.path().join("out");
    run(
      &[
        paths[0].to_str().unwrap(),
        paths[1].to_str().unwrap(),
        "--seq-lengths",
        lengths,
      ],
      &out,
    );
    assert_eq!(serde_json::json!([1701.0, 1410.0]), parameters(&out)["seq_lengths"]);
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::pre_resolve(    "pair-1", &["--no-pre-resolve", "--pre-resolve"],    "pre_resolve",            true)]
  #[case::no_pre_resolve( "pair-2", &["--pre-resolve", "--no-pre-resolve"],    "pre_resolve",            false)]
  #[case::final_round(    "pair-3", &["--no-final-round", "--final-round"],    "final_unresolved_round", true)]
  #[case::no_final_round( "pair-4", &["--final-round", "--no-final-round"],    "final_unresolved_round", false)]
  #[case::likelihood(     "pair-5", &["--no-likelihood", "--likelihood"],      "likelihood_sort",        true)]
  #[case::no_likelihood(  "pair-6", &["--likelihood", "--no-likelihood"],      "likelihood_sort",        false)]
  #[case::naive(          "pair-7", &["--no-naive", "--naive"],                "naive",                  true)]
  #[case::no_naive(       "pair-8", &["--naive", "--no-naive"],                "naive",                  false)]
  #[trace]
  fn the_last_of_a_flag_and_its_opposite_wins(#[case] name: &str, #[case] flags: &[&str], #[case] key: &str, #[case] expected: bool) {
    let dir = TempDir::new(name);
    let paths = write_trees(dir.path(), &[("ha", HA), ("na", NA)]);
    let out = dir.path().join("out");
    let mut args = vec![paths[0].to_str().unwrap(), paths[1].to_str().unwrap()];
    args.extend(flags);
    run(&args, &out);
    assert_eq!(serde_json::json!(expected), parameters(&out)[key]);
  }

  #[test]
  fn no_pre_resolve_is_a_current_option_without_a_warning() {
    let dir = TempDir::new("no-pre-resolve");
    let paths = write_trees(dir.path(), &[("ha", HA), ("na", NA)]);
    let out = dir.path().join("out");
    run(
      &[
        paths[0].to_str().unwrap(),
        paths[1].to_str().unwrap(),
        "--no-pre-resolve",
      ],
      &out,
    );
    assert_eq!(
      (serde_json::json!(false), false),
      (
        parameters(&out)["pre_resolve"].clone(),
        text(&out.join("log.txt")).contains("deprecated")
      )
    );
  }

  #[test]
  fn example_gives_the_mccs_of_its_tree_files() {
    let dir = TempDir::new("example");
    let from_example = dir.path().join("example");
    run(&["--example", "h3n2-2017"], &from_example);
    let data = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/h3n2-2017");
    let from_files = dir.path().join("files");
    run(&[&format!("{data}/ha.nwk"), &format!("{data}/na.nwk")], &from_files);
    assert_eq!(mccs(&from_files), mccs(&from_example));
  }

  #[test]
  fn unknown_example_suggests_the_closest_id() {
    let dir = TempDir::new("example-unknown");
    let output = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .args(["--example", "h3n2-2071", "-o"])
      .arg(dir.path().join("out"))
      .args(["--verbosity-level", "-1"])
      .output()
      .unwrap();
    let expected = "Error: there is no example \"h3n2-2071\"; did you mean \"h3n2-2017\"? (see --list-examples)\n";
    assert_eq!(
      (Some(1), expected),
      (output.status.code(), String::from_utf8(output.stderr).unwrap().as_str())
    );
  }

  #[test]
  fn list_examples_prints_every_id() {
    let output = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .arg("--list-examples")
      .output()
      .unwrap();
    let ids: Vec<String> = String::from_utf8(output.stdout)
      .unwrap()
      .lines()
      .filter_map(|l| l.split_whitespace().next().map(str::to_owned))
      .collect();
    let expected: Vec<String> = treeknit_io::examples::EXAMPLES
      .iter()
      .map(|e| e.id.to_owned())
      .collect();
    assert_eq!(expected, ids);
  }

  #[test]
  fn analysis_options_change_the_settings_of_a_session_file() {
    let dir = TempDir::new("session-options");
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

  #[test]
  fn printed_link_of_an_example_names_it_and_runs_the_same_analysis() {
    let dir = TempDir::new("print-example");
    let first = dir.path().join("first");
    let link = run_stdout(&["--example", "5-leaves", "--gamma", "3", "--print-link"], &first);
    assert_eq!(
      "https://neherlab.github.io/treeknit-rs/?example=5-leaves&gamma=3&run\n",
      link
    );
    let second = dir.path().join("second");
    run(&["--link", link.trim()], &second);
    assert_eq!(mccs(&first), mccs(&second));
  }

  #[test]
  fn printed_link_of_tree_files_holds_an_inline_session_that_runs_the_same_analysis() {
    let dir = TempDir::new("print-files");
    let paths = write_trees(dir.path(), &[("ha", HA), ("na", NA)]);
    let first = dir.path().join("first");
    let link = run_stdout(
      &[
        paths[0].to_str().unwrap(),
        paths[1].to_str().unwrap(),
        "--seed",
        "9",
        "--print-link",
      ],
      &first,
    );
    let prefix = "https://neherlab.github.io/treeknit-rs/?run#session=data:application/gzip;base64,";
    assert!(link.starts_with(prefix), "{link}");
    let second = dir.path().join("second");
    run(&["--link", link.trim()], &second);
    let written = ["first", "second"].map(|d| dir.path().join(d).join("treeknit_session.json").exists());
    assert_eq!((mccs(&first), [true, true]), (mccs(&second), written));
  }

  #[test]
  fn link_with_data_trees_runs_them_with_their_labels() {
    let dir = TempDir::new("link-data");
    let out = dir.path().join("out");
    let link = "https://neherlab.github.io/treeknit-rs/?tree=HA=data:,((A,B),(C,(D,X)));&tree=NA=data:,((A,(B,X)),(C,D));&run&view=mccs";
    run(&["--link", link], &out);
    let present = ["HA_resolved.nwk", "NA_resolved.nwk"].map(|f| out.join(f).exists());
    assert_eq!([true, true], present);
  }
}
