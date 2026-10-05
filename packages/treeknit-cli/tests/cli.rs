//! End-to-end runs of the `treeknit` binary.

#[cfg(test)]
mod tests {
  use std::path::{Path, PathBuf};
  use std::process::Command;

  fn run(args: &[&str], out: &Path) {
    if out.exists() {
      std::fs::remove_dir_all(out).unwrap();
    }
    let status = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .args(args)
      .arg("-o")
      .arg(out)
      .args(["--verbosity-level", "-1"])
      .status()
      .unwrap();
    assert!(status.success());
  }

  /// Run `treeknit` on Newick `trees` written to files `t0.nwk`, `t1.nwk`, ..., expecting a
  /// failure; return the exit code, the error output, and whether any result file was written.
  fn fail(name: &str, trees: &[&str], args: &[&str]) -> (Option<i32>, String, bool) {
    let dir = tmp(&format!("{name}-in"));
    std::fs::create_dir_all(&dir).unwrap();
    let paths: Vec<PathBuf> = trees
      .iter()
      .enumerate()
      .map(|(i, t)| {
        let p = dir.join(format!("t{i}.nwk"));
        std::fs::write(&p, t).unwrap();
        p
      })
      .collect();
    let out = tmp(name);
    if out.exists() {
      std::fs::remove_dir_all(&out).unwrap();
    }
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
    (output.status.code(), String::from_utf8(output.stderr).unwrap(), results)
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
      ("lengths-former", &["--better-MCCs", "--seq-lengths", "0 0"], "sequence length 1 must be a positive number, got 0"),
      ("seed-large",     &["--seed", "9007199254740992"], "seed must be at most 9007199254740991, got 9007199254740992"),
      ("mcmc-zero",      &["--n-mcmc-it", "0"],          "MCMC steps per leaf must be at least 1"),
    ] {
      let (code, stderr, results) = fail(name, &[HA, NA], args);
      assert_eq!(Some(1), code, "{name}");
      assert!(stderr.contains(message), "{name}: {stderr}");
      assert!(!results, "{name}: result files written");
    }
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
      let (code, stderr, results) = fail(name, &[HA, other], args);
      assert_eq!(Some(1), code, "{name}");
      assert!(stderr.contains("trees t0 and t1 share fewer than two leaves"), "{name}: {stderr}");
      assert!(!results, "{name}: result files written");
    }
  }

  #[test]
  fn colliding_pair_names_exit_before_writing_results() {
    let dir = tmp("stems-in");
    std::fs::create_dir_all(&dir).unwrap();
    let paths: Vec<PathBuf> = ["a_b", "c", "a", "b_c"]
      .iter()
      .map(|l| {
        let p = dir.join(format!("{l}.nwk"));
        std::fs::write(&p, "((A,B),(C,D));").unwrap();
        p
      })
      .collect();
    let out = tmp("stems");
    let output = Command::new(env!("CARGO_BIN_EXE_treeknit"))
      .args(&paths)
      .arg("-o")
      .arg(&out)
      .args(["--verbosity-level", "-1"])
      .output()
      .unwrap();
    assert_eq!(Some(1), output.status.code());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
      stderr.contains("tree pairs (a_b, c) and (a, b_c) give the same output file names (a_b_c)"),
      "{stderr}"
    );
    assert!(!out.join("MCCs.json").exists());
  }

  fn tmp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("treeknit-cli-test-{name}-{}", std::process::id()))
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
    let out = tmp("two");
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
    let dir = tmp("three-in");
    std::fs::create_dir_all(&dir).unwrap();
    let trees = [
      "((A,B),(C,(D,(E,X))));",
      "((A,(B,X)),(C,D,E,P));",
      "((A,(B,P)),((C,D),(E,X)));",
    ];
    let paths: Vec<String> = trees
      .iter()
      .enumerate()
      .map(|(i, t)| {
        let p = dir.join(format!("seg{i}.nwk"));
        std::fs::write(&p, t).unwrap();
        p.to_string_lossy().into_owned()
      })
      .collect();
    let out = tmp("three");
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
