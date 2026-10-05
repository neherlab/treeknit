# Trees with different leaf sets (new)

TreeKnit.jl requires all trees to have the same leaves and stops with "Trees must share leaves". The port accepts trees with different leaf sets: it infers the MCCs of each pair on the leaves both trees share, then places the other leaves. `packages/treeknit-core/src/impute.rs` describes the method [[src](../../packages/treeknit-core/src/impute.rs#L1-L8)]. No publication describes or validates it.

## Inference on shared leaves

- [x] **Taxon table**: the union of the leaves of all trees. `fn report_overlap` logs, for each tree, how many leaves it lacks, on the command line and in the web app [[src](../../packages/treeknit-io/src/run.rs#L113-L126)]
- [x] **Restriction**: each pair is restricted to its shared leaves. Unary nodes are removed and their branch lengths added [[src](../../packages/treeknit-core/src/pipeline.rs#L412-L423)]. A unit test checks that a pair with one extra leaf gives the same MCCs as the pair without it
- [x] **Pre-resolution**: splits are compared on the leaves each two trees share (see [`resolution.md`](resolution.md#resolution-with-topology-only))
- [x] **Resolution with MCCs**: on the restricted pair, then inserted into the full trees with the shared leaves as mask, so subtrees of other leaves do not move
- [x] **Polytomy sorting**: on the restricted pair, then applied to the full trees (see [`visualization.md`](visualization.md))
- [x] **Pairs with fewer than two shared leaves**: the shared validation rejects the request on the command line and in the web app. The core called directly skips the pair with a warning and gives it no MCCs ([`kb/decisions/pairs-with-fewer-than-two-shared-leaves.md`](../decisions/pairs-with-fewer-than-two-shared-leaves.md))

## Attachment of missing leaves

`fn attach_private` [[src](../../packages/treeknit-core/src/impute.rs#L37-L65)]:

- [x] **Unit**: each largest subtree of one tree whose leaves the other tree lacks, so a cherry of two missing leaves moves as one unit
- [x] **MCC of the attachment point**: the MCC that the Fitch map gives to the parent of the subtree, with the missing leaves as wildcards
- [x] **Ambiguous attachment**: when the parent has no MCC, the MCC below it with the fewest edges to its root, then the shortest branch-length distance, then the larger MCC, then the lower index. The attachment is marked ambiguous [[src](../../packages/treeknit-core/src/impute.rs#L67-L94)]
- [x] **MCC extension**: the leaves join their MCC, and the MCC list is sorted again [[src](../../packages/treeknit-core/src/pipeline.rs#L570-L591)]
- [x] **Report**: the `imputed` list of `MCCs.json` gives each leaf, its source tree, its MCC, and the ambiguous flag (see [`formats.md`](formats.md#mccsjson))

## Imputed trees

`fn imputed_trees` [[src](../../packages/treeknit-core/src/pipeline.rs#L593-L637)]:

- [x] **Choice of the pair**: for each tree and each missing leaf, the pair whose MCC with that leaf is largest, ties to the lower tree index
- [x] **Grafting**: a copy of the source subtree goes below the LCA of the shared leaves of its MCC under the attachment point. With one such leaf, a new node above that leaf takes both [[src](../../packages/treeknit-core/src/impute.rs#L96-L133)]. New nodes are named `IMPUTED_<n>`
- [x] **Whole or split subtrees**: a subtree is grafted whole when all its leaves chose it, otherwise leaf by leaf
- [x] **Output**: `<label>_imputed<ext>` with `--impute`. The run result always holds these trees, and the web app lists their files

## ARG with missing leaves

- [/] **Inputs**: the imputed trees, restricted to the shared leaves and the leaves with an unambiguous attachment [[src](../../packages/treeknit-core/src/pipeline.rs#L639-L655)]. When the imputed trees place leaves differently inside one MCC, the construction fails and no ARG is written ([`M-arg-fails-after-imputation.md`](../issues/M-arg-fails-after-imputation.md))

## Evaluation and tests

- [x] **Accuracy experiment**: `examples/accuracy.rs <fraction>` drops that fraction of leaves (default 0.2) from each tree of the simulated cases. It counts a placed leaf as correct when its true MCC is the true MCC of most of the leaves it was placed next to, and compares the rate with placement in the largest MCC [[src](../../packages/treeknit-io/examples/accuracy.rs#L137-L230)]
- [x] **Tests**: unit tests in `impute.rs` and `pipeline.rs`, the command-line test `three_trees_partial_overlap_imputed` [[src](../../packages/treeknit-cli/tests/cli.rs#L789)], and the run-result tests `run_imputes_a_leaf_missing_from_one_tree` and `output_files_place_a_leaf_missing_from_one_tree` in `treeknit-io`
