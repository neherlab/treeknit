# phytools feature survey: tree rendering, semantics, and edge cases

This report describes the tree plotting and visualization functions of phytools, an R package for phylogenetic comparative methods, as an idea inventory for tree, tanglegram, network, and ARG display. It records what the functions draw, how they compute layouts, which scientific conventions they assume, which inputs break them, and which problems the maintainer left open. Most words go to `cophylo`, the phytools tanglegram, and to its node rotation algorithm. The report does not compare phytools with `packages/web`.

phytools is licensed GPL (>= 2). Its behavior and design may be studied, but copying its code needs approval (see the project rules).

- **Source**: [liamrevell/phytools](https://github.com/liamrevell/phytools) at commit `5c18553` (2026-09-15), version 2.6-9
  - **History**: 1447 commits since the repository import of 2015-08-18. The source comments date some functions back to 2011
  - **Links**: source links point to this commit
  - **CRAN**: CRAN has the older version 2.5-2
- **Other sources**:
  - **Paper**: [Revell 2024, phytools 2.0](https://doi.org/10.7717/peerj.16505), section "Co-phylogenetic plotting" and Fig. 19
  - **Book**: Revell and Harmon 2022, _Phylogenetic Comparative Methods in R_ (Princeton University Press, ISBN 978-0691219035). Only the table of contents was available: chapter 13 "Plotting phylogenies and comparative data" with section 13.4 "Algorithms for drawing trees" ([publisher page](https://press.princeton.edu/books/paperback/9780691219035/phylogenetic-comparative-methods-in-r))
  - **Blog**: posts on [rotation optimization](https://blog.phytools.org/2016/08/optimizing-node-rotations-with-cophylo.html), [polytomies](https://blog.phytools.org/2016/08/co-phylogenetic-plotting-for-trees-with.html), [`cotangleplot`](https://blog.phytools.org/2021/11/new-co-phylogenetic-method-for-phytools.html), [slanted cladograms in `cophylo`](https://blog.phytools.org/2021/12/slanted-cladogram-tree-style-for-co.html), [duplicate tip labels](http://blog.phytools.org/2019/06/update-to-plotcophylo-to-permit.html), and [labels after rotation](https://blog.phytools.org/2018/06/preserving-node-edge-labels-after.html)
  - **Tracker**: the titles of all 158 issues and 16 pull requests, and the full text of the 30 issues about plotting
  - **Manual pages**: `man/cophylo.Rd`, `man/minRotate.Rd`, `man/untangle.Rd`
- **Evidence labels**:
  - **Observed**: seen in a trial run of version 2.6-9, built from the surveyed commit, inside a `rocker/r-ver` container with ape 5.8.1. Output went to SVG through the Cairo `svg()` device
  - **Derived**: read from the code without running it
  - **Conflicts**: where the documentation and the code disagree, the code wins and the difference is noted
- **Shape of the code**: R functions on the base `graphics` system. There is no scene graph and no interactivity beyond `locator()` clicks
  - **Drawing**: each plot function computes coordinates and calls `lines`, `segments`, `polygon`, and `text` directly
  - **Shared state**: after drawing, each function stores the node coordinates in `last_plot.phylo` in the ape environment `.PlotPhyloEnv`, so that ape helpers (`nodelabels`, `tiplabels`, `edgelabels`) can annotate the plot
  - **Key files**:
    - [`R/cophylo.R`](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R): `cophylo`, `plot.cophylo`, `tipRotate`, link drawing
    - [`R/plotSimmap.R`](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R): `plotTree`, `plotSimmap`, and the phylogram, fan, arc, and cladogram layouts
    - [`R/utilities.R`](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R): `untangle`, `allRotations`, `rotate.multi`, `nodeHeights`, clade labels, and many helpers
    - [`R/contMap.R`](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/contMap.R), [`R/densityMap.R`](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/densityMap.R), [`R/densityTree.R`](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/densityTree.R), [`R/phylo.to.map.R`](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/phylo.to.map.R), [`R/cotangleplot.R`](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cotangleplot.R)

## Summary

### Two trees and tanglegrams

- **`cophylo` is a two-step design**: `cophylo()` computes node rotations and returns an object, and `plot()` draws it
  - The rotated trees can therefore be reused, inspected, or drawn by other code
- **Rotation minimizes squared vertical displacement**: the objective is the sum of squared differences between the positions of linked tips. It does not count link crossings
  - **Search**: a greedy search rotates one node at a time in preorder and keeps a rotation only when the objective improves
  - **Alternation**: the search alternates between the two trees until neither improves
- **The search finds local optima and depends on argument order** (observed):
  - In 6 of 11 random 8-tip tree pairs that differ by one leaf move, the greedy result had 1 to 4 crossings where a brute-force search found 0
  - On two 100-tip TreeKnit simulation trees, `cophylo(a, b)` left 414 crossings, and `cophylo(b, a)` left 748
- **Polytomies need `rotate.multi=TRUE`**: this option tries all k! child orders at each polytomy
  - It is exact but factorial. A 33-child polytomy fails with a memory error ([#23](https://github.com/liamrevell/phytools/issues/23), [#150](https://github.com/liamrevell/phytools/issues/150))
- **Links**: straight (default) or sigmoid, dashed by default
  - Colour, width, and line type can be set per association row
  - One tip can link to many tips, and duplicate tip labels are allowed
- **Non-identical tip sets**: without an association table, only the shared labels are linked. Unshared tips are drawn without a link
- **Each tree has its own scale**: each tree is scaled to fill its half of the plot, so the two time axes differ. Optional scale bars are drawn per tree
- **`cotangleplot`** is a second tanglegram style: one shared column of tip labels in the middle
  - Tip positions are set to the mean of the two rotated orders, so the branches of both trees bend to reach the labels

### Risks for TreeKnit users

- **TreeKnit resolved trees break the slanted cladogram** (observed): zero-length branches make `plot.cophylo(type="cladogram")` fail with "replacement has length zero"
  - The cause is node placement, which divides by branch length
- **No network support**: phytools reads and draws trees only
  - The phytools Newick reader hangs on input that ends with a root branch length, which TreeKnit ARG files have
- **Large trees**: `cophylo` takes 12 s for 500 tips and 44 s for 1000 tips (observed)
  - Labels are drawn at full size with no thinning, so a 522-tip tanglegram at the default font size is unreadable (observed)

### Other displays

- **Painted branches (`plotSimmap`)**: every branch is a sequence of segments with a state each, so regimes can change along a branch
  - `plotTree`, `contMap`, and `densityMap` all reduce to this one drawing routine
- **Continuous colour (`contMap`, `densityMap`)**: branches are cut at 100 global time steps and each piece gets one of 1001 colours
  - The legend is a colour bar of 1000 line segments
- **Many trees**:
  - `densityTree` overlays trees with transparency (a "cloudogram")
  - `ltt` and `ltt95` plot lineage counts of many trees
  - `plotTree` of a `multiPhylo` object pages through the trees
- **Layouts**: rectangular phylogram, slanted cladogram, fan, and arc (a fan with a hole in the middle)
  - Each layout is available in four directions where it applies
  - Six rules place internal nodes

## Layouts

### Coordinate system (`plotSimmap` and `plotTree`)

- **One routine for all trees**: `plotTree` copies each branch length into a single-state map named `"1"` and calls `plotSimmap` [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L740-L747)]
- **Missing branch lengths**: when the tree has no lengths at all, `plotTree` computes Grafen lengths with ape `compute.brlen` (power 1), so all tips align at the right [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L740)]
  - Observed: tip x = 1 for all five tips of `((A,B),(C,(D,E)));`
- **Partly missing lengths fail**: `((A:1,B):1,C:2);` gives a NaN node height and `plotTree` stops with "need finite 'xlim' values" (observed)
- **Horizontal position**: the distance from the root, computed by `nodeHeights` as a cumulative sum in preorder [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R#L1717-L1732)]
  - For `direction="leftwards"` the heights are mirrored, `H = max(H) - H` [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L362)]
- **Vertical position of tips**: 1 to n in cladewise order of the edge matrix, or a user vector `tips` (named by tip label or ordered by tip number) [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L306-L309)]
  - Custom `tips` positions are the basis of `cotangleplot`, `densityTree`, `expand.clade`, and the tree overlay of `ltt`
- **Label space fit**: the plot finds a scale factor `a` with `optimize` so that `a * 1.04 * max(H) + sw` equals the plot width in inches [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L353-L360)]
  - `sw` is the widest label plus 1.37 widths of "W" (an "empirically determined" fudge factor)
  - The x range is then extended by `sw / a`
- **Child order**: phytools never sorts children for display. Order comes from the edge matrix [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R#L2271-L2292)]
  - `untangle(tree, "read.tree")` normalizes it by writing the tree to Newick and reading it back, which makes the tip numbers follow the drawing order

### Internal node placement (`nodes` argument)

Six rules place a parent vertically [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L312-L345)]. Observed values for the root of `((A:1,B:1,C:1,D:1):1,(E:0.5,F:0):1.5,G:2);` with tips at rows 1 to 7:

- **`"intermediate"`** (default): midpoint of the outermost children, 4.75
- **`"centered"`**: midpoint of the range of all descendant tips, 4.0
  - This differs from the default when a subtree is unbalanced
- **`"weighted"`**: weighted mean of the two outermost children with weights `1/length`, so the parent sits closer to the child with the shorter branch
  - A zero-length branch gives `1/0` and the plot fails with "replacement has length zero" (observed)
- **`"inner"`**: the descendant tip nearest the median row of the whole tree, 4.0
  - The parent therefore sits on one tip row and branches point outward from the middle
  - `densityTree` uses this rule by default [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/densityTree.R#L58)]
- **`"left"`** and **`"right"`**: the lowest or the highest descendant tip row (1 and 7)
  - The tree becomes a "comb" with one straight lineage through each parent

### Layout types

#### Rectangular and slanted

- **Phylogram** (default): horizontal branches plus one vertical connector per parent [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L366-L384)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L758-L795)]
  - The connector takes the colour of the first state on the first child branch
  - With `split.vertical=TRUE`, each part of the connector takes the colour of the child below it
- **Directions**: rightwards, leftwards, upwards, and downwards. The vertical directions rotate the tip labels by 90 or 270 degrees [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L141-L284)]
- **Slanted cladogram** (`type="cladogram"`): each branch is a straight line from parent to child, and the horizontal axis still uses branch lengths [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L604-L623)]
  - A painted map is drawn as pieces along the slope
  - A branch with infinite slope (zero length) becomes a vertical line
- **Outline**: `outline=TRUE` first draws the tree in the foreground colour at `lwd + 2`, then the coloured tree on top, which gives each branch a dark border [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L53-L65)]

#### Circular

- **Fan** (`type="fan"`): the tip rows map to angles `Y / maxY * 2 * pi * part`, and the height maps to the radius [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L425-L501)]
  - **Connectors**: parents use the intermediate rule only. Parent connectors are arcs drawn with `draw.arc`. When the root has two children, its two branches are drawn as one continuous line through the centre
  - **Partial fan**: `part` below 1 uses only a fraction of the circle. The plot limits switch to a quarter or a half plane at `part <= 0.25` and `part <= 0.5`
  - **Labels**: each label is rotated to its tip angle. Labels between 90 and 270 degrees are turned by 180 degrees and right-aligned so that they read left to right. The offset is a number of space characters ([#38](https://github.com/liamrevell/phytools/issues/38)) [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L503-L512)]
- **Arc** (`type="arc"`, 2023): a fan with `part=0.5` and an invisible root branch of length `arc_height * max(H)` (default `arc_height=2`) [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L110-L137)]
  - The result is a half circle with an empty centre, so dense trees have room near the root

#### Curved branches

- **Sigmoid phylogram** (`sigmoidPhylogram`): each branch is a generalized logistic curve `A + (K - A) / (C + exp(-B (t - M)))^(1/v)` from the parent row to the child row [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/roundPhylogram.R#L69-L187)]
  - **Parameters**: `B = b * Ntip / h`, with defaults `b=5`, `m1=0.01`, `m2=0.5`, `v=1`
  - **Drawing**: the curve is drawn as 199 segments
- **Round phylogram** (`roundPhylogram`) and **spline phylogram** (`splinePhylogram`): curved branch styles [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/roundPhylogram.R#L4-L33)]

#### Tip spacing and paging

- **Non-uniform tip spacing** (`expand.clade`): tips inside the selected clades keep a spacing of 1, and all other tips get `1/factor` (default 5) [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R#L462-L481)]
  - This is a static focus-and-context view
- **Split plots**:
  - `plotTree.splits` draws a tall tree across several pages or a PDF by setting `ylim` windows [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/splitplotTree.R#L4-L20)]
  - `splitplotTree` cuts the tree into two side-by-side columns [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/splitplotTree.R#L22-L60)]

## Two trees and tanglegrams (`cophylo`)

### The `cophylo` object

- **Inputs**: two trees, an optional two-column association table `assoc`, and `rotate=TRUE` [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L4-L111)]
- **Normalization first**: both trees go through `untangle(tree, "read.tree")`, so tip numbers follow the cladewise drawing order [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L12-L13)]
  - Link positions are later read from the tip index
- **Default association**: without `assoc`, the shared tip labels are linked one to one [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L15-L22)]
  - With no shared label, the function prints "No associations provided or found." and turns rotation off
- **Association filter**: rows whose label is missing from a tree are removed with a message [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L24-L33)]
- **Value**: a list with `trees` (the two rotated trees as `multiPhylo`) and `assoc` [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L408-L420)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L581-L602)]
  - Each tree carries the final objective in the attribute `minRotate`
  - `print`, `summary`, `Ntip`, `Nnode`, and `Nedge` methods exist

### Rotation algorithm

#### Objective and greedy search

- **Target positions**: for tree 1, each linked tip gets as target the index of its partner in tree 2, scaled by `Ntip(tr1) / Ntip(tr2)` [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L84-L85)]
  - The scaling lets trees with different tip counts compare on one scale
- **Objective**: `sum(fn(x - position))` over all association rows, with `fn = function(x) x^2` by default [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L424-L426)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L493-L494)]
  - The user can pass another `fn`, for example `abs`
  - The objective counts displacement, not crossings, so it prefers many small displacements over one large one
- **Greedy node sweep** (`methods="pre"`, the default): for each internal node in preorder, rotate it (swap its two children), normalize with `untangle`, and keep the new tree only when the objective strictly decreases [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L489-L508)]
  - **Postorder**: `methods="post"` sweeps from the last node to the first, and `c("pre","post")` does both [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L509-L528)]
  - **Node indices stay valid**: `untangle` renumbers nodes in preorder after each accepted rotation. A rotation only permutes nodes inside the rotated subtree, which all have higher indices, so each node is still visited once (derived)
- **Alternation**: tree 1 is optimized against tree 2, then tree 2 against the new tree 1. The loop repeats while either objective still decreases [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L84-L97)]
- **Ties are random**: when several candidates have the same objective, one is chosen with `sample`, so the result depends on the random seed [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L472-L473)]

#### Polytomies

- **`rotate.multi=TRUE`**: for a node with more than two children, `rotate.multi` builds every permutation of the children with `combinat::permn` and keeps the best [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R#L946-L961)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L434-L436)]
  - **Binary trees**: the option switches off automatically
  - **Cost**: k! trees per polytomy, each written to Newick and read back. Observed on the TreeKnit fixture `sim_k2_n50_r0.02_poly` (50 tips, largest polytomies with 6 and 7 children): 0.12 s and 112 crossings without the option, 21.6 s and 39 crossings with it
  - **Without the option**: ape `rotate` on a polytomy only swaps two children, so the order inside a polytomy is barely optimized
- **`allRotations` and polytomies**: a non-binary tree is first resolved with `multi2di`, rotated, and collapsed again with `di2multi` [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R#L911-L942)]
  - `di2multi` collapses every branch shorter than its tolerance, so real zero-length branches of the input are lost too (derived)
  - The binary resolution also reaches only some of the child orders of a polytomy (4 of 6 for three children)

#### Exhaustive search

- **`methods="exhaustive"`**: three variants exist, and none of these values is documented in the manual [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L38-L82)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L458-L479)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L314-L317)]
  - **`tipRotate`**: evaluates all 2^m rotations of one tree from `allRotations`, refused above `max.exhaustive=20` tips
  - **`cophylo`**: evaluates all pairs of rotations of both trees and returns all optimal pairs as a `multiCophylo` object, which `plot` pages through with `par(ask=TRUE)`
  - **`methods="all"`**: returns every pair

#### Animation and one-sided rotation

- **Animation**: `anim.cophylo=TRUE` redraws the tanglegram after each step and marks the current node with a red dot, with a pause of `sleep=0.1` s [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L437-L451)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L480-L488)]
  - `only.accepted=FALSE` also shows rejected rotations
- **Rotating one tree only**: call `tipRotate` on one tree with the tip order of the other, then `cophylo(..., rotate=FALSE)` ([blog, 2022](https://blog.phytools.org/2022/06/co-phylogeny-plots-rotating-only-left.html))

### Rotation results

- **Identical topologies**: a 30-tip tree against a copy with 15 random rotations went from 194 crossings to 0 (observed)
  - The blog reports objective 0 for 80 and 1000 tips ([blog, 2016](https://blog.phytools.org/2016/08/optimizing-node-rotations-with-cophylo.html))
  - Objective 0 for 200, 500, and 1000 tips was observed too
- **One leaf moved**: in a 30-tip pair, the moved leaf caused all 17 remaining crossings (observed)
  - In 11 random 8-tip pairs with one leaf move, a brute-force search over all 2^7 * 2^7 rotation pairs found 0 crossings for every pair
  - The greedy search left 1, 1, 1, 1, 2, and 4 crossings in 6 of these pairs (observed)
- **Minimum squared displacement agrees with minimum crossings** in all 11 small pairs (observed). The two objectives can still disagree in principle
- **TreeKnit simulation trees** (`sim_k2_n100_r0.02`, 100 tips): 2249 crossings before and 414 after rotation, in 1.1 s (observed)
  - The pre, post, and pre+post sweeps, and `fn=abs`, all ended at 414
  - A local search that tries every single rotation of either tree against the crossing count did not improve 414 either
  - Swapping the argument order gave 748 crossings
- **The maintainer's own limit**: two different topologies with the same cladewise tip order cannot be untangled to 0 (80 tips: objective 41422 to 12256), and the search has no optimality proof ([blog, 2016](https://blog.phytools.org/2016/08/optimizing-node-rotations-with-cophylo.html))

### Drawing (`plot.cophylo`)

#### Geometry and scaling

- **Plot region**: x from -0.5 to 0.5 and y from 0 to 1. Each tree uses a width of `part=0.4`, so the link zone is 0.2 wide [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L322-L353)]
  - `part` can be set, but the width of the link zone has no separate option ([#173](https://github.com/liamrevell/phytools/issues/173), open)
- **Tree types**: `type="phylogram"` (default) or `"cladogram"`, either one value or one per tree (2021 and 2023, [#93](https://github.com/liamrevell/phytools/issues/93)) [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L324-L326)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L372-L378)]
- **Own drawing code**: `plot.cophylo` uses its own functions `phylogram` and `cladogram`, not `plotTree`, so most `plotTree` options are missing [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L116-L276)]
- **Independent scaling**: each tree is scaled so that its height plus `tip.len` plus its widest label fits into `part` [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L137-L143)]
  - The two trees therefore use different time scales, and equal branch lengths look different on the two sides
- **Tip rows**: tips are spaced evenly from 0 to 1 in each tree, `(i - 1) / (n - 1)` [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L145-L155)]
  - Trees with different tip counts therefore get different row spacing
- **Parent placement**: midpoint of the outer children in the phylogram [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L238-L245)]
  - In the slanted cladogram, the parent uses the `1/length` weighting of the `"weighted"` rule, which fails on zero-length branches (see "Defects")

#### Labels

- **Aligned labels**: labels face the link zone [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L175-L189)]
  - A dotted leader of `tip.len=0.1` (relative to the tree width) runs from each tip to its label
  - A dot (`pts=TRUE`) marks each tip
- **Label boxes**: each label gets a rectangle in the background colour (`frame=TRUE`), so lines behind a label do not cross the text [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L280-L287)]
  - `frame=FALSE` turns the boxes off ([`fae8255`](https://github.com/liamrevell/phytools/commit/fae8255), 2024)
- **Underscores become spaces** in labels [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L132)]
- **Font options**: `fsize` and `ftype` take one value or one per tree. A third `fsize` value sets the scale bar font [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L357-L369)]

#### Colour, scale bars, and annotation

- **Edge colours**: `edge.col=list(left=..., right=...)` in the edge order of the rotated trees [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L343-L346)]
  - The user must map colours to the rotated edge order ([blog, 2017](http://blog.phytools.org/2017/04/coloring-edges-of-plotted-trees-in.html), [#53](https://github.com/liamrevell/phytools/issues/53))
- **Scale bars**: `scale.bar=c(left, right)` draws a bar under each tree at its outer edge [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L389-L406)]
  - The bars cannot be moved ([#76](https://github.com/liamrevell/phytools/issues/76), open)
  - A wrong bar length that depended on `fsize` was fixed in 2023 ([#127](https://github.com/liamrevell/phytools/issues/127))
- **Annotation after drawing**: `nodelabels.cophylo`, `edgelabels.cophylo`, and `tiplabels.cophylo` take `which="left"` or `"right"` and forward to the ape label functions [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L545-L564)]
  - They support pies and symbols, for example host colour pies on the tips in Fig. 19 of the paper

### Links

- **Endpoints**: links start at the outer end of the labels on both sides, so they occupy only the middle zone [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L379-L380)]
- **Straight links** (default `link.type="straight"`) and **sigmoid links** (`"curved"`): the curve is a logistic function `plogis(x, location = midpoint, scale = 0.01)` in plot units [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L566-L576)]
  - **Shape**: on the default zone 0.2 wide, 90% of the rise lies within ±0.03 of the middle, so the curve runs flat near both ends and turns steeply in the centre (derived from the logistic quantile 2.94 times the scale)
  - **Origin**: the function comes from a Stack Overflow answer
- **Style per row**: `link.col`, `link.lwd`, and `link.lty` (default `"dashed"`) take one value or one per association row [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L293-L310)]
  - Colouring by host, as in the paper, is done by indexing a palette with the first column of `assoc`
- **Many-to-many**: one label may appear in many rows [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L299-L308)]
  - Since 2019, a label may also occur more than once in a tree, and every occurrence gets a link ([blog, 2019](http://blog.phytools.org/2019/06/update-to-plotcophylo-to-permit.html))
  - Observed: `A-A` and `A-C` drew two links from A
- **Unlinked tips**: a tip without an association row is drawn with no link and no other mark (observed)

### `cotangleplot` (2021)

- **Three panels**: the widths are `layout=c(0.45, 0.1, 0.45)` [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cotangleplot.R#L11-L12)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cotangleplot.R#L68-L71)]
  - Tree 1 faces right in the left panel
  - The tip labels sit once in the middle panel in italics
  - Tree 2 faces left in the right panel
- **Shared tip order**: with `tangle="both"`, both trees are rotated by `cophylo`, and each tip gets the rank of the mean of its two positions [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cotangleplot.R#L25-L35)]
  - With `"tree1"` or `"tree2"`, the order of the other tree is used, so only one tree is drawn tangled
- **Branches cross instead of links**: each tree is drawn with `tips` set to the shared order, so its own branches cross each other where the topologies disagree [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cotangleplot.R#L37-L67)]
  - Dotted leaders run from each tip to the middle
- **Cladogram mode**: each tip gets a path to the root, drawn first in the background colour at `lwd + 2` and then in colour, so crossing paths stay separable [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cotangleplot.R#L52-L67)]
- **Scope**: the manual restricts it to trees with matching tip labels
  - The blog motivates it with comparing trees "estimated using different methodologies" for the same taxa ([blog, 2021](https://blog.phytools.org/2021/11/new-co-phylogenetic-method-for-phytools.html))

### Other two-tree displays

- **`compare.chronograms`**: two trees with the same topology drawn on top of each other in transparent blue and red, with an arrow from each node of tree 1 to the same node in tree 2 [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/compare.chronograms.R#L4-L37)]
  - The arrow is red when the node is older in tree 1
  - Node matching assumes the same node numbers in both trees, without a check
- **`phylo.to.map`**: a tree faces a geographic map, with dashed links from tips to coordinates [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/phylo.to.map.R#L25-L29)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/phylo.to.map.R#L314-L331)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/phylo.to.map.R#L215-L222)]
  - **Many points per tip**: a tip may have several coordinate rows, and each gets a link
  - **Rotation**: the tree is rotated by `minRotate` to match the mean latitude (tree above the map) or longitude (tree left of the map)
  - **`minRotate`**: a single preorder pass with the objective `sum(abs(rank - position))`
- **`linklabels`**: labels for a subset of tips are spread evenly over the height of the tree and joined to their tips by bent, curved, or straight links [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R#L678-L722)]
  - This avoids overlap when the labelled tips are close together

## Many trees

- **`densityTree`** (cloudogram): all trees are drawn on top of each other in one colour with `alpha = max(1/N, 0.01)` [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/densityTree.R#L29-L121)]
  - **Shared tip order**: the order of the majority-rule consensus (`compute.consensus=TRUE`), applied to every tree through `tips`. Trees with other topologies then show crossing branches (observed with two different 8-tip trees) [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/densityTree.R#L49-L52)]
  - **Depth**: `fix.depth=TRUE` rescales every tree to the mean height. A tree without lengths switches all trees to Grafen lengths
  - **Gradient mode** (`use.gradient=TRUE`): trees are ordered on one axis by classical multidimensional scaling of their Robinson-Foulds distances, coloured along a rainbow, and shifted vertically by up to a quarter of a row, so similar trees sit together [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/densityTree.R#L95-L120)]
- **Colour by clade in `densityTree`** is not possible ([#171](https://github.com/liamrevell/phytools/issues/171), open)
- **`plotTree` and `plotSimmap` of many trees**: one plot per tree with `par(ask=TRUE)` between them [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L9-L14)]
- **Lineages through time**: `ltt` of a `multiPhylo` object returns one curve per tree, and `plot` draws them on shared axes [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/ltt.R#L253-L262)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/ltt.R#L469-L491)]
  - **Tree under the curve**: `show.tree=TRUE` draws the tree in transparent blue behind the LTT curve, with tips spread over the lineage axis (log-spaced on a log axis) [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/ltt.R#L443-L463)]
  - **`ltt95`**: a median or mean LTT with a (1 - alpha) band, either across lineage counts at 101 time points (`method="lineages"`) or across times for each lineage count (`method="times"`). The trees are aligned at their youngest tip [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/ltt95.R#L4-L59)]
- **`densityMap`**: the posterior probability of state 1 along the branches over many stochastic maps of one topology (see "Colour and legends")

## Branches, nodes, and painted regimes

### Painted regimes

- **Painted maps**: a `simmap` tree stores for each branch a named vector of segment durations, from parent to child [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L23-L31)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L371-L384)]
  - `plotSimmap` draws one line per segment in the colour of its state
  - Without `colors`, states get `palette()` colours in sorted order, and the legend is printed to the console
- **Painting by hand**: `paintSubTree` and `paintBranches` assign states to clades and branches [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/paintSubTree.R#L4-L79)]
  - `paintSubTree(tree, node, state, stem)` paints a clade, and `stem` (0 to 1) also paints that fraction of the branch above it
  - `paintBranches(tree, edge, state)` paints whole branches
  - Unpainted branches get `anc.state="1"`
- **Change marks**: `markChanges` draws a short perpendicular tick at each state change on a painted tree [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R#L1181-L1208)]

### Clade, node, and tip decorations

- **Clade highlight**: `cladebox` draws a transparent box behind a clade [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cladebox.R#L3-L130)]
  - In fan and arc layouts the box is an annular sector between the clade's stem and its tips
- **Node age bars**: `plotTree.errorbars` draws each node interval as a transparent line of width 11 (`bar.width`), with grid lines and an age axis [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotTree.errorbars.R#L4-L61)]
  - The tree faces left by default, so age increases to the left
- **Bars and boxes at the tips**: `plotTree.wBars` draws one bar per tip beyond the labels, in rectangular or fan layout [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotTree.wBars.R#L129-L259)]
  - A non-ultrametric tree gets dotted extensions to the youngest tip first

### Pruning and collapse

- **Pruning view**: `fancyTree(type="droptip")` draws two panels [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/fancyTree.R#L239-L273)]
  - The top panel shows the full tree with the branches to be removed red and dashed
  - The bottom panel shows the pruned tree on the same time axis, with a root branch that keeps the original root age
- **Interactive collapse** (`collapseTree`): a fan tree in which a click on a node collapses its clade into a tip labelled with the node label, and a click on such a tip expands it again [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/collapseTree.R#L5-L170)]
  - **Animation**: the change is animated over 4 to 10 frames that interpolate the tip angles
  - **End**: a right-click ends the session and returns the pruned tree
  - **Model**: the design follows the shark and ray tree viewer of sharksrays.org

## Labels and annotation

### Tip labels

- **Font types**: `ftype` is one of `"off"`, `"reg"`, `"b"`, `"i"`, `"bi"` [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L20-L33)]
  - `ftype="off"` is the only way to hide labels, because ape's `show.tip.label` is ignored ([#73](https://github.com/liamrevell/phytools/issues/73))
  - Underscores become spaces unless `underscore=TRUE` (added for [#119](https://github.com/liamrevell/phytools/issues/119))
- **No label thinning**: every label is drawn at the given size
  - Observed: a 522-tip tanglegram at `fsize=1` on a 7-inch page gives overlapping label boxes and a solid band of links
- **Long labels squeeze the trees** (observed): with labels of 62 characters in a 6-inch plot, `plot.cophylo` shrinks each tree to a narrow bracket, and the labels are cut at the page edges
- **Labels after rotation**: rotation keeps node labels with their clades, but ape `edgelabels` follows the row order of the edge matrix, so labels must be mapped to edges by hand ([blog, 2018](https://blog.phytools.org/2018/06/preserving-node-edge-labels-after.html))

### Clade labels

- **Clade bars** (`cladelabels`): a vertical bar with short wings beside the clade tips, and the text rotated 90 degrees or horizontal [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R#L1212-L1256)]
  - The offset is measured from the highest tip of the clade, so bars of different clades align only in ultrametric trees ([#26](https://github.com/liamrevell/phytools/issues/26))
- **Curved clade labels** (`arc.cladelabels`): fan trees only [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R#L727-L779)]
  - An arc at `ln.offset=1.02` times the tree radius spans the clade
  - The text follows the arc at `lab.offset=1.06` (`orientation="curved"`) or stands horizontal

### Node marks and arrows

- **Node numbers**: `node.numbers=TRUE` prints node indices in white boxes [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L385-L404)]
- **Click to label**: `labelnodes` puts text in a circle, ellipse, or rectangle on a node chosen by click (`getnode` picks the nearest node) or by number [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R#L783-L841)]
- **Arrows to tips**: `add.arrow` draws an arrow that points at a tip from beyond its label, in rectangular and fan layouts [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R#L245-L308)]

## Colour and legends

### Colour mapped onto branches

- **Continuous trait on branches** (`contMap`): ancestral values come from `fastAnc` (or `anc.ML`, or user values) [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/contMap.R#L18-L70)]
  - **Interpolation**: branches are cut at `res=100` global time steps, and each piece gets the value interpolated linearly between the parent and child values at the piece midpoint
  - **Palette**: `rainbow(1001, start=0, end=0.7)`, red to blue. `setMap` replaces it [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/contMap.R#L52-L56)]
  - **Encoding**: the result is a `simmap` tree with 1001 discrete states, so the gradient is drawn by `plotSimmap`
- **`contMap` special cases**:
  - **Zero-length branches** get the parent value [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/contMap.R#L62-L66)]
  - **Nodes only**: `nodes_only=TRUE` draws a plain tree with coloured circles at the nodes and tips [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/contMap.R#L144-L198)]
- **Posterior state probability** (`densityMap`): for two states, each piece of each branch gets the time-weighted mean of the state over all stochastic maps [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/densityMap.R#L19-L71)]
  - **Palette**: the mean is rounded to 1 of 1001 colours (`rainbow(1001, start=0.7, end=0)`, blue to red)
  - **Topology check**: all maps must share one topology, checked only with `check=TRUE`
  - **Scaling**: trees are first rescaled to the greatest height
- **Colour by branch value** (`plotBranchbyTrait`): one colour per branch from 1000 breaks over the value range plus `tol=1e-6` [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotBranchbyTrait.R#L4-L101)]
  - **Palettes**: `"rainbow"` (blue to red), `"heat.colors"`, `"gray"`, or a function
  - **Inputs**: values can be given for branches, tips (nodes by `fastAnc`), or nodes
  - **Reduction**: a branch value from tips or nodes is the mean of its two end values
- **Branch width by value** (`edge.widthMap`): the width follows the mean of the end values from `fastAnc`, optionally with tapering vertical polygons [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotBranchbyTrait.R#L171-L200)]

### Legends

- **Colour bar** (`add.color.bar`): one line segment per colour [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotBranchbyTrait.R#L106-L168)]
  - **Text**: the minimum and maximum sit above the ends, the title above the middle, and "length=<x>" below it
  - **Length**: the bar length is in branch-length units, so it doubles as a scale bar
  - **Placement**: without `prompt=FALSE`, the user clicks the position. A 4-tip `contMap` SVG has 289 KB, mostly bar and branch pieces (observed)
- **No ticks or classes on the bar** ([#164](https://github.com/liamrevell/phytools/issues/164), open)
- **Discrete legend** (`add.simmap.legend`): squares or circles with labels, vertical or horizontal, placed by click or by coordinates [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L641-L678)]
- **Geological time scale**: `geo.palette` holds the period boundaries and colours, and `geo.legend` draws them as background bands [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R#L502-L655)]

## Input and export

- **Readers**: phytools uses ape `read.tree` and `read.nexus` in examples [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/read.newick.R#L4-L106)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/readNexus.R#L37-L40)]
  - Its own `read.newick` accepts singleton nodes
  - `readNexus` reads bootstrap values stored as `[&label=...]`
- **`read.newick` conventions** (derived and observed):
  - **Missing lengths become 0**: `((A:1,B):1,C:2);` gives lengths `1,1,0,2` (observed). ape gives NaN instead [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/read.newick.R#L96)]
  - **Root length is discarded**: it is parsed into a local variable that never reaches the tree [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/read.newick.R#L60-L64)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/read.newick.R#L101-L102)]
  - **Comments are not removed**: labels stop at `,`, `:`, `)`, or `;` only [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/read.newick.R#L110-L115)]
- **Export**: any R graphics device (PDF, SVG, PNG)
  - The Cairo `svg()` device writes text as glyph outlines, so labels in the SVG are paths and cannot be searched or edited (observed: no `<text>` element in any output)
- **`plot=FALSE` and `setEnv`**: most plot functions can compute the layout without drawing and leave the coordinates in `last_plot.phylo` [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/densityTree.R#L62-L65)]
  - Book section 13.3 is titled "Plotting phylogenies without actually plotting them"
  - `densityTree` and `compare.chronograms` use this to set shared axis limits

## Scientific semantics

### Time and branch length

- **Branch length is the only time source**: positions come from cumulative branch lengths. Dates and node height annotations are not read
  - Trees are drawn from the root, so the right edge is the farthest tip, and serially sampled trees are drawn correctly
- **Time direction**: phylograms grow from the root
  - Functions that need ages flip the tree with `direction="leftwards"` and label the axis as time before the youngest tip (`plotTree.errorbars`, `densityTree`, `compare.chronograms`)
- **Ultrametric coercion**: `force.ultrametric` has two methods [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R#L66-L90)]
  - **`method="nnls"`**: least-squares fit to the cophenetic distances
  - **`"extend"`**: lengthen tip branches to the deepest tip
  - **Warning**: each call prints a box saying that the function is only for rounding errors and does not replace "formal rate-smoothing methods"
- **Floating-point heights**: the maintainer recommends tolerance comparisons for node heights, for example `abs(x - y) <= 1e-12` ([#91](https://github.com/liamrevell/phytools/issues/91))
  - `getExtant` uses `tol=1e-8` [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R#L1261-L1272)]
- **Zero-length branches**: drawn as zero-length lines, so a zero-length tip sits on its parent's connector
  - The slanted cladogram draws them as vertical lines
  - The `"weighted"` rule and the cophylo cladogram fail on them (see "Defects")

### Topology and values

- **Polytomies**: drawn as one connector with several children
  - Display functions never resolve them, except `phylo.to.map`, which calls `multi2di` and therefore draws a random binary resolution [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/phylo.to.map.R#L188-L189)]
- **Rerooting and support**: `reroot` splits the tree at a branch position and pastes it back [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R#L1330-L1368)]
  - Node labels follow the ape `root` convention unless `edgelabel=TRUE` is passed ([#40](https://github.com/liamrevell/phytools/issues/40))
  - For `midpoint.root`, the maintainer calls the moved support labels "a known issue with re-rooting in R" and points to `phangorn::midpoint` ([#107](https://github.com/liamrevell/phytools/issues/107))
- **Node values and branch values**:
  - A painted map belongs to the branch, from the parent end to the child end
  - `contMap` values belong to nodes and are interpolated along branches
  - `plotBranchbyTrait` reduces node values to one branch value by the mean
- **Tanglegram objective**: the squared displacement objective weights a link that moves 10 rows like 100 links that move 1 row
  - A reassortment that moves one large clade far can therefore win over a layout that crosses many small links

## Defects

Each entry gives the input, the effect, and the evidence. Observed entries were reproduced with version 2.6-9.

### Tanglegrams and rotation

#### Failures on TreeKnit-like input

- **Slanted cladogram with zero-length branches** (observed): `plot.cophylo(type="cladogram")` fails with "replacement has length zero" [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L238-L245)]
  - **Inputs**: `((A:1,B:0):1,(C:1,D:1):1);` and the TreeKnit fixture pair `sim_k2_n50_r0.02_poly`
  - **Cause**: the `1/v` weights become infinite
- **Unreadable large tanglegrams** ([#168](https://github.com/liamrevell/phytools/issues/168), [#153](https://github.com/liamrevell/phytools/issues/153), open): users with 522 tips see only lines or one tree
  - The trial at 522 tips drew both trees, but the labels and links formed a solid band (observed)
- **Factorial polytomy search** ([#23](https://github.com/liamrevell/phytools/issues/23), [#150](https://github.com/liamrevell/phytools/issues/150)): `permn` allocates `gamma(k + 1)` permutations
  - The maintainer suggests random resolution with `multi2di` as a workaround

#### Edge cases

- **Association table with one remaining row** (observed): `cophylo(t1, t2, assoc=rbind(c("A","A"), c("Z","Z")))` removes the row with Z and then fails with "incorrect number of dimensions" [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L24-L33)]
  - The filter `assoc[ii,]` drops the matrix to a vector
- **One-tip tree** (observed): `cophylo` with a one-tip tree fails during rotation with "missing value where TRUE/FALSE needed" [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L302-L303)]
  - The cause was not traced
  - Plotting would also divide the tip rows by `n - 1 = 0`
- **Exhaustive search without a size limit** (derived): `cophylo(..., methods="exhaustive")` calls `allRotations` on both trees directly [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L38-L41)]
  - The `max.exhaustive=20` guard of `tipRotate` therefore does not apply, and the 2^m * 2^m pairs exhaust memory for moderate trees
- **Malformed edge order after `rotateNodes`** (observed): `rotateNodes` keeps the attribute `order="cladewise"`, but the edges of a clade are no longer contiguous [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/utilities.R#L1851-L1865)]
  - `allRotations` on such a tree fails with "subscript out of bounds"
  - Writing and reading the tree back fixes it
  - `cophylo` is safe because it normalizes its inputs first

### Layout and drawing

#### Crashes and empty output

- **`nodes="weighted"` with a zero-length branch** (observed): `plotTree(read.tree(text="((A:1,B:1):1,(C:0,D:1):1.5);"), nodes="weighted")` fails with "replacement has length zero" [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L320-L326)]
- **Partly missing branch lengths** (observed): `plotTree` stops with "need finite 'xlim' values" (see "Coordinate system")
- **`contMap(type="cladogram")` draws nothing** (observed: an empty 181-byte SVG and no message) [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/densityMap.R#L139)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/densityMap.R#L195)]
  - `plot.densityMap` handles only `"phylogram"`, `"fan"`, and `"arc"`
- **`add.color.bar(direction="downwards")`** (observed): fails with "object 'X' not found" [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotBranchbyTrait.R#L132-L139)]
  - The code reverses `X` before it exists, and it reverses the wrong axis
- **`plot` of a one-tree `multiLtt`** (observed): fails with "subscript out of bounds", because the code reads `x[[2]]` [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/ltt.R#L482-L490)]
- **`collapseTree` with some empty node labels** (observed): fails with "argument to 'which' is not logical" [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/collapseTree.R#L25-L29)]
  - The parenthesis of `which(tree$node.label)==""` is misplaced

#### Code errors without a crash

- **Typo disables a guard** (derived): `split.vertical=TRUE` with `setEnv=FALSE` prints that the option is turned off, but assigns `spit.vertical` instead [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L291-L294)]
- **Wrong parenthesis in a tie test** (derived): `if(length(mm>1))` tests the length of a logical vector, which is always true when `mm` is not empty [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/plotSimmap.R#L332)]
  - The effect is harmless, because the tie rule then picks the lowest row

#### Maps

- **Map tree drawn upside down** ([#158](https://github.com/liamrevell/phytools/issues/158), open) and **maps across longitude 180** ([#159](https://github.com/liamrevell/phytools/issues/159), open)

### Parsing

- **`read.newick` hangs on a root branch length** (observed): `((A:1,B:1):1,C:2):5;` and any string that ends with `:<length>;` loop forever [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/readNexus.R#L26-L35)] [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/read.newick.R#L119-L125)]
  - **Cause**: a second function `getEdgeLength` in `readNexus.R` is loaded after the one in `read.newick.R` and replaces it
  - **Effect**: its stop characters lack `;`, so the loop runs past the end of the text
- **Annotations with commas change the topology** (observed): `((A:1,B:1)n1[&segments={0,1}]:1,C:2);` gives four tips `A B 1}] C` and the node label `n1[&segments={0`
- **One-tip Newick** ([#31](https://github.com/liamrevell/phytools/issues/31), closed): `read.newick` froze on `Ursus_americanus;`. The maintainer pointed to ape `read.tree`

## Performance

- **Rotation cost** (observed, identical topologies with half the nodes rotated): 2.1 s for 200 tips, 12.0 s for 500 tips, and 44.4 s for 1000 tips [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L492)]
  - Each candidate rotation writes the tree to Newick and parses it back with `untangle`, so one sweep costs O(n²), and the alternation adds further sweeps (derived)
- **Drawing cost** (observed): `plot.cophylo` took 0.16 s, 0.49 s, and 0.89 s for 200, 500, and 1000 tips
  - Each branch, connector, leader, label, and link is a separate graphics call
- **Output size** (observed): a 522-tip tanglegram SVG has 2.1 MB, and a 100-tip one 540 KB
  - Colour gradients multiply the segment count by up to `res=100` per branch
- **`ltt` of non-ultrametric trees** uses a double loop over times and branches, which is O(n²) [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/ltt.R#L323-L328)]
- **`make.simmap`** is the main speed complaint of the tracker ([#25](https://github.com/liamrevell/phytools/issues/25), [#45](https://github.com/liamrevell/phytools/issues/45), [#170](https://github.com/liamrevell/phytools/issues/170), all open)

## Interaction with TreeKnit output

TreeKnit writes three kinds of output ([packages/treeknit-io/src/arg.rs#L12-L86](../../packages/treeknit-io/src/arg.rs#L12-L86), [packages/treeknit-io/src/mccs.rs#L58-L60](../../packages/treeknit-io/src/mccs.rs#L58-L60), [packages/treeknit-io/src/output.rs#L206-L240](../../packages/treeknit-io/src/output.rs#L206-L240)):

- **Resolved trees**: Newick with internal labels and zero-length branches
- **ARG**: extended Newick with `label#Hi` hybrid nodes and `[&segments={0,1}]` on every branch
- **MCCs**: JSON, or one comma-separated MCC per line

phytools handles these files as follows:

- **Resolved trees** (observed with `fixtures/sim/sim_k2_n50_r0.02_poly`): ape `read.tree` keeps the labels such as `internal_63` as node labels and keeps the 28 zero-length branches
  - `plotTree`, fan layout, and `cophylo` with the phylogram style work
  - `plot.cophylo(type="cladogram")` fails (see "Defects")
- **Polytomies in resolved trees**: without `rotate.multi`, polytomy children stay in input order, which left 112 crossings on the 50-tip pair
  - With it, 39 crossings remained after 21.6 s (observed)
- **Tanglegram of a segment pair**: `cophylo(tree1, tree2)` links the shared labels
  - MCC membership can colour the links through `link.col`, one colour per association row
  - Colouring branches by MCC needs `edge.col` in the edge order of the rotated trees, which the user must compute
- **ARG file**:
  - **ape `read.tree`** (observed): the leaf occurrence of a hybrid becomes an extra tip named `R1#H1`, and the other occurrence a node labelled `R1#H1`. The annotations disappear. A tanglegram of this tree would show a spurious tip
  - **phytools `read.newick`** (observed): hangs, because the ARG string ends with `:0.0;`
  - **ape `read.evonet`** (observed): returns an `evonet` object with 1 reticulation. `plotTree` accepts it but draws only the tree part, without the reticulation and without a warning
- **Auspice JSON and `MCCs.json`**: phytools has no JSON reader for trees

## Ideas from the issue tracker and history

### Open requests

- **Tanglegram requests**:
  - **More than two trees in a row with linked tips** ([#87](https://github.com/liamrevell/phytools/issues/87), 2021): requested for gene tree incongruence and recombination between genes. The requester expects joint rotation across many trees to be hard
  - **Width of the link zone** ([#173](https://github.com/liamrevell/phytools/issues/173), 2026)
  - **Tree names above the two sides of a tanglegram** ([#28](https://github.com/liamrevell/phytools/issues/28), 2018)
  - **Movable scale bars in `cophylo`** ([#76](https://github.com/liamrevell/phytools/issues/76))
  - **Hiding labels in `cophylo`** ([#156](https://github.com/liamrevell/phytools/issues/156)): `ftype="off"` exists, but the user did not find it
- **Colour requests**:
  - **Cloudogram with branches coloured by clade and a grey backbone** ([#171](https://github.com/liamrevell/phytools/issues/171))
  - **Ticks and discrete classes on the colour bar** ([#164](https://github.com/liamrevell/phytools/issues/164))
  - **A colour for missing values** in heat maps and dot plots, because the black cross for missing data looks like a value in trees with 2000 tips ([#155](https://github.com/liamrevell/phytools/issues/155))
  - **Branch colours in `phylo.to.map`** ([#151](https://github.com/liamrevell/phytools/issues/151)): the user found `edge.col` in `cophylo`, but `phylo.to.map` ignores it
- **Other requests**:
  - **Time axis for stacked tree plots** ([#54](https://github.com/liamrevell/phytools/issues/54))
  - **Pull requests**: varying bar borders in `plotTree.wBars` ([#56](https://github.com/liamrevell/phytools/pull/56)) and custom offsets for `cladelabels` ([#55](https://github.com/liamrevell/phytools/pull/55)), open since 2019

### Declined

- **Large polytomies in `cophylo`** ([#23](https://github.com/liamrevell/phytools/issues/23)): "Not really" fixable, according to the maintainer. Random resolution was suggested instead
- **Correct support after midpoint rooting** ([#107](https://github.com/liamrevell/phytools/issues/107)): closed with a pointer to phangorn
- **One-tip Newick in `read.newick`** ([#31](https://github.com/liamrevell/phytools/issues/31)): closed with a pointer to ape

### Features found only in the history

- **History of `cophylo`**:
  - **2016**: sigmoid links ([`7e4ccd9`](https://github.com/liamrevell/phytools/commit/7e4ccd9)), polytomy rotation ([`290da2a`](https://github.com/liamrevell/phytools/commit/290da2a)), and exhaustive search ([`2de1be4`](https://github.com/liamrevell/phytools/commit/2de1be4))
  - **2017**: edge colours ([`ca832e7`](https://github.com/liamrevell/phytools/commit/ca832e7))
  - **2019**: duplicate tip labels ([`31c3f1b`](https://github.com/liamrevell/phytools/commit/31c3f1b))
  - **2021**: `cotangleplot` and the slanted cladogram ([`a54910f`](https://github.com/liamrevell/phytools/commit/a54910f), [`048d23d`](https://github.com/liamrevell/phytools/commit/048d23d))
  - **2023**: one type per tree ([`ab92cf4`](https://github.com/liamrevell/phytools/commit/ab92cf4))
  - **2024**: label boxes as an option ([`fae8255`](https://github.com/liamrevell/phytools/commit/fae8255))
- **Tree digitizer** (`tree.drawer`, 2017): the user clicked the root and the tips on a scanned tree image, and the function rebuilt the tree
  - It was removed in [`dbbb105`](https://github.com/liamrevell/phytools/commit/dbbb105) without a stated reason
- **`MULTI2DI`**: a helper added with polytomy support in 2016 ([`290da2a`](https://github.com/liamrevell/phytools/commit/290da2a)) that no code calls anymore [[src](https://github.com/liamrevell/phytools/blob/5c185533dbb3c3076200c1e0aa45de36701e1116/R/cophylo.R#L534-L540)]

## Open scientific problems

- **Which objective matches what readers see**: phytools minimizes squared displacement, while readers count crossings
  - Neither objective says how a reassortment should look
  - A swapped clade of many tips gives many crossings that may hide a single event
- **Optimality**: the greedy sweep has no guarantee, and the result depends on which tree comes first
  - Two-tree crossing minimization is NP-hard for binary trees ([Fernau et al. 2010](https://doi.org/10.1016/j.jcss.2009.10.014))
  - The exhaustive search in phytools is limited to about 20 tips
- **Polytomies**: an exact search over child orders is factorial, and a resolution with `multi2di` adds structure that the data do not support
- **Shared time scale**: drawing both trees on one time axis shows branch length differences, and drawing each tree at full width uses the space better
  - `cophylo` always chooses the second
- **More than two trees**: joint rotation for a chain of trees is an open request with no algorithm in phytools

## Not covered by phytools

- **Networks and ARGs**: no reading of hybrid nodes and no drawing of reticulations
- **A shared time axis in `cophylo`**: each tree is scaled on its own
- **Crossing count as an objective**: no function reports or minimizes the number of link crossings
- **Label thinning and collision handling**: labels are always drawn at full size
- **Interactivity**: no zoom, pan, hover, or selection
  - Interaction is limited to single `locator()` clicks in `collapseTree`, `labelnodes`, `reroot`, and legend placement
- **Searchable SVG text**: export depends on the R device, and the default Cairo SVG device converts text to outlines

## Method and limits

- **Code**: read at commit `5c18553` (pulled and unshallowed from an existing clone)
  - **Read in full**: `cophylo.R`, `cotangleplot.R`, `plotSimmap.R`, `contMap.R`, `densityMap.R`, `densityTree.R`, `phylo.to.map.R`, `plotBranchbyTrait.R`, `ltt.R`, `ltt95.R`, `collapseTree.R`, `cladebox.R`, `compare.chronograms.R`, `plotTree.errorbars.R`, `plotTree.wBars.R`, `splitplotTree.R`, `roundPhylogram.R`, `paintSubTree.R`, `fancyTree.R`, `read.newick.R`, `readNexus.R`, and the plotting helpers in `utilities.R`
  - **Only skimmed and not described**: `phylomorphospace`, `phylo.heatmap`, `dotTree`, `plotFanTree.wTraits`, `plotTree.datamatrix`, and `plotTree.lollipop`
- **History**: `git log` of the tanglegram files, the deleted files, and the export list in `NAMESPACE`
- **Tracker**: titles of all 158 issues and 16 pull requests through `gh`, and the full text of 30 issues
- **Trials**: phytools 2.6-9 installed from the surveyed commit into a `rocker/r-ver` container (with `libglpk40` added for `igraph`), run without network
  - **Inputs**: test trees, two TreeKnit simulation fixtures, and a hand-written ARG string
  - **Inspection**: the SVG files were converted to PNG and inspected
  - **Brute force**: the comparison used only 8-tip trees, because the pair search grows as 4^(n-1)
- **Fetch failures**:
  - **Paper**: the PeerJ page returned HTTP 403 and PubMed Central showed a browser check. The author's PDF copy was read instead
  - **Book**: only the publisher's table of contents was available. The companion site `phytools.org/Rbook` failed with an expired TLS certificate
  - **Blog**: the search pages were cut off. Individual posts were read from search results and direct links
- **Not verified**: the cause of the "two vertical lines" plot in [#168](https://github.com/liamrevell/phytools/issues/168). The trial with the same tip count did not reproduce it
