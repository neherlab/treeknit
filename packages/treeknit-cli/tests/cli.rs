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
      --seq-lengths should look like \"1500 2000\", got \"x 1\": invalid float literal\n\
      former method options (--better-trees, --better-MCCs, --no-resolve, --liberal-resolve, \
      --resolve-all-rounds, --no-pre-resolve, --match-topologies) cannot be combined with \
      --resolve, --pre-resolve or --no-final-round; see --help-resolve\n\
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
    let (code, lines) = (f.code, f.stderr.lines().collect::<Vec<_>>());
    // The text of the operating system error differs between systems; the rest is fixed.
    let read_error = "Error: missing-tree-file.nwk: cannot read the file: ";
    assert_eq!(
      (Some(1), true, false),
      (code, lines[0].starts_with(read_error), f.created),
      "{}",
      f.stderr
    );
    let expected = vec![
      format!(
        "{}:1:5: tree \"t1\": Newick parse error: expected ',' or ')' at byte 4",
        f.dir.path().join("t1.nwk").display()
      ),
      "gamma must be a non-negative number, got -1".to_owned(),
    ];
    assert_eq!(expected, lines[1..]);
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
    let output = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .args(&paths)
      .arg("-o")
      .arg(dir.path().join("out"))
      .output()
      .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(
      (Some(1), true),
      (
        output.status.code(),
        stderr.contains("na: more than one tree in file, using the first\n")
      ),
      "{stderr}"
    );
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
  fn request_of_the_two_tree_case_keeps_the_bytes_and_adds_the_session_file() {
    // Oracle: the captured output directory of `output_files_keep_their_bytes`, whose trees carry
    // the labels of their file stems.
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/outputs/two");
    let dir = TempDir::new("bytes-request");
    let tree = |name: &str| serde_json::json!({"label": name, "newick": std::fs::read_to_string(data.join(format!("input/{name}.nwk"))).unwrap()});
    let request = serde_json::json!({"trees": [tree("ha"), tree("na")]});
    let path = dir.path().join("treeknit_request.json");
    std::fs::write(&path, request.to_string()).unwrap();
    let out = dir.path().join("out");
    run(&["--request", path.to_str().unwrap()], &out);
    let mut written = readable(files_below(&out));
    let session: serde_json::Value = serde_json::from_str(&written.remove("treeknit_request.json").unwrap()).unwrap();
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
  fn command_line_of_the_web_app_writes_the_web_file_set() {
    // The command that the web app shows, run on its session file, writes every file of the web
    // file set with the same bytes, except `log.txt`, whose lines carry times.
    let dir = TempDir::new("web-file-set");
    let results = dir.path().join(treeknit_io::output::RESULTS_DIR);
    std::fs::create_dir_all(&results).unwrap();
    let request = write_request(&results, &serde_json::json!({"seed": 3}));
    let command = treeknit_io::output::command_line();
    let args: Vec<&str> = command.split(' ').skip(1).collect();
    let out = dir.path().join("out");
    let status = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .args(&args)
      .arg("-o")
      .arg(&out)
      .args(["--verbosity-level", "-1"])
      .current_dir(dir.path())
      .status()
      .unwrap();
    assert!(status.success());
    let parsed = treeknit_io::analysis::read_request(&std::fs::read_to_string(&request).unwrap()).unwrap();
    let opts = treeknit_io::analysis::options(&parsed.settings, 2, true).unwrap();
    let texts = treeknit_io::analysis::parse_trees(&parsed.trees).unwrap();
    let run = treeknit_io::run::run(texts, &opts, parsed.settings.seed, &|_| {});
    let expected: BTreeMap<String, String> = treeknit_io::output::web_files(&parsed, &run, parsed.settings.seed, &[])
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
  fn request_with_plot_gives_the_figures_of_the_tree_files() {
    let dir = TempDir::new("plot-request");
    let request = write_request(dir.path(), &serde_json::json!({}));
    let from_request = dir.path().join("from-request");
    run(&["--request", request.to_str().unwrap(), "--plot"], &from_request);
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
    assert_eq!(expected, figures(&from_request));
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
  fn write_request(dir: &Path, settings: &serde_json::Value) -> PathBuf {
    let request = serde_json::json!({
      "trees": [{"label": "ha", "newick": HA}, {"label": "na", "newick": NA}],
      "settings": settings,
    });
    let path = dir.join("treeknit_request.json");
    std::fs::write(&path, request.to_string()).unwrap();
    path
  }

  #[test]
  fn request_gives_the_mccs_of_the_tree_files_with_the_same_settings() {
    let dir = TempDir::new("request-same");
    let request = write_request(
      dir.path(),
      &serde_json::json!({"gamma": 3, "resolve": "strict", "seed": 5}),
    );
    let from_request = dir.path().join("from-request");
    run(&["--request", request.to_str().unwrap()], &from_request);
    let paths = write_trees(dir.path(), &[("ha", HA), ("na", NA)]);
    let from_files = dir.path().join("from-files");
    let (a, b) = (paths[0].to_str().unwrap(), paths[1].to_str().unwrap());
    run(
      &[a, b, "--gamma", "3", "--resolve", "strict", "--seed", "5"],
      &from_files,
    );
    assert_eq!(mccs(&from_files), mccs(&from_request));
  }

  #[test]
  fn request_writes_the_session_file_and_trees_named_by_label() {
    let dir = TempDir::new("request-files");
    let request = write_request(dir.path(), &serde_json::json!({}));
    let out = dir.path().join("out");
    run(&["--request", request.to_str().unwrap(), "--impute"], &out);
    let written: serde_json::Value =
      serde_json::from_str(&std::fs::read_to_string(out.join("treeknit_request.json")).unwrap()).unwrap();
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

  /// Path of the session file of `run_request` for the test `name`.
  fn request_path(name: &str) -> PathBuf {
    TempDir::path_of(name).join("treeknit_request.json")
  }

  /// Run `treeknit` with `args` in the directory of a session file, returning the exit code and
  /// the error output.
  fn run_request(name: &str, settings: &serde_json::Value, args: &[&str]) -> (Option<i32>, String) {
    let dir = TempDir::new(name);
    let request = write_request(dir.path(), settings);
    let output = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .arg("--request")
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
  #[case::tree_file("request-tree", &["ha.nwk"], "the argument '--request <FILE>' cannot be used with '[TREES]...'")]
  #[case::gamma("request-gamma", &["--gamma", "3"], "the argument '--request <FILE>' cannot be used with '--gamma <GAMMA>'")]
  #[case::former("request-former", &["--better-MCCs"], "the argument '--request <FILE>' cannot be used with '--better-MCCs'")]
  #[trace]
  fn request_with_tree_files_or_analysis_options_is_a_usage_error(
    #[case] name: &str,
    #[case] args: &[&str],
    #[case] message: &str,
  ) {
    let (code, stderr) = run_request(name, &serde_json::json!({}), args);
    assert_eq!((Some(2), true), (code, stderr.contains(message)), "{stderr}");
  }

  #[test]
  fn request_with_invalid_settings_exits_with_their_message_and_field() {
    let (code, stderr) = run_request("request-invalid", &serde_json::json!({"gamma": -1}), &[]);
    let request = request_path("request-invalid");
    let expected = format!(
      "Error: {}: settings.gamma: gamma must be a non-negative number, got -1\n",
      request.display()
    );
    assert_eq!((Some(1), expected), (code, stderr));
  }

  #[test]
  fn request_with_an_invalid_tree_names_the_file_the_field_and_the_position() {
    let name = "request-tree-error";
    let dir = TempDir::new(name);
    let request = serde_json::json!({
      "trees": [{"label": "ha", "newick": HA}, {"label": "na", "newick": "((A,B),\n(C,D)x y);"}],
    });
    let path = dir.path().join("treeknit_request.json");
    std::fs::write(&path, request.to_string()).unwrap();
    let out = dir.path().join("out");
    let output = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .arg("--request")
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
  fn request_with_a_wrong_structure_names_the_file() {
    let (code, stderr) = run_request("request-structure", &serde_json::json!({"gama": 1}), &[]);
    assert_eq!(
      (Some(1), true),
      (
        code,
        stderr.contains("treeknit_request.json: not a TreeKnit session file: unknown field `gama`")
      ),
      "{stderr}"
    );
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
