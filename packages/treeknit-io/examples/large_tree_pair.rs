//! Pair of large random Newick trees for manual performance checks of the CLI and the web app.
//!
//! Tree A is a random binary tree with the topology and branch lengths (in coalescent units)
//! of a Kingman coalescent. Tree B is a copy of A changed by random subtree moves (prune and
//! regraft) that keep the node times, the way a reassortment changes one segment tree against
//! another, so both trees are ultrametric. Leaf names look
//! like strain names (`A/Sim/17/2020`). The program only writes input: it computes nothing
//! that TreeKnit reports.
//!
//! `just example large_tree_pair <leaves> <outdir> [seed] [moves]` writes `<outdir>/tree_a.nwk`
//! and `<outdir>/tree_b.nwk`. The seed defaults to 1 and the number of moves to 10; the same
//! arguments always write the same files.

use clap::Parser;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use std::path::PathBuf;
use std::process::ExitCode;
use treeknit_core::{NodeId, Tree};
use treeknit_io::newick;

/// Smallest number of leaves on which a subtree move changes the topology.
const MIN_LEAVES: u32 = 3;

/// Largest number of leaves, so that a mistyped count fails at once instead of exhausting the
/// memory. A million leaves is far beyond the trees TreeKnit is used on.
const MAX_LEAVES: u32 = 1_000_000;

#[derive(Debug, PartialEq, Parser)]
#[command(about = "Write a pair of large random Newick trees that differ by subtree moves")]
struct Args {
  /// Number of leaves of each tree
  #[arg(value_parser = clap::value_parser!(u32).range(i64::from(MIN_LEAVES)..=i64::from(MAX_LEAVES)))]
  leaves: u32,
  /// Directory of `tree_a.nwk` and `tree_b.nwk`
  outdir: PathBuf,
  /// Seed of the random numbers
  #[arg(default_value_t = 1)]
  seed: u64,
  /// Number of subtree moves from tree A to tree B
  #[arg(default_value_t = 10)]
  moves: u32,
}

fn main() -> ExitCode {
  match write_pair(&Args::parse()) {
    Ok(()) => ExitCode::SUCCESS,
    Err(e) => {
      eprintln!("large_tree_pair: {e}");
      ExitCode::FAILURE
    },
  }
}

/// Write both trees, then read each file back with the parser of the CLI.
fn write_pair(args: &Args) -> Result<(), String> {
  let (a, b) = tree_pair(args.leaves, args.seed, args.moves);
  std::fs::create_dir_all(&args.outdir).map_err(|e| format!("{}: {e}", args.outdir.display()))?;
  for (t, file) in [(&a, "tree_a.nwk"), (&b, "tree_b.nwk")] {
    let path = args.outdir.join(file);
    let io_err = |e: std::io::Error| format!("{}: {e}", path.display());
    std::fs::write(&path, format!("{}\n", newick::write(t))).map_err(io_err)?;
    let parsed = newick::parse_first(&std::fs::read_to_string(&path).map_err(io_err)?, file)
      .map_err(|e| e.to_string())?
      .tree;
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
    .map(|i| add_timed_node(&mut tree, &mut time, format!("A/Sim/{i}/2020")))
    .collect();
  let mut now = 0.0;
  while lineages.len() > 1 {
    // Inverse transform sampling: 1 - u lies in (0, 1], so the waiting time is finite.
    now += -(1.0 - rng.r#gen::<f64>()).ln() / lineage_pairs(lineages.len());
    let first = lineages.swap_remove(rng.gen_range(0..lineages.len()));
    let second = lineages.swap_remove(rng.gen_range(0..lineages.len()));
    let parent = if lineages.is_empty() {
      tree.root
    } else {
      add_timed_node(&mut tree, &mut time, String::new())
    };
    time[parent] = now;
    for child in [first, second] {
      tree.nodes[child].branch_length = Some(now - time[child]);
      tree.attach(parent, child);
    }
    if parent != tree.root {
      lineages.push(parent);
    }
  }
  tree
}

/// Add a node at time 0 to `tree` and to `time`, which is indexed by node id.
fn add_timed_node(tree: &mut Tree, time: &mut Vec<f64>, name: String) -> NodeId {
  let id = tree.add_node(name, None);
  assert_eq!(time.len(), id, "node ids must be sequential to index the node times");
  time.push(0.0);
  id
}

/// Number k(k-1)/2 of pairs of `k` lineages, the coalescence rate.
#[expect(
  clippy::as_conversions,
  reason = "at most MAX_LEAVES lineages, a count that f64 holds exactly"
)]
fn lineage_pairs(k: usize) -> f64 {
  let k = k as f64;
  k * (k - 1.0) / 2.0
}

/// Prune a random subtree and regraft it onto a random branch elsewhere, keeping node times so
/// that the tree stays ultrametric, as a reassortment keeps the sampling times.
///
/// The new parent of the subtree lies on the target branch, older than both the subtree root
/// and the lower end of the branch, so only branches whose upper end is older than the subtree
/// root are targets. The pruned subtree leaves at least two leaves behind, and a leaf always has
/// a target, so a move always exists. The branch of the former sibling is excluded, because
/// regrafting there restores the tree; every other branch changes the rooted topology.
fn move_subtree(t: &mut Tree, rng: &mut impl Rng) {
  let order = t.postorder();
  let mut below = vec![0_usize; t.nodes.len()];
  // Time of each node before the present, the leaves being at time 0.
  let mut time = vec![0.0_f64; t.nodes.len()];
  for &n in &order {
    if t.is_leaf(n) {
      below[n] = 1;
    } else {
      below[n] = t.children(n).iter().map(|&c| below[c]).sum();
      let c = t.children(n)[0];
      time[n] = time[c] + t.node(c).branch_length.unwrap_or(0.0);
    }
  }
  let total = below[t.root];
  let movable: Vec<NodeId> = order
    .into_iter()
    .filter(|&n| n != t.root && below[n] + 2 <= total)
    .collect();
  let (n, targets) = loop {
    let n = movable[rng.gen_range(0..movable.len())];
    let targets = regraft_targets(t, n, &time);
    if !targets.is_empty() {
      break (n, targets);
    }
  };
  let target = targets[rng.gen_range(0..targets.len())];
  let upper = t.parent(target).map_or(time[target], |p| time[p]);
  let joint = f64::midpoint(time[target].max(time[n]), upper);
  t.detach(n);
  t.remove_unary();
  let s = t.insert_above(target, String::new(), Some(joint - time[target]));
  t.nodes[n].branch_length = Some(joint - time[n]);
  t.attach(s, n);
}

/// Branches, by their lower node, onto which the subtree of `n` can move with node `time`s
/// kept: outside the subtree, not the branches that disappear or merge when `n` is pruned, and
/// with an upper end older than `n`.
fn regraft_targets(t: &Tree, n: NodeId, time: &[f64]) -> Vec<NodeId> {
  let parent = t.parent(n);
  let sibling = parent.and_then(|p| t.children(p).iter().copied().find(|&c| c != n));
  let mut inside = vec![false; t.nodes.len()];
  t.preorder()
    .into_iter()
    .filter(|&c| {
      inside[c] = c == n || t.parent(c).is_some_and(|p| inside[p]);
      !inside[c]
        && c != t.root
        && Some(c) != parent
        && Some(c) != sibling
        && t.parent(c).is_some_and(|p| time[p] > time[n])
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use clap::CommandFactory;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use std::collections::BTreeSet;
  use std::iter;
  use treeknit_core::Taxa;
  use treeknit_testing::assert_err;

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
  fn test_large_tree_pair_args_definition_is_valid() {
    Args::command().debug_assert();
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::defaults(      &["10000", "tmp/large-pair"],  (10000,   "tmp/large-pair", 1, 10))]
  #[case::seed_and_moves(&["3", "out", "7", "0"],        (3,       "out",            7, 0))]
  #[case::most_leaves(   &["1000000", "out"],            (1000000, "out",            1, 10))]
  #[trace]
  fn test_large_tree_pair_args_are_parsed(#[case] args: &[&str], #[case] (leaves, outdir, seed, moves): (u32, &str, u64, u32)) {
    let expected = Args { leaves, outdir: PathBuf::from(outdir), seed, moves };
    let parsed = Args::try_parse_from(iter::once("large_tree_pair").chain(args.iter().copied())).unwrap();
    assert_eq!(expected, parsed);
  }

  /// The end of a clap error that shows the usage.
  const USAGE: &str = "Usage: large_tree_pair <LEAVES> <OUTDIR> [SEED] [MOVES]\n\n";

  #[rustfmt::skip]
  #[rstest]
  #[case::too_few_leaves( &["2", "out"],                 ("invalid value '2' for '<LEAVES>': 2 is not in 3..=1000000",             ""))]
  #[case::too_many_leaves(&["1000001", "out"],           ("invalid value '1000001' for '<LEAVES>': 1000001 is not in 3..=1000000", ""))]
  #[case::missing_outdir( &["10"],                       ("the following required arguments were not provided:\n  <OUTDIR>",     USAGE))]
  #[case::extra_argument( &["10", "out", "1", "2", "3"], ("unexpected argument '3' found",                                         USAGE))]
  #[case::bad_number(     &["10", "out", "x"],           ("invalid value 'x' for '[SEED]': invalid digit found in string",         ""))]
  #[trace]
  fn test_large_tree_pair_args_are_rejected(#[case] args: &[&str], #[case] (error, usage): (&str, &str)) {
    let parsed = Args::try_parse_from(iter::once("large_tree_pair").chain(args.iter().copied()));
    assert_err!(parsed, format!("error: {error}\n\n{usage}For more information, try '--help'.\n"));
  }

  #[test]
  fn test_large_tree_pair_lineage_pairs() {
    assert_eq!(vec![0.0, 1.0, 3.0, 6.0], (1..=4).map(lineage_pairs).collect::<Vec<_>>());
  }

  #[test]
  fn test_large_tree_pair_tree_a_binary_with_strain_names() {
    let (a, _) = tree_pair(50, 1, 0);
    let expected: BTreeSet<String> = (1..=50).map(|i| format!("A/Sim/{i}/2020")).collect();
    assert_eq!(expected, leaf_names(&a));
    assert!(is_binary(&a));
  }

  /// Whether every leaf of `t` has the same positive distance to the root.
  fn is_ultrametric(t: &Tree) -> bool {
    let depths: Vec<f64> = t.leaves().into_iter().map(|n| t.divtime(n, t.root).unwrap()).collect();
    let first = depths[0];
    first > 0.0 && depths.iter().all(|d| (d - first).abs() < 1e-12)
  }

  /// A coalescent tree is ultrametric: every leaf has the root time as its distance to the root.
  #[test]
  fn test_large_tree_pair_tree_a_ultrametric() {
    let (a, _) = tree_pair(200, 3, 0);
    assert!(is_ultrametric(&a));
  }

  /// Moves keep the node times, so tree B stays ultrametric like tree A.
  #[rustfmt::skip]
  #[rstest]
  #[case::fewest_leaves((MIN_LEAVES, 1, 5))]
  #[case::ten_leaves(   (10,         2, 10))]
  #[case::many_leaves(  (200,        3, 50))]
  #[trace]
  fn test_large_tree_pair_tree_b_ultrametric(#[case] (leaves, seed, moves): (u32, u64, u32)) {
    let (_, b) = tree_pair(leaves, seed, moves);
    assert!(is_ultrametric(&b));
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

  /// For every seed, one move keeps the leaves and a binary tree B and changes its clades.
  #[rstest]
  #[trace]
  fn test_large_tree_pair_one_move_changes_tree(#[values(MIN_LEAVES, 4, 5, 10, 200)] leaves: u32) {
    let unchanged: Vec<u64> = (0..20)
      .filter(|&seed| {
        let (a, b) = tree_pair(leaves, seed, 1);
        leaf_names(&a) != leaf_names(&b) || !is_binary(&b) || clades(&a) == clades(&b)
      })
      .collect();
    assert_eq!(Vec::<u64>::new(), unchanged);
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
    let a2 = newick::parse_first(&format!("{}\n", newick::write(&a)), "a")
      .unwrap()
      .tree;
    let b2 = newick::parse_first(&format!("{}\n", newick::write(&b)), "b")
      .unwrap()
      .tree;
    assert_eq!(clades(&a), clades(&a2));
    assert_eq!(clades(&b), clades(&b2));
  }
}
