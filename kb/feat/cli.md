# `treeknit` command: parity checklist

Counterpart: [`v0/cli.md`](v0/cli.md). The port parses the command line with `clap` in `struct Cli` [[src](../../packages/treeknit-cli/src/main.rs#L71-L166)] and runs it in `fn main` [[src](../../packages/treeknit-cli/src/main.rs#L168-L278)].

## Installation

- [x] **Native binary**: `cargo build --release`, or `just build prod` for the shipped profile, builds the `treeknit` binary. There is no compilation at start-up, so each run starts in milliseconds
- [x] **Prebuilt binaries (new)**: the workflow `.github/workflows/release.yml` builds the CLI every night for the targets in `dev/cross/targets` (Linux x86_64 and aarch64 with glibc and musl, Windows x86_64, macOS x86_64 and aarch64) and publishes the binaries that build as a prerelease on the releases page; a pushed version tag publishes them as a release (`dev/release`) once every target builds. The shipped builds need a recent CPU ([`M-shipped-binaries-require-recent-cpu.md`](../issues/M-shipped-binaries-require-recent-cpu.md))
- [ ] **Shell completion**: Comonicon installs completion for `treeknit`. The port has none
- [x] **`--version` (new)**: prints `treeknit <version>`: `<crate version>-dev` for local builds, `<crate version>-nightly.<UTC time>+<commit>` for nightlies, and the crate version for releases [[src](../../packages/treeknit-cli/build.rs)]

## Arguments

- [x] **Positional**: two or more Newick files. Fewer stop the command with "need at least two tree files" [[src](../../packages/treeknit-cli/src/main.rs#L174-L176)]
- [x] **Value syntax**: `clap` accepts `-o val`, `-o=val`, `--outdir val`, and `--outdir=val`. A negative value needs `=`, for example `--gamma=-1`, because `clap` reads `-1` after a space as an unknown flag

## Options

- [x] **`-o, --outdir`**: default `treeknit_results`. The command creates the directory with its parents
- [x] **`-g, --gamma`**: default 2, from the shared settings. The value takes effect; TreeKnit.jl ignores it. A negative or non-finite value stops the command (see [Input validation](#input-validation))
- [x] **`--seq-lengths`**: numbers separated by whitespace, decimals accepted, one positive number per tree [[src](../../packages/treeknit-cli/src/main.rs#L314-L371)]. The value takes effect; TreeKnit.jl ignores it
- [x] **`--n-mcmc-it`**: default 50, at least 1. The value takes effect; TreeKnit.jl ignores it
- [x] **`--rounds`**: default 1. The value 1 also takes effect with the former method options; TreeKnit.jl ignores the value 1 ([`N-undocumented-differences-from-treeknit-jl.md`](../issues/N-undocumented-differences-from-treeknit-jl.md))
- [x] **`--verbosity-level`**: `-1` to `2`, default 0. See [Logging](#logging)
- [x] **`--resolve <matched|strict|liberal|none>` (new)**: the resolution mode, default `matched` (see [`resolution.md`](resolution.md))
- [x] **`--pre-resolve` (new)**: joint resolution of all trees before inference. Off by default; TreeKnit.jl has it on by default
- [x] **`--no-final-round` (new)**: skips the extra round without resolution for `strict` and `liberal` with more than two trees
- [x] **`--seed` (new)**: default 1, at most 2^53 - 1, so a seed passes through the web app unchanged. Same seed, same output (see [`pipeline.md`](pipeline.md#reproducibility))
- [x] **`--threads` (new)**: worker threads for independent pairs, default 0 for all cores [[src](../../packages/treeknit-cli/src/main.rs#L575-L580)]
- [x] **`--impute` (new)**: also writes the trees with missing leaves placed (see [`partial-overlap.md`](partial-overlap.md))
- [x] **`--help-resolve` (new)**: explains the resolution modes and the former options [[src](../../packages/treeknit-cli/src/main.rs#L14-L41)]

## Flags

- [/] **`--help-defaults`**: hidden. It prints the `--help-resolve` text instead of the TreeKnit.jl text on `--better-trees` and `--better-MCCs`. The positional arguments are not required
- [x] **`--better-trees`, `--better-MCCs`**: hidden former options. They log a deprecation warning and apply the TreeKnit.jl presets, including the number of rounds [[src](../../packages/treeknit-cli/src/main.rs#L398-L477)]. Both together are a `clap` conflict error
- [x] **`--no-resolve`, `--liberal-resolve`, `--resolve-all-rounds`, `--no-pre-resolve`**: hidden former options with the TreeKnit.jl meaning and a deprecation warning. `--resolve-all-rounds` sets only the final-round flag, as in TreeKnit.jl. A former option together with `--resolve`, `--pre-resolve`, or `--no-final-round` is an error [[src](../../packages/treeknit-cli/src/main.rs#L342-L348)]
- [x] **`--naive`**: naive MCCs of each pair; pre-resolution, resolution, and sorting still run
- [x] **`--no-likelihood`**: turns off the likelihood tie break. TreeKnit.jl ignores it
- [x] **`--parallel`**: hidden and without effect, because independent pairs always run in parallel (see [`pipeline.md`](pipeline.md#parallel-mode)). No warning for two trees
- [x] **`-v, --verbose`**: verbosity 1 when `--verbosity-level` is 0
- [x] **`--auspice-view`**: one Auspice JSON file per tree (see [`visualization.md`](visualization.md))
- [/] **`--match-topologies`**: hidden. It maps to `--resolve matched`. The help text calls it a former option, but TreeKnit.jl has no such option ([`N-cli-help-lists-match-topologies-as-former-option.md`](../issues/N-cli-help-lists-match-topologies-as-former-option.md))

## Input validation

- [x] **Tree labels**: the file name without its extension. When two names are equal, every tree gets `<name>_<parent directory>`. Labels that are still not unique stop the command with "input trees must be identifiable by file name" [[src](../../packages/treeknit-cli/src/main.rs#L501-L526)]
- [x] **Shared leaves (new behavior)**: the trees may have different leaf sets. TreeKnit.jl stops with "Trees must share leaves". The log reports the number of missing leaves per tree [[src](../../packages/treeknit-cli/src/main.rs#L537-L549)]. A pair with fewer than two shared leaves stops the command with `trees "<a>" and "<b>" share fewer than 2 leaves`, with the resolution modes and with the former options ([`H-pairs-with-fewer-than-two-shared-leaves.md`](../issues/H-pairs-with-fewer-than-two-shared-leaves.md))
- [x] **Shared checks (new)**: the command line and the web app validate with the same functions of `treeknit_io::analysis` [[src](../../packages/treeknit-io/src/analysis.rs#L159-L170)]: labels usable as file names, distinct pair file names such as `MCCs_<a>_<b>.dat`, Newick syntax, γ finite and not negative, one finite positive sequence length per tree, at least one round and one MCMC step, and a seed of at most 2^53 - 1. The command reports every error, one per line, and exits with 1 before it writes `parameters.json` or any result. An error of a tree starts with the path of its file and, for a Newick syntax error, the line and column (`ha.nwk:2:8: ...`). The former options get the same checks of γ, the sequence lengths, the rounds, the MCMC steps, and the seed
- [x] **Sequence lengths**: the number format is checked by the command line, the count and the values by the shared checks
- [x] **Rounds (new)**: `--rounds 0` stops with "rounds must be at least 1", also with the former options
- [x] **Newick syntax**: a wider dialect than TreeTools.jl (see [`formats.md`](formats.md#input-newick))
- [ ] **Configuration warnings**: TreeKnit.jl warns about resolution in the final round for more than two trees, and about `--better-trees` for two trees. The port has no such warnings. It logs the round plan instead, for example "γ = 2, resolution: strict, pre-resolve: true, 1 round(s) + final round without resolution" [[src](../../packages/treeknit-cli/src/main.rs#L373-L383)]

## Run sequence

- [x] **Output directory and log**: created before the trees are read
- [x] **Reading**: read each file, assign labels, and build one taxon table from all leaves
- [x] **`parameters.json`**: written before the inference, with the values the run uses (see [`formats.md`](formats.md#parametersjson))
- [x] **Inference**: `treeknit_core::run` on the trees, then a log line with the MCC count per pair and the runtime
- [x] **MCC and tree outputs**: `MCCs.json`, one resolved tree per input, Auspice files with `--auspice-view`. New: `MCCs.dat` and imputed trees with `--impute`
- [x] **ARG for two trees**: built from the resolved output trees, which differs from TreeKnit.jl on purpose (see [`arg.md`](arg.md)). The log reports the hybrid count as the number of reassortments [[src](../../packages/treeknit-cli/src/main.rs#L281-L303)]
- [/] **Errors**: an error stops the command with `Error: <message>` and exit code 1. A failed ARG construction logs "ARG construction failed: <reason>; no ARG written" and the command exits with 0 [[src](../../packages/treeknit-cli/src/main.rs#L284-L290)]. In TreeKnit.jl, the error stops the command

## Output directory

See [`formats.md`](formats.md) for the content of each file.

- [x] **`log.txt`**, **`parameters.json`**, **`MCCs.json`**
- [x] **`<name>_resolved<ext>`**: one per input. `<ext>` is the extension of that input file; TreeKnit.jl uses the extension of the first file for all trees [[src](../../packages/treeknit-cli/src/main.rs#L528-L535)]
- [x] **`auspice_<label>.json`**: with `--auspice-view`
- [x] **`ARG/arg.nwk`**, **`ARG/nodes.dat`**, **`ARG/<name>_liberal_resolved<ext>`**: two trees only
- [x] **`MCCs.dat` (new)**: the MCCs in the line format of TreeKnit.jl 0.4. With more than two trees, `MCCs_<a>_<b>.dat` per pair [[src](../../packages/treeknit-cli/src/main.rs#L239-L249)]
- [x] **`<name>_imputed<ext>` (new)**: with `--impute`

## Logging

`fn setup_logging` sends each message to the terminal and to `log.txt` with `simplelog` [[src](../../packages/treeknit-cli/src/main.rs#L551-L573)].

- [/] **`log.txt`**: messages at debug level and above, or more with verbosity 2. Each line has the form `<RFC 3339 time> [LEVEL] <message>`. TreeKnit.jl writes `<path>:<line> [<level>] [HH:MM] - <message>`
- [/] **Terminal**: standard error. Verbosity `-1` shows nothing, `0` info, `1` debug, `2` trace, which adds the likelihoods of tied configurations. TreeKnit.jl still shows warnings at `-1`
- [ ] **Progress bar**: TreeKnit.jl shows a progress bar for annealing runs longer than one second. The port has none
