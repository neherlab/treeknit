# The `topological_ordering` fixer offers no rewrite for some modules

The `topological_ordering` lint of the custom dylint library suggests a machine-applicable reorder of a module's items, and `just dylint-fix` applies it. For some modules the lint reports the ordering finding without a suggestion, so their items have to be moved by hand. `fn reorder_fix` returns `None` when `fn item_chunks` cannot delimit the text of every item, or when the computed order does not cover every item [[src](../../dev/lints/dylint-custom/src/lints/topological_ordering.rs#L384-L467)]; which of these cases applies was not investigated.

A second limit: `cargo fix` applied no suggestion in `treeknit-io` while it fixed the whole workspace at once (`--workspace --all-targets`). Running the fixer for the library target alone (`just dylint_cargo_args='--locked -p treeknit-io --lib' dylint-fix`) applied the suggestions.

## Locations

Modules that had ordering findings without a suggestion, all fixed by hand:

- `packages/treeknit-io/src/`: `analysis.rs`, `analysis/settings.rs`, `display.rs`, `examples.rs`, `figure.rs`, `inspect.rs`, `launch/link.rs`, `launch/location.rs`, `output.rs`, `output/figures.rs`, `palette.rs`, `progress.rs`, `schema.rs`
- `packages/treeknit-cli/src/`: `input.rs`, `main.rs`, and `packages/treeknit-cli/build.rs`
- `packages/treeknit-wasm/src/log_capture.rs` and `packages/treeknit-wasm/build.rs`

Most of these modules hold types with derives of `serde` or `tsify`, and the two build scripts take their helper from `include!`.

## Fix direction

- Reproduce on one of the modules above in the UI tests of the lint, find the case of `fn item_chunks` that fails, and delimit items that derive macros or `include!` produce. treetime.rs carries the same library

## Validation

- `just dylint-fix` reorders a module with a `serde` derive, and `./dev/docker/run just dylint` then reports no `topological_ordering` finding for it
