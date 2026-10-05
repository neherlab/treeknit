//! Comparison with reference outputs of the Julia implementation (`fixtures/`, see `ref/`).
//!
//! Deterministic functions must match exactly. Annealing results are compared as
//! distributions and only reported (run with `--nocapture` to see them).

use serde_json::Value;
use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;
use treeknit_core::bits::Bits;
use treeknit_core::{arg, mcc_map, naive, pair, resolve, splitgraph, Options, Taxa, Tree};
use treeknit_io::newick;

type Clades = Vec<Vec<String>>;

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
    let mut ts: Vec<Tree> = f["trees"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, s)| newick::parse(s.as_str().unwrap(), &format!("t{}", i + 1)).unwrap())
        .collect();
    let taxa = Taxa::from_trees(&ts);
    for t in ts.iter_mut() {
        t.assign_taxa(&taxa).unwrap();
    }
    (ts, taxa)
}

fn clades_json(v: &Value) -> Clades {
    let mut c: Clades = serde_json::from_value(v.clone()).unwrap();
    for x in c.iter_mut() {
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
    fn check<T: PartialEq + std::fmt::Debug>(&mut self, case: &str, what: &str, got: T, want: T) {
        self.checks += 1;
        if got != want {
            self.failures
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
        let k = ts.len();
        let same_leaves = ts.iter().all(|t| t.leaf_set(n) == ts[0].leaf_set(n));

        if let Some(want) = f.get("naive_mccs").filter(|_| same_leaves) {
            let refs: Vec<&Tree> = ts.iter().collect();
            let got = mcc_names(&naive::naive_mccs(&refs, n), &taxa);
            r.check(
                &case,
                "naive_mccs",
                got,
                serde_json::from_value::<Clades>(want.clone()).unwrap(),
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
                    got,
                    serde_json::from_value::<Clades>(want.clone()).unwrap(),
                );
            }
        }
        if let Some(pr) = f.get("pre_resolve") {
            let mut tt = ts.clone();
            let new = resolve::resolve_trees(&mut tt, n);
            for i in 0..k {
                r.check(
                    &case,
                    &format!("pre_resolve new {i}"),
                    split_names(&new[i], &taxa),
                    clades_json(&pr["new_splits"][i]),
                );
                r.check(
                    &case,
                    &format!("pre_resolve after {i}"),
                    tree_splits(&tt[i], &taxa),
                    clades_json(&pr["splits_after"][i]),
                );
            }
        }
        if k != 2 || !same_leaves {
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
                    split_names(&new[i], &taxa),
                    clades_json(&want["new_splits"][i]),
                );
                r.check(
                    &case,
                    &format!("{field} after {i}"),
                    tree_splits(t, &taxa),
                    clades_json(&want["splits_after"][i]),
                );
            }
        }

        if let Some(want) = f.get("fitch").and_then(Value::as_array) {
            let map = mcc_map::map_mccs(&ts[0], &mcc_map::leaf_mcc_map(&mccs, n));
            let c = ts[0].clades(n);
            let mut got: Vec<(Vec<String>, Option<u64>)> = ts[0]
                .postorder()
                .into_iter()
                .map(|x| (names(&c[x], &taxa), map[x].map(|m| m as u64)))
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
            r.check(&case, "fitch", got, want);
        }

        if let Some(want) = f.get("sorted_leaf_order") {
            let (mut a, mut b) = (ts[0].clone(), ts[1].clone());
            a.ladderize();
            mcc_map::sort_polytomies_by_mccs(&a, &mut b, &mccs, n);
            let want: Clades = serde_json::from_value(want.clone()).unwrap();
            r.check(&case, "sorted_leaf_order", vec![a.leaf_names(), b.leaf_names()], want);
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
                    g.energy(&conf, true) as u64,
                    entry["E_resolve"].as_u64().unwrap(),
                );
                r.check(
                    &case,
                    &format!("energy[{q}] no resolve"),
                    g.energy(&conf, false) as u64,
                    entry["E_noresolve"].as_u64().unwrap(),
                );
                if let Some(lk) = entry["lk"].as_f64() {
                    let got = g.likelihood(&conf, true, &[&r1, &r2], &sl);
                    r.check(
                        &case,
                        &format!("lk[{q}]"),
                        (got - lk).abs() < 1e-8 * (1.0 + lk.abs()),
                        true,
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
                        a.n_hybrids() as u64,
                        want["n_hybrids"].as_u64().unwrap(),
                    );
                    for c in 0..2 {
                        let mut seg = a.segment_tree(c);
                        seg.remove_unary();
                        seg.assign_taxa(&taxa).unwrap();
                        let got = tree_splits(&seg, &taxa);
                        r.check(
                            &case,
                            &format!("arg segment {c}"),
                            got,
                            clades_json(&want["segment_trees_no_singletons"][c]),
                        );
                    }
                }
                Err(e) => r.failures.push(format!("{case}/arg: {e}")),
            }
        }
    }
    println!("{} checks, {} failures", r.checks, r.failures.len());
    for x in &r.failures {
        println!("FAIL {x}");
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
        let jset: HashSet<&Clades> = julia.iter().collect();
        let o = Options::treeknit_jl(2, None);
        let mut rust: Vec<Clades> = Vec::new();
        for seed in 0..julia.len() as u64 {
            let mut tt = ts.clone();
            let res = treeknit_core::run(&mut tt, &taxa, &o, seed);
            rust.push(mcc_names(&res[0].mccs, &taxa));
        }
        let shared = rust.iter().filter(|x| jset.contains(x)).count();
        let mean = |v: &[Clades]| v.iter().map(|x| x.len()).sum::<usize>() as f64 / v.len() as f64;
        let distinct_r: BTreeSet<&Clades> = rust.iter().collect();
        println!(
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

/// With matched resolution, every MCC of every pair has the same topology in both output trees,
/// on simulated data with three segments (pairwise MCCs need not be transitive).
#[test]
fn matched_topologies_on_three_segments() {
    for (case, f) in fixtures() {
        if f["trees"].as_array().unwrap().len() < 3 {
            continue;
        }
        let (ts, taxa) = load(&f);
        let n = taxa.len();
        for seed in 0..3 {
            let o = Options::for_trees(ts.len()); // matched resolution is the default
            let mut tt = ts.clone();
            let res = treeknit_core::run(&mut tt, &taxa, &o, seed);
            let bad = treeknit_core::unmatched_mccs(&tt, &res, n);
            assert!(bad.is_empty(), "{case} seed {seed}: unmatched MCCs {bad:?}");
            // Matching only adds splits: every input split is still present.
            for (t, t0) in tt.iter().zip(&ts) {
                let before = tree_splits(t0, &taxa);
                let after = tree_splits(t, &taxa);
                assert!(before.iter().all(|s| after.contains(s)), "{case}: a split was removed");
            }
        }
    }
}
