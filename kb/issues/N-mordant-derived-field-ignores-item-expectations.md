# Mordant `derived_field` ignores `#[expect]` on the struct

The `derived_field` lint of the mordant library (`dev/lints/dylint-mordant`) reports a struct field that always holds the same value for a given sibling field. An `#[expect(derived_field)]` or `#[allow(derived_field)]` on the struct has no effect: the expectation is reported as unfulfilled, and the finding still counts against the baseline. Only a crate-level `#![expect]` reaches it, which would hide every other struct of the crate.

The lint collects the constructions of each struct during the crate and reports in `check_crate_post` [[src](../../dev/lints/dylint-mordant/src/derived_field.rs#L186-L250)]. It reports through `emit_with_note`, which reads the lint level at `cx.last_node_with_lint_attrs` [[src](../../dev/lints/dylint-mordant/src/baseline.rs#L299-L318)]. After the crate pass, that node is the crate root, not the struct.

## Locations

- `SettingKey` in `packages/treeknit-io/src/schema.rs`: the rows of the table `SETTING_KEYS` hold the key and the help of their setting by design, so the two findings stay in `.config/mordant-baseline.toml` instead of next to the struct

## Fix direction

- Report through `emit_hir_then` with the `HirId` of the struct (`cx.tcx.local_def_id_to_hir_id` of its local `DefId`), so the level is read where the finding points, as the other mordant lints do for a match arm. treetime.rs carries the same library in `dev/lints/dylint-mordant`, so the change belongs in both copies
- Then move the exception for `SettingKey` from the baseline to an `#[expect]` with its reason on the struct, and rewrite the baseline with `just dylint-baseline`

## Validation

- An `#[cfg_attr(dylint_lib = "mordant", expect(derived_field, reason = ".."))]` on `SettingKey` is fulfilled, and `./dev/docker/run just dylint` reports no `derived_field` finding with the entry removed from the baseline
