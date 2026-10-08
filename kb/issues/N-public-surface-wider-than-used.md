# Public items wider than their use

`just hawk` (cargo-hawk) reports 424 public items whose visibility is wider than their uses: 409 `pub` items that only their own crate uses (`hawk::unnecessary_public`) and 15 `pub(crate)` or `pub(super)` items that a narrower visibility would serve (`hawk::unnecessary_restricted_visibility`). No public item is unreachable from the production binary (`hawk::dead_public`), apart from `Taxa::is_empty`, which `.config/hawk.toml` records. The checks skip `util-newick`, whose API stays identical to its TreeTime copy (`public_api_crates` in `justfile`). The recipe reports the findings without failing, so they do not block a check.

## Locations

| Crate           | `unnecessary_public` | `unnecessary_restricted_visibility` |
| --------------- | -------------------- | ----------------------------------- |
| `treeknit-io`   | 370                  | 15                                  |
| `treeknit-core` | 39                   | 0                                   |

Most findings are in [packages/treeknit-io/src/display.rs](../../packages/treeknit-io/src/display.rs) (187), [packages/treeknit-io/src/schema.rs](../../packages/treeknit-io/src/schema.rs) (26), and [packages/treeknit-io/src/inspect.rs](../../packages/treeknit-io/src/inspect.rs) (22). `./dev/docker/run just hawk` lists every finding with its file and line.

## Fix direction

- Narrow each item to the visibility hawk suggests
- Record an item that stays public on purpose as an `[[override]]` with a reason in `.config/hawk.toml`

## Validation

- `./dev/docker/run just hawk` reports no finding
- `./dev/docker/run just lint-rs` and `./dev/docker/run just test-rs` pass
