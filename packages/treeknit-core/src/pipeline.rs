//! TreeKnit on K ≥ 2 trees: pre-resolution, rounds of pair inference, polytomy sorting,
//! and placement of leaves missing from one tree of a pair.

#![expect(
  clippy::as_conversions,
  clippy::expect_used,
  reason = "findings from before the strict lint set; kb/issues/N-lint-baseline.md tracks their removal"
)]

mod imputed;
mod matching;
mod sorting;

pub use imputed::{arg_inputs, imputed_trees};
pub use matching::{match_topologies, unmatched_mccs};
pub use sorting::{keeps_run_order, last_sorting_pair, sort_for_pair, sort_strictness};

use crate::bits::Bits;
use crate::impute::Attachment;
use crate::naive::{Mcc, naive_mccs, sort_mccs};
use crate::options::{Options, Resolution};
use crate::pair::{PairParams, infer_pair};
use crate::progress::Progress;
use crate::resolve::{insert_all_on, resolve_trees, resolve_with_mccs};
use crate::tree::{Taxa, Tree};
use imputed::attach_pair;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use rayon::prelude::*;
use sorting::sort_pair;
use std::borrow::Cow;

/// Bit position of the round in the word that [`mix`] packs; the first tree index sits at
/// [`TREE_SHIFT`] and the second at bit 0, so up to 2^20 trees give distinct words.
const ROUND_SHIFT: u32 = 40;

/// Bit position of the first tree index in the word that [`mix`] packs.
const TREE_SHIFT: u32 = 20;

/// The increment of SplitMix64: 2^64 divided by the golden ratio, rounded to an odd number.
const SPLITMIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

/// First multiplier of the SplitMix64 output function.
const SPLITMIX_MUL_1: u64 = 0xBF58_476D_1CE4_E5B9;

/// Second multiplier of the SplitMix64 output function.
const SPLITMIX_MUL_2: u64 = 0x94D0_49BB_1331_11EB;

/// MCCs of one pair of trees.
#[derive(Clone, Debug)]
pub struct PairResult {
  pub i: usize,
  pub j: usize,
  /// MCCs over shared leaves, extended by attached private leaves; sorted.
  pub mccs: Vec<Mcc>,
  /// Leaves that are in exactly one tree of the pair, with their attachment.
  pub attached: Vec<Attachment>,
}

impl PairResult {
  /// MCCs restricted to the leaves that trees `i` and `j` of `trees` share; sorted. These are
  /// the MCCs the run sorted the pair by, before the leaves of one tree only were attached.
  /// After `Matched` resolution they hold the parts of every MCC that matching split, so they
  /// can differ from the MCCs inferred for the pair. Empty when the pair shares fewer than two
  /// leaves.
  ///
  /// # Panics
  ///
  /// If the trees `i` and `j` of the pair are not trees of `trees`, or have a leaf without a
  /// taxon of `0..n`.
  pub fn shared_mccs(&self, trees: &[Tree], n: usize) -> Vec<Mcc> {
    assert!(
      self.i < self.j && self.j < trees.len(),
      "({}, {}) is not a pair i < j of the {} trees",
      self.i,
      self.j,
      trees.len()
    );
    assert!(
      trees[self.i].has_taxa_below(n) && trees[self.j].has_taxa_below(n),
      "a tree of the pair has a leaf that is not one of the {n} taxa"
    );
    let Some(shared) = shared_leaves(&trees[self.i], &trees[self.j], n) else {
      return Vec::new();
    };
    let restricted = self
      .mccs
      .iter()
      .map(|m| m.iter().copied().filter(|&x| shared.contains(x)).collect::<Mcc>())
      .filter(|m| !m.is_empty())
      .collect();
    sort_mccs(restricted)
  }
}

/// Number of rounds of a run on `k` trees, and whether the last of them is the extra round
/// without resolution.
fn schedule(opts: &Options, k: usize) -> (usize, bool) {
  let extra_round = opts.extra_unresolved_round(k);
  (opts.rounds + extra_round as usize, extra_round)
}

/// Whether a round resolves the trees, and whether it adds only unambiguous splits with MCCs
/// (all splits in liberal mode). `unresolved_round` is the extra round without resolution.
fn round_mode(opts: &Options, unresolved_round: bool) -> (bool, bool) {
  let resolve = opts.resolves() && !unresolved_round;
  (resolve, opts.resolution != Resolution::Liberal && resolve)
}

/// Pairs of `k` trees in pipeline order (0,1), (0,2), …, (1,2), …
fn pipeline_pairs(k: usize) -> Vec<(usize, usize)> {
  (0..k).flat_map(|i| (i + 1..k).map(move |j| (i, j))).collect()
}

/// Run TreeKnit on `trees` (leaves must have taxa assigned from `taxa`).
/// Trees are resolved and sorted in place. Pairs are returned in order (0,1), (0,2), …
pub fn run(trees: &mut [Tree], taxa: &Taxa, opts: &Options, seed: u64) -> Vec<PairResult> {
  run_observed(trees, taxa, opts, seed, &|_| {})
}

/// As [`run`], calling `observe` with the progress of the run: at the start of each pair, after
/// each temperature step of its annealing, once when the matching of `Matched` resolution
/// starts, and once with fraction 1 at the end; every earlier event has a fraction of at most
/// [`PAIRS_SHARE`](crate::progress::PAIRS_SHARE). Pre-resolution reports nothing. In a parallel
/// round, `observe` is called on the calling thread only: before the round and after all its
/// pairs. The observer consumes no random numbers, so the result equals that of `run`.
pub fn run_observed(
  trees: &mut [Tree],
  taxa: &Taxa,
  opts: &Options,
  seed: u64,
  observe: &dyn Fn(Progress),
) -> Vec<PairResult> {
  let k = trees.len();
  let n = taxa.len();
  assert!(k >= 2, "need at least two trees");
  assert!(opts.seq_lengths.len() == k, "need one sequence length per tree");
  if opts.pre_resolve {
    log::info!("pre-resolving all trees with each other");
    let new = resolve_trees(trees, n);
    log::debug!(
      "new splits per tree: {:?}",
      new.iter().map(|s| s.len()).collect::<Vec<_>>()
    );
  }
  let pairs = pipeline_pairs(k);
  let mut mccs: Vec<Vec<Mcc>> = vec![Vec::new(); pairs.len()];
  let matched = opts.resolution == Resolution::Matched;
  let (rounds, extra_round) = schedule(opts, k);
  let sort_strict = sort_strictness(opts, k);
  let reached = std::cell::Cell::new(0.0);
  let report = |p: Progress| {
    reached.set(p.fraction);
    observe(p);
  };
  for round in 1..=rounds {
    let last = round == rounds;
    // Splits added with MCCs: unambiguous ones only, except in liberal mode.
    let (resolve, strict) = round_mode(opts, extra_round && last);
    log::info!("round {round}/{rounds}{}", if resolve { " (resolving)" } else { "" });
    let at = |pair: usize, within: f64| report(Progress::at(round - 1, rounds, pair, pairs.len(), within));
    let inference = Inference {
      opts,
      n,
      seed,
      round,
      resolve,
    };
    if resolve || !opts.parallel {
      for (p, &(i, j)) in pairs.iter().enumerate() {
        at(p, 0.0);
        mccs[p] = infer(trees, i, j, &inference, &|within| at(p, within));
        if resolve {
          resolve_pair(trees, i, j, &mccs[p], n, strict);
        }
        if last && !matched {
          sort_pair(trees, i, j, &mccs[p], n, sort_strict);
        }
      }
    } else {
      let shared: &[Tree] = trees;
      at(0, 0.0);
      mccs = pairs
        .par_iter()
        .map(|&(i, j)| infer(shared, i, j, &inference, &|_| {}))
        .collect();
      at(pairs.len() - 1, 1.0);
      if last && !matched {
        for (p, &(i, j)) in pairs.iter().enumerate() {
          sort_pair(trees, i, j, &mccs[p], n, sort_strict);
        }
      }
    }
  }
  if matched {
    report(Progress::matching(reached.get(), rounds, pairs.len()));
    match_topologies(trees, &pairs, &mut mccs, n);
    for (p, &(i, j)) in pairs.iter().enumerate() {
      sort_pair(trees, i, j, &mccs[p], n, sort_strict);
    }
  }
  let results = pairs
    .iter()
    .zip(mccs)
    .map(|(&(i, j), m)| attach_pair(trees, i, j, m, n))
    .collect();
  report(Progress::done(rounds, pairs.len()));
  results
}

/// Leaves common to trees `a` and `b`, or `None` if they share fewer than two leaves.
///
/// This is the only check of the shared-leaf count: a pair without two shared leaves has no
/// MCCs, and resolution and sorting leave its trees unchanged.
fn shared_leaves(a: &Tree, b: &Tree, n: usize) -> Option<Bits> {
  let shared = &a.leaf_set(n) & &b.leaf_set(n);
  (shared.count_ones(..) >= 2).then_some(shared)
}

/// Tree `t` restricted to the `shared` leaves of a pair (borrowed if no restriction is needed).
/// `shared` comes from `shared_leaves`, so it holds at least two leaves of `t`.
fn restricted_to_shared<'a>(t: &'a Tree, shared: &Bits, n: usize) -> Cow<'a, Tree> {
  if t.leaf_set(n) == *shared {
    Cow::Borrowed(t)
  } else {
    Cow::Owned(t.restricted(shared).expect("shared leaves are in both trees"))
  }
}

/// Settings that all pairs of one round infer their MCCs with.
struct Inference<'a> {
  opts: &'a Options,
  /// Number of taxa.
  n: usize,
  /// Seed of the run, mixed with the round and the pair into the seed of each pair.
  seed: u64,
  /// Round, 1-based.
  round: usize,
  /// Whether the round resolves the trees with the MCCs it infers.
  resolve: bool,
}

/// Infer the MCCs of trees `i` and `j` on their shared leaves; none if they share fewer than two.
fn infer(trees: &[Tree], i: usize, j: usize, inference: &Inference<'_>, on_progress: &dyn Fn(f64)) -> Vec<Mcc> {
  let &Inference {
    opts,
    n,
    seed,
    round,
    resolve,
  } = inference;
  let Some(shared) = shared_leaves(&trees[i], &trees[j], n) else {
    log::warn!(
      "trees {} and {} share fewer than two leaves: skipped",
      trees[i].label,
      trees[j].label
    );
    return Vec::new();
  };
  log::info!(
    "inferring MCCs for {} and {} ({} shared leaves)",
    trees[i].label,
    trees[j].label,
    shared.count_ones(..)
  );
  let ti = restricted_to_shared(&trees[i], &shared, n);
  let tj = restricted_to_shared(&trees[j], &shared, n);
  let m = if opts.naive {
    naive_mccs(&[&ti, &tj], n)
  } else {
    let p = PairParams {
      gamma: opts.gamma,
      itmax: opts.itmax,
      likelihood_sort: opts.likelihood_sort,
      resolve,
      seq_lengths: [opts.seq_lengths[i], opts.seq_lengths[j]],
      n_mcmc: opts.n_mcmc,
      sa_rep: opts.sa_rep,
      temperatures: opts.temperatures(),
    };
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(mix(seed, round, i, j));
    infer_pair(&ti, &tj, n, &p, &mut rng, on_progress)
  };
  log::info!("found {} MCCs for {} and {}", m.len(), trees[i].label, trees[j].label);
  m
}

/// Per-pair seed, independent of execution order: the round and the tree indices, packed into one
/// word and multiplied by [`SPLITMIX_GAMMA`], are mixed into the seed by the output function of
/// SplitMix64 (Steele, Lea, and Flood 2014, "Fast splittable pseudorandom number generators").
fn mix(seed: u64, round: usize, i: usize, j: usize) -> u64 {
  let pair = ((round as u64) << ROUND_SHIFT) | ((i as u64) << TREE_SHIFT) | j as u64;
  let mut z = seed ^ pair.wrapping_mul(SPLITMIX_GAMMA);
  z = (z ^ (z >> 30)).wrapping_mul(SPLITMIX_MUL_1);
  z = (z ^ (z >> 27)).wrapping_mul(SPLITMIX_MUL_2);
  z ^ (z >> 31)
}

/// Resolve trees `i` and `j` with their MCCs (computed on shared leaves). A pair that shares
/// fewer than two leaves is left unchanged.
/// Returns the number of splits added to the two trees.
fn resolve_pair(trees: &mut [Tree], i: usize, j: usize, mccs: &[Mcc], n: usize, strict: bool) -> usize {
  let Some(shared) = shared_leaves(&trees[i], &trees[j], n) else {
    return 0;
  };
  let ti = restricted_to_shared(&trees[i], &shared, n);
  let tj = restricted_to_shared(&trees[j], &shared, n);
  let (mut ti, mut tj) = (ti.into_owned(), tj.into_owned());
  let [mut si, mut sj] = resolve_with_mccs(&mut ti, &mut tj, mccs, n, strict);
  log::debug!(
    "resolved {} new splits in {}, {} in {}",
    si.len(),
    trees[i].label,
    sj.len(),
    trees[j].label
  );
  insert_all_on(&mut trees[i], &mut si, &shared, n);
  insert_all_on(&mut trees[j], &mut sj, &shared, n);
  si.len() + sj.len()
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::pipeline::matching::mismatched_restrictions;
  use crate::progress::{PAIRS_SHARE, Phase, ratio};
  use crate::tree::test_util::{splits, trees};
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  pub(crate) fn ids(taxa: &Taxa, m: &[&[&str]]) -> Vec<Mcc> {
    sort_mccs(m.iter().map(|x| x.iter().map(|s| taxa.index[*s]).collect()).collect())
  }

  #[test]
  fn three_trees_better_trees() {
    let (mut ts, taxa) = trees(&["((A,(B,C)),(D,E));", "((A,B,C,D),E);", "((A,B),((C,D),E));"]);
    let o = Options::treeknit_jl(3, None);
    let res = run(&mut ts, &taxa, &o, 1);
    // Every pair's MCCs hold all five leaves.
    let totals: Vec<usize> = res.iter().map(|r| r.mccs.iter().map(Vec::len).sum()).collect();
    assert_eq!(vec![5, 5, 5], totals);
  }

  /// Run `f` in a pool of four threads when `parallel`, so that the pairs of a parallel run run
  /// concurrently although the global pool of the tests has one thread.
  fn in_pool<T: Send>(parallel: bool, f: impl FnOnce() -> T + Send) -> T {
    if parallel {
      rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap()
        .install(f)
    } else {
      f()
    }
  }

  /// Two trees that share fewer than two leaves get no MCCs and no attachments and stay
  /// unchanged, in every resolution mode, sequentially and in parallel. Ladderizing either tree
  /// changes its leaf order, so an unchanged order shows that the pair was not sorted.
  #[rustfmt::skip]
  #[rstest]
  #[case::no_shared_leaf( ["(A,((B,C),D));", "(P,((Q,R),S));"])]
  #[case::one_shared_leaf(["(A,((B,C),D));", "(A,((Q,R),S));"])]
  #[trace]
  fn pair_sharing_fewer_than_two_leaves_is_skipped(
    #[case] nwks: [&str; 2],
    #[values(Resolution::None, Resolution::Strict, Resolution::Liberal, Resolution::Matched)] resolution: Resolution,
    #[values(false, true)] parallel: bool,
  ) {
    let (mut ts, taxa) = trees(&nwks);
    let before: Vec<_> = ts.iter().map(|t| (t.leaf_names(), splits(t, &taxa))).collect();
    let o = Options {
      resolution,
      parallel,
      pre_resolve: true,
      ..Options::for_trees(2)
    };
    let res = in_pool(parallel, || run(&mut ts, &taxa, &o, 1));
    let after: Vec<_> = ts.iter().map(|t| (t.leaf_names(), splits(t, &taxa))).collect();
    let skipped = res.iter().map(|r| (r.mccs.len(), r.attached.len())).collect::<Vec<_>>();
    assert_eq!((vec![(0, 0)], before), (skipped, after));
  }

  #[test]
  fn skipped_pairs_leave_other_pairs_unaffected() {
    // Tree 2 shares one leaf (Y) with tree 0 and none with tree 1, so the run skips one pair of
    // each kind. The pair (0, 1) gets the MCCs that it gets without tree 2: the same per-pair
    // seed, and no resolution across pairs. Y is in tree 0 only, inside the clade (B,Y) of the
    // MCC {A,B,C,D}, so attachment adds it to that MCC.
    let pair = ["((A,(B,Y)),(C,(D,X)));", "((A,(B,X)),(C,D));"];
    let o = |k| Options {
      resolution: Resolution::None,
      ..Options::for_trees(k)
    };
    let names = |r: &PairResult, t: &Taxa| r.mccs.iter().map(|m| t.names_of(m)).collect::<Vec<_>>();
    let (mut ts2, taxa2) = trees(&pair);
    let expected = run(&mut ts2, &taxa2, &o(2), 1);
    let (mut ts3, taxa3) = trees(&[pair[0], pair[1], "(Y,(P,Q));"]);
    let res = run(&mut ts3, &taxa3, &o(3), 1);
    assert_eq!(
      res.iter().map(|r| (r.i, r.j)).collect::<Vec<_>>(),
      [(0, 1), (0, 2), (1, 2)]
    );
    assert_eq!(names(&res[0], &taxa3), names(&expected[0], &taxa2));
    assert_eq!(names(&res[0], &taxa3), [vec!["X"], vec!["A", "B", "C", "D", "Y"]]);
    assert!(res[1].mccs.is_empty());
    assert!(res[2].mccs.is_empty());
  }

  /// The two-tree example: X moved between the trees.
  pub(crate) const TWO_TREE_EXAMPLE: [&str; 2] = ["((A,B),(C,(D,X)));", "((A,(B,X)),(C,D));"];

  /// A polytomy in the first tree.
  pub(crate) const POLYTOMY_PAIR: [&str; 2] = ["((A,B,C),(D,X),E);", "((A,(B,X)),(C,D),E);"];

  /// Leaves in one tree only: P in the first tree, Q in the second.
  pub(crate) const ONE_TREE_LEAVES: [&str; 2] = ["((A,B),((C,P),(D,X)));", "((A,(B,X)),(C,(D,Q)));"];

  /// The two-tree inputs of the display sort.
  pub(crate) const PAIR_INPUTS: [[&str; 2]; 3] = [TWO_TREE_EXAMPLE, POLYTOMY_PAIR, ONE_TREE_LEAVES];

  #[test]
  fn shared_mccs_drop_the_leaves_of_one_tree_only() {
    let (mut ts, taxa) = trees(&ONE_TREE_LEAVES);
    let o = Options {
      resolution: Resolution::None,
      ..Options::for_trees(2)
    };
    let res = run(&mut ts, &taxa, &o, 1);
    assert_eq!(2, res[0].attached.len());
    let shared = res[0].shared_mccs(&ts, taxa.len());
    assert_eq!(ids(&taxa, &[&["X"], &["A", "B", "C", "D"]]), shared);
    let attached: Vec<usize> = res[0].mccs.iter().map(Vec::len).collect();
    assert_eq!(vec![1, 6], attached);
  }

  /// Number in the log line of `lines` that starts with `before` and has `after` right after
  /// the number.
  fn logged_count(lines: &[String], before: &str, after: &str) -> Vec<usize> {
    lines
      .iter()
      .filter_map(|l| l.strip_prefix(before)?.strip_suffix(after)?.parse().ok())
      .collect()
  }

  #[test]
  fn shared_mccs_are_the_parts_after_matching_splits_an_mcc() {
    // With seed 1, matching finds conflicting topologies inside an MCC of this run and splits
    // it, and the run sorts the pair by the parts. The run's log gives the expected number of
    // MCCs per pair: the MCCs it inferred, plus one per part beyond the first of each split.
    let (mut ts, taxa) = trees(&["((D,A),(B,E),C);", "(E,(B,C),(D,A));", "((B,C),D,(E,A));"]);
    for (t, label) in ts.iter_mut().zip(["t0", "t1", "t2"]) {
      label.clone_into(&mut t.label);
    }
    let n = taxa.len();
    let o = Options {
      n_t: 10,
      ..Options::for_trees(3)
    };
    assert_eq!(Resolution::Matched, o.resolution);
    let mut res = Vec::new();
    let lines = logged(|| res = run(&mut ts, &taxa, &o, 1));
    let counts = |r: &PairResult| {
      let pair = format!("{} and {}", ts[r.i].label, ts[r.j].label);
      let inferred = logged_count(&lines, "INFO found ", &format!(" MCCs for {pair}"));
      let conflict = format!("conflicting topologies in {pair}: split into ");
      let parts: Vec<usize> = lines
        .iter()
        .filter_map(|l| l.split_once(&conflict)?.1.strip_suffix(" MCCs")?.parse().ok())
        .collect();
      (inferred, parts)
    };
    let logged_counts: Vec<_> = res.iter().map(counts).collect();
    assert!(
      logged_counts.iter().any(|(_, parts)| !parts.is_empty()),
      "no MCC was split: {lines:?}"
    );
    let expected: Vec<usize> = logged_counts
      .iter()
      .map(|(inferred, parts)| inferred.iter().sum::<usize>() + parts.iter().map(|p| p - 1).sum::<usize>())
      .collect();
    let shared: Vec<Vec<Mcc>> = res.iter().map(|r| r.shared_mccs(&ts, n)).collect();
    assert_eq!(expected, shared.iter().map(Vec::len).collect::<Vec<_>>());
    // The MCC that matching split still has conflicting topologies in the final trees, so only
    // its parts agree within every MCC.
    let conflicting: Vec<(usize, usize, Mcc)> = res
      .iter()
      .zip(&shared)
      .flat_map(|(r, ms)| {
        ms.iter()
          .filter(|m| mismatched_restrictions(&ts[r.i], &ts[r.j], m, n).is_some())
          .map(|m| (r.i, r.j, m.clone()))
      })
      .collect();
    assert_eq!(Vec::<(usize, usize, Mcc)>::new(), conflicting);
  }

  /// Log lines, each with the thread that logged it.
  type Records = Vec<(std::thread::ThreadId, String)>;

  /// Logger that keeps every record, with the thread that logged it, since other tests may log
  /// from other threads of the same process.
  #[expect(
    clippy::disallowed_types,
    reason = "the lock of a test logger; the crate does not depend on parking_lot"
  )]
  struct Capture(std::sync::Mutex<Records>);

  impl log::Log for Capture {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
      true
    }

    fn log(&self, record: &log::Record<'_>) {
      let line = format!("{} {}", record.level(), record.args());
      self.0.lock().unwrap().push((std::thread::current().id(), line));
    }

    fn flush(&self) {}
  }

  #[expect(
    clippy::disallowed_types,
    reason = "the lock of a test logger; the crate does not depend on parking_lot"
  )]
  static CAPTURE: Capture = Capture(std::sync::Mutex::new(Vec::new()));

  /// Lines that `f` logs on this thread.
  pub(crate) fn logged(f: impl FnOnce()) -> Vec<String> {
    static INSTALL: std::sync::Once = std::sync::Once::new();
    INSTALL.call_once(|| {
      log::set_logger(&CAPTURE).unwrap();
      log::set_max_level(log::LevelFilter::Trace);
    });
    let me = std::thread::current().id();
    CAPTURE.0.lock().unwrap().retain(|(t, _)| *t != me);
    f();
    let records = CAPTURE.0.lock().unwrap();
    records
      .iter()
      .filter(|(t, _)| *t == me)
      .map(|(_, l)| l.clone())
      .collect()
  }

  /// `k` trees on the same leaves.
  pub(crate) fn same_leaves(k: usize) -> (Vec<Tree>, Taxa) {
    trees(&vec!["((A,B),(C,D));"; k])
  }

  #[test]
  #[should_panic(expected = "(0, 2) is not a pair i < j of the 2 trees")]
  fn shared_mccs_reject_a_pair_out_of_range() {
    let (ts, taxa) = same_leaves(2);
    let pair = PairResult {
      i: 0,
      j: 2,
      mccs: Vec::new(),
      attached: Vec::new(),
    };
    pair.shared_mccs(&ts, taxa.len());
  }

  #[test]
  #[should_panic(expected = "a tree of the pair has a leaf that is not one of the 3 taxa")]
  fn shared_mccs_reject_a_tree_leaf_out_of_range() {
    let (ts, _) = same_leaves(2);
    let pair = PairResult {
      i: 0,
      j: 1,
      mccs: Vec::new(),
      attached: Vec::new(),
    };
    pair.shared_mccs(&ts, 3);
  }

  /// Progress events of a run of `nwks` with `opts` and seed 1.
  fn observed(nwks: &[&str], opts: &Options) -> Vec<Progress> {
    let (mut ts, taxa) = trees(nwks);
    let events = std::cell::RefCell::new(Vec::new());
    run_observed(&mut ts, &taxa, opts, 1, &|p| events.borrow_mut().push(p));
    events.into_inner()
  }

  fn never_decreasing(events: &[Progress]) -> bool {
    events
      .iter()
      .zip(events.iter().skip(1))
      .all(|(a, b)| a.fraction <= b.fraction)
  }

  /// Ten leaves whose MCC inference with seed 1 needs two prune iterations: B, D, G, and I
  /// swap places between the two trees.
  const TWO_ITERATIONS: [&str; 2] = [
    "(((((A,B),C),D),E),((((F,G),H),I),J));",
    "(((((A,G),C),I),E),((((F,B),H),D),J));",
  ];

  /// Options for one pair with `itmax` and ten temperatures, so that a test run stays short.
  fn one_pair(itmax: usize) -> Options {
    Options {
      itmax,
      n_t: 10,
      resolution: Resolution::Strict,
      parallel: false,
      ..Options::for_trees(2)
    }
  }

  /// Number of iterations that MCC inference completed in a run of one pair with `itmax`,
  /// counted from its progress events: the last temperature step of iteration `k` reports the
  /// completed fraction `k / (itmax + 1)` of the pair, and no other step reports a multiple of
  /// `1 / (itmax + 1)`.
  fn iterations(events: &[Progress], itmax: usize) -> usize {
    let max = itmax + 1;
    (1..=max)
      .filter(|&k| {
        let end = Progress::at(0, 1, 0, 1, ratio(k, max));
        events
          .iter()
          .any(|p| p.phase == Phase::Pairs && p.fraction.to_bits() == end.fraction.to_bits())
      })
      .count()
  }

  #[test]
  fn progress_of_a_pair_that_runs_all_iterations_rises_to_the_pairs_share() {
    // With itmax = 1, pair inference runs both of its itmax + 1 = 2 iterations.
    let events = observed(&TWO_ITERATIONS, &one_pair(1));
    assert_eq!(2, iterations(&events, 1), "{events:?}");
    assert_eq!(Some(&Progress::at(0, 1, 0, 1, 0.0)), events.first());
    assert_eq!(Some(&Progress::done(1, 1)), events.last());
    assert!(never_decreasing(&events), "{events:?}");
    // With one round and one pair, the fraction is the pairs' share of the completed fraction of
    // the pair: the last temperature step of iteration 1 completes 1/2, that of iteration 2
    // completes all of it.
    let before_end = &events[..events.len() - 1];
    assert!(before_end.iter().all(|p| p.fraction <= PAIRS_SHARE), "{events:?}");
    let half = PAIRS_SHARE * 0.5;
    assert!(
      before_end.iter().any(|p| p.fraction.to_bits() == half.to_bits()),
      "{events:?}"
    );
    assert_eq!(Some(PAIRS_SHARE), before_end.last().map(|p| p.fraction), "{events:?}");
  }

  #[test]
  fn progress_of_a_pair_that_stops_early_jumps_to_done_at_the_end() {
    // With itmax = 2 the same pair stops by itself after two of its three iterations, at 2/3.
    let events = observed(&TWO_ITERATIONS, &one_pair(2));
    assert_eq!(2, iterations(&events, 2), "{events:?}");
    let before_end = &events[..events.len() - 1];
    assert!(never_decreasing(&events), "{events:?}");
    assert_eq!(
      Some(PAIRS_SHARE * (2.0 / 3.0)),
      before_end.last().map(|p| p.fraction),
      "{events:?}"
    );
    assert_eq!(Some(&Progress::done(1, 1)), events.last());
  }

  #[test]
  fn progress_of_sequential_rounds_visits_every_pair_in_order() {
    // Strict resolution with three trees adds a final round without resolution: two rounds.
    let o = Options {
      itmax: 2,
      n_t: 5,
      resolution: Resolution::Strict,
      parallel: false,
      ..Options::for_trees(3)
    };
    let events = observed(&["((A,B),(C,(D,X)));", "((A,(B,X)),(C,D));", "((A,X),(B,(C,D)));"], &o);
    let mut visited: Vec<(usize, usize, usize, usize)> =
      events.iter().map(|p| (p.round, p.rounds, p.pair, p.pairs)).collect();
    visited.dedup();
    let expected = [
      (1, 2, 1, 3),
      (1, 2, 2, 3),
      (1, 2, 3, 3),
      (2, 2, 1, 3),
      (2, 2, 2, 3),
      (2, 2, 3, 3),
    ];
    assert_eq!(expected.as_slice(), visited);
    assert!(never_decreasing(&events), "{events:?}");
    let before_end = &events[..events.len() - 1];
    assert!(before_end.iter().all(|p| p.fraction <= PAIRS_SHARE), "{events:?}");
    assert_eq!(Some(&Progress::done(2, 3)), events.last());
  }

  #[test]
  fn progress_of_parallel_rounds_is_reported_before_and_after_each_batch() {
    let o = Options {
      rounds: 2,
      resolution: Resolution::None,
      parallel: true,
      ..Options::for_trees(3)
    };
    let nwks = ["((A,B),(C,(D,X)));", "((A,(B,X)),(C,D));", "((A,X),(B,(C,D)));"];
    let events = in_pool(true, || observed(&nwks, &o));
    // Fractions PAIRS_SHARE * (round + (pair + within) / 3) / 2: 0, (0 + 3 / 3) / 2 = 0.5,
    // (1 + 0) / 2 = 0.5, (1 + 3 / 3) / 2 = 1 of the share, and 1 at the end.
    let expected = [
      Progress {
        phase: Phase::Pairs,
        fraction: 0.0,
        round: 1,
        rounds: 2,
        pair: 1,
        pairs: 3,
      },
      Progress {
        phase: Phase::Pairs,
        fraction: PAIRS_SHARE * 0.5,
        round: 1,
        rounds: 2,
        pair: 3,
        pairs: 3,
      },
      Progress {
        phase: Phase::Pairs,
        fraction: PAIRS_SHARE * 0.5,
        round: 2,
        rounds: 2,
        pair: 1,
        pairs: 3,
      },
      Progress {
        phase: Phase::Pairs,
        fraction: PAIRS_SHARE,
        round: 2,
        rounds: 2,
        pair: 3,
        pairs: 3,
      },
      Progress {
        phase: Phase::Done,
        fraction: 1.0,
        round: 2,
        rounds: 2,
        pair: 3,
        pairs: 3,
      },
    ];
    assert_eq!(expected.as_slice(), events);
  }

  /// Three trees with polytomies: pre-resolution adds the split (C,D), which the other two trees
  /// have, to the first tree, and the placements of X differ, so the pairs have several MCCs.
  const POLYTOMIES: [&str; 3] = ["((A,B),(C,D,X));", "((A,(B,X)),(C,D));", "((A,X),B,(C,D));"];

  #[test]
  fn pre_resolution_changes_the_trees_of_the_progress_bounds() {
    // Oracle: (C,D) is a split of the second and third tree and fits the polytomy (C,D,X) of the
    // first; no other split is compatible with all three trees and missing from one.
    let (mut ts, taxa) = trees(&POLYTOMIES);
    let new = resolve_trees(&mut ts, taxa.len());
    let added: Vec<usize> = new.iter().map(Vec::len).collect();
    assert_eq!(vec![1, 0, 0], added);
  }

  /// A run of `POLYTOMIES` in `resolution`, `parallel`, and `pre_resolve` reports
  /// fractions that never decrease and stay at most `PAIRS_SHARE` before the last event, which is
  /// `Done` with fraction 1, and the phases `Pairs`, then `Matching` for `Matched` resolution, then
  /// `Done`.
  ///
  /// Matched resolution resolves in every round, so its rounds run sequentially with or without
  /// `parallel` and have no parallel cases.
  #[rustfmt::skip]
  #[rstest]
  #[case::none_sequential(                (Resolution::None,    false, false))]
  #[case::none_sequential_pre_resolved(   (Resolution::None,    false, true))]
  #[case::none_parallel(                  (Resolution::None,    true,  false))]
  #[case::none_parallel_pre_resolved(     (Resolution::None,    true,  true))]
  #[case::strict_sequential(              (Resolution::Strict,  false, false))]
  #[case::strict_sequential_pre_resolved( (Resolution::Strict,  false, true))]
  #[case::strict_parallel(                (Resolution::Strict,  true,  false))]
  #[case::strict_parallel_pre_resolved(   (Resolution::Strict,  true,  true))]
  #[case::liberal_sequential(             (Resolution::Liberal, false, false))]
  #[case::liberal_sequential_pre_resolved((Resolution::Liberal, false, true))]
  #[case::liberal_parallel(               (Resolution::Liberal, true,  false))]
  #[case::liberal_parallel_pre_resolved(  (Resolution::Liberal, true,  true))]
  #[case::matched(                        (Resolution::Matched, false, false))]
  #[case::matched_pre_resolved(           (Resolution::Matched, false, true))]
  #[trace]
  fn progress_bounds(#[case] (resolution, parallel, pre_resolve): (Resolution, bool, bool)) {
    let o = Options {
      itmax: 2,
      n_t: 5,
      resolution,
      parallel,
      pre_resolve,
      ..Options::for_trees(3)
    };
    let events = in_pool(parallel, || observed(&POLYTOMIES, &o));
    assert!(never_decreasing(&events), "{events:?}");
    let (last, before_end) = events.split_last().unwrap();
    assert!(
      before_end
        .iter()
        .all(|p| p.phase != Phase::Done && p.fraction <= PAIRS_SHARE),
      "{events:?}"
    );
    assert_eq!((Phase::Done, 1.0_f64.to_bits()), (last.phase, last.fraction.to_bits()));
    let mut phases: Vec<Phase> = events.iter().map(|p| p.phase).collect();
    phases.dedup();
    let expected = if resolution == Resolution::Matched {
      vec![Phase::Pairs, Phase::Matching, Phase::Done]
    } else {
      vec![Phase::Pairs, Phase::Done]
    };
    assert_eq!(expected, phases, "{events:?}");
  }

  /// Events of a run with `Matched` resolution (whose rounds resolve, so they run sequentially):
  /// the pair events, the start of matching, and the end.
  fn matched_events() -> (Vec<Progress>, Progress, Progress) {
    let o = Options {
      n_t: 5,
      ..Options::for_trees(3)
    };
    assert_eq!(Resolution::Matched, o.resolution);
    let mut events = observed(&["((A,B),(C,(D,X)));", "((A,(B,X)),(C,D));", "((A,X),(B,(C,D)));"], &o);
    let done = events.pop().unwrap();
    let matching = events.pop().unwrap();
    (events, matching, done)
  }

  #[test]
  fn progress_of_matched_resolution_reports_matching_at_the_fraction_reached() {
    let (pairs, matching, done) = matched_events();
    assert!(pairs.iter().all(|p| p.phase == Phase::Pairs), "{pairs:?}");
    assert!(never_decreasing(&pairs), "{pairs:?}");
    let reached = pairs.last().unwrap().fraction;
    assert_eq!(Progress::matching(reached, 1, 3), matching);
    assert_eq!(Progress::done(1, 3), done);
  }
}
