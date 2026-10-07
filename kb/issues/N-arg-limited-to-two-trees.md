# No ARG for more than two trees

With three or more segment trees, the port writes the MCCs of every pair and no ARG, as TreeKnit.jl does. The ARG code supports two segments only, and the pairwise MCCs that an ARG of all segments needs contradict each other on simulated and real data. [`multi-segment-arg.md`](../proposals/multi-segment-arg.md) gives the analysis.

## Locations

- `pub struct ArgNode`, `pub struct Arg` [packages/treeknit-core/src/arg.rs#L30-L51](../../packages/treeknit-core/src/arg.rs#L30-L51): parents, branch lengths, roots, and trees for two segments
- `pub fn arg_from_trees()` [packages/treeknit-core/src/arg.rs#L140-L170](../../packages/treeknit-core/src/arg.rs#L140-L170): builds the ARG from one pair of trees
- `pub fn extended_newick()` [packages/treeknit-io/src/arg.rs#L14-L30](../../packages/treeknit-io/src/arg.rs#L14-L30): writes `GlobalRoot[&segments={0,1}]`
- `pub struct ArgNodeView` [packages/treeknit-io/src/display.rs#L445-L474](../../packages/treeknit-io/src/display.rs#L445-L474): the node type that the web app receives through WebAssembly
- `pub(super) fn draw()` [packages/treeknit-io/src/figure/arg.rs#L13](../../packages/treeknit-io/src/figure/arg.rs#L13): the SVG figure for two segment names
- `packages/web/src/arg/`: the ARG view of the web app

> [!IMPORTANT]
> **Decision required.** Three choices are open. The proposal gives the evidence for each.
>
> - **Use of the ARG**, which decides what a correct result is:
>   - counts of events with the segments they separate
>   - a figure
>   - shared branches for joint estimation
> - **Conflicts between pairs**:
>   - report them and build no ARG
>   - merge MCCs
>   - split MCCs to the meet of the other pairs
>   - infer all pairs with one joint score
> - **Events at one node with three or more parents**:
>   - one node with more than two parents
>   - a chain of two-parent nodes in a fixed order

> [!IMPORTANT]
> **Investigation required.** It is not known whether pairwise MCCs that satisfy topological compatibility and the triplet condition can always be joined into an acyclic ARG. Join the shared nodes of the true pairwise MCCs of simulated ARGs with three or more segments, and count the classes that contain two nodes of one tree and the cycles.

## Fix direction

- Join the shared nodes of all pairs with a union-find over the nodes of all trees. Report classes with two nodes of one tree and cycles as conflicts, and build the ARG when there are none
- Store parents and branch lengths per segment in vectors of length $K$ in `Arg` and `ArgNodeView`, and run `just gen`
- Write the segments of the global root and of each branch from the ARG
- Derive the pairwise MCCs of the output from the ARG, so that `MCCs.json` and the ARG agree

## Validation

- Each embedded segment tree of the ARG equals its resolved input tree, as `fn one_reassortment()` [packages/treeknit-core/src/arg.rs#L549-L572](../../packages/treeknit-core/src/arg.rs#L549-L572) checks for two trees
- The pairwise MCCs derived from the ARG equal the reported MCCs
- On simulated fixtures with a stored true ARG ([`N-fixtures-lack-true-arg.md`](N-fixtures-lack-true-arg.md)), inferred and true reassortments are compared with the metric chosen in the proposal
