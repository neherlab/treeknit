//! Comparison with reference outputs of the Julia implementation (`fixtures/`, see `ref/`).
//!
//! Deterministic functions must match exactly. Annealing results are compared as
//! distributions and only reported (run with `--nocapture` to see them).

#[cfg(test)]
mod tests {
  use approx::abs_diff_eq;
  use ctor::ctor;
  use rstest::rstest;
  use serde_json::Value;
  use std::collections::BTreeSet;
  use std::path::PathBuf;
  use treeknit_core::bits::Bits;
  use treeknit_core::{Options, Taxa, Tree, arg, mcc_map, naive, pair, resolve, splitgraph};
  use treeknit_io::analysis::{self, ParsedTrees};
  use treeknit_io::newick;

  type Clades = Vec<Vec<String>>;

  /// One thread in the global pool of this test binary, as in the unit tests of the crates: the
  /// test runner runs many tests at once.
  #[ctor(unsafe)]
  fn init() {
    rayon::ThreadPoolBuilder::new()
      .num_threads(1)
      .build_global()
      .expect("the global thread pool of the tests is built once");
  }

  fn fixtures() -> Vec<(String, Value)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let mut v: Vec<_> = std::fs::read_dir(&dir)
      .unwrap()
      .filter_map(|e| e.ok())
      .map(|e| e.path())
      .filter(|p| p.extension().is_some_and(|x| x == "json"))
      .collect();
    v.sort();
    v.into_iter()
      .map(|p| {
        (
          p.file_stem().unwrap().to_string_lossy().into_owned(),
          serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap(),
        )
      })
      .collect()
  }

  fn load(f: &Value) -> (Vec<Tree>, Taxa) {
    let ts: Vec<Tree> = f["trees"]
      .as_array()
      .unwrap()
      .iter()
      .enumerate()
      .map(|(i, s)| {
        newick::parse(
          &without_invalid_root_length(s.as_str().unwrap()),
          &format!("t{}", i + 1),
        )
        .unwrap()
      })
      .collect();
    let ParsedTrees { trees, taxa } = analysis::number_leaves(ts);
    (trees, taxa)
  }

  /// `newick` without the root length `:0.R` of the trees of `test_srg_2`, which come from the
  /// TreeKnit.jl tests. TreeKnit.jl reads an invalid branch length as missing and the port rejects
  /// it (README, "Deliberate differences from TreeKnit.jl"); both ignore the root length.
  fn without_invalid_root_length(newick: &str) -> String {
    newick
      .strip_suffix(":0.R;")
      .map_or_else(|| newick.to_owned(), |tree| format!("{tree};"))
  }

  fn clades_json(v: &Value) -> Clades {
    let mut c: Clades = serde_json::from_value(v.clone()).unwrap();
    for x in &mut c {
      x.sort();
    }
    c.sort();
    c
  }

  fn names(b: &Bits, taxa: &Taxa) -> Vec<String> {
    b.ones().map(|i| taxa.names[i].clone()).collect()
  }

  fn split_names(v: &[Bits], taxa: &Taxa) -> Clades {
    let mut c: Clades = v.iter().map(|b| names(b, taxa)).collect();
    c.sort();
    c
  }

  /// Non-root internal clades.
  fn tree_splits(t: &Tree, taxa: &Taxa) -> Clades {
    let c = t.clades(taxa.len());
    let mut v: Clades = t
      .internals()
      .into_iter()
      .filter(|&n| n != t.root)
      .map(|n| names(&c[n], taxa))
      .collect();
    v.sort();
    v.dedup();
    v
  }

  fn ids(m: &[Vec<String>], taxa: &Taxa) -> Vec<naive::Mcc> {
    naive::sort_mccs(m.iter().map(|x| x.iter().map(|s| taxa.index[s]).collect()).collect())
  }

  fn mcc_names(m: &[naive::Mcc], taxa: &Taxa) -> Clades {
    m.iter().map(|x| taxa.names_of(x)).collect()
  }

  struct Report {
    checks: usize,
    failures: Vec<String>,
  }

  impl Report {
    fn check<T: PartialEq + std::fmt::Debug>(&mut self, case: &str, what: &str, got: &T, want: &T) {
      self.checks += 1;
      if got != want {
        self
          .failures
          .push(format!("{case}/{what}:\n   got  {got:?}\n   want {want:?}"));
      }
    }
  }

  #[test]
  fn deterministic_functions_match_julia() {
    let mut r = Report {
      checks: 0,
      failures: vec![],
    };
    for (case, f) in fixtures() {
      let (ts, taxa) = load(&f);
      let n = taxa.len();
      let n_trees = ts.len();
      let same_leaves = ts.iter().all(|t| t.leaf_set(n) == ts[0].leaf_set(n));

      if let Some(want) = f.get("naive_mccs").filter(|_| same_leaves) {
        let refs: Vec<&Tree> = ts.iter().collect();
        let got = mcc_names(&naive::naive_mccs(&refs, n), &taxa);
        r.check(
          &case,
          "naive_mccs",
          &got,
          &serde_json::from_value::<Clades>(want.clone()).unwrap(),
        );
      }
      if let Some(pw) = f.get("pairwise_naive_mccs").and_then(Value::as_object) {
        for (key, want) in pw {
          let (i, j) = key.split_once('-').unwrap();
          let (i, j): (usize, usize) = (i.parse().unwrap(), j.parse().unwrap());
          let got = mcc_names(&naive::naive_mccs(&[&ts[i], &ts[j]], n), &taxa);
          r.check(
            &case,
            &format!("naive_mccs {key}"),
            &got,
            &serde_json::from_value::<Clades>(want.clone()).unwrap(),
          );
        }
      }
      if let Some(pr) = f.get("pre_resolve") {
        let mut tt = ts.clone();
        let new = resolve::resolve_trees(&mut tt, n);
        for i in 0..n_trees {
          r.check(
            &case,
            &format!("pre_resolve new {i}"),
            &split_names(&new[i], &taxa),
            &clades_json(&pr["new_splits"][i]),
          );
          r.check(
            &case,
            &format!("pre_resolve after {i}"),
            &tree_splits(&tt[i], &taxa),
            &clades_json(&pr["splits_after"][i]),
          );
        }
      }
      if n_trees != 2 || !same_leaves {
        continue;
      }
      let Some(mv) = f.get("mccs") else { continue };
      let mccs = ids(&serde_json::from_value::<Clades>(mv.clone()).unwrap(), &taxa);

      for (field, strict) in [("mcc_resolve_strict", true), ("mcc_resolve_liberal", false)] {
        let Some(want) = f.get(field) else { continue };
        let (mut a, mut b) = (ts[0].clone(), ts[1].clone());
        let new = resolve::resolve_with_mccs(&mut a, &mut b, &mccs, n, strict);
        for (i, t) in [&a, &b].iter().enumerate() {
          r.check(
            &case,
            &format!("{field} new {i}"),
            &split_names(&new[i], &taxa),
            &clades_json(&want["new_splits"][i]),
          );
          r.check(
            &case,
            &format!("{field} after {i}"),
            &tree_splits(t, &taxa),
            &clades_json(&want["splits_after"][i]),
          );
        }
      }

      if let Some(want) = f.get("fitch").and_then(Value::as_array) {
        let map = mcc_map::map_mccs(&ts[0], &mcc_map::leaf_mcc_map(&mccs, n));
        let c = ts[0].clades(n);
        let mut got: Vec<(Vec<String>, Option<u64>)> = ts[0]
          .postorder()
          .into_iter()
          .map(|x| (names(&c[x], &taxa), map[x].map(|m| u64::try_from(m).unwrap())))
          .collect();
        let mut want: Vec<(Vec<String>, Option<u64>)> = want
          .iter()
          .map(|e| {
            (
              serde_json::from_value::<Vec<String>>(e["clade"].clone()).unwrap(),
              e["mcc"].as_u64(),
            )
          })
          .map(|(mut c, m)| {
            c.sort();
            (c, m)
          })
          .collect();
        got.sort();
        want.sort();
        r.check(&case, "fitch", &got, &want);
      }

      if let Some(want) = f.get("sorted_leaf_order") {
        let (mut a, mut b) = (ts[0].clone(), ts[1].clone());
        a.ladderize();
        mcc_map::sort_polytomies_by_mccs(&a, &mut b, &mccs, n);
        let want: Clades = serde_json::from_value(want.clone()).unwrap();
        r.check(&case, "sorted_leaf_order", &vec![a.leaf_names(), b.leaf_names()], &want);
      }

      if let Some(e) = f.get("energy").and_then(Value::as_array) {
        let nm = naive::naive_mccs(&[&ts[0], &ts[1]], n);
        let r1 = pair::reduce_to_mccs(&ts[0], &nm, n);
        let r2 = pair::reduce_to_mccs(&ts[1], &nm, n);
        let g = splitgraph::Graph::new(&[&r1, &r2], nm.len());
        let sl: Vec<f64> = serde_json::from_value(f["seq_lengths"].clone()).unwrap();
        for (q, entry) in e.iter().enumerate() {
          let removed = ids(
            &serde_json::from_value::<Clades>(entry["removed"].clone()).unwrap(),
            &taxa,
          );
          let mut conf = treeknit_core::bits::full(nm.len());
          for m in &removed {
            conf.set(nm.iter().position(|x| x == m).expect("removed MCC is not naive"), false);
          }
          r.check(
            &case,
            &format!("energy[{q}] resolve"),
            &u64::try_from(g.energy(&conf, true)).unwrap(),
            &entry["E_resolve"].as_u64().unwrap(),
          );
          r.check(
            &case,
            &format!("energy[{q}] no resolve"),
            &u64::try_from(g.energy(&conf, false)).unwrap(),
            &entry["E_noresolve"].as_u64().unwrap(),
          );
          if let Some(lk) = entry["lk"].as_f64() {
            let got = g.likelihood(&conf, true, &[&r1, &r2], &sl);
            // The largest difference over the fixtures is 4.5e-13, one ulp of a likelihood near
            // 3000; the tolerance leaves about ten ulps at the largest likelihood, 7344, for
            // another build of the logarithm.
            r.check(
              &case,
              &format!("lk[{q}] = {lk}, got {got}"),
              &abs_diff_eq!(lk, got, epsilon = 1e-11),
              &true,
            );
          }
        }
      }

      if let Some(want) = f.get("arg") {
        match arg::arg_from_trees(&ts[0], &ts[1], &mccs, n) {
          Ok(a) => {
            r.check(
              &case,
              "arg n_hybrids",
              &u64::try_from(a.n_hybrids()).unwrap(),
              &want["n_hybrids"].as_u64().unwrap(),
            );
            for c in 0..2 {
              let mut seg = a.segment_tree(c);
              seg.remove_unary();
              seg.assign_taxa(&taxa).unwrap();
              let got = tree_splits(&seg, &taxa);
              r.check(
                &case,
                &format!("arg segment {c}"),
                &got,
                &clades_json(&want["segment_trees_no_singletons"][c]),
              );
            }
          },
          Err(e) => r.failures.push(format!("{case}/arg: {e}")),
        }
      }
    }
    eprintln!("{} checks, {} failures", r.checks, r.failures.len());
    for x in &r.failures {
      eprintln!("FAIL {x}");
    }
    assert!(
      r.failures.is_empty(),
      "{} of {} checks failed",
      r.failures.len(),
      r.checks
    );
  }

  /// Annealing outcomes vs Julia: share of Rust runs whose MCCs Julia also produced, and
  /// mean number of MCCs. Informational; asserts only on cases where Julia is unanimous.
  #[test]
  fn annealing_distribution_vs_julia() {
    let mut bad = Vec::new();
    for (case, f) in fixtures() {
      let Some(runs) = f.get("runs").and_then(Value::as_array) else {
        continue;
      };
      let (ts, taxa) = load(&f);
      let julia: Vec<Clades> = runs
        .iter()
        .map(|r| serde_json::from_value(r.clone()).unwrap())
        .collect();
      let jset: BTreeSet<&Clades> = julia.iter().collect();
      let o = Options::treeknit_jl(2, None);
      let mut rust: Vec<Clades> = Vec::new();
      for seed in 0..u64::try_from(julia.len()).unwrap() {
        let mut tt = ts.clone();
        let res = treeknit_core::run(&mut tt, &taxa, &o, seed);
        rust.push(mcc_names(&res[0].mccs, &taxa));
      }
      let shared = rust.iter().filter(|x| jset.contains(x)).count();
      let mean = |v: &[Clades]| {
        f64::from(u32::try_from(v.iter().map(Vec::len).sum::<usize>()).unwrap())
          / f64::from(u32::try_from(v.len()).unwrap())
      };
      let distinct_r: BTreeSet<&Clades> = rust.iter().collect();
      eprintln!(
        "{case:32} julia {:5.2} MCCs ({} distinct)  rust {:5.2} MCCs ({} distinct)  rust∈julia {shared}/{}",
        mean(&julia),
        jset.len(),
        mean(&rust),
        distinct_r.len(),
        rust.len()
      );
      if jset.len() == 1 && shared < rust.len() {
        bad.push(case);
      }
    }
    assert!(bad.is_empty(), "Rust differs where Julia is unanimous: {bad:?}");
  }

  /// With matched resolution, every MCC of every pair has the same topology in both output trees
  /// (pairwise MCCs need not be transitive), and matching only adds splits: every input split is
  /// still present. Runs on every fixture of three or more trees and reports each failing one.
  #[rstest]
  #[trace]
  fn matched_topologies_on_three_segments(#[values(0, 1, 2)] seed: u64) {
    let failures: Vec<(String, Vec<(usize, usize)>, Vec<usize>)> = fixtures()
      .into_iter()
      .filter(|(_, f)| f["trees"].as_array().unwrap().len() >= 3)
      .filter_map(|(case, f)| {
        let (ts, taxa) = load(&f);
        let o = Options::for_trees(ts.len()); // matched resolution is the default
        let mut tt = ts.clone();
        let res = treeknit_core::run(&mut tt, &taxa, &o, seed);
        let unmatched = treeknit_core::unmatched_mccs(&tt, &res, taxa.len());
        let removed: Vec<usize> = (0..ts.len())
          .filter(|&i| {
            let after = tree_splits(&tt[i], &taxa);
            !tree_splits(&ts[i], &taxa).iter().all(|s| after.contains(s))
          })
          .collect();
        (!unmatched.is_empty() || !removed.is_empty()).then_some((case, unmatched, removed))
      })
      .collect();
    assert_eq!(Vec::<(String, Vec<(usize, usize)>, Vec<usize>)>::new(), failures);
  }
}
