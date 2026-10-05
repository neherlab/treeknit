# Feature parity of the Rust port

The documents in `kb/feat/` compare this port with TreeKnit.jl, feature by feature. Each document takes one document of [`v0/`](v0/overview.md), which describes what TreeKnit.jl does, and marks each feature in the port:

- `[x]` done: the port has the feature. When the result differs from TreeKnit.jl on purpose, the item says so
- `[/]` partial: the port has part of the feature, or has the feature without the TreeKnit.jl interface
- `[ ]` not done

Items marked **(new)** have no counterpart in TreeKnit.jl. Each item links to the code of the port. The `v0/` documents give the TreeKnit.jl side.

Surveyed revision: `main` at `93a8e03` (2026-10-05). The survey reads the code and runs the `treeknit` binary. It does not rely on the README or on code comments.

## Summary

- **Method for two trees**: done. Naive MCCs, the split graph energy, the simulated annealing, the branch-length tie break, and the iteration in `runopt` match TreeKnit.jl
- **Pipeline for more than two trees**: done, with a new default resolution mode (`matched`), seeded runs, and race-free parallel pairs. The presets of TreeKnit.jl remain available
- **Command line**: done. Every option of TreeKnit.jl 0.5.8 takes effect, including the five options that TreeKnit.jl ignores. No shell completion and no progress bar
- **ARG for two trees**: done, without the `eps()` offset on branch lengths
- **Output files**: done, with differences in `parameters.json`, `log.txt`, and the Newick root
- **New in this port**: trees with different leaf sets and imputation of missing leaves ([`partial-overlap.md`](partial-overlap.md)), a wider Newick dialect, a web app that runs in the browser ([`web-app.md`](web-app.md)), and prebuilt binaries
- **Defects found in this survey**: two inputs crash the command line, and the ARG output fails in two cases. The issues are listed below

## Documents

- [`cli.md`](cli.md): the `treeknit` command, its options, validation, outputs, and logging
- [`pipeline.md`](pipeline.md): run options, method presets, rounds, the pair order, naive mode, parallel mode, and seeds
- [`mcc-inference.md`](mcc-inference.md): naive MCCs, the split graph, the energy, the annealing, the tie break, and the likelihood
- [`resolution.md`](resolution.md): polytomy resolution with topology only, during inference, with MCCs, and the new `matched` mode
- [`arg.md`](arg.md): construction of the ARG for two trees and its extended Newick output
- [`formats.md`](formats.md): the Newick dialect and every output file
- [`visualization.md`](visualization.md): ladderizing, polytomy sorting for tanglegrams, Auspice JSON, and ARG viewers
- [`library-api.md`](library-api.md): the Rust library interface compared with the Julia library interface
- [`partial-overlap.md`](partial-overlap.md) **(new)**: trees with different leaf sets
- [`web-app.md`](web-app.md) **(new)**: the WebAssembly bindings and the web app

## Where differences are recorded

- **Approved differences**: the README section [Deliberate differences from TreeKnit.jl](../../README.md#deliberate-differences-from-treeknitjl), and [`kb/decisions/web-app.md`](../decisions/web-app.md)
- **Differences without approval**: [`N-undocumented-differences-from-treeknit-jl.md`](../issues/N-undocumented-differences-from-treeknit-jl.md) lists the behavior that differs from TreeKnit.jl and that no document approves yet

## Comparison with TreeKnit.jl

The tests compare the port with fixtures that TreeKnit.jl writes (see [`ref/README.md`](../../ref/README.md)):

- **`deterministic_functions_match_julia`** [[src](../../packages/treeknit-io/tests/fixtures.rs#L109)]: exact comparison of naive MCCs (also per pair for three trees), pre-resolution, strict and liberal resolution with MCCs, the node-to-MCC map, the non-strict polytomy sort, the energy with and without resolution, the likelihood, and the hybrid count and segment-tree splits of the ARG
- **`annealing_distribution_vs_julia`** [[src](../../packages/treeknit-io/tests/fixtures.rs#L301)]: two-tree runs with 20 seeds against the 20 seeded Julia runs of each case. The test fails only where all Julia runs agree and a Rust run differs
- **`matched_topologies_on_three_segments`** [[src](../../packages/treeknit-io/tests/fixtures.rs#L344)]: the invariant of the `matched` mode on simulated three-segment cases
- **Accuracy report**: `examples/accuracy.rs` measures the distance to the true MCCs of simulated cases for Julia and Rust runs [[src](../../packages/treeknit-io/examples/accuracy.rs#L1-L6)]

[`N-reference-comparison-gaps.md`](../issues/N-reference-comparison-gaps.md) lists the features that no test compares with TreeKnit.jl.

## Issues found in this survey

- [`H-pairs-with-fewer-than-two-shared-leaves.md`](../issues/H-pairs-with-fewer-than-two-shared-leaves.md): a pair without shared leaves panics, and a pair with one shared leaf gives one MCC of all leaves
- [`H-cli-accepts-invalid-settings.md`](../issues/H-cli-accepts-invalid-settings.md): the command line accepts a negative γ and zero sequence lengths, and zero lengths make it panic
- [`M-arg-fails-after-imputation.md`](../issues/M-arg-fails-after-imputation.md): the ARG construction fails when imputed trees disagree inside an MCC
- [`M-arg-outputs-unquoted-labels.md`](../issues/M-arg-outputs-unquoted-labels.md): `ARG/arg.nwk` writes labels without quotes
- [`N-undocumented-differences-from-treeknit-jl.md`](../issues/N-undocumented-differences-from-treeknit-jl.md): differences from TreeKnit.jl that need a decision
- [`N-reference-comparison-gaps.md`](../issues/N-reference-comparison-gaps.md): features without a comparison with TreeKnit.jl
- [`N-cli-help-lists-match-topologies-as-former-option.md`](../issues/N-cli-help-lists-match-topologies-as-former-option.md): the help text names an option that TreeKnit.jl never had

Items marked `[ ]` without a linked issue have no work item. They are parity gaps, and the checklist is their record.
