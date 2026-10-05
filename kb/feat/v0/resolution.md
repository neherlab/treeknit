# Polytomy resolution

Trees built from short segments have polytomies (see [`overview.md`](overview.md) for the terms). A polytomy in one tree and a resolved clade in the other tree make the topologies differ for a reason other than reassortment. TreeKnit adds splits from one tree to another to remove such differences [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/resolving.md?plain=1#L1-L20)]. There are three kinds of resolution:

- **With topology only**: add a split of one tree to another tree when the other tree can take it. This runs as pre-resolution and inside `runopt`
- **During inference**: the energy of the annealing treats a polytomy as compatible with a resolved clade when a resolution exists (see [`mcc-inference.md`](mcc-inference.md)). The trees stay as they are in this step
- **With MCCs**: after inference, add the splits that the MCCs imply, in strict or liberal mode

## Common properties

- **New nodes**: each added split becomes a new internal node with the label `RESOLVED_<i>` and branch length `tau`, default `0.0`. The number `<i>` continues after the largest number in labels that contain `RESOLVED` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/resolving.jl#L13-L41)]
- **Placement**: the new node takes the children of the LCA that hold the leaves of the split, and becomes a child of that LCA. The children keep their branch lengths
- **Conflicts**: `resolve!(t, S; conflict = :fail)` raises "Tried to resolve tree with an incompatible split." for an incompatible split. With `conflict = :ignore`, it skips the split. A split already in the tree is skipped
- **Dictionary form**: `resolve(trees::Dict, splits::Dict)` resolves copies of trees indexed by a key, for example the segment name, with the split lists of the same key [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/resolving.jl#L50-L56)]

## Resolution with topology only

`resolve!(t1, t2, tn...; tau = 0.)` resolves all given trees with each other and returns the new splits of each tree [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/resolving.jl#L161-L169)]:

- **Rule**: a split $s$ of tree $k$ goes into the other trees only if every other tree either has $s$ already, or can get $s$ by grouping some children of the LCA of the leaves of $s$. The union of the children inside $s$ must be exactly $s$ [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/resolving.jl#L64-L114)]
- **Iteration**: the procedure loops over all trees as the source until no split is added, at most 20 passes. At 20 passes it warns "Maximum number of iterations reached" [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/resolving.jl#L116-L135)]
- **More than two trees**: one tree that contradicts a split blocks it for all trees. The documentation shows `(B,C)` added to two trees and then blocked by a fourth tree that has `(A,B)` [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/resolving.md?plain=1#L50-L72)]
- **Limit**: a reassorted leaf inside a clade makes the clade incompatible, so this rule cannot use the clade. For `((A,B),(C,(D,(E,X))))` and `((A,(B,X)),(C,D,E))`, no split goes into the second tree [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/resolving.md?plain=1#L74-L113)]

## Resolution with MCCs

`resolve!(t1, t2, MCCs; tau = 0., strict = true)` adds to each tree the splits that the other tree has inside the MCCs [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/resolving.jl#L177-L183)]. It returns the new splits of each tree. Inside an MCC the branches are shared, so the two trees must have the same splits there, and leaves of other MCCs can be ignored. In the example above, the MCC `[A,B,C,D,E]` lets TreeKnit ignore `X` and add `(D,E)` to the second tree [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/resolving.md?plain=1#L115-L129)].

Steps of `function new_splits` for a target tree `tref` and a source tree `t` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_splits.jl#L62-L95)]:

1. For each MCC, take the splits of `t` under the MCC root, restricted to the leaves of the MCC. The split of the MCC root itself is included when it is not the root of `t` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_splits.jl#L4-L21)]
2. Map each split onto `tref`: find the children of the LCA in `tref` that hold the split's leaves, and take the union of their full clades. The result can contain leaves of other MCCs [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_splits.jl#L156-L204)]
3. Drop empty splits and splits that `tref` already has, then drop duplicates

### Strict and liberal modes

When a reassortment is attached at a polytomy, the place of the reassorted branch inside a new split can be unknown. For `t1 = ((A,(B,C)),D)`, `t2 = (A,B,C,D)`, and the MCCs `[A,B,C]` and `[D]`, the tree `t2` could become `((A,(B,C)),D)`, `(A,(B,(C,D)))`, or `(A,((B,C),D))`. All three agree with the MCCs and with `t1` [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/resolving.md?plain=1#L131-L196)].

- **Liberal** (`strict = false`): add every mapped split. In the example, the split `(A,B,C)` goes into `t2`, which gives `((A,(B,C)),D)`. This can add wrong splits
- **Strict** (`strict = true`, the default) [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_splits.jl#L163-L194)]:
  - Before a split is mapped, assign every node of `tref` to an MCC with the Fitch method of `map_mccs` (see [`julia-api.md`](julia-api.md)), and find the MCC of the split from the LCA children that hold its leaves
  - Call the other children of the LCA "sisters". The split is skipped when a sister has no MCC or an MCC different from the MCC of the LCA, and also contains no leaf of the split's MCC. The code also tests whether a sister is the root, which cannot happen for a child of the LCA
  - In the example, `t2` stays `(A,B,C,D)`
  - The published paper has only the liberal behavior, and no publication evaluates strict mode (see [`treeknit-paper-vs-code.md`](../../reports/treeknit-paper-vs-code.md))
- **Use of the modes**:
  - The pipeline resolves with `oa.strict` after each pair. In a last round with `final_no_resolve`, it does not resolve, and `strict = false` only selects the non-strict polytomy sort
  - ARG construction always resolves liberally, because the two trees must be resolved in the same way inside each MCC
  - The strict sorting of polytomies resolves copies liberally to find a leaf order (see [`visualization.md`](visualization.md))
- **Failure**: the MCC resolution calls `resolve!` with `conflict = :fail`, so a mapped split that is incompatible with the target tree stops the run with an error
