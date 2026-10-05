# Feature history of TreeKnit.jl

This page lists when each capability of TreeKnit.jl appeared, what was removed, what exists only on unmerged branches, and what the open pull request proposes. The sources are the git history of <https://github.com/PierreBarrat/TreeKnit.jl>, its pull requests (#1 to #42), and its issues (#5, #6, #21). The issue tracker has almost no entries; bug reports are in pull requests.

## Releases

- **Before v0.1.0** (2019 to 2021): the split graph, the simulated annealing, `runopt`, `OptArgs`, the likelihood sort, and the `resolve` option. The `SimpleReassortmentGraph` module and the Comonicon command line came on 2021-10-21 [[src](https://github.com/PierreBarrat/TreeKnit.jl/commit/4b6013939bff6c806db3b7b10362982845c78dc8)]. `log.txt` and `--verbose` came with [#3](https://github.com/PierreBarrat/TreeKnit.jl/pull/3)
- **v0.1.0 (2021-12-10)**: `treeknit nwk1 nwk2` with `-o`, `-g`, `--seq-lengths`, `--n-sa-it`, `--naive`, `--no-likelihood`, `--no-resolve`, `-v`. Outputs `MCCs.dat`, `<name>.resolved.nwk`, `arg.nwk`, `nodes.dat`, `log.txt`. Exports included `computeMCCs` and `inferARG`
- **v0.1.1 (2021-12-20)**, **v0.1.2 (2021-12-22)**: shell wrapper fixes ([#7](https://github.com/PierreBarrat/TreeKnit.jl/pull/7), [#9](https://github.com/PierreBarrat/TreeKnit.jl/pull/9)), a progress bar for the annealing ([#11](https://github.com/PierreBarrat/TreeKnit.jl/pull/11), [#12](https://github.com/PierreBarrat/TreeKnit.jl/pull/12)), and a much faster `are_equal_with_resolution` ([#10](https://github.com/PierreBarrat/TreeKnit.jl/pull/10))
- **v0.2.0 (2022-01-06)**: annealing parameters `nMCMC` and `nT`, the `:acos` schedule, and `--n-mcmc-it` instead of `--n-sa-it` ([#14](https://github.com/PierreBarrat/TreeKnit.jl/pull/14)). Output trees are ladderized and sorted for tanglegrams ([#16](https://github.com/PierreBarrat/TreeKnit.jl/pull/16)). Input files with the same name get directory suffixes
- **v0.2.1 to v0.3.0 (2022-01-07 to 2022-05-06)**: compatibility with Julia 1.6 and new TreeTools versions
- **v0.3.1 (2022-05-18)**: geometric cooling with `nT = 100`, `nMCMC = 50`, `Tmax = 1` ([#19](https://github.com/PierreBarrat/TreeKnit.jl/pull/19)). The `OptArgs` docstring still gives the earlier values
- **v0.3.2 (2022-06-10)**: warning for a root with a branch length in the ARG construction, and a clear error for a bad `--seq-lengths`
- **v0.3.3 (2022-09-12)**: the split mapping functions moved from TreeTools into TreeKnit, which fixed [issue #21](https://github.com/PierreBarrat/TreeKnit.jl/issues/21)
- **v0.4.0 (2022-11-03)**: strict resolution ([#20](https://github.com/PierreBarrat/TreeKnit.jl/pull/20)), merged into `resolve!(...; strict)` ([#28](https://github.com/PierreBarrat/TreeKnit.jl/pull/28)). The ARG is built from liberally resolved trees ([#22](https://github.com/PierreBarrat/TreeKnit.jl/pull/22)). `map_mccs` with the Fitch method replaced earlier MCC mapping functions ([#27](https://github.com/PierreBarrat/TreeKnit.jl/pull/27))
- **v0.5.0 (2023-01-26)**: MultiTreeKnit for two or more trees ([#32](https://github.com/PierreBarrat/TreeKnit.jl/pull/32)): the pair loop with `rounds`, `pre_resolve`, `final_no_resolve`, the `MCC_set` type, Dagger parallel mode, strict resolution as default, and the methods `--better-trees` and `--better-MCCs`. New options `--rounds`, `--verbosity-level`, `--no-pre-resolve`, `--liberal-resolve`, `--resolve-all-rounds`, `--parallel`, `--auspice-view`. `MCCs.json` and `parameters.json` replaced `MCCs.dat`. The ARG files moved to `ARG/` and are written for two trees only. `run_treeknit!` replaced `computeMCCs`. Pipelines written for 0.4 (the Nextstrain seasonal-flu TreeKnit rule and TreeTime `arg`) expect the old file names and the line-based MCC file (see [`treeknit-ecosystem.md`](../reports/treeknit-ecosystem.md))
- **v0.5.2 (2023-03-23)**: the non-mutating `run_treeknit`
- **v0.5.3 (2023-05-26)**: no crash for missing branch lengths in `introduce_singleton!` and a warning for a `missing` likelihood ([#35](https://github.com/PierreBarrat/TreeKnit.jl/pull/35))
- **v0.5.4 (2023-05-28)**: `--help-defaults` ([#37](https://github.com/PierreBarrat/TreeKnit.jl/pull/37))
- **v0.5.5 (2023-05-28)**: ARG fix for missing branch lengths, `:better_mccs` renamed to `:better_MCCs`, warnings shown on the console
- **v0.5.6 (2023-10-18)**, **v0.5.7 (2024-02-22)**: `--gamma` parses again, and `treeknit` without `--gamma` no longer fails ([#38](https://github.com/PierreBarrat/TreeKnit.jl/pull/38), [#39](https://github.com/PierreBarrat/TreeKnit.jl/pull/39)). The value still has no effect (see [`documented-vs-actual.md`](documented-vs-actual.md))
- **v0.5.8 (2024-08-09)**: adaptation to the TreeTools accessor functions
- **After v0.5.8**: the `@main` macro is qualified for Julia 1.11, and the documentation states that Julia 1.7 is required ([#41](https://github.com/PierreBarrat/TreeKnit.jl/pull/41), 2025-02-18)

## Removed capabilities

- **Inputs and outputs**: the fixed two-file arguments became variadic, `--n-sa-it` became `--n-mcmc-it`, `MCCs.dat` became `MCCs.json`, `<name>.resolved.nwk` became `<name>_resolved.nwk`, and `arg.nwk` and `nodes.dat` moved to `ARG/`
- **API**: `computeMCCs` (v0.5.0); the re-exports `node2tree` and `parse_newick` (v0.5.0); `assign_mccs!`, `mcc_map`, `leaf_mcc_map`, `sort_polytomies_strict!`, `resolve_strict!` (v0.4.0)
- **Before v0.1.0**: mutation cross-mapping between segments (`src/mut_crossmap.jl`), influenza helpers (`src/Flu.jl`), artificial data generation (`src/artificialdata.jl`) [[src](https://github.com/PierreBarrat/TreeKnit.jl/commit/94407c4534375c744f9cdb7f84b8539ff7b632f7)]. ARG simulation moved to the separate ARGTools package, and split operations moved to TreeTools
- **Consistency constraint for more than two trees**: an annealing term that pushed the MCCs of different pairs toward consistency, removed before v0.5.0 because it had little effect [[src](https://github.com/PierreBarrat/TreeKnit.jl/commit/698836f24f596276b7f392550e167c5b5dc952ea)]

## Unmerged branches

The count is the number of commits not in `master`.

- **`MultiTreeKnit_extended_newick`** (71, 2022-11-18): an ARG for more than two trees (`src/SimpleReassortmentGraph/construct_multi.jl`), consistency fixing of MCCs, MCC labels, accuracy measures, and ARG plots
- **`MultiTreeKnit_fix_consist`** (83, 2022-10-12): consistency fixing of MCCs for three or more trees, Dagger parallel runs, auspice output, and shared-branch accuracy measures
- **`MultiTreeKnit_v1`** (123, 2022-11-03): the branch behind closed [#25](https://github.com/PierreBarrat/TreeKnit.jl/pull/25). Most of it reached v0.5.0 in a different form
- **`MultiTreeKnit`** (22, 2022-06-08): the first prototype for more than two trees, with ARG plotting
- **`feature/MCC_confidence_branch_likelihood`** (1, 2022-05-03): a confidence score per MCC from a branch-length likelihood ratio (`fn distance_likelihood_ratio()`, `fn confidence_likelihood_ratio()`). No pull request was opened
- **`fix/sort_polytomies_strict`** (3, 2022-10-27): sorting of strict trees by the leaf order of liberal trees. `master` has an equivalent
- **`MTK_clean`** (2, 2023-01-18): aligns the `OptArgs` defaults with the command-line defaults
- **`MTK_no_consistency`** (1, 2023-01-25): a `Project.toml` change only

## Open pull request #42

[#42](https://github.com/PierreBarrat/TreeKnit.jl/pull/42) "Fix/issues from rust port", opened 2026-10-02 from a fork, proposes fixes for defects found while porting TreeKnit.jl. The defects it addresses are present on `master` (see [`documented-vs-actual.md`](documented-vs-actual.md)):

- **Option handling**: `OptArgs(K; ...)` drops `γ`, `nMCMC`, `likelihood_sort`, and `parallel`, and misspells `seq_lengths`
- **Parallel mode**: the keyword mismatch, the concurrent resolution of shared trees, the shared mutable `OptArgs`, and the loss of all rounds but the last
- **Naive mode**: `run_treeknit!(trees; naive = true)` ignores `naive`
- **Naive MCCs for three or more trees**: the `MethodError` in `is_coherent_clade`
- **Missing likelihood**: configurations with a `missing` likelihood win the tie break
- **Broken and unused code**: `inferARG`, `iter_shared`, and the unreachable line in `getM`
- **ARG output**: `nodes.dat` names nodes that are not in the written liberal trees
- **Documentation**: the `OptArgs` and `--n-mcmc-it` defaults
- **Tests**: some tests depend on dictionary iteration order and fail on newer Julia versions
