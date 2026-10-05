# Matched resolution is slow on large trees

`Matched` is the default resolution. With it, `fn match_topologies` runs after the last round [[src](../../packages/treeknit-core/src/pipeline.rs#L231-L273)]. On a pair of 10,000-leaf trees this step takes about 183 s of a 185 s run, although it adds no split. MCC inference on the same pair takes about 1 s.

## Measurements

Input: `just example large_tree_pair <leaves> tmp/large-pair` (seed 1, 10 subtree moves), then `just run release tree_a.nwk tree_b.nwk`. Times come from the timestamps of the log lines that end MCC inference and matching. Every run found 11 MCCs, and the matching added no split and split no MCC.

| Leaves | MCC inference | Matching | Whole run |
| -----: | ------------: | -------: | --------: |
|  2,500 |         0.2 s |    2.8 s |    3.05 s |
|  5,000 |         0.5 s |   15.6 s |   16.09 s |
| 10,000 |         1.1 s |  183.5 s |  184.61 s |

Each time the number of leaves doubles, the matching time grows by a factor of 5.57 (2,500 to 5,000 leaves) and then 11.76 (5,000 to 10,000 leaves). Cubic growth gives a factor of 8 per doubling, so the hypothesis of cubic growth fits these two ratios, but two ratios cannot establish the exponent.

The measured pairs came from a version of the generator that did not keep node times when it moved a subtree, so the same command now writes a different tree B.

## Cause

The cause comes from reading the code. No profile confirms it yet.

- `fn propagate_splits` calls `insert_split` once for every internal node below the root of each MCC, in both directions of the pair [[src](../../packages/treeknit-core/src/pipeline.rs#L278-L307)]. With one large MCC, this is about one call per internal node of the tree, so O(N) calls for N leaves
- `pub fn insert_split` recomputes `t.clades(n_taxa)` and `t.leaf_of(n_taxa)` for the whole tree on every call, even when the split is already present [[src](../../packages/treeknit-core/src/resolve.rs#L47-L65)]. `clades` allocates one bitset of `n_taxa` bits for every node, so each call costs O(N²/64) word operations [[src](../../packages/treeknit-core/src/tree.rs#L201-L214)]
- Together, O(N) calls of O(N²/64) each give O(N³/64). This hypothesis agrees with the measured ratios, which lie on both sides of the cubic factor 8

> [!IMPORTANT]
> **Investigation required.** Confirm the cause with a profile of the 10,000-leaf run (for example `just build profiling` and `perf record`). The profile should show that `Tree::clades` inside `insert_split` takes most of the matching time.

## Impact

- The CLI spends about 3 minutes on matching at 10,000 leaves, with the default settings. The plan for the web app requires it to stay usable at this size
- The web app runs the same `treeknit_core::run` in WebAssembly in one worker, without threads. A browser run at 10,000 leaves therefore also spends minutes in matching, probably more than the native build. This time is not measured yet
- The matching step reports no progress, so the run seems to stop after MCC inference

## Fix direction

- Compute the clades and the leaf index of the destination tree once per MCC, or once per call of `propagate_splits`, and update them when `insert_split` adds a node. An added node changes only the clade of the new node, which is the union of the clades of its children
- Skip a split that the destination tree already has before calling `insert_split`, by looking up the clade restricted to the MCC in a set of the restricted clades of the destination tree
- Keep the results unchanged: the same splits in the same order, so the resolved trees, the MCCs, and the output files stay byte-identical

## Validation

- The existing tests of `match_topologies` and the fixture comparison with TreeKnit.jl pass unchanged
- The output files of the 10,000-leaf pair of `just example large_tree_pair 10000 tmp/large-pair` are byte-identical before and after the fix
- On that pair, the matching step takes a few seconds or less in the release build
