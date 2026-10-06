# Validating MCCs and ARGs as agreement forests

TreeKnit's MCCs have the defining properties of an agreement forest of the two resolved trees, and the ARG that the port builds from them is the network of that forest. Agreement-forest theory gives two checks that the port does not run: the forest must be acyclic for the ARG to have a time order, and the number of reassortments can never be lower than exact distances between the trees. This proposal states the relation, the argument for acyclicity, and the options for checking both.

## Background

- **Agreement forest**: a partition of the leaves of two trees into blocks whose restricted subtrees are identical in both trees and whose spanning subtrees are vertex-disjoint in each tree. A maximum agreement forest has the fewest blocks; their number minus one is the rooted subtree-prune-and-regraft (rSPR) distance <a id="cite-1a"></a>[Whidden et al. 2013](https://doi.org/10.1137/110845045) [[1](#ref-1)]
- **Cycle graph and acyclicity**: Whidden et al. define a graph on the blocks with an edge from block $C_i$ to block $C_j$ when the lowest common ancestor of $C_i$ is an ancestor of the lowest common ancestor of $C_j$ in either tree. A forest is acyclic when this graph has no directed cycle <a id="cite-1b"></a>[Whidden et al. 2013](https://doi.org/10.1137/110845045) [[1](#ref-1)]. The smallest acyclic agreement forest has as many blocks as the <a id="gloss-use-1"></a>hybridization number <sup>[1](#gloss-1)</sup> plus one, a theorem that Whidden et al. and <a id="cite-2"></a>[Bordewich et al. 2007](https://doi.org/10.1177/117693430700300017) [[2](#ref-2)] attribute to <a id="cite-3"></a>[Baroni et al. 2005](https://doi.org/10.1007/s00285-005-0315-9) [[3](#ref-3)]. Since every acyclic agreement forest is an agreement forest, the rSPR distance is at most the hybridization number
- **TreeKnit**: removes effective leaves, which are maximal subtrees identical in both trees, prunes them, and repeats until no leaf is removed <a id="cite-4"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[4](#ref-4)]. The paper does not mention agreement forests. The exact algorithm of Whidden et al. has the same "prune maximal agreeing subtrees" and "grow agreeing subtrees" steps and differs in how it chooses the cuts

## MCCs form an acyclic agreement forest

The argument below is derived from the definitions; no published source states it.

- **Each MCC is a common pendant subtree when it is removed**: a removed effective leaf is a naive MCC of the remaining trees, a clade with the same topology in both. The MCCs left at the end are naive MCCs of the remaining trees too
- **Agreement**: the subtree of each MCC is identical in both trees by the step above
- **Vertex-disjointness**: when MCC $A$ is removed, its clade in the remaining trees holds only $A$ and leaves removed earlier, so the spanning subtree of a later MCC does not pass through it
- **Acyclicity**: let $A$ be removed before $B$. If the LCA of $A$ were an ancestor of the LCA of $B$ in a tree, the leaves of $B$ would lie in the clade of $A$ in the remaining tree, and $A$ would not be a clade there. So every edge of the cycle graph points from a later MCC to an earlier one, and MCCs removed in the same round are disjoint clades without an edge between them. Refining a tree later adds clades and removes none, so the argument holds for the resolved output trees
- **Consequence**: the number of reassortments in the ARG, the number of MCCs minus one, is at least the hybridization number of the resolved trees and therefore at least their rSPR distance. For multifurcating trees the argument uses the strict forest, whose blocks have identical restricted topologies; definitions for multifurcating trees that allow refinement give smaller or equal numbers, so the bound still holds

The argument covers MCCs as inference returns them. Three later steps change MCCs or trees and are not covered: topology matching splits MCCs into the naive MCCs of their restrictions (`pub fn match_topologies()`, [packages/treeknit-core/src/pipeline.rs#L285-L327](../../packages/treeknit-core/src/pipeline.rs#L285-L327)), imputation adds leaves to MCCs (`pub fn attach_private()`, [packages/treeknit-core/src/impute.rs#L38-L65](../../packages/treeknit-core/src/impute.rs#L38-L65)), and ARG construction resolves the trees with the MCCs in liberal mode.

## Current state

- **No acyclicity check**: `pub fn arg_from_trees()` [packages/treeknit-core/src/arg.rs#L141-L170](../../packages/treeknit-core/src/arg.rs#L141-L170) builds one hybrid node above every MCC root that is not a root in the other tree and checks shared nodes, but no code in the core or io crates tests the graph for cycles; TreeKnit.jl has no such check either
- **Silent effect in the drawing**: the ARG layout orders nodes with Kahn's algorithm (`fn topological_order()`, [packages/treeknit-io/src/display/arg_view.rs#L141-L160](../../packages/treeknit-io/src/display/arg_view.rs#L141-L160)). A node on a cycle never enters the order, and the layout [packages/treeknit-io/src/display/arg_view.rs#L22-L29](../../packages/treeknit-io/src/display/arg_view.rs#L22-L29) does not compare the length of the order with the number of nodes; only a unit test does, for its own inputs
- **No comparison with exact solvers**: the tests compare the port with TreeKnit.jl and with simulated truth, not with exact distances

## Design axes

### Acyclicity check

- **Error in `arg_from_trees` (recommended)**: run Kahn's algorithm on the finished ARG and return an `ArgError` when it orders fewer nodes than the ARG has. The cost is linear. The run then writes no ARG, as for other ARG errors
- **Debug assertion**: no cost in release builds, but a cyclic ARG in a release build stays silent
- **Tests only**: a property test over random tree pairs, without a runtime check

A property test is useful with any of the three: random pairs from `examples/large_tree_pair.rs` with every resolution mode, checking that every ARG is acyclic.

### Exact lower bound in tests

- **Brute force over partitions for small trees (recommended)**: for trees of up to about 8 leaves, enumerate all partitions of the leaves (4140 for 8 leaves), keep the acyclic agreement forests, and take the smallest. A test then checks that the number of MCCs is at least that size. No external tool
- **`rspr` from the reference scripts**: `rspr` [[src](https://github.com/cwhidden/rspr/tree/4530e11bac642ec8f1389f837314628208950605)] computes exact rSPR distances of rooted binary trees ("The second tree may be multifurcating") and is licensed GPL-3.0-or-later. Its documentation lists no hybridization-number mode, so it gives the weaker bound "reassortments at least rSPR distance". As an external command in `ref/`, run in a container, it is not linked into the project
- **No exact comparison**

## Work items

- [`N-arg-acyclicity-unchecked.md`](../issues/N-arg-acyclicity-unchecked.md)
- [`N-mcc-counts-not-compared-with-exact-solutions.md`](../issues/N-mcc-counts-not-compared-with-exact-solutions.md)

## Glossary

1. <a id="gloss-1"></a> **Hybridization number.** The smallest number of reticulation events in a network that displays both trees; for segment trees, the smallest number of reassortments that explain both topologies. [↩](#gloss-use-1)

## References

1. <a id="ref-1"></a> Whidden, Chris, Robert G. Beiko, and Norbert Zeh. 2013. "Fixed-parameter algorithms for maximum agreement forests." _SIAM Journal on Computing_ 42:1431-1466. https://doi.org/10.1137/110845045. Preprint: https://arxiv.org/abs/1108.2664 [↩¹](#cite-1a) [↩²](#cite-1b)
2. <a id="ref-2"></a> Bordewich, Magnus, Simone Linz, Katherine St. John, and Charles Semple. 2007. "A reduction algorithm for computing the hybridization number of two trees." _Evolutionary Bioinformatics_ 3:117693430700300017. https://doi.org/10.1177/117693430700300017 [↩](#cite-2)
3. <a id="ref-3"></a> Baroni, Mihaela, Stefan Grünewald, Vincent Moulton, and Charles Semple. 2005. "Bounding the number of hybridisation events for a consistent evolutionary history." _Journal of Mathematical Biology_ 51:171-182. https://doi.org/10.1007/s00285-005-0315-9 [↩](#cite-3)
4. <a id="ref-4"></a> Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2022. "TreeKnit: Inferring ancestral reassortment graphs of influenza viruses." _PLOS Computational Biology_ 18:e1010394. https://doi.org/10.1371/journal.pcbi.1010394 [↩](#cite-4)
