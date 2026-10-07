# One ARG for three or more segment trees

For two segment trees, TreeKnit builds an ARG from the MCCs of the pair. For three or more trees, TreeKnit.jl and this port report the MCCs of every pair and build no ARG. TreeKnit.jl gives the reason: "MultiTreeKnit may still return MCCs that are inconsistent with each other, which prevents the construction of an ARG. Therefore, we do not reconstruct an ARG for more than two trees" [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/multitreeknit.md?plain=1#L7)]. A literature search found no published method that combines pairwise comparisons of segment trees into one network (see [`reassortment-methods-literature.md`](../reports/reassortment-methods-literature.md#networks-from-more-than-two-trees)). This proposal describes what one ARG of $K \ge 3$ segments needs, the conflicts measured in the port, the open questions, and the work items.

## What the ARG would add

Pairwise MCCs already tell, for each pair of segments, which strains share their history. One ARG of all segments would also give:

- **Reassortment events with their segments**: one count of events, each with the groups of segments that it separates. Pairwise counts cannot give this, because one event that separates segment groups $A$ and $B$ appears in $|A| \times |B|$ pairs
- **One figure** of the genealogy of all segments
- **One set of shared branches**: each branch with the set of segments that use it

The main downstream consumer works without the ARG. TreeTime pull request [#961](https://github.com/neherlab/treetime/pull/961) analyzes each segment tree in turn as the focal tree, with the pairwise MCCs of that tree only (see [`treeknit-ecosystem.md`](../reports/treeknit-ecosystem.md#downstream-users)). TreeSort maps reassortments onto one reference tree (see [`reassortment-methods-literature.md`](../reports/reassortment-methods-literature.md#clock-and-feature-based-detectors)). The intended use of the ARG therefore decides what counts as a correct result, and a first implementation needs that criterion before it can be evaluated.

## Conditions

An ARG of $K$ segments has one embedded tree per segment, and the regions that two segments share in it are the MCCs of their pair. Pairwise MCCs can come from one ARG only if they satisfy these conditions:

- **Topological compatibility**: each MCC has the same topology in its two output trees. `Resolution::Matched` guarantees this for every pair (see [README section "Resolving trees"](../../README.md#resolving-trees))
- **Triplet condition**: $P_{ij} \wedge P_{ik} \le P_{jk}$ for every triplet of trees and every orientation, where $P_{ij}$ is the partition of the leaves into the MCCs of trees $i$ and $j$ (see [`cross-pair-mcc-consistency.md`](cross-pair-mcc-consistency.md#the-condition))
- **Consistent joining**: when the shared nodes of all pairs are joined transitively, no class of joined nodes contains two nodes of the same tree
- **Acyclicity**: the joined graph has no directed cycle. The MCCs of each pair form an <a id="gloss-use-1"></a>acyclic agreement forest <sup>[1](#gloss-1)</sup> (see [`agreement-forest-validation.md`](agreement-forest-validation.md)), but no argument shows that the graph joined over all pairs stays acyclic

The first two conditions are necessary. Whether the four conditions together are sufficient is not known.

The theory of two trees does not carry over to more trees. For two trees, the minimum number of reticulations that explain both trees equals the number of blocks of a maximum acyclic agreement forest minus one <a id="cite-1"></a>[Baroni et al. 2005](https://doi.org/10.1007/s00285-005-0315-9) [[1](#ref-1)]. For more trees, "characterising this number in terms of agreement forests for |P|>2 remains elusive" <a id="cite-2"></a>[Linz and Semple 2019](https://arxiv.org/abs/1712.04131) [[2](#ref-2)], and "the connection to acyclic agreement forests is much weaker for more than two trees, so even given the right agreement forest, the reconstruction of the network poses major challenges" <a id="cite-3"></a>[van Iersel et al. 2016](https://doi.org/10.1137/15M1036579) [[3](#ref-3)].

## Conflicts measured in the port

[`cross-pair-mcc-consistency.md`](cross-pair-mcc-consistency.md#measured-inconsistency) gives the counts. In summary:

- **Present in simulated and real data**: with the default settings, the triplet condition fails on 3 of the 4 simulated three-tree fixtures for at least one of five seeds. On the four segments of `data/h3n2-2k-4-segments`, it fails in all four triplets with both settings measured
- **Caused by separate pairs**: on the fully resolved fixture `sim_k3_n50_r0.05`, the failures are the same for every seed and every resolution setting, and most other counts are the same for every seed. Each pair is optimized without the MCCs of the other pairs, and that causes them. The randomness of the annealing and the resolution settings do not
- **A signal of errors**: on the fully resolved fixture `sim_k3_n50_r0.05`, the pair (tree1, tree2) has one MCC fewer than the true MCCs, and the pair (tree1, tree3) has as many MCCs as the truth but a different partition. The pair (tree2, tree3) equals the truth. Each pair alone is plausible, and the conflict between them marks an error in some pair, which a joint method could use

## Open questions

### Which pair is wrong

A conflict between pairs can be removed in two ways:

- **Merge MCCs of one pair**: this gives fewer reassortments, but the merged MCC can have incompatible topologies in its two trees
- **Split MCCs of another pair**: this gives more reassortments. Splitting every pair to the <a id="gloss-use-2"></a>meet <sup>[2](#gloss-2)</sup> of the other pairs removes all conflicts. It also moves all pairs toward one common partition, in which every reassortment separates all segments, so the groups of segments that moved together are lost

No known algorithm finds the smallest change that satisfies the triplet condition and keeps each pair an acyclic agreement forest of its two trees.

### Score for K segments

- One reassortment that separates segment groups $A$ and $B$ appears in $|A| \times |B|$ pairs: 7 pairs for a 1|7 split of the 8 segments of influenza A, 16 pairs for a 4|4 split. The cost $\gamma$ of one removed MCC in a pair therefore has no single meaning for an ARG
- A joint score, $\gamma$ times the number of reassortment nodes of the ARG plus the incompatibilities, is not a sum of pair scores, so it needs a search over all trees together
- TreeKnit.jl added to the annealing a penalty for removals that the other pairs contradict, and removed it because it had "little effect on results" (see [`cross-pair-mcc-consistency.md`](cross-pair-mcc-consistency.md#history-in-treeknitjl)). A hard constraint was not tested

### Several ARGs for the same MCCs

- **Order of events at one node**: when the segments of one node have three or more different parents, with no coalescence between the events, the order of the events cannot be observed. The output can be one node with more than two parents, or a chain of two-parent nodes in an arbitrary order. IcyTree reads the first form, while ARGTools simulates and CoalRe writes only nodes with two parents (see [`treeknit-ecosystem.md`](../reports/treeknit-ecosystem.md#extended-newick-arg))
- **Identifiability**: the number and pattern of reassortments are "not necessarily identifiable" from segment trees <a id="cite-4"></a>[Lin et al. 2024](https://doi.org/10.1093/molbev/msae078) [[4](#ref-4)]. That study uses two segments. No result for more segments was found

### Position of reassortment nodes

The ARG of two trees places a reassortment halfway along the shorter of the two branches above an MCC root (`fn set_branch_lengths()`, [packages/treeknit-core/src/arg.rs#L490-L492](../../packages/treeknit-core/src/arg.rs#L490-L492)). With several events on one branch and branch lengths from $K$ trees, no rule gives their order and heights.

### Comparison with a true ARG

CoalRe counts a true reassortment as recovered when "the subtree of each segment below that node is the same and if the relative direction of each segment at the reassortment event is exactly the same". Its authors note that this requirement "becomes harder to satisfy as the number of segments increases" <a id="cite-5"></a>[Müller et al. 2020](https://doi.org/10.1073/pnas.1918304117) [[5](#ref-5)]. A metric must be chosen before inferred and true ARGs can be compared.

### Size on real data

The run on the four segments of `data/h3n2-2k-4-segments` gives 606 to 1067 MCCs per pair for 1997 leaves, so an ARG of these MCCs would have hundreds of reassortment nodes. TreeKnit takes every internal node of the input trees as a hard constraint, so poorly supported branches add reassortments (see [`reassortment-methods-literature.md`](../reports/reassortment-methods-literature.md#limits-of-tree-based-inference)). A figure of the 8 segments of influenza A can have up to 255 different segment sets on its branches.

## Problems in the current code

- **Two segments in the types**: `pub struct ArgNode` and `pub struct Arg` store parents, branch lengths, roots, and trees in arrays of length 2 ([packages/treeknit-core/src/arg.rs#L30-L51](../../packages/treeknit-core/src/arg.rs#L30-L51)). `pub struct ArgNodeView`, which the web app receives through WebAssembly, has the same arrays ([packages/treeknit-io/src/display.rs#L445-L474](../../packages/treeknit-io/src/display.rs#L445-L474)), so a change also changes the generated TypeScript declarations. The extended Newick writer writes a fixed `GlobalRoot[&segments={0,1}]` ([packages/treeknit-io/src/arg.rs#L14-L30](../../packages/treeknit-io/src/arg.rs#L14-L30)), the SVG figure takes two segment names ([packages/treeknit-io/src/figure/arg.rs#L13](../../packages/treeknit-io/src/figure/arg.rs#L13)), and the web app draws the ARG of two segments (`packages/web/src/arg/`)
- **Construction from one pair**: `pub fn arg_from_trees()` builds the ARG of the first tree and maps the second tree onto it through the shared nodes of the pair ([packages/treeknit-core/src/arg.rs#L140-L170](../../packages/treeknit-core/src/arg.rs#L140-L170)). A third tree needs partner nodes from two pairs, and these must agree
- **Tree changes during construction**: construction resolves both trees liberally with the MCCs and inserts shared singletons (`fn fix_shared_singletons()`, [packages/treeknit-core/src/arg.rs#L277-L331](../../packages/treeknit-core/src/arg.rs#L277-L331)). With $K$ trees, a change made for one pair alters a tree of other pairs. This is the problem that consistent resolution solves for MultiTreeKnit (see [`treeknit-and-multitreeknit.md`](../reports/treeknit-and-multitreeknit.md#consistent-resolution))
- **Input order**: `Resolution::Matched` gives earlier trees precedence when splits conflict ([packages/treeknit-core/src/pipeline.rs#L273-L274](../../packages/treeknit-core/src/pipeline.rs#L273-L274)), so its MCCs, and an ARG built from them, depend on the order of the input trees
- **Different leaf sets**: the ARG of two trees already fails after imputation in some cases ([`M-arg-fails-after-imputation.md`](../issues/M-arg-fails-after-imputation.md)). With $K$ trees, a leaf can lack several segments, and the triplet condition can be checked only on the leaves present in all three trees
- **Open defects of the ARG of two trees**: [`N-arg-acyclicity-unchecked.md`](../issues/N-arg-acyclicity-unchecked.md), [`M-resolution-and-arg-cubic-on-deep-trees.md`](../issues/M-resolution-and-arg-cubic-on-deep-trees.md), and [`M-arg-labels-collide-with-leaf-names.md`](../issues/M-arg-labels-collide-with-leaf-names.md)
- **Two sources of truth**: after a repair, `MCCs.json` and the ARG can disagree. Deriving the pairwise MCCs from the ARG, as ARGTools derives the true MCCs of a simulation, keeps them equal

## Design axes

### Construction

- **Join the shared nodes of all pairs (recommended first)**: a <a id="gloss-use-3"></a>union-find <sup>[3](#gloss-3)</sup> over the nodes of all trees joins the nodes that a pair marks as shared. A class with two nodes of one tree, and a cycle, are reported as conflicts and not repaired. Run on the true MCCs of simulations, this tests whether the conditions above are sufficient, without inference errors
- **Joint inference**: one search over all trees with a joint score (see "Score for K segments")
- **No ARG**: keep the pairwise MCCs and report conflicts only

### Conflict handling

- **Report only (recommended first)**: the diagnostic of [`N-cross-pair-mcc-consistency-unreported.md`](../issues/N-cross-pair-mcc-consistency-unreported.md), which changes no result
- **Merge**: join MCCs that the other pairs join, after a topology check
- **Split**: split MCCs to the meet of the other pairs
- **Joint score**: infer all pairs together

### Events at one node

- **One node with more than two parents**: records only what the trees show
- **A chain of two-parent nodes** in a fixed order, for example by segment index: matches the simulators and CoalRe, and adds an order that the data do not give

## Validation

- **Simulation truth**: ARGTools simulates ARGs of any number of segments and writes them in TreeKnit's extended Newick syntax, but the fixtures do not store them yet ([`N-fixtures-lack-true-arg.md`](../issues/N-fixtures-lack-true-arg.md))
- **Properties of any ARG**: each embedded segment tree equals its resolved input tree, as the test of the ARG of two trees checks; the pairwise MCCs derived from the ARG equal the reported MCCs; the ARG is acyclic

## Work items

- [`N-cross-pair-mcc-consistency-unreported.md`](../issues/N-cross-pair-mcc-consistency-unreported.md)
- [`N-fixtures-lack-true-arg.md`](../issues/N-fixtures-lack-true-arg.md)
- [`N-arg-limited-to-two-trees.md`](../issues/N-arg-limited-to-two-trees.md)

## Glossary

1. <a id="gloss-1"></a> **Acyclic agreement forest.** A partition of the leaves of two trees into blocks whose subtrees are identical in both trees and do not overlap in either tree, with no cycle in the ancestor relation between blocks that the two trees define together. [↩](#gloss-use-1)
2. <a id="gloss-2"></a> **Meet of partitions.** The partition in which two elements share a block exactly when they share a block in every given partition. [↩](#gloss-use-2)
3. <a id="gloss-3"></a> **Union-find.** A data structure that keeps a partition of elements into classes and joins two classes in almost constant time. [↩](#gloss-use-3)

## References

1. <a id="ref-1"></a> Baroni, Mihaela, Stefan Grünewald, Vincent Moulton, and Charles Semple. 2005. "Bounding the number of hybridisation events for a consistent evolutionary history." _Journal of Mathematical Biology_ 51:171-182. https://doi.org/10.1007/s00285-005-0315-9 [↩](#cite-1)
2. <a id="ref-2"></a> Linz, Simone, and Charles Semple. 2019. "Attaching leaves and picking cherries to characterise the hybridisation number for a set of phylogenies." _Advances in Applied Mathematics_ 105:102-129. https://doi.org/10.1016/j.aam.2019.01.004. Preprint: https://arxiv.org/abs/1712.04131 [↩](#cite-2)
3. <a id="ref-3"></a> van Iersel, Leo, Steven Kelk, Nela Lekić, Chris Whidden, and Norbert Zeh. 2016. "Hybridization number on three rooted binary trees is EPT." _SIAM Journal on Discrete Mathematics_ 30:1607-1631. https://doi.org/10.1137/15M1036579. Preprint: https://arxiv.org/abs/1402.2136 [↩](#cite-3)
4. <a id="ref-4"></a> Lin, Qianying, Emma E. Goldberg, Thomas Leitner, Carmen Molina-París, Aaron A. King, and Ethan O. Romero-Severson. 2024. "The number and pattern of viral genomic reassortments are not necessarily identifiable from segment trees." _Molecular Biology and Evolution_ 41:msae078. https://doi.org/10.1093/molbev/msae078 [↩](#cite-4)
5. <a id="ref-5"></a> Müller, Nicola F., Ugnė Stolz, Gytis Dudas, Tanja Stadler, and Timothy G. Vaughan. 2020. "Bayesian inference of reassortment networks reveals fitness benefits of reassortment in human influenza viruses." _Proceedings of the National Academy of Sciences_ 117:17104-17111. https://doi.org/10.1073/pnas.1918304117 [↩](#cite-5)
