# Deep trees: recursion limits and benchmarks

Influenza segment trees are deep and ladder-like: most internal nodes have one small and one large child, so the depth of the tree grows almost in proportion to the number of leaves. Two parts of the port assume shallow trees. Some functions recurse once per tree level, and the Newick reader overflows the stack on deep input. The benchmark generator writes coalescent trees, whose depth grows with the logarithm of the number of leaves, so costs that grow with depth do not show in benchmarks. This proposal collects both.

## Problem

### Recursion on tree depth

- **Newick reader**: `fn Parser.subtree()` [packages/treeknit-io/src/newick.rs#L232-L254](../../packages/treeknit-io/src/newick.rs#L232-L254) calls itself once per nesting level. The writer is iterative "to avoid deep recursion on ladder-like trees" ([packages/treeknit-io/src/newick.rs#L347-L382](../../packages/treeknit-io/src/newick.rs#L347-L382))
- **Naive MCCs**: `fn is_coherent()` [packages/treeknit-core/src/naive.rs#L60-L99](../../packages/treeknit-core/src/naive.rs#L60-L99) recurses once per level of an identical subtree ([`naive-mccs-by-subtree-hashing.md`](naive-mccs-by-subtree-hashing.md) removes it)
- **Imputation**: `fn copy_subtree()` [packages/treeknit-core/src/impute.rs#L118-L133](../../packages/treeknit-core/src/impute.rs#L118-L133) recurses once per level of a grafted subtree

Measured with caterpillar trees (every internal node has one leaf child) and the `prod` build of the command line on 2026-10-06: a nesting depth of 20,000 parses; depths of 60,000 and 200,000 abort with "thread 'main' has overflowed its stack" while the trees are read ([`H-newick-parser-overflows-stack-on-deep-trees.md`](../issues/H-newick-parser-overflows-stack-on-deep-trees.md)). The main thread of the command line has an 8 MiB stack under the usual Linux limit (`ulimit -s` 8192). The web app runs the core in a WebAssembly worker, which Rust links with a 1 MiB stack by default [[src](https://github.com/rust-lang/rust/blob/1b7609cf1cb04d5dc480375f8addd357e5fa83aa/compiler/rustc_target/src/spec/base/wasm.rs#L12-L14)]; the depth it reads was not measured.

### Benchmarks without deep trees

`examples/large_tree_pair.rs` writes a Kingman coalescent tree and a copy changed by subtree moves ([packages/treeknit-io/examples/large_tree_pair.rs#L116-L229](../../packages/treeknit-io/examples/large_tree_pair.rs#L116-L229)). The measurements of [`M-matched-topologies-slow-on-large-trees.md`](../issues/M-matched-topologies-slow-on-large-trees.md) use it. Costs that grow with depth, such as the LCA walks of [`M-resolution-and-arg-cubic-on-deep-trees.md`](../issues/M-resolution-and-arg-cubic-on-deep-trees.md) and the incremental energy, which walks the ancestors of the flipped leaf ([`kb/reports/performance.md`](../reports/performance.md)), stay small on these trees.

## Design axes

### Recursion

- **Iterative rewrites with an explicit stack (recommended)**: removes the limit on every target, including WebAssembly. The reader can keep a stack of open nodes; the parser of the `phylo` crate (6.0.0, 2026-08-08, MIT) is an iterative example of the same grammar
- **A larger stack**: the command line can run the analysis in a thread with a larger stack (`std::thread::Builder::stack_size`). It does not help the web worker, and it moves the limit without removing it
- **A depth limit with an error**: a clear message instead of an abort, but it rejects valid trees

### Benchmark trees

- **A ladder-like generator in `large_tree_pair` (recommended)**: an option that writes caterpillar trees or ladder-like trees, for example by attaching each new leaf near the root path of the previous one with high probability. Keeps one tool and its seed handling
- **Real data only**: `data/h3n2-2k-4-segments` has 1997 leaves; larger real sets would have to be added to `data/`
- **Fixed caterpillar files written by a test helper**: simple, but tests no realistic branching

## Rejected options

- **A Newick crate**: no Rust crate reads the `#H` hybrid nodes of the extended Newick format that the ARG output uses. `phylo` 6.0.0 states that "`#H` hybrid nodes are out of scope" and brings its own tree type; `newick` 0.12.0 is LGPL-3.0-or-later and reads no quoted labels; `bio` 4.2.1 has no writer and no quoted labels; `phylotree` 0.1.3 is GPL-3.0

## Work items

- [`H-newick-parser-overflows-stack-on-deep-trees.md`](../issues/H-newick-parser-overflows-stack-on-deep-trees.md): iterative reader and the other recursion sites
- [`N-benchmarks-lack-deep-trees.md`](../issues/N-benchmarks-lack-deep-trees.md): deep trees in the benchmark generator
- [`H-naive-mccs-overlap-with-unary-nodes.md`](../issues/H-naive-mccs-overlap-with-unary-nodes.md): its proposed fix also removes the recursion of `is_coherent`
