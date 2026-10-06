# Consistency of MCCs across tree pairs

With three or more segment trees, TreeKnit infers the MCCs of each pair of trees separately. The MCCs of different pairs can contradict each other, and the port neither prevents nor reports this. TreeKnit.jl once added a penalty for such contradictions to the annealing and removed it again. This proposal describes the condition, the history, and the options.

## The condition

If two leaves share an MCC in the pair of trees 1 and 2 and also in the pair of trees 1 and 3, then segments 2 and 3 both share their history with segment 1 for these two leaves, so the leaves should share an MCC in the pair of trees 2 and 3. As partitions of the leaves present in all three trees:

$$P_{12} \wedge P_{13} \le P_{23}$$

where:

- $P_{ij}$ -- the partition of the leaves into the MCCs of trees $i$ and $j$
- $\wedge$ -- the meet of two partitions: two leaves are in one block exactly when they are in one block of both partitions
- $\le$ -- "refines": every block of the left side lies inside one block of the right side

The same condition holds for every permutation of the three trees. The TreeKnit.jl documentation of its multi-tree mode states that it "may still return MCCs that are inconsistent with each other, which prevents the construction of an ARG" [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/multitreeknit.md?plain=1#L7)], and explains the transitivity of MCCs with an example [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/multitreeknit.md?plain=1#L101)].

## History in TreeKnit.jl

- **Sampled triplet check**: `consistent_mcc_triplets` [[src](https://github.com/PierreBarrat/TreeKnit.jl/commit/56acdf0ac0346fd9cbcc644b185978390e3a831f)] sampled triplets of MCCs from the pairs (1, 2), (1, 3), (2, 3) and returned the fraction that satisfied the condition. It moved to the evaluation package TestRecombTools in 2021 [[src](https://github.com/PierreBarrat/TreeKnit.jl/commit/02553bf14db0992819cb4216bfbbe14f08103bd1)]
- **Penalty in the annealing**: on the multi-tree development line, `MCC_join_constraint` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/2ae2565447c1ed5bdcfcf48c88d63b5056f43804/src/MultiTreeKnit/multitree_constraints.jl#L6-L42)] computed the meet of the MCC partitions of the other pairs by grouping leaves on their tuple of MCC indices, `get_consistency_mask` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/2ae2565447c1ed5bdcfcf48c88d63b5056f43804/src/SplitGraph/SplitGraph.jl#L137-L155)] marked the effective leaves inside shared regions, and `compute_F` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/2ae2565447c1ed5bdcfcf48c88d63b5056f43804/src/SplitGraph/energy.jl#L180-L184)] added a cost for each marked leaf that the annealing removed
- **Removal**: commit `698836f` (2023-01-09) removed the penalty with the message "remove consistent constraint from SA as shown to have little effect on results" [[src](https://github.com/PierreBarrat/TreeKnit.jl/commit/698836f24f596276b7f392550e167c5b5dc952ea)]. TreeKnit.jl 0.5.8 has neither the penalty nor a check

## Current state of the port

The pipeline infers each pair on its own (`pub fn run_observed()` in `packages/treeknit-core/src/pipeline.rs`). `Resolution::Matched` makes the trees agree within every MCC of every pair, which is a condition on topologies, not on the partitions above. No code compares the partitions of different pairs.

## Check algorithm

For each triple of trees, restrict the three partitions to the leaves present in all three trees. Label every leaf with the pair (block in $P_{12}$, block in $P_{13}$); group the leaves by this label with a hash map; a group whose leaves fall into more than one block of $P_{23}$ violates the condition. Repeat for the other two orientations. The cost is linear in the number of leaves per triple, for $\binom{K}{3}$ triples of $K$ trees. The number of leaf pairs that violate the condition, or of groups that do, measures the inconsistency.

## Design axes

- **Report as a diagnostic (recommended)**: a warning in the log and a count in the run summary (`packages/treeknit-io/src/summary.rs`), without changing inference. Users of more than two segments learn where pairs disagree
- **Port the penalty**: changes inference results; the TreeKnit.jl authors found little effect
- **Repair after inference**: split the MCCs of each pair to the meet of the other pairs. Removes inconsistencies by adding reassortments, which changes results and the meaning of the output
- **Nothing**

## Work items

- [`N-cross-pair-mcc-consistency-unreported.md`](../issues/N-cross-pair-mcc-consistency-unreported.md)
