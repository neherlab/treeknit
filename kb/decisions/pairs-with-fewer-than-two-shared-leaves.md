# Pairs with fewer than two shared leaves

TreeKnit.jl requires every tree to have the same leaves. The port accepts trees with different leaf sets ([`kb/feat/partial-overlap.md`](../feat/partial-overlap.md)) and infers the MCCs of each pair on the leaves both trees share. A pair that shares no leaf or one leaf has no MCC to infer: an MCC needs a subtree that both trees have, and one leaf gives no topology to compare.

## Decision

- **A request is invalid on both surfaces.** The shared tree checks of `treeknit_io::analysis` report every pair with fewer than two shared leaves as `trees "<a>" and "<b>" share fewer than 2 leaves` on the field `trees` [[src](../../packages/treeknit-io/src/analysis.rs#L184-L230)]. The command line stops with exit code 1 before it writes `parameters.json`, with the resolution modes and with the former options. The web app shows the error before a run, and `Session.run` throws a `ValidationError`
- **The core skips the pair.** `treeknit_core::run` called directly (a library user, a test) skips such a pair with the warning "trees <a> and <b> share fewer than two leaves: skipped": inference returns no MCCs, and resolution, sorting, and matching leave the pair's trees unchanged [[src](../../packages/treeknit-core/src/pipeline.rs#L435-L477)]. `attach_pair` attaches nothing to a pair without MCCs, so imputation places no leaf through it

## Rationale

- **No result is the true result.** Zero or one shared leaf gives no information on co-inheritance. One MCC per leaf would state the most reassortments, and one MCC of all leaves, the earlier behavior with one shared leaf, would state that the trees share every branch. The input supports neither
- **Error before a run.** A user who loads such a pair has the wrong trees or the wrong leaf names (for example different strain-name formats). An error that names the pair says what to fix; a skipped pair in a finished run is easy to miss
- **No panic in the core.** The earlier `restrict_pair` panicked on a pair without shared leaves, and a panic traps the WebAssembly instance. The skip at the pair boundary makes the core total, so the validation is the user-facing rule and not a guard against a crash

## Tests

- **Core**: `pair_without_shared_leaves_is_skipped` and `pair_with_one_shared_leaf_is_skipped` in `packages/treeknit-core/src/pipeline.rs`, in all four resolution modes
- **Validation**: `pairs_sharing_fewer_than_two_leaves_are_rejected` in `packages/treeknit-io/src/analysis.rs`
- **Command line**: `pairs_sharing_fewer_than_two_leaves_exit_with_their_message` in `packages/treeknit-cli/tests/cli.rs`, on the `--resolve` path and on a former-options path
