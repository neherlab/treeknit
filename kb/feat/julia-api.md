# Julia library interface

TreeKnit.jl is also a Julia library. The module exports the names below [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/TreeKnit.jl#L1-L53)]. The documentation uses some internal functions with the `TreeKnit.` prefix; this page lists them too. The trees are TreeTools `Tree` objects, which users read with `read_tree` or `parse_newick_string` from TreeTools.

## Exported names

- **`run_treeknit!`, `run_treeknit`**: run the pipeline on a list of trees or on two trees (see [`pipeline.md`](pipeline.md))
- **`OptArgs`**: run parameters, with the keyword constructor and `OptArgs(K; method, ...)` (see [`pipeline.md`](pipeline.md))
- **`MCC_set`**: the result type (see below)
- **`naive_mccs(trees...)`**, **`naive_mccs(treelist)`**: naive MCCs (see [`mcc-inference.md`](mcc-inference.md))
- **`resolve!`**: all resolution forms (see [`resolution.md`](resolution.md))
- **`map_mccs`, `map_mccs!`**: assign nodes to MCCs (see below)
- **`write_mccs`, `read_mccs`**: MCC files (see [`formats.md`](formats.md))
- **`SRG`**: the ARG submodule (see [`arg.md`](arg.md))
- **`inferARG`**: exported, but every call fails (see [`arg.md`](arg.md))
- **`parse_newick_string`**: re-exported from TreeTools for convenience

## Internal functions used in the documentation

- **`TreeKnit.runopt(oa, t1, t2; output)`**: MCCs of one pair
- **`TreeKnit.run_standard_treeknit!`**, **`TreeKnit.run_parallel_treeknit!`**: the pair loops
- **`TreeKnit.sort_polytomies!(t1, t2, MCCs; strict)`**: tanglegram order (see [`visualization.md`](visualization.md))
- **`TreeKnit.write_auspice_json`**, **`TreeKnit.get_auspice_json`**: auspice output
- **`TreeKnit.name_mcc_clades!`**, **`TreeKnit.reduce_to_mcc`**, **`TreeKnit.reduce_to_mcc!`**, **`TreeKnit.pruneconf!`**: the coarse-graining and pruning steps of `runopt`
- **`TreeKnit.new_splits`**, **`TreeKnit.splits_in_mcc`**, **`TreeKnit.splits_in_mccs`**, **`TreeKnit.map_splits_to_tree!`**: split computations behind the MCC resolution
- **`TreeKnit.SplitGraph`**: `trees2graph`, `compute_energy`, `compute_F`, `count_mismatches`, `sa_opt`, `opttrees`, `conf_likelihood`, and the types `Graph`, `SplitNode`, `LeafNode`

## `MCC_set`

`struct MCC_set` stores the MCCs of all pairs of a run [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/objects.jl#L130-L227)]:

- **Fields**: `no_trees`, `order_trees` (tree labels in input order), and `mccs`, a dictionary from the set of the two tree labels to the MCC list
- **Construction**: `MCC_set(K, labels)` is empty. `MCC_set(K, labels, list)` takes a list of MCC lists in pair order `(1,2), (1,3), ...` and warns when the labels are not unique, because pairs are keyed by label
- **Access**: `get(M, "a", "b")`, `get(M, ("a", "b"))`, `get(M, 1, 2)`, `get(M, (1, 2))`, and `M["a", "b"]`. The order of the two trees does not matter. A missing pair gives `nothing`
- **Update**: `TreeKnit.add!(M, mccs, "a", "b")` or with tree positions
- **Iteration**: `TreeKnit.iter_pairs(M)` returns the label pairs and the MCC lists in pair order
- **Other**: `print(M)` and `copy(M)` (deep copy)
- **Broken**: `TreeKnit.iter_shared(M, label)` reads the field `M.tree_order`, which does not exist, and fails

## Assignment of nodes to MCCs

`map_mccs(tree, MCCs; internals = true)` returns a dictionary from node label to the index of its MCC, or `nothing` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_tools.jl#L1-L131)]. `map_mccs(MCCs)` and `internals = false` map the leaves only. Internal nodes get an MCC through a two-pass Fitch method:

- **Upward pass**: a leaf gets the set with its MCC. An internal node gets the intersection of its children's sets, or their union when the intersection is empty. The root gets the intersection only
- **Downward pass**: the root gets its MCC if its set has one element, else `nothing`. Another node gets the only element of its set. If its set has several elements, it gets the MCC of its parent when that MCC is in the set, and otherwise `nothing`
- **`map_mccs!`**: stores the result in the node data as `"mcc"` and `"child_mccs"`. It needs trees with `TreeTools.MiscData` node data. For other trees, it logs an error message and throws "Incorrect method"

The tests include a node that could belong to two MCCs and gets `nothing` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/test/mcc_tools/test.jl#L64-L94)].

## Other MCC queries

`src/mcc_tools.jl` and `src/mcc_splits.jl` contain helpers that no part of the pipeline calls [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_tools.jl#L266-L383)]:

- **`is_branch_in_mcc(n, mcc)`**, **`is_branch_in_mccs(n, mccs)`**: whether the branch above node `n` is inside an MCC, which is true when the clade of `n` contains some, but not all, leaves of the MCC
- **`find_mcc_with_branch(n, mccs)`**: the MCC that contains the branch above `n`, as `(index or key, mcc)`, or `nothing`
- **`is_linked_pair(n1, n2, mccs)`**: whether two leaves are in the same MCC
- **`find_mcc_with_node(n, mccs)`**: the MCC that contains a leaf
- **`get_leaves_order(tree, MCCs)`**: the position of each leaf within its MCC in tree order
- **`true_splits(S, Sref)`**: indices of splits of `S` that are also in `Sref`, a helper for comparisons with simulated trees [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_splits.jl#L206-L219)]

## Logging from Julia

TreeKnit writes its messages with `@logmsg LogLevel(0)`, `LogLevel(-1)`, and `LogLevel(-2)`. The default Julia logger shows none of them. To see the detail of command-line verbosity `v`, run the call inside `with_logger(ConsoleLogger(LogLevel(-v)))` [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/options.md?plain=1#L67-L95)].

## Reproducibility

TreeKnit uses the global Julia random generator and never seeds it. A Julia user can call `Random.seed!` before `run_treeknit!` to repeat a run.
