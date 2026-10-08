# Dylint findings fail `just dylint`

`just dylint` fails: the custom and Trail of Bits lint libraries report 168 warnings in the workspace. The recipe denies warnings, so the full gate (`just check-all`, group `dylint`) fails as well. The findings predate the libraries: the code was written before `just dylint` ran. All remaining findings concern style and structure; the findings about lost errors, printed output, and type design are fixed or carry an `#[expect]` with its reason.

The mordant library does not contribute: `.config/mordant-baseline.toml` records its findings per lint and file, and a run reports only the findings above the recorded counts. The baseline holds only `derived_field` (2) on `SettingKey` in `packages/treeknit-io/src/schema.rs`: the rows of the table `SETTING_KEYS` hold the key and the help that their setting decides, by design. An `#[expect]` on the struct cannot record this exception, because the lint reads its level at the crate root ([kb/issues/N-mordant-derived-field-ignores-item-expectations.md](N-mordant-derived-field-ignores-item-expectations.md)).

## Locations

Warnings by crate, each finding counted once:

| Crate           | Warnings |
| --------------- | -------- |
| `treeknit-io`   | 77       |
| `treeknit-core` | 54       |
| `util-newick`   | 18       |
| `treeknit-wasm` | 14       |
| `treeknit-cli`  | 5        |

By lint:

- **`topological_ordering`** (78): items that are not ordered callers before callees
- **`unnamed_constant`** (37): integer literals without a name, 32 of them in the color table of `packages/treeknit-io/src/palette.rs`
- **`assert_eq_arg_misordering`** (31): `assert_eq` arguments in actual-expected order, all in the tests of `treeknit-core`
- **`value_cloned_to_deserialize`** (12): `serde_json::from_value(v.clone())` instead of deserializing from the borrowed `&Value`, in `treeknit-io`
- **`file_too_long`** (6): `packages/treeknit-io/src/launch.rs`, `packages/treeknit-core/src/pipeline.rs`, `packages/treeknit-io/src/output.rs`, `packages/treeknit-cli/src/main.rs`, `packages/treeknit-io/src/analysis.rs`, `packages/treeknit-cli/tests/cli.rs`
- **`handwritten_fmt_impl`** (3): hand-written `Display` impls in `util-newick`
- **`assert_in_loop`** (1): in the tests of `packages/treeknit-io/src/display/auspice.rs`

`./dev/docker/run just dylint` lists every finding.

## Fix direction

- Fix the findings crate by crate, smallest first, or record a deliberate exception next to the item with `#[cfg_attr(dylint_lib = "<library>", expect(<lint>, reason = ".."))]`. Put the attribute on the enclosing match arm, `let` statement, or function, because rustc ignores attributes on a macro call (`unused_attributes`). In a test function of `rstest`, put it on a statement of the body: an attribute on the function does not reach the body that `rstest` expands
- `util-newick` stays identical to its TreeTime copy: a change to it, including an `#[expect]`, belongs in TreeTime as well ([packages/util-newick/README.md](../../packages/util-newick/README.md), section Origin)
- The unused-public-code report has no findings. A public item that only another crate's tests use keeps `#[cfg_attr(all(dylint_lib = "custom", not(test)), expect(pub_unused_in_workspace, reason = ".."))]`: the lint takes no part in test builds, so an expectation without `not(test)` is unfulfilled there
- Fix a mordant finding, then rewrite the baseline with `just dylint-baseline` so the lower count holds

## Validation

- `./dev/docker/run just dylint` passes
