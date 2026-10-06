# Documentation says pairs always run in parallel

The hidden help of `--parallel` says "independent pairs always run in parallel (see --threads)", and `kb/feat/cli.md` repeats it. Pairs run in parallel only in rounds without resolution. With the default `Matched` resolution every round resolves, so the pairs run one after another and `--threads` has no effect ([`parallelism.md`](../proposals/parallelism.md#current-state)). A reader expects a default run on more than two trees to use all cores.

## Locations

- `--parallel` help [packages/treeknit-cli/src/main.rs#L282](../../packages/treeknit-cli/src/main.rs#L282)
- `--threads` help, "Worker threads for independent tree pairs" [packages/treeknit-cli/src/main.rs#L242](../../packages/treeknit-cli/src/main.rs#L242)
- `--parallel` entry [kb/feat/cli.md#L46](../feat/cli.md#L46)
- `--threads` entry, whose source link points to `main.rs#L728-L733` instead of the option [kb/feat/cli.md#L31](../feat/cli.md#L31)

## Fix direction

- State in the `--parallel` and `--threads` help and in `kb/feat/cli.md` that pairs run in parallel only in rounds without resolution: with `--resolve none`, and in the final round of `strict` and `liberal` with more than two trees
- Point the `--threads` source link to the option

## Validation

- `just test-rs -E 'package(treeknit-cli)'` passes
