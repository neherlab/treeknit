# Deep trees: recursion limits and benchmarks

Influenza segment trees are deep and ladder-like: most internal nodes have one small and one large child, so the depth of the tree grows almost in proportion to the number of leaves. Two parts of the port assume shallow trees. Some functions of the core recurse once per tree level and overflow the stack on deep input. The benchmark generator writes coalescent trees, whose depth grows with the logarithm of the number of leaves, so costs that grow with depth do not show in benchmarks. This proposal collects both.

## Problem

### Recursion on tree depth

- **Naive MCCs**: `fn is_coherent()` [packages/treeknit-core/src/naive.rs#L60-L98](../../packages/treeknit-core/src/naive.rs#L60-L98) recurses once per level of an identical subtree ([`naive-mccs-by-subtree-hashing.md`](naive-mccs-by-subtree-hashing.md) removes it)
- **Imputation**: `fn copy_subtree()` [packages/treeknit-core/src/impute.rs#L118-L133](../../packages/treeknit-core/src/impute.rs#L118-L133) recurses once per level of a grafted subtree

The Newick reader and writer of `util-newick` keep their own stack of open nodes ([packages/util-newick/README.md](../../packages/util-newick/README.md)); a test reads and writes a caterpillar tree (every internal node has one leaf child) of depth 200,000. Measured with the `release` build of the command line on 2026-10-07: two identical caterpillar trees of depth 60,000 or 200,000 read, then abort `--naive` with "thread 'main' has overflowed its stack" ([`H-core-recursion-overflows-stack-on-deep-trees.md`](../issues/H-core-recursion-overflows-stack-on-deep-trees.md)). The main thread of the command line has an 8 MiB stack under the usual Linux limit (`ulimit -s` 8192). The web app runs the core in a WebAssembly worker, which Rust links with a 1 MiB stack by default [[src](https://github.com/rust-lang/rust/blob/1b7609cf1cb04d5dc480375f8addd357e5fa83aa/compiler/rustc_target/src/spec/base/wasm.rs#L12-L14)]; the depth it runs was not measured.

### Benchmarks without deep trees

`examples/large_tree_pair.rs` writes a Kingman coalescent tree and a copy changed by subtree moves ([packages/treeknit-io/examples/large_tree_pair.rs#L116-L229](../../packages/treeknit-io/examples/large_tree_pair.rs#L116-L229)). The measurements of [`M-matched-topologies-slow-on-large-trees.md`](../issues/M-matched-topologies-slow-on-large-trees.md) use it. Costs that grow with depth, such as the LCA walks of [`M-resolution-and-arg-cubic-on-deep-trees.md`](../issues/M-resolution-and-arg-cubic-on-deep-trees.md) and the incremental energy, which walks the ancestors of the flipped leaf ([`kb/reports/performance.md`](../reports/performance.md)), stay small on these trees.

## Design axes

### Recursion

- **Iterative rewrites with an explicit stack (recommended)**: removes the limit on every target, including WebAssembly. The Newick reader of `util-newick` is an example in this code base
- **A larger stack**: the command line can run the analysis in a thread with a larger stack (`std::thread::Builder::stack_size`). It does not help the web worker, and it moves the limit without removing it
- **A depth limit with an error**: a clear message instead of an abort, but it rejects valid trees

### Benchmark trees

- **A ladder-like generator in `large_tree_pair` (recommended)**: an option that writes caterpillar trees or ladder-like trees, for example by attaching each new leaf near the root path of the previous one with high probability. Keeps one tool and its seed handling
- **Real data only**: `data/h3n2-2k-4-segments` has 1997 leaves; larger real sets would have to be added to `data/`
- **Fixed caterpillar files written by a test helper**: simple, but tests no realistic branching

## Work items

- [`H-core-recursion-overflows-stack-on-deep-trees.md`](../issues/H-core-recursion-overflows-stack-on-deep-trees.md): the recursion sites of the core
- [`N-benchmarks-lack-deep-trees.md`](../issues/N-benchmarks-lack-deep-trees.md): deep trees in the benchmark generator
- [`H-naive-mccs-overlap-with-unary-nodes.md`](../issues/H-naive-mccs-overlap-with-unary-nodes.md): its proposed fix also removes the recursion of `is_coherent`
