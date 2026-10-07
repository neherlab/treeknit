# Published algorithms are not named in the code

Several functions implement published algorithms without naming them, so a reader cannot look up their correctness conditions or faster versions. The list of algorithms, sources, and the options for where to record them are in [`algorithm-names.md`](../proposals/algorithm-names.md). `kb/algo/`, which `kb/README.md` reserves for algorithm documentation, does not exist yet.

## Locations

Functions whose doc comments do not name the algorithm:

- `pub fn naive_mccs()` [packages/treeknit-core/src/naive.rs#L24-L56](../../packages/treeknit-core/src/naive.rs#L24-L56): subtree reduction
- `pub fn infer_pair()` [packages/treeknit-core/src/pair.rs#L34-L83](../../packages/treeknit-core/src/pair.rs#L34-L83): prune-and-grow of agreement-forest algorithms, with annealing
- `fn branch_likelihood()` [packages/treeknit-core/src/splitgraph.rs#L537-L545](../../packages/treeknit-core/src/splitgraph.rs#L537-L545): Poisson deviance
- `pub fn resolve_trees()` [packages/treeknit-core/src/resolve.rs#L72-L157](../../packages/treeknit-core/src/resolve.rs#L72-L157): refinement by the loose consensus
- `pub fn match_topologies()` [packages/treeknit-core/src/pipeline.rs#L285-L327](../../packages/treeknit-core/src/pipeline.rs#L285-L327): common refinement within MCCs
- `packages/treeknit-core/src/impute.rs`: tree completion
- `pub fn arg_from_trees()` [packages/treeknit-core/src/arg.rs#L141-L170](../../packages/treeknit-core/src/arg.rs#L141-L170): network of an agreement forest
- `fn mix()` [packages/treeknit-core/src/pipeline.rs#L488-L494](../../packages/treeknit-core/src/pipeline.rs#L488-L494): SplitMix64 finalizer
- `pub fn schedule()` [packages/treeknit-core/src/anneal.rs#L21-L50](../../packages/treeknit-core/src/anneal.rs#L21-L50): geometric cooling without a convergence guarantee

> [!IMPORTANT]
> **Decision required.** Record the names in doc comments only, in pages of `kb/algo/` only, or in both: a one-line name and source link in the doc comment and the background and differences in `kb/algo/`.

## Fix direction

- Add the name and the source to each doc comment above
- Depending on the decision, add `kb/algo/` pages by algorithm family: MCC inference and agreement forests, resolution, annealing and tie-break, output order

## Validation

- Each function above names its algorithm and links a source
