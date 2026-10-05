# `treeknit` command: parity checklist

Counterpart: [`v0/cli.md`](v0/cli.md). The port parses the command line with `clap` in `struct Cli` [[src](../../packages/treeknit-cli/src/main.rs#L71-L166)] and runs it in `fn main` [[src](../../packages/treeknit-cli/src/main.rs#L168-L277)].

## Installation

- [x] **Native binary**: `cargo build --release`, or `just build prod` for the shipped profile, builds the `treeknit` binary. There is no compilation at start-up, so each run starts in milliseconds
- [x] **Prebuilt binaries (new)**: the workflow `.github/workflows/release.yml` builds the CLI every night for the targets in `dev/cross/targets` (Linux x86_64 and aarch64 with glibc and musl, Windows x86_64, macOS x86_64 and aarch64) and publishes the binaries that build as a prerelease on the releases page; a pushed version tag publishes them as a release (`dev/release`) once every target builds. The shipped builds need a recent CPU ([`M-shipped-binaries-require-recent-cpu.md`](../issues/M-shipped-binaries-require-recent-cpu.md))
- [ ] **Shell completion**: Comonicon installs completion for `treeknit`. The port has none
- [x] **`--version` (new)**: prints `treeknit <version>`: `<crate version>-dev` for local builds, `<crate version>-nightly.<UTC time>+<commit>` for nightlies, and the crate version for releases [[src](../../packages/treeknit-cli/build.rs)]

## Arguments

- [x] **Positional**: two or more Newick files. Fewer stop the command with "need at least two tree files" [[src](../../packages/treeknit-cli/src/main.rs#L189-L191)]. With `--request`, no tree file is given
- [x] **Value syntax**: `clap` accepts `-o val`, `-o=val`, `--outdir val`, and `--outdir=val`. A negative value needs `=`, for example `--gamma=-1`, because `clap` reads `-1` after a space as an unknown flag

## Options

- [x] **`-o, --outdir`**: default `treeknit_results`. The command creates the directory with its parents
- [x] **`-g, --gamma`**: default 2, from the shared settings. The value takes effect; TreeKnit.jl ignores it. A negative or non-finite value stops the command (see [Input validation](#input-validation))
- [x] **`--seq-lengths`**: numbers separated by whitespace, decimals accepted, one positive number per tree [[src](../../packages/treeknit-cli/src/main.rs#L395-L406)]. The value takes effect; TreeKnit.jl ignores it
- [x] **`--n-mcmc-it`**: default 50, at least 1 and at most 2^32 - 1 (4294967295). The value takes effect; TreeKnit.jl ignores it
- [x] **`--rounds`**: default 1, at most 2^32 - 2 (4294967294). The value 1 also takes effect with the former method options; TreeKnit.jl ignores the value 1 ([`N-undocumented-differences-from-treeknit-jl.md`](../issues/N-undocumented-differences-from-treeknit-jl.md))
- [x] **`--verbosity-level`**: `-1` to `2`, default 0. See [Logging](#logging)
- [x] **`--resolve <matched|strict|liberal|none>` (new)**: the resolution mode, default `matched` (see [`resolution.md`](resolution.md))
- [x] **`--pre-resolve` (new)**: joint resolution of all trees before inference. Off by default; TreeKnit.jl has it on by default
- [x] **`--no-final-round` (new)**: skips the extra round without resolution for `strict` and `liberal` with more than two trees
- [x] **`--seed` (new)**: default 1, at most 2^53 - 1, so a seed passes through the web app unchanged. Same seed, same output (see [`pipeline.md`](pipeline.md#reproducibility))
- [x] **`--threads` (new)**: worker threads for independent pairs, default 0 for all cores [[src](../../packages/treeknit-cli/src/main.rs#L610-L615)]
- [x] **`--impute` (new)**: also writes the trees with missing leaves placed (see [`partial-overlap.md`](partial-overlap.md))
- [x] **`--request <FILE>` (new)**: runs the trees and settings of a session file, `treeknit_request.json`, which the web app saves (see [`formats.md`](formats.md#session-file-treeknit_requestjson)) [[src](../../packages/treeknit-cli/src/main.rs#L291-L310)]. It cannot be combined with tree files or with the analysis options (`--gamma`, `--seq-lengths`, `--n-mcmc-it`, `--resolve`, `--pre-resolve`, `--rounds`, `--no-final-round`, `--no-likelihood`, `--naive`, `--seed`, and the former options); `-o`, `-v`, `--verbosity-level`, `--threads`, `--impute`, and `--auspice-view` apply. The trees keep the labels of the file, and their output files get the extension `.nwk`. The command also writes the request it ran to `treeknit_request.json` in the results directory. `treeknit --request treeknit_results/treeknit_request.json --impute --auspice-view`, run where the ZIP archive of the web app was extracted, writes the file set of the web app (`treeknit_io::output::command_line`)
- [x] **`--help-resolve` (new)**: explains the resolution modes and the former options [[src](../../packages/treeknit-cli/src/main.rs#L14-L41)]

## Flags

- [/] **`--help-defaults`**: hidden. It prints the `--help-resolve` text instead of the TreeKnit.jl text on `--better-trees` and `--better-MCCs`. The positional arguments are not required
- [x] **`--better-trees`, `--better-MCCs`**: hidden former options. They log a deprecation warning and apply the TreeKnit.jl presets, including the number of rounds [[src](../../packages/treeknit-cli/src/main.rs#L430-L512)]. Both together are a `clap` conflict error
- [x] **`--no-resolve`, `--liberal-resolve`, `--resolve-all-rounds`, `--no-pre-resolve`**: hidden former options with the TreeKnit.jl meaning and a deprecation warning. `--resolve-all-rounds` sets only the final-round flag, as in TreeKnit.jl. A former option together with `--resolve`, `--pre-resolve`, or `--no-final-round` is an error [[src](../../packages/treeknit-cli/src/main.rs#L358-L368)]
- [x] **`--naive`**: naive MCCs of each pair; pre-resolution, resolution, and sorting still run
- [x] **`--no-likelihood`**: turns off the likelihood tie break. TreeKnit.jl ignores it
- [x] **`--parallel`**: hidden and without effect, because independent pairs always run in parallel (see [`pipeline.md`](pipeline.md#parallel-mode)). No warning for two trees
- [x] **`-v, --verbose`**: verbosity 1 when `--verbosity-level` is 0
- [x] **`--auspice-view`**: one Auspice JSON file per tree (see [`visualization.md`](visualization.md))
- [/] **`--match-topologies`**: hidden. It maps to `--resolve matched`. The help text calls it a former option, but TreeKnit.jl has no such option ([`N-cli-help-lists-match-topologies-as-former-option.md`](../issues/N-cli-help-lists-match-topologies-as-former-option.md))

## Input validation

- [x] **Tree labels**: `fn path_labels`: the file name without its extension. When two names are equal, also when they differ only in case (`treeknit_io::analysis::label_key`), every tree gets `<name>_<parent directory>`. Labels that are still not unique stop the command with "input trees must be identifiable by file name" [[src](../../packages/treeknit-cli/src/main.rs#L531-L560)]. The web app labels trees by file name only, with `_2`, `_3` on a collision (`treeknit_io::analysis::tree_labels`), because it has no directories
- [x] **Shared leaves (new behavior)**: the trees may have different leaf sets. TreeKnit.jl stops with "Trees must share leaves". The log reports the number of missing leaves per tree [[src](../../packages/treeknit-io/src/run.rs#L68-L81)]. A pair with fewer than two shared leaves stops the command with `trees "<a>" and "<b>" share fewer than 2 leaves`, with the resolution modes, with the former options, and with `--request` ([`kb/decisions/pairs-with-fewer-than-two-shared-leaves.md`](../decisions/pairs-with-fewer-than-two-shared-leaves.md))
- [x] **Shared checks (new)**: the command line and the web app validate with the same functions of `treeknit_io::analysis` [[src](../../packages/treeknit-io/src/analysis.rs#L160-L172)]: labels usable as file names, distinct pair file names such as `MCCs_<a>_<b>.dat`, Newick syntax, γ finite and not negative, one finite positive sequence length per tree, at least one and at most 2^32 - 2 rounds, at least one and at most 2^32 - 1 MCMC steps per leaf, and a seed of at most 2^53 - 1. The command reports every error, one per line, and exits with 1 before it writes `parameters.json` or any result. An error of a tree starts with the path of its file and, for a Newick syntax error, the line and column (`ha.nwk:2:8: ...`). The former options get the same checks of γ, the sequence lengths, the rounds, the MCMC steps, and the seed
- [x] **Sequence lengths**: the number format is checked by the command line, the count and the values by the shared checks
- [x] **Rounds (new)**: `--rounds 0` stops with "rounds must be at least 1", also with the former options. The web app runs the core as 32-bit WebAssembly, whose counts end at 2^32 - 1, and a final round can come on top of `--rounds`; so both surfaces accept at most 2^32 - 2 rounds and 2^32 - 1 MCMC steps per leaf (`treeknit_io::analysis::MAX_ROUNDS`, `MAX_MCMC_IT`), and a larger value stops with, for example, "rounds must be at most 4294967294, got 4294967295"
- [x] **Newick syntax**: a wider dialect than TreeTools.jl (see [`formats.md`](formats.md#input-newick))
- [ ] **Configuration warnings**: TreeKnit.jl warns about resolution in the final round for more than two trees, and about `--better-trees` for two trees. The port has no such warnings. It logs the round plan instead, for example "γ = 2, resolution: strict, pre-resolve: true, 1 round(s) + final round without resolution" [[src](../../packages/treeknit-cli/src/main.rs#L408-L418)]

## Run sequence

- [x] **Output directory and log**: created before the trees are read
- [x] **Reading**: read each file, assign labels, and build one taxon table from all leaves
- [x] **`parameters.json`**: written before the inference, with the values the run uses (see [`formats.md`](formats.md#parametersjson))
- [x] **Inference**: `treeknit_io::run::run` builds the run result that the web app builds too: `treeknit_core::run_observed` on the trees, the imputed trees, and for two trees the ARG [[src](../../packages/treeknit-io/src/run.rs#L44-L66)]. A log line then gives the MCC count per pair and the runtime
- [x] **MCC and tree outputs**: `fn output_files` of `treeknit-io` builds every output file from the run result, with the bytes the command line has always written, and the command writes them in order, creating `ARG/` when needed [[src](../../packages/treeknit-io/src/output.rs#L78-L141)]: `MCCs.json`, one resolved tree per input, Auspice files with `--auspice-view`. New: `MCCs.dat` and imputed trees with `--impute`. The test `output_files_keep_their_bytes` compares every file of a two-tree run and of a three-tree run with `--impute --auspice-view` byte for byte with the captured output of the command [[src](../../packages/treeknit-cli/tests/cli.rs#L215)]
- [x] **ARG for two trees**: built from the resolved output trees, which differs from TreeKnit.jl on purpose (see [`arg.md`](arg.md)). The log reports the hybrid count as the number of reassortments [[src](../../packages/treeknit-io/src/run.rs#L83-L93)]
- [/] **Errors**: an error stops the command with `Error: <message>` and exit code 1. A failed ARG construction logs "ARG construction failed: <reason>; no ARG written" and the command exits with 0 [[src](../../packages/treeknit-io/src/run.rs#L83-L93)]. In TreeKnit.jl, the error stops the command

## Output directory

See [`formats.md`](formats.md) for the content of each file.

- [x] **`log.txt`**, **`parameters.json`**, **`MCCs.json`**
- [x] **`<label>_resolved<ext>`**: one per input, named by the tree label, so files with one name in different directories (`a/ha.nwk`, `b/ha.nwk`) give `ha_a_resolved.nwk` and `ha_b_resolved.nwk`. `<ext>` is the extension of that input file, `.nwk` with `--request`; TreeKnit.jl uses the extension of the first file for all trees [[src](../../packages/treeknit-cli/src/main.rs#L562-L569)]
- [x] **`auspice_<label>.json`**: with `--auspice-view`
- [x] **`ARG/arg.nwk`**, **`ARG/nodes.dat`**, **`ARG/<label>_liberal_resolved<ext>`**: two trees only
- [x] **`MCCs.dat` (new)**: the MCCs in the line format of TreeKnit.jl 0.4. With more than two trees, `MCCs_<a>_<b>.dat` per pair [[src](../../packages/treeknit-io/src/output.rs#L101-L111)]
- [x] **`<label>_imputed<ext>` (new)**: with `--impute`
- [x] **`treeknit_request.json` (new)**: with `--request`, the request that ran

## Logging

`fn setup_logging` sends each message to the terminal and to `log.txt` with `simplelog` [[src](../../packages/treeknit-cli/src/main.rs#L586-L608)].

- [/] **`log.txt`**: messages at debug level and above, or more with verbosity 2. Each line has the form `<RFC 3339 time> [LEVEL] <message>`. TreeKnit.jl writes `<path>:<line> [<level>] [HH:MM] - <message>`
- [/] **Terminal**: standard error. Verbosity `-1` shows nothing, `0` info, `1` debug, `2` trace, which adds the likelihoods of tied configurations. TreeKnit.jl still shows warnings at `-1`
- [ ] **Progress bar**: TreeKnit.jl shows a progress bar for annealing runs longer than one second. The port has none
