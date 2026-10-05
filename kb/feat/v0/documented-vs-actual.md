# Defects of TreeKnit.jl and differences from its documentation

This page lists the defects of TreeKnit.jl at `master` (`dbbc89a`) and the places where the code does not do what the help text, a docstring, or the documentation says. The sections are ordered by their effect on results. The other documents in `kb/feat/v0/` link here instead of repeating these items. Upstream pull request [#42](https://github.com/PierreBarrat/TreeKnit.jl/pull/42) proposes fixes for several of them (see [`history.md`](history.md)).

## Results change without a message

### Command-line options without effect

`set_up_optargs` passes the option values to `OptArgs(K; method, γ, likelihood_sort, nMCMC, seq_lengths, parallel)` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L270)]. That constructor reads five correctly spelled keywords (`pre_resolve`, `strict`, `resolve`, `final_no_resolve`, `rounds`) and the misspelled key `seq_lenghts` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/objects.jl#L61-L92)]. It drops all other values. The constructor exists since v0.5.0 (2023-01-26), and the misspelling since v0.5.4 (2023-05-28).

- **`-g, --gamma`**: documented as the parsimony parameter. Every run uses $\gamma = 2$. The log line "γ: <value>" shows the given value
- **`--n-mcmc-it`**: documented as "number of MCMC iterations per leaf; default 25". Every run uses `nMCMC = 50`
- **`--seq-lengths`**: documented as the sequence lengths for the likelihood test. The command parses and checks the value, then every run uses length 1 for each tree. With equal sequence lengths this changes no result, because a common length scales all log-ratios by the same factor (see [`treeknit-paper-vs-code.md`](../../reports/treeknit-paper-vs-code.md))
- **`--no-likelihood`**: documented as turning off the likelihood sort. The likelihood sort always runs
- **`--parallel`**: documented as parallel MultiTreeKnit for three or more trees. The command logs "Running in parallel" and runs the pairs one after another
- **`parameters.json`**: shows the defaults for these five fields

The same constructor drops keywords in library calls:

- **`run_treeknit!(trees; kwargs...)`**: the docstring says the keyword arguments go to `OptArgs`. Only the keywords above have an effect. `naive = true` also goes to `OptArgs`, so the annealing runs. The two-tree form `run_treeknit!(t1, t2; naive = true)` uses the keyword constructor and works [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L160-L164)]
- **`runopt(t1, t2; kwargs...)`**: the docstring says the keywords go to `OptArgs`. The function uses `OptArgs(2; kwargs...)`, so $\gamma$ and the annealing parameters keep their defaults [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L202-L204)]

### Other result-changing behavior

- **Missing branch lengths**: one `missing` length makes the likelihood of a configuration `missing`, and the configurations with `missing` likelihood win the tie break. Only a warning about the maximum appears (see [`mcc-inference.md`](mcc-inference.md))
- **Unseeded randomness**: the annealing, the tie breaks, and the ARG node labels use the global Julia random generator, and no option or code path seeds it [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/energy.jl#L220-L231)]. Two command-line runs on the same input can give different MCCs
- **Support values as labels**: numeric internal labels get a random suffix on reading, so output labels change from run to run (see [`formats.md`](formats.md))
- **Parallel mode**: the documentation describes a schedule that follows the dependencies between pairs. In the code, `run_treeknit!` with `parallel = true` raises a `MethodError` (since v0.5.2, 2023-03-23). `run_parallel_treeknit!`, called directly, starts all tasks at once on shared mutable trees and resolves in non-final rounds even when `resolve = false` (see [`pipeline.md`](pipeline.md))

## Command-line options with other effects

- **`--resolve-all-rounds`**: the help says it "overrides `--no-resolve`". The code sets `final_no_resolve = false` and leaves `resolve` unchanged [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L276)]. With `--better-trees`, the default for more than two trees, `resolve` is `false`, so the flag has no effect
- **`--rounds 1`**: the help describes `--rounds` as the number of rounds. Because the value is applied only when it differs from 1, `--rounds 1` keeps the two rounds of `--better-MCCs` for more than two trees
- **`--verbosity-level -1`**: the help says "no output at all". Warnings and the annealing progress bar still appear
- **`ARG/nodes.dat`**: the documentation says that the node table refers to the liberally resolved trees in `ARG/`. It also names `Singleton_...` nodes that those files do not contain (see [`formats.md`](formats.md))

## Library functions that fail

- **`naive_mccs(treelist)`**: the docstring describes any number of trees. With three or more trees that share a clade, `is_coherent_clade` raises a `MethodError`, because its signature accepts only a tuple of two split lists [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_base.jl#L104)]
- **`inferARG`**: exported and documented. It uses the undefined name `trees`, so every call raises an `UndefVarError` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L180-L188)]
- **`iter_shared`**: reads the field `tree_order`, which `MCC_set` does not have [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/objects.jl#L210-L212)]
- **`SRG.set_ancestor!(n, a)`, `SRG.unset_child!`, `SRG.prune!`**: iterate over a Boolean vector as `(i, c)` pairs, which fails at run time. The ARG construction does not call them [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/objects.jl#L168-L174)]
- **`parse_nexus`**: defined in `src/reading.jl`, which the module does not include. It also calls `parse_muts`, which is defined nowhere
- **`MCC_` label parsing**: a node label that contains "MCC" in another form, for example `A/MCC/x`, stops the annealing with a parse error (see [`mcc-inference.md`](mcc-inference.md))

## Documentation that differs from the code

- **`OptArgs` docstring**: gives `nMCMC = 25`, `Tmax = 0.8`, `nT = 3000`. The code uses 50, 1, and 100 since v0.3.1 (2022-05-18) [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/objects.jl#L25-L50)]
- **`write_auspice_json`**: the documentation gives the form `write_auspice_json(filepath, tree1, tree2, MCCs::Vector{Vector{String}})`. The only method takes a vector of trees and an `MCC_set` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_IO.jl#L90)]
- **`sort_polytomies!`**: the docstring says the order of the first tree is left unchanged. With `strict = true`, the function reorders both trees [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_tools.jl#L176-L196)]
- **Counting reassortments**: the documentation states a rule that differs from the published paper and the ARG construction (see [`mcc-inference.md`](mcc-inference.md))
