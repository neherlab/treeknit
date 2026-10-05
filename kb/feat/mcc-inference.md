# MCC inference for one pair: parity checklist

Counterpart: [`v0/mcc-inference.md`](v0/mcc-inference.md). `fn infer_pair` takes the role of `runopt` [[src](../../packages/treeknit-core/src/pair.rs#L31-L83)]. Its inputs are two trees with the same leaf set; the pipeline restricts each pair to its shared leaves first (see [`partial-overlap.md`](partial-overlap.md)).

## Interpretation of MCCs

- [x] **Partition**: the MCCs of a pair cover every shared leaf once. Leaves that only one tree has join an MCC afterwards
- [x] **Output order**: each MCC is sorted, and the list is sorted by size, then by the first leaf [[src](../../packages/treeknit-core/src/naive.rs#L14-L21)]. Taxon numbers follow the sorted leaf names [[src](../../packages/treeknit-core/src/tree.rs#L36-L49)], so the order equals the label order of TreeKnit.jl
- [x] **Number of reassortments**: the hybrid count of the ARG (see [`arg.md`](arg.md))

## Naive MCCs

`fn naive_mccs` [[src](../../packages/treeknit-core/src/naive.rs#L23-L56)]:

- [x] **Algorithm**: from each unassigned leaf, move up in all trees while the parents have the same clade and identical subtrees. Fixture comparison: `naive_mccs` of every case
- [x] **Any number of trees**: the coherence check compares the children of all trees [[src](../../packages/treeknit-core/src/naive.rs#L58-L98)]. TreeKnit.jl fails with three or more trees that share a clade. Fixture comparison: the three-tree cases
- [x] **No resolution**: a polytomy and a resolved clade count as different
- [/] **Precondition**: trees with different leaf sets panic with "trees do not share leaves". TreeKnit.jl raises an error. The pipeline calls the function only on trees restricted to the same leaves

## Iteration in `runopt`

- [x] **Resolution of the copies**: `resolve_trees` on both copies when the round resolves
- [x] **Steps per temperature**: $M = \lceil n \cdot n_{MCMC} / n_{temp} \rceil$ with the number of remaining leaves, at most 2^32 - 1 on every target [[src](../../packages/treeknit-core/src/pair.rs#L85-L95)]
- [x] **Stop rules**: nothing removed, removed MCCs cover all leaves, one naive MCC left, or the iteration counter above `itmax`, so at most `itmax + 1` annealing runs [[src](../../packages/treeknit-core/src/pair.rs#L63-L80)]
- [x] **Pruning**: `prune_mccs` removes each removed MCC and splices out unary nodes, adding branch lengths [[src](../../packages/treeknit-core/src/pair.rs#L189-L197)]
- [ ] **Tree check after each iteration**: TreeKnit.jl asserts `check_tree`. `Tree::check` exists [[src](../../packages/treeknit-core/src/tree.rs#L455-L462)], but `infer_pair` does not call it
- [ ] **`output = :all`**: TreeKnit.jl can also return the two pruned trees. `infer_pair` returns only the MCCs

## One annealing pass

`fn remove_mccs` takes the role of `opttrees` [[src](../../packages/treeknit-core/src/pair.rs#L103-L131)]:

- [x] **Single naive MCC**: returned without annealing
- [x] **Coarse-graining**: `reduce_to_mccs` replaces each naive MCC by one leaf whose taxon number is the MCC index [[src](../../packages/treeknit-core/src/pair.rs#L168-L187)]. The port does not read numbers back from labels, so a label such as `A/MCC/x` works. In TreeKnit.jl it stops the run with a parse error
- [x] **Split graph**: `Graph::new` keeps, for each tree, the clades, parents, and children of the internal nodes and the parent of each leaf [[src](../../packages/treeknit-core/src/splitgraph.rs#L152-L201)]. A tree that is one leaf gets an artificial root
- [x] **Result**: the naive MCCs that the chosen configuration removes

## Energy and score

- [x] **Energy $E(c)$**: for each kept leaf and each pair of trees, one mismatch when the first ancestors with two kept leaves have different kept clades [[src](../../packages/treeknit-core/src/splitgraph.rs#L83-L104)]. Fixture comparison: `E_noresolve` of every configuration
- [x] **Compatibility with resolution**: a clade inside the other counts as equal when the larger node can be resolved to contain it [[src](../../packages/treeknit-core/src/splitgraph.rs#L62-L81)]. Fixture comparison: `E_resolve`
- [x] **Score**: $F(c) = E(c) + \gamma (N - \sum_i c_i)$ [[src](../../packages/treeknit-core/src/anneal.rs#L85-L87)]
- [x] **`count_mismatches`**: `Graph::energy` with all leaves kept
- [x] **Incremental energy (new)**: `EnergyState` recomputes only the terms of the leaves that a flip can change, and can undo the last flip [[src](../../packages/treeknit-core/src/splitgraph.rs#L203-L357)]. A unit test compares it with the full computation over 3,000 random flips. TreeKnit.jl recomputes the full energy at each step

## Simulated annealing

`fn optimize` [[src](../../packages/treeknit-core/src/anneal.rs#L142-L173)]:

- [x] **Chain**: start with all leaves kept, flip one leaf chosen uniformly, accept with the Metropolis rule, $M$ steps per temperature from hot to cold, the chain continues between temperatures [[src](../../packages/treeknit-core/src/anneal.rs#L89-L139)]
- [x] **Memory of optima**: every distinct configuration with the lowest score seen, in the order found
- [x] **Repetitions**: `sa_rep` runs; the runs with the lowest score contribute. Library only, as in the TreeKnit.jl command line
- [x] **Resolution flag**: passed as an argument. TreeKnit.jl stores it in a module-level variable
- [x] **Random generator**: the seeded generator of the pair (see [`pipeline.md`](pipeline.md#reproducibility))
- Not ported: the restart option `reset_chance`, which has probability 0 in TreeKnit.jl

## Choice among configurations with the same score

`fn choose_conf` [[src](../../packages/treeknit-core/src/pair.rs#L133-L166)]:

- [x] **Order of the rules**: drop the configuration that removes nothing; take the only one left; without the likelihood, take one at random; otherwise keep the highest likelihood, then the lowest energy, then take one at random
- [x] **Missing branch lengths**: they contribute 0, so no configuration has a missing likelihood and no warning is needed. TreeKnit.jl prefers the configurations with a `missing` likelihood. The port avoids this on purpose ([README](../../README.md#deliberate-differences-from-treeknitjl))
- [x] **Undefined likelihood**: zero sequence lengths would give `NaN` for every configuration, and the choice would panic. The shared validation accepts only finite positive sequence lengths, on the command line and in the web app [[src](../../packages/treeknit-io/src/analysis.rs#L284-L335)]

## Branch-length likelihood

- [x] **Log-ratio $\ell(\tau_1, \tau_2)$**: the Poisson log-ratio of TreeKnit.jl, with the term $-\bar n_k$ when $n_k = 0$ [[src](../../packages/treeknit-core/src/splitgraph.rs#L359-L366)]
- [x] **Configuration likelihood**: the mean over the compatible ancestor pairs of kept leaves and over all ordered tree pairs of removed leaves, as in TreeKnit.jl [[src](../../packages/treeknit-core/src/splitgraph.rs#L106-L139)]. This combination differs from the published method in the same way (see [`treeknit-paper-vs-code.md`](../reports/treeknit-paper-vs-code.md)). Fixture comparison: `lk` of every configuration, with a computed tolerance ([`N-fixture-likelihood-tolerance.md`](../issues/N-fixture-likelihood-tolerance.md))
- [x] **Missing lengths**: one or two missing lengths give 0. TreeKnit.jl gives `missing` for one
- [x] **Sequence lengths**: from `--seq-lengths`. TreeKnit.jl uses length 1 for every tree on the command line
- Not ported: the ignored `mode` keyword and the uncalled `conf_likelihood_times`

## Outcomes compared with TreeKnit.jl

- [x] **Two-tree runs**: `annealing_distribution_vs_julia` runs every fixture case with 20 seeds and compares the MCC sets with the 20 Julia runs. It fails where all Julia runs agree and a Rust run differs [[src](../../packages/treeknit-io/tests/fixtures.rs#L298-L339)]
- [x] **Accuracy**: `examples/accuracy.rs` reports the scaled variation of information to the true MCCs of the simulated cases, for Julia and Rust
