# TreeKnit and MultiTreeKnit: purpose, inputs, outputs, and the multi-tree simulation study

This report explains what TreeKnit computes, which analyses it offers, what its inputs and outputs represent, and why it is used. It then describes MultiTreeKnit, the extension of TreeKnit from two segment trees to all segments of a genome, and the simulation study that evaluated it and chose its default settings. The last part relates both to this port.

Sources:

- **TreeKnit paper** [[1](#ref-1)]
- **TreeKnit.jl**: documentation and code at [`dbbc89a`](https://github.com/PierreBarrat/TreeKnit.jl/commit/dbbc89ac691fed0949a622eedbae103787b89320)
- **MTKSimulations**: the simulation repository at [`30d8793`](https://github.com/anna-parker/MTKSimulations/tree/30d8793e7ffb6eab42532cd30a496918fcd55c61)
- **MTKTools**: the helper package of MTKSimulations at [`b54f9c8`](https://github.com/anna-parker/MTKTools/tree/b54f9c8cb50fb10f0bf3959a43877a413967b945)

Related reports:

- [`treeknit-paper-vs-code.md`](treeknit-paper-vs-code.md): the method in detail and its mapping to the code of TreeKnit.jl
- [`reassortment-methods-literature.md`](reassortment-methods-literature.md): related methods and independent benchmarks
- [`treeknit-ecosystem.md`](treeknit-ecosystem.md): downstream users and file formats

## Summary

- **Purpose**: TreeKnit finds the parts of the genealogy that two genome segments share and the <a id="gloss-use-1"></a>reassortment <sup>[1](#gloss-1)</sup> events that separate them, from the <a id="gloss-use-2"></a>segment trees <sup>[2](#gloss-2)</sup> alone. For two trees it also builds an <a id="gloss-use-3"></a>ARG <sup>[3](#gloss-3)</sup> <a id="cite-1a"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]
- **Central output**: the <a id="gloss-use-4"></a>MCCs <sup>[4](#gloss-4)</sup> of each pair of trees. Each MCC is a set of strains whose history is shared by both segments. Each MCC that does not contain a root marks one reassortment
- **One command**: `treeknit` takes two or more Newick trees. The number of trees and the resolution settings select the analysis. The command has no subcommands
- **Why use it**: it needs only trees, takes about 40 ms per pair of trees with 100 leaves, and in the published simulations is about as accurate as the Bayesian method CoalRe. It takes every internal node of the input trees as a hard constraint, so poorly supported branches cause false reassortments
- **MultiTreeKnit**: the extension to more than two trees, released in TreeKnit.jl 0.5.0 (2023-01-26). It resolves each tree with all other trees and infers MCCs for every pair. It does not build an ARG, because the MCCs of different pairs can contradict each other
- **Simulation study**: on simulated 8-segment influenza genealogies, more segment trees restore more missing branches and identify more shared branches, and pooling shared branches divides the variance of branch length estimates by a factor close to $K$ for $K$ trees. A final round without resolution removes all invalid MCCs, whose two trees no longer have compatible topologies on the output trees. These results set the TreeKnit.jl presets `--better-trees` and `--better-MCCs`
- **This port**: implements the multi-tree pipeline with a new default resolution mode, `matched`, which keeps every MCC valid on the output trees. It does not check whether the MCCs of different pairs agree, and it builds an ARG for two trees only

## What TreeKnit does

### Reassortment and segment trees

When two influenza viruses infect the same cell, an offspring virus can carry segments from both parents. Reassortment "has been found to be the cause of most pandemic influenza strains", and the genome of influenza A has 8 segments, each with its own genealogy <a id="cite-1b"></a>[Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]. Below a reassortment the segment trees agree. Above it, a lineage attaches at different places in the trees of the segments that it took from different parents. TreeKnit uses these topological differences as evidence of reassortment.

### MCCs and the number of reassortments

An MCC (maximally compatible clade) is a largest set of leaves whose subtrees have the same topology in both trees. The MCCs of a pair partition the leaves. No reassortment happened inside an MCC, and one reassortment happened above the root of each MCC, except for an MCC that contains the root of one of the trees. The number of reassortments is therefore the number of MCCs, or the number of MCCs minus one (see [`treeknit-paper-vs-code.md`](treeknit-paper-vs-code.md#mccs-and-the-number-of-reassortments)).

### Search

TreeKnit starts with the clades that are identical in both trees (naive MCCs), reduces each to one effective leaf, and then removes effective leaves (marks them as reassortants) by simulated annealing to minimize the score

$$N_\gamma(\vec\sigma) = \sum_{n=1}^{L} \Delta(n, \vec\sigma)\,\sigma_n + \gamma\,\bigl(L - |\vec\sigma|\bigr)$$

where:

- $L$ -- number of effective leaves
- $\vec\sigma \in \{0,1\}^L$ -- configuration, in which $\sigma_n = 0$ marks effective leaf $n$ as removed
- $\Delta(n, \vec\sigma)$ -- 1 if the parents of leaf $n$ define different clades in the two trees once the removed leaves are ignored, else 0
- $|\vec\sigma|$ -- number of leaves kept
- $\gamma$ -- cost of one removed leaf, that is, of one reassortment, with default 2

The loop repeats until no leaf is removed. With $\gamma = 1$ the score approximates parsimony, and $\gamma \to \infty$ gives the naive MCCs. The paper found $\gamma = 2$ "a robust choice" [[1](#ref-1)].

### Unresolved trees

Trees built from closely related sequences have many <a id="gloss-use-5"></a>polytomies <sup>[5](#gloss-5)</sup>, because branches without mutations cannot be resolved. TreeKnit treats two clades as compatible when one can be resolved into the other, and after inference it adds to each tree the <a id="gloss-use-6"></a>splits <sup>[6](#gloss-6)</sup> of the other tree inside the MCCs. Joint analysis of segments therefore resolves trees as well as finding reassortments, because in shared parts of the genealogy two segments roughly double the number of mutations per branch [[1](#ref-1)].

## Analyses and the questions they answer

All analyses run through one command, `treeknit <tree> <tree> [<tree> ...]`, in TreeKnit.jl and in this port.

- **Two trees**: which strains share the history of the two segments, how many reassortments happened, and where in the trees? Output: the MCCs, both trees resolved with each other, and the ARG
- **Three or more trees** (MultiTreeKnit): the MCCs of every pair, and each tree resolved with all other trees. TreeKnit builds no ARG for more than two trees
- **Resolution settings**: how much to resolve, trading wrong splits against extra reassortments
  - TreeKnit.jl presets `--better-trees` (joint resolution before inference, then each pair without further resolution: few wrong splits, more reassortments) and `--better-MCCs` (pairs resolved one after another, then for more than two trees a final round without resolution: more accurate MCCs, more wrong splits) [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/overview.md?plain=1#L72-L75)]
  - This port: `--resolve matched|strict|liberal|none` and `--pre-resolve`, with the presets kept as former options [[doc](../../README.md#resolving-trees)]
- **Naive mode** (`--naive`): keeps only the clades that are identical in both trees, so it overestimates reassortments
- **Display**: `--auspice-view` writes Auspice JSON for a <a id="gloss-use-7"></a>tanglegram <sup>[7](#gloss-7)</sup> colored by MCC. This port also writes SVG figures with `--plot`
- **Trees with different leaf sets** (this port only): inference on the shared leaves, and placement of the missing leaves with `--impute` (see [`partial-overlap.md`](../feat/partial-overlap.md))

## Inputs

- **Segment trees**: two or more rooted Newick trees, one per segment, with the sampled strains as leaves. Each tree must describe a genomic unit without internal reassortment or recombination. TreeKnit.jl requires the same leaves in all trees [[1](#ref-1)]. This port accepts different leaf sets
- **Consistent rooting**: all trees rooted the same way, for example with the same outgroup
- **Supported branches only**: TreeKnit takes every internal node as a hard constraint. The paper recommends removing internal branches that no mutation supports, and internal nodes with bootstrap support below 75, before running TreeKnit [[1](#ref-1)]
- **Branch lengths** (optional): average number of mutations per site. They only break ties between MCC sets of equal score, together with the segment lengths `--seq-lengths`
- **Parameters**: $\gamma$ (`--gamma`), the resolution settings, and in this port the random seed (`--seed`)

## Outputs

The file names are those of TreeKnit.jl 0.5 and this port. The full list is in the [README](../../README.md#usage) and in [`formats.md`](../feat/formats.md).

- **`MCCs.json`**: for each pair of trees, the list of MCCs, each a list of leaf names. `MCCs.dat` holds the same MCCs in the text format of TreeKnit.jl before 0.5, which `treetime arg` reads
- **`<tree>_resolved.nwk`**: each input tree with polytomies resolved by the other trees, with polytomies sorted for tanglegrams
- **`ARG/arg.nwk`** (two trees only): the ARG in <a id="gloss-use-8"></a>extended Newick <sup>[8](#gloss-8)</sup>. Hybrid nodes `#H<i>` are reassortments, and `[&segments={0,1}]` lists the segments that use a branch. `ARG/nodes.dat` maps ARG nodes to tree nodes. IcyTree displays the file
- **Display and records**: Auspice JSON, SVG figures (this port), `parameters.json`, `log.txt`

## Uses and limits

- **Speed**: on trees of 100 leaves the paper reports 40 ms per run against 40 s for the graph-based reassortment detector GiRaF, and calls TreeKnit "orders of magnitude faster" than CoalRe [[1](#ref-1)]. Runtimes of this port are in [`performance.md`](performance.md)
- **Accuracy**: on simulations of 100 leaves, TreeKnit and CoalRe "perform similarly well". On about 150 A/H3N2 strains sampled in New York from 1999 to 2004, TreeKnit recovered the two reassortments found by manual inspection, the one more found by GiRaF, and three more that manual inspection of the trees supports [[1](#ref-1)]
- **Downstream uses**: better resolved segment trees, joint estimation of branch lengths and divergence times on shared branches in TreeTime, and tanglegrams colored by MCC. Public workflows that run TreeKnit are listed in [`treeknit-ecosystem.md`](treeknit-ecosystem.md#downstream-users)
- **Observable ARG only**: TreeKnit "infers only the observable part of the ARG that is directly connected to the observed taxa" [[1](#ref-1)]
- **Trusted topology**: uncertainty in internal nodes becomes uncertainty in the reassortments. Independent benchmarks report low precision on noisy trees (see [`reassortment-methods-literature.md`](reassortment-methods-literature.md))
- **Pairs only**: inference is pairwise. With more than two trees the MCCs of different pairs can contradict each other, which prevents an ARG of all segments

## MultiTreeKnit

### Origin

The TreeKnit paper ends with the plan "to extend TreeKnit to multiple [segments] and complement it with ARG visualization tools in Nextstrain" [[1](#ref-1)]. MultiTreeKnit is that extension. It was developed in TreeKnit.jl on the branch `MTK_no_consistency` and released in TreeKnit.jl 0.5.0 (2023-01-26) [[src](https://github.com/PierreBarrat/TreeKnit.jl/commit/300571ea51b4a52f1ef8f9e9db014bd7f8cc61f3)]. This report found no publication that describes or validates it. The sources are its documentation and the simulation study below.

### Goals

For $K \ge 3$ segment trees, MultiTreeKnit aims to [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/multitreeknit.md?plain=1#L3-L7)]:

- resolve each tree with all other trees, consistently
- find the MCCs of every pair of trees
- find the reassortment events between the pairs

where $K$ is the number of segment trees in the analysis.

### Consistent resolution

Running two-tree TreeKnit on every pair separately can resolve one tree in contradictory ways. For the trees $((A,B),C)$, $(A,B,C)$, and $(A,(B,C))$, the pair with the first tree resolves the polytomy as $((A,B),C)$ and the pair with the third tree as $(A,(B,C))$ [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/multitreeknit.md?plain=1#L74-L94)]. MultiTreeKnit avoids this with three steps:

- **Pre-resolution**: before inference, add to each tree only the splits that are compatible with all trees
- **Ordered pairs**: process the pairs in input order, $(1,2), (1,3), \ldots, (1,K), (2,3), \ldots$, each pair starting from the trees that earlier pairs resolved. A pair waits only for the earlier pairs that resolved its two trees, so pairs such as $(1,4)$ and $(2,3)$ can run in parallel
- **Strict resolution**: add a split only when the other tree determines the position of every branch in it. The alternative, liberal resolution, also adds ambiguous splits and places the branches of other MCCs arbitrarily

### Conditions for an ARG of all segments

Pairwise MCCs can be combined into one ARG only if they satisfy two conditions.

- **Topological compatibility**: every MCC must still be an MCC of the output trees. Resolution in later pairs can add splits to a tree after an earlier pair inferred its MCCs, so an earlier MCC can lose its shared topology, and in some cases become incompatible even up to resolution [[doc](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/Consistency/README.md?plain=1#L10-L54)]
- **Triplet consistency**: if a reassortment separates segment 1 from segment 2 above some leaves but not segment 1 from segment 3, then it also separates segments 2 and 3 there. For every triplet of trees [[doc](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/Consistency/README.md?plain=1#L75-L104)]:

$$\forall\, m_{12} \in \mathcal{M}_{12},\ \forall\, m_{13} \in \mathcal{M}_{13}:\quad m_{12} \cap m_{13} \neq \emptyset \;\Rightarrow\; \exists\, m_{23} \in \mathcal{M}_{23}:\ m_{12} \cap m_{13} \subseteq m_{23}$$

where:

- $\mathcal{M}_{ij}$ -- the set of MCCs of trees $i$ and $j$
- $m_{ij}$ -- one MCC of trees $i$ and $j$, a set of leaves

The condition holds for every relabeling of the three trees. When the overlap $m_{12} \cap m_{13}$ is smaller than $m_{12}$ or $m_{13}$, it must equal an MCC of trees 2 and 3. When it equals both, it may also lie strictly inside one.

As partitions of the leaves (see [`cross-pair-mcc-consistency.md`](../proposals/cross-pair-mcc-consistency.md)), the condition reads

$$P_{12} \wedge P_{13} \le P_{23}$$

where:

- $P_{ij}$ -- the partition of the leaves into the MCCs of trees $i$ and $j$
- $\wedge$ -- the <a id="gloss-use-9"></a>meet <sup>[9](#gloss-9)</sup> of two partitions
- $\le$ -- every block of the left side lies inside one block of the right side

Each pair is inferred without the MCCs of the other pairs, so the triplet condition can fail even when all trees are resolved consistently. The TreeKnit.jl documentation attributes this to the annealing, which "removes branches at random" [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/multitreeknit.md?plain=1#L103)]. In this port, the failures on a fully resolved simulated case are the same for every seed and every resolution setting, so the separate inference of the pairs causes them, and the randomness does not (see [`cross-pair-mcc-consistency.md`](../proposals/cross-pair-mcc-consistency.md#measured-inconsistency)). The TreeKnit.jl documentation states that MultiTreeKnit "may still return MCCs that are inconsistent with each other, which prevents the construction of an ARG. Therefore, we do not reconstruct an ARG for more than two trees" [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/multitreeknit.md?plain=1#L7)].

## The MTKSimulations study

MTKSimulations evaluates MultiTreeKnit on simulated data with five pipelines: MCC accuracy, polytomy resolution, shared branches, divergence times, and consistency [[doc](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/README.md?plain=1#L49-L76)]. It ran against the TreeKnit.jl branch `MTK_no_consistency` and, for divergence times, against the TreeTime branch `feat/multiTK_arg`. The repository was active from 2022-12-12 to 2023-01-25.

### Setup

- **Genealogies**: ARGTools simulates ARGs of 8 segments under a <a id="gloss-use-10"></a>flu-like coalescent <sup>[10](#gloss-10)</sup> or the <a id="gloss-use-11"></a>Kingman coalescent <sup>[11](#gloss-11)</sup>, with 100 leaves in most pipelines and 100 simulations per parameter point. The reassortment rate, relative to the coalescence rate, ranges from $10^{-4}$ to $1$
- **Poor resolution**: internal branches of the true segment trees are removed at random, with a probability that gives a target <a id="gloss-use-12"></a>resolution rate <sup>[12](#gloss-12)</sup>, typically 0.3 to 0.4 as in influenza trees
- **Inference**: a random subset of $K \le 8$ segment trees goes to MultiTreeKnit. $K = 2$ is two-tree TreeKnit
- **Evaluation**: the output is compared with the true MCCs and the true trees

A branch is removed with probability

$$p(\tau) = e^{-\tau / (cN)}$$

where:

- $\tau$ -- branch length in generations
- $N$ -- population size
- $c$ -- parameter chosen so that the trees reach the target resolution rate

The numbers below are read from the published figures and are approximate.

### MCC accuracy

Measured with the <a id="gloss-use-13"></a>VI distance <sup>[13](#gloss-13)</sup>, the <a id="gloss-use-14"></a>Rand distance <sup>[14](#gloss-14)</sup>, and <a id="gloss-use-15"></a>homogeneity and completeness <sup>[15](#gloss-15)</sup> against the true MCCs [[doc](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/MCCAccuracySimulations/README.md?plain=1#L60-L100)]:

- More trees improve the VI and Rand distances little, with strict resolution a little more than with liberal resolution
- With more trees, completeness decreases (true MCCs are split into several inferred MCCs) and homogeneity increases
- More rounds of inference and the consistency penalty (below) change little

### Polytomy resolution

Measured as the fraction of the removed splits that the output trees contain again, the fraction of added splits that are wrong (both scaled by the number of removed splits), and the improvement of the <a id="gloss-use-16"></a>Robinson-Foulds distance <sup>[16](#gloss-16)</sup> [[doc](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/PolytomySimulations/README.md?plain=1#L20-L60)]. For the flu model at resolution 0.3:

- **More trees restore more splits**: with strict resolution and reassortment rates below $10^{-2}$, about 20% of the removed splits for $K = 2$, 41% for $K = 4$, and 62% for $K = 8$ [[fig](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/PolytomySimulations/Figures/PercentageCorrectResolution_flu_0.3_true.png)]
- **Strict resolution adds few wrong splits**: nearly none below rate $10^{-2}$, and at most about 3% ($K = 2$) to 18% ($K = 8$) near rate 0.3
- **Liberal resolution adds many wrong splits at high rates**: up to about 19% ($K = 2$) to 43% ($K = 8$) at rate 1 [[fig](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/PolytomySimulations/Figures/PercentageCorrectResolution_flu_0.3_false.png)]

### Shared branches

Measured as the fraction of branches of the output trees that are correctly or wrongly labeled as shared by a pair, out of all branches [[doc](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/AccuracySharedBranches/README.md?plain=1#L1-L20)]. For the flu model at resolution 0.3 with strict resolution [[fig](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/AccuracySharedBranches/Figures/Percentage_shared_branches_flu_0.3_true.png)]:

- Below rate $10^{-2}$ almost all branches are shared in the true trees. MultiTreeKnit labels about 71% of all branches correctly as shared for $K = 2$ and 87% for $K = 8$
- Wrongly labeled shared branches stay near 0 below rate $10^{-2}$, reach about 1% at rate 0.1, and about 8% at rate 1
- Liberal resolution raises the wrongly labeled shared branches and leaves the correct ones almost unchanged

### Divergence times

Sequences of 1000 sites evolve along the true segment trees, and TreeTime estimates the branch lengths of one segment tree either alone or, with `treetime arg`, together with the other segments on the branches that the MCCs mark as shared [[doc](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/DivergenceTimeEstimations/README.md?plain=1#L1-L30)]. Without reassortment, a branch shared by $K$ segments of equal length carries $K$ times as many expected mutations, so the variance of its length estimate falls by the factor $K$. For the flu model at resolution 0.35 with strict resolution [[fig](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/DivergenceTimeEstimations/Figures/var_divergence_times_flu_0.35_true.png)]:

- Below rate 0.1 the variance of the normalized error is about $9.7 \times 10^{-4}$ for one tree, $4.9 \times 10^{-4}$ for $K = 2$, $2.5 \times 10^{-4}$ for $K = 4$, and $1.3 \times 10^{-4}$ for $K = 8$, close to the $1/K$ expectation
- Above rate 0.1 the variances of the joint estimates rise, and at rate 1 some of them exceed the single-tree value. Branches wrongly labeled as shared have a larger variance with joint estimation than alone

### Consistency

This pipeline tests which settings satisfy the two conditions for an ARG, for $K = 4$ under the flu model at resolution 0.3 [[doc](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/Consistency/README.md?plain=1#L106-L137)]. The README states that $K = 8$ gives similar results. The pipeline varies four settings:

- strict or liberal resolution
- 1 or 2 rounds
- a final round that infers the MCCs again on the resolved trees without resolving them
- pre-resolution

The script [[src](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/Consistency/Consistency_TopoCompatibility.jl#L82-L151)] measures:

- **Not identical**: for one random pair, the fraction of all leaves that lie in MCCs whose splits differ between the two output trees [[src](https://github.com/anna-parker/MTKTools/blob/b54f9c8cb50fb10f0bf3959a43877a413967b945/src/measures.jl#L163-L184)]
- **Invalid MCCs**: for the same pair, the fraction of all leaves that lie in invalid MCCs, whose splits are incompatible between the two output trees even up to resolution [[src](https://github.com/anna-parker/MTKTools/blob/b54f9c8cb50fb10f0bf3959a43877a413967b945/src/measures.jl#L193-L206)]
- **Inconsistency**: for one random triplet and each of its three orientations, the branches that the meet of two pairs' MCCs marks as shared, and the fraction of them that the third pair does not place inside one MCC, averaged over the orientations [[src](https://github.com/anna-parker/MTKTools/blob/b54f9c8cb50fb10f0bf3959a43877a413967b945/src/measures.jl#L56-L95)]

Results with strict resolution [[fig](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/Consistency/Figures/Topo_Compat_k4_flu_0.3_true_false.png)] [[fig](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/Consistency/Figures/Consistency_k4_flu_0.3_true_false.png)]:

- **Invalid MCCs**: with 1 round and no other option, up to about 15% of the leaves lie in invalid MCCs, with the peak near rate 0.3. Pre-resolution and a second round reduce this share. A final round without resolution reduces it to zero
- **No resolution at all is the least consistent**: 1 round with the final round without resolution and without pre-resolution resolves no tree. It gives 11% to 16% inconsistent branches at every reassortment rate. The README suggests, without confirming, that the annealing assumes resolutions that are then not applied
- **Other settings**: near 0 below rate $10^{-2}$ and up to 5% to 7% at rate 1. Pre-resolution with a final round without resolution gives the fewest, about 1% to 1.5% at rate 1. Liberal resolution gives similar results [[fig](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/Consistency/Figures/Consistency_k4_flu_0.3_false_false.png)]
- **Consistency penalty**: a cost in the annealing for leaves that the other pairs place in a shared region had little effect in every pipeline. Commit `698836f` (2023-01-09) removed it from TreeKnit.jl "as shown to have little effect on results" [[src](https://github.com/PierreBarrat/TreeKnit.jl/commit/698836f24f596276b7f392550e167c5b5dc952ea)]

### Conclusions and the TreeKnit.jl presets

The study recommends 1 round with pre-resolution and a final round without resolution as the best overall setting: no invalid MCCs, the fewest inconsistencies, few wrong splits, and the lowest cost. With 2 rounds and the same options the MCCs were most accurate, slightly more with liberal than with strict resolution [[doc](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/Consistency/README.md?plain=1#L138-L164)]. TreeKnit.jl 0.5 encodes both as presets in `function OptArgs(K::Int; method)` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/objects.jl#L60-L91)]:

- **`:better_trees`** (default for $K > 2$): pre-resolution, then 1 round without resolution
- **`:better_MCCs`** (default for $K = 2$): pre-resolution and 1 round with strict resolution. For $K > 2$ a final round without resolution follows

## Relation to this port

- **Pipeline**: the port runs the multi-tree pipeline in `pub fn run_observed()` [[src](../../packages/treeknit-core/src/pipeline.rs#L189)], with independent pairs in parallel and seeded runs
- **Default resolution**: `matched` resolves all trees so that, for every pair and every MCC, the two trees restricted to the MCC's leaves have the same topology. When splits from different trees conflict inside a shared region, the MCC is replaced by the maximal clades on which its two trees agree [[doc](../../README.md#resolving-trees)]. This satisfies the topological compatibility condition by construction. The TreeKnit.jl presets remain available as former options with their original meaning (see [README section "Deliberate differences from TreeKnit.jl"](../../README.md#deliberate-differences-from-treeknitjl))
- **Triplet consistency**: no mode enforces it, and the port does not report violations. [`N-cross-pair-mcc-consistency-unreported.md`](../issues/N-cross-pair-mcc-consistency-unreported.md) records the open decision. The check it proposes is the partition test that MTKTools uses for its metric. Measured with this check, the MCCs of the port violate the condition on the simulated fixtures in every resolution mode, and on the four-segment H3N2 data in both settings measured (see [`cross-pair-mcc-consistency.md`](../proposals/cross-pair-mcc-consistency.md#measured-inconsistency))
- **ARG**: built for two trees only, as in TreeKnit.jl

## Gaps and open questions

- **No publication**: MultiTreeKnit and its presets rest on the TreeKnit.jl documentation and the MTKSimulations READMEs. No peer-reviewed source evaluates them
- **Sampling of the consistency metrics**: each simulation scores one random pair and one random triplet. Each point of a curve therefore averages one pair or one triplet from each of the 100 simulations at that point
- **Rate scale**: the simulations and the paper both use ARGTools, but this report did not verify that the reassortment rate axis of MTKSimulations equals the scaled rate $r^\star$ of the paper, for which A/H3N2 HA and NA give $r^\star \approx 0.05$ [[1](#ref-1)]
- **ARG for more than two trees**: still open in both implementations. It needs MCCs that satisfy the triplet condition. [`multi-segment-arg.md`](../proposals/multi-segment-arg.md) records the further conditions, the open questions, and the work items

## Glossary

1. <a id="gloss-1"></a> **Reassortment.** Exchange of whole genome segments between two viruses that infect the same cell, so that an offspring carries segments from both parents. [↩](#gloss-use-1)
2. <a id="gloss-2"></a> **Segment tree.** The genealogy of one genome segment of a set of sampled viruses. Segments without internal recombination each have one tree. [↩](#gloss-use-2)
3. <a id="gloss-3"></a> **ARG (ancestral reassortment graph).** A graph that holds the genealogies of all segments, with shared history drawn once and one node with two parents for each reassortment. [↩](#gloss-use-3)
4. <a id="gloss-4"></a> **MCC (maximally compatible clade).** A largest set of leaves whose subtrees have the same topology in two segment trees. An MCC is a region of the genealogy without reassortment ([Barrat-Charlaix et al. 2022](https://doi.org/10.1371/journal.pcbi.1010394) [[1](#ref-1)]). [↩](#gloss-use-4)
5. <a id="gloss-5"></a> **Polytomy.** An internal node with more than two children. In trees inferred from sequences, polytomies appear where no mutation resolves the branching order, or where weakly supported branches were collapsed. [↩](#gloss-use-5)
6. <a id="gloss-6"></a> **Split.** The bipartition of the leaves that removing one branch of a tree creates. A tree is determined by its set of splits. [↩](#gloss-use-6)
7. <a id="gloss-7"></a> **Tanglegram.** A drawing of two rooted trees facing each other, with lines that join the leaves of the same name. [↩](#gloss-use-7)
8. <a id="gloss-8"></a> **Extended Newick.** A Newick string in which each node with more than one parent appears once per parent, marked with `#` and an index <a id="cite-2"></a>[Cardona et al. 2008](https://doi.org/10.1186/1471-2105-9-532) [[2](#ref-2)]. [↩](#gloss-use-8)
9. <a id="gloss-9"></a> **Meet of partitions.** The partition in which two elements share a block exactly when they share a block in both given partitions. [↩](#gloss-use-9)
10. <a id="gloss-10"></a> **Flu-like coalescent.** The ARGTools model of influenza genealogies. Its total coalescence rate grows about as $n^{1.2}$ with the number of lineages $n$, against $n^2$ in the Kingman coalescent, so deep parts of its genealogies have fewer reassortments per coalescence than under the Kingman coalescent at the same reassortment rate [[doc](https://github.com/anna-parker/MTKSimulations/blob/30d8793e7ffb6eab42532cd30a496918fcd55c61/AccuracySharedBranches/README.md?plain=1#L22-L27)]. [↩](#gloss-use-10)
11. <a id="gloss-11"></a> **Kingman coalescent.** A model of the genealogy of a sample in which every pair of lineages merges at the same rate, so the total merge rate grows quadratically with the number of lineages. [↩](#gloss-use-11)
12. <a id="gloss-12"></a> **Resolution rate.** In MTKSimulations, the ratio of internal branches to leaves of a tree. A fully resolved binary tree has a ratio near 1, and influenza trees about 0.3 to 0.4. [↩](#gloss-use-12)
13. <a id="gloss-13"></a> **VI distance (variation of information).** An information-theoretic distance between two partitions, $H(X \mid Y) + H(Y \mid X)$ for the conditional entropies $H$ of the block labels $X$ and $Y$, scaled in MTKSimulations by the logarithm of the number of leaves. [↩](#gloss-use-13)
14. <a id="gloss-14"></a> **Rand distance.** One minus the Rand index, the fraction of leaf pairs that two partitions both place together or both place apart. [↩](#gloss-use-14)
15. <a id="gloss-15"></a> **Homogeneity and completeness.** The two parts of the V-measure. An inferred partition is homogeneous if each inferred MCC contains leaves of one true MCC only, and complete if each true MCC lies in one inferred MCC. [↩](#gloss-use-15)
16. <a id="gloss-16"></a> **Robinson-Foulds distance.** The number of splits that occur in exactly one of two trees on the same leaves. [↩](#gloss-use-16)

## References

1. <a id="ref-1"></a> Barrat-Charlaix, Pierre, Timothy G. Vaughan, and Richard A. Neher. 2022. "TreeKnit: Inferring ancestral reassortment graphs of influenza viruses." _PLOS Computational Biology_ 18:e1010394. https://doi.org/10.1371/journal.pcbi.1010394. Preprint: https://doi.org/10.1101/2021.12.20.473456 [↩¹](#cite-1a) [↩²](#cite-1b)
2. <a id="ref-2"></a> Cardona, Gabriel, Francesc Rosselló, and Gabriel Valiente. 2008. "Extended Newick: It is time for a standard representation of phylogenetic networks." _BMC Bioinformatics_ 9:532. https://doi.org/10.1186/1471-2105-9-532 [↩](#cite-2)
