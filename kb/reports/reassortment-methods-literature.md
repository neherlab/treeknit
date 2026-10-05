# Reassortment inference from segment trees: methods related to TreeKnit

This report places TreeKnit among the published methods that infer reassortment or reticulate histories from trees. It covers the reassortment detectors that have been benchmarked against TreeKnit, the combinatorial problems behind its approach, polytomy handling, ARG formats, and tanglegram layout. The feature inventory in [`kb/feat/`](../feat/overview.md) describes TreeKnit.jl itself. [`treeknit-paper-vs-code.md`](treeknit-paper-vs-code.md) compares the TreeKnit paper with its code.

## Summary

- **Benchmarks are mixed to negative for TreeKnit outside its own paper**. The TreeKnit paper finds it about as accurate as the Bayesian method CoalRe on 100-leaf simulations and orders of magnitude faster <a id="cite-1a"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]. Later studies report that TreeSort is more accurate on 1,000 or more strains, that TreeKnit and CoalRe get the number of reassortments right only 15% to 45% of the time on 50-tip simulations, that a network method finds a lower <a id="gloss-use-4"></a>hybridization number <sup>[4](#gloss-4)</sup> in many cases, and that TreeKnit has low precision on noisy trees
- **Identifiability limits all tree-based methods**: the number and pattern of reassortments are "not necessarily identifiable" from segment trees alone <a id="cite-2a"></a>[Lin et al. 2024](https://doi.org/10.1093/molbev/msae078) [[2](#ref-2)]
- **Combinatorial relatives**: TreeKnit searches for compatible regions of two trees, a goal close to <a id="gloss-use-2"></a>agreement forests <sup>[2](#gloss-2)</sup> and minimum hybridization networks. Those problems are NP-hard and have exact fixed-parameter algorithms. No source states the formal relation between TreeKnit's MCCs and agreement forests
- **Polytomies**: TreeKnit adds the compatible splits of each tree to the other. Other methods use the same mutual refinement, a parsimony resolution on other segments, or agreement-forest algorithms for multifurcating trees
- **Name collision**: in BEAST and CoalRe, "MCC" means maximum clade credibility, not maximally compatible clade. Lin et al. use both meanings in one figure caption

## Method families

### Topology and compatibility heuristics

- **TreeKnit**: finds <a id="gloss-use-1"></a>maximally compatible clades <sup>[1](#gloss-1)</sup> by simulated annealing over clade removals, with a cost $\gamma$ per removed clade against the number of topological incompatibilities. It reads only two trees and needs the same leaves in both [[1](#ref-1)]
- **GiRaF**: "Graph-incompatibility-based Reassortment Finder", which identifies reassortments from samples of uncertain phylogenies by graph mining <a id="cite-3"></a>[Nagarajan and Kingsford 2011](https://doi.org/10.1093/nar/gkq1232) [[3](#ref-3)]. In the TreeKnit paper, GiRaF "misses reassortment events, but very rarely reports a reassortment that did not happen" [[1](#ref-1)]
- **MLreassort and Breassort**: algorithms that detect strains with a composite genomic architecture from incongruent segment trees <a id="cite-4"></a>[Svinti et al. 2013](https://doi.org/10.1186/1471-2148-13-1) [[4](#ref-4)]. Abstract only

### Network inference from gene trees

- **RF-Net 2**: infers reassortment and hybridization networks from gene trees with errors, using a Robinson-Foulds embedding cost <a id="cite-5"></a>[Markin et al. 2022](https://doi.org/10.1093/bioinformatics/btac075) [[5](#ref-5)]. Abstract only
- **Minimum hybridization networks**: Dendroscope 3 computes a representative set of minimum hybridization networks for two bifurcating trees <a id="cite-6"></a>[Albrecht et al. 2012](https://doi.org/10.1093/bioinformatics/btr618) [[6](#ref-6)]. The Autumn algorithm handles "realistic" trees with multifurcations and different taxa through maximum acyclic agreement forests <a id="cite-7"></a>[Huson and Linz 2018](https://doi.org/10.1109/TCBB.2016.2537326) [[7](#ref-7)]. FHyNCH is a heuristic for large sets of multifurcating trees <a id="cite-8"></a>[Bernardini et al. 2024](https://doi.org/10.1016/j.ympev.2024.108137) [[8](#ref-8)]. All abstract only
- **PhyloFusion**: fuses rooted trees into a tree-child network. Its benchmark on 100 tree pairs with 10 to 400 taxa states that "both TreeKnit and FHyNCH required only seconds", that "PhyloFusion performs better than TreeKnit in many cases" in hybridization number, and that "the TreeKnit program only accepts two input trees, and it does not accept missing taxa" (Results, Fig. 11) <a id="cite-9"></a>[Zhang et al. 2025](https://doi.org/10.1093/sysbio/syaf049) [[9](#ref-9)]. Full text read

### Agreement forests and SPR distance

- **Espalier**: reconciles discordant trees and reconstructs ARGs with maximum agreement forests, defined as "the smallest possible set of topologically concordant subtrees present in both trees" <a id="cite-10"></a>[Rasmussen and Guo 2023](https://doi.org/10.1093/sysbio/syad040) [[10](#ref-10)]. It keeps discordance only where the sequences support it. It does not cite TreeKnit. Full text read
- **Complexity**: the rooted <a id="gloss-use-3"></a>SPR distance <sup>[3](#gloss-3)</sup> is NP-hard to compute <a id="cite-11"></a>[Bordewich and Semple 2005](https://doi.org/10.1007/s00026-004-0229-z) [[11](#ref-11)]. Whidden et al. model the SPR distance with maximum agreement forests and the hybridization number with maximum acyclic agreement forests, call both "NP-hard to compute", and give fixed-parameter algorithms <a id="cite-12"></a>[Whidden et al. 2013](https://doi.org/10.1137/110845045) [[12](#ref-12)], extended to multifurcating trees <a id="cite-13"></a>[Whidden et al. 2016](https://doi.org/10.1007/s00453-015-9983-z) [[13](#ref-13)]. The agreement forest goes back to <a id="cite-14"></a>[Hein et al. 1996](<https://doi.org/10.1016/S0166-218X(96)00062-5>) [[14](#ref-14)], whose proofs had "subtle mistakes" according to Whidden et al.
- **Relation to MCCs**: an agreement forest and a set of MCCs both split the leaves into blocks whose subtrees match in the two trees. The TreeKnit paper does not mention agreement forests, and no paper found in this survey states whether TreeKnit's MCCs form an agreement forest or how their number relates to the SPR distance. The TreeKnit paper notes that "MCCs are not necessarily clades in the segment trees, since they can be nested" [[1](#ref-1)]

### Bayesian coalescent with reassortment

- **CoalRe**: Bayesian inference of reassortment networks under the coalescent with reassortment, in BEAST 2 <a id="cite-15"></a>[Müller et al. 2020](https://doi.org/10.1073/pnas.1918304117) [[15](#ref-15)]. It summarizes the posterior as a maximum clade credibility network. Full text read
- **SCoRe**: the structured coalescent with reassortment, which adds migration between subpopulations <a id="cite-16"></a>[Stolz et al. 2021](https://doi.org/10.1093/molbev/msab342) [[16](#ref-16)]. Abstract only
- **Cost**: in the TreeKnit paper, CoalRe takes "several hours" against 40 ms for TreeKnit on 100 leaves [[1](#ref-1)]

### Clock and feature-based detectors

- **TreeSort**: tests a molecular clock along a reference segment tree to find reassortment. "TreeSort was the most accurate method on datasets with at least 1,000 strains", and "the accuracy of TreeSort and TreeKnit improved as the number of strains and the sampling density increased" <a id="cite-17"></a>[Markin et al. 2025](https://doi.org/10.1093/molbev/msaf133) [[17](#ref-17)]. The study ran TreeKnit v0.5.8 with default settings on the true simulated gene trees. Full text read; the F1 values are only in figures
- **VReassort**: deep learning on features of segment trees. It criticizes TreeKnit and CoalRe because "they do not directly specify which strains are recent reassortant", and reports that TreeKnit "produced many false-positive reassortant strains because it is not robust to noise in phylogenetic trees, resulting in low precision" <a id="cite-18"></a>[Cai et al. 2026](https://doi.org/10.1093/nar/gkag255) [[18](#ref-18)]. Full text read; the supplementary figures with the TreeKnit numbers could not be downloaded

## Limits of tree-based inference

- **Identifiability**: Lin et al. describe "invisible", "inaccurate", "reversed", and "obfuscated" reassortments and show that CoalRe and TreeKnit "tend to get the right number of reassortment events about 15% to 45% of the time" on simulations with 50 tips [[2](#ref-2)]. They ran TreeKnit on pairs of a first-infection tree with each segment tree and added the counts. They suggest that topology-based methods "like those implemented in TreeKnit" could search the space of remove-and-rejoin moves, which on topology alone "is identical to the subtree-prune-regraft (SPR) method"
- **Phylogenetic noise**: segments simulated without reassortment still give about 32% as many SPR moves as the real avian influenza data, so tree discordance alone can strongly overestimate reassortment <a id="cite-19"></a>[Castelán-Sánchez and Poon 2025](https://doi.org/10.1101/2025.02.24.639875) [[19](#ref-19)]. Preprint, full text read
- **What TreeKnit sees**: the TreeKnit paper states the same limit for its own method: deep reassortments "leave almost no trace on the segment trees", and TreeKnit "infers only the observable part of the ARG" [[1](#ref-1)]

## Polytomy handling

- **TreeKnit**: adds to each tree every split of the other tree that is compatible with it, then resolves inside the MCCs after inference [[1](#ref-1)]
- **PhyloFusion**: "mutually refine all input trees with respect to each other", followed by its own refinement heuristic for cases that mutual refinement does not cover [[9](#ref-9)]
- **TreeSort**: resolves multifurcations of the reference tree by parsimony on the concatenated other segments, which reduced the number of inferred reassorted segments on an H5N1 data set from 362 to 218 [[17](#ref-17)]
- **Common refinement**: when trees on the same leaves are compatible, their common refinement can be built in $O(k\,|L|)$ time for $k$ trees on leaf set $L$ <a id="cite-20"></a>[Schaller et al. 2021](https://doi.org/10.1186/s13015-021-00202-8) [[20](#ref-20)]. Abstract only. TreeKnit's pre-resolution adds splits one by one with a compatibility test instead
- **Agreement forests with polytomies**: Whidden et al. 2016 and the Autumn algorithm work on multifurcating trees directly [[13](#ref-13)] [[7](#ref-7)]

## ARG representation and viewers

- **Extended Newick**: a network written as one Newick string in which each hybrid node appears once per parent, marked `#` with a type and an index <a id="cite-21"></a>[Cardona et al. 2008](https://doi.org/10.1186/1471-2105-9-532) [[21](#ref-21)]. TreeKnit writes this format with an additional `[&segments={...}]` annotation (see [`formats.md`](../feat/formats.md))
- **IcyTree**: a browser viewer for trees and networks, "ancestral recombination graphs in particular" <a id="cite-22"></a>[Vaughan 2017](https://doi.org/10.1093/bioinformatics/btx155) [[22](#ref-22)]. The TreeKnit documentation recommends it for the ARG
- **Interval-based ARGs**: a recent formalism defines an ARG by genomes and their intervals of inheritance and fits the outputs of current recombination tools <a id="cite-23"></a>[Wong et al. 2024](https://doi.org/10.1093/genetics/iyae100) [[23](#ref-23)]. Abstract only. No source found applies it to segment reassortment

## Tanglegram layout

TreeKnit orders the output trees so that leaves of the same MCC face each other, with a greedy sort of the children at each node (see [`visualization.md`](../feat/visualization.md)). Finding a tanglegram with the minimum number of crossings is NP-hard, fixed-parameter tractable, and has no constant-factor approximation under the Unique Games Conjecture <a id="cite-24"></a>[Buchin et al. 2012](https://doi.org/10.1007/s00453-010-9456-3) [[24](#ref-24)]. The NN-tanglegram heuristic extends tanglegrams to networks <a id="cite-25"></a>[Scornavacca et al. 2011](https://doi.org/10.1093/bioinformatics/btr210) [[25](#ref-25)]. Both are abstract only. TreeKnit's sort does not minimize the total number of crossings; it only keeps the lines of each MCC from crossing each other.

## Simulation in benchmarks

- **TreeKnit**: its own coalescent-with-reassortment simulator (ARGTools), with a parameter $c$ that removes short branches to imitate poor resolution; $c \approx 0.8$ matches A/H3N2 HA [[1](#ref-1)]
- **TreeSort**: the CoalRe simulator with up to 5,000 strains and simulated sequences [[17](#ref-17)]
- **Lin et al.**: a linear birth-death epidemic model with per-segment reassortment rates [[2](#ref-2)]
- **VReassort**: relocation of clades in real trees [[18](#ref-18)]
- **PhyloFusion**: random rooted SPR moves from a seed tree [[9](#ref-9)]

The benchmarks use different simulators, tree sizes, and accuracy measures, so their numbers are not directly comparable.

## Gaps

- **MCC and agreement forest**: no source states the formal relation. Two sources that cite TreeKnit and might discuss it were not read: a chapter on phylogenetic networks (10.1016/b978-0-443-15750-9.00068-9) and a 2026 review (10.1093/sysbio/syag045)
- **Numbers in figures only**: the TreeKnit results of TreeSort, Lin et al., and VReassort are reported here from the text; the values in the figures were not extracted
- **More than two trees**: no publication describes or benchmarks TreeKnit's pipeline for more than two trees (see [`treeknit-paper-vs-code.md`](treeknit-paper-vs-code.md)). PhyloFusion and FHyNCH accept more than two trees and different taxon sets
- **Abstract-only sources**: the claims marked "abstract only" rest on the abstract and the bibliographic record

## Glossary

1. <a id="gloss-1"></a> **Maximally compatible clade (MCC).** In TreeKnit, a maximal set of leaves whose subtrees have the same topology in two segment trees; the region of the ARG shared by both segments [[1](#ref-1)]. Not to be confused with maximum clade credibility in Bayesian phylogenetics. [↩](#gloss-use-1)
2. <a id="gloss-2"></a> **Agreement forest.** A set of subtrees that is a forest of both input trees; a maximum agreement forest has the fewest components [[12](#ref-12)]. [↩](#gloss-use-2)
3. <a id="gloss-3"></a> **SPR (subtree prune and regraft).** A tree move that cuts a subtree and attaches it elsewhere; the SPR distance is the minimum number of such moves between two trees [[12](#ref-12)]. [↩](#gloss-use-3)
4. <a id="gloss-4"></a> **Hybridization number.** The minimum number of reticulation events in a network that displays the input trees [[12](#ref-12)]. [↩](#gloss-use-4)

## References

1. <a id="ref-1"></a> Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2022. "TreeKnit: Inferring Ancestral Reassortment Graphs of Influenza Viruses." _PLOS Computational Biology_ 18 (8): e1010394. https://doi.org/10.1371/journal.pcbi.1010394 [↩](#cite-1a)
2. <a id="ref-2"></a> Lin, Qianying, Emma E. Goldberg, Thomas Leitner, Carmen Molina-París, Aaron A. King, and Ethan O. Romero-Severson. 2024. "The Number and Pattern of Viral Genomic Reassortments Are Not Necessarily Identifiable from Segment Trees." _Molecular Biology and Evolution_ 41 (6): msae078. https://doi.org/10.1093/molbev/msae078 [↩](#cite-2a)
3. <a id="ref-3"></a> Nagarajan, Niranjan, and Carl Kingsford. 2011. "GiRaF: Robust, Computational Identification of Influenza Reassortments via Graph Mining." _Nucleic Acids Research_ 39 (6): e34. Published online 2010. https://doi.org/10.1093/nar/gkq1232 [↩](#cite-3)
4. <a id="ref-4"></a> Svinti, Victoria, James A. Cotton, and James O. McInerney. 2013. "New Approaches for Unravelling Reassortment Pathways." _BMC Evolutionary Biology_ 13 (1): 1. https://doi.org/10.1186/1471-2148-13-1 [↩](#cite-4)
5. <a id="ref-5"></a> Markin, Alexey, Sanket Wagle, Tavis K. Anderson, and Oliver Eulenstein. 2022. "RF-Net 2: Fast Inference of Virus Reassortment and Hybridization Networks." _Bioinformatics_ 38 (8): 2144-2152. https://doi.org/10.1093/bioinformatics/btac075 [↩](#cite-5)
6. <a id="ref-6"></a> Albrecht, Benjamin, Celine Scornavacca, Alberto Cenci, and Daniel H. Huson. 2012. "Fast Computation of Minimum Hybridization Networks." _Bioinformatics_ 28 (2): 191-197. Published online 2011. https://doi.org/10.1093/bioinformatics/btr618 [↩](#cite-6)
7. <a id="ref-7"></a> Huson, Daniel H., and Simone Linz. 2018. "Autumn Algorithm -- Computation of Hybridization Networks for Realistic Phylogenetic Trees." _IEEE/ACM Transactions on Computational Biology and Bioinformatics_ 15 (2): 398-410. https://doi.org/10.1109/TCBB.2016.2537326 [↩](#cite-7)
8. <a id="ref-8"></a> Bernardini, Giulia, Leo van Iersel, Esther Julien, and Leen Stougie. 2024. "Inferring Phylogenetic Networks from Multifurcating Trees via Cherry Picking and Machine Learning." _Molecular Phylogenetics and Evolution_ 199: 108137. https://doi.org/10.1016/j.ympev.2024.108137 [↩](#cite-8)
9. <a id="ref-9"></a> Zhang, Louxin, Banu Cetinkaya, and Daniel H. Huson. 2025. "PhyloFusion -- Fast and Easy Fusion of Rooted Phylogenetic Trees into Rooted Phylogenetic Networks." _Systematic Biology_ 75 (1): 88-99. https://doi.org/10.1093/sysbio/syaf049 [↩](#cite-9)
10. <a id="ref-10"></a> Rasmussen, David A., and Fangfang Guo. 2023. "Espalier: Efficient Tree Reconciliation and Ancestral Recombination Graphs Reconstruction Using Maximum Agreement Forests." _Systematic Biology_ 72 (5): 1154-1170. https://doi.org/10.1093/sysbio/syad040 [↩](#cite-10)
11. <a id="ref-11"></a> Bordewich, Magnus, and Charles Semple. 2005. "On the Computational Complexity of the Rooted Subtree Prune and Regraft Distance." _Annals of Combinatorics_ 8 (4): 409-423. https://doi.org/10.1007/s00026-004-0229-z [↩](#cite-11)
12. <a id="ref-12"></a> Whidden, Chris, Robert G. Beiko, and Norbert Zeh. 2013. "Fixed-Parameter Algorithms for Maximum Agreement Forests." _SIAM Journal on Computing_ 42 (4): 1431-1466. https://doi.org/10.1137/110845045. Preprint: https://arxiv.org/abs/1108.2664 [↩](#cite-12)
13. <a id="ref-13"></a> Whidden, Chris, Robert G. Beiko, and Norbert Zeh. 2016. "Fixed-Parameter and Approximation Algorithms for Maximum Agreement Forests of Multifurcating Trees." _Algorithmica_ 74 (3): 1019-1054. Published online 2015. https://doi.org/10.1007/s00453-015-9983-z [↩](#cite-13)
14. <a id="ref-14"></a> Hein, Jotun, Tao Jiang, Lusheng Wang, and Kaizhong Zhang. 1996. "On the Complexity of Comparing Evolutionary Trees." _Discrete Applied Mathematics_ 71 (1-3): 153-169. https://doi.org/10.1016/S0166-218X(96)00062-5 [↩](#cite-14)
15. <a id="ref-15"></a> Müller, Nicola F., Ugnė Stolz, Gytis Dudas, Tanja Stadler, and Timothy G. Vaughan. 2020. "Bayesian Inference of Reassortment Networks Reveals Fitness Benefits of Reassortment in Human Influenza Viruses." _Proceedings of the National Academy of Sciences_ 117 (29): 17104-17111. https://doi.org/10.1073/pnas.1918304117 [↩](#cite-15)
16. <a id="ref-16"></a> Stolz, Ugnė, Tanja Stadler, Nicola F. Müller, and Timothy G. Vaughan. 2021. "Joint Inference of Migration and Reassortment Patterns for Viruses with Segmented Genomes." _Molecular Biology and Evolution_ 39 (1): msab342. https://doi.org/10.1093/molbev/msab342 [↩](#cite-16)
17. <a id="ref-17"></a> Markin, Alexey, Catherine A. Macken, Amy L. Baker, and Tavis K. Anderson. 2025. "Revealing Reassortment in Influenza A Viruses with TreeSort." _Molecular Biology and Evolution_ 42 (8): msaf133. https://doi.org/10.1093/molbev/msaf133 [↩](#cite-17)
18. <a id="ref-18"></a> Cai, Dehan, Jiayu Shang, Cheng Peng, Herui Liao, Mang Shi, and Yanni Sun. 2026. "Fast and Accurate Identification of Emerging Viral Reassortment from Genome Sequences." _Nucleic Acids Research_ 54 (6): gkag255. https://doi.org/10.1093/nar/gkag255 [↩](#cite-18)
19. <a id="ref-19"></a> Castelán-Sánchez, Hugo G., and Art F. Y. Poon. 2025. "Phylogenetic Discordance Can Substantially Overestimate Genomic Reassortment in Avian Influenza Virus." _bioRxiv_ 2025.02.24.639875. https://doi.org/10.1101/2025.02.24.639875 [↩](#cite-19)
20. <a id="ref-20"></a> Schaller, David, Marc Hellmuth, and Peter F. Stadler. 2021. "A Simpler Linear-Time Algorithm for the Common Refinement of Rooted Phylogenetic Trees on a Common Leaf Set." _Algorithms for Molecular Biology_ 16 (1): 23. https://doi.org/10.1186/s13015-021-00202-8 [↩](#cite-20)
21. <a id="ref-21"></a> Cardona, Gabriel, Francesc Rosselló, and Gabriel Valiente. 2008. "Extended Newick: It Is Time for a Standard Representation of Phylogenetic Networks." _BMC Bioinformatics_ 9: 532. https://doi.org/10.1186/1471-2105-9-532 [↩](#cite-21)
22. <a id="ref-22"></a> Vaughan, Timothy G. 2017. "IcyTree: Rapid Browser-Based Visualization for Phylogenetic Trees and Networks." _Bioinformatics_ 33 (15): 2392-2394. https://doi.org/10.1093/bioinformatics/btx155 [↩](#cite-22)
23. <a id="ref-23"></a> Wong, Yan, Anastasia Ignatieva, Jere Koskela, Gregor Gorjanc, Anthony W. Wohns, and Jerome Kelleher. 2024. "A General and Efficient Representation of Ancestral Recombination Graphs." _Genetics_ 228 (1): iyae100. https://doi.org/10.1093/genetics/iyae100 [↩](#cite-23)
24. <a id="ref-24"></a> Buchin, Kevin, Maike Buchin, Jaroslaw Byrka, Martin Nöllenburg, Yoshio Okamoto, Rodrigo I. Silveira, and Alexander Wolff. 2012. "Drawing (Complete) Binary Tanglegrams." _Algorithmica_ 62 (1-2): 309-332. Published online 2010. https://doi.org/10.1007/s00453-010-9456-3 [↩](#cite-24)
25. <a id="ref-25"></a> Scornavacca, Celine, Franziska Zickmann, and Daniel H. Huson. 2011. "Tanglegrams for Rooted Phylogenetic Trees and Networks." _Bioinformatics_ 27 (13): i248-i256. https://doi.org/10.1093/bioinformatics/btr210 [↩](#cite-25)
