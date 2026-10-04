//! Accuracy on simulated fixtures (those with `true_mccs`): scaled variation of information
//! between inferred and true MCC partitions, for the Julia runs stored in the fixture and
//! for Rust runs with the same number of seeds. Also runs the partial-overlap experiment:
//! drop a fraction of leaves from each tree and check where the dropped leaves are placed.
//!
//! `cargo run --release -p treeknit-io --example accuracy [drop_fraction]`

use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde_json::Value;
use std::collections::HashMap;
use treeknit_core::{Options, Taxa, Tree};
use treeknit_io::newick;

type Partition = Vec<Vec<String>>;

/// Variation of information between two partitions of the same set, divided by ln(n).
fn scaled_vi(a: &Partition, b: &Partition) -> f64 {
    let la: HashMap<&str, usize> = a
        .iter()
        .enumerate()
        .flat_map(|(i, m)| m.iter().map(move |x| (x.as_str(), i)))
        .collect();
    let lb: HashMap<&str, usize> = b
        .iter()
        .enumerate()
        .flat_map(|(i, m)| m.iter().map(move |x| (x.as_str(), i)))
        .collect();
    let n = la.len() as f64;
    let mut joint: HashMap<(usize, usize), f64> = HashMap::new();
    for (x, &i) in &la {
        *joint.entry((i, lb[x])).or_default() += 1.0;
    }
    let h = |p: &Partition| -p.iter().map(|m| m.len() as f64 / n).map(|q| q * q.ln()).sum::<f64>();
    let mi: f64 = joint
        .iter()
        .map(|(&(i, j), &c)| {
            let p = c / n;
            p * (p / (a[i].len() as f64 / n * b[j].len() as f64 / n)).ln()
        })
        .sum();
    (h(a) + h(b) - 2.0 * mi) / n.ln()
}

fn load(f: &Value) -> (Vec<Tree>, Taxa) {
    let mut ts: Vec<Tree> = f["trees"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, s)| newick::parse(s.as_str().unwrap(), &format!("t{i}")).unwrap())
        .collect();
    let taxa = Taxa::from_trees(&ts);
    ts.iter_mut().for_each(|t| t.assign_taxa(&taxa).unwrap());
    (ts, taxa)
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len().max(1) as f64
}

fn main() {
    let drop: f64 = std::env::args().nth(1).map_or(0.2, |s| s.parse().unwrap());
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures");
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.to_string_lossy().contains("/sim_"))
        .collect();
    files.sort();
    println!(
        "{:30} {:>5} {:>9} {:>9} {:>8} {:>8} | drop {:.0}%: {:>9} {:>8} {:>8} {:>9}",
        "case",
        "pair",
        "julia VI",
        "rust VI",
        "julia #",
        "rust #",
        drop * 100.0,
        "VI on S",
        "placed",
        "ambig.",
        "baseline"
    );
    for path in files {
        let f: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let case = path.file_stem().unwrap().to_string_lossy().into_owned();
        let (ts, taxa) = load(&f);
        let k = ts.len();
        let truth: HashMap<String, Partition> = serde_json::from_value(f["true_mccs"].clone()).unwrap();
        let julia: Vec<HashMap<String, Partition>> = if k == 2 {
            f["runs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| HashMap::from([("0-1".to_string(), serde_json::from_value(r.clone()).unwrap())]))
                .collect()
        } else {
            serde_json::from_value(f["multi_runs"].clone()).unwrap()
        };
        let opts = Options::treeknit_jl(k, None); // as the Julia runs it is compared with
        let rust: Vec<HashMap<String, Partition>> = (0..julia.len() as u64)
            .map(|seed| {
                let mut tt = ts.clone();
                treeknit_core::run(&mut tt, &taxa, &opts, seed)
                    .into_iter()
                    .map(|p| {
                        (
                            format!("{}-{}", p.i, p.j),
                            p.mccs.iter().map(|m| taxa.names_of(m)).collect(),
                        )
                    })
                    .collect()
            })
            .collect();

        // Partial overlap: drop a random subset of leaves from each tree independently.
        let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(1);
        let mut partial = ts.clone();
        for t in partial.iter_mut() {
            let mut leaves: Vec<usize> = t.leaves().into_iter().map(|n| t.taxon(n)).collect();
            leaves.shuffle(&mut rng);
            let keep = treeknit_core::bits::from_iter(
                taxa.len(),
                leaves[(leaves.len() as f64 * drop) as usize..].iter().copied(),
            );
            *t = t.restricted(&keep).unwrap();
        }
        let res = treeknit_core::run(&mut partial, &taxa, &opts, 0);

        for (key, true_p) in {
            let mut v: Vec<_> = truth.iter().collect();
            v.sort();
            v
        } {
            let jv: Vec<f64> = julia.iter().map(|r| scaled_vi(&r[key], true_p)).collect();
            let rv: Vec<f64> = rust.iter().map(|r| scaled_vi(&r[key], true_p)).collect();
            let jn: Vec<f64> = julia.iter().map(|r| r[key].len() as f64).collect();
            let rn: Vec<f64> = rust.iter().map(|r| r[key].len() as f64).collect();

            let (i, j) = key.split_once('-').unwrap();
            let (i, j): (usize, usize) = (i.parse().unwrap(), j.parse().unwrap());
            let p = res.iter().find(|p| p.i == i && p.j == j).unwrap();
            let true_of: HashMap<&str, usize> = true_p
                .iter()
                .enumerate()
                .flat_map(|(q, m)| m.iter().map(move |x| (x.as_str(), q)))
                .collect();
            // VI restricted to shared leaves, and placement accuracy of attached leaves: an
            // attached leaf is correct if it shares a true MCC with most of the MCC it joined.
            let shared = |name: &str| {
                p.attached
                    .iter()
                    .all(|a| a.leaves.iter().all(|&x| taxa.names[x] != name))
            };
            let inferred_s: Partition = p
                .mccs
                .iter()
                .map(|m| taxa.names_of(m).into_iter().filter(|x| shared(x)).collect::<Vec<_>>())
                .filter(|m: &Vec<String>| !m.is_empty())
                .collect();
            let true_s: Partition = true_p
                .iter()
                .map(|m| {
                    m.iter()
                        .filter(|x| inferred_s.iter().flatten().any(|y| y == *x))
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .filter(|m| !m.is_empty())
                .collect();
            // A placed leaf is correct if its true MCC is that of most of its anchor leaves
            // (the neighbours it was placed next to). Baseline: join the largest MCC.
            let largest = inferred_s.iter().max_by_key(|m| m.len()).unwrap();
            let majority = |names: &mut dyn Iterator<Item = &str>| {
                let mut c: HashMap<usize, usize> = HashMap::new();
                for x in names {
                    *c.entry(true_of[x]).or_default() += 1;
                }
                c.into_iter()
                    .max_by_key(|&(q, n)| (n, std::cmp::Reverse(q)))
                    .map(|(q, _)| q)
            };
            let base_mcc = majority(&mut largest.iter().map(|x| x.as_str()));
            let (mut ok, mut base, mut tot, mut amb) = (0, 0, 0, 0);
            for a in &p.attached {
                let anchor_mcc = majority(&mut a.anchor.iter().map(|&y| taxa.names[y].as_str()));
                for &x in &a.leaves {
                    let tx = true_of[taxa.names[x].as_str()];
                    tot += 1;
                    amb += a.ambiguous as usize;
                    ok += (anchor_mcc == Some(tx)) as usize;
                    base += (base_mcc == Some(tx)) as usize;
                }
            }
            println!(
                "{case:30} {key:>5} {:9.3} {:9.3} {:8.2} {:8.2} | {:>15.3} {:>5}/{:<3} {:>7} {:>9}",
                mean(&jv),
                mean(&rv),
                mean(&jn),
                mean(&rn),
                scaled_vi(&inferred_s, &true_s),
                ok,
                tot,
                amb,
                base
            );
        }
    }
}
