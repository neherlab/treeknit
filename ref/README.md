# Reference fixtures from legacy Julia TreeKnit

Reference version: TreeKnit 0.5.8 from `/workspace/legacy_julia_version`, branch
`fix/issues-from-rust-port` (commit `186bf0d`), with TreeTools 0.6.14 on Julia 1.13.1. That branch
fixes bugs found during the port; the Rust implementation follows it, including the strict-resolution
fix for sisters whose MCC continues above a polytomy. Output goes to `../fixtures/<case>.json`.
Simulated trees are also written to `../fixtures/sim/<case>/tree<k>.nwk`. The web app bundles these files as the examples of its **Load example** menu.

## Environment

One environment serves both scripts:

```sh
mkdir -p /tmp/tkref/fixedenv && julia --project=/tmp/tkref/fixedenv -e 'using Pkg;
  Pkg.develop([PackageSpec(path="/workspace/legacy_julia_version"), PackageSpec(path="/workspace/ARGTools")]);
  Pkg.add(["JSON3","Distributions"]); Pkg.add(name="TreeTools", version="0.6.14")'
```

## Regenerate

```sh
julia --project=/tmp/tkref/fixedenv /workspace/treeknit-rs/ref/dump_fixtures.jl   # doc/test/real cases (~20 s)
julia --project=/tmp/tkref/fixedenv /workspace/treeknit-rs/ref/simulate.jl        # simulated cases (~20 s)
```

- `--no-runs` skips the 20 seeded `run_treeknit!` runs.
- Extra positional args restrict the run to the named cases, e.g. `dump_fixtures.jl real_ny`.
- The output is byte-identical across reruns and does not depend on the order cases are run in.

## Files

- `fixture_lib.jl`: computes all fields for one case (`make_fixture`).
- `dump_fixtures.jl`: the case list taken from docs, tests and real data.
- `simulate.jl`: the ARGTools simulation grid. It also contains a port of `TestRecombTools.remove_branches!`, with a guard so the root is never deleted.

## Format notes

These go beyond the spec in the task.

- Leaf sets are sorted string arrays. MCC lists are sorted by `(length, leaves)`. Split lists are sorted lexicographically.
- The trees in each pair are labelled `t1`, `t2`, ….
- `mccs_source` records where `mccs` came from: `given` (MCCs taken from the legacy tests), the seeded run, or true MCCs.
- `fitch[i].mcc` is a 0-based index into `mccs`.
- `n_leaves` is a per-tree leaf count.
- `arg.segment_trees` lists the raw clades of the segment trees. Hybrid nodes are internal singletons there, so their clades appear twice. `arg.segment_trees_no_singletons` is the same after `remove_internal_singletons!`.
- `energy[].lk` is `SplitGraph.conf_likelihood(conf, g, seq_lengths, treelist)` with `set_resolve(true)`. The `mode` keyword is ignored by the legacy code, and `divtime` is always used.
- Energy configurations are: all ones, each single MCC removed, and 20 random configurations. The random ones use `MersenneTwister(1)` with removal probability cycling through 0.1/0.25/0.5. Duplicate configurations are removed.
- Fields only for K>2:
  - `pairwise_naive_mccs` maps `"i-j"` to the naive MCCs of that pair.
  - `multi_runs` holds 5 runs of `run_treeknit!(copies, OptArgs(K))` with `Random.seed!(i)`. Each run maps `"i-j"` to its MCCs.
- Simulated cases also have `sim_params` and `true_mccs`, which maps `"i-j"` to `ARGTools.MCCs_from_arg(arg, i+1, j+1)`.
