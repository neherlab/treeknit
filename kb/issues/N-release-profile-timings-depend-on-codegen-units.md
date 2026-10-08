# Timings of the release profile depend on how the compiler splits the crate

`just example energy_cost` times the energy of the split graph, but it builds with the `release` profile, which uses 16 codegen units and incremental compilation (`Cargo.toml`, `[profile.release]`). Which functions share a codegen unit, and so which calls the compiler inlines, changes with edits outside the hot code. Timings of two code versions in this profile can therefore differ by a third although the hot code is the same.

Example: a version of `Color::new` in `packages/treeknit-core/src/splitgraph.rs` that built the same vectors in the same order as before made the example 1.33 ± 0.06 times slower (hyperfine, 6 runs each, a pair of 3000-leaf trees from `just example large_tree_pair 3000 <dir> 1 60`, 100 000 steps). The constructor runs once per graph, and a test confirmed that both versions build the same data. The shipped build (`dist` profile: one codegen unit, fat LTO) of the same two versions ran the command line on that pair equally fast (1.01 ± 0.01).

## Locations

- `packages/treeknit-io/examples/energy_cost.rs`, run by the `example` recipe of `justfile` with `cargo run --release`

## Fix direction

- Measure performance changes with the `dist` profile: `just build prod` binaries compared with `hyperfine`, as `kb/issues/N-lint-baseline.md` describes
- Or give the examples that time code a recipe that builds them with the `bench` profile, which inherits `dist`

## Validation

- Two builds of the same source in the timing profile give timings within the noise of the machine
