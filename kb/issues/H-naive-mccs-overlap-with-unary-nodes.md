# Naive MCCs overlap when an input tree has a unary node

`fn naive_mccs` returns MCCs that share leaves when one tree has an internal node with a single child and the other tree has the same clade without that node. Inference then panics; `--naive` writes the overlapping MCCs. TreeKnit.jl `naive_mccs` returns disjoint MCCs for the same input.

## Reproduction

```
t0.nwk: (((A,B),(C,D)),E);
t1.nwk: ((((A,B)),(C,D)),E);
```

- `treeknit t0.nwk t1.nwk` panics: "index out of bounds: the len is 1 but the index is 18446744073709551615" at [splitgraph.rs#L258](../../packages/treeknit-core/src/splitgraph.rs#L258)
- `treeknit t0.nwk t1.nwk --naive` writes the MCCs `[A, B]` and `[A, B, C, D, E]`
- TreeKnit.jl 0.5.8 `naive_mccs(t0, t1)` gives `[E]`, `[A, B]`, `[C, D]`

## Cause

`fn is_coherent` ([naive.rs#L58-L111](../../packages/treeknit-core/src/naive.rs#L58-L111)) caches coherent subtrees in a memo indexed by the tree-0 node only. The climb from A stores the tree-0 node of `(A,B)` as coherent, matched with the binary `(A,B)` node of tree 1. The climb from C later matches the same tree-0 node with the unary node above `(A,B)` in tree 1, which has the same clade. The memo returns `true` before the child counts are compared (one child against two), so the climb continues to the root and the MCC of C contains A and B.

TreeKnit.jl `is_coherent_clade` (`src/mcc_base.jl`) has no memo and rejects the pair on the different child counts.

`fn reduce_to_mccs` then maps the leaves of the inner MCC to no node of the reduced tree, and `EnergyState::climb` reads index `usize::MAX`.

The result depends on leaf order: with the leaves named so that the climb from the third clade comes first, the memo is not consulted and the output is correct.

## Scope

Neither the Newick reader ([newick.rs](../../packages/treeknit-io/src/newick.rs)) nor the pipeline removes unary nodes from input trees, so any input with a unary node above a clade that the other tree also has can trigger the defect. Trees written by tree builders usually have no unary nodes; the real datasets in `data/` have none.

## Fix direction

- Find naive MCCs with canonical subtree identifiers, which treat a unary node as a node and need no memo ([`naive-mccs-by-subtree-hashing.md`](../proposals/naive-mccs-by-subtree-hashing.md))
- Key the memo by the matched node of every tree, not by the tree-0 node alone, or drop the memo and measure the cost on large trees
- Alternatively, splice unary nodes out of input trees when reading them. That changes the topology TreeKnit.jl compares (it keeps unary nodes), so it needs a recorded decision

## Validation

- A unit test in `naive.rs` with the trees above expects `[E]`, `[A, B]`, `[C, D]` (oracle: TreeKnit.jl 0.5.8 `naive_mccs`)
- The same test with the leaf names permuted gives the same partition
- `treeknit t0.nwk t1.nwk` completes
