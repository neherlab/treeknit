# Run pipeline for two or more trees

`run_treeknit!` infers the MCCs of every pair in a list of trees [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L141-L168)]. One pair of trees uses the procedure of [`mcc-inference.md`](mcc-inference.md). This page describes the parameters, the order of the pairs, the resolution between pairs, and the sorting at the end. The documentation calls the pipeline for more than two trees "MultiTreeKnit".

## `OptArgs` parameters

`struct OptArgs` holds all run parameters [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/objects.jl#L33-L53)]. It is mutable, and the run changes some fields (see [Side effects](#side-effects)).

- **`γ = 2.0`**: cost of one removed clade in the annealing score
- **`itmax = 15`**: maximum number of iterations of the annealing in `runopt`
- **`likelihood_sort = true`**: use the branch-length likelihood to choose among configurations with the same score
- **`resolve = true`**: resolve during the inference of a pair and after it
- **`strict = true`**: add only unambiguous splits when resolving with MCCs
- **`seq_lengths = [1, 1]`**: sequence length of each tree, used by the likelihood
- **`pre_resolve = true`**: resolve all trees together before the first pair
- **`rounds = 1`**: number of passes over all pairs
- **`final_no_resolve = false`**: do not resolve in the last round
- **`parallel = false`**: use the parallel pair scheduler
- **`nMCMC = 50`**: annealing steps per leaf, summed over all temperatures
- **`sa_rep = 1`**: number of independent annealing runs per iteration
- **`Tmin = 0.05`, `Tmax = 1`, `nT = 100`, `cooling_schedule = :geometric`**: the temperature schedule. The constructor asserts `Tmin > 0` and `Tmax > Tmin`
- **`Trange`**: the list of temperatures, computed once at construction from the four fields above. A later change of `Tmin`, `Tmax`, `nT`, or `cooling_schedule` does not update it

The docstring gives different defaults for `nMCMC` (25), `Tmax` (0.8), and `nT` (3000) [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/objects.jl#L25-L31)].

### Cooling schedules

`fn get_cooling_schedule()` returns the temperatures from hot to cold [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/objects.jl#L94-L124)]. An unknown name is an error.

- **`:geometric`** (default): $T_i = T_{max}\,\alpha^i$ for $i = 0 \dots n$, with $\alpha = \exp\left((\ln T_{min} - \ln T_{max}) / n_T\right)$ and $n = \lceil (\ln T_{min} - \ln T_{max}) / \ln \alpha \rceil$. This gives $n_T + 1$ temperatures, 101 with the defaults
- **`:linear`**: $n_T$ equally spaced temperatures from $T_{max}$ to $T_{min}$
- **`:acos`**: $n_T$ temperatures $T = (T_{max} - T_{min})\,f(x) + T_{min}$ for $x$ equally spaced in $[0, 1]$, where $f$ falls from 1 to 0 along an arccosine curve with exponent $K = 1.5$. The code uses 3.14 for $\pi$

where $T_{min}$, $T_{max}$, and $n_T$ are the fields `Tmin`, `Tmax`, and `nT`.

## Constructors and run methods

- **`OptArgs(; kwargs...)`**: the keyword constructor from `Parameters.jl`. It accepts every field, for example `OptArgs(γ = 3, resolve = false)`. Without arguments it gives the values above, which are the `:better_MCCs` values for two trees
- **`OptArgs(K; method = :none, kwargs...)`**: the constructor for `K` trees [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/objects.jl#L61-L92)]. It starts from `OptArgs()`, sets the method defaults, and reads only the keywords `pre_resolve`, `strict`, `resolve`, `final_no_resolve`, and `rounds`. It ignores all other keywords, such as `γ`, `nMCMC`, `likelihood_sort`, and `parallel`. It reads sequence lengths from the misspelled key `seq_lenghts`, so `seq_lengths` is ignored. The command line and `run_treeknit!(trees; kwargs...)` use this constructor
- **`:better_trees`** (default for `K > 2`): `resolve = false`, `final_no_resolve = true`, `rounds = 1`. The trees change only in the pre-resolution, so all pairs see the same trees
- **`:better_MCCs`** (default for `K = 2`): `resolve = true`. For `K = 2`: `final_no_resolve = false`, `rounds = 1`. For `K > 2`: `final_no_resolve = true`, `rounds = 2`, so the first round resolves and the second round infers the MCCs again without resolution
- **Clean-up rule**: if `final_no_resolve`, `rounds == 1`, and `resolve` are all true, the constructor sets `resolve = false` and `final_no_resolve = false`
- **Other method values**: an error "`method` should be one of `:better_trees`, `:better_MCCs` or `:none`"

The documentation explains the trade-off between the methods with a three-tree example [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/multitreeknit.md?plain=1#L13-L72)]. `:better_trees` adds fewer wrong splits but infers more reassortments. `:better_MCCs` gives more accurate MCCs but can add wrong splits, more so with more trees.

## Sequence of a run

`fn run_treeknit!()` with a list of trees and an `OptArgs` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L141-L158)]:

1. **Pre-resolution**: if `pre_resolve`, call `resolve!(trees...)`, which adds each split that all trees accept (see [`resolution.md`](resolution.md))
2. **Pair function**: `runopt`, or `naive_mccs` when `naive = true`
3. **Pairs**: `fn run_standard_treeknit!()` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L28-L74)] loops over the rounds and, in each round, over the pairs `(1,2), (1,3), ..., (1,K), (2,3), ..., (K-1,K)`. For each pair:
   1. If there are more than two sequence lengths, set `seq_lengths` to the lengths of the two trees
   2. In the last round with `final_no_resolve`, set `resolve = false` and `strict = false`
   3. Infer the MCCs of the pair and store them in the `MCC_set`. A later round replaces the result of an earlier round
   4. If `resolve`, resolve both trees with the MCCs, strictly or liberally
   5. In the last round, ladderize tree 1 when the pair contains it, then sort the polytomies of the pair so that the MCCs face each other (see [`visualization.md`](visualization.md))
4. **Result**: an `MCC_set` that maps each pair of tree labels to its MCCs (see [`julia-api.md`](julia-api.md))

Properties of this sequence:

- **The order of the trees matters**: with `resolve`, the trees resolved by one pair are the input of the next pairs. A different input order can give different trees and MCCs. The documentation draws the dependency graph between pairs [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/multitreeknit.md?plain=1#L74-L94)]
- **The MCCs of a pair can become wrong**: a later pair can add splits to a tree after the MCCs of an earlier pair were inferred. The second round of `:better_MCCs` for `K > 2` exists to infer the MCCs again on the final trees
- **No consistency between pairs**: the MCCs of the pairs `(a,b)`, `(a,c)`, and `(b,c)` can contradict each other. The pipeline does not check or fix this [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/multitreeknit.md?plain=1#L96-L123)]
- **The input trees change**: `run_treeknit!` resolves and reorders the trees in place. `run_treeknit` (without `!`) runs on copies

### Side effects

- **The trees**: resolved (new nodes `RESOLVED_<i>` with branch length 0), ladderized, and sorted
- **`OptArgs`**: the loop writes `oa.seq_lengths` for each pair when there are more than two lengths, and writes `oa.resolve = false` and `oa.strict = false` in the last round when `final_no_resolve` is true. The caller's object keeps these values after the run

## Entry points

- **`run_treeknit!(trees::AbstractVector{<:Tree}, oa::OptArgs; naive = false)`**: the main form
- **`run_treeknit!(trees; kwargs...)`**: builds `OptArgs(length(trees); kwargs...)`. The keyword `naive` goes to `OptArgs`, which ignores it, so this form always runs the annealing
- **`run_treeknit!(t1, t2; naive = false, kwargs...)`**: builds `OptArgs(; kwargs...)`, so all fields can be set
- **`run_treeknit!(t1, t2, oa)`**: two trees with given options
- **`run_treeknit(trees, oa)`**, **`run_treeknit(t1, t2, oa)`**: the same on copies

## Naive mode

With `naive = true`, the pair function returns the naive MCCs of the two trees [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L148-L152)]: the largest clades that have exactly the same topology in both trees. The documentation describes this as the limit $\gamma \to \infty$ [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/options.md?plain=1#L19)]. Pre-resolution, resolution with the MCCs, and sorting run as in the normal mode.

## Parallel mode

`fn run_parallel_treeknit!()` starts one Dagger task per pair and per round, each running `fn run_step!()` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L76-L118)]. The documentation describes a schedule in which each pair waits for the pairs that resolve its trees first, which gives a critical path of $2K - 2$ pairs instead of $K(K-1)/2$ [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/multitreeknit.md?plain=1#L125-L133)]. The code differs from this description:

- **Not reachable from `run_treeknit!`**: `run_treeknit!` passes the keyword `infer_mccs_function`, but `run_parallel_treeknit!` accepts only `func_`, so Julia raises a `MethodError` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L153-L154)]. `OptArgs(K; parallel = true)` also drops the keyword. The tests call `run_parallel_treeknit!` directly
- **No dependencies between tasks**: the tasks get the tree objects and the shared `OptArgs` as plain arguments. Dagger 0.16 creates dependencies only for task results passed as arguments and passes other objects as they are (see [`treeknit-ecosystem.md`](../reports/treeknit-ecosystem.md)). Tasks that share a tree can run at the same time and resolve the same tree. All rounds are started at once
- **Shared mutable `OptArgs`**: `run_step!` sets `resolve` and `strict` on the shared object in the last round, which other tasks can see
- **Last round only**: each task writes its result into one dictionary entry per pair, so the result of the last started task for a pair remains
