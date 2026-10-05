# CLI end-to-end test passes without running when the Julia examples are absent

`two_trees_with_arg` in `packages/treeknit-cli/tests/cli.rs` reads the H3N2 example trees from `../../../legacy_julia_version/examples`, relative to the crate, and returns early when that directory does not exist. The test then reports a pass although it checked nothing. A checkout of this repository alone and the build container have no such directory, so the test of the two-tree output (MCC files, auspice JSON, ARG files) never runs there.

## Fix direction

- Read the two example trees from `data/h3n2-2017/`, which holds byte-identical copies of the TreeKnit.jl example files, so the test runs in every checkout
- Alternatively, fail the test, or mark it `#[ignore]` with the reason, when the inputs are missing, so a skip is visible

## Validation

- `just test-rs two_trees_with_arg` runs the assertions in the build container, verified by temporarily breaking one of them
