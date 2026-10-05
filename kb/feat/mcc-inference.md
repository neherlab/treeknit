# MCC inference for one pair of trees

`fn runopt()` infers the maximally compatible clades (MCCs) of two trees [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L206-L265)]. An MCC is a set of leaves whose subtrees have the same topology in both trees and that cannot be made larger. The procedure repeats four steps until the trees are compatible:

1. Group the leaves into naive MCCs and replace each naive MCC by one leaf
2. Search, with simulated annealing, for the set of these leaves whose removal best reconciles the two trees
3. Record the removed leaves as final MCCs and prune them from both trees
4. Resolve the pruned trees again and start the next iteration

The documentation explains the steps with small examples [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/opttrees.md)] [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/runopt.md)].

## Interpretation of MCCs

- **Shared branches**: inside an MCC, the branches belong to both segment trees. The two trees must be joined ("knitted") along these regions
- **Number of reassortments**: the root of each MCC marks a reassortment, except for an MCC that contains the root of one of the trees, as in the published paper and the ARG construction. The number of reassortments is the number of MCCs, minus one if such an MCC exists. The TreeKnit.jl documentation states the exception for "the roots of both trees" [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/mccs.md?plain=1#L44-L47)]
- **Output order**: each MCC is a sorted list of leaf labels. The list of MCCs is sorted by size, smallest first, then by the first label [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_base.jl#L81-L92)]
- **Partition**: the MCCs of a pair cover every leaf exactly once

## Naive MCCs

`fn naive_mccs()` returns the largest clades that are already identical in all given trees [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_base.jl#L13-L78)]:

- **Precondition**: all trees have the same leaf labels, otherwise the error "Can only be used on trees that share leaf nodes."
- **Algorithm**: from each leaf not yet assigned, move up in all trees at the same time. Continue while the parents define the same split in all trees and all their subtrees match (`fn is_coherent_clade()`). The leaves under the last common node form one naive MCC
- **No resolution**: a polytomy in one tree and a resolved clade in the other count as different
- **Number of trees**: the function accepts any number of trees, but `fn is_coherent_clade()` accepts only a tuple of two split lists [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_base.jl#L104)]. With three or more trees, a shared clade raises a `MethodError`

## Iteration in `runopt`

`runopt(oa, t1, t2; output = :mccs)` works on copies of the trees [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L206-L265)]:

1. If `oa.resolve`, resolve the copies with each other (topology only, see [`resolution.md`](resolution.md))
2. Run `opttrees` with $M = \lceil n \cdot n_{MCMC} / |T| \rceil$ annealing steps per temperature [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L227)], where $n$ is the number of leaves that remain in the trees, $n_{MCMC}$ is `oa.nMCMC`, and $|T|$ is the number of temperatures in `oa.Trange`
3. Apply the stop rules of `fn stop_conditions!()` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L267-L316)]:
   - **Nothing removed**: add the naive MCCs of the remaining trees to the result and stop
   - **Removed MCCs cover all leaves**: stop
   - **Otherwise**: prune the removed MCCs from both trees, remove the singleton nodes that this leaves (branch lengths are added), and resolve again if `oa.resolve`. Then compute the naive MCCs of the remaining trees:
     - **One naive MCC**: the remaining trees are compatible. Add it to the result and stop
     - **Iteration counter above `oa.itmax`**: add the remaining naive MCCs and stop. The counter starts at 1, so at most `itmax + 1` annealing runs take place
     - **Otherwise**: start the next iteration
4. Check the trees with TreeTools `check_tree` after each iteration (assertion)
5. Return the sorted MCCs. With `output = :all`, also return the two pruned trees

Earlier iterations can make later reassortments visible: a reassorted leaf inside a clade hides the compatibility of the clade until it is pruned. The documentation gives a nine-leaf example that needs two iterations [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/runopt.md?plain=1#L12-L30)].

`runopt(t1, t2; kwargs...)` builds `OptArgs(2; kwargs...)`, which ignores `γ` and the annealing parameters (see [`pipeline.md`](pipeline.md)).

## One annealing pass: `opttrees`

`fn SplitGraph.opttrees!()` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/SplitGraph.jl#L36-L72)]:

1. Compute the naive MCCs. If there is only one, return it
2. **Coarse-graining**: rename the root of each naive MCC to `MCC_<i>` in both trees (leaves keep their name; `<i>` continues after the largest `MCC_` number already present), and replace each naive MCC by one leaf (`fn name_mcc_clades!()`, `fn reduce_to_mcc!()`) [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_base.jl#L160-L221)]
3. **Split graph**: build one graph from the reduced trees (`fn trees2graph()`) [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/tools.jl#L10-L40)]. Each leaf is a `LeafNode` with one parent per tree. Each internal node of tree $k$ is a `SplitNode` with color $k$ and the sorted indices of the leaves below it
4. **Annealing**: find the configurations with the lowest score (see below)
5. **Tie break**: choose one configuration with `fn sortconf()` (see below)
6. Return the naive MCCs that the configuration removes, its energy, its score, and the likelihoods

### Energy and score

A configuration $c \in \{0,1\}^N$ marks which of the $N$ coarse-grained leaves stay in the trees. Its energy counts the topological mismatches of the remaining leaves [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/energy.jl#L15-L48)]:

$$E(c) = \sum_{i \,:\, c_i = 1} \;\sum_{k < l} \mathbb{1}\left[ a_k(i, c) \not\sim a_l(i, c) \right]$$

where:

- $i$ -- a remaining leaf
- $k, l$ -- trees of the pair (the code supports $K$ trees and sums over all pairs)
- $a_k(i, c)$ -- the first ancestor of $i$ in tree $k$ that has at least two remaining leaves below it, or the root
- $\sim$ -- equality of the two ancestors' sets of remaining leaves. With `resolve = true`, two different sets also count as equal when one contains the other and every child clade of the larger ancestor is either inside the smaller set or disjoint from it, so that resolving the larger polytomy gives the smaller set [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/energy.jl#L98-L123)]

The score adds a cost for each removed leaf [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/energy.jl#L180-L183)]:

$$F(c) = E(c) + \gamma \left( N - \sum_i c_i \right)$$

where $\gamma$ is `OptArgs.γ`. With $\gamma = 1$, a removal and a fixed mismatch weigh the same, which is the parsimonious choice. A larger $\gamma$ needs a removal to fix more than $\gamma$ mismatches, so fewer reassortments are inferred. As $\gamma \to \infty$, no removal pays off, and the result is the naive MCCs [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/options.md?plain=1#L11-L19)].

`SplitGraph.count_mismatches(g)` returns $E$ with all leaves present, and `count_mismatches(trees...)` builds the graph first [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/energy.jl#L312-L331)].

### Simulated annealing

`fn sa_opt()` and `fn _sa_opt()` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/energy.jl#L187-L304)]:

- **Start**: all leaves present
- **Move**: flip the state of one leaf chosen uniformly at random
- **Acceptance** (Metropolis): accept when $F_{new} < F$ or when $e^{-(F_{new} - F)/T} > u$, with $u$ uniform in $[0, 1)$ and $T$ the current temperature
- **Schedule**: $M$ moves at each temperature of `Trange`, from hot to cold. The chain continues from one temperature to the next
- **Memory of optima**: the search keeps every distinct configuration that reaches the lowest score seen so far, so the result is a set of equally good configurations
- **Repetitions**: `sa_rep` independent runs. The runs with the lowest score contribute their configurations
- **Global flag**: `sa_opt` stores `resolve` in a module-level variable that `compute_energy` and the likelihood read
- **Random generator**: the global Julia generator, never seeded by TreeKnit. A restart option (`reset_chance`) exists in the code with probability 0

### Choice among equal configurations

`fn sortconf()` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/SplitGraph.jl#L75-L109)]:

1. Drop the configuration that removes nothing
2. If one configuration remains, take it
3. If `likelihood_sort` is off, take one at random
4. Otherwise, compute the branch-length likelihood of each configuration and keep those with the highest value. If several remain, keep those with the lowest energy, then take one at random. If the highest likelihood is `missing`, warn "Maximum likelihood is `missing`: this may be due to missing branch lengths"

The documentation shows a three-leaf case where topology allows three solutions and the likelihood selects one [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/options.md?plain=1#L50-L65)].

## Branch-length likelihood

The likelihood compares, for each branch that a configuration declares shared, the hypothesis "same branch in both trees" against "independent branches" under a Poisson model of mutations [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/likelihood.jl#L8-L30)]. For branch lengths $\tau_1, \tau_2$ and sequence lengths $L_1, L_2$, the expected mutation counts are $n_k = \tau_k L_k$ for independent branches and $\bar n_k = L_k (\tau_1 L_1 + \tau_2 L_2)/(L_1 + L_2)$ for one shared branch length. The log-ratio is

$$\ell(\tau_1, \tau_2) = \sum_{k=1}^{2} \left[ n_k - \bar n_k + n_k \ln \frac{\bar n_k}{n_k} \right]$$

with the term $-\bar n_k$ when $n_k = 0$. Here $n_k$ takes the role of the observed count, so $\ell \le 0$, and $\ell = 0$ when the two scaled lengths are equal.

- **Missing lengths**: one `missing` length gives `missing`. Two `missing` lengths give 0
- **Configuration likelihood** (`fn conf_likelihood_()`) [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/likelihood.jl#L44-L104)]: for each remaining leaf and pair of trees where the ancestors match (same rule as the energy), add $\ell$ of the distances from the leaf to its first non-trivial ancestor in each tree (TreeTools `divtime`). For each removed leaf, subtract $\ell$ of the branches directly above it, over all ordered pairs of trees including a tree with itself. Divide by the number of terms
- **Sequence lengths**: `seq_lengths` scales the counts. Through the command line, all lengths are 1 (see [`cli.md`](cli.md))
- **Unused code**: the `mode` keyword of `conf_likelihood` is ignored, and `conf_likelihood_times` has no caller

Because `maximum` of a vector with `missing` returns `missing`, one missing branch length in a configuration makes the configurations with `missing` likelihood win over all others.

[`treeknit-paper-vs-code.md`](../reports/treeknit-paper-vs-code.md) compares this procedure with the published method. The likelihood combination differs from the paper: the code averages the log-ratios and counts each removed leaf twice, while the paper multiplies the ratios.
