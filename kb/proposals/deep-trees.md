# Deep trees: recursion limits and benchmarks

Influenza segment trees are deep and ladder-like: most internal nodes have one small and one large child, so the depth of the tree grows almost in proportion to the number of leaves. Two parts of the port assumed shallow trees. Functions that recursed once per tree level overflowed the stack on deep input; the Auspice view of the web app still does. The benchmark generator writes coalescent trees, whose depth grows with the logarithm of the number of leaves, so costs that grow with depth do not show in benchmarks. This proposal collects both.

## Problem

### Recursion on tree depth

The analysis and every output file handle trees of any depth without recursion: the Newick reader and writer of `util-newick` keep their own stack of open nodes ([packages/util-newick/README.md](../../packages/util-newick/README.md)), and naive MCC inference (`fn is_coherent`), imputation (`fn copy_subtree`), and the Auspice JSON files (`pub fn auspice_json`) use explicit stacks. Tests run each of them on a caterpillar tree (every internal node has one leaf child) of depth 5,000 on a 256 KiB thread stack, and in WebAssembly at depth 12,000. Before these rewrites, the command line overflowed at depths between 5,000 and 60,000, depending on the function and the thread, and the web app failed every run with a tree deeper than 1,000 to 2,000 levels.

The Auspice view of the web app still nests one object per tree level and fails at about 1,100 levels ([`M-auspice-view-fails-on-deep-trees.md`](../issues/M-auspice-view-fails-on-deep-trees.md)). The command line and the web worker have fixed stacks: 8 MiB on the main thread of the command line under the usual Linux limit (`ulimit -s` 8192), 2 MiB on the `rayon` workers that infer the pairs of three or more trees, and 1 MiB in WebAssembly [[src](https://github.com/rust-lang/rust/blob/1b7609cf1cb04d5dc480375f8addd357e5fa83aa/compiler/rustc_target/src/spec/base/wasm.rs#L12-L14)], so new code that recurses per level fails on deep trees again.

Memory, not stack, limits the depth now: `Tree::clades` stores one bit per taxon for every node, about 10 GB per caterpillar of 200,000 leaves, and three such trees run out of memory ([`tree-query-indices.md`](tree-query-indices.md)).

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

- [`M-auspice-view-fails-on-deep-trees.md`](../issues/M-auspice-view-fails-on-deep-trees.md): the nesting of the Auspice view of the web app
- [`N-benchmarks-lack-deep-trees.md`](../issues/N-benchmarks-lack-deep-trees.md): deep trees in the benchmark generator
- [`H-naive-mccs-overlap-with-unary-nodes.md`](../issues/H-naive-mccs-overlap-with-unary-nodes.md): the climb of naive MCC inference, which its proposed fix replaces
