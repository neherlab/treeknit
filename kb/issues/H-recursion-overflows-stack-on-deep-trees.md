# Recursion overflows the stack on deep trees

Three functions recurse once per level of a tree: two in the core and the Auspice JSON writer of io. On deep trees they overflow the stack. The command line then aborts without naming the input and writes no output. The web app fails every run with a tree deeper than 1,000 to 2,000 levels, because each run writes the Auspice JSON file of every tree, and reports the failure as a probable out-of-memory condition. The Newick reader and writer of `util-newick` keep their own stack of open nodes, so the trees read; the overflow comes later. The design options are in [`deep-trees.md`](../proposals/deep-trees.md).

## Locations

- `fn is_coherent()` [packages/treeknit-core/src/naive.rs#L60-L98](../../packages/treeknit-core/src/naive.rs#L60-L98): one call per level of a subtree that is identical in all trees. Naive MCC inference calls it first in every pair, also without `--naive`
- `fn copy_subtree()` [packages/treeknit-core/src/impute.rs#L125-L140](../../packages/treeknit-core/src/impute.rs#L125-L140): one call per level of a subtree of leaves that the other tree lacks, when imputation grafts it. `fn imputed_trees` calls it in every run with missing leaves, and `fn arg_inputs` again for the ARG
- `pub fn auspice_json()` [packages/treeknit-io/src/auspice.rs#L18-L59](../../packages/treeknit-io/src/auspice.rs#L18-L59): builds the nested `serde_json::Value` bottom-up, but the final `json!` passes the tree through `serde_json::to_value`, which copies the whole value recursively. Formatting the value (`format!("{:#}")` in [packages/treeknit-io/src/output.rs#L297](../../packages/treeknit-io/src/output.rs#L297)) and dropping it recurse again. The command line calls it for `--auspice-view`; the web app calls it in every run (`OutputOptions::web`)
- `pub fn auspice_view()` [packages/treeknit-io/src/display/auspice.rs#L52](../../packages/treeknit-io/src/display/auspice.rs#L52): `struct AuspiceNode` nests its children, so its derived `Serialize`, `Clone`, `PartialEq`, and its drop recurse. The Auspice view of the web app converts it to JSON (`fn to_js`)

A search with `ast-grep` for functions that call themselves found no other site in the production code of the core, io, WebAssembly, command-line, and `util-newick` crates (`nwk_node` in the test helpers of `tree.rs` also recurses). The tanglegram data of `pair_view` is flat.

## Measured limits

Caterpillar trees (every internal node has one leaf child), `release` build, 2026-10-08, in the project container. "Identical" pairs two copies of one caterpillar; "different" pairs it with the caterpillar of reversed leaf order, which shares no subtree; "private" is `((A,B),(C,<caterpillar of P leaves>))` against `((A,C),B)`.

| Site                                        | Command line                                          | WebAssembly (Node, `wasm-bindgen-test-runner`) |
| ------------------------------------------- | ----------------------------------------------------- | ---------------------------------------------- |
| `is_coherent`, two trees (main thread)      | runs at 30,000, overflows at 60,000                   | runs at 5,000, fails at 10,000                 |
| `is_coherent`, three trees (`rayon` worker) | runs at 5,000, overflows at 10,000                    | not measured                                   |
| `copy_subtree`                              | runs at 60,000, overflows at 200,000                  | runs at 2,000, fails at 10,000                 |
| `auspice_json`                              | runs at 5,000, overflows at 10,000 (`--auspice-view`) | runs at 1,000, fails at 2,000                  |
| `auspice_view` with JSON conversion         | web app only                                          | runs at 5,000, fails at 10,000                 |

- **Worker threads**: with three or more trees, `pairs.par_iter()` in `packages/treeknit-core/src/pipeline.rs` runs pair inference on `rayon` workers. They get the Rust default stack of 2 MiB, a quarter of the 8 MiB main thread (`ulimit -s` 8192, also in the project container), so the limit is about four times lower. The abort names the thread `'<unknown>'`
- **Control**: the different pair of depth 10,000 completes without `--auspice-view` and overflows with it, during "writing results"
- **Real data**: the deepest tree in `data/` has a nesting depth of 136 (`data/h5n1-587x3-timetree/tree_pb1.nwk`, 587 leaves); `data/h3n2-2k-4-segments/ha.nwk` (1,997 leaves) has 46

## WebAssembly behavior

Rust links WebAssembly with a 1 MiB stack in linear memory, placed before the static data (`--stack-first`) so that an overflow traps instead of corrupting data [[src](https://github.com/rust-lang/rust/blob/1b7609cf1cb04d5dc480375f8addd357e5fa83aa/compiler/rustc_target/src/spec/base/wasm.rs#L10-L31)]. The engine also limits its own call stack. Which limit a site hits first depends on its frame size, and the two give different errors:

- **Linear-memory stack** (`auspice_json`): `RuntimeError: memory access out of bounds`. `withPanicText` in `packages/web/src/analysis/panicText.ts` then shows "The analysis stopped without an error message. It most likely ran out of memory. ...", which names the wrong cause
- **Engine call stack** (`is_coherent`, `copy_subtree`, `auspice_view`): `RangeError: Maximum call stack size exceeded`, which the client reports as an internal failure with that message

The client closes the run worker after any failed run, so a failure does not affect later runs. A `RangeError` in a session export such as `auspiceView` keeps the session worker: the JavaScript exception unwinds the WebAssembly frames without running Rust destructors or restoring the stack pointer. Whether that worker still works afterwards was not tested.

The measurements ran in Node; browser workers may have other engine stack limits.

## Fix direction

- Make `copy_subtree` iterative
- Remove the recursion of `is_coherent`, either with an explicit stack or with the algorithm of [`naive-mccs-by-subtree-hashing.md`](../proposals/naive-mccs-by-subtree-hashing.md)
- Write the Auspice JSON of `auspice_json` and `AuspiceNode` without a recursive serializer or drop, for example by writing the text with an explicit stack, and drop nested nodes iteratively
- Map a trap with an out-of-bounds message to a stack-overflow message in `panicText.ts` only after deciding how to tell it apart from other traps

## Validation

The run of two identical deep caterpillars cannot complete until [`M-resolution-and-arg-cubic-on-deep-trees.md`](M-resolution-and-arg-cubic-on-deep-trees.md) is fixed: identical caterpillars of depth 5,000 do not finish within 300 s even with `--naive`, because resolution, topology matching, and the ARG grow about as the cube of the depth. Three trees with `--resolve none` build no ARG and finish depth 5,000 in 0.04 s.

- The command line completes `--naive --resolve none` on three identical caterpillar trees of depth 200,000
- The command line completes `--naive --resolve none --auspice-view` on the different caterpillar pair of depth 200,000 once the cubic issue no longer blocks the ARG
- A WebAssembly test runs naive MCCs, imputation of a private caterpillar, `auspice_json`, and `auspice_view` at depth 200,000
- The existing tests pass unchanged, and the output files are byte-identical
