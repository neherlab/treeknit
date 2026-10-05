# `treeknit` command

The `treeknit` command is the command-line interface of TreeKnit.jl. Comonicon generates it from `function treeknit` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L36-L179)]. The command reads two or more Newick files and infers the MCCs of every pair of trees. It writes the MCCs, the resolved trees, and the parameters to one directory. For exactly two trees, it also builds an ARG.

This page describes the behavior of the code. Five options have no effect; [`documented-vs-actual.md`](documented-vs-actual.md) gives the cause.

## Installation

- **Build step**: `Pkg.build("TreeKnit")` runs `deps/build.jl`, which calls `TreeKnit.comonicon_install()` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/deps/build.jl#L1-L2)]. This writes the `treeknit` script to `~/.julia/bin`
- **Comonicon settings**: the command name is `treeknit`, shell completion is installed, and the script runs Julia with `optimize = 2` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/Comonicon.toml#L1-L6)]
- **Start-up cost**: Julia compiles the code at each call, which adds some seconds per run. The documentation recommends a Julia session for many tree pairs [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/overview.md?plain=1#L13-L14)]
- **Julia version**: the documentation states that TreeKnit needs Julia 1.7 (1.7.0 released 2021-11-30) and can fail with other versions [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/index.md?plain=1#L6)]. Commit [`856d3c7`](https://github.com/PierreBarrat/TreeKnit.jl/commit/856d3c7), after 0.5.8, makes the `@main` macro work in Julia 1.11 (1.11.0 released 2024-10-08)

## Arguments

- **Positional**: `nwk_file1 nwk_file2 [nwk_files...]`. Two files are required and any number can follow [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L36-L37)]
- **Value syntax**: Comonicon accepts `-o=val`, `-o val`, and `-oval` for short options, and `--option=val` and `--option val` for long ones [[src](https://github.com/comonicon/Comonicon.jl/blob/fe08cdb1a458f08df9b40bd515f3e165d3afc8c9/src/codegen/julia.jl#L414-L448)] [[src](https://github.com/comonicon/Comonicon.jl/blob/fe08cdb1a458f08df9b40bd515f3e165d3afc8c9/src/codegen/julia.jl#L499-L522)]. The `blab/cartography` pipeline calls `treeknit ha na --better-MCCs -o=<dir> -g=2` (see [`treeknit-ecosystem.md`](../../reports/treeknit-ecosystem.md))

## Options

The defaults below are the values in the function signature [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L38-L45)].

- **`-o, --outdir <path>`**: output directory, default `treeknit_results`. The command creates it with all parent directories
- **`-g, --gamma <float>`**: the cost $\gamma$ of one removed coarse-grained leaf, default `OptArgs().γ = 2`. The command logs the value. It has no effect
- **`--seq-lengths "<int> <int> ..."`**: one sequence length per tree, separated by single spaces, default `1` for each tree. The command checks the format and the count. The value has no effect
- **`--n-mcmc-it <int>`**: annealing steps per leaf, default `OptArgs().nMCMC = 50`. It has no effect
- **`--rounds <int>`**: number of rounds over all tree pairs. The value replaces the method default only when it is not `1` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L277)]
- **`--verbosity-level <int>`**: console detail from `-1` to `2`, default `0`. See [Logging](#logging)

## Flags

- **`--help-defaults`**: prints a Markdown text that explains `--better-trees` and `--better-MCCs`, then exits before it creates the output directory [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L59-L62)]. The two positional arguments are still required, but they can be dummy values
- **`--better-trees`**: no resolution between pairs, one round. This is the default for more than two trees. See [`pipeline.md`](pipeline.md)
- **`--better-MCCs`**: resolution after each pair. With more than two trees, a second round without resolution follows. This is the default for two trees. Both method flags together are an error: "Cannot use both `--better-trees` and `--better-MCCs`" [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L251-L253)]
- **`--naive`**: uses the naive MCCs of each pair instead of the annealing (see [`mcc-inference.md`](mcc-inference.md)). The pre-resolution, the resolution between pairs, and the sorting still run
- **`--no-resolve`**: sets `resolve = false`. Trees are not resolved during inference or after each pair
- **`--liberal-resolve`**: sets `strict = false`. See [`resolution.md`](resolution.md)
- **`--resolve-all-rounds`**: sets `final_no_resolve = false` and leaves `resolve` unchanged
- **`--no-pre-resolve`**: skips the joint resolution of all trees before the pairs
- **`--no-likelihood`**: has no effect
- **`--parallel`**: with two trees, the command warns "Cannot run in parallel for 2 trees" and turns the flag off. With more trees, it logs "Running in parallel" and has no effect
- **`-v, --verbose`**: sets the verbosity to `1` when `--verbosity-level` is `0`
- **`--auspice-view`**: also writes one Auspice JSON file per tree. See [`visualization.md`](visualization.md)

## Input validation

- **Tree labels**: each tree gets the file name without its extension as its label. If two files have the same name, every tree gets the name of the first file plus `_` and the name of its own parent directory, for example `tree_examples1` and `tree_examples2`. If the labels are still not unique, the command stops with "Input trees must be identifiable by file name" [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L317-L329)]
- **Shared leaves**: every tree must have the same leaf labels as the first tree, otherwise the command stops with "Trees must share leaves" [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L86-L90)]. Topology and internal labels are not compared
- **Sequence lengths**: the value must be integers separated by single spaces, and the count must equal the number of trees. Otherwise the command logs "Unrecognized format for `--seq-lengths`" and stops with a new `ErrorException` that holds the original error as text [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L93-L101)]
- **Newick syntax**: TreeTools.jl reads the files. One tree per line, each line ends with `;`. See [`formats.md`](formats.md) for the accepted dialect
- **Warnings**: the command warns in two cases [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L279-L286)]:
  - More than two trees, resolution on, and `final_no_resolve = false`: resolution in the final round is not recommended
  - Two trees and `final_no_resolve = true`, which happens only with `--better-trees`: the final round does not resolve

## Run sequence

1. Create the output directory and open `log.txt`
2. Read the trees and check the labels and leaves
3. Build `OptArgs` from the method and the flags, and write `parameters.json`. The file is written before the inference, so it holds the values before the run changes them (see [`pipeline.md`](pipeline.md))
4. Run `run_treeknit!` on copies of the input trees and log the number of MCCs per pair and the runtime [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L139-L145)]
5. Write `MCCs.json` and one resolved tree per input. With `--auspice-view`, write the Auspice files
6. For exactly two trees: resolve the input trees liberally with the MCCs, write them to `ARG/`, build the ARG from them, and write `ARG/arg.nwk` and `ARG/nodes.dat`. The ARG uses the input trees, which step 4 did not change. The log reports the number of hybrid nodes as the number of reassortments [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L159-L171)]

An error in any step stops the command. The code has no `try`/`finally`, so after an error the log file stays open and the global logger is not restored.

## Output directory

[`formats.md`](formats.md) describes the content of each file.

- `log.txt`: the full log of the run
- `parameters.json`: the `OptArgs` of the run
- `MCCs.json`: the MCCs of every tree pair
- `<name>_resolved<ext>`: one resolved and sorted tree per input. `<ext>` is the extension of the first input file, used for all trees [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L304-L314)]
- `auspice_<label>.json`: one per tree, only with `--auspice-view`
- `ARG/arg.nwk`, `ARG/nodes.dat`, `ARG/<name>_liberal_resolved<ext>`: only for two trees

The upstream repository has an example of this directory for two H3N2 trees [[src](https://github.com/PierreBarrat/TreeKnit.jl/tree/dbbc89ac691fed0949a622eedbae103787b89320/examples/treeknit_results)].

## Logging

The command sends each message to two loggers [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L337-L384)]. Every message gets the prefix `[HH:MM] - `.

- **`log.txt`**: always gets messages at level `-1` and higher, which is the detail of verbosity 1. Each line has the form `<path>:<line> [<level>] [HH:MM] - <message>`, where the path starts at the first `TreeKnit` in the source path
- **Console**: shows messages at level `-verbosity` and higher, relabelled as `Info`. Warnings always show, also at verbosity `-1`. Verbosity `2` shows the configurations and likelihoods of each annealing step
- **Progress bar**: the annealing shows a ProgressMeter bar "Simulated annealing: " with the temperature and the best score [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/energy.jl#L271-L301)]. The verbosity does not control it. It appears when one annealing run takes more than one second
