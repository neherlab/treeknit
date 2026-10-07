# ape feature survey: tree rendering, semantics, and edge cases

This report describes the tree and network plotting of ape (Analyses of Phylogenetics and Evolution), the base R package for phylogenetics, as an idea inventory for tree, tanglegram, network, and ARG display. It records what ape draws, how it computes the layouts, which scientific conventions it assumes, which inputs break it, and which problems remain open. It does not compare ape with `packages/web`.

ape is licensed GPL-2 or GPL-3 (the user chooses). Both are copyleft licenses: the behavior and design of ape may be studied, but copying its code needs approval (see the project rules).

- **Source**: [emmanuelparadis/ape](https://github.com/emmanuelparadis/ape) at commit `c73e48d` (2026-09-15, version 5.8-3). Source links point to this commit
  - **Public repository**: 284 commits since 2020-12-28
  - **Package age**: the package itself dates from 2002 (NEWS starts at version 0.1)
- **Other sources**:
  - the man pages and the vignette `DrawingPhylogenies.Rnw` in the repository
  - the NEWS file (the full change history since 2002)
  - all 89 issues and 66 pull requests on GitHub, with the comments of the plotting-related ones
  - the ape 5.0 paper ([Paradis and Schliep 2019](https://doi.org/10.1093/bioinformatics/bty633))
  - the papers that ape cites for its algorithms: tidy trees ([van der Ploeg 2014](https://doi.org/10.1002/spe.2213)), extended Newick ([Cardona et al. 2008](https://doi.org/10.1186/1471-2105-9-532)), and support values after rerooting ([Czech et al. 2017](https://doi.org/10.1093/molbev/msx055))
  - the ape book ([Paradis 2012](https://doi.org/10.1007/978-1-4614-1743-9)) is cited for context only, see "Method and limits"
- **Evidence labels**:
  - **Observed**: seen in a trial run of the surveyed commit, built from source in a throwaway `rocker/r-ver` container (R 4.6.1)
  - **Derived**: read from the code without running it
  - **Conflicts**: where the documentation and the code disagree, the code wins and the difference is noted
- **Shape of the code**: R functions that draw with the base `graphics` package (`segments()`, `text()`, `polygon()`), with a few C routines for coordinates
  - **Layout and drawing**: `plot.phylo()` and its helpers `phylogram.plot()`, `cladogram.plot()`, `circular.plot()`, `unrooted.xy()`, and `tidy.xy()` in [`R/plot.phylo.R`](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R). Node coordinates come from [`src/plot_phylo.c`](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/src/plot_phylo.c)
  - **Two trees**: [`R/cophyloplot.R`](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/cophyloplot.R) with its own coordinate function [`R/plotPhyloCoor.R`](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plotPhyloCoor.R)
  - **Networks**: [`R/evonet.R`](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/evonet.R)
  - **Annotation**: [`R/nodelabels.R`](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/nodelabels.R), [`R/scales.R`](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/scales.R), [`R/plot.phyloExtra.R`](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phyloExtra.R)
  - **Parsers**: [`R/read.tree.R`](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/read.tree.R), [`R/read.nexus.R`](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/read.nexus.R), and the C parser [`src/tree_build.c`](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/src/tree_build.c)
  - **Shared plot state**: every plot stores its coordinates in the list `last_plot.phylo` in the environment `.PlotPhyloEnv`. All annotation functions read this list instead of the tree

## Summary

### Two trees

Details are in "Two trees and tanglegrams".

- **`cophyloplot()` draws two trees face to face**: it matches tips through a two-column association matrix
  - **Links**: each link is two horizontal stubs and one connecting segment
  - **Link style**: link colour, width, and line type are per-link vectors
  - **Rotation**: a click mode rotates nodes one at a time to reduce crossings
- **No automatic untangling**: crossing reduction is manual or a one-shot heuristic
  - **Manual**: `rotate = TRUE`
  - **Heuristic**: `rotateConstr()` sorts children by the smallest target position of their tips
- **Weak spots**: the layout and styling of `cophyloplot()` have gaps
  - **Spacing and colour**: the space between the trees is measured in character counts, and the left tree is drawn red in cladogram mode
  - **Height**: the right tree is stretched to the height of the left tree
  - **Missing support**: edge styling, label size, and singleton nodes are not supported

### Networks

Details are in "Networks and reticulations".

- **Data model**: an `evonet` object is a tree plus a two-column `reticulation` matrix of extra parent-to-child edges
  - **Reading**: `read.evonet()` reads extended Newick and keeps the occurrence with children as the tree node
- **Drawing**: `plot.evonet()` draws the base tree and then each reticulation as one straight, semi-transparent blue segment between the two node positions
  - **Arrows**: optional
  - **Layout**: no space is reserved, and no layout takes the reticulations into account
- **Lost data**: the length of each reticulation edge and all `[&...]` annotations are discarded on read

### Scientific conventions

Details are in "Scientific semantics".

- **Branch lengths drive positions**: ape draws the stored lengths
  - **Missing lengths**: one missing length switches the whole tree to a topology-only layout with a warning
  - **Negative lengths**: drawn backwards without a warning
  - **Zero-length edges**: drawn as they are (no collapsing)
- **Node labels are node data**: rerooting keeps labels on nodes unless the user asks for `root(edgelabel = TRUE)`
  - **Support values**: `edgelabel = TRUE` is the only correct choice for them
- **Time axis**: `axisPhylo()` takes the tip farthest from the root as time 0, or uses `root.time` to label absolute dates

### Risks on real inputs

Details are in "Defects".

- **Labels**: two label forms break `read.tree()`
  - a label longer than 512 bytes crashes R (stack buffer overflow, observed)
  - doubled single quotes in a quoted label turn the label into `NA` (observed)
- **Networks**: `read.evonet()` fails on a tree without reticulations and when all occurrences of a hybrid node are leaves (observed)
- **`node.depth = 2`**: this default of `plot.evonet()` and `cophyloplot()` causes wrong drawings (observed)
  - **Unrooted layouts**: it breaks them
  - **`cophyloplot()`**: it does something different from what the documentation says

### TreeKnit output

Details are in "Interaction with TreeKnit output".

- **Loading**: resolved trees and the ARG load and draw
- **Tanglegram**: a pair tanglegram coloured by MCC takes about five lines of R
- **Lost data**: the `[&segments=...]` annotations are lost

### Design

- **One static pass**: ape draws a static figure in one pass with R base graphics
  - **Drawing**: layout functions compute node coordinates, then plain line segments and text are drawn
  - **Annotation**: a set of separate low-level calls (`nodelabels()`, `edgelabels()`, `axisPhylo()`) read the stored coordinates of the last plot
  - **Interaction**: clicks through `locator()` are the only interaction

## Layouts

### Coordinate model

The vignette describes the drawing as seven steps [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/vignettes/DrawingPhylogenies.Rnw#L101-L117)]:

1. compute node coordinates
2. measure the tip labels
3. rotate or mirror
4. set the axis limits
5. open the plot
6. draw `segments()`
7. draw `text()`

- **Node depth with branch lengths**: the x position of a node is the sum of the edge lengths from the root, computed in one preorder pass. The root is at 0 [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/src/plot_phylo.c#L10-L20)]
- **Node depth without branch lengths** (`node.depth`): tips get 1, then [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/src/plot_phylo.c#L22-L48)]:
  - `node.depth = 1` (default): a node gets the number of tips below it, and the x position is `max - (count - 1)`
    - **Observed** on an 8-tip tree: a cherry sits at x = 6, the root at 0, and the tips at 7
    - **Effect**: small clades hang close to the tips (the classic "V" cladogram spacing)
  - `node.depth = 2`: a node gets one more than its highest child, so internal levels are evenly spaced
    - **Observed** on the same tree: tips at 3, the root at 0
- **Node height** (`node.pos`): tips get the rows 1 to n in edge order. Then [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/src/plot_phylo.c#L50-L104)]:
  - `node.pos = 1`: a node takes the **mean of its direct children**
    - **Polytomies**: the node is pulled toward the side with more children
    - **Observed** with `((A:1,B:1):1,C:2,D:2);`: the root sits at y = 2.833, the mean of 1.5, 3, and 4
  - `node.pos = 2`: a node takes the **mean of all tips below it** (children weighted by their tip counts)
  - **Default**: 2 for a cladogram drawn without branch lengths, otherwise 1 [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L159-L180)]
- **Child order**: the order of the rows of the edge matrix after `reorder()` to cladewise order. ape does not sort children at plot time. `ladderize()` and `rotate()` change the stored tree instead (see "Tree manipulation for display")
- **Recorded state**: after each plot, `last_plot.phylo` holds the plot settings and all node coordinates [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L532-L543)]
  - **Settings**: `type`, `use.edge.length`, `node.pos`, `node.depth`, `show.tip.label`, `show.node.label`, `font`, `cex`, `adj`, `srt`, `no.margin`, `label.offset`, `x.lim`, `y.lim`, `direction`, `tip.color`, `Ntip`, `Nnode`, `root.time`, `align.tip.label`
  - **Tree and coordinates**: the original `edge` matrix, and the vectors `xx` and `yy` of all node coordinates (tips first)
  - **Reuse**: other packages build on this list, and maintainers point users to it for custom drawing ([#145](https://github.com/emmanuelparadis/ape/issues/145))
- **`plot = FALSE`**: computes the coordinates and sets up the plot region without drawing, for later custom drawing

### Layout types

`type` accepts any unambiguous abbreviation [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L54-L57)].

- **`"phylogram"`** (default): rectangular elbow tree. Each node gets one vertical segment that spans its children, and each edge one horizontal segment [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L546-L578)]
- **`"cladogram"`**: each edge is one straight segment from parent to child, so the tree looks like a "V" diagram [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L711-L713)]
  - **Branch lengths**: only the x projection of each slanted edge is the branch length
- **`"fan"`**: circular phylogram [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L199-L223)]
  - **Angles and radius**: tip angles are evenly spaced over `2π(1 - 1/n) - open.angle`, an internal node takes the mean angle of its children, and the radius is the node depth
  - **Arcs**: drawn as polylines with `2 + |Δθ| / 0.03` segments, which made large fan trees faster and the files smaller ([#129](https://github.com/emmanuelparadis/ape/pull/129)) [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L758-L768)]
  - **Without branch lengths**: the radius is `(max_r - r + 1) / max_r`. The root is therefore drawn at radius `1/max_r` with its own small arc around the centre (observed: the root at radius 0.125 for 8 tips) [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L213-L219)]
- **`"unrooted"`**: equal-angle layout, which the vignette attributes to Felsenstein's book _Inferring Phylogenies_ (2004) [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/vignettes/DrawingPhylogenies.Rnw#L283-L290)]
  - **Wedges**: each child gets a wedge of the parent's angle in proportion to its weight, and its node is placed along the wedge bisector at the edge length [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L771-L802)]
  - **Weight is the node depth vector**: `unrooted.xy()` receives `node_depth(..., node.depth)` [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L224-L232)]
    - **`node.depth = 1`**: the weight is the tip count (true equal-angle)
    - **`node.depth = 2`**: the weight is the level, and wedges no longer add up (see "Defects")
  - **No equal-daylight**: ape implements only equal-angle. The root is placed at the start of the recursion, so the drawing depends on where the stored root is
  - **Aspect ratio**: forced to 1 for fan, radial, and unrooted trees [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L362)]
- **`"radial"`**: tips on the unit circle, internal nodes at radius `1 - depth/n`, connected by straight segments [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L233-L240)]
  - **Branch lengths**: always ignored, as NEWS has said since the feature was added ([NEWS](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/NEWS#L3883-L3884))
  - **Observed**: the coordinates for `use.edge.length = TRUE` and `FALSE` are identical
- **`"tidy"`** (since 5.6-2, 2022): a phylogram whose rows are compressed with the contour algorithm of [van der Ploeg (2014)](https://doi.org/10.1002/spe.2213), as proposed for phylogenies in the paper [tidy tree (MBE 2022)](https://doi.org/10.1093/molbev/msac204)
  - **Contours**: sister subtrees are moved together until their contours are one row apart, and tip label widths are added to the contours so that labels do not collide [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L181-L193)] [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L952-L1052)]
  - **Negative lengths**: refused with an error [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L89-L91)]
  - **Cost**: the R implementation is slow (observed: 4.8 s for 1,000 tips, 19.6 s for 5,000 tips, see "Performance")

### Direction and rotation

- **`direction`**: `"rightwards"` (default), `"leftwards"`, `"upwards"`, or `"downwards"` for the phylogram, cladogram, and tidy types
  - **Mirroring**: leftwards and downwards mirror the coordinates inside the axis limits [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L243-L253)] [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L308-L309)]
  - **Facing trees**: a leftwards tree next to a rightwards tree is the manual way to draw facing trees (an example in the man page)
- **`rotate.tree`**: rotation in degrees for fan, unrooted, and radial trees
- **`open.angle`**: a gap in degrees for fan and radial trees, needed to place `axisPhylo()` in the gap

## Branches and nodes

### Edge styling

- **Per-edge vectors**: `edge.color`, `edge.width`, and `edge.lty` are recycled and indexed in the order of the edge matrix [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L95-L117)]
  - **Reordering**: when the tree is reordered for plotting, the vectors are reordered with it so that values stay on their edges
- **Defaults from graphical parameters**: `par("fg")`, `par("lwd")`, `par("lty")` since 5.5 ([#15](https://github.com/emmanuelparadis/ape/pull/15))
- **Vertical segment colour in a phylogram** [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L590-L601)] [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L654-L692)]:
  - **Same style on all child edges**: the vertical segment takes that style
  - **Binary node with two styles**: the vertical segment is split at the node into two halves, each in the style of its child edge. Observed: a cherry with orange and purple child edges gets an orange lower half and a purple upper half
  - **Polytomy with mixed styles**: the vertical segment falls back to the default foreground (black, observed)
- **`node.color`, `node.width`, `node.lty`** (added in 2021, [#16](https://github.com/emmanuelparadis/ape/pull/16)): style the vertical segment of each node on its own
  - **Use**: for example, mark collapsed polytomies with dashed lines
  - **Only node styles given**: edges take the style of their child node
  - **Types**: phylogram and tidy only
- **Fan arcs**: the arc style depends on the styles of the child edges [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L737-L756)]
  - **First and last child edges agree**: the arc takes the style of its first child edge
  - **Binary node with two styles**: the arc is meant to be split in two halves (see "Defects")
  - **Other cases**: the arc gets the default style
- **Selecting edges by clade**: `which.edge(phy, tips)` returns the edge indices of the clade spanned by the given tips, for use as an index into the style vectors (vignette, "Connecting Data to the Branches")

### Root edge

- **`root.edge = TRUE`** draws `phy$root.edge` as a stub before the root, in all four directions and in fan trees [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L82-L84)] [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L426-L444)]
  - **Skipped**: for unrooted and radial trees, without branch lengths, or when the tree has no root edge
- **Style**: the stub uses the edge style only when that style is a single value, otherwise the default

### Long edges

- **`plotBreakLongEdges(phy, n = 1)`**: shortens the `n` longest edges to the length of the longest remaining edge and marks each with a white dot and a `//` label [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phyloExtra.R#L10-L17)]
  - **Axis**: the axis then no longer matches the drawing for those edges

## Labels

### Tip labels

- **Drawing**: tip labels are drawn with `text()`, by default in italics (`font = 3`), at the tip plus `label.offset` (user units, default 0)
  - **Tip data**: `font`, `cex`, and `tip.color` are recycled per tip, so they carry tip data (vignette, "Connecting Data to the Tips")
- **`underscore = FALSE`** (default): underscores in tip labels are drawn as spaces. Expressions (plotmath labels) are never changed [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L445-L447)]
- **Mixed fonts**: `mixedFontLabel()` builds plotmath expressions so that, for example, a genus is italic and a voucher number plain
- **`def(x, ..., default)`**: builds a style vector from label names, for example `def(tip.label, Homo = "red", default = "blue")`, to colour named tips without indexing by hand

### Label space

- **Horizontal space for labels**: the labels keep their size in inches while the tree scales, so the x limit is solved iteratively [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L137-L155)]:
  - **Rule**: the limit grows until every label fits. The conversion factor is `alp = x_i / (W - s_i)`, where `W` is the plot width in inches and `s_i` the label width in inches
  - **Very long labels**: if any label is wider than the plot, the limit becomes `1.5 * max(x)`, so one third of the width goes to labels
  - **Observed**: a 40-character label widened the x limit from 2.09 to 4.67 user units
- **Circular and unrooted label space**: a crude margin based on character counts instead of measured widths [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L279-L294)]
  - **Fan and unrooted**: `nchar * 0.018 * max(y) * cex`
  - **Radial**: `nchar * 0.03 * cex`
  - **Alignment**: the man page warns that `align.tip.label` with a fan tree needs manual `x.lim` and `y.lim`
- **No label culling**: every label is drawn, and labels of dense trees overlap (observed in a 40-tip `cophyloplot()`)
  - **Workaround**: label size is fixed in inches, so the user must change `cex` or the device size

### Orientation and alignment

- **`lab4ut`** (labels for unrooted trees): two placement modes [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L469-L525)]
  - **`"horizontal"`** (default for unrooted): shifts each horizontal label by an amount that depends on the angle of its terminal edge
  - **`"axial"`** (default for fan and radial): rotates each label along its terminal edge and flips labels on the left half by 180 degrees so they stay readable
- **`align.tip.label`**: `TRUE` or a line type number. Tips are joined to a common label column by dotted lines (lty 3 by default) [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L70-L80)] [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L450-L461)]
  - **Fan trees**: the labels move to the largest radius
  - **Turned off** for ultrametric trees, unrooted and radial types, and plots without branch lengths
- **`adj` and `srt`**: label justification (default 0, or 1 for leftwards trees) and rotation in degrees. Vertical trees turn the labels by a further 90 degrees

### Node labels

- **`show.node.label = TRUE`**: node labels are drawn at the node plus `label.offset` in x, with the tip font and size, unrotated in every layout [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L528-L530)]
  - **Observed**: node labels overlap edges and tip labels, for example the hybrid node label over tip `X` in the TreeKnit ARG

## Annotation

The annotation functions draw on top of the last plot. Without arguments they print the node, tip, or edge numbers, which is the usual way to find indices for later calls (vignette, "Tips, Nodes, and Edges").

### Node, tip, and edge labels

- **`nodelabels()`, `tiplabels()`, `edgelabels()`**: one shared engine `BOTHlabels()` with these modes [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/nodelabels.R#L47-L155)]:
  - **Text**: framed by a rectangle (default), a circle, or nothing (`frame = "none"`). Frame background defaults: light blue for nodes, yellow for tips, light green for edges
  - **Symbols**: `pch` 1 to 25 with `col` and `bg`
  - **Pie charts** (`pie`): one row of proportions per item, slices coloured by `piecol` (default `rainbow()`)
    - **Radius**: `cex * (x range) / 50`
    - **Origin**: the pie code is adapted from `plotrix::floating.pie()` with a correction that keeps pies round when the axes have different scales [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/nodelabels.R#L16-L45)]
  - **Thermometers** (`thermo`): stacked bars, vertical by default or horizontal (`horiz = TRUE`)
    - **Size**: 1/40 by 1/15 of the plot range by default
    - **Single vector**: a vector `p` becomes the two-column matrix `p, 1 - p`
  - **`adj`**: shifts text by the string size, but shifts pies, thermometers, and symbols by `adj - 0.5` user units, which is a different unit
- **Edge label position** [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/nodelabels.R#L223-L241)]:
  - **Phylogram**: the middle of the horizontal segment
  - **Fan**: the middle of the radial segment
  - **Other types**: the midpoint between the parent and child nodes
  - **`date`**: places edge labels at the x position `max(xx) - date`, which marks an event at a given age on its edge
- **`tiplabels(offset = )`**: moves tip annotations outward in the plot direction, or radially in fan and radial trees. The offset is ignored with a warning for unrooted trees [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/nodelabels.R#L181-L200)]
- **`edges(nodes0, nodes1)`**: draws extra segments between any two nodes [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/nodelabels.R#L250-L335)]
  - **Arrows**: optional, of type `"classical"`, `"triangle"`, or `"harpoon"` (`fancyarrows()`). The arrow angle is corrected for different x and y scales
  - **Use**: `plot.evonet()` uses it for reticulations

### Support values

- **`drawSupportOnEdges(value)`**: draws node-ordered support values on the edge above each node and drops the root value. It accepts `Nnode` or `Nnode - 1` values [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phyloExtra.R#L19-L30)]
- **Rerooting**: see "Scientific semantics"

### Scales and time axes

- **`add.scale.bar()`**: a bar of length `pretty(range / 6, 1)[2]` by default [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/scales.R#L13-L73)]
  - **Position**: the corner where the root is (rightwards: x = 0, y = 1)
  - **`ask = TRUE`**: places it with a click
- **`axisPhylo(side, root.time, backward = TRUE)`**: a time axis under or beside the tree [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/scales.R#L75-L126)]:
  - **Backward (default)**: the tip farthest from the root is time 0 and values grow toward the root. For non-ultrametric trees "present" is therefore the youngest tip
  - **Forward**: the root is time 0
  - **`root.time`**: a value for the root (taken from `phy$root.time` when present). Its meaning changes with `backward` [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/scales.R#L113-L119)]
    - **Observed forward** with `root.time = 2020` on a tree of height 4: the axis reads 2020 at the root and 2024 at the farthest tip (a date)
    - **Observed backward** on the same tree: the axis reads 2020 at the root and 2016 at the farthest tip (an age before present)
  - **Root edge**: the axis extends over a drawn root edge since 2024 ([#116](https://github.com/emmanuelparadis/ape/issues/116), [`570cae6`](https://github.com/emmanuelparadis/ape/commit/570cae6))
  - **Side**: chosen from the direction since 5.8-1 (side 1 for horizontal trees, side 2 for vertical trees)
  - **Fan trees**: a radial axis drawn along the bisector of the widest gap between tips, with tick labels `r0 - r`, so `open.angle` is needed to make room [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/scales.R#L127-L161)]
  - **Refused types**: unrooted (with a pointer to `add.scale.bar()`) and radial
- **`plotTreeTime(phy, tip.dates)`**: draws the tree in its own branch length units and a separate date axis under it [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phyloExtra.R#L32-L64)]
  - **Date lines**: each tip is joined to its sampling date on the axis by a line coloured from blue (oldest) to red (newest), with 50% transparency
  - **Missing dates**: those tips get no line
  - **Observed**: this is a tanglegram between a tree and a time line, which shows root-to-tip temporal signal at a glance

### Data beside the tree

- **`phydataplot(x, phy, style)`**: draws data aligned with the tips to the right of a rightwards tree. Rows are matched to tips by names [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/phydataplot.R#L82-L111)]:
  - **Styles**: `"bars"`, `"segments"`, `"image"` (a matrix such as distances, as a heat map), `"arrows"`, `"boxplot"`, `"dotchart"`, and `"mosaic"` (categorical or binned continuous data, with a legend)
  - **Placement**: the data start at `offset` past the farthest tip. Room must be left with `x.lim`
  - **Directions**: only rightwards trees, and only segments and arrows for circular trees
- **`ring(x, phy, style)`**: the circular counterpart, with rings, radial segments, or arrows outside a fan tree. Several calls with growing `offset` stack rings [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/phydataplot.R#L28-L80)]

## Two trees and tanglegrams

### `cophyloplot()`

`cophyloplot(x, y, assoc, use.edge.length = FALSE, space = 0, length.line = 1, gap = 2, type = "phylogram", rotate = FALSE, col, lwd, lty, show.tip.label = TRUE, font = 3, ..., node.depth = 2)` was added in version 2.2 together with `subtrees()` and `subtreeplot()` ([NEWS](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/NEWS#L2850-L2853)) [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/cophyloplot.R#L11-L52)].

#### Input and coordinates

- **Association matrix**: two columns of tip labels, one row per link [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/cophyloplot.R#L22-L25)]
  - **Flexibility**: a tip may have several links, the trees may have different tips, and rows need no order
  - **Without `assoc`**: the function prints "No association matrix specified. Links will be omitted." and draws the trees
- **Coordinates**: both trees come from `plotPhyloCoor()`, the left tree rightwards and the right tree leftwards [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/cophyloplot.R#L71-L89)]:
  - **Height**: both trees are shifted so that their lowest tip is at y = 0. Then the right tree is **scaled to the height of the left tree**
    - **Effect**: two trees with different tip counts get different row spacings
    - **Observed**: a 20-tip tree beside a 40-tip tree is stretched to the full height
  - **Width**: the right tree is scaled to the width of the left tree, so the two depth scales are not comparable when the trees have different heights
  - **Gap between the trees**: `space` is raised to at least `left + right + 2 * gap`, where `left` and `right` are the **character counts** of the longest labels plus `length.line` [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/cophyloplot.R#L62-L66)]
    - **Units**: character counts are added to tree coordinates (node depths or branch lengths), so the result depends on the units of the tree
    - **Tuning**: the man page tells users to tune `gap`, `length.line`, and `space` by hand
- **Types**: only `"phylogram"` and `"cladogram"`. Observed: `type = "fan"` fails with "object 'xx' not found"
- **Node placement**: see "Defects" for the effect of `node.depth`

#### Links

- **Link geometry** [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/cophyloplot.R#L137-L185)]:
  - **Stubs**: a horizontal stub runs from the end of each tip label (measured with `strwidth()`) plus `gap` to a fixed column at `gap + left` past the tip
  - **Connector**: one straight segment joins the two stub ends
  - **Observed with the minimum gap**: both stub columns coincide, so each connector is vertical and links look like right-angle steps. With `use.edge.length = TRUE` the tips end at different depths, the stubs start at different places, and connectors become slanted
  - **`length.line = 0`**: no stubs, only connectors
- **Link style**: `col`, `lwd`, and `lty` are per-link vectors, recycled
  - **MCC colouring**: this is the hook for colouring links by MCC (observed: links coloured red and blue by MCC)

#### Tree drawing and labels

- **Tree styling**: none [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/cophyloplot.R#L93-L101)]
  - **Edges**: drawn in the default colour with no `edge.color` argument, and `...` is not passed on
  - **Cladogram mode**: the left tree is always drawn red
- **Edge drawing**: the phylogram mode draws two segments per child (one vertical from the first child's row, one horizontal), so the vertical segments overlap. Observed: 32 black SVG paths for two 5-tip trees [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/cophyloplot.R#L102-L130)]
- **Labels**: italic by default (`font = 3`), with a fixed size [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/cophyloplot.R#L131-L136)]
  - **Size arguments**: the call for the left tree passes `cex = 0` and the right tree `cex = 1`
  - **Observed**: both render at the default size

#### Interaction and return value

- **Interactive rotation** (`rotate = TRUE`): the plot is redrawn in a loop [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/cophyloplot.R#L26-L44)]
  - **Click**: `identify()` waits for a click near any node of either tree, and the clicked internal node of the left or right tree is rotated with `rotate()`
  - **End**: a right click ends the loop
  - **Result**: the rotated trees are returned
- **Return value** (since 5.8-3): an invisible object of class `"cophylo"` with `trees` (a `multiPhylo` of the possibly rotated trees) and `assoc` [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/cophyloplot.R#L17-L21)]
  - **Replot**: passing this object back to `cophyloplot()` replots it (observed)
  - **phytools**: the same object works with `phytools::plot.cophylo()` ([#147](https://github.com/emmanuelparadis/ape/issues/147), [#148](https://github.com/emmanuelparadis/ape/pull/148))

### Untangling helpers

- **`rotate(phy, node, polytom = c(1, 2))`**: swaps two child clades of a node [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/rotate.R#L10-L49)]
  - **Polytomy**: `polytom` picks the two children, counted from bottom to top
  - **Two tips**: `node` may also be two tips, and their MRCA is used
- **`rotateConstr(phy, constraint)`**: reorders all children so that the tips follow a target order as closely as the topology allows [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/reorder.phylo.R#L75-L106)]
  - **Ranking**: each child is ranked by the **smallest** target position among its tips. A code comment records that `min()` worked better than `median()`, `mean()`, and `sum()`
  - **Tanglegram recipe** (man page of `rotate`): take the tip order of the first tree as the constraint for the second tree, then call `cophyloplot()`
  - **One-sided**: the first tree stays fixed, and nothing measures or minimizes crossings
  - **Observed**: on `((A,(B,C)),((D,E),(F,(G,H))));` with the constraint `H` to `A` the call reverses every child order, which is the best possible match
- **`ladderize(phy, right = TRUE)`**: sorts children by tip count [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/ladderize.R#L35)]
  - **Observed**: with `right = TRUE` the larger clade comes first in edge order and is drawn below the smaller one, and `right = FALSE` gives the reverse
  - **History**: the recursive version overflowed the C stack on almost caterpillar trees and was rewritten without recursion ([#54](https://github.com/emmanuelparadis/ape/issues/54), [#55](https://github.com/emmanuelparadis/ape/pull/55))

### Other displays of several trees

#### Comparison of two trees

- **`comparePhylo(x, y, plot = TRUE)`**: compares two trees and prints differences in tips, node counts, rooting, ultrametricity, clades, and splits [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/comparePhylo.R#L10-L162)]:
  - **Rooted trees**: the two trees are drawn side by side, with a filled circle on each node whose clade is missing from the other tree (blue on the first, red on the second)
    - **Clade matching**: clades are matched by MD5 hashes of their tip sets (`makeNodeLabel(, "md5sum")`)
  - **Same tip set**: both trees are rooted on the first tip and drawn unrooted [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/comparePhylo.R#L127-L157)]
    - **Highlighted splits**: splits present in both trees (or, with `commons = FALSE`, splits specific to each tree) are drawn with width 5
    - **Colour**: shared splits and other edges are both black, so width is the only cue
  - **Common clades**: tables of branching times (`$BT`, ultrametric trees) and node labels (`$NODES`) of clades shared by both trees
  - **Observed** on the TreeKnit pair: "3 clades in ha not in na", "1 split in common"
    - **One-page device**: when both trees are rooted, the clade figure is drawn first and then replaced by the split figure

#### Several trees in panels

- **`plot.multiPhylo(x, layout = 1)`**: draws trees one after the other in a grid of `layout` panels, asking before each new page on interactive devices [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L849-L857)]
- **`kronoviz(x, layout = length(x), horiz = TRUE)`**: stacks dated trees with a shared time scale and aligned tips [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L902-L950)]
  - **Panels**: each panel has height proportional to its tip count, and `axisPhylo()` is drawn on the chosen sides
  - **Directions**: all four directions are supported since 5.8-1
  - **Observed** with two coalescent trees: tips aligned at 0, panel heights 5:8

#### Exploration of one tree

- **`zoom(phy, focus)`**: draws the whole tree with the focus clades coloured (unlabelled) and each focus clade as an enlarged subtree in its own panel [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/zoom.R#L10-L37)]
- **`trex(phy)`**: tree exploration with two devices. A click on a node of the main tree draws that clade on a second device with a coloured background [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L859-L900)]
- **`subtreeplot(x)`**: the same idea on a split screen

#### Custom links between panels

- **Connecting separate plots**: the vignette shows how to draw across several panels [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/vignettes/DrawingPhylogenies.Rnw#L915-L1012)]
  - **Conversion**: node coordinates of each panel are converted into device inches with `par("usr")`, `par("pin")`, and `par("mai")`
  - **Drawing**: lines are drawn across panels after `par(new = TRUE)`
  - **Use**: this is the general way to draw custom links between two independently styled trees

## Networks and reticulations

### Data model

- **`evonet`**: a `phylo` tree (the base tree) plus `reticulation`, a two-column integer matrix of extra edges coded like `edge` (parent, child) [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/evonet.R#L10-L47)]
  - **History**: the class was added in version 2.7-2 and the extended Newick reader and writer in 5.0 ([NEWS](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/NEWS#L766-L768))
- **Constructor**: `evonet(phy, from, to)` checks only that the node numbers are in range. It does not check that the network stays acyclic or time-consistent
- **No lengths on reticulations**: the object has no field for reticulation edge lengths. `Nedge()` counts tree edges plus reticulations [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/evonet.R#L277)]
- **Optional `df_edge`**: a data frame of hybrid edge data, filled only for Rich Newick input (see below)
- **Conversions** [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/evonet.R#L49-L109)]:
  - **`as.phylo()`**: drops reticulations
  - **`as.networx()`**: phangorn split networks, refused when a tip is part of a reticulation
  - **`as.network()` and `as.igraph()`**: reticulations become ordinary edges

### Reading extended Newick

`read.evonet()` calls `read.tree(evonet = TRUE)` and then `as.evonet()` [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/evonet.R#L237-L241)] [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/read.nexus.R#L55-L67)].

- **Parsing as a tree first**: the string is parsed as a normal tree, so every occurrence of a hybrid node becomes a separate node. `as.evonet.phylo()` then finds the tip labels that contain `#` [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/evonet.R#L134-L203)]:
  - **Leaf occurrences**: each tip with `#` becomes a reticulation from its parent to the internal node whose label is **exactly equal** to the tip label, and the tip is removed [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/evonet.R#L160)]
  - **The occurrence with children** stays in the tree, so its parent is the primary parent and its edge length is kept. The order in the string does not matter (observed: a leaf occurrence before the internal one reads correctly)
  - **Hybrid type**: the text after `#` is never interpreted. `#H1`, `#LGT1`, and `#R1` are just parts of a label (observed: `#LGT1` reads as a reticulation)
  - **More than two parents**: one reticulation row per extra parent (observed: three occurrences give two rows)
  - **Parents of leaf occurrences become singletons**: when a leaf occurrence is removed, its parent may keep one child. Such singleton nodes stay in the base tree and are drawn as points on a line (observed)
- **Lost data**:
  - **Reticulation edge lengths**: the lengths of the removed leaf occurrences are dropped [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/evonet.R#L199)]
  - **Annotations**: all `[...]` comments are deleted before parsing, as in `read.tree()` (see "Input formats")
- **Rich Newick** (since 5.8-3, [#148](https://github.com/emmanuelparadis/ape/pull/148)): extra colon fields of hybrid tokens go into `df_edge` [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/evonet.R#L214-L235)]
  - **Condition**: every hybrid token has the same number of colon fields and at least three (`name:length:support:probability`)
  - **Columns**: `parent`, `node`, `edge.length`, `support`, and `probability`
  - **Observed**: `#H1:1::0.3` and `#H1:1::0.7` give inheritance probabilities 0.3 and 0.7 per parent
- **Failures**: see "Defects"

### Writing extended Newick

- **`write.evonet()`**: adds one tip per reticulation, labelled like the hybrid node (or `#H<node>` when the tree has no node labels), and writes the result with `write.tree()` [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/evonet.R#L243-L275)]
- **Reticulation lengths are recomputed**: the length of each written leaf occurrence is `depth(child) - depth(parent)`, computed from the base tree
  - **Accuracy**: the value is exact only when the network is time-consistent
  - **Observed**: the TreeKnit ARG round trip gives the same structure and the same length 0.5
- **Console output is lost**: with the default `file = ""` the function returns the converted tree object instead of the string (see "Defects")

### Drawing

- **`plot.evonet(x, col = "blue", lty = 1, lwd = 1, alpha = 0.5, arrows = 0, arrow.type = "classical", node.depth = 2, ...)`**: calls `plot.phylo()` on the base tree with any layout, then `edges()` for the reticulations [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/evonet.R#L56-L67)]
- **Straight segments**: each reticulation is one straight line from the position of the extra parent to the position of the hybrid node, in blue with 50% opacity by default. Observed:
  - in the TreeKnit ARG the line runs diagonally from `ARGNode_8` across the edge to `C` to the hybrid node `ARGNode_10#H1`
  - without branch lengths (`node.depth = 2`) both ends have the same depth, and the line is vertical
  - in a fan tree the line cuts across the middle of the circle
- **Arrows**: `arrows = 1`, `2`, or `3` (start, end, both), with classical, triangle, or harpoon heads
- **No network layout**: the base tree layout ignores reticulations. No row is reserved, no crossing is avoided, and lines can pass over labels
- **Style per reticulation**: `col`, `lty`, and `lwd` are recycled over the reticulation rows
  - **Documentation conflict**: the man page says these values are passed to `plot.phylo()`, but the code passes them to `edges()` only
- **Default `node.depth = 2`** (since 2025, [`891a4f7`](https://github.com/emmanuelparadis/ape/commit/891a4f7)): chosen so that singleton nodes look better without branch lengths
  - **Side effect**: the same default breaks `type = "unrooted"` (see "Defects")

## Tree manipulation for display

- **`root(phy, outgroup, node, resolve.root = FALSE, interactive = FALSE, edgelabel = FALSE)`**: places the root at a node, never on an edge midpoint
  - **`resolve.root = TRUE`**: adds a zero-length edge so that the outgroup and the ingroup are separate clades
  - **Rationale**: the maintainer explains that the root position along the outgroup edge is unknown, and that viewers which show a non-zero split there place it artificially ([#143](https://github.com/emmanuelparadis/ape/issues/143))
- **`collapse.singles(phy, root.edge = FALSE)`**: removes nodes with one child and adds their lengths together
  - **Test**: `has.singles()` tests for them
  - **Drawing**: `plot.phylo()` draws singletons since 5.0, but `cophyloplot()` refuses them
- **`di2multi(phy, tol = 1e-08)`**: collapses internal edges shorter than `tol` into polytomies [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/multi2di.R#L130-L135)]
  - **Inverse**: `multi2di()` resolves polytomies randomly with zero-length edges
- **`identify.phylo()`**: returns the node, tip, or clade nearest to a click. Coordinates are rescaled by `pin / max(xx)` and `pin / max(yy)` before the distance test [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/identify.phylo.R#L10-L44)]
- **`ltt.plot()`, `ltt.lines()`, `mltt.plot()`, `ltt.coplot()`**: lineages-through-time plots [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/ltt.plot.R#L10-L66)]
  - **Counting**: singletons are collapsed first and polytomies are counted by their degree
  - **Non-ultrametric trees**: resolved with `multi2di()`, and tips older than the present count as extinctions ([#3](https://github.com/emmanuelparadis/ape/issues/3), fixed 2021)
  - **`ltt.coplot()`**: draws the tree above its LTT plot on the same time scale

## Input formats

### Newick (`read.tree`)

#### Parser

- **Pipeline**: the R function prepares the text in steps, then passes each string to the C parser [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/read.tree.R#L10-L141)]:
  1. join all lines
  2. replace quoted labels with placeholders
  3. split trees at `;`
  4. delete comments
  5. remove leading and trailing underscores
  6. delete spaces and tabs
- **C parser**: one pass records the positions of `(`, `,`, and `)`, a second pass builds the edge matrix [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/src/tree_build.c#L133-L163)]
  - **Checks**: the parser checks that the parentheses balance and that the first `(` matches the last `)` ([#142](https://github.com/emmanuelparadis/ape/issues/142), 2025)
- **Several trees**: any number of trees per file or per line becomes a `multiPhylo` (observed: `(A,B);(C,D);` gives two trees). Text before the first `(` becomes the tree name
- **No semicolon**: a warning "no semicolon(s) [end(s) of tree] found" and `NULL` ([#114](https://github.com/emmanuelparadis/ape/issues/114))
- **Limits** [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/src/tree_build.c#L11-L13)]:
  - **Depth**: at most 100,000 nested internal edges (raised from 10,000 after a crash, [#70](https://github.com/emmanuelparadis/ape/issues/70))
  - **Labels**: at most 512 bytes
  - **Branch length tokens**: at most 100 bytes
  - **Checks**: the label and token limits are not checked (see "Defects")

#### Labels and annotations

- **Comments and annotations are deleted**: every `[...]` block is removed with the regex `\[[^]]*\]`, so BEAST `[&...]` annotations, FigTree colours, and NHX tags are lost [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/read.tree.R#L64-L83)]
  - **Observed**: `((A[&x=1]:1,B:1)[&posterior=0.9]90:1,C[&!color=#ff0000]:2);` gives the tips `A B C`, the node label `90`, and no annotation
  - **Annotated trees**: ape relies on other packages (treeio, phytools) for them
- **Spaces**: deleted outside quotes since an early version. Observed: `(A ,B C:1);` gives `A` and `BC` ([NEWS](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/NEWS#L4285-L4289))
- **Quoted labels**: single quotes are kept as part of the label (observed: `'A B'` stays `'A B'` with its quotes). Doubled quotes fail (see "Defects")
- **`#` in labels**: kept as text by `read.tree()` (observed: `Sample#3`). Only `read.evonet()` gives it meaning

#### Branch lengths and root

- **Missing branch lengths**: a token without `:` gets `NaN`. Trees without any `:` are built without `edge.length` [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/src/tree_build.c#L34-L47)]
- **Number syntax**: lengths are read with `R_strtod()`, so `0x1A` gives 26 and `Inf` is accepted (observed)
- **Root edge and root label**: text after the last `)` becomes the root label and the root edge [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/src/tree_build.c#L244-L262)]

### NEXUS (`read.nexus`)

- **Comments removed line by line**, including `[&R]` and `[&U]`, so the rooting flag is ignored. Observed: a `[&U]` tree with a binary root reads as rooted [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/read.nexus.R#L150-L179)]
- **TRANSLATE**: since 5.8-3, tokens are translated for tips and internal nodes ([#133](https://github.com/emmanuelparadis/ape/issues/133)) [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/read.nexus.R#L126-L145)]
  - **Unmatched tokens**: a token that matches no entry is tried as a taxon number, as the NEXUS standard requires
- **Only the first TREES block**: the block is located by the first `BEGIN TREES;` and the next `END;`. A taxon named "Translate" triggers a warning [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/read.nexus.R#L180-L201)]
- **Compressed tip labels**: a `multiPhylo` whose trees share one tip set stores the labels once (`TipLabel` attribute)

### Validation before drawing

- **Badly formed trees**: since 5.8-3 `plot.phylo()` stops with "tree badly conformed. Check the edge matrix." when node numbers are out of range or missing ([#133](https://github.com/emmanuelparadis/ape/issues/133)) [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L43-L51)]
  - **Before 5.8-3**: such trees crashed R
- **Fewer than two tips**: a warning and no plot [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L24-L28)]

## Export

- **Any R graphics device**: ape draws through base graphics, so `pdf()`, `svg()`, `png()`, `postscript()`, and so on all work. There is no export function of its own
- **SVG content** (observed with the Cairo `svg()` device): every edge, stub, and link is a separate `<path>` with an RGB stroke, and text is converted to glyph outlines
  - **No structure**: there are no groups, ids, or classes, so the SVG cannot be restyled or searched by label afterwards
- **Size**: the label space depends on the device size at plot time. The man page warns that a manually resized window needs a replot
- **Tree files**: ape writes four tree formats
  - **`write.tree()`**: Newick with `digits = 10` significant digits. Labels are not quoted: spaces become `_`, and `,`, `:`, `;`, `(`, `)` become `-` [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/write.tree.R#L10-L25)]
  - **`write.nexus()`**: NEXUS with a TRANSLATE block
  - **`write.evonet()` and `write.phyloXML()`**: extended Newick and PhyloXML

## Configuration surface

All options of `plot.phylo()` with their defaults [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L12-L22)]:

- **Shape**: `type = "phylogram"`, `use.edge.length = TRUE`, `node.pos = NULL` (1 or 2, see "Coordinate model"), `node.depth = 1`, `direction = "rightwards"`, `rotate.tree = 0`, `open.angle = 0`, `root.edge = FALSE`
- **Limits and margins**: `x.lim = NULL`, `y.lim = NULL` (one value sets the upper limit only), `no.margin = FALSE` (margins set to 0 and not restored)
  - **Refused**: `xlim`, `ylim`, and `asp` cannot be passed
- **Labels**: `show.tip.label = TRUE`, `show.node.label = FALSE`, `font = 3`, `cex = par("cex")`, `adj = NULL`, `srt = 0`, `label.offset = 0`, `underscore = FALSE`, `lab4ut = NULL`, `tip.color = par("col")`, `align.tip.label = FALSE`
- **Edges and nodes**: `edge.color`, `edge.width`, `edge.lty`, `node.color`, `node.width`, `node.lty`, all `NULL` (graphical parameter defaults)
- **Other**: `plot = TRUE`, and `...` passed to `plot.default()` (for example `main`)
- **Return value**: the same list as `last_plot.phylo` without the coordinates, invisibly

## Scientific semantics

### Branch lengths and time

- **All or nothing**: one `NA` (or `NaN`) length makes `plot.phylo()` ignore all lengths with the warning "1 branch length(s) NA(s): branch lengths ignored in the plot" (observed) [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L58-L68)]
  - **Before ape 4.0**: such trees failed to plot ([NEWS](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/NEWS#L972-L973))
- **Negative lengths are drawn backwards** (observed): in `((A:1,B:-0.5):1,C:2);` tip B is drawn at x = 0.5, left of its parent at 1, with no warning
  - **Tidy type**: the only type that refuses them
- **Ages**: `axisPhylo()` measures time backward from the tip farthest from the root
  - **Serially sampled trees**: "present" is the youngest sample unless `root.time` is set
  - **`branching.times()`**: assumes an ultrametric tree
- **Date annotations are not used**: positions come only from branch lengths. `plotTreeTime()` relates tips to dates visually but does not position the tree by date

### Zero-length edges and polytomies

- **No collapsing**: a zero-length edge is drawn as a point, and its child sits on top of its parent
  - **Observed** with `((A:0,B:1):1,(C:1,D:1):0);`: A at the x of its parent, and the C-D node on top of the root
  - **Effect**: this shows the stored structure but hides zero-length tips and sampled ancestors visually
- **Interpretation left to the user**: the maintainer notes that zero-length branches "may have different interpretations in different contexts" and points to `di2multi()` for treating them as polytomies ([#83](https://github.com/emmanuelparadis/ape/issues/83))
  - **`is.binary()`**: ignores branch lengths
- **Polytomies**: drawn natively in all layouts. The node sits at the mean of its children (phylogram) or of its tips (`node.pos = 2`)
- **Singleton nodes**: drawn by `plot.phylo()` as invisible points on a straight line, and refused by `cophyloplot()` and `plotPhyloCoor()`

### Node labels, support, and rerooting

- **Node labels belong to nodes by default**: `root()` keeps each label on its node. For support values, which describe the edge above a node, this is wrong after a reroot ([Czech et al. 2017](https://doi.org/10.1093/molbev/msx055))
- **`root(edgelabel = TRUE)`** (since 4.0): labels on the edges whose direction flips move to the other end of their edge, and the new root gets an empty label [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/root.R#L308-L312)]
  - **Observed** on `((A:1,B:1)95:1,(C:1,(D:1,E:1)80:1)70:1,F:1);` rooted on A
  - **With the option**: 95 moves to the clade `(C,D,E,F)`, which describes the same split `{A,B} | {C,D,E,F}`
  - **Without the option**: 95 stays on the node that now groups B with the rest
- **Display on edges**: `drawSupportOnEdges()` pairs with `edgelabel = TRUE` (vignette, "Function drawSupportOnEdges")
- **Duplicate labels**: two checks are weak
  - **Writing**: `write.tree()` can write duplicated internal labels without a warning ([#113](https://github.com/emmanuelparadis/ape/issues/113))
  - **Length check**: the length of `node.label` is checked only since 2024 ([#115](https://github.com/emmanuelparadis/ape/pull/115))

### Rooting

- **Every Newick tree is rooted at the stored root**: `is.rooted()` returns `TRUE` when the tree has a root edge or the root has at most two children [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/root.R#L12-L16)]
  - **Unrooted trees**: ape marks an unrooted tree by a basal polytomy
  - **NEXUS**: `[&U]` is ignored
- **Rooted layouts on unrooted trees**: the vignette says that phylogram, cladogram, and fan are valid only for rooted trees, and that unrooted trees should use `type = "unrooted"`

### Networks

- **Primary parent**: the parent of the occurrence with children. It defines the base tree and therefore the whole layout. Which parent is primary is a choice of the file author
- **Time consistency is not checked**: reticulations are drawn between whatever positions the base tree gives, and `write.evonet()` recomputes their lengths from those positions

## Defects

Each entry gives the input, the effect, and the evidence. Observed entries were reproduced on commit `c73e48d` (version 5.8-3) in a container.

### Parsing

- **Labels longer than 512 bytes crash R** (observed): `read.tree(text = "(<600 x 'a'>:1,B:1);")` aborts R with "stack smashing detected" [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/src/tree_build.c#L26-L32)] [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/src/tree_build.c#L179)]
  - **Cause**: the C parser copies labels into fixed 512-byte stack buffers without a bounds check
  - **513 bytes**: a 513-byte label reads without a visible error, which is also an overflow
  - **Branch length tokens**: they use a 100-byte buffer in the same way (derived)
- **Doubled single quotes** (observed): `('O''Brien':1,...)` gives the tip label `NA` with the warning "NAs introduced by coercion" [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/read.tree.R#L116-L120)]
  - **Cause**: the quote handler treats `''` as the end of one quoted label and the start of another, and the placeholder can then not be decoded
- **Quotes stay in labels** (observed): `'A B'` is stored with its quotes, so a label quoted in one file and unquoted in another does not match
- **Rooting flags ignored** (observed): `[&U]` and `[&R]` are deleted as comments

### Networks

- **Tree without reticulations** (observed): `read.evonet(text = "((A:1,B:1):1,C:2);")` fails with "vector size cannot be infinite". With no `#` tip, `edge[-ind, ]` removes every edge [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/evonet.R#L146-L149)]
- **All occurrences are leaves** (observed): `((A:1,#H1:1):1,(#H1:1,C:2):1);` fails with "NAs are not allowed in subscripted assignments" instead of a message
  - **Standard**: Cardona et al. allow this form for a hybrid leaf without children
- **Occurrence labels must be identical** (observed): `X#H1` and `Y#H1` fail with the same message. The hybrid identity is the whole label, not the `#H1` part
- **`write.evonet()` to the console returns nothing useful** (observed): with `file = ""`, `write.tree()` returns the string, but `write.evonet()` discards it and returns the converted tree [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/evonet.R#L270-L275)]
- **`read.evonet(...)` passes `...` to `read.tree()`**, which lost its `...` argument in 5.8-3, so any extra argument fails as unused (derived) [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/evonet.R#L237-L241)] [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/read.tree.R#L10-L11)]
- **Discarded reorder** (derived): `as.evonet.phylo()` calls `reorder(x)` without using the result [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/evonet.R#L200-L202)]

### Layouts

- **`node.depth = 2` breaks unrooted trees** (observed): wedges are allocated by level instead of tip count, so they do not add up to the parent wedge [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L224-L229)] [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/vignettes/DrawingPhylogenies.Rnw#L168-L171)]
  - **Observed**: on an 8-tip tree, tips A and H land on the same point and the drawing looks like a cycle
  - **Networks**: `plot.evonet()` uses `node.depth = 2` by default, so `plot(net, type = "unrooted")` is affected
  - **Documentation conflict**: the vignette states that only `use.edge.length` affects unrooted layouts
- **`node.depth = 2` moves the root of radial trees** (observed): the level is divided by the tip count, so the root lands at radius 0.5 on an 8-tip tree instead of the centre [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L233-L236)]
- **Fan arc colours for two different child styles** (observed): split arcs show the wrong colours [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L737-L768)]
  - **Segment count**: the split arc reuses a fixed vector of 50 plus 50 colours, but since [#129](https://github.com/emmanuelparadis/ape/pull/129) an arc has `2 + |Δθ| / 0.03` points. A short arc has fewer than 50 segments and takes only the first child's colour
  - **Degree test**: it indexes nodes by node number while arcs are in postorder, so the wrong node's degree is checked
  - **Observed** on six red and blue cherries: five arcs are fully red and one is black
- **Tidy plots with `axisPhylo()`** (observed): the tidy type is missing from the phylogram branch of `axisPhylo()`, so the fan code draws a diagonal radial axis across the tree [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/scales.R#L87)]
- **Tidy plots with `edgelabels()`** (observed): edge labels sit at the midpoint between parent and child, off the drawn segments, because only `"phylogram"` gets the segment midpoint [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/nodelabels.R#L223)]
  - **Other functions**: `tiplabels(offset = )` and `add.scale.bar()` also skip the tidy type (derived)

### Two trees

#### Layout and drawing

- **`node.depth` in `cophyloplot()` does not do what the man page says** (observed): the value is passed to `plotPhyloCoor()` as `node.pos`, which sets the vertical placement (2 = mean of tips) [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/cophyloplot.R#L71-L75)] [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plotPhyloCoor.R#L59-L64)] [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/man/cophyloplot.Rd#L40)]
  - **Horizontal depth**: always the tip count, because `plotPhyloCoor()` calls the C routine with method 1 hard-coded
  - **Observed**: `node.pos = 1` and `2` give identical x positions, while `plot.phylo(node.depth = 2)` gives evenly spaced levels
  - **Default**: the man page also calls 1 the default although the default is 2
- **Left tree drawn red in cladogram mode** (observed), a hard-coded `col = "red"` [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/cophyloplot.R#L97)]
- **Units mixed** (derived): character counts are added to tree coordinates when the gap is computed (see "`cophyloplot()`")

#### Input errors

- **Unknown label in `assoc`** (observed): a row naming a tip that does not exist fails with "replacement has length zero" instead of naming the label
- **No `assoc`** (observed): a dummy one-row matrix of `NA` is used, which prints the message and two warnings about replacement lengths
- **Singleton nodes** (observed): refused with "there are single (non-splitting) nodes in your tree" [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plotPhyloCoor.R#L18-L19)]
- **Other types** (observed): `type = "fan"` fails with "object 'xx' not found" instead of an argument check

### Other

- **Vignette and code disagree**: the vignette says `axisPhylo()` passes `...` to `text()`, but the code passes it to `axis()` [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/vignettes/DrawingPhylogenies.Rnw#L484-L487)] [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/scales.R#L126)]
- **`comparePhylo()` split figure** (derived): shared splits and other edges are both black, so only the width distinguishes them [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/comparePhylo.R#L129)]
- **`identify.phylo()` rescaling** (derived): dividing by `max()` instead of the range distorts the distance test for leftwards, downwards, and circular plots whose coordinates do not start at 0

## Performance

Observed timings, single R process, PNG device 1000 by 1000 pixels, tip labels off, random trees from `rtree()`:

- **Parsing**: `read.tree()` takes 0.02 s for 10,000 tips and 0.23 s for 100,000 tips
- **Single-tree layouts**: drawing time grows faster than the tip count
  - **Phylogram**: 0.21 s for 1,000 tips, 2.27 s for 10,000 tips, and **67 s for 100,000 tips** (30 times longer for 10 times more tips). The growth comes from `phylogram.plot()`, which finds the child edges of each node with `which(e1 == j)` over all edges [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/plot.phylo.R#L566-L569)]
  - **Fan and unrooted**: 0.43 s and 1.88 s for 10,000 tips. The unrooted recursion also scans all edges per node
  - **Tidy**: 4.8 s for 1,000 tips and 19.6 s for 5,000 tips (contour merging in R)
- **`cophyloplot()`**: 0.38 s for two 500-tip trees, 2.7 s for two 2,000-tip trees. It also subsets the edge matrix once per node and child
- **SVG size**: 0.62 MB for 1,000 tips and 6.2 MB for 10,000 tips, about 600 bytes per tip with labels off
- **History**: two changes improved speed and robustness
  - **Fan arcs**: arcs with fewer segments made large fan plots faster ([#129](https://github.com/emmanuelparadis/ape/pull/129))
  - **`ladderize()`**: the recursive version was replaced to avoid C stack overflows ([#55](https://github.com/emmanuelparadis/ape/pull/55))

## Interaction with TreeKnit output

Trial inputs were the expected outputs of the two-tree CLI test in `packages/treeknit-cli/tests/data/outputs/two/expected/`.

### Two-tree outputs

- **Resolved trees** (`ha_resolved.nwk`, `na_resolved.nwk`): Newick with internal labels `NODE_1` and so on and all branch lengths, except the root
  - **Observed**: they read and plot without warnings, and `show.node.label = TRUE` shows the node names
- **Pair tanglegram coloured by MCC** (observed): about five lines of R give a TreeKnit tanglegram:
  1. read `MCCs.dat` (one MCC per line, comma-separated) with `strsplit(readLines(f), ",")`
  2. build `assoc <- cbind(labels, labels)` and a colour per label from its MCC
  3. call `cophyloplot(ha, na, assoc = assoc, col = colours)`
  - **Result**: links of the MCC `X` drawn red and the others blue
  - **Tree edges**: the trees themselves stay black, because `cophyloplot()` has no edge styling
  - **Coloured edges**: colouring the edges of each MCC needs two `plot.phylo()` calls with `edge.color` (one leftwards) and links drawn by hand
- **`MCCs.json`**: ape has no JSON reader. The `jsonlite` package would read it
- **Auspice JSON and SVG figures**: ape has no reader for either

### ARG

The ARG file `ARG/arg.nwk` is written by `fn extended_newick()` in [packages/treeknit-io/src/arg.rs#L14-L66](../../packages/treeknit-io/src/arg.rs#L14-L66). It reads with `read.evonet()` (observed).

- **Structure**: 5 tips, 6 internal nodes, and 1 reticulation from `ARGNode_8` to `ARGNode_10#H1`
  - **Label form**: TreeKnit writes the occurrence with children first and repeats the full label `ARGNode_10#H1` on the leaf occurrence, which is exactly the form that ape's exact label match needs
- **Segment annotations**: every `[&segments={0,1}]` block is deleted, so the segments carried by each edge are lost
  - **Reticulation length**: the reticulation edge length (0.5) is dropped
- **Drawing**: `plot()` draws the base tree through the primary parent `ARGNode_4` and one blue diagonal line from `ARGNode_8`
  - **Overlap**: the hybrid node label overlaps the tip label `X`
- **Zero-length hybrid edge** (observed): a variant where the hybrid node has length 0 to its primary parent reads and draws correctly, because ape never collapses zero-length edges
  - **TreeKnit**: such edges can occur in TreeKnit ARGs, where reassortment edges get `min(b1, b2) / 2` (`fn set_branch_lengths()` in [packages/treeknit-core/src/arg.rs#L492-L539](../../packages/treeknit-core/src/arg.rs#L492-L539))
- **Unshared roots**: TreeKnit then writes `GlobalRoot[&segments={0,1}]:0.0` (derived from the writer), which ape reads as a root label with a root edge of 0
- **Failure case**: an ARG without reassortment would fail in `read.evonet()` ("vector size cannot be infinite"), so a TreeKnit run with a single MCC must be read with `read.tree()` (derived from the observed failure)

## Ideas from the issue tracker and history

### Open requests

- **Improvements to ape** ([#147](https://github.com/emmanuelparadis/ape/issues/147), open): a contributor's list that led to 5.8-3:
  - `node.depth = 2` as the default for `comparePhylo()`, `plot.evonet()`, and `cophyloplot()`, because it "results in nicer plots"
  - Rich Newick support for networks, in coordination with the Bioconductor package `tanggle` for network plotting
  - a shared `cophylo` object so that ape's `cophyloplot()` (interactive rotation) and phytools' `cophylo()` (automatic crossing reduction) can draw the same pair
  - `assert_phylo()` and `clean_phylo()` helpers to validate and repair trees before plotting
- **Zero-leaf trees** ([#94](https://github.com/emmanuelparadis/ape/issues/94), open): wider support for empty trees

### Declined

- **Translation tables with gaps** ([#133](https://github.com/emmanuelparadis/ape/issues/133)): first declined as documented behavior, then implemented in 5.8-3 after the reporter quoted the NEXUS standard
- **Lenient Newick without semicolon** ([#114](https://github.com/emmanuelparadis/ape/issues/114)): kept strict, because the semicolon separates trees in multi-tree files. A warning was added instead
- **`lab4ut` for `tiplabels()`** ([#137](https://github.com/emmanuelparadis/ape/issues/137)): answered with `plot(..., tip.color = )` and `def()` instead of a new option

### Features found only in the history

- **Radial trees without branch lengths**: the radial type has ignored lengths since it was added ([NEWS](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/NEWS#L3883-L3884))
- **Propagation of sister styles to the vertical segment** ([NEWS](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/NEWS#L3886-L3888)): the old rule that the vertical segment takes the style of identical sister edges, later extended with half-segments and node styles
- **`plot.ancestral()` and `evolve.phylo()`**: removed in version 2.5-2 ([NEWS](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/NEWS#L2350))
- **Fixed cophyloplot defaults**: before 5.8-3 the vertical placement in `cophyloplot()` followed the type (`node.pos = 1` for phylograms)
  - **Change**: commit [`827754b`](https://github.com/emmanuelparadis/ape/commit/827754b) (2025) fixed it at 2 through the `node.depth` argument
- **`.treeBuildWithTokens()`**: a faster C parser for NEXUS trees with numeric tokens, deleted in 2025 when TRANSLATE handling was rewritten (comment in [`src/tree_build.c`](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/src/tree_build.c#L165-L171), [#133](https://github.com/emmanuelparadis/ape/issues/133))

## Open scientific problems

- **Crossing minimization**: `rotateConstr()` sorts by the smallest target position, a heuristic chosen by trial
  - **Complexity**: tanglegram layout with minimal crossings is NP-hard for two free trees
  - **ape**: offers no exact or iterative method
- **Primary parent in networks**: the base tree, and therefore the whole drawing, depends on which parent the file author made primary
- **Reticulation lengths and times**: ape keeps no length for reticulation edges and recomputes them from the base tree on output, which assumes a time-consistent network
- **Node versus edge meaning of labels**: ape lets the user choose at reroot time (`edgelabel`), but the tree object does not record the choice
  - **Effect**: other functions (`drawSupportOnEdges()`, `nodelabels()`) must be told again
- **Zero-length edges**: ape draws them literally and leaves the interpretation (polytomy, sampled ancestor, real zero) to the user through `di2multi()`

## Not covered by ape

- **Two trees and networks**:
  - **Automatic untangling**: no crossing count and no optimization for two trees
  - **Edge styling in tanglegrams**: `cophyloplot()` cannot colour tree edges or scale labels
  - **Network layout**: reticulations are overlaid on a tree layout as straight lines, with no reserved rows, curves, or dashes by default
- **Data and layouts**:
  - **Annotations**: `[&...]` blocks are discarded by all readers, so colouring by node or edge attributes needs other packages
  - **Equal-daylight unrooted layout**: only equal-angle
  - **Legends**: none are drawn automatically (base `legend()` must be called)
- **Interaction and output**:
  - **Interaction**: no zoom, pan, hover, or search. Clicks work only through `locator()` and `identify()`
  - **Label thinning**: no hiding of overlapping labels
  - **Structured SVG**: no groups, ids, or text elements in vector output

## Method and limits

- **Code read**: the sources below were read
  - **Code and documentation**: the plotting, network, annotation, parser, and writer files listed under "Shape of the code", the C coordinate and parser routines, the man pages of the functions covered, the vignette `DrawingPhylogenies.Rnw`, and NEWS
  - **Excluded**: foreign agent configuration files were not read, and no text in the repository or the issues tried to give instructions
- **History**: the public repository starts in 2020 (284 commits)
  - **Older history**: comes from NEWS only, so features removed before 2020 are known only from their NEWS entries
- **Issues**: all 89 issues and 66 pull requests were listed with `gh`
  - **Read with comments**: the plotting, parsing, and network ones (#3, #15, #16, #19, #21, #34, #49, #54, #70, #80, #83, #85, #94, #95, #114, #116, #119, #129, #133, #137, #142, #143, #145, #147, #148)
  - **Before 2020**: GitHub issues exist only since 2020. Older discussions happened on the r-sig-phylo mailing list, which was not searched
- **Trial**: the surveyed commit was built from source with `R CMD INSTALL` in a throwaway `rocker/r-ver` container (R 4.6.1, network off during runs)
  - **CRAN version**: CRAN had version 5.8-1, which differs from the commit in `read.nexus()` and in the check for badly formed trees
  - **Outputs**: PNG files inspected visually, Cairo SVG files inspected as text, and the coordinate vectors of `last_plot.phylo` printed as numbers
  - **Not run**: the interactive modes (`cophyloplot(rotate = TRUE)`, `trex()`, `identify.phylo()`), because the container has no display
  - **Timings**: one run each on a shared 20-core machine, indicative only
- **Book**: the ape book ([Paradis 2012](https://doi.org/10.1007/978-1-4614-1743-9)) is not freely available and was not read
  - **Coverage**: its plotting chapter predates the tidy type, node styles, Rich Newick, and the `cophylo` class, and the vignette in the repository covers the same plotting model
- **Not verified**: `write.phyloXML()`, `phydataplot()` styles other than those described from code, and `mltt.plot()` were not run
