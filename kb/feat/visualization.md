# Tree ordering and visualization outputs: parity checklist

Counterpart: [`v0/visualization.md`](v0/visualization.md). The command line draws nothing, as in TreeKnit.jl. The web app draws nothing either (see [`web-app.md`](web-app.md)).

## Ordering of the output trees

`fn sort_pair` runs in the last round for each pair [[src](../../packages/treeknit-core/src/pipeline.rs#L454-L504)]:

- [x] **When**: in the last round, ladderize tree 0 in its pairs, then sort the polytomies of the pair. With more than two trees, the sort for the last pair of a tree gives its final order
- [x] **Ladderize**: children sorted by leaf count, smallest first, then by label [[src](../../packages/treeknit-core/src/tree.rs#L427-L444)]. Unit test `ladderize_sorts_by_size_then_name`
- [x] **Non-strict sort**: the first tree guides; children of the second tree in the same MCC as their parent follow the leaf order of the first tree, other children follow clade size [[src](../../packages/treeknit-core/src/mcc_map.rs#L94-L122)]. Fixture comparison: `sorted_leaf_order`. The sort is an insertion sort, because the comparison is not a total order and the standard sorts may panic on such a comparison
- [x] **Strict sort**: resolve copies liberally, ladderize the copy of the first tree, sort the copy of the second tree, then order the children of both trees by the smallest leaf position in the copies. No test compares it with TreeKnit.jl ([`N-reference-comparison-gaps.md`](../issues/N-reference-comparison-gaps.md))
- [x] **Node-to-MCC map**: the two-pass Fitch method of `map_mccs` [[src](../../packages/treeknit-core/src/mcc_map.rs#L25-L65)]. Fixture comparison: `fitch`. A leaf in no MCC, for example a leaf that the other tree lacks, is a wildcard and does not constrain its ancestors (new)
- [x] **Sort after matching (new)**: with `matched` resolution, every pair is sorted non-strictly after the matching
- [x] **Different leaf sets (new)**: the pair is sorted on copies restricted to the shared leaves, and the leaf order is applied to the full tree. Leaves outside the pair go last [[src](../../packages/treeknit-core/src/mcc_map.rs#L124-L138)]

## Auspice JSON

`fn auspice_json` returns the file content for one tree [[src](../../packages/treeknit-io/src/auspice.rs#L17-L59)]:

- [x] **Files**: `auspice_<label>.json` per tree with `--auspice-view`. The library function returns a JSON value, so there is no directory argument with a trailing `/`
- [x] **Colorings**: one per other tree, key and title `mcc_<a>_<b>` with the labels in order, type `ordinal` [[src](../../packages/treeknit-io/src/auspice.rs#L12-L16)]
- [x] **Node values**: the 1-based MCC index as a string, from the Fitch map, or `"null"`
- [x] **Divergence**: `div` from the root, a missing length counted as 0. The tree itself does not change; TreeKnit.jl sets missing lengths to `0.0` in the tree
- [x] **`meta.updated`**: the empty string, as in TreeKnit.jl. It does not match the date pattern of the augur schema, and Auspice still reads the file
- [x] **Tanglegram on auspice.us**: the same files as TreeKnit.jl, so the same procedure applies. No test loads them in Auspice

## ARG viewers

- [x] **IcyTree**: `ARG/arg.nwk` has the TreeKnit.jl syntax (see [`arg.md`](arg.md#extended-newick-output))
- [ ] **Drawing of trees, tanglegrams, or ARGs**: TreeKnit.jl links a separate prototype viewer, TreeKnit-web, that draws ARGs. The web app of this port shows tables and downloads only
