//! The output files of a run: their bytes, the log, the figures, and the ARG.

#[cfg(test)]
mod tests {
  use crate::tests::{HA, NA, mccs, run, write_session, write_trees};

  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use std::collections::BTreeMap;
  use std::path::Path;
  use std::process::Command;
  use tempfile::tempdir;

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

  #[rustfmt::skip]
  #[rstest]
  #[case::two_trees(                  "two", &["ha.nwk", "na.nwk"], &[])]
  #[case::three_trees_imputed_auspice("three", &["seg0.nwk", "seg1.nwk", "seg2.newick"], &["--impute", "--auspice-view"])]
  #[trace]
  fn output_files_keep_their_bytes(#[case] case: &str, #[case] inputs: &[&str], #[case] flags: &[&str]) {
    // Oracle: the output directories that the command line wrote before its writers moved to
    // `treeknit_io`, without `log.txt` (its lines carry times): the `treeknit` binary of commit
    // 6db59a4, run in `tests/data/outputs/<case>` as `treeknit <inputs> <flags> -o expected`. The
    // MCCs of the two-tree case are those of the TreeKnit.jl fixture `doc_mccs_1.json`. The
    // `auspice_<label>.json` files hold the captured JSON values, keys in the same order, written
    // compact as TreeKnit.jl writes them.
    let data = Path::new(env!("CARGO_MANIFEST_DIR"))
      .join("tests/data/outputs")
      .join(case);
    let dir = tempdir().unwrap();
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
    let dir = tempdir().unwrap();
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
    let dir = tempdir().unwrap();
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
    let dir = tempdir().unwrap();
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
    let dir = tempdir().unwrap();
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
    let not_svg: Vec<&String> = figures
      .iter()
      .filter(|path| {
        let svg = written.remove(*path).unwrap();
        !(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\"") && svg.ends_with("</svg>\n"))
      })
      .collect();
    assert_eq!(Vec::<&String>::new(), not_svg);
    assert_eq!(readable(files_below(&data.join("expected"))), written);
  }

  #[test]
  fn plot_writes_a_tanglegram_per_pair_of_three_trees_and_no_arg() {
    let dir = tempdir().unwrap();
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
    let dir = tempdir().unwrap();
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
  #[case::same_stem(    ("ha", "ha"), ["ha_a_resolved.nwk", "ha_b_resolved.nwk"], "ha_resolved.nwk")]
  #[case::stems_in_case(("HA", "ha"), ["HA_a_resolved.nwk", "ha_b_resolved.nwk"], "HA_resolved.nwk")]
  #[trace]
  fn files_with_one_stem_give_resolved_trees_named_by_label(
    #[case] (stem_a, stem_b): (&str, &str),
    #[case] written: [&str; 2],
    #[case] absent: &str,
  ) {
    // Stems that are equal, or equal ignoring case as in the label check, give each label its
    // directory: `a/HA.nwk` and `b/ha.nwk` are `HA_a` and `ha_b`.
    let dir = tempdir().unwrap();
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

  #[test]
  fn two_trees_with_arg() {
    // The H3N2 trees of the TreeKnit.jl examples.
    let data = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/h3n2-2017");
    let dir = tempdir().unwrap();
    let out = dir.path().join("out");
    run(
      &[&format!("{data}/ha.nwk"), &format!("{data}/na.nwk"), "--auspice-view"],
      &out,
    );
    let missing: Vec<&str> = [
      "MCCs.json",
      "parameters.json",
      "log.txt",
      "ha_resolved.nwk",
      "auspice_na.json",
      "ARG/arg.nwk",
      "ARG/nodes.dat",
      "ARG/na_liberal_resolved.nwk",
    ]
    .into_iter()
    .filter(|f| !out.join(f).exists())
    .collect();
    assert_eq!(Vec::<&str>::new(), missing);
    let json = mccs(&out)["MCC_dict"]["1"]["mccs"].clone();
    let n = json.as_array().unwrap().len();
    // Legacy text output holds the same MCCs, one per line.
    let dat = std::fs::read_to_string(out.join("MCCs.dat")).unwrap();
    let from_dat: Vec<Vec<String>> = dat.lines().map(|l| l.split(',').map(String::from).collect()).collect();
    assert_eq!(json, serde_json::to_value(from_dat).unwrap());
    // Each reassortment of the ARG has two `#H` nodes, one per segment, and there is one per MCC
    // beyond the first.
    let arg = std::fs::read_to_string(out.join("ARG/arg.nwk")).unwrap();
    assert_eq!(2 * (n - 1), arg.matches("#H").count());
  }

  #[test]
  fn three_trees_partial_overlap_imputed() {
    let dir = tempdir().unwrap();
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
    let pair_files: Vec<bool> = ["seg0_seg1", "seg0_seg2", "seg1_seg2"]
      .iter()
      .map(|pair| out.join(format!("MCCs_{pair}.dat")).exists())
      .collect();
    assert_eq!(vec![true, true, true], pair_files);
    // P is missing from seg0: pairs (0,1) and (0,2) place it.
    assert!(d["1"]["imputed"].as_array().unwrap().iter().any(|e| e["leaf"] == "P"));
    let imputed = std::fs::read_to_string(out.join("seg0_imputed.nwk")).unwrap();
    assert!(imputed.contains('P'));
  }
}
