# CLI end-to-end test passes without running when the Julia examples are absent

`two_trees_with_arg` in `packages/treeknit-cli/tests/cli.rs` reads the H3N2 example trees from `../../../legacy_julia_version/examples`, relative to the crate, and returns early when that directory does not exist. The test then reports a pass although it checked nothing. A checkout of this repository alone, the build container, and CI have no such directory, so the test of the two-tree output (MCC files, auspice JSON, ARG files) never runs there.

## Fix direction

- Copy the two example trees into `fixtures/` (the fixture JSON files already embed trees of the NY H3N2 data) and read them from there, so the test runs in every checkout
- Alternatively, fail the test, or mark it `#[ignore]` with the reason, when the inputs are missing, so a skip is visible

## Validation

- `just test-rs two_trees_with_arg` runs the assertions in the build container, verified by temporarily breaking one of them
