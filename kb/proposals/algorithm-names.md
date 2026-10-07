# Names and sources of the algorithms in the code

Most algorithms in the port are published algorithms or close variants of them, but the code and the knowledge base rarely say which. A reader who knows the name can look up correctness conditions, known limits, and faster versions; a reader who does not has to reverse-engineer them. This proposal lists the named algorithms found in the code and compares places to record them.

## Algorithms and their names

Each entry gives the code, the name, the source, and whether the code already states the name.

### Inference

- **Naive MCCs** (`pub fn naive_mccs()`, [packages/treeknit-core/src/naive.rs#L24-L56](../../packages/treeknit-core/src/naive.rs#L24-L56)): the subtree reduction of the kernels for <a id="gloss-use-1"></a>agreement forests <sup>[1](#gloss-1)</sup>, "Replace a maximal pendant subtree with at least two leaves that occurs identically in S and T by a single leaf" <a id="cite-1"></a>[Bordewich et al. 2007](https://doi.org/10.1177/117693430700300017) [[1](#ref-1)]. Not named in the code
- **Prune-and-repeat** (`pub fn infer_pair()`, [packages/treeknit-core/src/pair.rs#L34-L83](../../packages/treeknit-core/src/pair.rs#L34-L83)): the "prune maximal agreeing subtrees" and "grow agreeing subtrees" steps of the exact agreement-forest algorithm <a id="cite-2"></a>[Whidden et al. 2013](https://doi.org/10.1137/110845045) [[2](#ref-2)], with annealing in place of branching. Not named
- **Annealing** (`packages/treeknit-core/src/anneal.rs`): Metropolis acceptance <a id="cite-3"></a>[Metropolis et al. 1953](https://doi.org/10.1063/1.1699114) [[3](#ref-3)] with simulated annealing <a id="cite-4"></a>[Kirkpatrick et al. 1983](https://doi.org/10.1126/science.220.4598.671) [[4](#ref-4)]. Named. Convergence to global minima needs logarithmic cooling <a id="cite-5"></a>[Hajek 1988](https://doi.org/10.1287/moor.13.2.311) [[5](#ref-5)]; the geometric schedule has no such guarantee, which matches the TreeKnit S1 Text. Not stated in the code
- **Incremental energy** (`pub struct EnergyState`, [packages/treeknit-core/src/splitgraph.rs#L203-L357](../../packages/treeknit-core/src/splitgraph.rs#L203-L357)): delta evaluation of single-flip moves with undo. Described in the code
- **Tie-break likelihood** (`fn branch_likelihood()`, [packages/treeknit-core/src/splitgraph.rs#L537-L545](../../packages/treeknit-core/src/splitgraph.rs#L537-L545)): minus half the Poisson deviance, the G statistic of "shared branch" against "separate branches". The code calls it a log-ratio of Poisson likelihoods

The tie-break term for one pair of branches is

$$\ell = \sum_{s=1}^{2} \left( n_s - \mu_s + n_s \ln \frac{\mu_s}{n_s} \right) = \sum_{s=1}^{2} \left( \ln \mathrm{Pois}(n_s; \mu_s) - \ln \mathrm{Pois}(n_s; n_s) \right) = -\frac{D}{2}$$

where:

- $s$ -- segment of the pair
- $t_s$ -- length of the branch in the tree of segment $s$
- $L_s$ -- sequence length of segment $s$
- $n_s = t_s L_s$ -- expected number of mutations on the branch if the branches are separate
- $\mu_s = \bar{t} L_s$ with $\bar{t} = (t_1 L_1 + t_2 L_2) / (L_1 + L_2)$ -- expected number if the branch is shared
- $\mathrm{Pois}(n; \mu)$ -- Poisson probability of $n$ events with mean $\mu$; the $\ln n!$ terms cancel, so non-integer $n$ is allowed
- $D$ -- Poisson deviance of the shared model; the term $n_s \ln(\mu_s / n_s)$ is taken as 0 when $n_s = 0$, its limit

### Resolution and placement

- **Pre-resolution** (`pub fn resolve_trees()`, [packages/treeknit-core/src/resolve.rs#L72-L157](../../packages/treeknit-core/src/resolve.rs#L72-L157)): for one leaf set, each tree is refined by the <a id="gloss-use-2"></a>loose consensus <sup>[2](#gloss-2)</sup> of all trees, the splits of any tree compatible with every tree; optimal $O(nm)$ algorithms exist for $m$ trees of $n$ leaves <a id="cite-6"></a>[Jansson et al. 2016](https://doi.org/10.1145/2925985) [[6](#ref-6)]. Not named
- **Matched resolution** (`pub fn match_topologies()`, [packages/treeknit-core/src/pipeline.rs#L285-L327](../../packages/treeknit-core/src/pipeline.rs#L285-L327)): a common refinement within each MCC <a id="cite-7"></a>[Schaller et al. 2021](https://doi.org/10.1186/s13015-021-00202-8) [[7](#ref-7)], with precedence for earlier trees. Not named
- **Node-to-MCC map** (`pub fn map_mccs()`, [packages/treeknit-core/src/mcc_map.rs#L29-L65](../../packages/treeknit-core/src/mcc_map.rs#L29-L65)): a two-pass labeling in the style of <a id="cite-8"></a>[Fitch 1971](https://doi.org/10.2307/2412116) [[8](#ref-8)], with wildcards and without a change count. Named ("Fitch-like two-pass")
- **Placement of missing leaves** (`packages/treeknit-core/src/impute.rs`): tree completion, the problem of adding to a tree the leaves it lacks given a reference tree; the published algorithms minimize the Robinson-Foulds distance on binary trees <a id="cite-9"></a>[Bansal 2020](https://doi.org/10.1186/s13015-020-00166-1) [[9](#ref-9)]; <a id="cite-10"></a>[Christensen et al. 2018](https://doi.org/10.1186/s13015-018-0124-5) [[10](#ref-10)]. The port uses MCC consistency instead. Not named
- **ARG construction** (`pub fn arg_from_trees()`, [packages/treeknit-core/src/arg.rs#L141-L170](../../packages/treeknit-core/src/arg.rs#L141-L170)): the network of an agreement forest, valid when the forest is acyclic <a id="cite-11"></a>[Baroni et al. 2005](https://doi.org/10.1007/s00285-005-0315-9) [[11](#ref-11)] ([`agreement-forest-validation.md`](agreement-forest-validation.md)). Not named

### Output order, display, and evaluation

- **Polytomy sort** (`pub fn sort_polytomies_by_mccs()`, [packages/treeknit-core/src/mcc_map.rs#L96-L122](../../packages/treeknit-core/src/mcc_map.rs#L96-L122)): a one-sided tanglegram ordering with the first tree fixed, by MCC ranks and clade sizes. The optimal one-sided ordering by crossing count takes $O(n \log n)$ for $n$ leaves <a id="cite-12"></a>[Venkatachalam et al. 2010](https://doi.org/10.1109/tcbb.2010.57) [[12](#ref-12)]. Named as "polytomy sorting for tanglegrams"
- **Per-pair seeds** (`fn mix()`, [packages/treeknit-core/src/pipeline.rs#L488-L494](../../packages/treeknit-core/src/pipeline.rs#L488-L494)): the SplitMix64 finalizer <a id="cite-13"></a>[Steele et al. 2014](https://doi.org/10.1145/2660193.2660195) [[13](#ref-13)], with the constants of `rand_xoshiro::SplitMix64`; the generator is xoshiro256++ <a id="cite-14"></a>[Blackman and Vigna 2021](https://doi.org/10.1145/3460772) [[14](#ref-14)]. Not named
- **ARG layout order** (`fn topological_order()`, [packages/treeknit-io/src/display/arg_view.rs#L141-L160](../../packages/treeknit-io/src/display/arg_view.rs#L141-L160)): Kahn's topological sort <a id="cite-15"></a>[Kahn 1962](https://doi.org/10.1145/368996.369025) [[15](#ref-15)]. Named
- **MCC color slots** (`packages/treeknit-io/src/display/slots.rs`): sequential greedy graph coloring with a fixed number of colors. Named
- **Accuracy** (`fn scaled_vi()`, [packages/treeknit-io/examples/accuracy.rs#L17-L47](../../packages/treeknit-io/examples/accuracy.rs#L17-L47)): variation of information <a id="cite-16"></a>[Meilă 2007](https://doi.org/10.1016/j.jmva.2006.11.013) [[16](#ref-16)], the measure of the TreeKnit paper. Named, without the source

## Design axes

- **Short names in doc comments and pages in `kb/algo/` (recommended)**: a doc comment names the algorithm and links the source in one line, where a developer reads the code; a page per algorithm family in `kb/algo/`, the directory that `kb/README.md` reserves for algorithm documentation, holds the background, the differences from the published version, and the status in TreeKnit.jl and in the port
- **Doc comments only**: close to the code, but long explanations clutter the code
- **`kb/algo/` only**: complete, but a reader of the code does not find it

## Work items

- [`N-algorithm-names-undocumented.md`](../issues/N-algorithm-names-undocumented.md)

## Glossary

1. <a id="gloss-1"></a> **Agreement forest.** A partition of the leaves of two trees into blocks whose restricted subtrees are identical in both trees and whose spanning subtrees do not share nodes in either tree. [↩](#gloss-use-1)
2. <a id="gloss-2"></a> **Loose consensus.** The tree whose clades are the clades of any input tree that are compatible with every input tree. [↩](#gloss-use-2)

## References

1. <a id="ref-1"></a> Bordewich, Magnus, Simone Linz, Katherine St. John, and Charles Semple. 2007. "A reduction algorithm for computing the hybridization number of two trees." _Evolutionary Bioinformatics_ 3:117693430700300017. https://doi.org/10.1177/117693430700300017 [↩](#cite-1)
2. <a id="ref-2"></a> Whidden, Chris, Robert G. Beiko, and Norbert Zeh. 2013. "Fixed-parameter algorithms for maximum agreement forests." _SIAM Journal on Computing_ 42:1431-1466. https://doi.org/10.1137/110845045. Preprint: https://arxiv.org/abs/1108.2664 [↩](#cite-2)
3. <a id="ref-3"></a> Metropolis, Nicholas, Arianna W. Rosenbluth, Marshall N. Rosenbluth, Augusta H. Teller, and Edward Teller. 1953. "Equation of state calculations by fast computing machines." _The Journal of Chemical Physics_ 21:1087-1092. https://doi.org/10.1063/1.1699114 [↩](#cite-3)
4. <a id="ref-4"></a> Kirkpatrick, S., C. D. Gelatt, and M. P. Vecchi. 1983. "Optimization by simulated annealing." _Science_ 220:671-680. https://doi.org/10.1126/science.220.4598.671 [↩](#cite-4)
5. <a id="ref-5"></a> Hajek, Bruce. 1988. "Cooling schedules for optimal annealing." _Mathematics of Operations Research_ 13:311-329. https://doi.org/10.1287/moor.13.2.311 [↩](#cite-5)
6. <a id="ref-6"></a> Jansson, Jesper, Chuanqi Shen, and Wing-Kin Sung. 2016. "Improved algorithms for constructing consensus trees." _Journal of the ACM_ 63:1-24. https://doi.org/10.1145/2925985 [↩](#cite-6)
7. <a id="ref-7"></a> Schaller, David, Marc Hellmuth, and Peter F. Stadler. 2021. "A simpler linear-time algorithm for the common refinement of rooted phylogenetic trees on a common leaf set." _Algorithms for Molecular Biology_ 16:23. https://doi.org/10.1186/s13015-021-00202-8 [↩](#cite-7)
8. <a id="ref-8"></a> Fitch, Walter M. 1971. "Toward defining the course of evolution: Minimum change for a specific tree topology." _Systematic Zoology_ 20:406. https://doi.org/10.2307/2412116 [↩](#cite-8)
9. <a id="ref-9"></a> Bansal, Mukul S. 2020. "Linear-time algorithms for phylogenetic tree completion under Robinson-Foulds distance." _Algorithms for Molecular Biology_ 15:6. https://doi.org/10.1186/s13015-020-00166-1 [↩](#cite-9)
10. <a id="ref-10"></a> Christensen, Sarah, Erin K. Molloy, Pranjal Vachaspati, and Tandy Warnow. 2018. "OCTAL: Optimal completion of gene trees in polynomial time." _Algorithms for Molecular Biology_ 13:6. https://doi.org/10.1186/s13015-018-0124-5 [↩](#cite-10)
11. <a id="ref-11"></a> Baroni, Mihaela, Stefan Grünewald, Vincent Moulton, and Charles Semple. 2005. "Bounding the number of hybridisation events for a consistent evolutionary history." _Journal of Mathematical Biology_ 51:171-182. https://doi.org/10.1007/s00285-005-0315-9 [↩](#cite-11)
12. <a id="ref-12"></a> Venkatachalam, Balaji, Jim Apple, Katherine St. John, and Daniel Gusfield. 2010. "Untangling tanglegrams: Comparing trees by their drawings." _IEEE/ACM Transactions on Computational Biology and Bioinformatics_ 7:588-597. https://doi.org/10.1109/tcbb.2010.57 [↩](#cite-12)
13. <a id="ref-13"></a> Steele, Guy L., Doug Lea, and Christine H. Flood. 2014. "Fast splittable pseudorandom number generators." In _Proceedings of the 2014 ACM International Conference on Object Oriented Programming Systems Languages & Applications_, 453-472. ACM. https://doi.org/10.1145/2660193.2660195 [↩](#cite-13)
14. <a id="ref-14"></a> Blackman, David, and Sebastiano Vigna. 2021. "Scrambled linear pseudorandom number generators." _ACM Transactions on Mathematical Software_ 47:1-32. https://doi.org/10.1145/3460772. Preprint: https://arxiv.org/abs/1805.01407 [↩](#cite-14)
15. <a id="ref-15"></a> Kahn, A. B. 1962. "Topological sorting of large networks." _Communications of the ACM_ 5:558-562. https://doi.org/10.1145/368996.369025 [↩](#cite-15)
16. <a id="ref-16"></a> Meilă, Marina. 2007. "Comparing clusterings: An information based distance." _Journal of Multivariate Analysis_ 98:873-895. https://doi.org/10.1016/j.jmva.2006.11.013 [↩](#cite-16)
