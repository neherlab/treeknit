# Polytomy resolution: parity checklist

Counterpart: [`v0/resolution.md`](v0/resolution.md). `packages/treeknit-core/src/resolve.rs` holds the resolution with topology only and with MCCs. `packages/treeknit-core/src/pipeline.rs` holds the new `matched` mode.

## Common properties

- [x] **New nodes**: each added split becomes a node `RESOLVED_<i>` with branch length 0. The number continues after the largest `RESOLVED_<n>` in the tree [[src](../../packages/treeknit-core/src/tree.rs#L361-L373)]. TreeKnit.jl continues after the largest number in any label that contains `RESOLVED`
- [/] **Branch length of new nodes**: always 0. TreeKnit.jl has the parameter `tau`
- [x] **Placement**: the new node takes the children of the LCA that hold the leaves of the split and becomes a child of that LCA [[src](../../packages/treeknit-core/src/resolve.rs#L44-L65)]
- [x] **Split already present**: skipped
- [/] **Conflicts**: `insert_split` returns `Insert::Incompatible` for an incompatible split, and every caller skips the split with a warning [[src](../../packages/treeknit-core/src/resolve.rs#L279-L305)]. There is no mode that stops with an error. TreeKnit.jl stops the MCC resolution with an error ([`N-undocumented-differences-from-treeknit-jl.md`](../issues/N-undocumented-differences-from-treeknit-jl.md))
- [ ] **Dictionary form**: TreeKnit.jl `resolve(trees::Dict, splits::Dict)` resolves trees keyed by segment name. The port has no such form
- [x] **Mask (new)**: `insert_split` considers only the leaves in a mask. A split judged on the leaves that two trees share can go into a tree with more leaves, and subtrees outside the mask stay where they are

## Resolution with topology only

`fn resolve_trees` [[src](../../packages/treeknit-core/src/resolve.rs#L67-L157)]:

- [x] **Rule**: a split goes into the other trees only when every other tree has it or can take it as a union of children of the LCA. Fixture comparison: `pre_resolve` of every case
- [x] **Iteration**: passes until no split is added, at most 20, with the warning "resolve_trees: maximum number of iterations reached"
- [x] **More than two trees**: one tree that contradicts a split blocks it for all trees. Unit test `four_trees_incompatible` and the fixture `test_resolve_3_incompatible`
- [x] **Different leaf sets (new)**: splits are compared on the leaves that each two trees share

## Resolution during inference

- [x] **Energy with resolution**: see [`mcc-inference.md`](mcc-inference.md#energy-and-score)

## Resolution with MCCs

`fn resolve_with_mccs` and `fn mcc_splits` [[src](../../packages/treeknit-core/src/resolve.rs#L159-L254)]:

- [x] **Splits inside MCCs**: for each MCC, the splits of the source tree below the MCC root, restricted to the MCC, mapped to the union of the LCA children in the target tree. Empty, present, and duplicate splits are dropped
- [x] **Liberal mode**: adds every mapped split. Fixture comparison: `mcc_resolve_liberal`
- [x] **Strict mode**: skips a split when a sister in the polytomy has an uncertain place [[src](../../packages/treeknit-core/src/resolve.rs#L203-L245)]. Fixture comparison: `mcc_resolve_strict`. The port also accepts a sister that holds leaves of an MCC that continues above the polytomy, which TreeKnit.jl 0.5.8 rejects. This differs on purpose and matches the branch `fix/issues-from-rust-port` of TreeKnit.jl ([README](../../README.md#deliberate-differences-from-treeknitjl))
- [x] **Use of the modes**: resolving rounds use strict mode unless the resolution is `Liberal`; the extra final round does not resolve; the ARG construction resolves liberally; the strict polytomy sort resolves copies liberally
- [/] **Incompatible mapped split**: skipped with a warning. TreeKnit.jl stops the run with an error (see Conflicts above)
- [x] **Shared leaves only (new)**: `resolve_pair` resolves the restricted pair and inserts the new splits into the full trees with the shared leaves as mask [[src](../../packages/treeknit-core/src/pipeline.rs#L487-L507)]

## Matched resolution (new)

`fn match_topologies` runs after the last round when the resolution is `Matched` [[src](../../packages/treeknit-core/src/pipeline.rs#L264-L318)]:

- [x] **Propagation**: for each pair, the splits that either tree has inside an MCC of three or more leaves, restricted to the MCC, go into the other tree [[src](../../packages/treeknit-core/src/pipeline.rs#L320-L352)]
- [x] **Passes**: pairs in order, repeated until no split is added, at most 20 passes, so splits pass along chains of shared regions
- [x] **Precedence**: a split goes in only when it is compatible with the tree, so splits of earlier trees win conflicts. No split is removed
- [x] **Conflicting MCCs**: an MCC whose two trees still differ inside it is replaced by the naive MCCs of the two restricted trees, and the log reports it
- [x] **Check**: `unmatched_mccs` lists the MCCs whose trees differ inside the MCC [[src](../../packages/treeknit-core/src/pipeline.rs#L354-L368)]. The test `matched_topologies_on_three_segments` checks that the list is empty and that no input split is lost
- [x] **No extra round**: `Matched` needs no final round without resolution
- [/] **Pass limit**: after 20 passes the propagation stops without a warning. `resolve_trees` warns in the same case
- [/] **Large trees**: on a pair of 10,000-leaf trees, matching takes about 3 minutes, probably because `insert_split` recomputes the clades of the whole tree for every split ([`M-matched-topologies-slow-on-large-trees.md`](../issues/M-matched-topologies-slow-on-large-trees.md))
