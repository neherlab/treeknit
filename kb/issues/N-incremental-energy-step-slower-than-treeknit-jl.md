# Incremental energy step is slower than in the changed TreeKnit.jl

One annealing step of the port takes 15 to 17 µs without resolution and 71 to 79 µs with resolution on a pair of 1773 effective leaves. TreeKnit.jl with the draft pull requests [#43](https://github.com/PierreBarrat/TreeKnit.jl/pull/43) to [#47](https://github.com/PierreBarrat/TreeKnit.jl/pull/47) takes 4.5 to 5.7 µs and 11 to 12 µs for the same step, with byte-identical results to TreeKnit.jl 0.5.8 ([`kb/reports/performance.md`](../reports/performance.md)). On the four segment trees of 1997 leaves, that TreeKnit.jl runs in 18 to 19 s against 29 to 38 s for the port with one thread. The design options are in [`incremental-energy-step.md`](../proposals/incremental-energy-step.md).

## Locations

- `fn EnergyState.flip()` [packages/treeknit-core/src/splitgraph.rs#L326-L344](../../packages/treeknit-core/src/splitgraph.rs#L326-L344): allocates the candidate and undo vectors and sorts the candidates
- `fn EnergyState.leaf_term()` [packages/treeknit-core/src/splitgraph.rs#L267-L283](../../packages/treeknit-core/src/splitgraph.rs#L267-L283): allocates a vector of ancestors per candidate
- `fn Graph.compatible()` [packages/treeknit-core/src/splitgraph.rs#L62-L73](../../packages/treeknit-core/src/splitgraph.rs#L62-L73): compares bit sets without first comparing the numbers of kept leaves
- `fn eq_on`, `fn subset_on`, `fn disjoint_on` [packages/treeknit-core/src/bits.rs#L22-L40](../../packages/treeknit-core/src/bits.rs#L22-L40): read every word of both clades

## Fix direction

- Reuse buffers held in `EnergyState` and remove duplicate candidates with a per-leaf stamp
- Compare the numbers of kept leaves before the bit sets
- Measure the depth-first numbering of effective leaves with a permutation that keeps the leaf selected by each random draw

## Validation

- `incremental_energy_matches_full` and the fixture comparison with TreeKnit.jl pass unchanged
- `MCCs.json` of `data/h3n2-2k-4-segments` is byte-identical before and after for seeds 1 to 5
- `just example energy_cost` reports the step times before and after
