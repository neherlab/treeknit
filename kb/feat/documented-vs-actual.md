# Where TreeKnit.jl differs from its documentation

These differences were found by reading the source at `master` (`dbbc89a`). Each item gives what the help text, the docstring, or the documentation says, what the code does, and where. Upstream pull request [#42](https://github.com/PierreBarrat/TreeKnit.jl/pull/42), open since 2026-10-02, proposes fixes for several of them (see [`history.md`](history.md)).

## Command-line options without effect

`set_up_optargs` passes the option values to `OptArgs(K; method, γ, likelihood_sort, nMCMC, seq_lengths, parallel)` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L270)]. That constructor reads only `pre_resolve`, `strict`, `resolve`, `final_no_resolve`, `rounds`, and the misspelled key `seq_lenghts` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/objects.jl#L61-L92)]. All other values are dropped. The constructor exists since v0.5.0 (2023-01-26); the misspelling since v0.5.4 (2023-05-28).

- **`-g, --gamma`**: documented as the parsimony parameter. Every run uses `γ = 2`. The log line "γ: <value>" shows the ignored value
- **`--n-mcmc-it`**: documented as "number of MCMC iterations per leaf; default 25". Every run uses `nMCMC = 50`
- **`--seq-lengths`**: documented as the sequence lengths for the likelihood test. The value is parsed and checked, then every run uses length 1 for each tree
- **`--no-likelihood`**: documented as turning off the likelihood sort. The likelihood sort always runs
- **`--parallel`**: documented as parallel MultiTreeKnit for three or more trees. The command logs "Running in parallel" and runs sequentially
- **`parameters.json`**: shows the defaults for these five fields, not the values given on the command line

## Command-line options with other effects

- **`--resolve-all-rounds`**: the help says it "overrides `--no-resolve`". The code only sets `final_no_resolve = false` and leaves `resolve` unchanged [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L276)]. With `--better-trees`, the default for more than two trees, `resolve` is `false`, so the flag has no effect
- **`--rounds 1`**: the value replaces the method default only when it differs from 1, so `--rounds 1` cannot reduce the two rounds of `--better-MCCs` for more than two trees
- **`--verbosity-level -1`**: the help says "no output at all". Warnings and the annealing progress bar still appear
- **`ARG/nodes.dat`**: the documentation says it refers to the liberally resolved trees in `ARG/`. It names `Singleton_...` nodes that the ARG construction adds to its own copies after those trees are written (see [`formats.md`](formats.md))

## Library functions

- **`OptArgs(K; kwargs...)`**: the docstring of `run_treeknit!` says keyword arguments go to `OptArgs`. Only five of them have an effect (see above)
- **`run_treeknit!(trees; naive = true)`**: `naive` goes to `OptArgs` and is ignored, so the annealing runs. The two-tree form `run_treeknit!(t1, t2; naive = true)` works [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L160-L164)]
- **`runopt(t1, t2; kwargs...)`**: the docstring says the keywords go to `OptArgs`. It uses `OptArgs(2; kwargs...)`, which ignores `γ` and the annealing parameters [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L202-L204)]
- **Parallel mode**: the documentation describes a schedule that follows the dependencies between pairs. `run_treeknit!` with `parallel = true` raises a `MethodError` because of a keyword name mismatch, and `run_parallel_treeknit!` starts all tasks without dependencies on shared mutable trees (see [`pipeline.md`](pipeline.md)). The mismatch exists since v0.5.2 (2023-03-23)
- **`naive_mccs(treelist)`**: the docstring describes any number of trees. With three or more trees that share a clade, `is_coherent_clade` raises a `MethodError` because it accepts only two split lists [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_base.jl#L104)]
- **`inferARG`**: exported and documented. It uses the undefined name `trees` and always fails [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L180-L188)]
- **`iter_shared`**: reads the field `tree_order`, which `MCC_set` does not have [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/objects.jl#L210-L212)]
- **`write_auspice_json`**: the documentation gives a form with two trees and an MCC list. Only the form with a vector of trees and an `MCC_set` exists
- **`sort_polytomies!`**: the docstring says the order of the first tree is left unchanged. With `strict = true`, both trees are reordered [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_tools.jl#L176-L196)]
- **`parse_nexus`**: present in `src/reading.jl`, which the module does not include, and it calls the undefined `parse_muts`
- **`SRG` helpers**: `set_ancestor!(n, a)`, `unset_child!`, and `prune!` fail at run time (see [`arg.md`](arg.md))

## Defaults and descriptions

- **`OptArgs` docstring**: gives `nMCMC = 25`, `Tmax = 0.8`, `nT = 3000`. The code uses 50, 1, and 100 since v0.3.1 (2022-05-18) [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/objects.jl#L25-L50)]
- **Counting reassortments**: the documentation subtracts the MCC that contains "the roots of both trees" [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/mccs.md?plain=1#L44-L47)]. The published paper and the ARG construction use the MCC that contains the root of either tree (see [`arg.md`](arg.md))
- **TreeTools `ladderize!`**: the docstring says largest clades first. The code puts the smallest clades first (see [`visualization.md`](visualization.md))
- **TreeTools `read_tree`**: the docstring says duplicate labels get a random suffix. The code raises an error (see [`formats.md`](formats.md))

## Behavior that affects results without a message

- **Missing branch lengths**: one `missing` length in a compared pair makes the configuration likelihood `missing`. `maximum` then returns `missing`, and the configurations with `missing` likelihood are kept over all others. Only a warning about the maximum appears (see [`mcc-inference.md`](mcc-inference.md))
- **Unseeded randomness**: annealing, tie breaks, and ARG labels use the unseeded global random generator, so command-line runs are not reproducible
- **Support values as labels**: numeric internal labels get a random suffix on reading (see [`formats.md`](formats.md))
