//! Cost of the topological energy of the split graph of two trees, computed in full and updated
//! after single-leaf flips, and a check that both give the same energy.
//!
//! `just example energy_cost <tree1.nwk> <tree2.nwk> [steps]` reduces both trees to their naive
//! MCCs and builds the split graph that annealing works on. For annealing without and with
//! resolution, it times the full energy of the configuration that keeps every MCC, then runs a
//! Metropolis chain (γ = 2) with `steps` flips at each of the temperatures 1, 0.3, 0.1, and 0.05.
//! It times the incremental flips of that chain, then reruns it from the same seed and compares the
//! energy after every flip with the full computation. `steps` defaults to 100 000.

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use std::hint::black_box;
use std::process::ExitCode;
use std::time::{Duration, Instant};
use treeknit_core::bits;
use treeknit_core::pair::reduce_to_mccs;
use treeknit_core::splitgraph::{EnergyState, Graph};
use treeknit_core::{Taxa, Tree, naive_mccs};
use treeknit_io::newick;

const USAGE: &str = "usage: energy_cost <tree1.nwk> <tree2.nwk> [steps=100000]";

/// Cost of a removed leaf, the default γ of TreeKnit.
const GAMMA: f64 = 2.0;

/// Temperatures of the chain: the first and last of the default cooling schedule and two between.
const TEMPERATURES: [f64; 4] = [1.0, 0.3, 0.1, 0.05];

/// Full energy computations per timing.
const FULL_REPS: u32 = 200;

fn main() -> ExitCode {
  let args: Vec<String> = std::env::args().skip(1).collect();
  match run(&args) {
    Ok(()) => ExitCode::SUCCESS,
    Err(e) => {
      eprintln!("energy_cost: {e}");
      ExitCode::FAILURE
    },
  }
}

fn run(args: &[String]) -> Result<(), String> {
  let [file1, file2, rest @ ..] = args else {
    return Err(USAGE.to_owned());
  };
  let steps: u32 = match rest {
    [] => 100_000,
    [s] => s.parse().map_err(|e| format!("steps '{s}': {e}\n{USAGE}"))?,
    _ => return Err(USAGE.to_owned()),
  };
  let mut trees = vec![read_tree(file1)?, read_tree(file2)?];
  let taxa = Taxa::from_trees(&trees);
  for t in &mut trees {
    t.assign_taxa(&taxa)?;
  }
  let n = taxa.len();
  if trees[0].leaf_set(n) != trees[1].leaf_set(n) {
    return Err("the trees must have the same leaves".to_owned());
  }
  let mccs = naive_mccs(&[&trees[0], &trees[1]], n);
  let reduced = [reduce_to_mccs(&trees[0], &mccs, n), reduce_to_mccs(&trees[1], &mccs, n)];
  let g = Graph::new(&[&reduced[0], &reduced[1]], mccs.len());
  println!("leaves {n}, naive MCCs {}", mccs.len());

  for resolve in [false, true] {
    let all = bits::full(mccs.len());
    let start = Instant::now();
    let mut energy = 0;
    for _ in 0..FULL_REPS {
      energy = black_box(g.energy(black_box(&all), resolve));
    }
    println!(
      "resolve {resolve}: full energy {energy}, {:.1} µs per full computation",
      micros_per(start.elapsed(), FULL_REPS)
    );
    for t in TEMPERATURES {
      let timed = chain(&g, mccs.len(), resolve, t, steps, false);
      let checked = chain(&g, mccs.len(), resolve, t, steps, true);
      println!(
        "resolve {resolve}, T {t}: {:.2} µs per flip, {} accepted, final energy {}, {} mismatches with the full energy",
        micros_per(timed.elapsed, steps),
        timed.accepted,
        timed.energy,
        checked.mismatches
      );
    }
  }
  Ok(())
}

fn read_tree(path: &str) -> Result<Tree, String> {
  let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
  Ok(
    newick::parse_first(&text, path)
      .map_err(|e| format!("{path}: {e}"))?
      .tree,
  )
}

/// Run a Metropolis chain of `steps` single-leaf flips at temperature `t` from the configuration
/// that keeps all `n` graph leaves. With `check`, compare the energy after every flip with the full
/// computation.
fn chain(g: &Graph, n: usize, resolve: bool, t: f64, steps: u32, check: bool) -> Chain {
  let mut rng = Xoshiro256PlusPlus::seed_from_u64(1);
  let mut st = EnergyState::new(g, bits::full(n), resolve);
  let free_energy = |st: &EnergyState| free_energy(st.energy(), n - st.n_kept());
  let mut f = free_energy(&st);
  let (mut accepted, mut mismatches) = (0, 0);
  let start = Instant::now();
  for _ in 0..steps {
    st.flip(rng.gen_range(0..n));
    let f_new = free_energy(&st);
    if f_new < f || (-(f_new - f) / t).exp() > rng.r#gen::<f64>() {
      f = f_new;
      accepted += 1;
    } else {
      st.undo();
    }
    if check && st.energy() != g.energy(st.conf(), resolve) {
      mismatches += 1;
    }
  }
  Chain {
    elapsed: start.elapsed(),
    accepted,
    mismatches,
    energy: st.energy(),
  }
}

/// Outcome of one chain.
struct Chain {
  elapsed: Duration,
  accepted: u32,
  mismatches: u32,
  energy: usize,
}

/// Free energy of annealing: the topological energy plus γ for each removed leaf.
#[expect(
  clippy::as_conversions,
  reason = "energies and leaf counts of trees stay far below 2^53, so f64 holds them exactly"
)]
fn free_energy(energy: usize, removed: usize) -> f64 {
  energy as f64 + GAMMA * removed as f64
}

fn micros_per(elapsed: Duration, count: u32) -> f64 {
  elapsed.as_secs_f64() * 1e6 / f64::from(count)
}
