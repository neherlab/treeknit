# Ancestral reassortment graph: parity checklist

Counterpart: [`v0/arg.md`](v0/arg.md). `treeknit_core::arg` builds the ARG of two trees [[src](../../packages/treeknit-core/src/arg.rs#L140-L170)], and `treeknit_io::arg` writes it [[src](../../packages/treeknit-io/src/arg.rs#L19-L62)].

- [x] **Two segments only**: as in TreeKnit.jl. The command line and the web app build the ARG only for two trees

## Data model

- [x] **`ArgNode`**: one parent per segment, as `Absent`, `Root`, or `Node(index)`, the branch length per segment, the children, the label, and the hybrid flag [[src](../../packages/treeknit-core/src/arg.rs#L20-L40)]. A leaf flag is new. The segments of a node follow from its parents, so there is no color field
- [x] **`Arg`**: a vector of nodes, the root of each segment, the tree node labels of each ARG node, and the two trees the ARG was built from [[src](../../packages/treeknit-core/src/arg.rs#L42-L51)]. TreeKnit.jl keeps dictionaries keyed by label
- [/] **Node queries**: `has`, `is_shared`, `is_root`, `Arg::parents`, `Arg::edge_segments` [[src](../../packages/treeknit-core/src/arg.rs#L53-L88)]. There is no counterpart of `is_global_root`, `is_partial_root`, `degree`, or `share_color`
- [ ] **`check(n)` and `show`**: no node check and no display beyond the derived `Debug`

## Construction

`fn arg_from_trees` follows the steps of TreeKnit.jl:

- [x] **Root length check**: the warning "root of an input tree has a branch length; ARG branch lengths may be off"
- [x] **Copies**: the inputs do not change
- [x] **Liberal resolution** with the MCCs
- [x] **Node status**: MCC root, shared, shared singleton, or not shared, with the partner node in the other tree [[src](../../packages/treeknit-core/src/arg.rs#L172-L275)]
- [x] **Shared singletons**: inserted into the other tree as `Singleton_<n>`, from the tree with the shorter branch below, or from tree 1 when a length is missing [[src](../../packages/treeknit-core/src/arg.rs#L277-L369)]. The names use a counter; TreeKnit.jl uses random characters. This differs on purpose ([README](../../README.md#deliberate-differences-from-treeknitjl))
- [x] **ARG from tree 1**: a hybrid node above each MCC root whose partner is not the root of tree 2 [[src](../../packages/treeknit-core/src/arg.rs#L403-L429)]
- [x] **Tree 2 added**: shared nodes get a second parent, MCC roots connect through their hybrid node, other nodes are new [[src](../../packages/treeknit-core/src/arg.rs#L431-L487)]
- [x] **Node labels**: `ARGNode_<n>` from a counter; TreeKnit.jl uses random characters. This differs on purpose
- [/] **Label maps**: `Arg::tree_nodes` gives the labels in tree 1 and tree 2 of each ARG node, with none for hybrid nodes, like `rlm`. The tree-to-ARG maps `lm1` and `lm2` stay internal
- [x] **Hybrid count**: `Arg::n_hybrids` is the number of reassortments. Fixture comparison: `n_hybrids` of every two-tree case
- [x] **Errors**: the function returns `ArgError` for "inconsistent shared nodes" and for five other inconsistencies. The command line logs the error and continues without an ARG; the web app shows the message
- [x] **Input trees**: the command line and the web app build the ARG from the resolved output trees, not from the input trees. This differs on purpose ([README](../../README.md#deliberate-differences-from-treeknitjl)). With leaves missing from one tree, the inputs are the imputed trees (see [`partial-overlap.md`](partial-overlap.md))

## Branch lengths

`fn set_branch_lengths` [[src](../../packages/treeknit-core/src/arg.rs#L490-L542)]:

- [x] **Node in one tree only**: the length from that tree
- [x] **Fully shared node**: the mean of the two lengths, with the TreeKnit.jl rules for a missing length at a root and elsewhere
- [x] **MCC root below a hybrid**: $\tau/2$ for both segments with $\tau$ the shorter length, and the remainder $\tau_k - \tau/2$ on the hybrid branch of segment $k$
- [ ] **Machine epsilon**: TreeKnit.jl adds `eps()` to every length it sets, so a zero length appears as `2.220446049250313e-16`. The port adds nothing ([`N-undocumented-differences-from-treeknit-jl.md`](../issues/N-undocumented-differences-from-treeknit-jl.md))
- No test compares the branch lengths with TreeKnit.jl ([`N-reference-comparison-gaps.md`](../issues/N-reference-comparison-gaps.md))

## Extended Newick output

`fn extended_newick` [[src](../../packages/treeknit-io/src/arg.rs#L19-L44)] writes the extended Newick structure with BEAST annotations of `util-newick` (`NewickDialect::ENEWICK_BEAST`):

- [x] **Start node**: the shared root, an extra `GlobalRoot[&segments={0,1}]:0.0` above two unshared roots, or the unshared root when one root is shared. TreeKnit.jl writes `:0.` for the extra root
- [x] **Hybrid nodes**: written in full when first reached and as a reference afterwards, with `label#H<i>`. `i` counts the nodes in the order in which their first copies end in a depth-first walk, as in TreeKnit.jl. Each copy carries the annotation and length of its parent's branch
- [x] **Segment annotation**: `[&segments={0,1}]`, `{0}`, or `{1}` on each branch, 0-based
- [x] **Lengths**: omitted when missing
- [x] **Labels (new)**: quoted where Newick needs it, so `B,1` is written `'B,1'` and the file stays readable. TreeKnit.jl writes labels without quotes, which breaks the file for such labels
- [x] **Deep ARGs (new)**: written without recursion
- [x] **Viewers**: the syntax equals the TreeKnit.jl output, which IcyTree reads

## Trees from an ARG

- [x] **Segment tree**: `Arg::segment_tree(c)` extracts the tree of one segment, with hybrid nodes as internal nodes with one child [[src](../../packages/treeknit-core/src/arg.rs#L94-L109)]. Fixture comparison: the splits of both segment trees after removal of those nodes, on every two-tree case including the simulated ARGs `test_srg_*`

## Functions that fail in TreeKnit.jl

- [x] **`inferARG`**: fails on every call in TreeKnit.jl. The port has no single function; `run`, `arg_inputs`, and `arg_from_trees` together build the ARG (see [`library-api.md`](library-api.md))
- Not ported: the graph-editing helpers `set_ancestor!`, `unset_child!`, and `prune!`, which fail in TreeKnit.jl and have no caller
