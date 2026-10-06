# Performance of the port compared with TreeKnit.jl

This report measures the runtime and memory of the port and of TreeKnit.jl 0.5.8 on the same inputs with the same options, explains the difference from the code and from profiles, describes how both scale with the number of leaves, and measures changes to TreeKnit.jl proposed upstream that keep its results. The results of both implementations are compared in [`julia-rust-equivalence.md`](julia-rust-equivalence.md); the scripts and commands are in [`ref/README.md`](../../ref/README.md), section "Comparison with TreeKnit.jl".

## Summary

- **Speed**: with one thread, the port is 7 to 9 times faster than TreeKnit.jl on trees of 53 to 154 leaves, 26 times faster on 428 leaves, and about 230 times faster on four segment trees of 1997 leaves (34 s against 2.1 hours). TreeKnit.jl times exclude start-up and compilation
- **Cause**: TreeKnit.jl recomputes the topological energy over all leaves at every annealing step and stores splits as sorted integer arrays. The port updates the energy for the leaves a flip can affect and stores clades as bit sets. On one tree pair of 1997 leaves, a full energy computation takes 2.0 to 2.1 ms in TreeKnit.jl and 0.15 ms in the port, and one incremental update in the port takes 15 to 17 µs
- **Scaling**: on subsets of the same four segment trees, the time grows as about n^2.75 in TreeKnit.jl and n^1.65 in the port. The published estimate for TreeKnit is quadratic
- **Start-up**: a cold TreeKnit.jl command takes 10 to 12 s on 53 leaves, almost all compilation, or 0.4 to 0.7 s with a precompiled <a id="gloss-use-1"></a>system image <sup>[1](#gloss-1)</sup>; the port takes 0.01 s
- **Memory**: 470 MB peak for TreeKnit.jl on 1997 leaves, of which about 400 MB is the Julia runtime and system image, against 19 MB for the port
- **Parallel pairs**: with all cores, the port infers the six pairs of four segments at the same time and takes 7 to 11 s instead of 34 s on 1997 leaves, with the same results for each seed
- **Proposed TreeKnit.jl changes**: five draft pull requests to TreeKnit.jl ([#43](https://github.com/PierreBarrat/TreeKnit.jl/pull/43) to [#47](https://github.com/PierreBarrat/TreeKnit.jl/pull/47)) bring the bit sets and the incremental energy of the port to TreeKnit.jl, with byte-identical results for each seed. On the four segment trees of 1997 leaves, a run takes 18 to 19 s instead of 2.1 hours, faster than the port with one thread

## Method

### Builds

- **TreeKnit.jl**: version 0.5.8 (commit [`dbbc89a`](https://github.com/PierreBarrat/TreeKnit.jl/commit/dbbc89ac691fed0949a622eedbae103787b89320)) with TreeTools 0.6.14 on Julia 1.13.1, in the official `julia:1.13.1-trixie` container image. A PackageCompiler system image, built with `cpu_target="native"` from a workload that runs the command line on two small fixtures (`ref/perf/precompile.jl`), holds the compiled code. Runs use one thread
- **Port**: the `prod` build (`just build prod`, cargo profile `dist`), run in the project container (`./dev/docker/run`) with `--threads 1` unless stated otherwise. The algorithm code measured is that of commit `7630754`

### Options

Both implementations run the TreeKnit.jl defaults: `--better-MCCs` for two trees and `--better-trees` for more; TreeKnit.jl selects them without a flag, the port gets the flag. The `parameters.json` files of both record γ = 2, 50 annealing steps per leaf, geometric cooling from 1 to 0.05 over 101 temperatures, the likelihood test, and sequence lengths 1. Both implementations therefore run the same number of annealing steps: on the first pair of the 1997-leaf data, 989 steps per temperature in each pass of the prune-and-repeat loop.

### Measurements

- **TreeKnit.jl run time**: `ref/perf/seeds.jl` calls the command-line entry point `TreeKnit.command_main` once per seed in one Julia process and reports the in-process time with `@elapsed`, so start-up and compilation are excluded. Each run starts with `Random.seed!(seed)`
- **Port run time**: wall-clock time of the whole command, including process start-up and file output
- **Start-up**: wall-clock time of a cold `julia -e 'using TreeKnit; TreeKnit.command_main(...)'` with and without the system image, and of the port's command
- **Peak memory**: `/usr/bin/time -v` (maximum resident set size)
- **Energy cost**: `ref/perf/energy.jl` for TreeKnit.jl and `just example energy_cost` for the port, on the split graph of the naive MCCs of one tree pair; the port's example also times incremental updates in Metropolis chains at four temperatures
- **Profile**: Julia's sampling profiler (`Profile`) on one warm TreeKnit.jl run
- **Conditions**: all timings come from one machine that ran other work at the same time. Repeated runs of one configuration varied by about 20%, so ratios are more reliable than absolute times. Several TreeKnit.jl configurations ran in parallel containers, one thread each

### Data

- **Real trees**: the H3N2 tree pairs in `data/` (53, 100, 154, and 428 leaves), and the four segments of `data/h3n2-2k-4-segments` (1997 leaves)
- **Subsets**: random subsets of 150 and 400 leaves of the four segments, pruned from each tree (`ref/perf/subset.jl`)
- **Simulated ARGs**: two trees with 200 and 500 leaves and four trees with 200 leaves, with binary trees and with short branches collapsed into polytomies (`ref/perf/simulate.jl`, the ARGTools simulator of the TreeKnit paper)

## Results

### Run time

| Data                      | Trees | Leaves | Runs (TreeKnit.jl / port) | TreeKnit.jl    | Port    | Ratio |
| ------------------------- | ----- | ------ | ------------------------- | -------------- | ------- | ----- |
| `h3n2-2017-2018`          | 2     | 53     | 100 / 100                 | 0.058 s        | 0.008 s | 7     |
| `h3n2-2017`               | 2     | 100    | 100 / 100                 | 0.14 s         | 0.016 s | 9     |
| `h3n2-new-york-1999-2004` | 2     | 154    | 100 / 100                 | 0.20 s         | 0.021 s | 9     |
| `h3n2-2012-2018`          | 2     | 428    | 30 / 30                   | 4.3 s          | 0.16 s  | 26    |
| 4-segment subset          | 4     | 150    | 50 / 50                   | 6.1 s          | 0.46 s  | 13    |
| 4-segment subset          | 4     | 400    | 20 / 20                   | 93 s           | 2.2 s   | 42    |
| `h3n2-2k-4-segments`      | 4     | 1997   | 2 / 5                     | 7600 s (2.1 h) | 34 s    | 230   |

Means over the runs. On 53 leaves, the port's time is mostly process start-up and file output.

On the 1997-leaf data, each of the six tree pairs took 9 to 36 minutes in TreeKnit.jl and 4 to 6 s in the port. The two TreeKnit.jl runs took 8165 s and 7104 s; the five port runs took 29 to 38 s.

| Simulated setting (3 ARGs, 10 runs each) | TreeKnit.jl | Port    |
| ---------------------------------------- | ----------- | ------- |
| 2 trees, 200 leaves, binary              | 0.42 s      | 0.043 s |
| 2 trees, 200 leaves, polytomies          | 0.56 s      | 0.057 s |
| 2 trees, 500 leaves, binary              | 2.4 s       | 0.145 s |
| 2 trees, 500 leaves, polytomies          | 5.1 s       | 0.22 s  |
| 4 trees, 200 leaves, binary              | 0.50 s      | 0.16 s  |
| 4 trees, 200 leaves, polytomies          | 1.1 s       | 0.28 s  |

Averages over the reassortment rates ρ = 0.05, 0.1, and 0.2 (two trees) and 0.05 and 0.1 (four trees). The TreeKnit.jl runs on simulations shared the machine with seven other TreeKnit.jl containers.

### Where the time goes

Both implementations run the same annealing: the same number of steps, the same single-leaf flips, and the same Metropolis rule. The cost of one step differs.

- **TreeKnit.jl**: `mcmcstep!` flips one leaf and calls `compute_energy` (`src/SplitGraph/energy.jl`), which climbs from every kept leaf to its first ancestor with two kept leaves and compares the splits of the two trees. Splits are sorted `Vector{Int}`; membership tests use `in` for short splits and binary search (`insorted`) for splits longer than 25
- **Port**: `EnergyState` ([splitgraph.rs#L203-L357](../../packages/treeknit-core/src/splitgraph.rs#L203-L357)) keeps the number of kept leaves below every node. A flip recomputes the term of the flipped leaf and of the kept leaves whose first non-trivial ancestor is an ancestor of the flipped leaf in some tree, before and after the flip. Clades are `FixedBitSet`s, and the restricted comparisons (equal, subset, disjoint on the kept leaves) work on 64-bit words ([bits.rs](../../packages/treeknit-core/src/bits.rs))

**Profile of TreeKnit.jl** (428 leaves, `--better-MCCs`, one warm run, 2999 samples of the main thread):

- **Annealing**: 88% of the samples are in `_sa_opt`
- **Full energy**: 77% are in `compute_energy`, called from `mcmcstep!`
- **Sorted-array lookups**: 49% are in `insorted` and `searchsorted`, the binary searches inside the split comparisons
- **Rest**: naive MCCs, pruning, resolution, and the likelihood tie-break take the remaining 12%

**Energy cost on one tree pair** (`ha` and `na` of the 1997-leaf data, 1773 naive MCCs):

| Operation                             | Without resolution | With resolution |
| ------------------------------------- | ------------------ | --------------- |
| TreeKnit.jl, full computation         | 2.0 to 2.1 ms      | 5.2 to 6.0 ms   |
| Port, full computation                | 0.15 ms            | 0.48 ms         |
| Port, incremental update after a flip | 15 to 17 µs        | 71 to 79 µs     |

The bit sets make the full computation 11 to 14 times faster, and the incremental update is another 6 to 11 times faster than the full computation. Per annealing step, the port is about 130 times faster without resolution and about 75 times faster with resolution. The incremental energy equals the full computation after every one of 400 000 flips per resolution setting in that example.

On the 1997-leaf data the whole run is faster by about 230, more than the energy step alone. The steps outside the energy computation also cost more in TreeKnit.jl: each step sums the configuration vector to count removed leaves, and every configuration of minimal free energy is checked against a list with `in(_conf, oconf)` and copied, where the port uses a hash set. These were not timed separately.

### Scaling with the number of leaves

| Data                              | Leaves     | Exponent, TreeKnit.jl | Exponent, port |
| --------------------------------- | ---------- | --------------------- | -------------- |
| Subsets of the four segment trees | 150 to 400  | 2.8                   | 1.6            |
| Subsets of the four segment trees | 400 to 1997 | 2.7                   | 1.7            |
| Simulated pairs, binary           | 200 to 500  | 1.9                   | 1.3            |
| Simulated pairs, polytomies       | 200 to 500  | 2.4                   | 1.5            |

The exponent is log(t₂/t₁)/log(n₂/n₁) between the mean times at the two sizes.

- **Published estimate**: the TreeKnit paper derives quadratic runtime, an O(L) energy evaluation times O(L) annealing steps, and verifies it on simulated trees <a id="cite-1"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]. TreeKnit.jl matches this on simulated binary trees (1.9) and exceeds it on the influenza trees (2.7 to 2.8) and on trees with polytomies (2.4)
- **Why steeper on real trees**: the energy climbs from every kept leaf to its first non-trivial ancestor, so its cost grows with the depth of the trees as well as with the number of leaves; influenza trees are deep and ladder-like. The binary searches add a logarithmic factor, and data with more reassortment need more prune-and-repeat iterations
- **Port**: an incremental update visits the ancestors of the flipped leaf and the children of each, so its cost grows with depth and branching, not with the number of leaves. The total grows as the number of steps times that cost, between n^1.3 and n^1.7 here

### Start-up and compilation

Command on the 53-leaf pair, three runs each:

| Command                                                | Wall-clock time  | Peak memory  |
| ------------------------------------------------------ | ---------------- | ------------ |
| TreeKnit.jl with the system image                      | 0.4 to 0.7 s     | 410 MB       |
| TreeKnit.jl without system image, default optimization | 10 to 12 s       | 600 MB       |
| TreeKnit.jl without system image, `-O3`                | 63 to 69 s       | 1.46 GB      |
| Port                                                   | 0.007 to 0.011 s | not measured |

- **Package images**: loading TreeKnit takes 1.4 s at the default optimization level and 52 s with `-O3`, because Julia rejects the package images precompiled at the default level and compiles again. The rest of a cold run without the system image is compilation of the inference code
- **Optimization level**: warm runs on 428 leaves take 3.5 s with the `-O3` system image and 3.4 to 3.5 s at the default level; `-O3` brings no speed-up
- **System image**: the first run in a process with the system image still took 4.4 s against 3.6 s for later runs on 428 leaves, because the workload does not cover every compiled path

### Memory

On the 1997-leaf data, the peak resident memory was 470 MB for TreeKnit.jl and 19 MB for the port. TreeKnit.jl with the system image already uses about 410 MB on 53 leaves, so most of its memory is the Julia runtime and the system image.

### Parallel pairs

With all cores (`--threads 0`, the default), the port infers the independent pairs of an unresolved round in parallel. On the 1997-leaf data with four segments, a run took 7 to 11 s instead of 29 to 38 s with one thread, and wrote byte-identical `MCCs.json` files for each seed. The six pairs take 4 to 6 s each, so the longest pair, the pre-resolution, and file output bound the parallel run. TreeKnit.jl's command line runs the pairs one after another; its `--parallel` flag has no effect (`kb/feat/v0/documented-vs-actual.md`).

## Changes proposed to TreeKnit.jl

Five draft pull requests to TreeKnit.jl apply the causes found above to its own code. Each keeps the random-number calls and the energy of every step, so a run with the same seed writes byte-identical files. They form a stack: each builds on the previous one.

- **[#43](https://github.com/PierreBarrat/TreeKnit.jl/pull/43)**: bit sets of the leaves of every split instead of sorted-array searches, a count of removed leaves in the annealing step, one MCC map per resolution instead of one per split, and log messages filtered before they are built
- **[#44](https://github.com/PierreBarrat/TreeKnit.jl/pull/44)**: incremental energy, with the number of kept leaves below every node and the term of every leaf, as in the port; the comparisons first compare the numbers of kept leaves, and a per-leaf stamp removes duplicate affected leaves without sorting
- **[#45](https://github.com/PierreBarrat/TreeKnit.jl/pull/45)**: no search of the list of minimal configurations after a rejected step, when the configuration is unchanged
- **[#46](https://github.com/PierreBarrat/TreeKnit.jl/pull/46)**: the likelihood tie-break with the same layout and counts, and branch lengths summed only up to the known ancestor instead of through `TreeTools.distance`, which walks both nodes to the root
- **[#47](https://github.com/PierreBarrat/TreeKnit.jl/pull/47)**: leaves placed in the bit sets in depth-first order of the first tree, so that the comparisons read only the few words where a clade has leaves; leaf indices follow `Dict` key order and spread even small clades over all words

### Results

Times with one thread, measured as in [Method](#method) on the same machine; the port's times are those of [Run time](#run-time).

| Data                      | Trees | Leaves | TreeKnit.jl 0.5.8 | With #43        | With #43 to #47 | Port         |
| ------------------------- | ----- | ------ | ----------------- | --------------- | --------------- | ------------ |
| `h3n2-2012-2018`          | 2     | 428    | 4.8 to 5.5 s      | 0.5 to 1.1 s    | 0.22 to 0.30 s  | 0.16 s       |
| 4-segment subset          | 4     | 150    | 6.3 to 7.2 s      | 0.8 to 1.5 s    | 0.26 to 0.28 s  | 0.46 s       |
| `h3n2-2k-4-segments`      | 4     | 1997   | 7104 and 8165 s   | 467 and 501 s   | 18.4 and 19.2 s | 29 to 38 s   |

The 428 and 150-leaf rows are 5 seeds each and the 1997-leaf row seeds 1 and 2; every output file (`MCCs.json`, resolved trees, `ARG/`, `parameters.json`) was byte-identical to that of TreeKnit.jl 0.5.8 for the same seed.

| One annealing step on 1773 leaves              | Without resolution | With resolution |
| ---------------------------------------------- | ------------------ | --------------- |
| TreeKnit.jl 0.5.8, full computation            | 2.0 to 2.7 ms      | 5.2 to 6.6 ms   |
| With #43, full computation                     | 0.23 ms            | 0.5 to 1.1 ms   |
| With #43 to #47, incremental update            | 4.5 to 5.7 µs      | 11 to 12 µs     |
| Port, incremental update                       | 15 to 17 µs        | 71 to 79 µs     |

- **Where the time goes now**: on the 1997-leaf data, the annealing takes about 79% of a run, the likelihood tie-break 7%, resolution and polytomy sorting 5%, and writing the Newick files 2%. About 150 leaves are affected per flip, at a depth of about 32 nodes
- **Hardware counters**: `perf stat` on the annealing gave 2.2 instructions per cycle and almost no last-level cache misses, but about 1300 first-level cache misses per step before [#47](https://github.com/PierreBarrat/TreeKnit.jl/pull/47), from comparisons over all words of the clades
- **Measured without gain**: storing the children of the nodes in flat arrays instead of one vector per node changed the step time by about 3%, within the noise

## Discussion

- **The speed-up comes from the algorithm, not from compromises in the computation**: both implementations evaluate the same energy at every step of the same chain; [`julia-rust-equivalence.md`](julia-rust-equivalence.md) compares the results
- **Practical consequence**: four segments of 2000 leaves take two hours per run in TreeKnit.jl and half a minute in the port, or about ten seconds with all cores, so repeated runs over seeds and parameter scans become practical. The web app runs the same core in WebAssembly with one thread; its speed was not measured here
- **TreeKnit.jl gains the whole difference**: with the incremental energy and bit sets of [#43](https://github.com/PierreBarrat/TreeKnit.jl/pull/43) to [#47](https://github.com/PierreBarrat/TreeKnit.jl/pull/47), TreeKnit.jl is faster than the port with one thread on four segment trees, and slower on small pairs of trees, where resolution, the ARG and file output dominate. The port remains faster with all cores, because it infers the pairs in parallel
- **The port's incremental step is 3 to 7 times slower than that of the changed TreeKnit.jl**: the port allocates the candidate, undo and ancestor vectors in every flip, sorts the candidates to remove duplicates, compares clades without first comparing the numbers of kept leaves, and reads all words of both clades ([splitgraph.rs](../../packages/treeknit-core/src/splitgraph.rs), [bits.rs](../../packages/treeknit-core/src/bits.rs)). TreeKnit.jl avoids each of these
- **Start-up dominates small TreeKnit.jl runs**: without a system image, a run on 53 leaves takes 10 to 12 s, of which the inference is 0.06 s. A system image, or a long-running Julia session as the TreeKnit.jl documentation recommends, removes this cost

## Limitations

- **One machine, shared**: absolute times vary by about 20%; ratios and exponents are the robust results
- **Two sizes per exponent**: the exponents come from two sizes each and from datasets whose reassortment content also varies with the subset; they describe these data, not a general law
- **Not measured**: the web app and the port's memory on small inputs

## Glossary

1. <a id="gloss-1"></a> **System image.** A file with the compiled code of Julia packages, built ahead of time with PackageCompiler, so that a Julia process starts without compiling them. [↩](#gloss-use-1)

## References

1. <a id="ref-1"></a> Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2022. "TreeKnit: Inferring ancestral reassortment graphs of influenza viruses." _PLOS Computational Biology_ 18:e1010394. https://doi.org/10.1371/journal.pcbi.1010394 [↩](#cite-1)
