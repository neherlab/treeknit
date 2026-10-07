# Reassortment inference from segment trees: methods related to TreeKnit

This report places TreeKnit among the published methods that infer reassortment or reticulate histories from trees. It covers the reassortment detectors that have been benchmarked against TreeKnit, the combinatorial problems behind its approach, polytomy handling, <a id="gloss-use-1"></a>ARG <sup>[1](#gloss-1)</sup> formats, and <a id="gloss-use-2"></a>tanglegram <sup>[2](#gloss-2)</sup> layout. The feature inventory in [`kb/feat/v0/`](../feat/v0/overview.md) describes TreeKnit.jl itself. [`treeknit-paper-vs-code.md`](treeknit-paper-vs-code.md) compares the TreeKnit paper with its code.

## Summary

- **TreeKnit paper**: TreeKnit is about as accurate as the Bayesian method CoalRe on simulations with 100 leaves, with a runtime of 40 ms against several hours <a id="cite-1a"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]
- **Independent benchmarks**: four later studies compare TreeKnit with other methods:
  - TreeKnit and CoalRe get the number of reassortments right in 15% to 45% of simulations with 50 tips <a id="cite-2a"></a>[Lin et al. 2024](https://doi.org/10.1093/molbev/msae078) [[2](#ref-2)]
  - TreeSort is the most accurate method on 1,000 or more strains <a id="cite-17a"></a>[Markin et al. 2025](https://doi.org/10.1093/molbev/msaf133) [[17](#ref-17)]
  - PhyloFusion finds a lower <a id="gloss-use-3"></a>hybridization number <sup>[3](#gloss-3)</sup> than TreeKnit "in many cases" <a id="cite-9a"></a>[Zhang et al. 2025](https://doi.org/10.1093/sysbio/syaf049) [[9](#ref-9)]
  - TreeKnit has low precision on noisy trees <a id="cite-18a"></a>[Cai et al. 2026](https://doi.org/10.1093/nar/gkag255) [[18](#ref-18)]
- **Identifiability limits all tree-based methods**: the number and pattern of reassortments are "not necessarily identifiable" from segment trees alone <a id="cite-2b"></a>[Lin et al. 2024](https://doi.org/10.1093/molbev/msae078) [[2](#ref-2)]
- **Combinatorial relatives**: TreeKnit searches for compatible regions of two trees, a goal close to <a id="gloss-use-4"></a>agreement forests <sup>[4](#gloss-4)</sup> and minimum hybridization networks. Those problems are NP-hard and have exact <a id="gloss-use-5"></a>fixed-parameter <sup>[5](#gloss-5)</sup> algorithms. No source states the formal relation between TreeKnit's MCCs and agreement forests. For more than two trees, no agreement-forest characterization of the hybridization number is known, and no published method combines pairwise comparisons of segment trees into one network (see [Networks from more than two trees](#networks-from-more-than-two-trees))
- **Polytomies**: TreeKnit adds the compatible splits of each tree to the other. Other methods use the same mutual refinement, a parsimony resolution on other segments, or agreement-forest algorithms for multifurcating trees
- **Name collision**: in BEAST and CoalRe, "MCC" means maximum clade credibility. Lin et al. use both meanings of "MCC" in one figure caption

## Method families

### Topology and compatibility heuristics

- **TreeKnit**: finds <a id="gloss-use-6"></a>maximally compatible clades <sup>[6](#gloss-6)</sup> by simulated annealing over clade removals, with a cost $\gamma$ per removed clade against the number of topological incompatibilities. It reads only two trees at a time and needs the same leaves in both <a id="cite-1b"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]
- **GiRaF**: "Graph-incompatibility-based Reassortment Finder", which identifies reassortments from samples of uncertain phylogenies by graph mining <a id="cite-3"></a>[Nagarajan and Kingsford 2011](https://doi.org/10.1093/nar/gkq1232) [[3](#ref-3)]. In the TreeKnit paper, GiRaF "misses reassortment events, but very rarely reports a reassortment that did not happen" <a id="cite-1c"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]
- **MLreassort and Breassort**: algorithms that detect strains with a composite genomic architecture from incongruent segment trees <a id="cite-4"></a>[Svinti et al. 2013](https://doi.org/10.1186/1471-2148-13-1) [[4](#ref-4)]

### Network inference from gene trees

- **RF-Net 2**: infers reassortment and hybridization networks from gene trees with errors, with a Robinson-Foulds embedding cost <a id="cite-5"></a>[Markin et al. 2022](https://doi.org/10.1093/bioinformatics/btac075) [[5](#ref-5)]
- **Minimum hybridization networks**:
  - Dendroscope 3 computes a representative set of minimum hybridization networks for two bifurcating trees <a id="cite-6"></a>[Albrecht et al. 2012](https://doi.org/10.1093/bioinformatics/btr618) [[6](#ref-6)]
  - The Autumn algorithm handles "realistic" trees with multifurcations and different taxa through maximum acyclic agreement forests <a id="cite-7a"></a>[Huson and Linz 2018](https://doi.org/10.1109/TCBB.2016.2537326) [[7](#ref-7)]
  - FHyNCH is a heuristic for large sets of multifurcating trees <a id="cite-8"></a>[Bernardini et al. 2024](https://doi.org/10.1016/j.ympev.2024.108137) [[8](#ref-8)]
- **PhyloFusion**: fuses rooted trees into a <a id="gloss-use-7"></a>tree-child network <sup>[7](#gloss-7)</sup>. Its benchmark on 100 tree pairs with 10 to 400 taxa (Results, Fig. 11) states that "both TreeKnit and FHyNCH required only seconds", that "PhyloFusion performs better than TreeKnit in many cases" in hybridization number, and that "the TreeKnit program only accepts two input trees, and it does not accept missing taxa" <a id="cite-9b"></a>[Zhang et al. 2025](https://doi.org/10.1093/sysbio/syaf049) [[9](#ref-9)]

### Agreement forests and SPR distance

- **Espalier**: reconciles discordant trees and reconstructs ARGs with maximum agreement forests, defined as "the smallest possible set of topologically concordant subtrees present in both trees" <a id="cite-10"></a>[Rasmussen and Guo 2023](https://doi.org/10.1093/sysbio/syad040) [[10](#ref-10)]. It keeps discordance only where the sequences support it. It does not cite TreeKnit
- **Complexity**:
  - The rooted <a id="gloss-use-8"></a>SPR distance <sup>[8](#gloss-8)</sup> is NP-hard to compute <a id="cite-11"></a>[Bordewich and Semple 2005](https://doi.org/10.1007/s00026-004-0229-z) [[11](#ref-11)]
  - Whidden et al. model the SPR distance with maximum agreement forests and the hybridization number with maximum acyclic agreement forests, call both "NP-hard to compute", and give fixed-parameter algorithms <a id="cite-12a"></a>[Whidden et al. 2013](https://doi.org/10.1137/110845045) [[12](#ref-12)], extended to multifurcating trees <a id="cite-13a"></a>[Whidden et al. 2016](https://doi.org/10.1007/s00453-015-9983-z) [[13](#ref-13)]
  - The agreement forest goes back to <a id="cite-14"></a>[Hein et al. 1996](<https://doi.org/10.1016/S0166-218X(96)00062-5>) [[14](#ref-14)], whose proofs had "subtle mistakes" according to Whidden et al.
- **Relation to MCCs**: an agreement forest and a set of MCCs both split the leaves into blocks whose subtrees match in the two trees. The TreeKnit paper does not mention agreement forests, and no paper found in this survey states whether TreeKnit's MCCs form an agreement forest or how their number relates to the SPR distance. The TreeKnit paper notes that "MCCs are not necessarily clades in the segment trees, since they can be nested" <a id="cite-1d"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]

### Networks from more than two trees

TreeKnit compares each pair of trees separately and builds no ARG for more than two trees (see [`multi-segment-arg.md`](../proposals/multi-segment-arg.md)). The theory and the programs for one network of many trees are:

- **No agreement-forest characterization**: for two trees, maximum acyclic agreement forests give the hybridization number <a id="cite-12b"></a>[Whidden et al. 2013](https://doi.org/10.1137/110845045) [[12](#ref-12)]. For more trees, "characterising this number in terms of agreement forests for |P|>2 remains elusive". Linz and Semple characterize the number for any number of trees, also non-binary ones, with <a id="gloss-use-12"></a>cherry-picking sequences <sup>[12](#gloss-12)</sup> instead <a id="cite-26"></a>[Linz and Semple 2019](https://arxiv.org/abs/1712.04131) [[26](#ref-26)]
- **Three trees**: an algorithm for three binary trees runs in time $O(c^k \operatorname{poly}(n))$ for hybridization number $k$ and $n$ leaves. Its authors state that "the connection to acyclic agreement forests is much weaker for more than two trees, so even given the right agreement forest, the reconstruction of the network poses major challenges", and that one key lemma of their method does not extend to more than three trees <a id="cite-27"></a>[van Iersel et al. 2016](https://doi.org/10.1137/15M1036579) [[27](#ref-27)]
- **PIRN**: computes lower and upper bounds on the minimum number of reticulations for any number of trees. The bounds "match for many datasets (especially when the number of trees is small or reticulation level is low)", which solves those cases exactly <a id="cite-28"></a>[Wu 2010](https://doi.org/10.1093/bioinformatics/btq198) [[28](#ref-28)]
- **ALTS**: infers a minimum tree-child network, "for a set of up to 50 phylogenetic trees with 50 taxa that have only trivial common clusters in about a quarter of an hour on average" <a id="cite-29"></a>[Zhang et al. 2023](https://doi.org/10.1101/gr.277669.123) [[29](#ref-29)]
- **PhyloFusion and FHyNCH** (see [Network inference from gene trees](#network-inference-from-gene-trees)) also accept more than two trees
- **Reassortment methods**: no published method was found that combines pairwise comparisons of segment trees into one network, or that checks pairwise results for consistency across triplets of segments. TreeSort "processes each query segment independently and then combines the inferred reassortment annotations for each of the edges of the reference phylogeny" <a id="cite-17b"></a>[Markin et al. 2025](https://doi.org/10.1093/molbev/msaf133) [[17](#ref-17)]. CoalRe infers one network of all segments jointly
- **Comparison with a true network**: CoalRe counts a reassortment as recovered when "the subtree of each segment below that node is the same and if the relative direction of each segment at the reassortment event is exactly the same", a requirement that "becomes harder to satisfy as the number of segments increases" <a id="cite-15a"></a>[Müller et al. 2020](https://doi.org/10.1073/pnas.1918304117) [[15](#ref-15)]

### Bayesian coalescent with reassortment

- **CoalRe**: Bayesian inference of reassortment networks under the coalescent with reassortment, in BEAST 2 <a id="cite-15b"></a>[Müller et al. 2020](https://doi.org/10.1073/pnas.1918304117) [[15](#ref-15)]. It summarizes the posterior as a maximum clade credibility network
- **SCoRe**: the structured coalescent with reassortment, which adds migration between subpopulations <a id="cite-16"></a>[Stolz et al. 2021](https://doi.org/10.1093/molbev/msab342) [[16](#ref-16)]
- **Cost**: in the TreeKnit paper, CoalRe takes "several hours" against 40 ms for TreeKnit on 100 leaves <a id="cite-1e"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]

### Clock and feature-based detectors

- **TreeSort**: tests a molecular clock along a reference segment tree to find reassortment <a id="cite-17c"></a>[Markin et al. 2025](https://doi.org/10.1093/molbev/msaf133) [[17](#ref-17)]:
  - "TreeSort was the most accurate method on datasets with at least 1,000 strains"
  - "the accuracy of TreeSort and TreeKnit improved as the number of strains and the sampling density increased"
  - The study ran TreeKnit 0.5.8 (2024-08-09) with default settings on the true simulated gene trees. The F1 values are only in figures
- **VReassort**: deep learning on features of segment trees <a id="cite-18b"></a>[Cai et al. 2026](https://doi.org/10.1093/nar/gkag255) [[18](#ref-18)]:
  - It criticizes TreeKnit and CoalRe because "they do not directly specify which strains are recent reassortant"
  - It reports that TreeKnit "produced many false-positive reassortant strains because it is not robust to noise in phylogenetic trees, resulting in low precision"

## Limits of tree-based inference

- **Identifiability**: Lin et al. describe "invisible", "inaccurate", "reversed", and "obfuscated" reassortments <a id="cite-2c"></a>[Lin et al. 2024](https://doi.org/10.1093/molbev/msae078) [[2](#ref-2)]:
  - CoalRe and TreeKnit "tend to get the right number of reassortment events about 15% to 45% of the time" on simulations with 50 tips
  - They ran TreeKnit on pairs of a first-infection tree with each segment tree and added the counts
  - They suggest that topology-based methods "like those implemented in TreeKnit" could search the space of remove-and-rejoin moves, which on topology alone "is identical to the subtree-prune-regraft (SPR) method"
- **Phylogenetic noise**: segments simulated without reassortment still give about 32% as many SPR moves as the real avian influenza data, so tree discordance alone "can substantially overestimate" reassortment <a id="cite-19"></a>[Castelán-Sánchez and Poon 2025](https://doi.org/10.1101/2025.02.24.639875) [[19](#ref-19)]
- **Observable part of the ARG**: the TreeKnit paper states the same limit for its own method. Deep reassortments "leave almost no trace on the segment trees", and TreeKnit "infers only the observable part of the ARG" <a id="cite-1f"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]

## Polytomy handling

- **TreeKnit**: adds to each tree every split of the other tree that is compatible with it, then resolves inside the MCCs after inference <a id="cite-1g"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]
- **PhyloFusion**: "mutually refine all input trees with respect to each other", followed by its own refinement heuristic for cases that mutual refinement does not cover <a id="cite-9c"></a>[Zhang et al. 2025](https://doi.org/10.1093/sysbio/syaf049) [[9](#ref-9)]
- **TreeSort**: resolves multifurcations of the reference tree by parsimony on the concatenated other segments. On an H5N1 data set this reduced the number of inferred reassorted segments from 362 to 218 <a id="cite-17d"></a>[Markin et al. 2025](https://doi.org/10.1093/molbev/msaf133) [[17](#ref-17)]
- **<a id="gloss-use-9"></a>Common refinement <sup>[9](#gloss-9)</sup>**: when trees on the same leaves are compatible, their common refinement can be built in $O(k\,|X|)$ time for $k$ trees on leaf set $X$ <a id="cite-20"></a>[Schaller et al. 2021](https://doi.org/10.1186/s13015-021-00202-8) [[20](#ref-20)]. TreeKnit's pre-resolution adds splits one by one with a compatibility test
- **Agreement forests with polytomies**: two algorithms work on multifurcating trees directly <a id="cite-13b"></a>[Whidden et al. 2016](https://doi.org/10.1007/s00453-015-9983-z) [[13](#ref-13)]; <a id="cite-7b"></a>[Huson and Linz 2018](https://doi.org/10.1109/TCBB.2016.2537326) [[7](#ref-7)]

## ARG representation and viewers

- **Extended Newick**: a network written as one Newick string in which each <a id="gloss-use-10"></a>hybrid node <sup>[10](#gloss-10)</sup> appears once per parent, marked `#` with a type and an index <a id="cite-21"></a>[Cardona et al. 2008](https://doi.org/10.1186/1471-2105-9-532) [[21](#ref-21)]. TreeKnit writes this format with an additional `[&segments={...}]` annotation (see [`formats.md`](../feat/v0/formats.md))
- **IcyTree**: a browser viewer for trees and networks, "ancestral recombination graphs in particular" <a id="cite-22"></a>[Vaughan 2017](https://doi.org/10.1093/bioinformatics/btx155) [[22](#ref-22)]. The TreeKnit documentation recommends it for the ARG
- **Interval-based ARGs**: a 2024 formalism defines an ARG by genomes and their intervals of inheritance and fits the outputs of current recombination tools <a id="cite-23"></a>[Wong et al. 2024](https://doi.org/10.1093/genetics/iyae100) [[23](#ref-23)]. No source found applies it to segment reassortment

## Tanglegram layout

TreeKnit orders the output trees so that leaves of the same MCC face each other, with a greedy sort of the children at each node (see [`visualization.md`](../feat/v0/visualization.md)). Finding a tanglegram with the minimum number of crossings is NP-hard and fixed-parameter tractable, and it has no constant-factor approximation under the <a id="gloss-use-11"></a>Unique Games Conjecture <sup>[11](#gloss-11)</sup> <a id="cite-24"></a>[Buchin et al. 2012](https://doi.org/10.1007/s00453-010-9456-3) [[24](#ref-24)]. The NN-tanglegram heuristic extends tanglegrams to networks <a id="cite-25"></a>[Scornavacca et al. 2011](https://doi.org/10.1093/bioinformatics/btr210) [[25](#ref-25)]. TreeKnit's sort keeps the lines of each MCC from crossing each other and does not minimize the total number of crossings.

## Simulation in benchmarks

- **TreeKnit**: its own coalescent-with-reassortment simulator (ARGTools), with a parameter $c$ that removes short branches to imitate poor resolution. The value $c \approx 0.8$ matches A/H3N2 HA <a id="cite-1h"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]
- **TreeSort**: the CoalRe simulator with up to 5,000 strains and simulated sequences <a id="cite-17e"></a>[Markin et al. 2025](https://doi.org/10.1093/molbev/msaf133) [[17](#ref-17)]
- **Lin et al.**: a linear birth-death epidemic model with per-segment reassortment rates <a id="cite-2d"></a>[Lin et al. 2024](https://doi.org/10.1093/molbev/msae078) [[2](#ref-2)]
- **VReassort**: relocation of clades in real trees <a id="cite-18c"></a>[Cai et al. 2026](https://doi.org/10.1093/nar/gkag255) [[18](#ref-18)]
- **PhyloFusion**: random rooted SPR moves from a seed tree <a id="cite-9d"></a>[Zhang et al. 2025](https://doi.org/10.1093/sysbio/syaf049) [[9](#ref-9)]

The benchmarks use different simulators, tree sizes, and accuracy measures, so their numbers are not directly comparable.

## Gaps

- **MCC and agreement forest**: no source states the formal relation. Two sources that cite TreeKnit and might discuss it were not read: a chapter on phylogenetic networks (<https://doi.org/10.1016/b978-0-443-15750-9.00068-9>) and a 2026 review (<https://doi.org/10.1093/sysbio/syag045>)
- **Numbers in figures only**: this report gives the TreeKnit results of TreeSort, Lin et al., and VReassort from their text. The values in their figures were not extracted
- **VReassort supplement**: the supplementary figures with the TreeKnit numbers failed to download
- **More than two trees**: no publication describes or benchmarks TreeKnit's pipeline for more than two trees (see [`treeknit-paper-vs-code.md`](treeknit-paper-vs-code.md)). PhyloFusion and FHyNCH accept more than two trees and different taxon sets. The search for reassortment methods that combine pairwise comparisons of segment trees, or check them for consistency across triplets, found none. Some queries of that search failed, so this negative result is incomplete
- **Abstract-level sources**: the reference entries marked "Source read: abstract" support only the claims that their abstract states

## Glossary

1. <a id="gloss-1"></a> **ARG (ancestral reassortment graph).** A graph that holds the genealogies of all segments, with one node of two parents for each reassortment <a id="cite-1i"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]. [↩](#gloss-use-1)
2. <a id="gloss-2"></a> **Tanglegram.** A drawing of two trees facing each other, with a line between the two copies of each leaf. [↩](#gloss-use-2)
3. <a id="gloss-3"></a> **Hybridization number.** The minimum number of reticulation events in a network that displays the input trees <a id="cite-12c"></a>[Whidden et al. 2013](https://doi.org/10.1137/110845045) [[12](#ref-12)]. [↩](#gloss-use-3)
4. <a id="gloss-4"></a> **Agreement forest.** A set of subtrees that is a forest of both input trees. A maximum agreement forest has the fewest components <a id="cite-12d"></a>[Whidden et al. 2013](https://doi.org/10.1137/110845045) [[12](#ref-12)]. [↩](#gloss-use-4)
5. <a id="gloss-5"></a> **Fixed-parameter tractable.** Solvable in a time that is polynomial in the input size for each fixed value of a parameter, for example the distance between the two trees. [↩](#gloss-use-5)
6. <a id="gloss-6"></a> **Maximally compatible clade (MCC).** In TreeKnit, a largest set of leaves whose subtrees have the same topology in two segment trees. It is the region of the ARG that both segments share. [↩](#gloss-use-6)
7. <a id="gloss-7"></a> **Tree-child network.** A phylogenetic network in which every non-leaf node has at least one child that is not a hybrid node. [↩](#gloss-use-7)
8. <a id="gloss-8"></a> **SPR distance (subtree prune and regraft).** The minimum number of moves that cut a subtree and attach it elsewhere to turn one tree into the other <a id="cite-12e"></a>[Whidden et al. 2013](https://doi.org/10.1137/110845045) [[12](#ref-12)]. [↩](#gloss-use-8)
9. <a id="gloss-9"></a> **Common refinement.** A tree that contains every split of each input tree. It exists only when the input trees are compatible. [↩](#gloss-use-9)
10. <a id="gloss-10"></a> **Hybrid node.** A node of a phylogenetic network with more than one parent. In a reassortment graph, it marks the point where the segments of one virus have different parents. [↩](#gloss-use-10)
11. <a id="gloss-11"></a> **Unique Games Conjecture.** An unproven conjecture in complexity theory. Many results show that a problem has no efficient approximation if the conjecture holds. [↩](#gloss-use-11)
12. <a id="gloss-12"></a> **Cherry-picking sequence.** A sequence of steps that each remove a leaf from a cherry, a pair of sibling leaves, in a set of trees. Linz and Semple characterize the hybridization number of the trees by such sequences ([Linz and Semple 2019](https://arxiv.org/abs/1712.04131) [[26](#ref-26)]). [↩](#gloss-use-12)

## References

1. <a id="ref-1"></a> Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2022. "TreeKnit: Inferring ancestral reassortment graphs of influenza viruses." _PLOS Computational Biology_ 18:e1010394. https://doi.org/10.1371/journal.pcbi.1010394. Source read: full text [↩¹](#cite-1a) [↩²](#cite-1b) [↩³](#cite-1c) [↩⁴](#cite-1d) [↩⁵](#cite-1e) [↩⁶](#cite-1f) [↩⁷](#cite-1g) [↩⁸](#cite-1h) [↩⁹](#cite-1i)
2. <a id="ref-2"></a> Lin, Qianying, Emma E. Goldberg, Thomas Leitner, Carmen Molina-París, Aaron A. King, and Ethan O. Romero-Severson. 2024. "The number and pattern of viral genomic reassortments are not necessarily identifiable from segment trees." _Molecular Biology and Evolution_ 41:msae078. https://doi.org/10.1093/molbev/msae078. Source read: full text [↩¹](#cite-2a) [↩²](#cite-2b) [↩³](#cite-2c) [↩⁴](#cite-2d)
3. <a id="ref-3"></a> Nagarajan, Niranjan, and Carl Kingsford. 2011. "GiRaF: Robust, computational identification of influenza reassortments via graph mining." _Nucleic Acids Research_ 39:e34. Published online 2010. https://doi.org/10.1093/nar/gkq1232. Source read: abstract [↩](#cite-3)
4. <a id="ref-4"></a> Svinti, Victoria, James A. Cotton, and James O. McInerney. 2013. "New approaches for unravelling reassortment pathways." _BMC Evolutionary Biology_ 13:1. https://doi.org/10.1186/1471-2148-13-1. Source read: abstract [↩](#cite-4)
5. <a id="ref-5"></a> Markin, Alexey, Sanket Wagle, Tavis K. Anderson, and Oliver Eulenstein. 2022. "RF-Net 2: Fast inference of virus reassortment and hybridization networks." _Bioinformatics_ 38:2144-2152. https://doi.org/10.1093/bioinformatics/btac075. Source read: abstract [↩](#cite-5)
6. <a id="ref-6"></a> Albrecht, Benjamin, Celine Scornavacca, Alberto Cenci, and Daniel H. Huson. 2012. "Fast computation of minimum hybridization networks." _Bioinformatics_ 28:191-197. Published online 2011. https://doi.org/10.1093/bioinformatics/btr618. Source read: abstract [↩](#cite-6)
7. <a id="ref-7"></a> Huson, Daniel H., and Simone Linz. 2018. "Autumn algorithm: Computation of hybridization networks for realistic phylogenetic trees." _IEEE/ACM Transactions on Computational Biology and Bioinformatics_ 15:398-410. https://doi.org/10.1109/TCBB.2016.2537326. Source read: abstract [↩¹](#cite-7a) [↩²](#cite-7b)
8. <a id="ref-8"></a> Bernardini, Giulia, Leo van Iersel, Esther Julien, and Leen Stougie. 2024. "Inferring phylogenetic networks from multifurcating trees via cherry picking and machine learning." _Molecular Phylogenetics and Evolution_ 199:108137. https://doi.org/10.1016/j.ympev.2024.108137. Source read: abstract [↩](#cite-8)
9. <a id="ref-9"></a> Zhang, Louxin, Banu Cetinkaya, and Daniel H. Huson. 2025. "PhyloFusion: Fast and easy fusion of rooted phylogenetic trees into rooted phylogenetic networks." _Systematic Biology_ 75:88-99. https://doi.org/10.1093/sysbio/syaf049. Source read: full text [↩¹](#cite-9a) [↩²](#cite-9b) [↩³](#cite-9c) [↩⁴](#cite-9d)
10. <a id="ref-10"></a> Rasmussen, David A., and Fangfang Guo. 2023. "Espalier: Efficient tree reconciliation and ancestral recombination graphs reconstruction using maximum agreement forests." _Systematic Biology_ 72:1154-1170. https://doi.org/10.1093/sysbio/syad040. Source read: full text [↩](#cite-10)
11. <a id="ref-11"></a> Bordewich, Magnus, and Charles Semple. 2005. "On the computational complexity of the rooted subtree prune and regraft distance." _Annals of Combinatorics_ 8:409-423. https://doi.org/10.1007/s00026-004-0229-z. Source read: bibliographic record and citing literature [↩](#cite-11)
12. <a id="ref-12"></a> Whidden, Chris, Robert G. Beiko, and Norbert Zeh. 2013. "Fixed-parameter algorithms for maximum agreement forests." _SIAM Journal on Computing_ 42:1431-1466. https://doi.org/10.1137/110845045. Preprint: https://arxiv.org/abs/1108.2664. Source read: preprint full text [↩¹](#cite-12a) [↩²](#cite-12b) [↩³](#cite-12c) [↩⁴](#cite-12d) [↩⁵](#cite-12e)
13. <a id="ref-13"></a> Whidden, Chris, Robert G. Beiko, and Norbert Zeh. 2016. "Fixed-parameter and approximation algorithms for maximum agreement forests of multifurcating trees." _Algorithmica_ 74:1019-1054. Published online 2015. https://doi.org/10.1007/s00453-015-9983-z. Source read: abstract [↩¹](#cite-13a) [↩²](#cite-13b)
14. <a id="ref-14"></a> Hein, Jotun, Tao Jiang, Lusheng Wang, and Kaizhong Zhang. 1996. "On the complexity of comparing evolutionary trees." _Discrete Applied Mathematics_ 71:153-169. https://doi.org/10.1016/S0166-218X(96)00062-5. Source read: abstract [↩](#cite-14)
15. <a id="ref-15"></a> Müller, Nicola F., Ugnė Stolz, Gytis Dudas, Tanja Stadler, and Timothy G. Vaughan. 2020. "Bayesian inference of reassortment networks reveals fitness benefits of reassortment in human influenza viruses." _Proceedings of the National Academy of Sciences_ 117:17104-17111. https://doi.org/10.1073/pnas.1918304117. Source read: full text [↩¹](#cite-15a) [↩²](#cite-15b)
16. <a id="ref-16"></a> Stolz, Ugnė, Tanja Stadler, Nicola F. Müller, and Timothy G. Vaughan. 2021. "Joint inference of migration and reassortment patterns for viruses with segmented genomes." _Molecular Biology and Evolution_ 39:msab342. https://doi.org/10.1093/molbev/msab342. Source read: abstract [↩](#cite-16)
17. <a id="ref-17"></a> Markin, Alexey, Catherine A. Macken, Amy L. Baker, and Tavis K. Anderson. 2025. "Revealing reassortment in influenza A viruses with TreeSort." _Molecular Biology and Evolution_ 42:msaf133. https://doi.org/10.1093/molbev/msaf133. Source read: full text [↩¹](#cite-17a) [↩²](#cite-17b) [↩³](#cite-17c) [↩⁴](#cite-17d) [↩⁵](#cite-17e)
18. <a id="ref-18"></a> Cai, Dehan, Jiayu Shang, Cheng Peng, Herui Liao, Mang Shi, and Yanni Sun. 2026. "Fast and accurate identification of emerging viral reassortment from genome sequences." _Nucleic Acids Research_ 54:gkag255. https://doi.org/10.1093/nar/gkag255. Source read: full text without supplement [↩¹](#cite-18a) [↩²](#cite-18b) [↩³](#cite-18c)
19. <a id="ref-19"></a> Castelán-Sánchez, Hugo G., and Art F. Y. Poon. 2025. "Phylogenetic discordance can substantially overestimate genomic reassortment in avian influenza virus." _bioRxiv_ 2025.02.24.639875. https://doi.org/10.1101/2025.02.24.639875. Source read: preprint full text [↩](#cite-19)
20. <a id="ref-20"></a> Schaller, David, Marc Hellmuth, and Peter F. Stadler. 2021. "A simpler linear-time algorithm for the common refinement of rooted phylogenetic trees on a common leaf set." _Algorithms for Molecular Biology_ 16:23. https://doi.org/10.1186/s13015-021-00202-8. Source read: abstract [↩](#cite-20)
21. <a id="ref-21"></a> Cardona, Gabriel, Francesc Rosselló, and Gabriel Valiente. 2008. "Extended Newick: It is time for a standard representation of phylogenetic networks." _BMC Bioinformatics_ 9:532. https://doi.org/10.1186/1471-2105-9-532. Source read: abstract [↩](#cite-21)
22. <a id="ref-22"></a> Vaughan, Timothy G. 2017. "IcyTree: Rapid browser-based visualization for phylogenetic trees and networks." _Bioinformatics_ 33:2392-2394. https://doi.org/10.1093/bioinformatics/btx155. Source read: abstract [↩](#cite-22)
23. <a id="ref-23"></a> Wong, Yan, Anastasia Ignatieva, Jere Koskela, Gregor Gorjanc, Anthony W. Wohns, and Jerome Kelleher. 2024. "A general and efficient representation of ancestral recombination graphs." _Genetics_ 228:iyae100. https://doi.org/10.1093/genetics/iyae100. Source read: abstract [↩](#cite-23)
24. <a id="ref-24"></a> Buchin, Kevin, Maike Buchin, Jaroslaw Byrka, Martin Nöllenburg, Yoshio Okamoto, Rodrigo I. Silveira, and Alexander Wolff. 2012. "Drawing (complete) binary tanglegrams." _Algorithmica_ 62:309-332. Published online 2010. https://doi.org/10.1007/s00453-010-9456-3. Source read: abstract [↩](#cite-24)
25. <a id="ref-25"></a> Scornavacca, Celine, Franziska Zickmann, and Daniel H. Huson. 2011. "Tanglegrams for rooted phylogenetic trees and networks." _Bioinformatics_ 27:i248-i256. https://doi.org/10.1093/bioinformatics/btr210. Source read: abstract [↩](#cite-25)
26. <a id="ref-26"></a> Linz, Simone, and Charles Semple. 2019. "Attaching leaves and picking cherries to characterise the hybridisation number for a set of phylogenies." _Advances in Applied Mathematics_ 105:102-129. https://doi.org/10.1016/j.aam.2019.01.004. Preprint: https://arxiv.org/abs/1712.04131. Source read: abstract [↩](#cite-26)
27. <a id="ref-27"></a> van Iersel, Leo, Steven Kelk, Nela Lekić, Chris Whidden, and Norbert Zeh. 2016. "Hybridization number on three rooted binary trees is EPT." _SIAM Journal on Discrete Mathematics_ 30:1607-1631. https://doi.org/10.1137/15M1036579. Preprint: https://arxiv.org/abs/1402.2136. Source read: abstract [↩](#cite-27)
28. <a id="ref-28"></a> Wu, Yufeng. 2010. "Close lower and upper bounds for the minimum reticulate network of multiple phylogenetic trees." _Bioinformatics_ 26:i140-i148. https://doi.org/10.1093/bioinformatics/btq198. Source read: abstract [↩](#cite-28)
29. <a id="ref-29"></a> Zhang, Louxin, Niloufar Abhari, Caroline Colijn, and Yufeng Wu. 2023. "A fast and scalable method for inferring phylogenetic networks from trees by aligning lineage taxon strings." _Genome Research_ 33:1053-1060. https://doi.org/10.1101/gr.277669.123. Source read: abstract [↩](#cite-29)
