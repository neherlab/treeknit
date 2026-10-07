# Dylint findings fail `just dylint`

`just dylint` fails: the custom, mordant, and Trail of Bits lint libraries report 367 warnings in the workspace, and the unused-public-code report of the custom library lists 20 public items that no workspace crate uses. The recipe denies warnings, so the full gate (`just check-all`, group `dylint`) fails as well. The findings predate the libraries: the code was written before `just dylint` ran.

## Locations

Warnings by crate (one warning can repeat for the library and its test target):

| Crate            | Warnings |
| ---------------- | -------- |
| `treeknit-io`    | 238      |
| `treeknit-core`  | 101      |
| `treeknit-wasm`  | 24       |
| `treeknit-cli`   | 18       |

The largest groups are `assert_eq` arguments in actual-expected order (52), items that are not ordered callers before callees (43, `topological_ordering`), and `println!` in build scripts reported as debug remnants (21, `debug_remnants`). `./dev/docker/run just dylint` lists every finding.

## Fix direction

- Fix the findings crate by crate, smallest first, or record a deliberate exception next to the item with `#[expect(<lint>, reason = "..")]`
- The `println!` calls of `build.rs` and `version_rule.rs` are cargo build-script instructions, which `debug_remnants` cannot tell from debug output; they need a suppression with that reason, or an exemption of build scripts in the lint
- Remove each item that `pub_unused_in_workspace` lists, or narrow its visibility

## Validation

- `./dev/docker/run just dylint` passes
