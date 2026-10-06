# Benchmarks lack deep trees

The benchmark generator `examples/large_tree_pair.rs` writes a Kingman coalescent tree and a copy changed by subtree moves [packages/treeknit-io/examples/large_tree_pair.rs#L116-L229](../../packages/treeknit-io/examples/large_tree_pair.rs#L116-L229). The depth of a coalescent tree grows with the logarithm of the number of leaves, while influenza segment trees are ladder-like and much deeper. Costs that grow with depth therefore stay hidden in benchmarks:

- Resolution and ARG construction walk to the root for every leaf of a split; on caterpillar trees of 2000 leaves the ARG step takes 48.9 s ([`M-resolution-and-arg-cubic-on-deep-trees.md`](M-resolution-and-arg-cubic-on-deep-trees.md))
- The Newick reader recurses once per level ([`H-newick-parser-overflows-stack-on-deep-trees.md`](H-newick-parser-overflows-stack-on-deep-trees.md))
- The incremental energy walks the ancestors of the flipped leaf ([`kb/reports/performance.md`](../reports/performance.md))

The design options are in [`deep-trees.md`](../proposals/deep-trees.md).

## Fix direction

- Add an option to `large_tree_pair` that writes deep trees: a caterpillar, and a ladder-like random tree in which most internal nodes have one small child
- Keep the seed handling: the same arguments write the same files
- Record the deep-tree runs next to the coalescent runs in the performance measurements

## Validation

- The deep-tree option writes trees whose depth is a large fraction of the number of leaves
- With the deep trees, the benchmark shows the costs of the two issues above before their fixes and their absence after
