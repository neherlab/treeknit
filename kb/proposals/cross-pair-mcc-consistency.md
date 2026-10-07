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

## Measured inconsistency

The counts below apply the check algorithm to the output of the port at commit `348a086`. A violating group is a group of leaves that two pairs place in one MCC each and the third pair splits. A violating leaf pair is a pair of leaves in such a group that the third pair separates.

### Simulated fixtures

- **Data**: the four three-tree fixtures `fixtures/sim/sim_k3_*`, simulated with ARGTools under the Kingman coalescent with 50 leaves (see [`ref/README.md`](../../ref/README.md)). The `_poly` cases are separate simulations whose short branches were collapsed into polytomies
- **Runs**: `treeknit tree1.nwk tree2.nwk tree3.nwk --resolve <mode> [--pre-resolve] --seed <s>`, with seeds 1 to 5
- **Values**: violating leaf pairs, summed over the three orientations, each of which has 1225 leaf pairs. A single value is the same for all five seeds, and a range covers the five seeds

| Case                    | `matched` | `matched --pre-resolve` | `strict`  | `strict --pre-resolve` | `liberal`  | `none`  | `none --pre-resolve` |
| ----------------------- | --------- | ----------------------- | --------- | ---------------------- | ---------- | ------- | -------------------- |
| `sim_k3_n50_r0.05`      | 380       | 380                     | 380       | 380                    | 380        | 380     | 380                  |
| `sim_k3_n50_r0.05_poly` | 0         | 0                       | 0         | 0                      | 0          | 9 to 13 | 158                  |
| `sim_k3_n50_r0.1`       | 0 to 3    | 0 to 3                  | 0         | 0                      | 0          | 0 to 3  | 0 to 3               |
| `sim_k3_n50_r0.1_poly`  | 54        | 54                      | 60 to 120 | 120                    | 120 to 260 | 5 to 8  | 97                   |

- **True MCCs**: 0 in all four cases, as one ARG requires
- **TreeKnit.jl**: its default for three trees is `--resolve none --pre-resolve` in the port. The five seeded runs stored in the fixtures (`multi_runs`) give 380, 158, 0 to 3, and 97, the same values as the port
- **Cause**: the fully resolved case `sim_k3_n50_r0.05` gives 380 in every setting and with every seed, so neither the resolution nor the randomness of the annealing causes the conflict. Compared with the true MCCs:
  - the pair (tree1, tree2) has one MCC fewer
  - the pair (tree1, tree3) has as many MCCs, but a different partition
  - the pair (tree2, tree3) is equal
- **Finer partitions**: `none` without pre-resolution gives few violations in the `_poly` cases, but 25 to 37 MCCs per pair with seed 1, against 3 to 17 with the other settings. Finer partitions satisfy the condition more easily, so a low count alone does not show good MCCs

### Four-segment H3N2 data

`data/h3n2-2k-4-segments` has four segment trees with 1997 leaves. Each run used seed 1. The table gives the violating groups per orientation, with the first, second, and third tree of the triplet as the tree shared by the two pairs.

| Triplet      | `matched`  | `strict --pre-resolve` |
| ------------ | ---------- | ---------------------- |
| ha, na, pb1  | 44, 28, 19 | 9, 12, 14              |
| ha, na, pb2  | 46, 20, 22 | 13, 19, 9              |
| ha, pb1, pb2 | 45, 14, 30 | 13, 19, 12             |
| na, pb1, pb2 | 41, 30, 39 | 10, 13, 13             |

- **Leaf pairs**: 343 to 2898 violating leaf pairs per orientation with `matched`, and 50 to 366 with `strict --pre-resolve`, out of 1993006 leaf pairs
- **MCCs per pair**: 606 to 823 with `matched`, and 994 to 1067 with `strict --pre-resolve`

## Design axes

- **Report as a diagnostic (recommended)**: a warning in the log and a count in the run summary (`packages/treeknit-io/src/summary.rs`), without changing inference. Users of more than two segments learn where pairs disagree
- **Port the penalty**: changes inference results; the TreeKnit.jl authors found little effect
- **Repair after inference**: split the MCCs of each pair to the meet of the other pairs. Removes inconsistencies by adding reassortments, which changes results and the meaning of the output. Repeated over all pairs, the splits move every pair toward one common partition, in which every reassortment separates all segments. Merging MCCs where the other pairs join leaves is the alternative, and it needs a topology check. [`multi-segment-arg.md`](multi-segment-arg.md) describes both
- **Nothing**

## Work items

- [`N-cross-pair-mcc-consistency-unreported.md`](../issues/N-cross-pair-mcc-consistency-unreported.md)

A consistent set of pairwise MCCs is a precondition of one ARG for all segments, see [`multi-segment-arg.md`](multi-segment-arg.md).
