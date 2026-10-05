# Differences from TreeKnit.jl without a recorded decision

The README section "Deliberate differences from TreeKnit.jl" and `kb/decisions/` record the approved differences. The port has more differences than these. The list below gives each one with the TreeKnit.jl behavior from `kb/feat/v0/`. The items change error handling, inference bounds, command-line behavior, and output files.

## Error handling

- **Incompatible split in the MCC resolution**: TreeKnit.jl calls `resolve!` with `conflict = :fail` and stops the run. The port skips the split with the warning "skipping split incompatible with tree <label>" ([resolve.rs#L300-L305](../../packages/treeknit-core/src/resolve.rs#L300-L305))
- **Failed ARG construction**: TreeKnit.jl stops the command. The port logs the error, writes no ARG, and exits with 0 ([run.rs#L83-L93](../../packages/treeknit-io/src/run.rs#L128-L138))
- **`--verbosity-level -1`**: TreeKnit.jl still shows warnings. The port turns the terminal output off, warnings included ([main.rs#L618-L623](../../packages/treeknit-cli/src/main.rs#L663-L668))

## Inference

- **MCMC steps per temperature**: TreeKnit.jl computes `M = Int(ceil(length(ot1.lleaves) * oa.nMCMC / length(oa.Trange)))` as an unbounded `Int64` (`src/main.jl`). The port bounds the steps at 2^32 - 1, the largest `usize` of 32-bit WebAssembly, so the command line and the web app run the same steps; a pair reaches the bound only with more than 2^32 - 1 leaves times steps per leaf per temperature ([pair.rs#L85-L95](../../packages/treeknit-core/src/pair.rs#L85-L95))

## Command line

- **`--rounds 1` with the former method options**: TreeKnit.jl applies `--rounds` only when the value is not 1, so `--better-MCCs --rounds 1` keeps two rounds for more than two trees. The port applies the value 1 ([main.rs#L534-L536](../../packages/treeknit-cli/src/main.rs#L578-L580)). The README says that the former options reproduce the TreeKnit.jl results
- **`--help-defaults`**: TreeKnit.jl explains the method presets. The port prints the `--help-resolve` text
- **Extension of output trees**: TreeKnit.jl uses the extension of the first input file for all output trees. The port uses the extension of each input file

## Newick input and output

- **Unnamed leaves**: TreeTools.jl names them `NODE_<n>`; the port stops with "unnamed leaf"
- **Numeric internal labels**: TreeTools.jl renames them `<label>__<random>`, which keeps the support value in the name; the port renames them `NODE_<k>`, which drops it
- **Duplicate internal labels**: TreeTools.jl raises an error; the port renames them `NODE_<k>`
- **Root length**: TreeTools.jl writes `:0` after the root; the port writes no root length
- **Several trees in one file**: TreeKnit.jl fails; the port uses the first tree with a warning

## Other output files

- **`parameters.json`**: the field names and the field set differ (see `kb/feat/formats.md`)
- **`log.txt`**: the line format differs
- **ARG branch lengths**: TreeKnit.jl adds `eps()` to each length that it sets, so zero lengths appear as `2.220446049250313e-16`; the port adds nothing ([arg.rs#L490-L542](../../packages/treeknit-core/src/arg.rs#L490-L542))
- **`nodes.dat`**: the port ends the file with a newline and writes the lines in ARG node order

> [!IMPORTANT]
> **Decision required.** For each item: record it as a deliberate difference (in the README section or in `kb/decisions/`), or change the port to match TreeKnit.jl. The items most likely to matter to users are the error handling (a run that TreeKnit.jl stops completes in the port) and `--rounds 1`, because the README promises TreeKnit.jl results for the former options.

## Validation

- Each item is in the README section or in `kb/decisions/`, or a test shows the TreeKnit.jl behavior
