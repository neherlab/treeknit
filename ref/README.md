# Reference fixtures from legacy Julia TreeKnit

Reference version: TreeKnit 0.5.8 from `/workspace/legacy_julia_version`, branch
`fix/issues-from-rust-port` (commit `186bf0d`), with TreeTools 0.6.14 on Julia 1.13.1. That branch
fixes bugs found during the port; the Rust implementation follows it, including the strict-resolution
fix for sisters whose MCC continues above a polytomy. Output goes to `../fixtures/<case>.json`.
Simulated trees are also written to `../fixtures/sim/<case>/tree<k>.nwk`. The web app bundles these files as the examples of its **Examples** menu.

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
- `simulate.jl`: the ARGTools simulation grid.
- `sim_lib.jl`: a port of `TestRecombTools.remove_branches!`, which collapses short branches into polytomies, with a guard so the root is never deleted; shared with `perf/simulate.jl`.

## Format notes

- Leaf sets are sorted string arrays. MCC lists are sorted by `(length, leaves)`. Split lists are sorted lexicographically.
- The trees in each pair are labelled `t1`, `t2`, and so on.
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

## Tests that use the fixtures

- `./dev/docker/run just test-rs` runs `packages/treeknit-io/tests/fixtures.rs`
  - It checks every deterministic function exactly against the fixtures: naive MCCs, K-tree resolution, strict and liberal resolution with MCCs, node-to-MCC maps, polytomy sorting, energies and likelihoods of given configurations, and ARGs
  - It compares the annealing outcomes of the two-tree cases with the 20 seeded runs of TreeKnit.jl, on the cases where all runs agree
  - It does not read the `multi_runs` of cases with more than two trees yet (`kb/issues/N-reference-comparison-gaps.md`)
- `./dev/docker/run just example accuracy [drop]`
  - It compares the accuracy of TreeKnit.jl and of the port against the true MCCs of the simulated cases, as scaled variation of information
  - It drops a fraction `drop` of the leaves of each tree (default 0.2) and reports how often the dropped leaves are placed with their true MCC

`kb/reports/julia-rust-equivalence.md` describes the results and the remaining differences.

## Comparison with TreeKnit.jl (`perf/`)

The scripts in `perf/` measure the runtime of TreeKnit.jl and compare its results with the port's. The findings are in `kb/reports/julia-rust-equivalence.md` and `kb/reports/performance.md`.

### Environment

TreeKnit.jl 0.5.8 (commit `dbbc89a`) with TreeTools 0.6.14 on Julia 1.13.1, in the official `julia:1.13.1-trixie` image, started from the repository root:

```sh
docker run --rm -it --volume="$PWD:$PWD" --workdir="$PWD" julia:1.13.1-trixie bash
```

In the container, install a C compiler, which TreeKnit.jl (its package build installs the command line with Comonicon) and PackageCompiler need, then the packages, and build a system image, which removes the compilation time from every run:

```sh
apt-get update && apt-get install --yes --no-install-recommends build-essential time
export JULIA_PROJECT=/opt/tkperf
julia -e 'using Pkg;
  Pkg.add(url="https://github.com/PierreBarrat/TreeKnit.jl", rev="dbbc89ac691fed0949a622eedbae103787b89320");
  Pkg.add(name="TreeTools", version="0.6.14");
  Pkg.add(url="https://github.com/PierreBarrat/ARGTools", rev="824b371cd0a2fd79fe80d4848e3445b3e6718686");
  Pkg.add(["JSON3", "Distributions", "PackageCompiler"])'
julia -e 'using PackageCompiler; create_sysimage(["TreeKnit"]; sysimage_path="/opt/tkperf/treeknit.so",
  precompile_execution_file="ref/perf/precompile.jl", cpu_target="native")'
alias jl='julia --sysimage=/opt/tkperf/treeknit.so --startup-file=no'
```

Keep the default optimization level. `-O3` makes Julia reject the precompiled package images, which adds about 50 s to loading TreeKnit without the system image, and it does not change the speed of warm runs.

The port: `./dev/docker/run just build prod` writes `.out/treeknit`.

### Commands

Output goes to `tmp/perf/` in these examples.

- Seeded runs of the TreeKnit.jl command line, with in-process times: `jl ref/perf/seeds.jl tmp/perf/jl 1 100 data/h3n2-2017/ha.nwk data/h3n2-2017/na.nwk [-- OPTION...]`
- Seeded runs with options that the TreeKnit.jl command line ignores (`--no-likelihood`, `--no-pre-resolve`, `--better-MCCs`): `jl ref/perf/api_seeds.jl tmp/perf/jl 1 300 A.nwk B.nwk -- --no-pre-resolve --no-likelihood`
- Seeded runs of the port, with wall-clock times: `./dev/docker/run bash -c 'for s in $(seq 1 100); do .out/treeknit data/h3n2-2017/ha.nwk data/h3n2-2017/na.nwk --better-MCCs --threads 1 --seed "$s" --verbosity-level -1 -o tmp/perf/rs/s"$s"; done'`. Give the port the method flag that TreeKnit.jl selects by default: `--better-MCCs` for two trees, `--better-trees` for more
- Distributions of the two sets of runs: `jl ref/perf/compare.jl tmp/perf/jl tmp/perf/rs [NPERM] [--freq]`
- MCCs that are not compatible with the input trees: `jl ref/perf/check_mccs.jl data/h3n2-2017 tmp/perf/jl tmp/perf/rs`
- Splits of the resolved output trees of two runs: `jl ref/perf/splits.jl tmp/perf/jl/s1 tmp/perf/rs/s1`
- Random subsets of the four segment trees: `jl ref/perf/subset.jl tmp/perf/k4-n400 400 400 data/h3n2-2k-4-segments/{ha,na,pb1,pb2}.nwk`
- Simulated ARGs with true MCCs, then accuracy of each set of runs stored as `tmp/perf/runs/<simulation>/<arm>/s<seed>/`: `jl ref/perf/simulate.jl tmp/perf/sim`, then `jl ref/perf/accuracy.jl tmp/perf/sim tmp/perf/runs jl rs`
- Cost of one energy computation: `jl ref/perf/energy.jl A.nwk B.nwk` for TreeKnit.jl, and `./dev/docker/run just example energy_cost A.nwk B.nwk` for the port, which also times the incremental update
- Wall-clock time and peak memory of one command: `/usr/bin/time -v` (Debian package `time`)
