# TreeKnit method: published paper compared with TreeKnit.jl

This report maps each part of the TreeKnit method, as published, to the code of TreeKnit.jl at `master` [`dbbc89a`](https://github.com/PierreBarrat/TreeKnit.jl/commit/dbbc89ac691fed0949a622eedbae103787b89320). For each part it states whether the code follows the paper, extends it, or diverges from it. The feature inventory in [`kb/feat/v0/`](../feat/v0/overview.md) describes the code alone.

## Summary

- **Same core method**: the score function, the iteration of <a id="gloss-use-1"></a>naive MCCs <sup>[1](#gloss-1)</sup> and annealing, the Poisson branch-length test, and the default $\gamma = 2$ in the code follow the paper <a id="cite-1a"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]
- **Extensions not in the paper**:
  - compatibility up to resolution inside the score
  - resolution between iterations
  - strict and liberal resolution with MCCs
  - the iteration limit `itmax`
  - the branch-length rule of the <a id="gloss-use-2"></a>ARG <sup>[2](#gloss-2)</sup>
  - the whole pipeline for more than two trees, which no publication describes or validates
- **Divergence in the likelihood test**: the paper multiplies the likelihood ratios of all branches. The code averages their logarithms and counts each removed leaf twice. Both changes can select another configuration when several have the same score
- **Unspecified in the paper**: the annealing temperatures, the cooling schedule, the move set, and the number of steps. The code values cannot be compared with the paper: geometric cooling from 1 to 0.05 over 101 temperatures, and about $50\,N$ single-leaf flips in total for $N$ coarse-grained leaves
- **Ignored sequence lengths**: the log-ratio scales linearly with a common sequence length (see [Branch-length likelihood](#branch-length-likelihood)). The command-line defect that ignores `--seq-lengths` therefore changes results only for segments of different lengths

## Sources

- **Paper**: the published version and its supplement S1 Text [[doc](https://journals.plos.org/ploscompbiol/article/file?type=supplementary&id=10.1371/journal.pcbi.1010394.s001)]. Section names in this report are those of the paper
- **Preprint**: <a id="cite-2"></a>[Barrat-Charlaix et al. 2021](https://doi.org/10.1101/2021.12.20.473456) [[2](#ref-2)], compared where noted
- **Code**: TreeKnit.jl `master`. Source links point to that revision

## Correspondence of conventions

### MCCs and the number of reassortments

- **Definition (same)**: the paper obtains <a id="gloss-use-3"></a>MCCs <sup>[3](#gloss-3)</sup> from an ARG "by removing all branches that correspond to only one tree", and characterizes them as "maximal sets of leaves that give rise to subtrees with matching topologies" (Methods, "Maximally Compatible Clades (MCC)"). The code represents an MCC as a sorted list of leaf labels, and the MCCs of a pair partition the leaves
- **Naive MCCs (same)**: the paper grows clades "as long as the obtained clades are exactly equivalent in the two trees". `function naive_mccs` does this by climbing from each leaf in both trees [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_base.jl#L13-L78)]
- **Number of reassortments (same as published)**: the published paper subtracts the MCC that "contains the root of one of the trees". The preprint and the TreeKnit.jl documentation say "root of both trees". The ARG code inserts no hybrid node above an MCC whose root is the root of either tree, which matches the published rule [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/construct.jl#L96-L111)]

### Score function

The paper defines the score of a configuration (Methods, Eq 1). In the notation of this report, which writes $N$ for the leaf count that the paper calls $L$:

$$N_\gamma(\vec\sigma) = \sum_{n} \Delta(n, \vec\sigma)\,\sigma_n + \gamma\,(N - |\vec\sigma|)$$

where:

- $\vec\sigma \in \{0,1\}^N$ -- the configuration. $\sigma_n = 1$ if <a id="gloss-use-4"></a>effective leaf <sup>[4](#gloss-4)</sup> $n$ stays in the trees
- $N$ -- the number of effective leaves
- $\Delta(n, \vec\sigma)$ -- 1 if the clades defined by the parents of $n$ in the two trees differ under $\vec\sigma$, else 0
- $|\vec\sigma|$ -- the number of leaves that stay
- $\gamma$ -- the cost of one removed leaf, that is, one enforced <a id="gloss-use-5"></a>reassortment <sup>[5](#gloss-5)</sup>

The code computes $F = E + \gamma\,(N - \sum_i c_i)$ [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/energy.jl#L15-L48)], where the code's configuration $c$ is $\vec\sigma$ and the energy $E$ sums $\Delta$ over the remaining leaves. The "parent" is the first ancestor with at least two remaining leaves. A removal therefore "resolves" an incompatibility as in the paper.

- **Formula and interpretation (same)**: the counting one level above the leaves (the paper calls the method greedy for this reason), and the meaning of $\gamma$: $\gamma = 1$ approximates parsimony, and $\gamma \to \infty$ gives the naive MCCs (Methods, "Approximately parsimonious MCCs")
- **Compatibility up to resolution (extension)**: with `resolve = true`, the code counts two different clades as equal when the larger one can be resolved into the smaller one [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/energy.jl#L98-L123)]. The paper handles polytomies only by resolving the trees before inference, and notes that "many incompatibilities due to polytomies are not found in this way" (Methods, "Poorly resolved trees")
- **More than two trees in the score (extension)**: the code sums over all pairs of $K$ trees. The pipeline calls it with two trees only
- **Default $\gamma$ (same)**: the paper concludes "$\gamma = 2$ is a robust choice" (Discussion). The code default is 2, and the command line always uses 2 (see [`documented-vs-actual.md`](../feat/v0/documented-vs-actual.md))

### Iteration and stopping

- **Paper** (Methods, "Finding MCCs: The TreeKnit method"): (i) naive MCCs, (ii) <a id="gloss-use-6"></a>coarse-graining <sup>[6](#gloss-6)</sup>, (iii) counting incompatibilities, (iv) annealing and removal. The loop stops when no leaf was removed and otherwise returns to (i)
- **Code** (`function runopt`, `function stop_conditions!`) [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L206-L316)]: the same loop with three more stop rules: the removed MCCs cover all leaves, the pruned trees are compatible, or the iteration counter exceeds `itmax = 15`. After each removal, the code resolves the pruned trees again
- **Assessment**: the first two rules end the loop in states where the paper's loop would also end at its next pass. The `itmax` limit can stop the search early on trees with many incompatibilities. The re-resolution is an extension

### Simulated annealing

- **Paper** (S1 Text §1): configurations follow "a random walk in the energy landscape defined by $N_\gamma$ at a temperature $T$", sampling $P_T(\vec\sigma) \propto e^{-N_\gamma(\vec\sigma)/T}$, with $T$ "initialized at a high value and slowly brought to 0". Results are reproducible "as long as the cooling speed is inversely proportional to the number of leaves", and the starting point is the naive result. The method follows <a id="cite-3"></a>[Kirkpatrick et al. 1983](https://doi.org/10.1126/science.220.4598.671) [[3](#ref-3)]
- **Code** [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/energy.jl#L187-L304)]:
  - <a id="gloss-use-7"></a>Metropolis acceptance <sup>[7](#gloss-7)</sup> of single-leaf flips
  - all leaves present at the start, which is the naive result
  - a geometric schedule from $T_{max} = 1$ to $T_{min} = 0.05$ with 101 temperatures
  - $\lceil 50\,N/101 \rceil$ flips per temperature, about $50\,N$ in total
- **Same**: the sampling target, the start, and the number of steps proportional to the number of leaves
- **Unspecified in the paper**: the move set, $T_{max}$, $T_{min}$, the schedule, and the constant 50
- **Final temperature**: with integer $\gamma$, score differences are integers. At $T_{min} = 0.05$, an uphill move of 1 has acceptance probability $e^{-1/0.05} = e^{-20} \approx 2.1 \times 10^{-9}$, so the end of the schedule is practically a descent that accepts no uphill move. This agrees with "brought to 0"

### Branch-length likelihood

The paper (S1 Text §5) models the number of mutations on a branch as Poisson. In the notation of this report, which writes $\theta_i$ for the branch time that the paper calls $T_i$:

$$P(n \mid \lambda) = \frac{e^{-\lambda}\,\lambda^{n}}{n!}, \qquad n_i = L_i\,t_i, \qquad \lambda_i = L_i\,\mu_i\,\theta_i$$

where:

- $i \in \{1, 2\}$ -- the segment
- $L_i$ -- the sequence length of segment $i$
- $t_i$ -- the branch length in tree $i$, in mutations per site
- $n_i$ -- the observed number of mutations on the branch of segment $i$
- $\mu_i$ -- the mutation rate of segment $i$
- $\theta_i$ -- the time length of the branch
- $\lambda_i$ -- the expected number of mutations

For branches that are not shared, the paper sets $\theta_i = t_i/\mu_i$, so $\lambda_i = n_i$. For a shared branch, it sets one time $\theta = (L_1 t_1 + L_2 t_2)/(L_1 + L_2)$ ("if the mutation rates are similar"), so with $\mu_i = 1$ the expected count is $\bar n_i = L_i\,\theta$. The likelihoods are $\mathcal{L}_{ns} = P(n_1 \mid n_1)\,P(n_2 \mid n_2)$ and $\mathcal{L}_s = P(n_1 \mid \bar n_1)\,P(n_2 \mid \bar n_2)$. Each removed leaf contributes $\mathcal{L}_{ns}/\mathcal{L}_s$, each remaining leaf without an incompatibility above it contributes $\mathcal{L}_s/\mathcal{L}_{ns}$, and "these scores are multiplied". The configuration with the highest product wins.

The code computes, for one pair of branch lengths [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/likelihood.jl#L8-L25)]:

$$\ell(t_1, t_2) = \ln\frac{\mathcal{L}_s}{\mathcal{L}_{ns}} = \sum_{i=1}^{2} \left[ n_i - \bar n_i + n_i \ln\frac{\bar n_i}{n_i} \right]$$

This is the paper's ratio with $\mu_i = 1$, because the factorials cancel. [`mcc-inference.md`](../feat/v0/mcc-inference.md) writes the same quantity with $\tau_k$ for $t_i$.

How the code combines the branches [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/likelihood.jl#L44-L104)]:

- **Average of the terms (divergence)**: the code returns $\frac{1}{Z}\left(\sum_{\text{kept}} \ell - \sum_{\text{removed}} \ell\right)$, where $Z$ is the number of terms. The paper's product corresponds to the sum without $1/Z$. Configurations that remove different numbers of leaves have different $Z$, so the two rules can choose different configurations
- **Double weight of removed leaves (divergence)**: for a removed leaf, the code loops over all ordered pairs of trees, including each tree with itself. With two trees, the pairs (1,2) and (2,1) give the same value, because the formula is symmetric, and the pairs (1,1) and (2,2) give 0. A removed leaf therefore contributes $-2\ell$ and four terms to $Z$, and a kept leaf contributes $\ell$ and one term. The paper weighs both kinds of leaves once
- **Branches compared for kept leaves (same)**: the code uses the distance from the effective leaf to its first ancestor with two remaining leaves, and only when the ancestors match. This agrees with "each leaf such that $\sigma_n = 1$ and that does not have any incompatibility above it"
- **Missing branch lengths (code defect)**: the paper does not discuss them. In the code, one `missing` length makes the value of a configuration `missing`, and `missing` wins the comparison (see [`mcc-inference.md`](../feat/v0/mcc-inference.md))
- **Scale invariance**: if $L_1 = L_2 = L$, then $\bar n_i = L\,(t_1 + t_2)/2$ and every term of $\ell$ is $L$ times its value at $L = 1$. A common factor keeps the same configuration on top. The command line always uses $L_1 = L_2 = 1$, so ignoring `--seq-lengths` matters only for different lengths, for example 1701 for HA and 1410 for NA
- **Effect size**: the paper reports that the test "has little influence over the results. The quality of the inferred MCCs is only marginally improved" (S1 Text §5). The divergences above therefore affect tie breaks among configurations with the same topological score

### Polytomy resolution

- **Pre-resolution (same for two trees, extended for more)**: the paper introduces "every split of T2 that is compatible with the set of splits in T1, and inversely" (Methods, "Poorly resolved trees"). The code does this and, for more than two trees, accepts a split only if every tree can take it [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/resolving.jl#L64-L135)]
- **Resolution with MCCs (same)**: the paper resolves "the subtrees corresponding to an MCC by complementing each others splits" (Results, "Increasing phylogenetic resolution of segment trees"). This is the liberal mode of the code
- **Strict mode (extension without published validation)**: neither the paper nor the preprint has it. It exists in the code since v0.4.0 (2022-11-03) and is the default since v0.5.0 (2023-01-26). The paper's Fig 4C evaluates the liberal behavior, and no published evaluation of strict mode exists

### ARG construction

- **Paper**: "If both segment trees and all MCCs are known, so is the observable part of the corresponding ARG", except for "the times at which reassortments occurred on internal branches" (Methods). The ARG is written in extended Newick <a id="cite-4"></a>[Cardona et al. 2008](https://doi.org/10.1186/1471-2105-9-532) [[4](#ref-4)]
- **Branch lengths (extension)**: the construction is deterministic given the liberally resolved trees and the MCCs, and it places each reassortment at half of the shorter branch above the MCC root [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/construct.jl#L263-L318)]. The paper gives no rule for the reassortment time
- **Output files (same)**: the paper lists the MCC list, the ARG, the resolved trees, and the node mapping (Methods, "Implementation and code availability"). The command line writes all four, with the MCC list as JSON since v0.5.0

### More than two trees

The paper covers two segments and names more segments as future work (Discussion). The only description of the pipeline for more than two trees is the TreeKnit.jl documentation. It claims that `:better_trees` minimizes false positive shared branches and that `:better_MCCs` gives more accurate MCCs "on average", and it gives no numbers [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/multitreeknit.md?plain=1#L13-L72)]. Searches of OpenAlex, Europe PMC, and bioRxiv for "MultiTreeKnit" and for TreeKnit with multiple segments found no publication, preprint, or thesis. The scientific validity of the multi-tree methods is therefore unverified.

## Reported performance of the published method

These results describe the method as published, which predates the code at `master`. They come from simulations with 100 leaves (Results, "Validation on simulated genealogies" and "Comparison with other methods"):

- **Accuracy**: the <a id="gloss-use-8"></a>positive predictive value <sup>[8](#gloss-8)</sup> of inferred reassortments is above 80% for $\gamma = 2$ at intermediate reassortment rates. The naive method overestimates the number of MCCs, and $\gamma = 1$ underestimates it at high reassortment rates
- **Resolution**: with poorly resolved trees at the level found for A/H3N2 HA (resolution parameter $c_{res} \approx 0.8$, where $c_{res} = 0$ means fully resolved trees), the error nearly doubles compared with fully resolved trees
- **Comparison**: "TreeKnit and CoalRe perform similarly well, with CoalRe reporting slightly less [sic] false reassortments. GiRaF misses reassortment events, but very rarely reports a reassortment that did not happen"
- **Runtime**: "the average runtime of TreeKnit was 40ms, whereas GiRaF took 40s", and CoalRe takes "several hours". One score evaluation costs time linear in the number of leaves. The annealing as a whole is quadratic, and "closer to cubic" for very asymmetric trees (S1 Text §1, Results)
- **Real data**: on about 150 A/H3N2 strains from New York (1999 to 2004), with $\gamma = 2$, TreeKnit finds the known reassortments and three additional reassorted clades. The TreeKnit.jl test suite checks the MCCs on this data set [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/test/NYdata/test.jl#L1-L30)]

## Inconsistencies inside the paper

- **Fig 2**: the text says removing leaf D is optimal "for $\gamma \le 5$", and the caption says "$\gamma < 5$". At $\gamma = 5$ both options have score 5
- **Coalescence rate**: S1 Text §2 and §3 write the Kingman coalescence rate with and without a factor of one half
- **Reassortment rate for H3N2**: for the scaled reassortment rate $r^\star$, the reassortment rate divided by the coalescence rate, the main text gives $r^\star \approx 0.05$, S1 Text §3 gives 0.06, and the S12 Fig caption gives a range of 0.043 to 0.093

## Open questions

- **Likelihood combination**: the code, the commits read, and the paper do not say whether the average and the double weight of removed leaves are intended. Only the paper's product has a derivation
- **Strict resolution**: no published benchmark shows that strict mode improves the MCCs or the trees
- **Multi-tree methods**: the claims about `:better_trees` and `:better_MCCs` have no published evidence
- **Annealing parameters**: the paper shows convergence plots (S14 and S15 Figs) without the schedule they used, so it is unknown whether the code defaults match the validated setting

## Glossary

1. <a id="gloss-1"></a> **Naive MCC.** A largest clade with exactly the same topology in both trees, found without removing any leaf. Naive MCCs start each iteration and are the result in the limit $\gamma \to \infty$. [↩](#gloss-use-1)
2. <a id="gloss-2"></a> **ARG (ancestral reassortment graph).** A graph that holds the genealogies of all segments, with one node of two parents for each reassortment <a id="cite-1b"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]. [↩](#gloss-use-2)
3. <a id="gloss-3"></a> **MCC (maximally compatible clade).** A largest set of leaves whose subtrees have the same topology in two segment trees, which is a region of the ARG that both segments share. [↩](#gloss-use-3)
4. <a id="gloss-4"></a> **Effective leaf.** One naive MCC replaced by a single leaf before the annealing. The code calls it a coarse-grained leaf. [↩](#gloss-use-4)
5. <a id="gloss-5"></a> **Reassortment.** The exchange of genome segments between two viruses that infect the same cell, which gives the segments of one virus different ancestries. [↩](#gloss-use-5)
6. <a id="gloss-6"></a> **Coarse-graining.** The replacement of each naive MCC by one effective leaf. [↩](#gloss-use-6)
7. <a id="gloss-7"></a> **Metropolis acceptance.** The rule that accepts a move that lowers the score, and accepts a move that raises it by $\Delta F$ with probability $e^{-\Delta F/T}$ at temperature $T$ <a id="cite-3b"></a>[Kirkpatrick et al. 1983](https://doi.org/10.1126/science.220.4598.671) [[3](#ref-3)]. [↩](#gloss-use-7)
8. <a id="gloss-8"></a> **Positive predictive value.** The fraction of inferred reassortments that are present in the true ARG. [↩](#gloss-use-8)

## References

1. <a id="ref-1"></a> Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2022. "TreeKnit: Inferring ancestral reassortment graphs of influenza viruses." _PLOS Computational Biology_ 18:e1010394. https://doi.org/10.1371/journal.pcbi.1010394 [↩¹](#cite-1a) [↩²](#cite-1b)
2. <a id="ref-2"></a> Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2021. "TreeKnit: Inferring ancestral reassortment graphs of influenza viruses." _bioRxiv_ 2021.12.20.473456. Preprint of [1]. https://doi.org/10.1101/2021.12.20.473456 [↩](#cite-2)
3. <a id="ref-3"></a> Kirkpatrick, S., C. D. Gelatt, and M. P. Vecchi. 1983. "Optimization by simulated annealing." _Science_ 220:671-680. https://doi.org/10.1126/science.220.4598.671 [↩¹](#cite-3) [↩²](#cite-3b)
4. <a id="ref-4"></a> Cardona, Gabriel, Francesc Rosselló, and Gabriel Valiente. 2008. "Extended Newick: It is time for a standard representation of phylogenetic networks." _BMC Bioinformatics_ 9:532. https://doi.org/10.1186/1471-2105-9-532 [↩](#cite-4)
