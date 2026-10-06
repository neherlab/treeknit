# Resolution and ARG construction are cubic on deep trees

On deep trees with large MCCs, resolution and ARG construction take time that grows about as the cube of the number of leaves, while MCC inference stays fast. Two identical caterpillar trees of 2000 leaves need 5 ms for inference and 48.9 s for the ARG; with the default `--resolve matched` the run takes 238 s, and at 5000 leaves pair inference shows no progress after 600 s. The design options are in [`tree-query-indices.md`](../proposals/tree-query-indices.md).

## Reproduction

```sh
n=2000
awk -v n=$n 'BEGIN { s = ""; for (i = 0; i < n - 1; i++) s = s "(L" i ","; s = s "L" n - 1; for (i = 0; i < n - 1; i++) s = s ")"; print s ";" }' > a.nwk
cp a.nwk b.nwk
treeknit a.nwk b.nwk --resolve none -v -o out     # ARG step: about 49 s
treeknit a.nwk b.nwk -o out-matched               # default resolution: about 238 s
```

Measured on 2026-10-06 with the `prod` build in the project container on a shared machine:

| Leaves | `--resolve none` |                                            `--resolve matched` |
| -----: | ---------------: | -------------------------------------------------------------: |
|  1,000 |            5.1 s |                                                         23.2 s |
|  2,000 |           51.9 s |                                                        237.9 s |
|  5,000 |     not measured | no progress after 600 s; the last log line is "inferring MCCs" |

The real pair `data/h3n2-2k-4-segments/ha.nwk` and `na.nwk` (1997 leaves, 301 MCCs) takes 1.5 s for resolution and matching and 0.14 s for the ARG.

## Cause

The cause comes from the code and the log timestamps; no profile confirms it yet.

- `pub fn Tree.lca()` [packages/treeknit-core/src/tree.rs#L235-L251](../../packages/treeknit-core/src/tree.rs#L235-L251) walks both nodes to the root to compute their depths, then climbs; `pub fn Tree.lca_of()` [packages/treeknit-core/src/tree.rs#L253-L255](../../packages/treeknit-core/src/tree.rs#L253-L255) repeats this for every node of a set
- ARG construction calls `resolve_with_mccs` in liberal mode, whose `fn mcc_splits()` [packages/treeknit-core/src/resolve.rs#L161-L201](../../packages/treeknit-core/src/resolve.rs#L161-L201) calls `fn blca()` [packages/treeknit-core/src/resolve.rs#L24-L42](../../packages/treeknit-core/src/resolve.rs#L24-L42) once per internal node of every MCC. `blca` runs `lca_of` over the leaves of the split and climbs from every leaf. With one MCC of $n$ leaves in a caterpillar of depth about $n$, this gives about $n$ calls of $O(n^2)$ each
- `pub fn resolve_trees()` [packages/treeknit-core/src/resolve.rs#L72-L157](../../packages/treeknit-core/src/resolve.rs#L72-L157) calls `lca_of` once per split of every tree; it is the first step of pair inference when the run resolves
- `fn mcc_splits` compares every candidate split with every split of the tree (`ref_splits.iter().any(...)`), $O(n^2/64)$ word operations per MCC node

> [!IMPORTANT]
> **Investigation required.** Confirm the attribution with a profile of the 2000-leaf ARG step (`just build profiling`, then `perf record`). The profile should show `Tree::lca`, `Tree::depth`, and `blca` taking most of the time.

## Fix direction

- Build a depth-first leaf numbering and a depth array once per tree version, and compute the LCA of a leaf set as the LCA of its first and last leaf in that numbering
- Test clade membership with an interval index instead of comparing bit sets against every split
- Rebuild the indices after each batch of inserted splits instead of recomputing whole-tree data per split; the per-split recomputation in `insert_split` is [`M-matched-topologies-slow-on-large-trees.md`](M-matched-topologies-slow-on-large-trees.md)
- Keep the order of inserted splits, so node labels and output files stay byte-identical

## Validation

- The fixture comparison with TreeKnit.jl and the existing tests pass unchanged
- The output files of the 2000-leaf caterpillar pair and of `data/h3n2-2k-4-segments` (`ha`, `na`) are byte-identical before and after the change
- The ARG step of the 2000-leaf caterpillar pair takes well under one second; the run with `--resolve matched` at 5000 leaves completes in seconds
