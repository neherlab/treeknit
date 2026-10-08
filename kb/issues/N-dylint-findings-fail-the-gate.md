# Dylint findings fail `just dylint`

`just dylint` fails: the custom and Trail of Bits lint libraries report 227 warnings in the workspace, and the unused-public-code report of the custom library lists 30 public items that no workspace crate uses. The recipe denies warnings, so the full gate (`just check-all`, group `dylint`) fails as well. The findings predate the libraries: the code was written before `just dylint` ran.

The mordant library does not contribute: `.config/mordant-baseline.toml` records its findings per lint and file, and a run reports only the findings above the recorded counts. The baseline holds `wildcard_over_own_enum` (6), `stringified_error` (10), `derived_field` (2), `bare_bool_args` (2), `parallel_vecs`, `tuple_wants_struct`, and `same_match_twice` (1 each).

## Locations

Warnings by crate, each finding counted once:

| Crate           | Warnings | Unused public items |
| --------------- | -------- | ------------------- |
| `treeknit-io`   | 97       | 2                   |
| `treeknit-core` | 57       | 5                   |
| `util-newick`   | 35       | 23                  |
| `treeknit-wasm` | 27       | 0                   |
| `treeknit-cli`  | 11       | 0                   |

The largest groups are items that are not ordered callers before callees (78, `topological_ordering`), integer literals without a name (37, `unnamed_constant`, 32 of them in the color table of `packages/treeknit-io/src/palette.rs`), `assert_eq` arguments in actual-expected order (32, `assert_eq_arg_misordering`), and `println!` and `eprintln!` calls (28, `debug_remnants`). `./dev/docker/run just dylint` lists every finding.

## Fix direction

- Fix the findings crate by crate, smallest first, or record a deliberate exception next to the item with `#[expect(<lint>, reason = "..")]`
- The `debug_remnants` findings in the examples and in the command line are their intended output (results, help text, links): they need a suppression with that reason. The suppression in front of `eprintln!` in `packages/treeknit-cli/src/main.rs` has no effect, because rustc ignores attributes on a macro call (`unused_attributes`); it belongs on the enclosing match arm or function
- Remove each item that `pub_unused_in_workspace` lists, or narrow its visibility
- Fix a mordant finding, then rewrite the baseline with `just dylint-baseline` so the lower count holds

## Validation

- `./dev/docker/run just dylint` passes
