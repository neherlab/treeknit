# TreeKnit method: published paper compared with TreeKnit.jl

This report maps each part of the TreeKnit method, as published, to the code of TreeKnit.jl at `master` [`dbbc89a`](https://github.com/PierreBarrat/TreeKnit.jl/commit/dbbc89ac691fed0949a622eedbae103787b89320). For each part it states whether the code follows the paper, extends it, or diverges from it. The feature inventory in [`kb/feat/`](../feat/overview.md) describes the code alone.

## Summary

- **Same core method**: the score function, the iteration of naive MCCs and annealing, the Poisson branch-length test, and the default $\gamma = 2$ in the code follow the paper <a id="cite-1a"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]
- **Extensions not in the paper**: compatibility up to resolution inside the score, resolution between iterations, strict and liberal resolution with MCCs, the iteration limit `itmax`, the ARG branch-length rule, and the whole pipeline for more than two trees. No publication describes or validates the pipeline for more than two trees
- **Divergence in the likelihood test**: the paper multiplies the likelihood ratios of all branches. The code averages their logarithms and counts each removed leaf twice. Both change which configuration wins when several have the same score
- **Unspecified in the paper**: the annealing temperatures, the cooling schedule, the move set, and the number of steps. The code values (geometric cooling from 1 to 0.05 over 101 temperatures, about $50\,n$ single-leaf flips in total) cannot be checked against the paper
- **Ignored sequence lengths have no effect when all lengths are equal**: the log-ratio scales linearly with a common sequence length (see [Branch-length likelihood](#branch-length-likelihood)), so the command-line defect that ignores `--seq-lengths` changes results only for segments of different lengths

## Sources

- **Paper**: the published version with its supplement S1 Text [[doc](https://journals.plos.org/ploscompbiol/article/file?type=supplementary&id=10.1371/journal.pcbi.1010394.s001)], read in full. Sections are named as in the paper
- **Preprint**: <a id="cite-2"></a>[Barrat-Charlaix et al. 2021](https://doi.org/10.1101/2021.12.20.473456) [[2](#ref-2)], compared where noted
- **Code**: TreeKnit.jl `master`, read in full. Links point to that revision

## Correspondence of conventions

### MCCs and the number of reassortments

- **Definition**: the paper obtains MCCs from an ARG "by removing all branches that correspond to only one tree", and characterizes them as "maximal sets of leaves that give rise to subtrees with matching topologies" (Methods, "Maximally Compatible Clades (MCC)"). The code represents an MCC as a sorted list of leaf labels, and the MCCs of a pair partition the leaves. **Same**
- **Naive <a id="gloss-use-1"></a>MCCs <sup>[1](#gloss-1)</sup>**: the paper grows clades "as long as the obtained clades are exactly equivalent in the two trees". `fn naive_mccs()` does this by climbing from each leaf in both trees [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_base.jl#L13-L78)]. **Same**
- **Number of reassortments**: the published paper subtracts the MCC that "contains the root of one of the trees". The preprint says "root of both trees", and so does the TreeKnit.jl documentation. The ARG code inserts no hybrid node above an MCC whose root is the root of either tree, which matches the published rule [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/construct.jl#L96-L111)]. **Same as published; the documentation keeps the preprint wording**

### Score function

The paper defines the score of a configuration (Methods, Eq 1) as

$$N_\gamma(\vec\sigma) = \sum_{n} \Delta(n, \vec\sigma)\,\sigma_n + \gamma\,(L - |\vec\sigma|)$$

where:

- $\vec\sigma \in \{0,1\}^L$ -- the configuration; $\sigma_n = 1$ if effective leaf $n$ stays in the trees
- $L$ -- the number of effective leaves, which are the naive MCCs after coarse-graining
- $\Delta(n, \vec\sigma)$ -- 1 if the clades defined by the parents of $n$ in the two trees differ under $\vec\sigma$, else 0
- $|\vec\sigma|$ -- the number of leaves that stay
- $\gamma$ -- the cost of one removed leaf, that is, one enforced <a id="gloss-use-2"></a>reassortment <sup>[2](#gloss-2)</sup>

The code computes $F = E + \gamma\,(N - \sum_i c_i)$ with the same meaning: configuration $c$ is $\vec\sigma$, $N$ is $L$, and $E$ sums $\Delta$ over the remaining leaves [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/energy.jl#L15-L48)]. The "parent" is the first ancestor with at least two remaining leaves, which is how a removal "resolves" an incompatibility in the paper.

- **Same**: the formula, the per-leaf counting one level above the leaves (the paper calls the method greedy for this reason), and the interpretation of $\gamma$: $\gamma = 1$ approximates parsimony, $\gamma \to \infty$ gives the naive MCCs (Methods, "Approximately parsimonious MCCs")
- **Extension**: with `resolve = true`, the code counts two different clades as equal when the larger one can be resolved into the smaller one [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/energy.jl#L98-L123)]. The paper handles polytomies only by resolving the trees before inference and notes that "many incompatibilities due to polytomies are not found in this way" (Methods, "Poorly resolved trees")
- **Extension**: the code sums over all pairs of $K$ trees. The pipeline uses only $K = 2$
- **Default $\gamma$**: the paper concludes "$\gamma = 2$ is a robust choice" (Discussion). The code default is 2, and the command line always uses 2 (see [`documented-vs-actual.md`](../feat/documented-vs-actual.md))

### Iteration and stopping

- **Paper** (Methods, "Finding MCCs: The TreeKnit method"): steps (i) naive MCCs, (ii) coarse-graining, (iii) counting incompatibilities, (iv) annealing and removal; stop when no leaf was removed, otherwise return to (i)
- **Code** (`fn runopt()`, `fn stop_conditions!()`) [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/main.jl#L206-L316)]: the same loop with three additional stop rules: the removed MCCs cover all leaves, the pruned trees are compatible, or the iteration counter exceeds `itmax = 15`. After each removal, the code resolves the pruned trees again
- **Assessment**: the additional rules end the loop in states where the paper's loop would also end at its next pass, except the `itmax` limit, which can stop a valid search early on very incompatible trees. The re-resolution is an extension

### Simulated annealing

- **Paper** (S1 Text §1): configurations follow "a random walk in the energy landscape defined by $N_\gamma$ at a temperature $T$", sampling $P_T(\vec\sigma) \propto e^{-N_\gamma(\vec\sigma)/T}$, with $T$ "initialized at a high value and slowly brought to 0"; results are reproducible "as long as the cooling speed is inversely proportional to the number of leaves"; the starting point is the naive result. The method follows <a id="cite-3"></a>[Kirkpatrick et al. 1983](https://doi.org/10.1126/science.220.4598.671) [[3](#ref-3)]
- **Code** [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/energy.jl#L187-L304)]: Metropolis acceptance of single-leaf flips, all leaves present at the start (which is the naive result), a geometric schedule from $T_{max} = 1$ to $T_{min} = 0.05$ with 101 temperatures, and $M = \lceil 50\,n/101 \rceil$ flips per temperature for $n$ remaining leaves, so about $50\,n$ flips in total
- **Same**: the sampling target, the start, and the number of steps proportional to the number of leaves
- **Unspecified in the paper**: the move set, $T_{max}$, $T_{min}$, the schedule, and the constant 50. No comparison is possible
- **Final temperature**: with integer $\gamma$, score differences are integers. At $T_{min} = 0.05$, an uphill move of 1 has acceptance probability $e^{-1/0.05} = e^{-20} \approx 2.1 \times 10^{-9}$, so the end of the schedule is practically a greedy descent. This agrees with "brought to 0"

### Branch-length likelihood

The paper (S1 Text §5) models the number of mutations on a branch of segment $i$ as Poisson:

$$P(n_i \mid n_i^\star) = \frac{e^{-n_i^\star}\,(n_i^\star)^{n_i}}{n_i!}, \qquad n_i = L_i\,t_i, \qquad n_i^\star = L_i\,\mu_i\,T_i$$

where:

- $i \in \{1, 2\}$ -- the segment
- $L_i$ -- the sequence length of segment $i$
- $t_i$ -- the branch length in tree $i$, in mutations per site
- $\mu_i$ -- the mutation rate of segment $i$
- $T_i$ -- the time length of the branch

For branches that are not shared, $T_i^\star = t_i/\mu_i$. For a shared branch, the paper uses $T^\star = (L_1 t_1 + L_2 t_2)/(L_1 + L_2)$ ("if the mutation rates are similar"). It defines $\mathcal{L}_{ns} = P(t_1 \mid T_1^\star)\,P(t_2 \mid T_2^\star)$ and $\mathcal{L}_s = P(t_1 \mid T^\star)\,P(t_2 \mid T^\star)$. Each removed leaf contributes $\mathcal{L}_{ns}/\mathcal{L}_s$, each remaining leaf without an incompatibility above it contributes $\mathcal{L}_s/\mathcal{L}_{ns}$, and "these scores are multiplied". The configuration with the highest product wins.

The code computes, for one pair of branch lengths [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/likelihood.jl#L8-L25)]:

$$\ell(t_1, t_2) = \ln\frac{\mathcal{L}_s}{\mathcal{L}_{ns}} = \sum_{i=1}^{2} \left[ n_i - \bar n_i + n_i \ln\frac{\bar n_i}{n_i} \right], \qquad \bar n_i = L_i\,\frac{L_1 t_1 + L_2 t_2}{L_1 + L_2}$$

where $\bar n_i$ is the expected count of segment $i$ on a shared branch. This is the paper's ratio with $\mu_i = 1$; the factorials cancel. **Same per branch.**

How the code combines the branches [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SplitGraph/likelihood.jl#L44-L104)]:

- **Average instead of product**: the code returns $\frac{1}{Z}\left(\sum_{\text{kept}} \ell - \sum_{\text{removed}} \ell\right)$, where $Z$ is the number of terms. The paper's product corresponds to the sum without $1/Z$. Configurations that remove different numbers of leaves have different $Z$, so the two rules can choose different configurations. **Divergence**
- **Double weight of removed leaves**: for a removed leaf, the code loops over all ordered pairs of trees, including each tree with itself. With two trees, the pairs (1,2) and (2,1) give the same value (the formula is symmetric), and the pairs (1,1) and (2,2) give 0. A removed leaf therefore contributes $-2\ell$ and four terms to $Z$, while a kept leaf contributes $\ell$ and one term. The paper weighs both kinds of leaves once. **Divergence**
- **Branches compared for kept leaves**: the code uses the distance from the effective leaf to its first ancestor with two remaining leaves, and only when the ancestors match. This agrees with "each leaf such that $\sigma_n = 1$ and that does not have any incompatibility above it"
- **Missing branch lengths**: not discussed in the paper. In the code, one `missing` length makes the configuration's value `missing`, and `missing` wins the comparison (see [`mcc-inference.md`](../feat/mcc-inference.md)). **Code-specific defect**
- **Scale invariance**: if $L_1 = L_2 = L$, then $\bar n_i = L\,(t_1 + t_2)/2$ and every term of $\ell$ is $L$ times its value at $L = 1$. A common factor does not change which configuration has the highest value. The command line always uses $L_1 = L_2 = 1$, so ignoring `--seq-lengths` matters only when the user gives different lengths, for example 1701 for HA and 1410 for NA
- **Effect size**: the paper reports that the test "has little influence over the results. The quality of the inferred MCCs is only marginally improved" (S1 Text §5). The divergences above therefore affect tie breaks, not the bulk of the inference

### Polytomy resolution

- **Pre-resolution**: the paper introduces "every split of T2 that is compatible with the set of splits in T1, and inversely" (Methods, "Poorly resolved trees"). The code does this and, for more than two trees, accepts a split only if every tree can take it [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/resolving.jl#L64-L135)]. **Same for two trees; extension for more**
- **Resolution with MCCs**: the paper resolves "the subtrees corresponding to an MCC by complementing each others splits" (Results, "Increasing phylogenetic resolution of segment trees"). This is the code's liberal mode. **Same**
- **Strict mode**: neither the paper nor the preprint has it. It exists in the code since v0.4.0 (2022-11-03) and is the default since v0.5.0 (2023-01-26). The paper's Fig 4C evaluates the liberal behavior; no published evaluation of strict mode exists. **Extension without published validation**

### ARG construction

- **Paper**: "If both segment trees and all MCCs are known, so is the observable part of the corresponding ARG", except for "the times at which reassortments occurred on internal branches" (Methods). The ARG is written in extended Newick <a id="cite-4"></a>[Cardona et al. 2008](https://doi.org/10.1186/1471-2105-9-532) [[4](#ref-4)]
- **Code**: the construction is deterministic given the liberally resolved trees and the MCCs, and it places each reassortment at half of the shorter branch above the MCC root [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/construct.jl#L263-L318)]. The paper does not give a rule for the reassortment time. **Extension (heuristic)**
- **Output files**: the paper lists the MCC list, the ARG, the resolved trees, and the node mapping (Methods, "Implementation and code availability"). The command line writes all four, with the MCC list as JSON since v0.5.0

### More than two trees

The paper covers two segments and names more segments as future work (Discussion). The only description of the pipeline for more than two trees is the TreeKnit.jl documentation, which claims that `:better_trees` minimizes false positive shared branches and that `:better_MCCs` gives more accurate MCCs, "on average", without published numbers [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/multitreeknit.md?plain=1#L13-L72)]. Searches of OpenAlex, Europe PMC, and bioRxiv for "MultiTreeKnit" and for TreeKnit with multiple segments found no publication, preprint, or thesis. The scientific validity of the multi-tree methods is therefore unverified.

## Reported performance of the published method

These results describe the method as published, not the code at `master`, and they come from simulations with 100 leaves (Results, "Validation on simulated genealogies" and "Comparison with other methods"):

- **Accuracy**: the positive predictive value of inferred reassortments is above 80% for $\gamma = 2$ at intermediate reassortment rates. The naive method overestimates the number of MCCs, and $\gamma = 1$ underestimates it at high reassortment rates
- **Resolution**: with poorly resolved trees at the level found for A/H3N2 HA ($c \approx 0.8$), the error nearly doubles compared with fully resolved trees
- **Comparison**: "TreeKnit and CoalRe perform similarly well, with CoalRe reporting slightly less false reassortments. GiRaF misses reassortment events, but very rarely reports a reassortment that did not happen"
- **Runtime**: "the average runtime of TreeKnit was 40ms, whereas GiRaF took 40s", and CoalRe takes "several hours". The cost of one score evaluation is linear in the number of leaves, and the annealing is quadratic overall, "closer to cubic" for very asymmetric trees (S1 Text §1; Results)
- **Real data**: on about 150 A/H3N2 strains from New York (1999 to 2004), with $\gamma = 2$, TreeKnit finds the known reassortments and three additional reassorted clades. The TreeKnit.jl test suite checks the MCCs on this data set [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/test/NYdata/test.jl#L1-L30)]

## Inconsistencies inside the paper

- **Fig 2**: the text says removing leaf D is optimal "for $\gamma \le 5$", the caption says "$\gamma < 5$". At $\gamma = 5$ both options have score 5
- **Coalescence rate**: S1 Text §2 gives the Kingman rate as $n(n-1)/2N$ and §3 as $n(n-1)/N$
- **Reassortment rate for H3N2**: the main text gives $r^\star \approx 0.05$, S1 Text §3 gives 0.06, and the S12 Fig caption gives a range of 0.043 to 0.093

## Open questions

- **Likelihood combination**: whether the average and the double weight of removed leaves were intended is not documented in the code, the commits read, or the paper. Only the paper's product has a derivation
- **Strict resolution**: no published benchmark shows that strict mode improves the MCCs or the trees
- **Multi-tree methods**: the claims about `:better_trees` and `:better_MCCs` have no published evidence
- **Annealing parameters**: the paper shows convergence plots (S14 and S15 Figs) without stating the schedule they used, so it is unknown whether the code defaults match the validated setting

## Glossary

1. <a id="gloss-1"></a> **Naive MCC.** A maximal clade with exactly the same topology in both trees, found without removing any leaf. Naive MCCs are the starting point of each iteration and the result in the limit $\gamma \to \infty$. [↩](#gloss-use-1)
2. <a id="gloss-2"></a> **Reassortment.** The exchange of genome segments between two viruses that infect the same cell, which gives the segments of one virus different ancestries <a id="cite-1b"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]. [↩](#gloss-use-2)

## References

1. <a id="ref-1"></a> Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2022. "TreeKnit: Inferring Ancestral Reassortment Graphs of Influenza Viruses." _PLOS Computational Biology_ 18 (8): e1010394. https://doi.org/10.1371/journal.pcbi.1010394 [↩¹](#cite-1a) [↩²](#cite-1b)
2. <a id="ref-2"></a> Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2021. "TreeKnit: Inferring Ancestral Reassortment Graphs of Influenza Viruses." _bioRxiv_ 2021.12.20.473456. Preprint of [1]. https://doi.org/10.1101/2021.12.20.473456 [↩](#cite-2)
3. <a id="ref-3"></a> Kirkpatrick, S., C. D. Gelatt, and M. P. Vecchi. 1983. "Optimization by Simulated Annealing." _Science_ 220 (4598): 671-680. https://doi.org/10.1126/science.220.4598.671 [↩](#cite-3)
4. <a id="ref-4"></a> Cardona, Gabriel, Francesc Rosselló, and Gabriel Valiente. 2008. "Extended Newick: It Is Time for a Standard Representation of Phylogenetic Networks." _BMC Bioinformatics_ 9: 532. https://doi.org/10.1186/1471-2105-9-532 [↩](#cite-4)
