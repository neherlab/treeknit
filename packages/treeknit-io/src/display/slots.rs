//! Color slots of the MCCs of a pair: a deterministic greedy coloring of the graph whose edges
//! join MCCs with neighboring blocks.

use super::{Block, MCC_SLOTS};
use std::collections::BTreeSet;

/// For each of `n_mccs` MCCs, the MCCs it neighbors: two MCCs are neighbors when two of their
/// blocks are consecutive in the left order or in the right order. `blocks` are in left order.
pub(super) fn block_neighbors(n_mccs: usize, blocks: &[Block]) -> Vec<BTreeSet<usize>> {
  let mut right: Vec<&Block> = blocks.iter().collect();
  right.sort_by(|a, b| low(a.right).total_cmp(&low(b.right)));
  let mut neighbors = vec![BTreeSet::new(); n_mccs];
  for order in [blocks.iter().collect::<Vec<_>>(), right] {
    for pair in order.windows(2) {
      let [first, second] = pair else { continue };
      let (a, b) = (first.mcc, second.mcc);
      if a != b {
        neighbors[a].insert(b);
        neighbors[b].insert(a);
      }
    }
  }
  neighbors
}

/// The first row of a y range.
fn low(range: [f64; 2]) -> f64 {
  range[0].min(range[1])
}

/// One slot in `0..MCC_SLOTS` per MCC, from the MCC sizes and the neighbor sets. MCCs are
/// visited by size, largest first, then by index; each takes the lowest slot that no colored
/// neighbor uses, or, when every slot is used, the slot used least among its colored neighbors
/// (the lowest on ties). Neighbors therefore share a slot only when an MCC has at least
/// `MCC_SLOTS` colored neighbors.
pub(super) fn color_slots(sizes: &[usize], neighbors: &[BTreeSet<usize>]) -> Vec<usize> {
  let mut order: Vec<usize> = (0..sizes.len()).collect();
  order.sort_by_key(|&m| (std::cmp::Reverse(sizes[m]), m));
  let mut slot: Vec<Option<usize>> = vec![None; sizes.len()];
  for m in order {
    let mut uses = [0_usize; MCC_SLOTS];
    for &b in &neighbors[m] {
      if let Some(s) = slot[b] {
        uses[s] += 1;
      }
    }
    // `min_by_key` keeps the first of equal keys, so ties go to the lowest slot; an unused
    // slot has the least uses of all.
    let least = (0..MCC_SLOTS).min_by_key(|&s| uses[s]).unwrap_or(0);
    slot[m] = Some(least);
  }
  slot.into_iter().map(|s| s.unwrap_or(0)).collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use rand::{Rng, SeedableRng};
  use rand_xoshiro::Xoshiro256PlusPlus;

  fn block(mcc: usize, left: f64, right: f64) -> Block {
    Block {
      mcc,
      left: [left, left],
      right: [right, right],
    }
  }

  fn graph(n: usize, edges: &[(usize, usize)]) -> Vec<BTreeSet<usize>> {
    let mut g = vec![BTreeSet::new(); n];
    for &(a, b) in edges {
      g[a].insert(b);
      g[b].insert(a);
    }
    g
  }

  #[test]
  fn block_neighbors_join_consecutive_blocks_in_either_order() {
    // Left order: 0, 1, 2, 0. Right order: 2, 0, 1, 0 (rows 0, 1, 2, 3).
    let blocks = [
      block(0, 0.0, 1.0),
      block(1, 1.0, 2.0),
      block(2, 2.0, 0.0),
      block(0, 3.0, 3.0),
    ];
    let expected = graph(3, &[(0, 1), (1, 2), (2, 0)]);
    assert_eq!(expected, block_neighbors(3, &blocks));
  }

  #[test]
  fn block_neighbors_ignore_consecutive_blocks_of_one_mcc() {
    let blocks = [block(0, 0.0, 1.0), block(0, 1.0, 0.0)];
    assert_eq!(graph(1, &[]), block_neighbors(1, &blocks));
  }

  #[test]
  fn color_slots_visit_the_largest_mcc_first() {
    // MCC 1 is largest and takes slot 0; MCC 0 and MCC 2 neighbor it and take slot 1.
    let slots = color_slots(&[1, 5, 1], &graph(3, &[(0, 1), (1, 2)]));
    assert_eq!(vec![1, 0, 1], slots);
  }

  #[test]
  fn color_slots_break_size_ties_by_index() {
    let slots = color_slots(&[2, 2], &graph(2, &[(0, 1)]));
    assert_eq!(vec![0, 1], slots);
  }

  #[test]
  fn color_slots_of_an_mcc_with_nine_colored_neighbors_take_the_least_used_slot() {
    // Neighbors 1..=9 are larger, so they are colored first. A clique 1..=8 takes slots 0..=7
    // in index order; neighbor 9, adjacent to 1 only, takes slot 1.
    let sizes = [1, 10, 10, 10, 10, 10, 10, 10, 10, 10];
    let mut edges: Vec<(usize, usize)> = (1..=9).map(|b| (0, b)).collect();
    for a in 1..=8 {
      for b in a + 1..=8 {
        edges.push((a, b));
      }
    }
    edges.push((9, 1));
    let slots = color_slots(&sizes, &graph(10, &edges));
    assert_eq!(vec![0, 1, 2, 3, 4, 5, 6, 7], slots[1..=8].to_vec());
    assert_eq!(1, slots[9]);
    // Around MCC 0, slot 1 is used twice and every other slot once: the lowest of them is 0.
    assert_eq!(0, slots[0]);
  }

  #[test]
  fn color_slots_fallback_prefers_the_least_used_over_the_lowest() {
    // Neighbors 1..=9 of MCC 0: a clique 1..=8 uses slots 0..=7, and neighbor 9, adjacent to
    // 2..=8, takes slot 0, so slot 0 is used twice and slot 1 once: MCC 0 takes slot 1.
    let sizes = [1, 10, 10, 10, 10, 10, 10, 10, 10, 10];
    let mut edges: Vec<(usize, usize)> = (1..=9).map(|b| (0, b)).collect();
    for a in 1..=8 {
      for b in a + 1..=8 {
        edges.push((a, b));
      }
    }
    edges.extend((2..=8).map(|b| (9, b)));
    let slots = color_slots(&sizes, &graph(10, &edges));
    assert_eq!(0, slots[9]);
    assert_eq!(1, slots[0]);
  }

  #[test]
  fn color_slots_of_random_graphs_with_few_neighbors_never_share_a_slot() {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(7);
    for _ in 0..200 {
      let n = rng.gen_range(1..40);
      // Each MCC gains edges only while both ends have fewer than seven neighbors.
      let mut g = vec![BTreeSet::new(); n];
      for _ in 0..rng.gen_range(0..4 * n) {
        let (a, b) = (rng.gen_range(0..n), rng.gen_range(0..n));
        if a != b && g[a].len() < 7 && g[b].len() < 7 {
          g[a].insert(b);
          g[b].insert(a);
        }
      }
      let sizes: Vec<usize> = std::iter::repeat_with(|| rng.gen_range(1..20)).take(n).collect();
      let slots = color_slots(&sizes, &g);
      assert!(slots.iter().all(|&s| s < MCC_SLOTS));
      for (a, ns) in g.iter().enumerate() {
        for &b in ns {
          assert_ne!(slots[a], slots[b], "neighbors {a} and {b} share a slot");
        }
      }
      assert_eq!(slots, color_slots(&sizes, &g));
    }
  }
}
