# Chain and cluster reductions before annealing

Exact algorithms for agreement forests shrink the two trees with three reduction rules before they search. TreeKnit applies the first rule, the subtree reduction, as its naive MCCs. The other two, the chain reduction and the cluster reduction, are absent from TreeKnit.jl and from the port. The cluster reduction would split one tree pair into independent smaller problems, which shortens the annealing and allows parallel work within a pair. Whether TreeKnit's score allows the split is not known.

## The three rules

For two rooted binary trees $S$ and $T$ with hybridization number $h(S, T)$, Bordewich et al. state the rules as follows <a id="cite-1a"></a>[Bordewich et al. 2007](https://doi.org/10.1177/117693430700300017) [[1](#ref-1)]:

- **Subtree reduction**: "Replace a maximal pendant subtree with at least two leaves that occurs identically in S and T by a single leaf with a new label"; $h$ does not change. This is TreeKnit's naive MCC step
- **Chain reduction**: replace a maximal <a id="gloss-use-1"></a>chain <sup>[1](#gloss-1)</sup> of at least three leaves "that occur identically and with the same orientation relative to the root in S and T" by two new leaves with weight $c - 2$, where $c$ is the number of leaves in the chain; $h$ stays the same or grows by $c - 2$, depending on whether the two new leaves end up in the same block of a smallest acyclic agreement forest
- **Cluster reduction**: for a minimal <a id="gloss-use-2"></a>common cluster <sup>[2](#gloss-2)</sup> $A$ with at least two leaves, split the instance into the restrictions $S_1, T_1$ to $A$ and the trees $S_2, T_2$ in which $A$ is one leaf; then

$$h(S, T) = h(S_1, T_1) + h(S_2, T_2)$$

The proof of the cluster reduction for the hybridization number is in <a id="cite-2"></a>[Baroni et al. 2006](https://doi.org/10.1080/10635150500431197) [[2](#ref-2)]; a cluster reduction for the rooted SPR distance is in <a id="cite-3"></a>[Linz and Semple 2011](https://doi.org/10.1007/s00026-011-0108-3) [[3](#ref-3)]. All three rules are stated for binary trees.

## Relevance to TreeKnit

- **Chains**: a chain that both trees share stays as many effective leaves in the annealing. Ladder-like influenza trees have long chains, so the annealing walks over leaves whose removal cannot help more than the two chain ends
- **Clusters**: TreeKnit's score counts, for each kept effective leaf, whether its first ancestor with two kept leaves below has the same clade in both trees, plus $\gamma$ per removed leaf <a id="cite-4"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[4](#ref-4)], where $\gamma$ is the cost of one removal. Below a common cluster both trees hold the same leaves, so the term of a leaf inside the cluster depends only on decisions inside it while at least two of its leaves are kept. Whether the score of the whole pair is the sum of the scores of the two parts, including the case where the cluster keeps fewer than two leaves, is not proven
- **Parallel work**: independent parts could run in parallel; today the port runs pairs in parallel only in rounds without resolution (README, "Deliberate differences from TreeKnit.jl")

## Constraints

- **Equivalence with TreeKnit.jl**: the reductions change which effective leaves the annealing sees and the order of random draws. Seeded results change, and the comparison with TreeKnit.jl must then be made on distributions of results, as `annealing_distribution_vs_julia` does for two-tree cases
- **Polytomies**: the rules are stated for binary trees; the port works on multifurcating trees and compares clades up to resolution when it resolves

## Design axes

- **Prove or test additivity first (recommended)**: show that the score adds up across a common cluster, or measure with the simulated fixtures whether reduced and unreduced runs give the same distribution of MCCs and of the variation of information to the true MCCs
- **Chain reduction only**: smaller change, no question of additivity, but a weighted pair of leaves has no counterpart in the score yet
- **Cluster reduction only**: largest gain in run time and parallelism, depends on additivity
- **Both**

## Work items

- [`N-kernel-reductions-unevaluated.md`](../issues/N-kernel-reductions-unevaluated.md)

## Glossary

1. <a id="gloss-1"></a> **Chain.** A sequence of leaves that hang one after another from consecutive nodes of a path, as in a ladder; it occurs identically in two trees when both trees have the same sequence in the same direction. [↩](#gloss-use-1)
2. <a id="gloss-2"></a> **Common cluster.** A set of leaves that is a clade in both trees. [↩](#gloss-use-2)

## References

1. <a id="ref-1"></a> Bordewich, Magnus, Simone Linz, Katherine St. John, and Charles Semple. 2007. "A reduction algorithm for computing the hybridization number of two trees." _Evolutionary Bioinformatics_ 3:117693430700300017. https://doi.org/10.1177/117693430700300017 [↩](#cite-1a)
2. <a id="ref-2"></a> Baroni, Mihaela, Charles Semple, and Mike Steel. 2006. "Hybrids in real time." _Systematic Biology_ 55:46-56. https://doi.org/10.1080/10635150500431197 [↩](#cite-2)
3. <a id="ref-3"></a> Linz, Simone, and Charles Semple. 2011. "A cluster reduction for computing the subtree distance between phylogenies." _Annals of Combinatorics_ 15:465-484. https://doi.org/10.1007/s00026-011-0108-3 [↩](#cite-3)
4. <a id="ref-4"></a> Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2022. "TreeKnit: Inferring ancestral reassortment graphs of influenza viruses." _PLOS Computational Biology_ 18:e1010394. https://doi.org/10.1371/journal.pcbi.1010394 [↩](#cite-4)
