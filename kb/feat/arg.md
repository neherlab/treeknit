# Ancestral reassortment graph for two trees

The submodule `SimpleReassortmentGraph`, exported as `SRG`, builds an ancestral reassortment graph (ARG) from two trees and their MCCs [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/SimpleReassortmentGraph.jl#L1-L21)]. The ARG holds both segment trees in one graph: a branch belongs to segment 1, to segment 2, or to both, and a hybrid node, which has one parent per segment, marks each reassortment. The trees and the MCCs together define the observable part of the ARG. They do not give the time of a reassortment on its branch, so TreeKnit places it with a fixed rule (see [Branch lengths](#branch-lengths)).

The submodule supports exactly two segments: colors are vectors of length 2, and `othercolor` accepts only 1 and 2 [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/objects.jl#L205-L208)]. The command line builds the ARG only when it gets two trees.

## Data model

- **`ARGNode`** [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/objects.jl#L18-L25)]:
  - `anc`: one parent per segment, `nothing` when the node does not carry the segment, or a `RootNode` marker for the root of that segment
  - `color`: which segments the node carries
  - `tau`: branch length above the node, per segment
  - `children`: all children, of any segment
  - `label`: leaf name, or `ARGNode_<8 random letters and digits>` for internal nodes
  - `hybrid`: true for a reassortment node
- **`ARG`**: dictionaries of the per-segment roots, all nodes, hybrid nodes, and leaves, keyed by label [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/objects.jl#L226-L235)]
- **Node predicates**: `isroot(n)`, `isroot(n, c)`, `is_global_root`, `is_partial_root`, `isleaf`, `ishybrid`, `isshared` (carries both segments), `degree`, `hascolor`, `share_color`, `edgecolor(a, n)` (segments of the edge from `a` to `n`)
- **Checks and display**: `check(n)` asserts that a node without a parent has a `missing` length, that each color has a parent of the same color, and that each leaf carries both segments [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/misc.jl#L9-L32)]. `show` prints the label, the hybrid flag, the colors, the parents, the lengths, and the children

## Construction

`SRG.arg_from_trees(t1, t2, MCCs)` returns `(arg, rlm, lm1, lm2)` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/construct.jl#L6-L32)]:

1. **Root length check**: warn if the root of an input tree has a branch length that is not `missing`
2. **Copies**: the input trees are not changed
3. **Liberal resolution** with the MCCs, so that both trees have the same splits inside each MCC (see [`resolution.md`](resolution.md))
4. **Node status** (`fn shared_nodes()`) [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/construct.jl#L330-L408)]: each node of each tree gets one status:
   - `:mcc_root`: the LCA of an MCC. It records the matching node in the other tree and whether that node is the root there
   - `:shared`: inside an MCC and matched to one node of the other tree
   - `:shared_singleton`: inside an MCC, but its split restricted to the MCC equals the split of its child, so it has no partner in the other tree
   - `:non_shared`: outside all MCCs

   A `:shared` node whose partner does not point back raises "Inconsistent shared nodes"

5. **Shared singletons** (`fn fix_shared_singletons!()`) [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/construct.jl#L442-L539)]: insert into the other tree a node `Singleton_<8 random characters>` for each shared singleton, so that both trees have the same nodes inside each MCC. When both trees have a singleton at the same step, the code inserts the one from the tree with the shorter branch below it, or the one from tree 1 when a length is `missing`. The new node is placed at the same distance above the child as in the source tree, or at the parent when the branch is too short
6. **ARG from tree 1** (`fn arg_from_tree()`, `fn grow_arg_from_treenode!()`) [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/construct.jl#L39-L133)]: copy the tree into ARG nodes of color 1. Above each `:mcc_root` whose partner is not the root of tree 2, insert a hybrid node
7. **Add tree 2** (`fn add_tree_to_arg!()`) [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/construct.jl#L141-L225)]: shared nodes get color 2 and a color-2 parent. MCC roots connect to the existing hybrid node above them. Non-shared nodes become new color-2 nodes
8. **Label maps**: `lm1` and `lm2` map tree node labels to ARG labels. `rlm` maps each ARG label to the pair `(tree 1 label, tree 2 label)`, with `nothing` where the node is absent. Hybrid nodes map to `(nothing, nothing)` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/construct.jl#L230-L249)]. The labels refer to the copies made in step 2, after resolution and singleton insertion
9. **Branch lengths** (see below)

The number of hybrid nodes is the number of reassortments. An MCC that contains the root of one tree gets no hybrid node, which follows the counting rule of the published paper ("except if it contains the root of one of the trees"). The TreeKnit.jl documentation states the rule with "the roots of both trees" instead (see [`mcc-inference.md`](mcc-inference.md)).

### Branch lengths

`fn set_branch_length!(arg, t1, t2, lm)` uses a heuristic for each non-hybrid node. The paper states that the trees and MCCs do not determine the time of a reassortment and gives no rule for it (see [`treeknit-paper-vs-code.md`](../reports/treeknit-paper-vs-code.md)) [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/construct.jl#L263-L318)]:

- **Node in one tree only**: the length from that tree
- **Fully shared node**: the mean of the two lengths. If one length is `missing` and that node is a root, keep `missing` for that segment. If it is `missing` and the node is not a root, use the other length for both segments
- **MCC root with a hybrid above it**: let $\tau$ be the shorter of the two lengths, or the one that is not `missing`. The MCC root gets $\tau / 2$ for both segments, and the hybrid node gets the rest of each tree's length, $\tau_k - \tau/2$
- **Machine epsilon**: every numeric length set through `set_branch_length!` gets `eps()` (about $2.2 \times 10^{-16}$) added [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/objects.jl#L186-L193)]. Zero lengths therefore appear as `2.220446049250313e-16` in the output

## Extended Newick output

`write(filename, arg)` and `write(io, arg)` write the ARG as one extended Newick line [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/IO.jl#L1-L144)]. [`formats.md`](formats.md) describes the syntax. The writer chooses the start node:

- **Same root for both segments**: write from that root
- **Neither root shared**: write both root subtrees under an extra node `GlobalRoot[&segments={0,1}]:0.`
- **One root shared**: write from the root that is not shared

A hybrid node is written in full the first time the traversal reaches it and as a reference (label with `#H<i>`) the second time. The IcyTree viewer displays this format [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/visualization.md?plain=1#L3-L4)].

## Trees from an ARG

`SRG.trees_from_arg(arg)` and `SRG.tree_from_arg(arg, clr)` extract the tree of one segment as a TreeTools tree [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/trees.jl#L1-L28)]. Hybrid nodes become internal singletons. The test suite builds ARGs from 50 simulated pairs and checks that the extracted trees, after removal of singletons, have the same splits as the input trees [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/test/SRG/random_args.jl#L1-L31)]. This test runs only when the ARGTools package is in the test environment.

## Exported but broken

- **`inferARG(t1, t2, oa)`**: refers to an undefined variable `trees`, so every call fails with an `UndefVarError` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L180-L188)]
- **`set_ancestor!(n, a)`**, **`unset_child!`**, **`prune!`**: iterate over a Boolean vector as `(i, c)` pairs, which fails at run time. No construction path calls them [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/objects.jl#L168-L174)]
