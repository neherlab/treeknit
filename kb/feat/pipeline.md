# Run pipeline: parity checklist

Counterpart: [`v0/pipeline.md`](v0/pipeline.md). `treeknit_core::run` runs the pipeline for two or more trees [[src](../../packages/treeknit-core/src/pipeline.rs#L131-L135)]. `run_observed` is the same run with a progress observer [[src](../../packages/treeknit-core/src/pipeline.rs#L137-L217)].

## Run options

`struct Options` holds the run options [[src](../../packages/treeknit-core/src/options.rs#L35-L94)]. The run reads it and does not change it.

- [x] **`gamma`, `itmax`, `likelihood_sort`, `seq_lengths`, `pre_resolve`, `rounds`, `parallel`**: the `OptArgs` fields with the same meaning
- [x] **`n_mcmc`, `sa_rep`, `t_min`, `t_max`, `n_t`, `cooling`**: the annealing fields `nMCMC`, `sa_rep`, `Tmin`, `Tmax`, `nT`, `cooling_schedule`
- [x] **`resolution`**: one enum, `None`, `Strict`, `Liberal`, or `Matched` [[src](../../packages/treeknit-core/src/options.rs#L12-L22)], in place of the two flags `resolve` and `strict`. `Matched` is new (see [`resolution.md`](resolution.md#matched-resolution-new))
- [x] **`final_unresolved_round`**: with `Strict` or `Liberal` and more than two trees, one extra round without resolution after `rounds` resolving rounds. In TreeKnit.jl, `final_no_resolve` turns the last of the `rounds` rounds into a round without resolution. The former command-line options convert one form into the other (see [Method presets](#method-presets))
- [x] **`naive`**: a field of `Options`. In TreeKnit.jl it is a keyword of `run_treeknit!`
- [x] **`sort_strict` (new)**: chooses the strict or non-strict polytomy sort, to reproduce the output order of the former option combinations. `sort_strictness` gives the choice of a run [[src](../../packages/treeknit-core/src/pipeline.rs#L77-L87)]
- [x] **Temperatures**: `Options::temperatures` computes the list from the four schedule fields at each call [[src](../../packages/treeknit-core/src/options.rs#L124-L126)]. TreeKnit.jl computes `Trange` once, so a later change of a field has no effect there
- [x] **Defaults**: `matched` resolution and no pre-resolution, for any number of trees. This differs from TreeKnit.jl on purpose ([README](../../README.md#deliberate-differences-from-treeknitjl)). All other defaults are those of `OptArgs()`

### Cooling schedules

`fn schedule` [[src](../../packages/treeknit-core/src/anneal.rs#L20-L49)]:

- [x] **Geometric**: the default. A unit test compares three temperatures with TreeKnit.jl
- [x] **Linear**: implemented, without a test
- [x] **Acos**: implemented with 3.14 for $\pi$, as in TreeKnit.jl, without a test
- [x] **Range check**: the function panics unless $0 < T_{min} < T_{max}$. TreeKnit.jl asserts the same in the constructor
- [x] **Selection**: only through the library. The command line has no option for the schedule, as in TreeKnit.jl

## Method presets

- [x] **Keyword constructor**: Rust struct update syntax, for example `Options { gamma: 3.0, ..Options::default() }`, sets any field
- [x] **`OptArgs(K; method)`**: `Options::treeknit_jl(k, method)` gives `BetterTrees` (no resolution) for $K > 2$ and `BetterMccs` (strict resolution) for $K = 2$, both with pre-resolution [[src](../../packages/treeknit-core/src/options.rs#L107-L117)]. It drops no keyword, because callers set fields directly
- [x] **Rounds and the clean-up rule**: `fn former_options` of the command line applies the TreeKnit.jl rules for `rounds` and `final_no_resolve`, including the clean-up rule, and converts them to `rounds` and `final_unresolved_round` [[src](../../packages/treeknit-cli/src/main.rs#L398-L477)]

## Sequence of a run

- [x] **Pre-resolution**: when `pre_resolve` is on, `resolve_trees` on all trees
- [x] **Pair function**: `infer_pair`, or `naive_mccs` when `naive` is on [[src](../../packages/treeknit-core/src/pipeline.rs#L377-L422)]
- [x] **Pair order**: `(0,1), (0,2), ..., (K-2,K-1)` in each round. Each pair uses the sequence lengths of its two trees
- [x] **Rounds**: a later round replaces the MCCs of an earlier round
- [x] **Resolution after each pair**: with MCCs, strict or liberal, in each resolving round [[src](../../packages/treeknit-core/src/pipeline.rs#L432-L452)]
- [x] **Final round without resolution**: the extra round for `Strict` and `Liberal` with more than two trees (see `final_unresolved_round` above)
- [x] **Sorting in the last round**: ladderize tree 0 in its pairs and sort the polytomies of each pair, strictly or not as `sort_strictness` decides (see [`visualization.md`](visualization.md)) [[src](../../packages/treeknit-core/src/pipeline.rs#L454-L504)]
- [x] **Matching (new)**: with `Matched`, `match_topologies` runs after the last round, then every pair is sorted [[src](../../packages/treeknit-core/src/pipeline.rs#L203-L209)]
- [x] **Leaves missing from one tree (new)**: each pair attaches them to its MCCs at the end (see [`partial-overlap.md`](partial-overlap.md))
- [x] **Result**: a `Vec<PairResult>` in pair order, with the indices of the two trees, the MCCs, and the attached leaves [[src](../../packages/treeknit-core/src/pipeline.rs#L28-L60)]. It replaces `MCC_set` (see [`library-api.md`](library-api.md))

### Properties

- [x] **Order of the trees matters**: as in TreeKnit.jl. With `Matched`, the splits of earlier trees also win conflicts
- [x] **Stale MCCs of earlier pairs**: the extra round of `Strict` and `Liberal` infers the MCCs again on the final trees. `Matched` makes both trees of each pair agree inside each MCC, and replaces an MCC that cannot agree with smaller MCCs
- [/] **Consistency between pairs (new, partial)**: `Matched` gives each pair trees that agree inside its MCCs. It does not make the MCC partitions of different pairs consistent with each other. TreeKnit.jl does neither
- [x] **Side effects**: `run` resolves and sorts the trees in place. It does not change `Options`. TreeKnit.jl writes `seq_lengths`, `resolve`, and `strict` into the caller's `OptArgs`
- [x] **New nodes**: `RESOLVED_<i>` with branch length 0, as in TreeKnit.jl

### Progress (new)

- [x] **Observer**: `run_observed` calls an observer with a `Progress` value: at the start of each pair, after each temperature step of its annealing, when the matching of `Matched` resolution starts, and once at the end [[src](../../packages/treeknit-core/src/progress.rs#L9-L85)]. The fraction never decreases. Pair inference and resolution fill the fraction up to `PAIRS_SHARE` (0.95); matching, sorting, and attachment after the last round report no intermediate progress, so only the final event reports 1. Pre-resolution reports nothing
- [x] **Parallel rounds**: the observer runs on the calling thread only, before and after each parallel round
- [x] **Same result**: `run` is `run_observed` with an observer that does nothing, and the observer consumes no random numbers, so observing a run does not change its result

### Display sort (new)

The output trees are sorted for the last pair that sorted them. A view of another pair sorts copies of the two trees with the run's sort of a pair, with one display choice: it always ladderizes the left tree.

- [x] **`sort_for_pair`**: ladderizes the left copy and sorts the polytomies of both copies with the run's own sort function, so the strict and non-strict sorts cannot diverge [[src](../../packages/treeknit-core/src/pipeline.rs#L122-L131)]. The run ladderizes only tree 0, as TreeKnit.jl does; the display sort ladderizes the left tree of every pair, so all pair views follow one layout rule. For a pair `(i, j)` with `i > 0` its order can therefore differ from the order that the run's sort of that pair gave. It logs nothing, so drawing a pair does not repeat the run's warnings about skipped splits. A pair with fewer than two shared leaves stays unchanged
- [x] **Inputs**: `PairResult::shared_mccs` restricts the MCCs to the shared leaves, which gives the MCCs that the run sorted the pair with, before the leaves of one tree only were attached [[src](../../packages/treeknit-core/src/pipeline.rs#L45-L59)]. `sort_strictness` gives the strictness of the run's sort
- [x] **Final trees**: `last_sorting_pair` gives the last pair whose sort reordered a tree, and `keeps_run_order` tells whether both trees of a pair are still in the order that the pair's sort left [[src](../../packages/treeknit-core/src/pipeline.rs#L94-L120)]. A view of such a pair uses the run's trees without sorting them again
- [x] **Tests**: on two trees, `sort_for_pair` on the trees that the run's sort received reproduces the run's trees in the modes `None`, `Strict`, and `Matched`, with a polytomy and with leaves in one tree only. Other tests cover the strictness choice, the last sorting pair of 2, 3, and 4 trees, skipped pairs, and the absence of log lines

## Naive mode

- [x] **Naive MCCs per pair**: in every entry point. In TreeKnit.jl, `run_treeknit!(trees; naive = true)` ignores the keyword. Pre-resolution, resolution, and sorting run as in the normal mode

## Parallel mode

- [x] **Independent pairs in parallel**: in a round without resolution, `rayon` infers all pairs in parallel [[src](../../packages/treeknit-core/src/pipeline.rs#L177-L201)]. Resolving rounds run the pairs one after another, because each pair changes the trees of later pairs. TreeKnit.jl resolves shared trees in concurrent tasks; the port avoids this on purpose ([README](../../README.md#deliberate-differences-from-treeknitjl))
- [x] **Same result for any thread count**: each pair seeds its own generator (see below), so the order of execution does not change the result
- [x] **On by default**: `Options::default()` has `parallel = true`. The command line sets the thread count with `--threads`. The web app runs on one thread ([`kb/decisions/web-app.md`](../decisions/web-app.md))

## Reproducibility

- [x] **Seeded runs (new)**: `run` takes a seed. Each pair gets a `Xoshiro256++` generator seeded from the seed, the round, and the pair indices [[src](../../packages/treeknit-core/src/pipeline.rs#L424-L430)]. TreeKnit.jl uses the global Julia generator, and its command line has no seed
- [x] **Same seed, different generator**: a Rust run and a Julia run with the same seed give different random numbers. The tests compare annealing outcomes as distributions (see [`overview.md`](overview.md#comparison-with-treeknitjl))
- [ ] **Comparison for more than two trees**: the fixtures hold five seeded Julia runs of the pipeline for each case with three or more trees (`multi_runs`). No test reads them ([`N-reference-comparison-gaps.md`](../issues/N-reference-comparison-gaps.md))
