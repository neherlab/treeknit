//! The help texts, the analysis options, the examples, and the links of the web app.

#[cfg(test)]
mod tests {
  use crate::tests::{HA, NA, mccs, parameters, run, text, write_trees};

  use pretty_assertions::assert_eq;
  use rstest::rstest;

  use std::path::Path;
  use std::process::Command;
  use tempfile::tempdir;

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

  #[test]
  fn help_markdown_holds_the_help_texts_and_a_row_per_output_file() {
    let stdout = |arg: &str| {
      let output = Command::new(env!("CARGO_BIN_EXE_treeknit")).arg(arg).output().unwrap();
      assert!(output.status.success(), "{arg}");
      String::from_utf8(output.stdout).unwrap()
    };
    let markdown = stdout("--help-markdown");
    // Oracle: the texts that the binary prints for `--help` and `--help-resolve`.
    let help = stdout("--help");
    let resolve = stdout("--help-resolve");
    let rows: Vec<&str> = markdown.lines().filter(|l| l.starts_with("| `")).collect();
    let expected_rows: Vec<String> = treeknit_io::output::file_reference()
      .into_iter()
      .map(|f| f.paths.iter().map(|p| format!("`{p}`")).collect::<Vec<_>>().join(", "))
      .collect();
    let actual_rows: Vec<String> = rows
      .iter()
      .map(|r| r.trim_start_matches("| ").split(" | ").next().unwrap().to_owned())
      .collect();
    assert_eq!(
      (true, true, expected_rows),
      (
        markdown.contains(help.trim_end()),
        markdown.contains(resolve.trim_end()),
        actual_rows
      )
    );
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

  #[test]
  fn labeled_tree_arguments_name_the_output_trees() {
    let dir = tempdir().unwrap();
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

  #[rustfmt::skip]
  #[rstest]
  #[case::commas("1701,1410")]
  #[case::spaces("1701 1410")]
  #[trace]
  fn sequence_lengths_take_commas_or_spaces(#[case] lengths: &str) {
    let dir = tempdir().unwrap();
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
  #[case::pre_resolve(   &["--no-pre-resolve", "--pre-resolve"],    "pre_resolve",            true)]
  #[case::no_pre_resolve(&["--pre-resolve", "--no-pre-resolve"],    "pre_resolve",            false)]
  #[case::final_round(   &["--no-final-round", "--final-round"],    "final_unresolved_round", true)]
  #[case::no_final_round(&["--final-round", "--no-final-round"],    "final_unresolved_round", false)]
  #[case::likelihood(    &["--no-likelihood", "--likelihood"],      "likelihood_sort",        true)]
  #[case::no_likelihood( &["--likelihood", "--no-likelihood"],      "likelihood_sort",        false)]
  #[case::naive(         &["--no-naive", "--naive"],                "naive",                  true)]
  #[case::no_naive(      &["--naive", "--no-naive"],                "naive",                  false)]
  #[trace]
  fn the_last_of_a_flag_and_its_opposite_wins(#[case] flags: &[&str], #[case] key: &str, #[case] expected: bool) {
    let dir = tempdir().unwrap();
    let paths = write_trees(dir.path(), &[("ha", HA), ("na", NA)]);
    let out = dir.path().join("out");
    let mut args = vec![paths[0].to_str().unwrap(), paths[1].to_str().unwrap()];
    args.extend(flags);
    run(&args, &out);
    assert_eq!(serde_json::json!(expected), parameters(&out)[key]);
  }

  #[test]
  fn no_pre_resolve_is_a_current_option_without_a_warning() {
    let dir = tempdir().unwrap();
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
    let dir = tempdir().unwrap();
    let from_example = dir.path().join("example");
    run(&["--example", "h3n2-2017"], &from_example);
    let data = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/h3n2-2017");
    let from_files = dir.path().join("files");
    run(&[&format!("{data}/ha.nwk"), &format!("{data}/na.nwk")], &from_files);
    assert_eq!(mccs(&from_files), mccs(&from_example));
  }

  #[test]
  fn unknown_example_suggests_the_closest_id() {
    let dir = tempdir().unwrap();
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
  fn printed_link_of_an_example_names_it_and_runs_the_same_analysis() {
    let dir = tempdir().unwrap();
    let first = dir.path().join("first");
    let link = run_stdout(&["--example", "5-leaves", "--gamma", "3", "--print-link"], &first);
    assert_eq!(
      "https://neherlab.github.io/treeknit/?example=5-leaves&gamma=3&run\n",
      link
    );
    let second = dir.path().join("second");
    run(&["--link", link.trim()], &second);
    assert_eq!(mccs(&first), mccs(&second));
  }

  #[test]
  fn printed_link_of_tree_files_holds_an_inline_session_that_runs_the_same_analysis() {
    let dir = tempdir().unwrap();
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
    let prefix = "https://neherlab.github.io/treeknit/?run#session=data:application/gzip;base64,";
    assert!(link.starts_with(prefix), "{link}");
    let second = dir.path().join("second");
    run(&["--link", link.trim()], &second);
    let written = ["first", "second"].map(|d| dir.path().join(d).join("treeknit_session.json").exists());
    assert_eq!((mccs(&first), [true, true]), (mccs(&second), written));
  }

  #[test]
  fn link_with_data_trees_runs_them_with_their_labels() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("out");
    let link = "https://neherlab.github.io/treeknit/?tree=HA=data:,((A,B),(C,(D,X)));&tree=NA=data:,((A,(B,X)),(C,D));&run&view=mccs";
    run(&["--link", link], &out);
    let present = ["HA_resolved.nwk", "NA_resolved.nwk"].map(|f| out.join(f).exists());
    assert_eq!([true, true], present);
  }
}
