# Public items wider than their use

`just hawk` (cargo-hawk) reports 562 public items whose visibility is wider than their uses: 530 `pub` items that only their own crate uses (`hawk::unnecessary_public`), 12 public items that no production path reaches (`hawk::dead_public`), and 20 `pub(crate)` or `pub(super)` items that a narrower visibility would serve (`hawk::unnecessary_restricted_visibility`). The recipe reports the findings without failing, so they do not block a check.

## Locations

| Crate           | `unnecessary_public` | `dead_public` | `unnecessary_restricted_visibility` |
| --------------- | -------------------- | ------------- | ----------------------------------- |
| `treeknit-io`   | 371                  | 1             | 15                                  |
| `util-newick`   | 118                  | 9             | 4                                   |
| `treeknit-core` | 41                   | 2             | 1                                   |

Most findings are in [packages/treeknit-io/src/display.rs](../../packages/treeknit-io/src/display.rs) (189), [packages/treeknit-io/src/schema.rs](../../packages/treeknit-io/src/schema.rs) (26), [packages/util-newick/src/model/data.rs](../../packages/util-newick/src/model/data.rs) (25), and [packages/treeknit-io/src/inspect.rs](../../packages/treeknit-io/src/inspect.rs) (22). `./dev/docker/run just hawk` lists every finding with its file and line.

## Fix direction

- Narrow each item to the visibility hawk suggests, and delete the items that no production path reaches
- Record an item that stays public on purpose as an `[[override]]` with a reason in `.config/hawk.toml`

## Validation

- `./dev/docker/run just hawk` reports no finding
- `./dev/docker/run just lint-rs` and `./dev/docker/run just test-rs` pass
