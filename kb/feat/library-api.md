# Library interface: parity checklist

Counterpart: [`v0/julia-api.md`](v0/julia-api.md). The Rust crates `treeknit-core` (algorithms) and `treeknit-io` (formats) form the library [[src](../../packages/treeknit-core/src/lib.rs#L1-L27)]. The workspace sets `publish = false`, so the crates are not on crates.io. Trees are `treeknit_core::Tree` values with a shared taxon table `Taxa`; `treeknit_io::newick::parse` reads them, and `Tree::assign_taxa` links their leaves to the table.

## Exported names of TreeKnit.jl

- [x] **`run_treeknit!`**: `treeknit_core::run(&mut trees, &taxa, &options, seed)` for any number of trees [[src](../../packages/treeknit-core/src/pipeline.rs#L177-L181)]
- [/] **`run_treeknit`**: no form that works on copies. A caller clones the trees first
- [x] **`OptArgs`**: `Options`, `Options::for_trees`, and `Options::treeknit_jl` (see [`pipeline.md`](pipeline.md#run-options))
- [/] **`MCC_set`**: `Vec<PairResult>` in pair order. Each `PairResult` has the tree indices `i` and `j`, the MCCs as taxon numbers, and the attached leaves. `PairResult::mcc_of(leaf)` finds the MCC of a leaf, and `PairResult::shared_mccs` gives the MCCs on the shared leaves only [[src](../../packages/treeknit-core/src/pipeline.rs#L28-L78)]. There is no access by tree label, no `add!`, no `iter_pairs`, and no `print`
- [x] **`naive_mccs`**: `treeknit_core::naive_mccs(&trees, n_taxa)` for any number of trees
- [/] **`resolve!`**: `resolve_trees` (topology only), `resolve_with_mccs` (strict or liberal), and `insert_split` for one split and `insert_all_on` for a list of splits. No `tau` parameter, no conflict mode, no dictionary form (see [`resolution.md`](resolution.md))
- [x] **`map_mccs`**: `mcc_map::map_mccs(tree, &leaf_mcc_map(mccs, n))` gives the MCC of every node [[src](../../packages/treeknit-core/src/mcc_map.rs#L14-L65)]
- [ ] **`map_mccs!`**: nodes have no data field to store the result in
- [/] **`write_mccs`, `read_mccs`**: `mccs::to_lines` writes the format; `read_mccs` has no port. New: `mccs::to_json` for `MCCs.json`
- [x] **`SRG`**: `treeknit_core::arg` and `treeknit_io::arg` (see [`arg.md`](arg.md))
- [/] **`inferARG`**: no single function. `run`, then `arg_inputs` and `arg::arg_from_trees`, build the ARG of two trees [[src](../../packages/treeknit-core/src/pipeline.rs#L639-L655)]. The TreeKnit.jl function fails on every call
- [x] **`parse_newick_string`**: `treeknit_io::newick::parse`

## Internal functions used in the TreeKnit.jl documentation

- [/] **`runopt`**: `pair::infer_pair`, without `output = :all`
- [x] **`run_standard_treeknit!`, `run_parallel_treeknit!`**: both inside `run`, selected by the round and `Options::parallel`
- [/] **`sort_polytomies!`**: `mcc_map::sort_polytomies_by_mccs` is public and does the non-strict sort of one tree along another. `sort_for_pair` sorts two trees for display with the run's sort of a pair, strictly or not, and always ladderizes the left tree (see [`pipeline.md`](pipeline.md#display-sort-new))
- [x] **`write_auspice_json`, `get_auspice_json`**: `auspice::auspice_json` returns the content for one tree
- [x] **`name_mcc_clades!`, `reduce_to_mcc`, `pruneconf!`**: `pair::reduce_to_mccs` and `pair::prune_mccs`
- [x] **`new_splits`, `splits_in_mcc`, `splits_in_mccs`, `map_splits_to_tree!`**: private functions behind `resolve_with_mccs`
- [x] **`SplitGraph.trees2graph`, `compute_energy`, `count_mismatches`, `conf_likelihood`**: `splitgraph::Graph::new`, `Graph::energy`, and `Graph::likelihood`
- [x] **`SplitGraph.sa_opt`**: `anneal::optimize`
- [/] **`SplitGraph.opttrees`, `compute_F`**: private in `pair` and `anneal`

## Other MCC queries

- [x] **`find_mcc_with_node`**: `PairResult::mcc_of`
- [ ] **`is_branch_in_mcc`, `is_branch_in_mccs`, `find_mcc_with_branch`, `is_linked_pair`, `get_leaves_order`, `true_splits`**: not ported. No part of the TreeKnit.jl pipeline calls them

## Logging and reproducibility

- [x] **Logging**: the `log` crate with the levels info, debug, and trace in place of the Julia levels 0, -1, and -2. The caller installs a logger
- [x] **Progress (new)**: `run_observed(&mut trees, &taxa, &options, seed, &observe)` calls `observe` with a `Progress` value (phase, completed fraction, round, pair) during the run (see [`pipeline.md`](pipeline.md#progress-new))
- [x] **Reproducibility**: the `seed` argument of `run` (see [`pipeline.md`](pipeline.md#reproducibility))

## New in the library

- [x] **Trees and taxa**: arena trees with traversals, restriction to a leaf set, pruning, and ladderizing; clades as bit sets (`fixedbitset`) over one taxon table [[src](../../packages/treeknit-core/src/tree.rs#L1-L463)]
- [x] **`match_topologies`, `unmatched_mccs`**: the `matched` resolution and its check
- [x] **`imputed_trees`, `arg_inputs`**: trees with missing leaves placed, and the inputs of the ARG (see [`partial-overlap.md`](partial-overlap.md))
- [x] **`sort_for_pair`, `sort_strictness`, `last_sorting_pair`, `keeps_run_order`**: the order of a pair of trees for display, from the run's sort of a pair (see [`pipeline.md`](pipeline.md#display-sort-new))
