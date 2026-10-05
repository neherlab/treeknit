# Equivalence of the port with TreeKnit.jl: runtime, results, and accuracy

This report compares the port with TreeKnit.jl 0.5.8, the implementation that accompanies the published method <a id="cite-1a"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)], on real influenza trees and on simulated ARGs with known MCCs. It asks three questions: why the port is faster, whether it gives the same results, and whether it is as accurate. The comparison of the paper with TreeKnit.jl itself is in [`treeknit-paper-vs-code.md`](treeknit-paper-vs-code.md).

## Summary

- **Same results**: with the TreeKnit.jl options, the port gives <a id="gloss-use-1"></a>MCC <sup>[1](#gloss-1)</sup> sets whose distribution matches TreeKnit.jl on every dataset tested, for two and four trees, within the power of the tests. Of 58 distribution tests, two fall below 0.05 without correction, fewer than chance predicts, and on simulated ARGs the two implementations have the same accuracy within a 95% confidence interval of ±0.001 in scaled <a id="gloss-use-2"></a>variation of information <sup>[2](#gloss-2)</sup>
- **Same algorithm**: the score, the annealing schedule, the number of steps, the outer prune-and-repeat loop, and the tie-break are the same. The pre-resolved trees of both implementations have identical splits
- **Faster for two reasons**: the port updates the energy incrementally after each single-leaf flip, where TreeKnit.jl recomputes it over all leaves at every step, and it stores clades as bit sets, where TreeKnit.jl tests membership in sorted integer arrays. The gain grows with the number of leaves, from 7 times on 53 leaves to about 230 times on 1997 leaves with four segments
- **Defects found**: two inputs that TreeKnit.jl handles or rejects crash the port: trees with unary nodes ([`H-naive-mccs-overlap-with-unary-nodes.md`](../issues/H-naive-mccs-overlap-with-unary-nodes.md)) and negative branch lengths ([`H-non-finite-likelihood-panics.md`](../issues/H-non-finite-likelihood-panics.md)). None of the datasets tested has either
- **Port default**: the default of the port (`--resolve matched`, the web app setting) gives the same MCCs as `--better-MCCs` for two trees, and for four trees with polytomies it is markedly more accurate than the TreeKnit.jl default `--better-trees`

## Setup

- **TreeKnit.jl**: version 0.5.8 at commit [`dbbc89a`](https://github.com/PierreBarrat/TreeKnit.jl/commit/dbbc89ac691fed0949a622eedbae103787b89320) with TreeTools 0.6.14 on Julia 1.13.1 (official `julia:1.13.1-trixie` image), compiled ahead of time into a PackageCompiler system image with `cpu_target="native"` and run with `-O3`, one thread. Runs call the command-line entry point `TreeKnit.command_main` in a running process, so the times exclude start-up and compilation. TreeKnit.jl has no seed option; each run calls `Random.seed!(s)` first
- **Port**: the `prod` build (cargo profile `dist`) at commit `7630754`, run through the project container with `--threads 1` unless noted. The algorithm code is unchanged at `3d2bf85`
- **Options**: TreeKnit.jl with no method flag, which selects `--better-MCCs` for two trees and `--better-trees` for more; the port with the same flag given explicitly. The `parameters.json` of both runs record γ = 2, 50 steps per leaf, geometric cooling from 1 to 0.05 over 101 temperatures, the likelihood test, and sequence lengths 1. The TreeKnit.jl command line ignores `--gamma`, `--n-mcmc-it`, `--no-likelihood`, and `--seq-lengths`, so these values hold for every TreeKnit.jl run (`kb/feat/v0/documented-vs-actual.md`)
- **Machine**: Intel Core i9-10900K, 20 hardware threads, shared with other work during the measurements (load average 6 to 18), so times are indicative to about ±20%
- **Data**: the H3N2 trees in `data/`, all with identical leaf sets in each dataset, no unary nodes, and no negative branch lengths; random subsets of 150 and 400 leaves of the four segments of `data/h3n2-2k-4-segments`, made by pruning the same leaves from each tree; and simulated ARGs (see [Accuracy on simulated ARGs](#accuracy-on-simulated-args))

## Why the port is faster

Both implementations run the same number of annealing steps. On the first pair of `data/h3n2-2k-4-segments` (`ha`, `na`, 1997 leaves after pre-resolution, 1445 naive MCCs), both start each annealing pass with 989 steps per temperature over 101 temperatures, and both need five or six prune-and-repeat iterations. The difference is the cost of one step.

- **Incremental energy**: TreeKnit.jl `mcmcstep!` flips one leaf and calls `compute_energy`, which climbs from every kept leaf to its first ancestor with two kept leaves and compares the clades of all tree pairs (`src/SplitGraph/energy.jl`). The port keeps the count of kept leaves below every node and recomputes only the leaves whose first non-trivial ancestor is an ancestor of the flipped leaf in some tree, before and after the flip (`EnergyState` in [splitgraph.rs#L203-L357](../../packages/treeknit-core/src/splitgraph.rs#L203-L357))
- **Bit sets**: on the same graph (1773 naive MCCs of `ha` and `na`), one full energy computation takes 2.1 ms in TreeKnit.jl and 0.16 ms in the port without resolution (5.2 ms and 0.48 ms with resolution). One incremental step of the port takes about 10 µs
- **Check of the incremental energy**: a Metropolis chain on that graph with 100 000 steps at each of the temperatures 1, 0.3, 0.1, and 0.05, with and without resolution, compared the incremental energy with the full computation after every step and found no difference. The energies of the full configuration agree between the implementations (1652 without resolution, 988 with resolution). The unit test `incremental_energy_matches_full` covers only a 12-leaf example

| Data                      | Trees | Leaves | TreeKnit.jl    | Port    | Ratio |
| ------------------------- | ----- | ------ | -------------- | ------- | ----- |
| `h3n2-2017-2018`          | 2     | 53     | 0.058 s        | 0.008 s | 7     |
| `h3n2-2017`               | 2     | 100    | 0.14 s         | 0.016 s | 9     |
| `h3n2-new-york-1999-2004` | 2     | 154    | 0.20 s         | 0.021 s | 9     |
| `h3n2-2012-2018`          | 2     | 428    | 4.3 s          | 0.16 s  | 26    |
| 4-segment subset          | 4     | 150    | 6.1 s          | 0.46 s  | 13    |
| 4-segment subset          | 4     | 400    | 93 s           | 2.2 s   | 42    |
| `h3n2-2k-4-segments`      | 4     | 1997   | 7600 s (2.1 h) | 34 s    | 230   |

TreeKnit.jl times are in-process means over the seeds; port times are wall-clock times of the whole command. With all cores (`--threads 0`, the default), the port runs the six independent pairs of the four-segment data in parallel and takes 7 to 11 s on 1997 leaves, with byte-identical `MCCs.json` for each seed.

## Same results on real trees

The annealing is random, so single runs differ between seeds in both implementations. The comparison is between the distributions of results over seeds. For each tree pair:

- **MCC count**: the difference of the mean counts, with a permutation p-value
- **Partitions**: the scaled variation of information (VI divided by ln n) between every two runs, within TreeKnit.jl, within the port, and across. The <a id="gloss-use-3"></a>energy distance <sup>[3](#gloss-3)</sup> <a id="cite-2"></a>[Székely and Rizzo 2013](https://doi.org/10.1016/j.jspi.2013.03.018) [[2](#ref-2)] on these distances tests that both sets of runs come from the same distribution, with a permutation p-value; VI is the metric of <a id="cite-3"></a>[Meilă 2007](https://doi.org/10.1016/j.jmva.2006.11.013) [[3](#ref-3)]
- **Validity**: every MCC, restricted to its leaves, must have pairwise compatible splits in both input trees, so that its topology is the same up to resolution. A check that uses the code of neither implementation found no invalid MCC in any run of either implementation; it flags an MCC set made of one clade of all leaves of two different trees

| Data, method                 | Runs per side | Mean MCC count (TreeKnit.jl / port) | p    | Scaled VI within TreeKnit.jl / port / across | Energy test p |
| ---------------------------- | ------------- | ----------------------------------- | ---- | -------------------------------------------- | ------------- |
| 53 leaves, `--better-MCCs`   | 100           | 2.00 / 2.00                         | 1.00 | 0.025 / 0.024 / 0.024                        | 1.00          |
| 100 leaves, `--better-MCCs`  | 500           | 15.10 / 15.12                       | 0.47 | 0.036 / 0.040 / 0.038                        | 0.62          |
| 154 leaves, `--better-MCCs`  | 100           | 7.00 / 7.00                         | 1.00 | 0 / 0 / 0                                    | 1.00          |
| 154 leaves, `--better-trees` | 500           | 29.38 / 29.39                       | 0.91 | 0.020 / 0.019 / 0.019                        | 0.94          |
| 428 leaves, `--better-MCCs`  | 30            | 35.83 / 35.67                       | 0.54 | 0.073 / 0.074 / 0.073                        | 0.73          |

- **Same solutions with the same frequencies**: on 154 leaves with `--better-trees`, both implementations find the same three partitions, with 31, 14, and 13 MCCs, in 453, 38, and 9 of 500 TreeKnit.jl runs and 453, 41, and 5 of 500 port runs (plus one port run with a fourth partition). On 100 leaves, the two most frequent partitions occur 259 and 191 times in TreeKnit.jl and 253 and 190 times in the port. A first sample of 100 runs per side showed a split of 46/43 against 58/31 (p about 0.07); the larger sample does not reproduce it
- **Four segments, 150 leaves**: 50 runs per side with `--better-trees` and with `--better-MCCs`. For each of the six tree pairs, mean MCC counts differ by at most 1.3 (p at least 0.09), and the scaled VI across implementations differs from the mean of the two within-implementation values by at most 0.001 in all twelve comparisons (energy test p from 0.11 to 0.94). With `--better-MCCs` every run gives a different partition in both implementations, and the run-to-run variation is high in both (scaled VI 0.06 to 0.20 within each implementation, against 0.007 to 0.021 for `--better-trees`)
- **Four segments, 400 leaves**: 102 TreeKnit.jl runs against 500 port runs with `--better-trees`. Five of the six pairs agree (mean count p from 0.07 to 0.97, energy test p from 0.08 to 0.91). For `pb1`-`pb2`, most runs of both implementations end with 205 to 208 MCCs in nearly the same proportions (205: 57% and 55% in the first 60 TreeKnit.jl runs and the port runs), but a second state with 217 to 224 MCCs occurs in 7 of 102 TreeKnit.jl runs (6.9%) and in 13 of 500 port runs (2.6%). This raises the TreeKnit.jl mean by 0.7 MCCs (p = 0.03, Fisher exact p = 0.06 for the frequency of the second state; energy test p = 0.21). Corrected for the six pairs of this dataset, the difference is not significant, but it is the largest deviation in this report and is listed under [Open questions](#open-questions)
- **Four segments, 1997 leaves**: two TreeKnit.jl runs (seeds 1 and 2, 2.3 and 2.0 hours) against five port runs (seeds 1 to 5). For each pair, both TreeKnit.jl counts lie within the range of the port counts or within 7 of it (for example `ha`-`na`: 1263 and 1273 against 1265 to 1284; mean differences up to 8 MCCs of about 1270, p from 0.24 to 0.76), and the scaled VI across implementations (0.005 to 0.009) is of the same size as within each (0.003 to 0.010). With two runs on one side the permutation test has only 21 distinct labelings, so its smallest p is 0.048; it reached that value for one of the six pairs, which is expected by chance in about one of four such sets of six tests. The pre-resolved trees have identical splits (866, 828, 861, and 852 splits), and no MCC is invalid. Peak memory: 470 MB for TreeKnit.jl, including the Julia runtime and system image, and 19 MB for the port

The tables and items above make 29 pair comparisons, each with a count test and a partition test. Two of the 58 tests fall below 0.05 without correction (the `pb1`-`pb2` count on 400 leaves, p = 0.03, and one partition test on 1997 leaves at its smallest possible value, p = 0.048), where about three are expected by chance.

### Order of the pairs for more than two trees

In TreeKnit.jl, the unresolved rounds of more than two trees run the pairs one after another, and each pair sorts the child order of its trees before the next pair is inferred. The port infers the unresolved pairs in parallel on the trees as pre-resolved and sorts afterwards. Child order changes which leaf a random index selects, not the distribution of moves, and the four-segment comparisons above show no difference in the result distributions.

## Accuracy on simulated ARGs

ARGs were simulated with ARGTools (commit [`824b371`](https://github.com/PierreBarrat/ARGTools/commit/824b371cd0a2fd79fe80d4848e3445b3e6718686)), the simulator of the TreeKnit paper, with the same Kingman model, population size 10 000, and polytomy model as `ref/simulate.jl`: two trees with 200 and 500 leaves and reassortment rates ρ = 0.05, 0.1, 0.2, and four trees with 200 leaves and ρ = 0.05, 0.1; each with fully resolved trees and with short branches collapsed into polytomies (c = 0.1); three ARGs per setting. The true MCCs of each pair come from `ARGTools.MCCs_from_arg`. Each implementation ran ten seeds per ARG. Error measures per pair: the scaled VI between inferred and true MCCs, and the difference of the MCC counts.

Over the 108 (ARG, tree pair) units, the port with the TreeKnit.jl options against TreeKnit.jl:

- **Scaled VI to the truth**: mean difference -0.0002, 95% confidence interval [-0.0009, +0.0005], sign-flip permutation p = 0.57; the port is better on 39 units and worse on 35. The scaled VI itself ranges from 0.01 to 0.21
- **Absolute error of the MCC count**: mean difference -0.06 MCCs, 95% confidence interval [-0.25, +0.13], p = 0.56

The units of one four-tree ARG share trees, so the intervals are somewhat narrower than they would be for independent units.

| Setting                               | TreeKnit.jl VI / count error | Port, same options | Port default                   |
| ------------------------------------- | ---------------------------- | ------------------ | ------------------------------ |
| 2 trees, 200 leaves, binary (3 rates) | 0.033 / +0.9                 | 0.033 / +1.0       | same as port with same options |
| 2 trees, 200 leaves, polytomies       | 0.076 / +1.2                 | 0.080 / +0.3       | same                           |
| 2 trees, 500 leaves, binary           | 0.017 / +3.7                 | 0.016 / +3.7       | same                           |
| 2 trees, 500 leaves, polytomies       | 0.053 / +14.4                | 0.053 / +15.0      | same                           |
| 4 trees, 200 leaves, binary           | 0.070 / -2.6                 | 0.070 / -2.5       | same                           |
| 4 trees, 200 leaves, polytomies       | 0.187 / +31.5                | 0.186 / +31.2      | 0.104 / -0.5                   |

## Port default against the TreeKnit.jl default

The port's default (`--resolve matched` without pre-resolution, also the web app setting) is a deliberate difference from TreeKnit.jl (README, "Deliberate differences from TreeKnit.jl").

- **Two trees**: the default and `--better-MCCs` gave byte-identical `MCCs.json` in all 180 runs on simulated trees with polytomies. Only the resolved trees differ. The pairwise resolution inside inference equals the pre-resolution for two trees, and the matching step changed no MCC in these runs
- **Four trees**: on trees with polytomies the default is far more accurate than `--better-trees`, the TreeKnit.jl default for more than two trees: scaled VI 0.104 against 0.187, and MCC count error -0.5 against +31. `--better-trees` never resolves the trees with the inferred MCCs, so polytomies that the pre-resolution cannot remove remain as incompatibilities; the large excess of MCCs is consistent with this. On binary trees both give the same result
- **Over all 108 units**: the default is better than the TreeKnit.jl options by 0.028 in scaled VI (95% confidence interval [0.018, 0.037]) and by 9.7 MCCs in count error ([6.9, 12.5])

## Reference fixtures

The test `packages/treeknit-io/tests/fixtures.rs` compares the port with fixtures written by `ref/` from a TreeKnit.jl branch `fix/issues-from-rust-port` (commit `186bf0d`), which carries fixes made during the port. That branch is not in the TreeKnit.jl repository. Regenerating the deterministic fixture fields with the released 0.5.8 and comparing them with the committed fixtures shows that they differ only in:

- **Naive MCCs of three or more trees**: 0.5.8 fails with a `MethodError` in `is_coherent_clade`; the pairwise naive MCCs agree
- **Strict resolution with MCCs**: two simulated cases, where the port and the branch add a split that 0.5.8 rejects (README, "Deliberate differences from TreeKnit.jl")
- **Likelihood with missing branch lengths**: one case, where 0.5.8 gives `missing` and the branch gives 0 (README, "Deliberate differences from TreeKnit.jl")

The seeded runs of `run_treeknit!` for two trees, the pre-resolution, the energies, the Fitch maps, the leaf orders, and the ARGs agree with 0.5.8. The fixture test passes with 5169 checks. The annealing comparison in that test asserts only on cases where all TreeKnit.jl runs agree, and the `multi_runs` field is not read ([`N-reference-comparison-gaps.md`](../issues/N-reference-comparison-gaps.md)); the four-tree comparisons above fill that gap once, outside the test suite.

## Differences found in the code comparison

A comparison of the code of both implementations, function by function, found the inference identical except for these points:

- **Unary nodes**: the port's naive MCCs overlap and inference panics ([`H-naive-mccs-overlap-with-unary-nodes.md`](../issues/H-naive-mccs-overlap-with-unary-nodes.md))
- **Negative, infinite, or NaN branch lengths**: the port drops configurations with a NaN likelihood and panics when all are NaN; TreeKnit.jl stops with `DomainError` ([`H-non-finite-likelihood-panics.md`](../issues/H-non-finite-likelihood-panics.md))
- **Command-line options that TreeKnit.jl ignores**: the port applies `--gamma`, `--n-mcmc-it`, `--no-likelihood`, `--seq-lengths`, and an explicit `--rounds 1` ([`N-undocumented-differences-from-treeknit-jl.md`](../issues/N-undocumented-differences-from-treeknit-jl.md))
- **Order of the likelihood sum**: TreeKnit.jl sums the likelihood terms in the hash order of the graph labels, the port in MCC order. The test keeps only configurations of exactly maximal likelihood, so a rounding difference could turn an exact tie into a strict winner, or the reverse. This is not measured; the result distributions above show no effect
- **Random-number use**: TreeKnit.jl draws one unused random number per temperature, and the port draws one when choosing among a single configuration. This changes the random streams, not the distributions

Both implementations share the departures of TreeKnit.jl from the paper, for example the average of log-likelihood ratios where the paper multiplies the ratios ([`treeknit-paper-vs-code.md`](treeknit-paper-vs-code.md)).

## Open questions

- **Frequency of a rare state on 400 leaves**: for `pb1`-`pb2` of the 400-leaf subset, TreeKnit.jl reaches a state with 217 to 224 MCCs in 6.9% of runs and the port in 2.6% (Fisher exact p = 0.06). Both implementations reach the state, and the main states have the same frequencies. More TreeKnit.jl runs on this subset would decide whether the annealing of the port enters this state less often; a real difference would point to one of the code differences listed above, such as the order of the likelihood sum, which can change which tied configuration wins

## Limitations

- **TreeKnit.jl runs on 1997 leaves**: at two hours per run, only two TreeKnit.jl runs entered the comparison (a third run was stopped in its last pair). The comparison on 1997 leaves can show a gross difference, not a subtle one; the subsets of 150 and 400 leaves of the same trees carry the distribution test for four segments
- **Statistical power**: the standard errors allow shifts in the mean MCC count of about 0.05 to be detected on 100 leaves (500 runs), and of 2 to 5 MCCs on four segments with 50 runs, depending on the spread of the pair. Rare solutions that occur in fewer than 1% of runs may differ undetected
- **Untested paths**: liberal resolution, `--resolve-all-rounds`, `--rounds` above 2, ARG construction, and partial leaf overlap (which TreeKnit.jl does not support) were not part of the distribution comparisons

## Glossary

1. <a id="gloss-1"></a> **MCC (maximally compatible clade).** A largest set of leaves whose subtrees have the same topology in two segment trees, read as a region of the genealogy without reassortment <a id="cite-1b"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]. [↩](#gloss-use-1)
2. <a id="gloss-2"></a> **Variation of information (VI).** A metric between two partitions of the same set: H(A|B) + H(B|A), the conditional entropies of each partition given the other. Divided by ln n it lies between 0 (same partition) and 1 <a id="cite-3b"></a>[Meilă 2007](https://doi.org/10.1016/j.jmva.2006.11.013) [[3](#ref-3)]. [↩](#gloss-use-2)
3. <a id="gloss-3"></a> **Energy distance.** A two-sample statistic built from distances: twice the mean distance across the samples minus the mean distances within each sample. It is 0 in expectation when both samples come from the same distribution. [↩](#gloss-use-3)

## References

1. <a id="ref-1"></a> Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2022. "TreeKnit: Inferring ancestral reassortment graphs of influenza viruses." _PLOS Computational Biology_ 18:e1010394. https://doi.org/10.1371/journal.pcbi.1010394 [↩¹](#cite-1a) [↩²](#cite-1b)
2. <a id="ref-2"></a> Székely, Gábor J., and Maria L. Rizzo. 2013. "Energy statistics: A class of statistics based on distances." _Journal of Statistical Planning and Inference_ 143:1249-1272. https://doi.org/10.1016/j.jspi.2013.03.018 [↩](#cite-2)
3. <a id="ref-3"></a> Meilă, Marina. 2007. "Comparing clusterings: An information based distance." _Journal of Multivariate Analysis_ 98:873-895. https://doi.org/10.1016/j.jmva.2006.11.013 [↩¹](#cite-3) [↩²](#cite-3b)
