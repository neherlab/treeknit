# Naive MCCs by canonical subtree identifiers

Naive MCCs are the maximal subtrees that are identical in all trees. The port finds them with a climb from each leaf and a depth-first check with a memo, ported from TreeKnit.jl with the memo added. The memo causes overlapping MCCs and a panic on trees with unary nodes. This proposal replaces the climb with canonical identifiers of subtrees, which decide identity by construction and need no climb.

## Problem

- **Algorithm**: `pub fn naive_mccs()` and `fn is_coherent()` [packages/treeknit-core/src/naive.rs#L24-L117](../../packages/treeknit-core/src/naive.rs#L24-L117). From each unvisited leaf, the climb moves to the parent while the parent clades are equal in all trees and `is_coherent` confirms that the subtrees below are identical. `is_coherent` walks the matched children depth-first with an explicit stack and caches positive answers in a memo keyed by the node of the first tree only
- **Origin**: TreeKnit.jl `naive_mccs` and `is_coherent_clade` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_base.jl#L13-L149)] use the same climb without a memo
- **Defect**: the memo returns `true` for a node of the first tree that was matched earlier with a different node of another tree, before the child counts are compared. With a unary node in one tree, MCCs overlap and inference panics ([`H-naive-mccs-overlap-with-unary-nodes.md`](../issues/H-naive-mccs-overlap-with-unary-nodes.md))

## Background

The step is the subtree reduction of the kernels for agreement forests: "Replace a maximal pendant subtree with at least two leaves that occurs identically in S and T by a single leaf with a new label", which keeps the hybridization number of the two trees <a id="cite-1"></a>[Bordewich et al. 2007](https://doi.org/10.1177/117693430700300017) [[1](#ref-1)]. The rule goes back to the kernelization of the tree bisection and reconnection distance <a id="cite-2"></a>[Allen and Steel 2001](https://doi.org/10.1007/s00026-001-8006-8) [[2](#ref-2)]. TreeKnit applies it before every annealing and after every pruning <a id="cite-3"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[3](#ref-3)].

Rooted trees with labeled leaves are identical exactly when their <a id="gloss-use-1"></a>canonical forms <sup>[1](#gloss-1)</sup> are equal. A canonical form can be computed bottom-up by <a id="gloss-use-2"></a>hash-consing <sup>[2](#gloss-2)</sup>: a leaf gets the identifier of its taxon, and an internal node gets the identifier that an interning table assigns to the sorted list of its children's identifiers. Equal identifiers mean identical subtrees, without probability of collision, because the table compares the lists themselves.

## Proposed algorithm

- **Identifiers**: one interning table shared by all trees. In postorder, a leaf gets the identifier of its taxon; an internal node gets the identifier of the sorted list of its children's identifiers. A unary node gets the identifier of a one-element list, which differs from its child's
- **Occurrence**: leaf names are unique, so an identifier determines its leaf set, and it occurs at most once per tree. A node of the first tree is in a naive MCC when its identifier occurs in every tree
- **Maximality**: a naive MCC is a node whose identifier occurs in every tree while its parent's identifier does not, or the root
- **Order**: `sort_mccs` orders the result as today, independent of the traversal
- **Cost**: $O(n \log \delta)$ per tree for sorting children, where $n$ is the number of nodes and $\delta$ the largest number of children; no recursion
- **Example** (the trees of the issue): `(((A,B),(C,D)),E)` and `((((A,B)),(C,D)),E)`. `(A,B)` and `(C,D)` get the same identifiers in both trees; the unary node above `(A,B)` in the second tree gets a new identifier, so the parents differ and the MCCs are `[E]`, `[A, B]`, `[C, D]`, which TreeKnit.jl 0.5.8 also returns

## Design axes

### Algorithm

- **Canonical identifiers (recommended)**: fixes the defect by construction and handles any number of trees in one pass
- **Day's cluster table with a child-count test**: linear time, but a unary node shares its cluster with its child, so the matching of nodes with equal clusters needs extra rules
- **Keep the climb and key the memo by the tuple of matched nodes**: the smallest change; keeps the cost of comparing bit sets on every climb step

### Unary nodes in the input

- **Keep them, as TreeKnit.jl does (recommended)**: the identifiers treat a unary node as a node, which reproduces TreeKnit.jl
- **Splice them out when reading**: changes the topology that TreeKnit.jl compares, so it needs a recorded decision

## Work items

- [`H-naive-mccs-overlap-with-unary-nodes.md`](../issues/H-naive-mccs-overlap-with-unary-nodes.md): the defect and its validation, with this algorithm as the fix direction

## Glossary

1. <a id="gloss-1"></a> **Canonical form.** A representation of a tree that is the same for all trees isomorphic to it, so isomorphism becomes equality of representations. [↩](#gloss-use-1)
2. <a id="gloss-2"></a> **Hash-consing.** Assigning each distinct structure one identifier through a table, so equal structures share an identifier and can be compared in constant time. [↩](#gloss-use-2)

## References

1. <a id="ref-1"></a> Bordewich, Magnus, Simone Linz, Katherine St. John, and Charles Semple. 2007. "A reduction algorithm for computing the hybridization number of two trees." _Evolutionary Bioinformatics_ 3:117693430700300017. https://doi.org/10.1177/117693430700300017 [↩](#cite-1)
2. <a id="ref-2"></a> Allen, Benjamin L., and Mike Steel. 2001. "Subtree transfer operations and their induced metrics on evolutionary trees." _Annals of Combinatorics_ 5:1-15. https://doi.org/10.1007/s00026-001-8006-8 [↩](#cite-2)
3. <a id="ref-3"></a> Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2022. "TreeKnit: Inferring ancestral reassortment graphs of influenza viruses." _PLOS Computational Biology_ 18:e1010394. https://doi.org/10.1371/journal.pcbi.1010394 [↩](#cite-3)
