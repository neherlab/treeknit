# Tree queries with per-tree indices

Resolution and ARG construction answer two questions many times: "which node is the lowest common ancestor (LCA) of these leaves?" and "is this leaf set a clade of the tree?". The port answers both by walking the tree each time. On deep trees with large MCCs this makes these steps cubic in the number of leaves, while MCC inference itself stays fast. This proposal replaces the walks with indices that are built once per tree version.

## Problem

### The LCA walk

`pub fn Tree.lca()` [packages/treeknit-core/src/tree.rs#L235-L251](../../packages/treeknit-core/src/tree.rs#L235-L251) computes the depth of both nodes by walking to the root, then climbs both nodes until they meet. `pub fn Tree.lca_of()` [packages/treeknit-core/src/tree.rs#L253-L255](../../packages/treeknit-core/src/tree.rs#L253-L255) folds `lca` over all nodes of a set. The cost of one `lca_of` call is

$$O(k \, d)$$

where:

- $k$ -- number of nodes in the set, usually the leaves of a split or of an MCC
- $d$ -- depth of the tree

The walk comes from TreeTools.jl [[src](https://github.com/PierreBarrat/TreeTools.jl/blob/2fb33ac3a5891a9cf0d25c76d2bc0270e571a6b8/src/methods.jl#L355-L405)], whose docstring links a StackOverflow answer for it. TreeTools.jl skips a node that the current ancestor already covers; the port does not.

### Callers that run once per split or per MCC

- `fn blca()` [packages/treeknit-core/src/resolve.rs#L24-L42](../../packages/treeknit-core/src/resolve.rs#L24-L42): `lca_of` over the leaves of a split, then a climb from every leaf to a child of the LCA
- `pub fn resolve_trees()` [packages/treeknit-core/src/resolve.rs#L104](../../packages/treeknit-core/src/resolve.rs#L104): one `lca_of` per split of every tree, in up to 20 passes
- `fn mcc_splits()` [packages/treeknit-core/src/resolve.rs#L161-L201](../../packages/treeknit-core/src/resolve.rs#L161-L201): one `blca` per internal node of every MCC; ARG construction runs it through `resolve_with_mccs` in liberal mode
- `fn propagate_splits()` [packages/treeknit-core/src/pipeline.rs#L332-L362](../../packages/treeknit-core/src/pipeline.rs#L332-L362) with `pub fn insert_split()` [packages/treeknit-core/src/resolve.rs#L47-L65](../../packages/treeknit-core/src/resolve.rs#L47-L65), which also recomputes `clades` and `leaf_of` of the whole tree on every call ([`M-matched-topologies-slow-on-large-trees.md`](../issues/M-matched-topologies-slow-on-large-trees.md))
- `fn shared_nodes()` and `fn fix_shared_singletons()` [packages/treeknit-core/src/arg.rs#L173-L331](../../packages/treeknit-core/src/arg.rs#L173-L331), `fn reduce_to_mccs()` and `fn prune_mccs()` (`packages/treeknit-core/src/pair.rs`), `fn closest_mcc()` and `fn graft_attachments()` (`packages/treeknit-core/src/impute.rs`)

The clade test has the same pattern: `insert_split` recomputes one bit set per node to test a single split, and `mcc_splits` compares each candidate split with every split of the tree (`ref_splits.iter().any(...)`).

### Measurements

Two identical caterpillar trees (every internal node has one leaf child), the `prod` build in the project container, wall-clock time of the whole command on a shared machine (2026-10-06). The reproduction is in [`M-resolution-and-arg-cubic-on-deep-trees.md`](../issues/M-resolution-and-arg-cubic-on-deep-trees.md).

| Leaves | `--resolve none` | `--resolve matched` (default) |
| -----: | ---------------: | ----------------------------: |
|  1,000 |            5.1 s |                        23.2 s |
|  2,000 |           51.9 s |                       237.9 s |
|  5,000 |     not measured |       no progress after 600 s |

- **Where the time goes** (log timestamps, 2000 leaves, `none`): MCC inference finishes 5 ms after it starts; "building ARG from trees and MCCs" takes 48.9 s
- **Growth**: doubling the leaves multiplies the time by about 10, close to the factor 8 of cubic growth
- **Real data**: on the `ha` and `na` trees of `data/h3n2-2k-4-segments` (1997 leaves, 301 MCCs), inference takes 5.7 s, resolution and matching 1.5 s, and the ARG 0.14 s. The cost appears when large MCCs meet deep trees, as in large data sets with few reassortments

## Background

- **Constant-time LCA**: for a tree of $n$ nodes, after $O(n)$ preprocessing, an LCA query takes $O(1)$ on a RAM <a id="cite-1"></a>[Harel and Tarjan 1984](https://doi.org/10.1137/0213024) [[1](#ref-1)]. A simple version reduces LCA to <a id="gloss-use-1"></a>range-minimum queries <sup>[1](#gloss-1)</sup> on the <a id="gloss-use-2"></a>Euler tour <sup>[2](#gloss-2)</sup> of the tree <a id="cite-2"></a>[Bender and Farach-Colton 2000](https://doi.org/10.1007/10719839_9) [[2](#ref-2)]
- **LCA of a leaf set**: number the leaves in depth-first order. The leaves below any node have consecutive numbers, so the LCA of the first and the last leaf of a set already has every leaf of the set below it, and it is the LCA of the set. A set of $k$ leaves needs a min and a max over $k$ numbers and one LCA query
- **Clade test**: Day's <a id="gloss-use-3"></a>cluster table <sup>[3](#gloss-3)</sup> answers "is this set a clade of the tree" in constant time after linear preprocessing <a id="cite-3"></a>[Day 1985](https://doi.org/10.1007/bf01908061) [[3](#ref-3)], which also gives the Robinson-Foulds distance in linear time <a id="cite-4"></a>[Pattengale et al. 2007](https://doi.org/10.1089/cmb.2007.r012) [[4](#ref-4)]. With the depth-first numbering above, a set is a clade exactly when its leaves form the interval of one node
- **Whole-algorithm alternatives**: for one leaf set, pre-resolution adds to each tree the splits of the other trees that are compatible with all of them, that is the <a id="gloss-use-4"></a>loose consensus <sup>[4](#gloss-4)</sup>, which takes optimal $O(nm)$ for $m$ trees of $n$ leaves <a id="cite-5"></a>[Jansson et al. 2016](https://doi.org/10.1145/2925985) [[5](#ref-5)]. The common refinement of compatible trees takes $O(m n)$ <a id="cite-6"></a>[Schaller et al. 2021](https://doi.org/10.1186/s13015-021-00202-8) [[6](#ref-6)]. Neither handles the leaf masks that the port uses for trees with different leaves

## Constraints

- **Identical output**: splits must be inserted in the same order, because the order decides the labels `RESOLVED_<i>` and the written Newick text. Seeded results and the fixture comparison with TreeKnit.jl must not change
- **Trees change during resolution**: `insert_split` adds nodes, which shifts depth-first numbers. An index is valid for one tree version
- **WebAssembly**: the core runs in the browser with one thread and a small stack

## Design axes

### LCA structure

- **Depth-first numbers and depth arrays, LCA of the first and last leaf (recommended)**: one walk of $O(d)$ per set instead of $k$ walks, with no new structure beyond two arrays per tree. For example, `blca` over 2000 leaves walks once instead of 2000 times
- **Euler tour with a sparse table**: $O(1)$ per query and $O(n \log n)$ memory, about 40 lines without a dependency. Worth it if per-set walks stay visible in profiles after the first option
- **`vers-vecs` `FastRmq`**: `vers-vecs` 1.10.2 (2026-08-10, MIT OR Apache-2.0) provides range-minimum queries with $O(n)$ memory; a new dependency for a structure of about 40 lines

### Index lifetime

- **Rebuild after each batch of inserted splits (recommended)**: `insert_compatible`, `propagate_splits`, and `resolve_trees` insert splits in batches. Testing a whole batch against one index and rebuilding afterwards keeps the cost linear per batch. It needs care where a later split of the batch depends on an earlier one
- **Update the index on every insertion**: an inserted node's clade is the union of its children's clades, so a bit-set index is easy to update. Depth-first numbers are not, because they shift
- **Index on an immutable copy**: compute the splits on a copy and insert them into the original afterwards. `resolve_trees` already works this way for its compatibility tests (it never updates `clades` inside the loop)

### Clade storage

- **Keep one bit set per node, add the interval index for membership (recommended)**: the bit sets serve the masked comparisons of the energy; the interval index answers membership without allocation
- **Replace the bit sets of `Tree::clades` by intervals where only membership is needed**: `Tree::clades` [packages/treeknit-core/src/tree.rs#L210-L224](../../packages/treeknit-core/src/tree.rs#L210-L224) stores $n$ bits per node, about 25 MB per tree of 10,000 leaves

## Rejected options

- **A tree crate**: `phylo` 6.0.0 (2026-08-08, MIT) has an Euler-tour LCA oracle, but brings its own arena, heavy dependencies, and six major versions in about three months; `phylotree` 0.1.3 is GPL-3.0; `indextree`, `ego-tree`, and `petgraph` have no LCA. None keeps the taxon ids and the arena semantics that `Tree::compacted` relies on

## Work items

- [`M-resolution-and-arg-cubic-on-deep-trees.md`](../issues/M-resolution-and-arg-cubic-on-deep-trees.md): LCA and clade queries in resolution and ARG construction
- [`M-matched-topologies-slow-on-large-trees.md`](../issues/M-matched-topologies-slow-on-large-trees.md): whole-tree recomputation in `insert_split` during topology matching
- [`N-benchmarks-lack-deep-trees.md`](../issues/N-benchmarks-lack-deep-trees.md): a benchmark that shows these costs

## Glossary

1. <a id="gloss-1"></a> **Range-minimum query.** The position of the smallest value in a given range of an array, answered in constant time after preprocessing the array. [↩](#gloss-use-1)
2. <a id="gloss-2"></a> **Euler tour.** The sequence of nodes visited by a depth-first walk that records a node each time the walk enters or returns to it; the LCA of two nodes is the shallowest node between their first visits. [↩](#gloss-use-2)
3. <a id="gloss-3"></a> **Cluster table.** Day's representation of the clades of a tree as intervals over a depth-first numbering of the leaves, with constant-time membership tests. [↩](#gloss-use-3)
4. <a id="gloss-4"></a> **Loose consensus.** The tree whose clades are the clades of any input tree that are compatible with every input tree. [↩](#gloss-use-4)

## References

1. <a id="ref-1"></a> Harel, Dov, and Robert Endre Tarjan. 1984. "Fast algorithms for finding nearest common ancestors." _SIAM Journal on Computing_ 13:338-355. https://doi.org/10.1137/0213024 [↩](#cite-1)
2. <a id="ref-2"></a> Bender, Michael A., and Martín Farach-Colton. 2000. "The LCA problem revisited." In _LATIN 2000: Theoretical Informatics_, Lecture Notes in Computer Science, 88-94. Springer. https://doi.org/10.1007/10719839_9 [↩](#cite-2)
3. <a id="ref-3"></a> Day, William H. E. 1985. "Optimal algorithms for comparing trees with labeled leaves." _Journal of Classification_ 2:7-28. https://doi.org/10.1007/bf01908061 [↩](#cite-3)
4. <a id="ref-4"></a> Pattengale, Nicholas D., Eric J. Gottlieb, and Bernard M. E. Moret. 2007. "Efficiently computing the Robinson-Foulds metric." _Journal of Computational Biology_ 14:724-735. https://doi.org/10.1089/cmb.2007.r012 [↩](#cite-4)
5. <a id="ref-5"></a> Jansson, Jesper, Chuanqi Shen, and Wing-Kin Sung. 2016. "Improved algorithms for constructing consensus trees." _Journal of the ACM_ 63:1-24. https://doi.org/10.1145/2925985 [↩](#cite-5)
6. <a id="ref-6"></a> Schaller, David, Marc Hellmuth, and Peter F. Stadler. 2021. "A simpler linear-time algorithm for the common refinement of rooted phylogenetic trees on a common leaf set." _Algorithms for Molecular Biology_ 16:23. https://doi.org/10.1186/s13015-021-00202-8 [↩](#cite-6)
