# ggtree feature survey: tree rendering, semantics, and edge cases

This report describes ggtree, an R/Bioconductor package that draws phylogenetic trees as ggplot2 graphics, as an idea inventory for tree, tanglegram, network, and ARG display. ggtree turns a tree into a table with one row per node (`node`, `parent`, `x`, `y`, `label`, `isTip`, and the node data), and every display feature is a ggplot2 layer or a function that changes this table. The report also covers the companion packages treeio (file parsers) and ggtreeExtra (aligned outer layers) where they define display features. It records what the packages do, how they do it, which scientific conventions they assume, which inputs break them, and which problems are open. It does not compare ggtree with `packages/web`.

ggtree and treeio are licensed Artistic-2.0. ggtreeExtra is licensed GPL-3.0 or later: its behavior and design may be studied, but copying its code needs approval (see the project rules).

- **Source**: three repositories, and source links point to these commits
  - [YuLab-SMU/ggtree](https://github.com/YuLab-SMU/ggtree) at commit `68bb553` (2026-09-14, version 4.3.1), 2186 commits since 2014
  - [YuLab-SMU/treeio](https://github.com/YuLab-SMU/treeio) at commit `2cbfece` (2026-09-21, version 1.37.1), 639 commits since 2016
  - [YuLab-SMU/ggtreeExtra](https://github.com/YuLab-SMU/ggtreeExtra) at commit `b353b67` (2026-06-04, version 1.23.1), 327 commits since 2020
- **Other sources**:
  - the book "Data Integration, Manipulation and Visualization of Phylogenetic Trees" (<https://yulab-smu.top/treedata-book/>), chapter 2 for the two-tree recipe
  - the papers [Yu et al. 2017](https://doi.org/10.1111/2041-210X.12628) (ggtree), [Yu et al. 2018](https://doi.org/10.1093/molbev/msy194) (two methods to map data on trees), and [Yu 2020](https://doi.org/10.1002/cpbi.96) (protocols)
  - the titles of all 560 ggtree issues, 150 ggtree pull requests, 59 treeio issues, and 34 ggtreeExtra issues, with the full text and comments of the issues cited below
  - the NEWS files, the commit history, and the development plans in `.dev/` of the ggtree repository (a roadmap, a backlog, and a tanglegram design)
- **Evidence labels**: two labels mark how a claim was checked
  - **"Observed"**: seen in a trial run of the surveyed commits (R 4.x in a `rocker/r-ver` container, ggtree and treeio installed from the cloned sources)
  - **"Derived"**: read from the code without running it
  - **Documentation and code**: where they disagree, the code wins and the difference is noted
- **Shape of the code**: R on top of ggplot2
  - **Tree table**: `fortify()` methods compute `x` and `y` per node for each layout [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/method-fortify.R#L9-L79)]. The coordinate algorithms live in [`R/tree-utilities.R`](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tree-utilities.R)
  - **Edges**: ggplot2 stats turn the table into line segments [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_tree.R#L54-L161)]
  - **Annotation layers**: most layers are small objects that a `ggplot_add()` method turns into ggplot2 layers once it knows the plot data and the layout [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/method-ggplot-add.R)]
  - **Tree changes**: `collapse()`, `rotate()`, `flip()`, and `scaleClade()` edit the `x` and `y` columns of a finished plot [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/clade-functions.R)]
  - **Two trees**: [`R/tanglegram.R`](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tanglegram.R), added in March 2026

## Summary

### Two trees, links, and networks

- **Native tanglegram since 2026**: `ggdoubletree()` puts two rectangular trees face to face and links matching tips with curves. The paired view refuses circular and unrooted layouts (see "Two trees and tanglegrams")
  - **Untangling**: a deterministic greedy pass orders the children of each node by the mean rank of their linked tips in the other tree. It keeps the new order only when the number of crossing link pairs drops
  - **Observed on TreeKnit output**: on a 50-leaf pair the crossings fell from 246 to 125 with both sides optimized. On a 1997-leaf H3N2 pair they fell from 137,279 to 67,908, but the optimization took 216 s
  - **Defects**: link colour cannot be mapped to data, tip labels overlap the links and the right tree, and the documented `cophylo` input fails (see "Defects")
- **Older recipe**: the book mirrors the second tree by hand and joins same-label tips with lines
  - **Mirror**: `x' = max(x2) - x2 + max(x1) + 1`
  - **Links**: `geom_line(aes(group = label))`
  - **Three or more trees**: the same recipe without mirroring places them in a row
- **`geom_taxalink()`**: a curve between any two nodes of one tree, given by label or node number
  - it is the only tool for links inside one tree, and the only way to draw a reticulation
- **No network support**: treeio has no extended Newick or `evonet` reader, and ggtree has no network layout
  - **Observed**: a TreeKnit ARG loads as a tree. Each reassortment adds one extra tip named `ARGNode_n#Hk`, and no reticulation is drawn
  - **Workaround (observed)**: one `geom_taxalink()` per hybrid pair, by node number, draws the reticulations as dashed curves

### Scientific conventions

- **Tip order comes from `ladderize = TRUE`**: by default every tree is ladderized before layout, so the input child order is lost unless the user sets `ladderize = FALSE`
- **One missing branch length turns the tree into a cladogram**: all lengths are dropped with a warning that `ggtree()` itself suppresses (observed)
  - TreeKnit ARGs with two segment roots contain two such missing lengths
- **A parent sits at the mean `y` of its children**: a polytomy parent is pulled toward the side with more children
- **Node data and branch data share one row**: a branch attribute is stored on the child node, as in IcyTree

### Display features worth studying

- **Many layouts**: 15 named layouts, from rectangular to unrooted daylight, plus any igraph layout function
- **Collapse with three triangle shapes** (`min`, `max`, `mixed`) and a height control
- **Gradient branch colour**: `continuous = "colour"` splits each branch into 100 pieces and blends the parent and child values
- **Aligned data panels**: heatmaps, alignments, and any ggplot2 geom can sit next to the tips in the same panel or in extra panels with a shared `y`

## Layouts

### Layout list

`ggtree(tr, layout = ...)` accepts 15 names [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/ggtree.R#L74-L76)]. All 14 that draw were observed on a random 12-tip tree.

#### Rectangular family

- **Rectangular** (default): elbow branches. Branch length on `x`, leaf order on `y`
- **Slanted**: straight lines from parent to child. Without branch lengths a separate placement is used [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/method-fortify.R#L38-L45)]
- **Roundrect and ellipse**: each branch is one curve from parent to child, drawn with grid `curveGrob` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_tree.R#L384-L419)]
  - **Roundrect**: a quarter-circle corner (curvature 1, angle 90)
  - **Ellipse**: a half-strength curve (curvature 0.5, angle 20 or 160)
  - continuous colour and size are refused for these two layouts
- **Dendrogram**: the rectangular layout turned so that the root is at the top. Tip labels then default to `angle = 90` and `hjust = 1` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/method-ggplot-add.R#L199-L211)]

#### Circular family

- **Circular, fan, inward circular, radial**: the rectangular coordinates under `coord_polar(theta = "y")`. The layout is added as a separate step, so `p + layout_circular()` converts an existing plot [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/ggtree.R#L138-L157)]
  - **Fan**: `open.angle` leaves a gap in degrees. `layout_fan()` has a default of 180 degrees [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/layout.R#L111-L113)]
  - **Inward circular**: the root is on the outside and the tips point to the centre. `xlim` sets the size of the empty centre [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/layout.R#L93-L98)]
  - **Radial**: circular coordinates with straight (slanted) branches. The option `layout.radial.linetype = "curved"` switches back to elbow segments [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_tree.R#L125-L142)]
  - **`open_tree(p, angle)` and `rotate_tree(p, angle)`**: open a gap in a circular plot and turn it [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/layout.R#L15-L53)]

#### Unrooted layouts

`equal_angle`, `daylight`, `ape`, and `tree_and_leaf` use `coord_fixed()`. `unrooted` is an alias of `daylight` and prints a message [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/ggtree.R#L85-L88)].

- **Equal angle** (Felsenstein): each child gets a wedge in proportion to its number of tips, and the branch points to the middle of the wedge [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tree-utilities.R#L105-L129)]
- **Daylight**: starts from equal angle and rotates subtrees to equalize the gaps between them [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tree-utilities.R#L156-L189)]
  - **Stop rule**: at most `MAX_COUNT = 5` rounds, or until the mean angle change is at most 0.05
  - **Messages**: each round prints a message (observed: "Average angle change [1] 0.139", then 0.044)
- **Ape**: the coordinates of `ape::unrooted.xy` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tree-utilities.R#L1682-L1724)]
- **Tree and leaf** (2026, after the TreeAndLeaf package): a force simulation on top of daylight [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tree-utilities.R#L208-L325)]
  - **Forces**: tips repel each other, branches act as springs toward their length, and anchors pull nodes back to the start positions
  - **Defaults**: 200 iterations, step 0.2, cooling 0.98 per iteration, tolerance `1e-4`

#### Other placements

- **Tidy** (September 2026): the non-layered tidy tree algorithm of [van der Ploeg 2014](https://doi.org/10.1002/spe.2213) on the `y` axis. It packs subtrees to reduce vertical space, so two tips can share one `y` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tree-utilities.R#L1332-L1360)]
  - **Guard**: `gheatmap()` and `msaplot()` warn when tips are not on unique, evenly spaced rows [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/utilities.R#L33-L64)]
- **External layout functions**: a function such as `igraph::layout_with_kk` can be passed as `layout`, and its result replaces `x` and `y` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/ggtree.R#L193-L226)]
  - the tree is converted to an igraph graph unless `layout.params = list(as.graph = FALSE)`
- **`yscale`**: any numeric or categorical node attribute can replace `y`, which gives a "2D tree" (for example a trait value on `y`) [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/ggtree.R#L90-L93)]
  - any `yscale` forces the slanted layout

### Coordinates

- **`x` from branch lengths**: the root is at `root.position` (default 0), and each child is at `x(parent) + length` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tree-utilities.R#L1020-L1050)]
- **Cladogram** (`branch.length = "none"` or no lengths): tips at the same `x`, and each internal node one unit left of its leftmost child [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tree-utilities.R#L1057-L1100)]
  - Observed with `(((A,B),C),(D,(E,(F,G))));`: all tips at 4, the root at 0, the node of `((A,B),C)` at 2, and the node of `(D,(E,(F,G)))` at 1
- **`y` from tip order**: tips get rows `1..N` in the order of the edge matrix. Each internal node takes the **mean** of its children's rows [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tree-utilities.R#L1364-L1415)]
  - Observed with `((A:1,B:1):1,C:2,D:2);` and `ladderize = FALSE`: the root is at 2.83, the mean of 1.5, 3, and 4. The midpoint of the outer children would be 2.5
  - Row 1 is at the bottom of the plot, because ggplot2 `y` grows upward
- **Edge segments**: in the rectangular family each node has two segments [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_tree.R#L171-L262)]
  - **Horizontal**: from `x(parent)` to `x(node)` at `y(node)`
  - **Vertical**: at `x(parent)` from `y(parent)` to `y(node)`
  - **Aesthetics**: both take the node's aesthetics, so a vertical connector has the colour of the child below it
  - The root has a segment of length 0 to itself unless `rootnode = FALSE`, so the number of segments equals the number of nodes [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_tree.R#L189-L197)]
- **Branch midpoints**: every row gets `branch = (x(parent) + x)/2` for labels on branches [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tree-utilities.R#L1647-L1661)]
  - a missing `branch.length` becomes 0 in the table
- **Label angles for circular layouts**: `angle = 360 / (range(y) + 1) * y` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/fortify-utilities.R#L6-L18)]

### Child order

- **Ladderize by default**: `ggtree()` calls `ape::ladderize(right = FALSE)` before the layout [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/method-fortify.R#L21-L23)]
  - **Observed** with `((A,(B,(C,D))),E);`, from top to bottom:
    - default: `D C B A E`, so the smallest clade is at the bottom
    - `right = TRUE`: `E A B D C`
    - `ladderize = FALSE`: `E D C B A`
  - The input order of children is kept only with `ladderize = FALSE`, so a meaningful child order (for example from TreeKnit's own ordering) needs that option
- **Manual changes after layout**: see "Clade operations"

## Branches, nodes, and colour

### Colour groups

- **`groupOTU()` and `groupClade()`**: add a group column for colouring
  - **`groupOTU(tree, list_of_tip_sets)`** marks each set of tips and the paths up to their common ancestor
    - Observed with TreeKnit's five MCCs: 3, 6, 13, 16, and 46 nodes got the groups `MCC1` to `MCC5`, and the root kept the group `0`
    - Non-monophyletic MCCs colour the whole path to their ancestor, so the colours of two MCCs can meet on shared branches
  - **`groupClade(tree, nodes)`** marks whole clades
  - **An extra group `0`** for unassigned nodes is a known complaint ([#533](https://github.com/YuLab-SMU/ggtree/issues/533), open)

### Branch style and colour scales

- **Branch aesthetics**: `colour`, `linetype`, and `linewidth` can be mapped to any node column. `size` is translated to `linewidth` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_tree.R#L74-L86)]
- **Gradient along branches**: `continuous = "colour"`, `"size"`, or `"all"` splits each branch into `nsplit` pieces and interpolates from the parent's value to the child's value [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_tree.R#L198-L224)]
  - a missing value becomes 0
  - The documentation says the default `nsplit` is 200, but the code uses 100 [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_tree.R#L17-L18)] [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_tree.R#L176-L178)]
- **Colour scales are ggplot2 scales**: any ggplot2 or extension scale can replace the defaults
  - **Discrete values**: the ggplot2 default hue palette (observed)
  - **Numeric values**: a continuous gradient with a colour bar legend (observed)

### Node marks and root edge

- **Node marks**: `geom_nodepoint()`, `geom_tippoint()`, and `geom_rootpoint()` are point layers restricted to internal nodes, tips, or the root [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_point.R#L20-L115)]
- **Root edge**: `geom_rootedge(rootedge)` draws a stub of the given length left of the root. Without an argument it uses the tree's `root.edge` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_rootedge.R#L53)]

## Labels

### Tip labels

`geom_tiplab()` places text at `x + range(x)/200` with `hjust = 0` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_tiplab.R#L155-L162)].

- **`geom`**: `text`, `label` (boxed), `shadowtext`, `image`, or `phylopic` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_tiplab.R#L134-L151)]
- **`align = TRUE`**: every label moves to `max(x) + range(x)/200`, and a dotted line joins each tip to its label [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_tiplab.R#L16-L18)] [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_tiplab.R#L181-L192)]
  - **Line defaults**: `linetype = "dotted"`, `linesize = 0.5`
  - **Documentation and code**: the documentation speaks of "padding characters", but the code draws a segment
- **Circular layouts**: labels are turned to follow the radius by `geom_tiplab2()` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/method-ggplot-add.R#L235-L242)]
- **`as_ylab = TRUE`**: labels become the `y` axis text, outside the panel, so they never overlap the tree. Rectangular and dendrogram layouts only [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/method-ggplot-add.R#L248-L294)]
- **Overlap**: ggplot2's `check.overlap = TRUE` drops labels that overlap earlier ones
  - ggtree has no label thinning of its own and no repel layer
  - `ggrepel` layers work on the tree table because it is an ordinary data frame

### Node, branch, and clade labels

- **Node labels** (`geom_nodelab()`): text at internal nodes, centred (`hjust = 0.5`) [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_nodelab.R#L24)]
- **Labels on branches**: the `branch` column puts text at the branch midpoint. There is no dedicated layer ([#561](https://github.com/YuLab-SMU/ggtree/issues/561), open)
- **Clade labels**: `geom_cladelab(node, label)` draws a bar along the clade's tips and text next to it [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_cladelab.R#L133)]
  - **Placement options**: `offset`, `offset.text`, `align`, `extend`, and `angle = "auto"` for circular layouts
  - **Bar options**: `barsize`, `barcolour`
  - **`geom`**: text, label, shadowtext, image, phylopic
- **Strips**: `geom_strip(taxa1, taxa2, label)` draws a bar between any two tips, for groups that are not clades [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_strip.R#L35)]

## Highlighting and annotation

- **`geom_hilight(node)`**: a filled shape behind a clade [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_hilight.R#L8-L18)]
  - **Types**: four shapes
    - `rect`: default for rectangular, circular, and related layouts
    - `encircle`: an x-spline hull, default for unrooted layouts
    - `gradient`: fill fades from root to tips or back. Observed: in a circular plot it only prints a warning that it works in rectangular, ellipse, and roundrect layouts
    - `roundrect`
  - **Extent**: the rectangle starts half a branch length left of the clade root and extends 0.5 rows above and below the outer tips. The option `clade_width_extend` changes the 0.5 [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_hilight.R#L429-L461)]
  - **`align`**: `left`, `right`, or `both` give all rectangles a common edge
  - **Legend**: a mapped `fill` creates a legend automatically. The old `set_hilight_legend()` is defunct [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/defunct.R#L187-L188)]
- **`geom_balance(node)`**: two coloured boxes for the two child clades of a node [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_balance.R#L35)]
- **`inset()` and `geom_inset()`**: small ggplot2 charts (pies, bars) placed at nodes or branch midpoints [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/inset.R#L42-L60)]
  - `nodepie()` and `nodebar()` build them from node data
  - width and height must lie in (0, 1)
  - insets fail on circular layouts ([#599](https://github.com/YuLab-SMU/ggtree/issues/599), open)

## Axes, time, and scale bars

### Axes and time

- **`theme_tree()`**: no axes. **`theme_tree2()`**: an `x` axis with ticks and no `y` axis [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/theme.R#L20-L104)]
- **Time-scaled trees with `mrsd`**: the most recent sampling date shifts `x` so that the tip farthest from the root sits at that date in decimal years. `as.Date = TRUE` gives a Date axis [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/Date.R#L12-L26)]
  - Observed with `((A:1,B:2):1,C:3);` and `mrsd = "2020-06-01"`: B and C at 2020.41, A at 2019.41, and the root at 2017.41
  - **Decimal-year conversion**: day of year divided by 366 when the year is divisible by 4, else 365. The inverse always multiplies by 365 [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/Date.R#L37-L63)]
- **`revts(p)`**: shifts `x` so that the farthest tip is at 0 and the past is negative (observed: tips at 0 and -1, the root at -3) [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/axis.R#L124-L133)]
- **`xlim_tree()` and `xlim_expand()`**: set the `x` range of the tree panel alone when extra panels exist [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/axis.R#L63-L88)]

### Scale bars and intervals

- **`geom_treescale()`**: a scale bar with its length printed above it. The default position is `x = range/2`, `y = 0` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_treescale.R#L186-L227)]
  - **Default length**: a tenth of the `x` range
    - a value below 1 is cut to one significant digit
    - a value of 1 or above loses its fraction and keeps all its digits
    - Observed on a TreeKnit tree with an `x` range of 19,041: the bar is 1904 units long, an odd value for a scale bar
- **`geom_range(range)`**: horizontal bars for node intervals such as `height_0.95_HPD` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_range.R#L15-L70)]
  - **Placement**: the bar runs from `x + center - lower` to `x + center - upper`
    - with the default `center = "auto"` the centre is the midpoint of the interval, so the bar is always symmetric around the node, even when the interval is skewed around the point estimate
    - `center = "height"` gives the true placement
  - **Direction**: the formula assumes that the values are heights (ages). An interval of a forward quantity such as `length_0.95_HPD` comes out mirrored

## Clade operations

All of these functions change the `x` and `y` columns of a finished plot. They take node numbers, which the user finds with `geom_nodelab(aes(label = node))` or `nodeid(tree, label)`.

### Reorder

- **`rotate(p, node)`**: reverses the order of all tips below the node, which mirrors the whole clade, at every depth [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/clade-functions.R#L329-L358)]
  - **Name clash** (observed): `ape::rotate` masks `ggtree::rotate` when ape is attached after ggtree, and the call fails with "object "phy" is not of class "phylo""
- **`flip(p, node1, node2)`**: swaps two sister clades. Nodes with different parents are refused [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/clade-functions.R#L378-L436)]

### Collapse and scale

- **`collapse(p, node, mode)`**: hides a clade [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/clade-functions.R#L156-L255)]
  - **`mode = "none"`** (default): the clade becomes one tip row, and the rows above it move down
  - **`max`, `min`, `mixed`**: a triangle that keeps the clade's rows unless `height` is set
    - `max`: from the clade root to the farthest tip
    - `min`: from the clade root to the nearest tip
    - `mixed`: one corner at each
  - **`height`** (2026): `NULL`, `rel(0.2)` for 20% of the clade's span, or an absolute number of rows ([#409](https://github.com/YuLab-SMU/ggtree/issues/409))
  - **`clade_name`**: written into the label of the clade root. Observed: `geom_tiplab()` does not show it, because the collapsed node is still an internal node
  - **`expand(p, node)`** restores the clade from data stored as plot attributes [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/clade-functions.R#L271-L315)]
- **`scaleClade(p, node, scale)`**: stretches or squeezes a clade's rows by a factor, and with `vertical_only = FALSE` also its depth. Other rows move to make room [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/clade-functions.R#L456-L513)]
- **`as.polytomy(tree, feature, fun)`**: sets to 0 the branches whose feature passes a test (for example support below 50) and merges them into polytomies with `ape::di2multi` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/as.polytomy.R#L13-L42)]

### Zoom

- **`viewClade(p, node)`**: zooms the axes to one clade with `coord_cartesian` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/clade-functions.R#L55-L68)]
- **`zoomClade(p, node)`**: the whole tree and a zoomed clade in two panels, through `ggforce::facet_zoom` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/clade-functions.R#L550-L565)]

## Two trees and tanglegrams

### `ggdoubletree()`

`ggdoubletree(x, y, assoc, ...)` was added in ggtree 4.1.2 ([NEWS](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/NEWS.md?plain=1#L44), commit [`9de2590`](https://github.com/YuLab-SMU/ggtree/commit/9de2590), 2026-03-17) [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tanglegram.R#L48-L95)].

- **Inputs**: two trees, two finished ggtree plots, or a `phytools::cophylo` object
  - **`assoc`**: a table with `left` and `right` columns of tip labels. Unnamed columns are taken as left and right. One-to-many links are allowed [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tanglegram.R#L301-L313)]
  - **Missing labels**: a label in `assoc` that is not a tip stops the plot with "Association labels cannot be found in tree tips (right: 32_0)" (observed). Tips without an entry in `assoc` have no link [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tanglegram.R#L558-L587)]
- **Defaults**: the argument defaults of `ggdoubletree()`
  - **Layout and order**: `layout = "rectangular"`, `ladderize = TRUE`, `mirror = TRUE`
  - **Crossing reduction**: `optimize = FALSE`, `optimize_side = "right"`
  - **Spacing and layers**: `gap = 0.08`, `preserve_layers = TRUE`
- **Rectangular only**: raw trees must use `"rectangular"`, and plot inputs must be rectangular, slanted, or roundrect [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tanglegram.R#L249-L290)]
- **Geometry** [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tanglegram.R#L319-L367)]:
  - **Gap**: a value below 1 is a fraction of the wider tree's `x` range. A value of 1 or more is in data units
  - **Mirror**: the right tree is drawn at `x' = W_right - x + W_left + gap`, so its root is at the right edge and its tips face the left tree. Without mirroring both trees point right
  - **Unaligned tips**: each tree keeps its own branch lengths, so in non-ultrametric trees the tips of one tree are at different distances from the gap
  - **One table**: the two tree tables are stacked, the right node numbers are shifted by the largest left node number, and columns `side`, `tree_id`, and `orig_node` are added [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tanglegram.R#L620-L669)]
- **Shared scale**: both trees share one `x` axis, so their branch lengths are comparable. A tree with a much larger height compresses the other

### Crossing reduction

#### Algorithm

- **Score**: the number of crossing link pairs. Links are sorted by left rank and the inversions of the right ranks are counted with a double loop, so the cost grows with the square of the number of links [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tanglegram.R#L461-L477)]
- **One pass** on the side being optimized [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tanglegram.R#L407-L443)]:
  - every tip gets a target, the mean rank of its linked tips in the other tree
  - from the root down, the children of each node are sorted by the mean target of their tips (a barycentre heuristic), with the current mean row as tie-break
  - tips get new rows and internal nodes are placed again at the mean of their children [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tanglegram.R#L445-L459)]
- **Loop**: the new order is kept only when the score drops [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tanglegram.R#L479-L556)]
  - with `optimize_side = "both"` the right and the left tree take turns
  - the loop stops when nothing improves or after `max_iter = 10` rounds
- **Topology is kept**: only the order of children changes
- **Diagnostics**: `attr(p$data, "tangle_optimize")` holds the score before and after and the number of rounds

#### Observed results

Observed on TreeKnit resolved trees (`sim_k2_n50_r0.05_poly`, 50 leaves, 5 MCCs):

| `optimize_side` | crossings before | crossings after | rounds |
| --------------- | ---------------: | --------------: | -----: |
| none            |              246 |             246 |      0 |
| right           |              246 |             210 |      2 |
| left            |              246 |             175 |      2 |
| both            |              246 |             125 |      3 |

#### Limits

- **Plan and code differ**: the design note in `.dev/tanglegram-implementation.md` proposes to test single node rotations and accept the best one. The code sorts all children in one sweep instead
- **No goal beyond crossings**: the score ignores link length, the MCC structure, and the distance from the ladderized order

### Links: `geom_tanglelink()`

- **Shape**: `geom = "curve"` (default, `geom_curve` with `curvature = 0.15`) or `"segment"` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tanglegram.R#L755-L775)]
  - observed: all curves bend to the same side, and long links between distant rows become wide arcs
- **End points**: links join the tip points. They do not start at the end of the tip labels (see "Defects")
- **Style**: constant `colour`, `linewidth`, `alpha`, and `linetype` pass through `...`. A mapped colour is ignored (see "Defects")

### Layer replay

- **What is copied**: with plot inputs, every layer after the first is copied to the paired plot, and its data is mirrored and shifted like the tree [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tanglegram.R#L690-L737)]
  - layers without own data get the side's tree table, which is how tip points coloured by MCC appear on both trees (observed)
  - layers whose data is a function are dropped silently
- **The first layer only is skipped**: a rectangular `geom_tree()` is two layers (horizontal and vertical segments), so the vertical segments of each input tree are replayed on top of the new tree [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tanglegram.R#L280-L282)]
  - Observed: 9 layers for two inputs with points and labels, where 7 are expected
  - with a styled input tree the replayed vertical connectors keep the input's style and the horizontal ones do not

### Hand-made two-tree plots (book recipe)

- **Mirror by hand**: `d2$x <- max(d2$x) - d2$x + max(d1$x) + 1`, then `p1 + geom_tree(data = d2)` (book chapter 2.6, "ggtree-fortify")
- **Links**: `geom_line(aes(x, y, group = label), data = bind_rows(d1, d2))` after dropping internal nodes
  - `geom_line` joins all rows with the same label in order of `x`, so with three trees in a row one polyline runs through all of them
- **Second tree's labels**: `geom_tiplab(data = d2, hjust = 1)`
- **Second scale bar**: a second `geom_treescale(x = ...)` with a shifted position ([#278](https://github.com/YuLab-SMU/ggtree/issues/278))
- **Experimental leftovers**: two older two-tree functions remain in the code
  - **`coplot()`**: uses the removed `layout = "phylogram"` and plyr's `.()` subset syntax, so it cannot run [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/experimental_function.R#L187-L216)]
  - **`plot_fantrees()`**: stacks two fan plots as grobs [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/experimental_function.R#L62-L66)]

### Other multi-tree views

- **Facets**: a `multiPhylo` object becomes one table with a `.id` column, so `facet_wrap(~.id)` draws the trees side by side with independent layouts (observed) [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/method-fortify.R#L84-L101)]
- **`ggdensitree()`**: draws many trees on top of each other with one common tip order, like DensiTree [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/ggdensitree.R#L65-L149)]
  - **Tip order**: one of four sources
    - `mode`: the most frequent order among the trees
    - `mds` or `mds_dist`: a one-dimensional MDS of tip-to-tip path lengths or distances over all trees
    - the order of tree N
    - a given vector
  - **Placement**: every tree is laid out with the common tip order, and internal nodes take the mean row of their children. Where trees disagree, branches cross
  - **Alignment**: `align.tips = TRUE` (default) puts the farthest tip of every tree at `x = 0`
  - **Jitter**: `jitter` adds random noise to the tip rows of all trees except the first
  - **Default layout**: `slanted`
  - **Argument leak** (observed): the extra arguments go to both `fortify()` and `geom_tree()`, so `alpha` or `colour` produce "Arguments in `...` must be used" warnings, although the styling works
- **`gzoom()`** showed a tree and a zoomed subtree side by side. It was removed in 2019 ([`4f72df8`](https://github.com/YuLab-SMU/ggtree/commit/4f72df8)) and now points to `treeio::tree_subset()` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/defunct.R#L101-L127)]

## Links inside one tree: `geom_taxalink()`

- **Purpose**: a curve between two nodes of one tree, given by label or node number. The book uses it for horizontal gene transfer and similar events [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_taxalink.R#L35-L64)]
- **Data mode**: a table with `aes(taxa1 = ..., taxa2 = ...)` draws many links, and `colour`, `linewidth`, `linetype`, and `alpha` can be mapped
- **Curve shape**: grid `curveGrob` with `curvature = 0.5`, `ncp = 1`, and `curveangle = 90` by default [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_taxalink.R#L176-L189)]
  - **Polar layouts**: the curvature is computed from the angle between the two ends, and `hratio` scales it. `outward = "auto"` bends the curves away from the centre, except in the inward circular layout [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_taxalink.R#L247-L291)] [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/method-ggplot-add.R#L890-L900)]
  - **`offset`**: shifts both ends to the right by a fraction of the `x` range, for example past the tip labels
- **Ends**: the node positions themselves, so an internal node link starts at the node and a tip link at the tip point [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/method-ggplot-add.R#L919-L932)]
- **Duplicate labels**: a label resolves to its first node only. Two tips with the same name cannot both be linked by label ([#610](https://github.com/YuLab-SMU/ggtree/issues/610), open, a gene tree with two copies linked to a genome tree)

## Networks and reticulations

- **No network model**: treeio and ggtree know only rooted trees (`phylo`, `treedata`)
  - treeio has no `evonet` method, no extended Newick reader, and no hybrid node handling
  - the word "network" occurs in the treeio code only in error messages that refuse networks
- **Edge lists and graphs with two parents are refused**: `as.phylo()` stops with an error
  - **Edge table**: "Cannot find root. network is not a tree!" ([treeio #80](https://github.com/YuLab-SMU/treeio/issues/80))
  - **igraph graph**: "The igraph is a network not a tree graph." [[src](https://github.com/YuLab-SMU/treeio/blob/2cbfece4545788b64e9af274b258a4033bea8a9a/R/method-as-phylo.R#L172)] [[src](https://github.com/YuLab-SMU/treeio/blob/2cbfece4545788b64e9af274b258a4033bea8a9a/R/method-as-phylo.R#L275)]
- **Extended Newick read as a tree** (observed with `((A:2,(B:1)#H1:1):1,(#H1:1,C:2):1);`): `ape::read.tree` makes `#H1` an internal node label and also a fourth tip
  - the network becomes a tree with one extra leaf and no link between the two copies
- **Drawing reticulations by hand** (observed): add one dashed `geom_taxalink()` per hybrid pair, by node number
  - **Steps**: find the two nodes of each hybrid label in the plot table, then add `geom_taxalink(taxa1 = <node>, taxa2 = <node>, linetype = "dashed")`
  - **Result**: the leaf copy stays visible as a short unlabelled branch, and the curve runs from the hybrid node to the end of that branch
- **Roadmap**: "tree + network-style links" is listed as a candidate recipe in the development roadmap, without design or code (`.dev/ggtree-roadmap.md`)

## Aligned data panels

### In the tree panel

- **`gheatmap(p, data)`**: a matrix as coloured tiles to the right of the tips, in the tree panel [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/gheatmap.R#L43-L204)]
  - **Placement**: the first column starts at `max(x) + offset`, and each column is `width * range(x) / ncol` wide
  - **Rows**: matched to tip labels by row name. Missing rows are filled with NA and a warning
  - **Collapsed clades**: a collapsed clade gets a row when its label matches a row name
  - **Numeric colours**: a gradient with the defaults `low = "green"` and `high = "red"`, which red-green colour-blind readers cannot tell apart
  - **Other colours**: the default discrete palette
  - **Several heatmaps**: need `ggnewscale::new_scale_fill()` between them
- **`msaplot(p, fasta)`**: a multiple sequence alignment as coloured tiles next to the tips, with an optional `window` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/msaplot.R#L25-L60)]

### Extra panels and outer layers

- **`geom_facet()` and `facet_plot()`**: any ggplot2 geom in an extra panel that shares the tree's `y`. The first column of `data` is matched to tip labels [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/facet_plot.R#L1-L45)]
  - **Panel widths**: `facet_widths()` sets them, and it is reported broken in recent versions ([#685](https://github.com/YuLab-SMU/ggtree/issues/685), open)
  - **Axis limits**: a plain `xlim()` clips every panel, so `xlim_tree()` is needed ([#658](https://github.com/YuLab-SMU/ggtree/issues/658))
- **ggtreeExtra `geom_fruit()`**: any geom as an outer layer, in rectangular and circular layouts, without extra panels [[src](https://github.com/YuLab-SMU/ggtreeExtra/blob/b353b67109d84f344f63b5607567357a5c1a58e2/R/geom_fruit.R#L162-L174)]
  - **Width**: a layer is `pwidth = 0.2` of the tree's `x` range wide
  - **Offset**: a layer starts `offset = 0.03` of the range after the previous one
  - **Guides**: optional grid lines and pseudo axes are drawn per layer

## Input formats and data joining (treeio)

### Readers and writers

- **Readers**: `read.newick`, `read.nexus`, `read.beast` (also `read.mrbayes`), `read.beast.newick`, `read.nhx`, `read.raxml`, `read.iqtree`, `read.jplace`, `read.phyloxml`, `read.mega`, `read.mega_tabular`, `read.codeml`, `read.codeml_mlc`, `read.paml_rst`, `read.mcmctree`, `read.hyphy`, `read.r8s`, `read.astral`, `read.treetime`, `read.treeqza` (QIIME 2), `read.jtree`, `read.nextstrain.json`, `read.phylip`, `read.fasta` [[src](https://github.com/YuLab-SMU/treeio/blob/2cbfece4545788b64e9af274b258a4033bea8a9a/NAMESPACE#L59-L87)]
- **Writers**: `write.beast`, `write.beast.newick`, `write.jplace`, `write.jtree`, `write.nexus`, `write.tree` [[src](https://github.com/YuLab-SMU/treeio/blob/2cbfece4545788b64e9af274b258a4033bea8a9a/NAMESPACE#L96-L101)]

### Tree data and annotations

- **`treedata` object**: a `phylo` tree plus a data table keyed by node number. MrBayes files use the BEAST reader [[src](https://github.com/YuLab-SMU/treeio/blob/2cbfece4545788b64e9af274b258a4033bea8a9a/R/beast.R#L56)]
- **Annotations attach to nodes**: the `[&...]` blocks are split at every `:` and named by node number [[src](https://github.com/YuLab-SMU/treeio/blob/2cbfece4545788b64e9af274b258a4033bea8a9a/R/beast.R#L310-L311)]
  - BEAST 1 branch annotations after the length are moved in front of the length first, so they land on the child node [[src](https://github.com/YuLab-SMU/treeio/blob/2cbfece4545788b64e9af274b258a4033bea8a9a/R/beast.R#L296-L298)]
  - Vectors such as `{0,1}` become list columns (observed: `segments` as a list of numeric vectors)
- **Newick internal labels**: kept as labels by default. `read.newick(file, node.label = "support")` turns them into a numeric `support` column [[src](https://github.com/YuLab-SMU/treeio/blob/2cbfece4545788b64e9af274b258a4033bea8a9a/R/newick.R#L11-L24)]
- **Auspice JSON**: `read.nextstrain.json()` walks the `tree` object [[src](https://github.com/YuLab-SMU/treeio/blob/2cbfece4545788b64e9af274b258a4033bea8a9a/R/nextstrain.json.R#L10-L91)]
  - **Branch lengths**: `div` differences
  - **Attributes**: `node_attrs` are flattened, and the `.value` suffix is removed
  - **Dates**: `num_date` is read as data and is not used for positions

### Joining data and input types

- **Joining user data**: `p %<+% df` joins a table to the plot data by its first column, matched to labels of tips and internal nodes
  - the operator moved to the ggfun package in 2024 ([NEWS](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/NEWS.md?plain=1#L98))
  - `full_join()` and `as.treedata()` do the same on the tree object
- **Many inputs for `ggtree()`**: `phylo`, `multiPhylo`, `treedata`, `phylo4`, `phylo4d`, `hclust`, `dendrogram`, `agnes`, `diana`, `pvclust`, `igraph` trees, `phyloseq`, and `obkData` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/method-fortify.R#L153-L287)]

## Export and interaction

- **Export**: every plot is a ggplot2 object, so `ggsave()` writes PNG, PDF, SVG, and the other ggplot2 formats at any size and resolution
  - Observed: an SVG of the 1997-tip H3N2 tree with tip labels is 836 KB
- **Interactive SVG**: since 2025 the tree and most layers use ggiraph geoms
  - `ggtree_set_interactive(width, height)` makes printed plots open as girafe SVG widgets with tooltips and hover from `tooltip` and `data_id` aesthetics [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/method-print.r#L79-L87)]
  - plotly conversion stopped working because plotly does not know the ggiraph geoms ([#712](https://github.com/YuLab-SMU/ggtree/issues/712), open)
- **No zoom or pan of its own**: zooming is `viewClade()`, `zoomClade()`, or axis limits, each a new static plot

## Scientific semantics

### Branch length and time

- **Branch lengths only**: positions come from `edge.length`. Date and height annotations are never used for `x`, except when the user maps them through `yscale` or writes their own `x`
- **One missing length drops all lengths**: any NA in `edge.length` sets all lengths to NULL with a warning, and the tree becomes a cladogram [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/method-fortify.R#L25-L30)]
  - Observed with `((A:1,B):1,C:2);`: all tips at `x = 2` and no warning on screen, because `ggtree()` wraps the call in `suppressWarnings()` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/ggtree.R#L101-L113)]
- **Negative lengths**: drawn as given, with a warning that suggests `options(ignore.negative.edge = TRUE)`. That option uses the absolute value [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tree-utilities.R#L1030-L1047)]
  - Observed with `((A:1,B:-0.5):1,C:2);`: B is at `x = 0.5`, left of its parent at 1
- **Time origin**: `mrsd` pins the farthest tip from the root to the given date. In a tree whose youngest sample is not the farthest tip (rate variation) the axis is shifted
- **Unrooted trees**: the rooted layouts draw the Newick root as given. The unrooted layouts ignore the root position for drawing but start the wedges from it

### Polytomies and zero-length branches

- **Polytomies**: drawn as given, with the parent at the mean row of its children
- **Zero-length branches are kept**: a zero-length tip sits on its parent's `x`, and its label starts at the node
  - Observed with `((A:0,B:1):1,C:2);`: A at `x = 1`, the same as its parent
- **No collapse option**: the user collapses short branches with `as.polytomy()` or `ape::di2multi()`
- **Very short branches**: a request to extend them with dotted lines, as ETE does, is open ([#640](https://github.com/YuLab-SMU/ggtree/issues/640))

### Node and branch attributes

- **One row per node**: node and branch values share the row of the child node
  - `geom_range()` assumes heights
  - colour by an attribute colours both the horizontal branch and the vertical connector above the node
- **Support values**: a plain number column
  - ggtree does not move them when the tree is rerooted, and rerooting is delegated to treeio and ape ([treeio NEWS](https://github.com/YuLab-SMU/treeio/blob/2cbfece4545788b64e9af274b258a4033bea8a9a/NEWS.md?plain=1#L255))
  - a rerooted IQ-TREE tree shows a new polytomy at the root in ggtree but not in FigTree ([#644](https://github.com/YuLab-SMU/ggtree/issues/644), open)
- **Several support values per node**: a layer for symbols of several support methods was proposed by the maintainer and is open ([#451](https://github.com/YuLab-SMU/ggtree/issues/451))

## Defects

Each entry gives the input, the effect, and the evidence. Observed entries were reproduced with the surveyed commits.

### Tanglegram

- **Mapped link colour is ignored** (observed and derived): `geom_tanglelink(data = ..., mapping = aes(colour = mcc))` draws black links
  - the layer uses the mapping only to rename the `left` and `right` columns, and builds the geom with `x`, `y`, `xend`, and `yend` alone [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/method-ggplot-add-tanglegram.R#L25-L38)]
- **First argument is `data`** (observed): `geom_tanglelink(aes(colour = x))` fails with "cannot coerce class ... to a data.frame", because ggplot2 geoms take `mapping` first [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tanglegram.R#L755-L761)]
- **`cophylo` input fails** (observed): `ggdoubletree(cophylo(t1, t2))` stops with "argument is of length zero"
  - the arguments `y = NULL` and `assoc` travel through `...` into `fortify()`, where `y` partially matches `yscale` [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tanglegram.R#L206-L223)]
  - the backlog lists "Test `cophylo` plotting via `ggdoubletree()`" as not done
- **Tip labels and links overlap** (observed): links start at the tip points [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/geom_tiplab.R#L155-L158)]
  - **`hjust = 0`** (default): the right tree's labels lie over its own branches
  - **`hjust = 1`**: the labels lie on top of the links
  - **`align = TRUE`** on the right tree: the labels go beyond its root, with dotted lines across the whole tree, because the alignment uses the largest `x` of the mirrored data
- **Replayed vertical segments** (derived and observed, see "Layer replay"): the second layer of each input tree is copied as an annotation layer
- **Quadratic crossing count** (observed): 216 s for one optimized pair of 1997-leaf trees, mostly in the inversion count that every candidate order calls

### Links inside one tree

- **`geom_taxalink()` error message fails** ([#615](https://github.com/YuLab-SMU/ggtree/issues/615), open, observed): a missing `taxa1` or `taxa2` gives the message "Can't convert `NULL` to a string" and no name of the missing taxon [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/method-ggplot-add.R#L914-L918)]
  - the error message is built from the `data` mapping, which is NULL in argument mode

### Parsing

- **`read.beast.newick()` fails on missing lengths** (observed): on a TreeKnit ARG it stops with "'names' attribute [100] must be the same length as the vector [99]" [[src](https://github.com/YuLab-SMU/treeio/blob/2cbfece4545788b64e9af274b258a4033bea8a9a/R/beast.R#L310-L311)]
  - **Cause**: the annotations are named by splitting the text at `:`, so one node without a length shifts every name
  - **Workarounds that fail**: removing the annotations or the `#H` suffixes does not help
- **`read.nextstrain.json()` fails on mixed value types** (observed): on a TreeKnit Auspice file it stops with "Can't combine `..1$mcc_tree1_tree2` <character> and `..2$mcc_tree1_tree2` <double>" [[src](https://github.com/YuLab-SMU/treeio/blob/2cbfece4545788b64e9af274b258a4033bea8a9a/R/nextstrain.json.R#L53-L58)]
  - **Cause**: numeric-looking strings are converted per node, so the string `"null"` of one node and `"3"` of another get different types
  - **Earlier fix**: the fix for [treeio #126](https://github.com/YuLab-SMU/treeio/issues/126) covers mixed types inside one node only

### Display

- **Branch lengths, dates, and intervals**
  - **Silent cladogram** (observed, see "Branch length and time"): a single missing length drops all lengths without a visible warning
  - **Odd scale bar lengths** (observed, see "Axes, time, and scale bars"): 1904 for a tree of height 19,041
  - **Decimal dates do not round-trip** (derived): `Date2decimal` divides by 366 in leap years, and `decimal2Date` multiplies by 365. The century rule is also missing, so 2100 counts as a leap year [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/Date.R#L37-L63)]
  - **Symmetric HPD bars** (derived, see "Axes, time, and scale bars"): the default centre of `geom_range()` hides skew
- **Clade operations and circular layouts**
  - **`rotate()` on a tip removes the parent branch** ([#373](https://github.com/YuLab-SMU/ggtree/issues/373), open)
  - **Labels after `rotate()` or `collapse()` in fan layouts** have wrong angles ([#499](https://github.com/YuLab-SMU/ggtree/issues/499), [#647](https://github.com/YuLab-SMU/ggtree/issues/647), open)
  - **Collapsed triangles in circular layouts** are curved ([#593](https://github.com/YuLab-SMU/ggtree/issues/593), open)
  - **Circular arcs** are coarse ([#589](https://github.com/YuLab-SMU/ggtree/issues/589), [#614](https://github.com/YuLab-SMU/ggtree/issues/614), open)
- **Console messages**
  - **`geom_treescale()` warning** (observed): every use prints the ggplot2 deprecation warning for `size` on lines
  - **Daylight messages**: the daylight layout prints one message per round, also inside other functions

## Performance

Observed on the 1997-leaf H3N2 HA tree from TreeKnit's resolved output, in a container on a shared machine:

| Step                                           | Time   |
| ---------------------------------------------- | ------ |
| `ggtree()`, rectangular                        | 0.21 s |
| render to PNG, rectangular                     | 0.12 s |
| render to PNG, circular                        | 2.3 s  |
| `ggtree()`, equal angle                        | 13 s   |
| `ggtree()`, daylight                           | 136 s  |
| `ggdoubletree()` of HA and NA, no optimizing   | 0.15 s |
| `ggdoubletree()` with `optimize_side = "both"` | 216 s  |

- **Layout cost**: the equal angle layout calls `offspring()` once per node [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tree-utilities.R#L86)], and daylight repeats rotations over all internal nodes up to five times
- **Tree and leaf**: the tip repulsion compares all pairs of tips in each of up to 200 iterations, so it grows with the square of the number of tips [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/tree-utilities.R#L253-L271)]
- **Roadmap**: "Improve scalability for large trees", with benchmarks and a faster `gheatmap()`, is a listed priority without work so far (`.dev/ggtree-roadmap.md`)

## Interaction with TreeKnit output

Observed with the output of `treeknit --auspice-view --plot` on `fixtures/sim/sim_k2_n50_r0.05_poly` and with the resolved H3N2 HA and NA trees.

- **Resolved trees** (`*_resolved.nwk`): load with `read.tree()` and draw correctly
  - the `RESOLVED_n` splits of length 0 appear as zero-length internal branches, and nothing marks them
- **Tanglegram of the resolved trees**: `ggdoubletree(t1, t2, data.frame(left = labels, right = labels))` works, because TreeKnit keeps the leaf names identical across trees
  - see "Crossing reduction" for the observed scores
- **Colour by MCC**: `MCCs.json` lists the leaves of each MCC under `MCC_dict`
  - **Tip points**: turned into a table of `label` and `mcc`, it joins with `%<+%` and colours tip points on both sides of a tanglegram (observed)
  - **Branches**: `groupOTU()` with the MCC leaf sets colours branches
  - **Links**: link colour by MCC needs a plain `geom_segment()` on the link table in `attr(p$data, "tangle_assoc")`, because of the mapping defect of `geom_tanglelink()`
- **ARG** (`ARG/arg.nwk`):
  - **`read.tree()`** loads it as a tree with 55 tips for 50 leaves
    - the five extra tips are the leaf copies `ARGNode_n#Hk` of the reassortment nodes
    - the `[&segments=...]` annotations are dropped
  - **Two missing lengths**: when neither segment root is shared, TreeKnit writes a synthetic `GlobalRoot` above the two segment roots, and those roots have no length (`fn Writer.data()` in [packages/treeknit-io/src/arg.rs#L82-L85](../../packages/treeknit-io/src/arg.rs#L82-L85), [packages/treeknit-io/src/arg.rs#L25-L27](../../packages/treeknit-io/src/arg.rs#L25-L27))
    - ggtree then drops all lengths and draws the ARG as a cladogram without a visible warning
  - **`read.beast.newick()`** fails on the same two missing lengths (see "Defects")
  - **Reticulations** can be added by hand as dashed `geom_taxalink()` curves after the missing lengths are set to 0 (see "Networks and reticulations")
- **Auspice JSON** (`auspice_<tree>.json`): `read.nextstrain.json()` fails (see "Defects")
  - **Cause**: MCC values are strings, and nodes outside every MCC carry the string `"null"` (`fn auspice_json()` in [packages/treeknit-io/src/auspice.rs#L44](../../packages/treeknit-io/src/auspice.rs#L44))

## Ideas from the issue tracker and history

### Open requests

#### Two trees and links

- **Links between duplicated tip names** in a cophylogeny ([#610](https://github.com/YuLab-SMU/ggtree/issues/610))
- **Planned for tanglegrams** in `.dev/ggtree-roadmap.md` and `.dev/implementation-backlog.md`:
  - a `tangle_diagnostics()` helper that reports crossings and rounds
  - more control over link styling, and a list of the layer types that survive replay
  - "shared-clade detection and highlighting" and "consensus / disagreement visualization across trees", listed as possible extensions

#### Time trees

- **Explicit handling of TreeTime outputs** ([#583](https://github.com/YuLab-SMU/ggtree/issues/583))
- **Contour lines for time trees** ([#415](https://github.com/YuLab-SMU/ggtree/issues/415))

#### Branches, labels, and interaction

- **Extend very short branches with dotted lines** so that the topology stays visible ([#640](https://github.com/YuLab-SMU/ggtree/issues/640))
- **Several support values per node** as symbols ([#451](https://github.com/YuLab-SMU/ggtree/issues/451))
- **Labels on edges** ([#561](https://github.com/YuLab-SMU/ggtree/issues/561))
- **Repel-style placement of insets** ([#532](https://github.com/YuLab-SMU/ggtree/issues/532))
- **Interactive plots**: plotly support ([#712](https://github.com/YuLab-SMU/ggtree/issues/712), [#478](https://github.com/YuLab-SMU/ggtree/issues/478)) and ggiraph questions ([#623](https://github.com/YuLab-SMU/ggtree/issues/623))

### Declined

- **A built-in `cophyloplot` equivalent** ([#594](https://github.com/YuLab-SMU/ggtree/issues/594), 2023): closed with a pointer to the book recipe. The native `ggdoubletree()` came three years later
- **Two trees around a heatmap**, as in `pheatmap` ([#462](https://github.com/YuLab-SMU/ggtree/issues/462)): the maintainer pointed to the aplot package for composing plots
- **Subsetting through the tree table** ([treeio #80](https://github.com/YuLab-SMU/treeio/issues/80)): users are sent to `ape::keep.tip`

### Features found only in the history

- **`gzoom()`** (removed 2019, [`4f72df8`](https://github.com/YuLab-SMU/ggtree/commit/4f72df8)): a tree and a zoomed subtree side by side
- **`coplot()`**: an early two-tree view with a mirrored second tree and labels on both sides. It is still in the code but uses removed syntax [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/experimental_function.R#L187-L216)]
- **Tanglegram as a TODO since 2020** ([`6f2f83f`](https://github.com/YuLab-SMU/ggtree/commit/6f2f83f)): the item pointed to the book recipe and to `phytools::cophylo` for six years before the 2026 code
- **`reroot()` and `rescale_tree()`** moved from ggtree to treeio in 2018 and 2019 ([`9125827`](https://github.com/YuLab-SMU/ggtree/commit/9125827), [`a0cd85c`](https://github.com/YuLab-SMU/ggtree/commit/a0cd85c))
- **`scale_color_subtree()`** moved to ggtreeDendro in 2022 [[src](https://github.com/YuLab-SMU/ggtree/blob/68bb5532c1b0ed9283f2ca7b9b0df57c40d49b79/R/defunct.R#L1-L12)]

## Open scientific problems

- **Crossing reduction for large tanglegrams**: the greedy barycentre pass halves the crossings
  - it ignores link length and clade structure
  - its cost grows with the square of the number of links
- **Labels and links in a tanglegram**: no rule places tip labels so that they neither cover links nor the other tree
- **Networks in a tree grammar**: a tree table with one parent per node cannot hold a reticulation, so every reticulation is an annotation drawn over a tree
- **Time from heights or dates**: `mrsd` assumes that the tip farthest from the root is the most recent sample
- **Node or branch meaning of values**: values share the child node's row, and rerooting and `geom_range()` assume a meaning that the file does not record

## Not covered by ggtree

- **Network layouts**: no drawing of hybrid nodes, no extended Newick parser, and no ARG model
- **Tanglegrams in non-rectangular layouts**: the paired view is rectangular only
- **Label-aware links**: links always join tip points
- **Interactive zoom, pan, and selection**: every view is a static plot. ggiraph gives tooltips and hover only
- **Automatic label thinning**: only ggplot2's `check.overlap`
- **Collapse of zero-length branches** for display

## Method and limits

- **Read**: the code and the project documents of the three packages
  - **R sources**: ggtree, treeio (`beast.R`, `newick.R`, `nextstrain.json.R`, `NAMESPACE`), and ggtreeExtra (`geom_fruit.R`)
  - **Tests**: the ggtree tests for tanglegrams
  - **History and plans**: the NEWS files, the commit history, and the `.dev/` planning documents of ggtree
- **Book**: chapter 2 of the treedata book was fetched for the two-tree recipe
  - other chapters were not read in full, so book-only recipes may be missing
- **Issues**: titles of all issues and pull requests of the three repositories were scanned, and about 15 issues were read in full
  - most of the 174 open ggtree issues were judged by title only
- **Trial**: ggtree 4.3.1 and treeio 1.37.1 were installed from the cloned sources in a `rocker/r-ver` container, with CRAN dependencies from binary packages
  - **Setup**: about 5 minutes
  - **Inputs**: small test trees, the TreeKnit outputs of `fixtures/sim/sim_k2_n50_r0.05_poly`, and the resolved H3N2 HA and NA trees (1997 leaves), rendered to PNG and SVG and inspected
- **Not run**: ggtreeExtra, `gheatmap()`, `msaplot()`, the interactive mode, and the tidy and tree-and-leaf layouts on large trees
- **Timings**: single runs on a shared machine, so they show orders of magnitude only
