# Core functions overflow the stack on deep trees

Two functions of the core call themselves once per level of the tree. Two identical caterpillar trees with a nesting depth of 60,000 abort the command line with a stack overflow in `--naive` mode. The Newick reader and writer of `util-newick` keep their own stack of open nodes, so the trees read; the overflow comes later. The design options are in [`deep-trees.md`](../proposals/deep-trees.md).

## Reproduction

```sh
n=60000
awk -v n=$n 'BEGIN { s = ""; for (i = 0; i < n - 1; i++) s = s "(L" i ","; s = s "L" n - 1; for (i = 0; i < n - 1; i++) s = s ")"; print s ";" }' > a.nwk
cp a.nwk b.nwk
treeknit a.nwk b.nwk --naive -o out
```

Output: "thread 'main' has overflowed its stack", then "fatal runtime error: stack overflow, aborting". Measured on 2026-10-07 with the `release` build in the project container, for depths of 60,000 and 200,000. With `b.nwk` as the same caterpillar with its leaves in reverse order, which shares no deep subtree with `a.nwk`, the same command reads both trees and finds the 60,000 naive MCCs without an overflow. So the overflow needs an identical deep subtree, which is the recursion of `fn is_coherent`. No debugger in the container confirmed the frame.

## Locations

- `fn is_coherent()` [packages/treeknit-core/src/naive.rs#L60-L98](../../packages/treeknit-core/src/naive.rs#L60-L98): one call per level of an identical subtree
- `fn copy_subtree()` [packages/treeknit-core/src/impute.rs#L118-L133](../../packages/treeknit-core/src/impute.rs#L118-L133): one call per level of a grafted subtree

## Impact

- The command line aborts with the stack-overflow message of the Rust runtime, without naming the input, and writes no output
- The web app runs the core in a WebAssembly worker. Rust links WebAssembly targets with a 1 MiB stack ("stack-size=1048576" [[src](https://github.com/rust-lang/rust/blob/1b7609cf1cb04d5dc480375f8addd357e5fa83aa/compiler/rustc_target/src/spec/base/wasm.rs#L12-L14)]), and the project sets no other size, against the 8 MiB main-thread stack of the command line under the usual Linux limit (`ulimit -s` 8192, also in the project container)

> [!IMPORTANT]
> **Investigation required.** Measure the largest depth that the web app runs, and whether a stack overflow in the worker reaches the user as an error message.

## Fix direction

- Make `copy_subtree` iterative
- Remove the recursion of `is_coherent`, either with an explicit stack or with the algorithm of [`naive-mccs-by-subtree-hashing.md`](../proposals/naive-mccs-by-subtree-hashing.md)
- Search the core and io crates for other functions that recurse over tree nodes

## Validation

- The command line completes `--naive` on two identical caterpillar trees of depth 200,000
- The existing tests pass unchanged
