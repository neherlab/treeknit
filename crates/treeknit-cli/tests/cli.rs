//! End-to-end runs of the `treeknit` binary.

use std::path::{Path, PathBuf};
use std::process::Command;

fn run(args: &[&str], out: &Path) {
    let _ = std::fs::remove_dir_all(out);
    let status = Command::new(env!("CARGO_BIN_EXE_treeknit"))
        .args(args)
        .arg("-o")
        .arg(out)
        .args(["--verbosity-level", "-1"])
        .status()
        .unwrap();
    assert!(status.success());
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
    run(&[&format!("{ex}/tree_h3n2_ha.nwk"), &format!("{ex}/tree_h3n2_na.nwk"), "--auspice-view"], &out);
    for f in ["MCCs.json", "parameters.json", "log.txt", "tree_h3n2_ha_resolved.nwk", "auspice_tree_h3n2_na.json",
        "ARG/arg.nwk", "ARG/nodes.dat", "ARG/tree_h3n2_na_liberal_resolved.nwk"]
    {
        assert!(out.join(f).exists(), "missing {f}");
    }
    let n = mccs(&out)["MCC_dict"]["1"]["mccs"].as_array().unwrap().len();
    let arg = std::fs::read_to_string(out.join("ARG/arg.nwk")).unwrap();
    assert_eq!(arg.matches("#H").count(), 2 * (n - 1));
}

#[test]
fn three_trees_partial_overlap_imputed() {
    let dir = tmp("three-in");
    std::fs::create_dir_all(&dir).unwrap();
    let trees = ["((A,B),(C,(D,(E,X))));", "((A,(B,X)),(C,D,E,P));", "((A,(B,P)),((C,D),(E,X)));"];
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
    // P is missing from seg0: pairs (0,1) and (0,2) place it.
    assert!(d["1"]["imputed"].as_array().unwrap().iter().any(|e| e["leaf"] == "P"));
    let imputed = std::fs::read_to_string(out.join("seg0_imputed.nwk")).unwrap();
    assert!(imputed.contains('P'));
}
