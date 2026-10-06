# Faster incremental energy step

The annealing flips one effective leaf per step and updates the energy only for the leaves the flip can affect. The port introduced this incremental update; draft pull requests to TreeKnit.jl have since applied the same scheme with further changes, and one step there now takes 4.5 to 5.7 µs against 15 to 17 µs in the port on the same tree pair ([`kb/reports/performance.md`](../reports/performance.md), section "Changes proposed to TreeKnit.jl"). This proposal brings those changes back to the port.

## Problem

`pub struct EnergyState` with `fn flip()` and `fn undo()` [packages/treeknit-core/src/splitgraph.rs#L209-L357](../../packages/treeknit-core/src/splitgraph.rs#L209-L357) keeps the number of kept leaves below every node and the term of every leaf. On one pair of 1773 effective leaves:

| One annealing step                 | Without resolution | With resolution |
| ---------------------------------- | ------------------ | --------------- |
| TreeKnit.jl with the draft changes | 4.5 to 5.7 µs      | 11 to 12 µs     |
| Port                               | 15 to 17 µs        | 71 to 79 µs     |

The performance report names the differences, which the code confirms:

- **Allocations in every flip**: `fn flip()` builds a candidate vector and an undo vector, and `fn leaf_term()` [packages/treeknit-core/src/splitgraph.rs#L267-L283](../../packages/treeknit-core/src/splitgraph.rs#L267-L283) builds a vector of ancestors for every candidate
- **Duplicate removal by sorting**: `flip` sorts the candidates and removes duplicates; the draft changes mark visited leaves with a per-leaf stamp instead
- **No count comparison first**: `fn Graph.compatible()` [packages/treeknit-core/src/splitgraph.rs#L62-L73](../../packages/treeknit-core/src/splitgraph.rs#L62-L73) compares the clades word by word even when the numbers of kept leaves below the two nodes, which the state already holds, differ and decide the answer
- **All words of every clade**: the masked comparisons of [packages/treeknit-core/src/bits.rs#L22-L48](../../packages/treeknit-core/src/bits.rs#L22-L48) read every word of the bit sets. Effective leaves are numbered in the order of the sorted MCC list, so even a small clade spreads over all words. Numbering them in the depth-first order of the first tree puts each clade of that tree in a short range of words ([TreeKnit.jl #47](https://github.com/PierreBarrat/TreeKnit.jl/pull/47))

## Constraints

- **Identical results**: the random draws pick an effective leaf by its index (`gen_range(0..n)` in `fn Chain.mcmc()`, [packages/treeknit-core/src/anneal.rs#L95-L116](../../packages/treeknit-core/src/anneal.rs#L95-L116)). A new bit order must not change which leaf a draw selects, so it needs a permutation between the drawn index and the bit position. The energy of every step must stay the same integer, so the chain makes the same moves
- **The full computation stays the reference**: the test that compares the incremental energy with the full computation along random flips (`incremental_energy_matches_full` in `splitgraph.rs`) must keep passing

## Design axes

### Buffers

- **Reusable buffers in the state (recommended)**: candidate, undo, and ancestor buffers that `flip` clears instead of allocating; a stamp array for duplicate removal. No change to results
- **Small-vector types**: avoid heap allocation for few candidates, but add a dependency and keep the sort

### Comparison shortcuts

- **Compare kept counts first (recommended)**: equal clades on the kept leaves need equal counts, so different counts answer `false` without reading words. For the comparison with resolution, the subset tests can use the counts in the same way
- **Word ranges per node**: store the first and last word that holds a leaf of each clade and compare only the overlap of the two ranges. Needs the depth-first numbering to help

### Bit order

- **Depth-first order of the first tree with a permutation for the draws (recommended after measurement)**: the clades of the first tree become intervals; the clades of the second tree benefit as far as the trees agree
- **Keep the MCC order**: no permutation, no locality

## Work items

- [`N-incremental-energy-step-slower-than-treeknit-jl.md`](../issues/N-incremental-energy-step-slower-than-treeknit-jl.md)
