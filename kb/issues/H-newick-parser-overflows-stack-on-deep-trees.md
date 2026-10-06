# Newick reader overflows the stack on deep trees

The Newick reader calls itself once per nesting level of the tree. A caterpillar tree with a nesting depth of 60,000 aborts the command line with a stack overflow while it is read. Two more functions of the core recurse once per tree level. The design options are in [`deep-trees.md`](../proposals/deep-trees.md).

## Reproduction

```sh
n=60000
awk -v n=$n 'BEGIN { s = ""; for (i = 0; i < n - 1; i++) s = s "(L" i ","; s = s "L" n - 1; for (i = 0; i < n - 1; i++) s = s ")"; print s ";" }' > a.nwk
treeknit a.nwk a.nwk --naive -o out
```

Output: "thread 'main' has overflowed its stack", then "fatal runtime error: stack overflow, aborting". Measured on 2026-10-06 with the `prod` build in the project container. The two copies of one file have the same tree label, an error that `fn parse_trees` [packages/treeknit-io/src/analysis.rs#L303-L330](../../packages/treeknit-io/src/analysis.rs#L303-L330) reports only after it has read every tree, so the overflow occurs in the reader. A depth of 20,000 is read without error; 200,000 overflows too.

## Locations

- `fn Parser.subtree()` [packages/treeknit-io/src/newick.rs#L232-L254](../../packages/treeknit-io/src/newick.rs#L232-L254): one call per nesting level
- `fn is_coherent()` [packages/treeknit-core/src/naive.rs#L60-L99](../../packages/treeknit-core/src/naive.rs#L60-L99): one call per level of an identical subtree
- `fn copy_subtree()` [packages/treeknit-core/src/impute.rs#L118-L133](../../packages/treeknit-core/src/impute.rs#L118-L133): one call per level of a grafted subtree

## Impact

- The command line aborts with the stack-overflow message of the Rust runtime, without naming the input, and writes no output
- The web app runs the core in a WebAssembly worker. Rust links WebAssembly targets with a 1 MiB stack ("stack-size=1048576" [[src](https://github.com/rust-lang/rust/blob/1b7609cf1cb04d5dc480375f8addd357e5fa83aa/compiler/rustc_target/src/spec/base/wasm.rs#L12-L14)]), and the project sets no other size, against the 8 MiB main-thread stack of the command line under the usual Linux limit (`ulimit -s` 8192, also in the project container)

> [!IMPORTANT]
> **Investigation required.** Measure the largest depth that the web app reads, and whether a stack overflow in the worker reaches the user as an error message.

## Fix direction

- Read Newick with an explicit stack of open nodes instead of recursion; the writer [packages/treeknit-io/src/newick.rs#L347-L382](../../packages/treeknit-io/src/newick.rs#L347-L382) already works this way
- Make `copy_subtree` iterative
- Remove the recursion of `is_coherent`, either with an explicit stack or with the algorithm of [`naive-mccs-by-subtree-hashing.md`](../proposals/naive-mccs-by-subtree-hashing.md)
- Search the core and io crates for other functions that recurse over tree nodes

## Validation

- A test reads and writes a caterpillar tree of depth 200,000 and gets the same Newick text back
- The command line completes `--naive` on two caterpillar trees of depth 200,000
- The existing Newick tests pass unchanged
