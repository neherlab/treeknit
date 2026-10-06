# Libraries for the hand-written algorithms

The port writes its core algorithms by hand. This report records, for each of them, which libraries were evaluated as replacements, what they offer, and why the hand-written code stays, so that later work does not repeat the search. Improvements by better algorithms, as opposed to libraries, are in `kb/proposals/` ([`tree-query-indices.md`](../proposals/tree-query-indices.md), [`naive-mccs-by-subtree-hashing.md`](../proposals/naive-mccs-by-subtree-hashing.md), [`deep-trees.md`](../proposals/deep-trees.md), [`incremental-energy-step.md`](../proposals/incremental-energy-step.md)); the names and sources of the algorithms are in [`algorithm-names.md`](../proposals/algorithm-names.md).

Evaluated on 2026-10-06. Versions are the newest releases on crates.io or npm on that day, read from the registry APIs; library behavior was read from the library sources.

## Summary

- **Keep the hand-written code** for the annealing, the incremental energy, the masked bit-set operations, the tree arena, the Newick reader and writer, the node-to-MCC map, the tie-break formula, the polytomy sort, the seed mixing, the color slots, the layout, and the accuracy measure
- **Why**: no library meets the constraints below. The candidates change the random-number stream or the acceptance rule, lack a needed operation, have no extended-Newick support, or have a GPL or LGPL license
- **Where improvements are**: in algorithms (constant-time LCA, canonical subtree identifiers, iterative parsing, a faster incremental step), all of which fit in the existing code without a new library

## Constraints

- **Seeded results**: a run with a given seed must give the same MCCs, and the fixture tests compare the annealing with TreeKnit.jl. A change of the acceptance rule, of the order of random draws, or of the value stream of `rand` changes every seeded result
- **License**: the project is MIT-licensed, so GPL-3.0 and LGPL-3.0 libraries are excluded from linked code
- **WebAssembly**: the core runs in the browser through `packages/treeknit-wasm`
- **Extended Newick**: the ARG output uses `#H` hybrid nodes

## Annealing

- **Code**: `fn Chain.mcmc()` and `pub fn optimize()` in [`packages/treeknit-core/src/anneal.rs`](../../packages/treeknit-core/src/anneal.rs): one leaf flip per step with an incremental energy and undo, the Metropolis rule, a second random draw only for uphill moves, and a list of every configuration of minimal free energy for the tie-break
- **`argmin` 0.11.0** (2025-09-28, MIT OR Apache-2.0): its simulated annealing asks the problem for a new parameter (`anneal`), evaluates the full `cost` of it, accepts an uphill move with probability `1 / (1 + exp(Δ / T))`, changes the temperature after every iteration, and keeps one best state [[src](https://docs.rs/argmin/0.11.0/src/argmin/solver/simulatedannealing/mod.rs.html#508-543)]. Different acceptance rule, no undo, no tie list
- **`metaheuristics` 1.1.22** (2022-07-16): GPL-3.0
- **Decision**: keep

## Clade bit sets

- **Code**: `fn eq_on`, `fn subset_on`, `fn disjoint_on`, `fn trivial_on` [packages/treeknit-core/src/bits.rs#L22-L48](../../packages/treeknit-core/src/bits.rs#L22-L48) compare two clades restricted to the kept leaves, word by word, without allocating
- **`fixedbitset` 0.5.7** (in use): `is_subset`, `is_disjoint`, `intersection_count`, and the other set operations take two operands [[doc](https://docs.rs/fixedbitset/0.5.7/fixedbitset/struct.FixedBitSet.html)]; a comparison restricted to a mask needs the word loop or an allocated intersection
- **`bitvec` 1.1.1** (2026-06-18), **`roaring` 0.11.5** (2026-08-12), **`bit-set` 0.11.1** (2026-07-10), **`hi_sparse_bitset` 0.10.0** (2026-10-01): no three-operand predicates
- **Decision**: keep; faster comparisons are in [`incremental-energy-step.md`](../proposals/incremental-energy-step.md)

## Tree arena and LCA

- **Code**: `pub struct Tree` in [`packages/treeknit-core/src/tree.rs`](../../packages/treeknit-core/src/tree.rs), an arena in which detached nodes stay until `Tree::compacted`, with taxon ids on leaves, restriction to a leaf set, removal of unary nodes with summed branch lengths, ladderizing, and an LCA that walks to the root
- **`phylo` 6.0.0** (2026-08-08, MIT): an Euler-tour LCA, restriction, removal of unary nodes, and Robinson-Foulds distance, with its own arena and node type, no ladderizing, heavy dependencies, and six major versions in about three months
- **`phylotree` 0.1.3** (2024-12-11): GPL-3.0
- **`indextree` 4.9.1** (2026-08-27), **`ego-tree` 0.11.0** (2026-01-23), **`petgraph` 0.8.3** (2025-09-30): generic trees or graphs without LCA, restriction, or branch-length handling
- **Range-minimum queries for an Euler-tour LCA**: `vers-vecs` 1.10.2 (2026-08-10, MIT OR Apache-2.0)
- **Decision**: keep the arena; replace the LCA algorithm ([`tree-query-indices.md`](../proposals/tree-query-indices.md))

## Newick reading and writing

- **Code**: [`packages/treeknit-io/src/newick.rs`](../../packages/treeknit-io/src/newick.rs): quoted labels with `''` escapes, comments skipped, invalid lengths read as missing with a warning, the first of several trees, byte offsets in errors, a writer that quotes labels
- **`phylo` 6.0.0**: an iterative parser with the same quoting rules and a writer; its documentation states that "`#H` hybrid nodes are out of scope"
- **`newick` 0.12.0** (2026-08-04): LGPL-3.0-or-later, no quoted labels
- **`bio` 4.2.1** (2026-10-05, MIT): `bio::io::newick` reads without quoted labels or comments and has no writer
- **Decision**: keep; the reader must become iterative ([`deep-trees.md`](../proposals/deep-trees.md))

## Random numbers

- **Code**: `fn mix()` [packages/treeknit-core/src/pipeline.rs#L488-L494](../../packages/treeknit-core/src/pipeline.rs#L488-L494) derives the seed of each pair and round with the SplitMix64 finalizer; the generator is `rand_xoshiro::Xoshiro256PlusPlus`
- **`rand_xoshiro` 0.6.0** (in use): exports `SplitMix64` with the same constants [[doc](https://docs.rs/rand_xoshiro/0.6.0/rand_xoshiro/struct.SplitMix64.html)], and `Xoshiro256PlusPlus::seed_from_u64` already passes the seed through SplitMix64 [[src](https://docs.rs/rand_xoshiro/0.6.0/src/rand_xoshiro/xoshiro256plusplus.rs.html#78-80)]. Rewriting `mix` with `SplitMix64` keeps the values but reads worse; `jump()` per pair would make the streams depend on the order of pairs
- **`rand_seeder` 0.5.0** (2026-02-02): hashes arbitrary data into a seed with SipHash, so every seeded result would change
- **`rand` 0.9 and later**: the 0.9.0 changelog lists "Optimize distribution `Uniform`: use Canon's method (single sampling) / Lemire's method (distribution sampling) for faster sampling (breaks value stability" [[changelog](https://github.com/rust-random/rand/blob/9e7d328f60d6372ae22927f37d1a816fc980f25e/CHANGELOG.md?plain=1#L211)], so an upgrade from 0.8 changes the leaf that each draw selects
- **Decision**: keep `mix`; name SplitMix64 in its doc comment ([`N-algorithm-names-undocumented.md`](../issues/N-algorithm-names-undocumented.md)); upgrade `rand` only together with new reference results

## Partitions, node-to-MCC map, and tie-break

- **Meet of MCC partitions** (needed only by the check in [`cross-pair-mcc-consistency.md`](../proposals/cross-pair-mcc-consistency.md)): union-find crates such as `petgraph::unionfind::UnionFind` compute the join of partitions, which merges blocks; the meet groups leaves by their tuple of block indices, a hash-map grouping without a library
- **Node-to-MCC map** (`pub fn map_mccs()`, [packages/treeknit-core/src/mcc_map.rs#L29-L65](../../packages/treeknit-core/src/mcc_map.rs#L29-L65)): `cyanea-phylo` 0.1.1 (2026-07-30, Apache-2.0) has Fitch and Sankoff parsimony with states limited to `u8` and an error for leaves without a state, so it cannot express leaves without an MCC as wildcards. Keep
- **Tie-break** (`pub fn branch_likelihood()`, [packages/treeknit-core/src/splitgraph.rs#L359-L366](../../packages/treeknit-core/src/splitgraph.rs#L359-L366)): the term is a Poisson log-likelihood ratio in which the factorials cancel, so non-integer mutation counts are allowed; `statrs` 0.19.1 (2026-08-11) `Poisson::ln_pmf` takes integer counts. Keep

## Polytomy sort

- **Code**: `fn insertion_sort_by()` [packages/treeknit-core/src/mcc_map.rs#L84-L93](../../packages/treeknit-core/src/mcc_map.rs#L84-L93) sorts the children of a polytomy with a comparison that is not a total order
- **Standard library**: since Rust 1.81.0, "The new sort implementations may panic if a type's implementation of `Ord` (or the given comparison function) does not implement a total order" [[changelog](https://github.com/rust-lang/rust/blob/f614da46fc69df644d8e6ae0a8a0c68dc790f174/RELEASES.md?plain=1#L2576)]
- **Decision**: keep; the remaining difference from Julia's sort is [`N-polytomy-sort-differs-from-julia-for-large-polytomies.md`](../issues/N-polytomy-sort-differs-from-julia-for-large-polytomies.md)

## Display and figures

- **MCC color slots** (`fn color_slots()`, [packages/treeknit-io/src/display/slots.rs#L47-L69](../../packages/treeknit-io/src/display/slots.rs#L47-L69)): sequential greedy coloring with a fixed number of slots, largest MCC first, least-used free slot. `petgraph::algo::coloring::dsatur_coloring` uses as many colors as needed, without size order or balance. Keep
- **Tree layout** (`fn place()`, [packages/treeknit-io/src/display/tree.rs#L58-L88](../../packages/treeknit-io/src/display/tree.rs#L58-L88)): an internal node sits at the midpoint of its first and last child. IcyTree uses the mean of the non-hybrid children [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treelayouts.js#L281-L300)], as does `d3-hierarchy` 3.1.2 (2022-04-02) `cluster()`, which has no branch-length mode. About ten lines; keep
- **ARG node order** (`fn topological_order()`, [packages/treeknit-io/src/display/arg_view.rs#L141-L160](../../packages/treeknit-io/src/display/arg_view.rs#L141-L160)): Kahn's algorithm; `petgraph::algo::toposort` would need a conversion to a petgraph graph. Keep, with the cycle check of [`N-arg-acyclicity-unchecked.md`](../issues/N-arg-acyclicity-unchecked.md)
- **Legend line breaking** (`fn legend_places()` in `packages/treeknit-io/src/figure/svg.rs`): greedy first-fit; `textwrap` 0.16.4 (2026-09-13) `wrap_algorithms::wrap_first_fit` covers it, for about 15 lines. Optional
- **Label widths in SVG figures** (`fn char_advance()` in `packages/treeknit-io/src/figure/svg.rs`): Helvetica advance widths, documented in the code as an estimate, because the font that a viewer uses for a standalone SVG file is unknown. Exact metrics of one font would not fix this. Keep
- **Palette** (`packages/treeknit-io/src/palette.rs`): a fixed list; its tests compute WCAG contrast by hand, and the CIEDE2000 distances measured when the colors were chosen are not repeated by any test. `palette` 0.7.7 (2026-08-02) provides CIEDE2000 and would let a test repeat that check. Optional
- **Tanglegram order in Auspice**, for comparison: Auspice flips the children of each node in postorder and keeps a flip when the Pearson correlation of the tip positions of the two trees does not drop [[src](https://github.com/nextstrain/auspice/blob/37bf9ce1e3b9a8cbdf1ebfd77bd8d7bdb6dfdc2d/src/components/tree/tangle/untangling.js#L39-L74)]. The port orders the second tree by MCC ranks instead, so that MCCs face each other

## Evaluation and web app

- **Accuracy** (`fn scaled_vi()`, [packages/treeknit-io/examples/accuracy.rs#L17-L47](../../packages/treeknit-io/examples/accuracy.rs#L17-L47)): variation of information divided by ln n, the measure of the TreeKnit paper; only large machine-learning crates provide it. Keep
- **Progress throttle** (`class ProgressThrottle` in `packages/web/src/analysis/progressThrottle.ts`): forwards at most one progress event per interval, but at once when the phase, round, or pair changes. `funnel` of `remeda` 2.50.0 (2026-09-14, already a dependency) throttles by time and has no condition for forwarding at once. Keep
