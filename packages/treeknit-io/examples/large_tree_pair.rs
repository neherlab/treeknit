//! Pair of large random Newick trees for manual performance checks of the CLI and the web app.
//!
//! Tree A is a random binary tree with the topology and branch lengths (in coalescent units)
//! of a Kingman coalescent. Tree B is a copy of A changed by random subtree moves (prune and
//! regraft), the way a reassortment changes one segment tree against another. Leaf names look
//! like strain names (`A/Sim/17/2020`). The program only writes input: it computes nothing
//! that TreeKnit reports.
//!
//! `just example large_tree_pair <leaves> <outdir> [seed] [moves]` writes `<outdir>/tree_a.nwk`
//! and `<outdir>/tree_b.nwk`. The seed defaults to 1 and the number of moves to 10; the same
//! arguments always write the same files.

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use std::path::PathBuf;
use std::process::ExitCode;
use treeknit_core::{NodeId, Tree};
use treeknit_io::newick;

const USAGE: &str = "usage: large_tree_pair <leaves> <outdir> [seed=1] [moves=10]";

/// Smallest number of leaves on which a subtree move changes the topology.
const MIN_LEAVES: u32 = 3;

#[derive(Debug, PartialEq)]
struct Args {
  leaves: u32,
  outdir: PathBuf,
  seed: u64,
  moves: u32,
}

fn main() -> ExitCode {
  let args: Vec<String> = std::env::args().skip(1).collect();
  match parse_args(&args).and_then(|a| write_pair(&a)) {
    Ok(()) => ExitCode::SUCCESS,
    Err(e) => {
      eprintln!("large_tree_pair: {e}");
      ExitCode::FAILURE
    },
  }
}

fn parse_args(args: &[String]) -> Result<Args, String> {
  let [leaves, outdir, rest @ ..] = args else {
    return Err(USAGE.to_owned());
  };
  if rest.len() > 2 {
    return Err(USAGE.to_owned());
  }
  let leaves: u32 = leaves.parse().map_err(|e| format!("leaves '{leaves}': {e}\n{USAGE}"))?;
  if leaves < MIN_LEAVES {
    return Err(format!(
      "leaves: {leaves} is below {MIN_LEAVES}, the smallest tree that a subtree move changes\n{USAGE}"
    ));
  }
  let seed = rest
    .first()
    .map_or(Ok(1), |s| s.parse().map_err(|e| format!("seed '{s}': {e}\n{USAGE}")))?;
  let moves = rest
    .get(1)
    .map_or(Ok(10), |s| s.parse().map_err(|e| format!("moves '{s}': {e}\n{USAGE}")))?;
  Ok(Args {
    leaves,
    outdir: PathBuf::from(outdir),
    seed,
    moves,
  })
}

/// Write both trees, then read each file back with the parser of the CLI.
fn write_pair(args: &Args) -> Result<(), String> {
  let (a, b) = tree_pair(args.leaves, args.seed, args.moves);
  std::fs::create_dir_all(&args.outdir).map_err(|e| format!("{}: {e}", args.outdir.display()))?;
  for (t, file) in [(&a, "tree_a.nwk"), (&b, "tree_b.nwk")] {
    let path = args.outdir.join(file);
    let io_err = |e: std::io::Error| format!("{}: {e}", path.display());
    std::fs::write(&path, format!("{}\n", newick::write(t))).map_err(io_err)?;
    let parsed =
      newick::parse_first(&std::fs::read_to_string(&path).map_err(io_err)?, file).map_err(|e| e.to_string())?;
    if parsed.n_leaves() != t.n_leaves() {
      return Err(format!(
        "{}: read {} leaves back, wrote {}",
        path.display(),
        parsed.n_leaves(),
        t.n_leaves()
      ));
    }
  }
  Ok(())
}

/// Tree A from the coalescent, and tree B as A after `moves` subtree moves.
fn tree_pair(leaves: u32, seed: u64, moves: u32) -> (Tree, Tree) {
  let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
  let a = coalescent_tree(leaves, &mut rng);
  let mut b = a.clone();
  b.label = "b".to_owned();
  for _ in 0..moves {
    move_subtree(&mut b, &mut rng);
  }
  (a, b)
}

/// Random binary tree under the Kingman coalescent: while k lineages remain, wait an
/// exponential time with rate k(k-1)/2, then join two lineages drawn uniformly.
fn coalescent_tree(leaves: u32, rng: &mut impl Rng) -> Tree {
  let mut tree = Tree::new("a");
  // Node times, indexed by node id; leaves sit at time 0, node 0 is the root.
  let mut time = vec![0.0];
  let mut lineages: Vec<NodeId> = (1..=leaves)
    .map(|i| {
      time.push(0.0);
      tree.add_node(format!("A/Sim/{i}/2020"), None)
    })
    .collect();
  let mut now = 0.0;
  let mut remaining = f64::from(leaves);
  while lineages.len() > 1 {
    // Inverse transform sampling: 1 - u lies in (0, 1], so the waiting time is finite.
    now += -(1.0 - rng.r#gen::<f64>()).ln() / (remaining * (remaining - 1.0) / 2.0);
    let first = lineages.swap_remove(rng.gen_range(0..lineages.len()));
    let second = lineages.swap_remove(rng.gen_range(0..lineages.len()));
    let parent = if lineages.is_empty() {
      tree.root
    } else {
      time.push(0.0);
      tree.add_node("", None)
    };
    time[parent] = now;
    for child in [first, second] {
      tree.nodes[child].branch_length = Some(now - time[child]);
      tree.attach(parent, child);
    }
    if parent != tree.root {
      lineages.push(parent);
    }
    remaining -= 1.0;
  }
  tree
}

/// Prune a random subtree and regraft it onto the middle of a random branch elsewhere.
///
/// The pruned subtree leaves at least two leaves behind, so a regraft branch other than the
/// root always exists. The branch of the former sibling is excluded, because regrafting
/// there restores the tree; every other branch changes the rooted topology.
fn move_subtree(t: &mut Tree, rng: &mut impl Rng) {
  let order = t.postorder();
  let mut below = vec![0_usize; t.nodes.len()];
  for &n in &order {
    below[n] = if t.is_leaf(n) {
      1
    } else {
      t.children(n).iter().map(|&c| below[c]).sum()
    };
  }
  let total = below[t.root];
  let movable: Vec<NodeId> = order
    .into_iter()
    .filter(|&n| n != t.root && below[n] + 2 <= total)
    .collect();
  let n = movable[rng.gen_range(0..movable.len())];
  let sibling = t
    .parent(n)
    .and_then(|p| t.children(p).iter().copied().find(|&c| c != n));
  t.detach(n);
  t.remove_unary();
  let targets: Vec<NodeId> = t
    .preorder()
    .into_iter()
    .filter(|&c| c != t.root && Some(c) != sibling)
    .collect();
  let target = targets[rng.gen_range(0..targets.len())];
  let half = t.node(target).branch_length.map(|l| l / 2.0);
  let s = t.insert_above(target, String::new(), half);
  t.attach(s, n);
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::BTreeSet;
  use treeknit_core::Taxa;

  fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|&s| s.to_owned()).collect()
  }

  fn leaf_names(t: &Tree) -> BTreeSet<String> {
    t.leaf_names().into_iter().collect()
  }

  /// Clades of the reachable nodes, as sorted leaf names, comparable between trees.
  fn clades(t: &Tree) -> BTreeSet<Vec<String>> {
    let mut t = t.clone();
    let taxa = Taxa::from_trees(std::slice::from_ref(&t));
    t.assign_taxa(&taxa).unwrap();
    let c = t.clades(taxa.len());
    t.postorder()
      .into_iter()
      .map(|n| taxa.names_of(&c[n].ones().collect::<Vec<_>>()))
      .collect()
  }

  fn is_binary(t: &Tree) -> bool {
    t.internals().into_iter().all(|n| t.children(n).len() == 2)
  }

  #[test]
  fn test_large_tree_pair_args_defaults() {
    let expected = Args {
      leaves: 10000,
      outdir: PathBuf::from("tmp/large-pair"),
      seed: 1,
      moves: 10,
    };
    assert_eq!(Ok(expected), parse_args(&strings(&["10000", "tmp/large-pair"])));
  }

  #[test]
  fn test_large_tree_pair_args_seed_and_moves() {
    let expected = Args {
      leaves: 3,
      outdir: PathBuf::from("out"),
      seed: 7,
      moves: 0,
    };
    assert_eq!(Ok(expected), parse_args(&strings(&["3", "out", "7", "0"])));
  }

  #[test]
  fn test_large_tree_pair_args_rejects_too_few_leaves() {
    let err = parse_args(&strings(&["2", "out"])).unwrap_err();
    assert!(err.contains("below 3"), "{err}");
  }

  #[test]
  fn test_large_tree_pair_args_rejects_missing_outdir() {
    assert_eq!(Err(USAGE.to_owned()), parse_args(&strings(&["10"])));
  }

  #[test]
  fn test_large_tree_pair_args_rejects_extra_argument() {
    assert_eq!(
      Err(USAGE.to_owned()),
      parse_args(&strings(&["10", "out", "1", "2", "3"]))
    );
  }

  #[test]
  fn test_large_tree_pair_args_rejects_bad_number() {
    let err = parse_args(&strings(&["10", "out", "x"])).unwrap_err();
    assert!(err.starts_with("seed 'x'"), "{err}");
  }

  #[test]
  fn test_large_tree_pair_tree_a_binary_with_strain_names() {
    let (a, _) = tree_pair(50, 1, 0);
    let expected: BTreeSet<String> = (1..=50).map(|i| format!("A/Sim/{i}/2020")).collect();
    assert_eq!(expected, leaf_names(&a));
    assert!(is_binary(&a));
  }

  /// A coalescent tree is ultrametric: every leaf has the root time as its distance to the root.
  #[test]
  fn test_large_tree_pair_tree_a_ultrametric() {
    let (a, _) = tree_pair(200, 3, 0);
    let depths: Vec<f64> = a.leaves().into_iter().map(|n| a.divtime(n, a.root).unwrap()).collect();
    let first = depths[0];
    assert!(first > 0.0);
    assert!(depths.iter().all(|d| (d - first).abs() < 1e-12), "{depths:?}");
  }

  #[test]
  fn test_large_tree_pair_branch_lengths_nonnegative() {
    let (a, b) = tree_pair(200, 4, 20);
    let lengths = |t: &Tree| -> Vec<Option<f64>> {
      t.preorder()
        .into_iter()
        .filter(|&n| n != t.root)
        .map(|n| t.node(n).branch_length)
        .collect()
    };
    assert!(lengths(&a).into_iter().all(|l| l.is_some_and(|l| l >= 0.0)));
    assert!(lengths(&b).into_iter().all(|l| l.is_some_and(|l| l >= 0.0)));
  }

  #[test]
  fn test_large_tree_pair_same_seed_same_trees() {
    let (a1, b1) = tree_pair(100, 5, 10);
    let (a2, b2) = tree_pair(100, 5, 10);
    assert_eq!(newick::write(&a1), newick::write(&a2));
    assert_eq!(newick::write(&b1), newick::write(&b2));
  }

  #[test]
  fn test_large_tree_pair_other_seed_other_tree() {
    let (a1, _) = tree_pair(100, 5, 0);
    let (a2, _) = tree_pair(100, 6, 0);
    assert_ne!(clades(&a1), clades(&a2));
  }

  #[test]
  fn test_large_tree_pair_no_moves_same_tree() {
    let (a, b) = tree_pair(100, 7, 0);
    assert_eq!(newick::write(&a), newick::write(&b));
  }

  #[test]
  fn test_large_tree_pair_one_move_changes_smallest_tree() {
    let (a, b) = tree_pair(MIN_LEAVES, 8, 1);
    assert_eq!(leaf_names(&a), leaf_names(&b));
    assert!(is_binary(&b));
    assert_ne!(clades(&a), clades(&b));
  }

  #[test]
  fn test_large_tree_pair_one_move_changes_tree() {
    let (a, b) = tree_pair(200, 9, 1);
    assert_eq!(leaf_names(&a), leaf_names(&b));
    assert!(is_binary(&b));
    assert_ne!(clades(&a), clades(&b));
  }

  #[test]
  fn test_large_tree_pair_moves_keep_leaves_and_binary() {
    let (a, b) = tree_pair(300, 10, 25);
    assert_eq!(leaf_names(&a), leaf_names(&b));
    assert!(is_binary(&b));
    assert_eq!(a.n_leaves() - 1, b.internals().len());
  }

  #[test]
  fn test_large_tree_pair_newick_roundtrip() {
    let (a, b) = tree_pair(100, 11, 10);
    let a2 = newick::parse_first(&format!("{}\n", newick::write(&a)), "a").unwrap();
    let b2 = newick::parse_first(&format!("{}\n", newick::write(&b)), "b").unwrap();
    assert_eq!(clades(&a), clades(&a2));
    assert_eq!(clades(&b), clades(&b2));
  }
}
