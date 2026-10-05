# TreeKnit.jl: overview and feature inventory

The documents in `kb/feat/` describe what TreeKnit.jl does. It is an inventory of the reference implementation and does not describe this port. Each document states the behavior that the code shows, with links to the source. Where the code differs from the TreeKnit.jl documentation, the code is the reference, and [`documented-vs-actual.md`](documented-vs-actual.md) lists the differences.

Surveyed revision: TreeKnit.jl `master` at [`dbbc89a`](https://github.com/PierreBarrat/TreeKnit.jl/commit/dbbc89ac691fed0949a622eedbae103787b89320) (2025-02-18). This is version 0.5.8 (2024-08-09) plus one CLI fix for Julia 1.11 and documentation changes. The Newick reading, writing, and split operations come from TreeTools.jl 0.6.14 [[src](https://github.com/PierreBarrat/TreeTools.jl/tree/2fb33ac3a5891a9cf0d25c76d2bc0270e571a6b8)], the latest version that the TreeKnit.jl compatibility bound (`0.6.13`) allows.

## What TreeKnit does

Segmented viruses, for example human influenza, exchange segments when two strains infect the same host. This exchange is a **reassortment**. Each segment has its own genealogy, so a phylogenetic tree built from one segment can differ from a tree built from another segment. TreeKnit takes one tree per segment, all with the same leaves, and finds where the trees differ because of reassortment.

- **Maximally compatible clade (MCC)**: a set of leaves whose subtrees have the same topology in both trees of a pair, up to polytomy resolution, and that cannot be made larger. Inside an MCC, the branches are shared by both segments, so no reassortment occurred there. The root of each MCC marks one reassortment, except for an MCC that contains the root of a tree
- **Topological heuristic**: TreeKnit does not use sequences. It finds the MCCs from the tree topologies. It removes clades from the trees to make the remaining parts compatible, and it scores each removal with a cost `γ` against the number of topological mismatches that the removal fixes. A simulated annealing search finds the set of clades to remove. Branch lengths break ties between equally good solutions
- **Polytomy resolution**: trees built from short segments have polytomies. Two trees can differ only because one tree is less resolved. TreeKnit adds compatible splits from one tree to the other before and during inference, and uses the inferred MCCs to add more splits after inference
- **More than two trees**: for `K > 2` trees, TreeKnit infers the MCCs of every pair in a fixed order and can resolve the trees between pairs. It does not enforce consistency between the MCCs of different pairs
- **Ancestral reassortment graph (ARG)**: for exactly two trees, TreeKnit joins the trees along the MCCs into one graph with a hybrid node above each reassortment and writes it as extended Newick

The method is published in Barrat-Charlaix, Vaughan, and Neher, "TreeKnit: Inferring ancestral reassortment graphs of influenza viruses", _PLOS Computational Biology_ 18 (8): e1010394 (2022), <https://doi.org/10.1371/journal.pcbi.1010394>, with a preprint at <https://doi.org/10.1101/2021.12.20.473456> [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/README.md?plain=1#L8-L13)]. The paper covers two trees. On simulated genealogies with 100 leaves, it reports TreeKnit about as accurate as the Bayesian method CoalRe and much faster (40 ms per tree pair).

## Data flow of one command-line run

```mermaid
flowchart TD
  IN["<b>Newick files</b><br/><small>2 or more, same leaves</small>"]:::io
  READ["<b>Read and check</b><br/><small>labels from file names</small>"]:::step
  PRE["<b>Pre-resolve</b><br/><small>all trees together</small>"]:::step
  PAIR["<b>Each round, each pair i &lt; j</b><br/><small>order (1,2), (1,3) ... (K-1,K)</small>"]:::step
  OPT["<b>runopt</b><br/><small>naive MCCs, annealing, pruning, iteration</small>"]:::algo
  RES["<b>Resolve pair with MCCs</b><br/><small>strict or liberal; skipped in some rounds</small>"]:::step
  SORT["<b>Final round</b><br/><small>ladderize tree 1, sort polytomies</small>"]:::step
  OUT["<b>Outputs</b><br/><small>MCCs.json, resolved trees, parameters.json, log.txt, auspice JSON</small>"]:::io
  ARG["<b>K = 2 only</b><br/><small>liberal resolve, ARG, arg.nwk, nodes.dat</small>"]:::io
  IN --> READ --> PRE --> PAIR --> OPT --> RES --> PAIR
  RES --> SORT --> OUT
  SORT --> ARG
  classDef io fill:#4a6a8a,color:#ffffff
  classDef step fill:#6b7b5e,color:#ffffff
  classDef algo fill:#7a6a8a,color:#ffffff
```

## Documents

- [`cli.md`](cli.md): the `treeknit` command, its arguments, options, flags, validation, logging, and output directory
- [`pipeline.md`](pipeline.md): `OptArgs`, the two run methods, rounds, the pair order, tree mutation, naive mode, and the parallel mode
- [`mcc-inference.md`](mcc-inference.md): naive MCCs, the split graph, the energy, simulated annealing, the branch-length likelihood, and the iteration in `runopt`
- [`resolution.md`](resolution.md): polytomy resolution with tree topology only, during inference, and with MCCs (strict and liberal)
- [`arg.md`](arg.md): construction of the ancestral reassortment graph for two trees and its extended Newick format
- [`visualization.md`](visualization.md): ladderizing, sorting polytomies for tanglegrams, auspice JSON, and ARG viewers
- [`formats.md`](formats.md): every input and output file format, including the Newick dialect of TreeTools.jl
- [`julia-api.md`](julia-api.md): the Julia library interface, `MCC_set`, and the MCC utility functions
- [`documented-vs-actual.md`](documented-vs-actual.md): places where the code does not do what the documentation or the help text says
- [`history.md`](history.md): features by release, removed features, unmerged branches, and the open upstream pull request

Research reports in `kb/reports/` give the scientific and technical context:

- [`treeknit-paper-vs-code.md`](../reports/treeknit-paper-vs-code.md): each part of the published method compared with the code, including the divergences in the likelihood test and the extensions without published validation
- [`reassortment-methods-literature.md`](../reports/reassortment-methods-literature.md): other reassortment and network methods, the benchmarks that include TreeKnit, and the combinatorial problems behind it
- [`treeknit-ecosystem.md`](../reports/treeknit-ecosystem.md): distribution, downstream users, and the conformance of the output files to auspice and extended Newick readers

## Position among related tools

- **Speed and input**: TreeKnit needs only two or more rooted trees with the same leaves, no alignments, and runs in milliseconds to seconds. Bayesian methods such as CoalRe take hours, and TreeSort needs alignments
- **Accuracy in later benchmarks**: independent studies report that TreeSort is more accurate on 1,000 or more strains, that TreeKnit and CoalRe get the number of reassortments right in 15% to 45% of simulations with 50 tips, and that TreeKnit has low precision on noisy trees (see [`reassortment-methods-literature.md`](../reports/reassortment-methods-literature.md))
- **Users**: a few public pipelines run TreeKnit, mostly from the Neher and Bedford groups. Two of them expect the output names of TreeKnit 0.4 (see [`treeknit-ecosystem.md`](../reports/treeknit-ecosystem.md))

## Outside the scope of TreeKnit

The code has no implementation of the items below. A user must do these steps with other tools.

- **Tree inference**: TreeKnit does not build trees from sequences or alignments
- **Rooting**: TreeKnit uses the input roots as they are. The documentation tells users to root all trees with the same outgroup [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/overview.md?plain=1#L32-L33)]
- **Removal of weak branches**: every internal node is a hard topological constraint. The documentation tells users to collapse branches shorter than `(L/2)^-1` or with low bootstrap support before the run [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/overview.md?plain=1#L29-L30)]
- **Reproducible runs**: no option sets a random seed. The annealing, the choice between tied configurations, and the ARG node labels use the unseeded global random generator of Julia, so two runs on the same input can give different MCCs
- **ARG for more than two trees**: only pairs. An unmerged branch has a prototype (see [`history.md`](history.md))
- **Consistency between pairs**: for `K > 2`, the MCCs of different pairs can contradict each other. The documentation shows such a case and states that TreeKnit cannot fix it [[doc](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/docs/src/multitreeknit.md?plain=1#L96-L123)]
- **Reading results back**: no function reads `MCCs.json` or an ARG file. `read_mccs` reads only the line-based MCC list (see [`formats.md`](formats.md))
