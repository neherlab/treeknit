# Lint baseline: panics, hash collections, and numeric casts in library modules

The workspace enables the Clippy `restriction` group, plus `unwrap_used`, `expect_used`, `as_conversions`, `iter_over_hash_type`, and the `HashMap`/`HashSet` ban of `clippy.toml` (`disallowed_types`). The library code predates these lints. Each affected module carries a module-level `#![expect(...)]` with the lints it still violates, so `just lint-rs` passes while new modules get the full lint set.

`#![expect]` turns into a warning once its lint no longer fires in the module, so the baseline shrinks as the code is fixed: remove a lint from the attribute when the module is clean, and the attribute when the list is empty.

## Locations

| Module                                 | Lints in the baseline                                                 |
| -------------------------------------- | --------------------------------------------------------------------- |
| `packages/treeknit-core/src/anneal.rs`     | `as_conversions`                                                      |
| `packages/treeknit-core/src/arg.rs`        | `disallowed_types`, `iter_over_hash_type`, `unwrap_used`              |
| `packages/treeknit-core/src/impute.rs`     | `expect_used`, `unwrap_used`                                          |
| `packages/treeknit-core/src/mcc_map.rs`    | `as_conversions`, `disallowed_types`, `unwrap_used`                   |
| `packages/treeknit-core/src/naive.rs`      | `expect_used`, `unwrap_used`                                          |
| `packages/treeknit-core/src/pair.rs`       | `unwrap_used`                                                         |
| `packages/treeknit-core/src/pipeline.rs`   | `as_conversions`, `disallowed_types`, `expect_used`, `unwrap_used`    |
| `packages/treeknit-core/src/resolve.rs`    | `unwrap_used`                                                         |
| `packages/treeknit-core/src/splitgraph.rs` | `as_conversions`, `expect_used`, `unwrap_used`                        |
| `packages/treeknit-core/src/tree.rs`       | `disallowed_types`, `expect_used`, `unwrap_used`                      |

`just review-suppressions` lists the attributes.

## Fix direction

- **`unwrap_used`, `expect_used`**: most calls rely on tree invariants, such as the parent of a non-root node or the leaf of a taxon that both trees share. Encode the invariant in the types or the traversal (iterate parent-child pairs instead of looking up the parent), or return an error where the input can violate it. `arg.rs` already has `ArgError` for the second case
- **`disallowed_types`, `iter_over_hash_type`**: replace `HashMap` and `HashSet` with `BTreeMap` and `BTreeSet`, or with vectors indexed by `NodeId` where the keys are dense node ids (the node maps of `arg.rs`). The iterations over hash maps in `arg.rs` (`arg_from_trees`, `set_branch_lengths`) write each tree node into the vector slot of its ARG node, so their order reaches the output only if two tree nodes of one tree map to the same ARG node
- **`as_conversions`**: use `From` and `TryFrom` for integer conversions. Counts that become `f64` statistics have no lossless `From`; convert through `u32::try_from` and `f64::from`, or keep a narrow `#[expect]` with a reason on the function

## Validation

- `just lint-rs` passes without the module attribute
- `just test-rs` passes, including the exact comparison with the TreeKnit.jl fixtures in `packages/treeknit-io/tests/fixtures.rs`
- For changes of map types on hot paths (`anneal.rs`, `splitgraph.rs`, `pair.rs`), compare run times of `just build prod` binaries on the simulated fixtures before and after with `hyperfine`
