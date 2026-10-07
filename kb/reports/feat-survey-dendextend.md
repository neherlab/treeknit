# dendextend feature survey: tree rendering, semantics, and edge cases

This report describes dendextend, an R package that adjusts, plots, and compares dendrograms (R objects of class `dendrogram`), as an idea inventory for the web app. Its tanglegram, its untangling heuristics, and its tree comparison measures are the main subjects, because they address the same display problem as two segment trees side by side. The report records what dendextend does, how it does it, which conventions it assumes, which inputs break it, and which problems its maintainer left open. It does not compare dendextend with `packages/web`.

dendextend is licensed GPL-2 or GPL-3 (the user chooses). Its behavior and design may be studied, but copying its code needs approval (see the project rules).

- **Source**: [talgalili/dendextend](https://github.com/talgalili/dendextend) at commit `0af62aa` (2025-07-15, version 1.19.1), 1222 commits since 2013-04-05. Source links point to this commit
- **Other sources**:
  - the vignette `vignettes/dendextend.Rmd` and the function documentation in the source
  - the paper ([Galili 2015](https://doi.org/10.1093/bioinformatics/btv428))
  - the paper of the "stepBothSides" untangling method ([Nguyen et al. 2022](https://doi.org/10.1093/bioadv/vbac014))
  - all 113 issues and 49 pull requests by title, and 27 issues with their comments, `NEWS.md`, and the commit history of the tanglegram files
  - the conversion function of ape ([emmanuelparadis/ape](https://github.com/emmanuelparadis/ape) at commit `c73e48d`), because every phylogenetic tree enters dendextend through it
- **Evidence labels**: each claim carries one of two labels, and where the documentation and the code disagree, the code wins and the difference is noted
  - **Observed**: seen in a trial run of dendextend 1.19.1 with ape 5.8.1 and R 4.6.1 (see "Method and limits")
  - **Derived**: read from the code without running it
- **Shape of the code**: plain R on top of base graphics
  - **Node model**: a `dendrogram` is a nested list in which each node carries the attributes `height`, `members`, `midpoint`, `label`, and the optional style lists `nodePar` and `edgePar`. Almost every function walks this list recursively
  - **Key files**:
    - [`R/tanglegram.R`](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R): the tanglegram and the mirrored dendrogram plot
    - [`R/untangle.R`](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/untangle.R) and [`R/entanglement.R`](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/entanglement.R): the layout heuristics and their cost function
    - [`R/distinct_edges.R`](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/distinct_edges.R) and [`R/common_subtrees.R`](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/common_subtrees.R): the clade comparisons that drive the tanglegram highlights
    - [`R/set.dendrogram.R`](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/set.dendrogram.R), [`R/color_branches.R`](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/color_branches.R), and [`R/branches_attr_by.R`](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/branches_attr_by.R): the styling API

## Summary

### Tanglegram and TreeKnit

- **Tanglegram**: a three-panel base-graphics figure. The left tree faces right, the right tree faces left, and straight lines join leaves with the same label. Three highlights are on by default (see "Tanglegram"):
  - **Distinct edges dotted**: clades that exist in only one tree are drawn with line type 3
  - **Common subtrees coloured**: leaves of maximal subtrees with the same topology in both trees get one colour per subtree for their connecting lines. All other lines are grey
  - **Branch width by height**: branch width grows linearly from 1 at the leaves to 10 at the root
- **Common subtrees match TreeKnit's naive MCCs**: on the simulated fixture `sim_k2_n100_r0.02`, the 7 common subtrees that dendextend found are exactly the 7 `naive_mccs` of the fixture (observed)
- **Conversion failures**: phylogenetic trees enter only through ape's conversion, which accepts rooted, binary, ultrametric trees with branch lengths and doubles all heights (see "Conversion from phylogenetic trees")
  - **Tree shape**: polytomies and non-ultrametric (dated) trees fail with an error (observed)
  - **Missing lengths**: trees without branch lengths fail with an error (observed)
  - **ARG**: the TreeKnit ARG fails with an error (observed)
- **Different leaf sets**: the tanglegram prunes both trees to the shared leaves, with a warning that is off by default

### Untangling and comparison

- **Untangling**: seven methods behind `untangle()` minimise one cost, `entanglement`, the normalised L-norm of the vertical displacement of the connecting lines (see "Entanglement")
  - **Greedy rotation by cluster level** (`step1side`, `step2side`): the best general method. Observed at 100 leaves: entanglement from 0.63 to 0.08 in 11 seconds
  - **Exhaustive pair search** (`stepBothSides`, added in 2024): 140 seconds at 50 leaves with no gain over `step2side` (observed)
  - **DendSer seriation**: rotates each tree for itself and ignores the other tree, so it does not untangle (observed at 50 and 100 leaves)
- **Comparison measures**: `cor.dendlist` makes a correlation matrix for many trees from one of these measures
  - **Clade-based**: a rooted Robinson-Foulds distance and a share of common clades
  - **Correlations**: cophenetic correlation, and Baker's gamma (computed as a Spearman correlation, although the documentation calls it Goodman-Kruskal gamma)
  - **Per cut level**: the Fowlkes-Mallows index with an asymptotic rejection line (`Bk_plot`)
- **Scope**: dendextend is a toolkit for hierarchical clustering

### Defects

- **Defects found**: five defects affect the tanglegram and the untangling (see "Defects")
  - **Tanglegram**: a right-title size option is ignored, one colouring mode mixes grey and black lines, and per-line widths are silently dropped
  - **Untangling**: entanglement exceeds 1 for L below 1, and the list version of `untangle` lacks one method

## Data model

- **Node attributes**: a node of a `dendrogram` has three numeric attributes
  - **`height`**: distance from the leaf level, never a branch length
  - **`members`**: leaf count
  - **`midpoint`**: horizontal offset of the node from its leftmost leaf
  - **Leaves**: a leaf also has `label` and `leaf = TRUE`, and its integer value is its "order value", the index of the observation in the original data
- **Branch is the edge above a node**: the style of a branch lives in the `edgePar` list of the node below it (`col`, `lty`, `lwd`)
  - **Points and labels**: their style lives in `nodePar` (`pch`, `cex`, `col`, `bg`, `lab.col`, `lab.cex`, `lab.font`)
  - **Root branch**: the root has a branch that is not drawn by default ([Galili 2015](https://doi.org/10.1093/bioinformatics/btv428))
- **Depth-first value recycling**: a vector value for all nodes is applied in depth-first order (root first), and it is recycled when it is shorter than the node count. A value of `Inf` means "leave this node unchanged" [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/attr_access.R#L1080-L1115)]
- **Two orders**: the order value of a leaf and its position on screen are different things [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/entanglement.R#L65-L92)]
  - **Matching by order value**: many functions match the two trees by order value for speed
  - **`match_order_by_labels`**: first copies the order values of one tree onto the leaves with the same labels in the other tree
- **`dendlist`**: a list of dendrograms. `plot()` of a `dendlist` with two or more trees draws a tanglegram of the trees selected by `which` (default the first two) [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/dendlist.R#L241-L250)]

## Conversion from phylogenetic trees

- **One path through hclust**: `as.dendrogram()` of an ape `phylo` object calls `ape::as.hclust.phylo()` and converts the result [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/ape.R#L24-L27)]
- **Three hard requirements**: ape stops with an error unless the tree is ultrametric, binary, and rooted [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/as.phylo.R#L93-L130)]. Observed messages:
  - non-ultrametric `((A:1,B:2):1,C:2);`: "the tree is not ultrametric"
  - polytomy `((A:1,B:1,C:1):1,D:2);`: "the tree is not binary"
  - no lengths `((A,B),(C,D));`: "the tree has no branch lengths"
- **Heights are doubled**: ape sets the hclust height to twice the branching time (`height = 2*bt`), because a cophenetic distance between two leaves is twice their divergence time [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/as.phylo.R#L126)]
  - **Observed**: a tree of root age 4 has a dendrogram height of 8, and the tanglegram axis runs from 0 to 8
- **Zero-length branches pass**: both cases draw and take part in tanglegrams (observed)
  - **Tip branch**: a zero-length tip branch gives a leaf at height 0 under a parent at height 0
  - **Internal branch**: a zero-length internal branch gives a child at the same height as its parent
- **Issue reports about conversion**:
  - **Users force trees into the format**: users run `ape::chronos()` to make trees ultrametric before they call `tanglegram` ([#36](https://github.com/talgalili/dendextend/issues/36), [#92](https://github.com/talgalili/dendextend/issues/92))
    - In #92, chronograms with zero-length branches from collapsed clades produced a dendrogram with 0 members and the error "argument is of length zero"
    - A commenter explains that hierarchical clustering never yields zero branch lengths
  - **The way back**: `as.phylo()` of a dendrogram goes through `as.hclust` and `ape::as.phylo.hclust` and therefore forces an ultrametric tree [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/ape.R#L94-L97)]. A request to keep non-ultrametric trees is open ([#162](https://github.com/talgalili/dendextend/issues/162))
  - **A direct converter was requested** in 2015 ([#15](https://github.com/talgalili/dendextend/issues/15)). The issue was closed after an ape update, and the hclust path stayed

## Tanglegram

`tanglegram()` accepts two dendrograms, two `hclust` objects, two `phylo` objects, or one `dendlist`. `dendbackback` is another name for it [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L746-L793)] [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L1086-L1088)]. The code comes from a Stack Overflow answer by Johan Renaudie, which in turn follows `dueling.dendrograms` of the sharpshootR package [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L656-L662)].

### Arguments and defaults

All defaults are in the signature [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L798-L839)]. The function returns an invisible `dendlist` of the two trees after all modifications, so the user can reuse the styled and pruned trees [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L1077)].

#### Lines and highlights

- **Lines between leaves**:
  - `color_lines`: one colour per line, in the order of the left tree's leaves from the bottom up. A short vector is recycled. When missing, the colour comes from the common subtrees (see below) or is `"darkgrey"`
  - `lwd = 3.5`: width of the connecting lines
  - `common_subtrees_color_lines = TRUE` and `common_subtrees_color_lines_default_single_leaf_color = "grey"`
- **Branches**:
  - `edge.lwd = NULL`: a uniform branch width. When set, it turns `highlight_branches_lwd` off
  - `highlight_distinct_edges = TRUE`, `common_subtrees_color_branches = FALSE`, `highlight_branches_col = FALSE`, `highlight_branches_lwd = TRUE`
  - `k_branches`, `k_labels`: colour branches or labels by k clusters of each tree
  - `type = "r"`: rectangular elbows, or `"t"` for straight diagonal lines
- **Speed**: `faster = FALSE`. When `TRUE`, it turns off the distinct edges, both common-subtree options, and the width highlight [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L840-L845)]

#### Tree preparation

- **Leaf sets**:
  - `intersecting = TRUE`: prunes both trees to the shared labels
  - `match_order_by_labels = TRUE`: aligns the order values by label, which the documentation calls a must for correct lines
- **Tree shape**:
  - `sort = FALSE`: sort the children of both trees by label before drawing
  - `rank_branches = FALSE`: replace heights by topological ranks, so that each internal node is 1 above its highest child [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/attr_access.R#L679-L707)]
  - `hang = FALSE`: place each leaf at 10% of the root height below its parent [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/attr_access.R#L580-L612)]

#### Layout and labels

- **Layout**:
  - `columns_width = c(5, 3, 5)`: relative widths of the left tree, the line panel, and the right tree, passed to `layout()`
  - `margin_top = 3`, `margin_bottom = 2.5`, `margin_inner = 3`, `margin_outer = 0.5`: margins in lines of text. `margin_inner` is the only space for the labels
  - `left_dendo_mar`, `right_dendo_mar`: full `mar` vectors built from the four margins
  - `just_one = TRUE`: when `FALSE`, the function does not call `layout()`, so several tanglegrams can share one page (added in [#30](https://github.com/talgalili/dendextend/pull/30))
- **Labels and text**:
  - `lab.cex = NULL`: label size for both trees
  - `dLeaf = NULL`, `dLeaf_left`, `dLeaf_right`: distance between a leaf tip and its label in user coordinates. When `dLeaf` is set and both sides are equal, the right side takes the negative value, so the result is symmetric [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L993-L996)]
  - `remove_nodePar = FALSE`: remove leaf points
  - `main` and `sub` above and below the line panel, `main_left` and `main_right` above the trees, `cex_main = 2` with `cex_main_left`, `cex_main_right`, and `cex_sub` that default to it
- **Axes**: `axes = TRUE` draws a height axis under each tree

### Processing order

The function changes both trees in a fixed order before it draws anything [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L858-L988)]:

1. Convert to dendrograms, remove leaf points, sort by labels
2. Prune to the shared labels
3. Apply `lab.cex`, `edge.lwd`, `k_labels`, `k_branches`, ranked heights, and hanging leaves
4. Mark distinct edges with `lty = 3`, but only on a tree that has no `lty` yet
5. Colour common subtrees on the branches (optional), then choose the line colours
6. Apply the colour and width highlights by height, but only where the tree has no `col` or `lwd` yet

- **Existing styles win**: since version 1.4.0 the automatic highlights skip any tree that already carries the attribute ([`aef480e`](https://github.com/talgalili/dendextend/commit/aef480e))
  - **Reset**: `set("clear_branches")` removes the attributes
  - **Effect**: a tree coloured with `color_branches` keeps its colours, but its line types and widths still change

### Geometry

#### Trees and height scales

- **Independent height scales**: each tree fills its own panel from its own root height to 0
  - **Observed**: a tree of height 4 next to a tree of height 16 gives axes from 0 to 4 and from 0 to 16 of the same width
  - **Effect**: two trees with different root ages are stretched differently, and equal x positions do not mean equal times
- **Left tree**: the standard `plot(dend1, horiz = TRUE)` with the root at the left and the labels at the right. The x axis runs from the root height down to 0 with `xaxs = "i"`, so the root touches the panel border
- **Right tree**: `plot_horiz.dendrogram(side = TRUE)` reverses `xlim`, so the root is at the right and the labels at the left [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L359-L516)]
  - **Copied drawing code**: base R has no option for this, so the function copies the node drawing of `plot.dendrogram` and computes the label offset from the width of the longest label
- **No root edge**: neither tree draws an edge above the root unless the root has `edgetext`. With an edge, its length is `0.0625` times the root height [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L404-L410)]

#### Panels and connecting lines

- **Three panels**: `layout(matrix(1:3, nrow = 1), widths = columns_width)` places the left tree, the line panel, and the right tree in one row [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L1010-L1011)]
- **Leaf rows**: all three panels share `ylim = c(0, l)`, where `l` is the leaf count, so leaf i of either tree sits at y = i. The first leaf is at the bottom [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L1016-L1067)]
- **Connecting lines**: one straight segment per shared label from (0, y_left) to (1, y_right) in the middle panel, drawn with `arrows(code = 0)` [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L1006-L1050)]
  - **Rows**: they come from `ord_arrow`, a two-column matrix of the left and right positions of each label
- **Drawing order**: left tree, then the line panel with `main` and `sub`, then the right tree

#### Label space and resizing

- **Labels have a fixed space**: the label space is `margin_inner` lines
  - **Long labels are cut** at the edge of the tree's figure region (observed with a 30-character label, which showed as "A_very" on the left and "er_one" on the right)
  - **User fixes**: users found the fix by trial ([#82](https://github.com/talgalili/dendextend/issues/82), [#118](https://github.com/talgalili/dendextend/issues/118))
  - **No hide option**: there is no option to hide labels ([#48](https://github.com/talgalili/dendextend/issues/48))
- **No redraw on resize**: the documentation warns that the figure "does not resize well" and must be drawn again after a window resize [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L650-L652)]

### Highlight: distinct edges

- **Definition**: a node is distinct when its leaf set, as a sorted set of labels, does not occur as the leaf set of any node of the other tree. Leaves and the root always match when the leaf sets are equal [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/distinct_edges.R#L69-L137)]
- **Display**: the branch above each distinct node gets `lty = 3` (dotted) in the tanglegram [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L904-L908)]
  - **Observed in the SVG output**: the dotted branches are dash patterns such as `3.25,9.76`, scaled with the branch width
- **Documentation differs**: the argument documentation still says line type 2 (dashed) [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L615)]
  - **Change**: the value changed to 3 in 2017, "so that the gap from lty are not too wide" ([`01d4a47`](https://github.com/talgalili/dendextend/commit/01d4a47))
  - **Paper**: the paper also says "dashed"
- **History**: the option was turned off by default in 2015 because base R crashed when it drew a branch with both a line type and a character colour. Later it was turned on again (`NEWS.md`, version 1.0.0)
- **Cost**: the documentation warns that it "can be slow on large trees". `distinct_edges` compares every leaf set of one tree with all leaf sets of the other, which is quadratic in the node count

### Highlight: common subtrees

- **Definition**: a node of the left tree is "good" when it and all its descendants are not distinct. Each maximal good node defines a cluster of leaves. Leaves outside such a node are singletons [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/common_subtrees.R#L30-L62)] [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/common_subtrees.R#L168-L210)]
- **Meaning for TreeKnit**: a cluster is a clade whose whole internal topology is the same in both trees. This is the "naive MCC" of TreeKnit for binary trees
  - **Observed on `sim_k2_n100_r0.02`**: cluster sizes 28, 27, 12, 10, 9, 9, and 5, all equal to the `naive_mccs` of the fixture
  - **Limit**: MCCs that are not clades in both trees cannot be found this way
- **Line colours** (default): one hue per cluster from `colorspace::rainbow_hcl(n, c = 90, l = 50)`, and grey for singletons [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L959-L979)] [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/rainbow_fun.R#L21-L26)]
  - **Hue count includes singletons**: `n` is the number of distinct clusters with each singleton counted as its own cluster, so many singletons spread the real clusters over fewer distinct hues
  - **Without colorspace** the function falls back to `rainbow()`. colorspace is only suggested, which once made `tanglegram` fail ([#24](https://github.com/talgalili/dendextend/issues/24), open)
- **Branch colours** (`common_subtrees_color_branches = TRUE`): the left tree is coloured by `color_branches(clusters = ...)`, and the right tree takes the colour of the same labels [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L911-L956)]
  - **Upper branches**: branches above the clusters stay black
  - **Line colours**: the line colours then follow the branch colours (observed: bands of colour from the left tree through the lines into the right tree)
- **Pruning to common subtrees**: `prune_common_subtrees.dendlist` removes every singleton leaf from both trees, which leaves only the shared clusters [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/prune.R#L233-L245)]

### Highlight: branch width and colour by height

- **Width** (default on): `highlight_branches_lwd` maps each node height linearly onto 1000 steps between `lwd = 1` (lowest node, normally the leaves) and `lwd = 10` (root) [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/highlight_branches.R#L20-L35)] [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/highlight_branches.R#L59-L74)]
  - **Observed widths in the SVG**: 0.75 to 4.97 px for an 8-leaf tree
  - **Effect**: the figure looks heavy near the root, and deep splits draw the eye
- **Colour** (default off): `highlight_branches_col` maps the height onto `rev(viridis(1000, end = 0.9))`, so low branches are yellow and high branches are dark purple [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/highlight_branches.R#L37-L54)]
- **Branch is coloured by its lower node**: both functions take the height of the node below the branch, so a long branch above a low node is drawn thin

## Untangling

### Entanglement

`entanglement(dend1, dend2, L = 1.5)` measures the quality of a tanglegram layout [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/entanglement.R#L359-L395)].

- **Definition**:
  - **Formula**: number the left leaves 1 to n from the bottom. For the leaf at position i of the right tree, let p(i) be the position of the same label in the left tree. Then `E = sum_i |i - p(i)|^L / sum_i |i - (n + 1 - i)|^L`. The denominator is the value for a right tree in exactly reversed order
  - **Meaning of L**: L above 1 punishes steep lines more than many slightly sloped lines. L near 0 counts the lines that are not horizontal. `L = 0` is replaced by `1e-50`, because `0^0` would count straight lines too
  - **Observed values** for two 8-leaf trees that differ by two swaps of cherries (2 of 4 lines cross per half): 0.5 for L = 0, 0.2627 for L = 0.5, 0.125 for L = 1, 0.0557 for L = 1.5, and 0.0238 for L = 2
- **Inputs**:
  - **Label matching**: `leaves_matching_method = "labels"` (default) matches by label, and `"order"` matches by order value. The documentation calls `"order"` faster but unsafe, because wrong order values give wrong numbers without an error
  - **Leaf sets must be equal**: different leaf sets stop with "labels do not match in both trees" (observed). `tanglegram` prunes, but `entanglement` does not
- **Limits**:
  - **Crossings are ignored**: two layouts with the same displacement but different numbers of line crossings get the same value. The literature on the tanglegram layout problem usually minimises crossings
  - **Documentation error**: the documented return value is "The number of leaves in the tree" [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/entanglement.R#L265)]

### Rotation primitives

#### Whole-tree rotation

- **`rotate(dend, order)`**: `order` is a vector of labels in the wanted order, or a numeric vector of positions [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/rotate.R#L133-L160)]
  - **Mechanism**: the function gives each leaf a weight and calls `stats::reorder(x, weights, mean)`, which sorts the children of every node by the mean weight of their leaves
  - **Best effort**: an order that the topology does not allow gives the nearest order the mean rule finds, with no warning. Observed: asking for `A C B D E F G H` on `((A,B),(C,D)),...` returned the tree unchanged
  - **Scalar order**: `rotate(dend, i)` gives weight 1 to leaf i and 0 to all others, so every node on the path from the root to leaf i moves the child that holds leaf i to the end. Observed: `rotate(d, 3)` on `ABCDEFGH` gave `EFGHABDC`. The `stepBothSides` method uses this move
  - **Name clash**: ape also exports `rotate()`. When ape is attached after dendextend, `rotate(dendrogram, 3)` fails with "object "phy" is not of class "phylo"" (observed)
- **`sort(dend, type = "labels")`**: rotates by alphabetical label order. `type = "nodes"` calls `ladderize` [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/rotate.R#L185-L191)]
- **`ladderize(dend, right = TRUE)`**: sorts the children of each node by leaf count, largest first, then reverses the whole tree when `right = FALSE` [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/rotate.R#L417-L445)]
- **`shuffle(dend)`**: a random rotation, `rotate(dend, sample(n))` [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/untangle.R#L216-L222)]

#### Local swaps and interaction

- **`flip_leaves(dend, leaves1, leaves2)`**: swaps two adjacent leaf bundles [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/untangle.R#L418-L443)]
  - **Mechanism**: the code joins order values into a string such as `zzz1zzz||zzz2zzz` and swaps substrings, which the comment calls possibly not "the smartest" way
- **`all_couple_rotations_at_k(dend, k)`**: cuts the tree into k - 1 and k clusters by height [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/untangle.R#L490-L571)]
  - **Result**: for each cluster of the coarser cut that splits into several clusters of the finer cut, it returns one tree per pair of sub-clusters swapped
  - **Binary trees**: this is the one node that the k-th cut opens
  - **No exact cut**: when no cut gives exactly k clusters, the tree comes back unchanged
- **`click_rotate(dend)`**: interactive rotation. The user clicks on the top branch of a cluster in the plot [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/rotate.R#L323-L354)]
  - **Effect**: the click height cuts the tree, and the cluster under the click is reversed
  - **Loop**: with `continue = TRUE`, clicking below the leaves ends the loop

### Methods

`untangle(dend1, dend2, method = ...)` dispatches to the methods below. The default is `"labels"` [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/untangle.R#L127-L140)]. Observed results start from the same shuffled 8-leaf pair (entanglement 0.2208) and, for the larger rows, from the TreeKnit simulated fixtures with lengths set by Grafen's method.

| Method          | Algorithm                                                | 8 leaves | 50 leaves      | 100 leaves     |
| --------------- | -------------------------------------------------------- | -------- | -------------- | -------------- |
| `labels`        | rotate the right tree toward the left tree's label order | 0.0951   | not run        | not run        |
| `ladderize`     | ladderize both trees                                     | 0.2208   | not run        | not run        |
| `random`        | R = 100 random shuffles of both trees, keep the best     | 0.1067   | 0.0822, 2.2 s  | 0.2849, 4.1 s  |
| `step1side`     | greedy pass over k = 2..n on the left tree               | 0.0951   | 0.0556, 1.0 s  | 0.2311, 5.5 s  |
| `step2side`     | alternate `step1side` on both trees until no change      | 0.0557   | 0.0323, 3.0 s  | 0.0840, 11.2 s |
| `stepBothSides` | `step2side`, then all pairs of path rotations            | 0.0557   | 0.0323, 140 s  | not run        |
| `DendSer`       | seriation of each tree by its own cophenetic distance    | 0.0557   | 0.3763, 0.03 s | 0.5839, 0.08 s |

Start values: 0.3618 at 50 leaves and 0.6297 at 100 leaves.

#### Greedy methods

- **`step1side`** (`untangle_step_rotate_1side`, `L = 1.5`): for each level k, it builds all rotations at level k and keeps the one with the lowest entanglement against the fixed tree [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/untangle.R#L650-L692)]
  - **Level order**: k runs over `2:n` (`direction = "forward"`) or `n:2` (`"backward"`), or over a custom `k_seq`. Nodes are visited from the root downward in height order
  - **Cached heights**: the heights per k are computed once, because the comment notes that this step "takes a looong time"
  - **Cost**: about n - 1 levels with about 2 candidates each, and each candidate costs one rotation and one entanglement of O(n) R work. A pass is therefore O(n²), with large constants from recursion over lists
  - **Polytomies**: a non-binary dendrogram cannot become an `hclust`, so cuts fall back to the slower dendextend `cutree`. Observed with `collapse_branch` polytomies at 50 leaves: entanglement from 0.10 to 0.02 in 30 seconds with more than 50 warnings
  - **Tied heights**: the hclust `cutree` breaks ties in an arbitrary way ([#21](https://github.com/talgalili/dendextend/issues/21)), so nodes at the same height still get visited. Observed: two trees with all cherries at the same height reached entanglement 0
- **`step2side`** (`untangle_step_rotate_2side`, `L = 1.5`, `max_n_iterations = 10`): it runs `step1side` on the left tree against the right, then on the right against the left [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/untangle.R#L777-L828)]
  - **Stop rule**: it repeats until neither tree changes or the iteration limit is reached
  - **Message**: it prints "We ran untangle N times" when warnings are on
- **`stepBothSides`** (`untangle_step_rotate_both_side`, added in 2024 by [#122](https://github.com/talgalili/dendextend/pull/122)): it implements the pseudo-code of [Nguyen et al. (2022)](https://doi.org/10.1093/bioadv/vbac014) [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/untangle.R#L923-L965)]
  - **Round**: each round runs `step2side` to convergence, then tries all `(n - 1)²` pairs `rotate(dend1, i)`, `rotate(dend2, j)` and keeps the best pair
  - **Stop rule**: rounds repeat until the entanglement stops changing, at most 9 times, because the loop stops when its counter reaches `max_n_iterations = 10`
  - **Cost**: `(n - 1)²` entanglement calls per round, each O(n), so O(n³) per round. Observed: 140 seconds at 50 leaves on one core
  - **Warnings**: the scalar `rotate()` call raises "number of items to replace is not a multiple of replacement length". The code suppresses it, but `step2side` inside does not, which matches the report in [#113](https://github.com/talgalili/dendextend/issues/113) (open, no reproducible example)

#### Other methods

- **`labels`**: `rotate(dend2, labels(dend1))`. It is cheap and good when the trees are similar [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/untangle.R#L117-L120)]
- **`random`** (`untangle_random_search`, `R = 100`, `L = 1`): it shuffles both trees R times and keeps the pair with the lowest entanglement [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/untangle.R#L323-L353)]
  - **No accumulation**: each shuffle starts from the input, so the search does not build on earlier improvements
- **`DendSer`** (`untangle_DendSer`): it calls `DendSer::DendSer` on each tree with that tree's own cophenetic distance as the weight [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/DendSer.R#L93-L97)] [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/DendSer.R#L140-L145)]
  - **Independent seriation**: the other tree is never used, so the method seriates both trees independently
  - **Silent fallback**: when DendSer fails or is missing, the tree comes back unchanged with no message. A test that expected entanglement 0 failed on Debian because DendSer was not installed ([#154](https://github.com/talgalili/dendextend/issues/154)). The fix skips the test
  - **Documentation**: the help page calls it "a good (and fast) starting point" for `step2side`, and the `TODO` file says `DendSer.dendrogram` does not add value

#### Usage advice

- **Recommended combination**: the examples combine a random start with a greedy pass and report entanglement 0 on small trees [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/untangle.R#L82-L101)]
  - **Example**: `untangle(method = "random", R = 5)` followed by `untangle(method = "step1side")`
- **Choice of L matters**: a small L in the search accepts a few long lines to keep many lines straight
  - **Observed on the 50-leaf pair**: `step2side` with L = 0.5 reached 0.0669 measured at L = 1.5, while L = 1.5 and L = 2 reached 0.0323

### Untangling and reassortment

- **Remaining crossings are signal**: observed on the 100-leaf fixture after `step2side`, the crossings that remain are bundles of parallel lines that move whole common subtrees between positions
  - **Meaning**: each bundle is a block of leaves whose placement differs between the trees, which is what a reassortment does to a clade
- **Untangling cannot see clusters**: entanglement counts displacement leaf by leaf. A layout that keeps each common subtree contiguous and in parallel is a natural goal for a reassortment display, but no method optimises for it

## Tree comparison measures

All measures assume equal leaf sets. Their help pages tell the user to call `intersect_trees` first.

### Clade-based measures

- **`dist.dendlist(dendlist)`**: the count of distinct edges in both directions, a Robinson-Foulds distance on rooted clusters [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/distinct_edges.R#L337-L415)]
  - **Difference from ape**: the help page states that results differ from ape and that the reason is an open issue ([#97](https://github.com/talgalili/dendextend/issues/97))
  - **Cause**: the issue compares with ape's unrooted distance, so rooted clusters and unrooted splits explain the difference
  - **Observed for one 8-leaf pair**: both gave 8
- **`cor_common_nodes(dend1, dend2)`**: `(N - D) / N`, where N is the node count of both trees and D the distinct-edge count. Leaves count as common nodes, so the value never reaches 0 [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/cor.dendlist.R#L141-L147)]
- **`dend_diff(dend1, dend2)`**: two trees side by side in a `par(mfrow = c(1, 2))` grid, with distinct edges drawn in colour 2 (red) and no lines between them [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/distinct_edges.R#L301-L317)]
- **`all.equal` for dendrograms**: compares topology and branch lengths of two or more trees, with `use.edge.length` and `use.topology` switches (vignette)

### Correlation measures

- **`cor_cophenetic(dend1, dend2, method_coef = "pearson")`**: the correlation of the two cophenetic distance matrices, where the cophenetic distance of two leaves is the height of their lowest common ancestor ([Sokal and Rohlf 1962](https://doi.org/10.2307/1217208))
  - **Label sort**: both matrices are sorted by label before the correlation, except when the first tree is an `hclust` [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/cor_cophenetic.R#L165-L184)]
  - **Heights**: the value depends on heights
  - **`dist` input**: a `dist` object can replace the second tree, which gives the classic cophenetic correlation of a clustering with its data
- **`cor_bakers_gamma(dend1, dend2)`**: for each pair of leaves, the largest k at which both are still in one cluster, in each tree, then the correlation of the two k vectors ([Baker 1974](https://doi.org/10.1080/01621459.1974.10482971)) [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/cor_bakers_gamma.R#L62-L101)]
  - **Computed as a Spearman correlation**: the code calls `cor(..., method = "spearman")`, but the documentation says the measure is "also known as Goodman-Kruskal-gamma index" [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/cor_bakers_gamma.R#L120-L121)]. The two coefficients treat ties differently, and these k vectors have many ties
  - **Identical vectors give 1**: an `NA` correlation is replaced by 1 with the comment that it occurs only for identical vectors. Any constant vector also gives `NA`, for example a tree that is a star
  - **Ignores heights**: only the order of merges counts, so the measure suits trees with different time scales
  - **Inference**: the vignette builds a null distribution by permuting labels and a bootstrap confidence interval with `sample.dendrogram`. A user reported cut warnings with this recipe ([#103](https://github.com/talgalili/dendextend/issues/103), open)
- **`cor.dendlist(dendlist, method)`**: a symmetric matrix of one measure for all pairs of trees [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/cor.dendlist.R#L75-L98)]
  - **Methods**: `"cophenetic"` (default), `"baker"`, `"common_nodes"`, and `"FM_index"`
  - **Display**: the vignette displays it with `corrplot(..., "pie", "lower")`

### Cut-level measures

- **`Bk(tree1, tree2, k)` and `Bk_plot`**: the Fowlkes-Mallows index `B_k = T_k / sqrt(P_k * Q_k)` for each cut level k = 2..n - 1, with its expectation and variance under the null hypothesis of random labels ([Fowlkes and Mallows 1983](https://doi.org/10.1080/01621459.1983.10478008)) [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/bk_method.R#L181-L247)] [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/bk_method.R#L567-L621)]
  - **Plot**: B_k against k as points and a line, the expectation as a dashed line, and a one-sided rejection line `E + qnorm(0.95) * sd` in red. Options add a permutation rejection line (`R = 1000`) and a Bonferroni correction [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/bk_method.R#L881-L996)]
  - **Ties**: a cut level that no height gives is `NA` for dendrograms. For `hclust` objects, R's `cutree` still returns a partition, which the vignette labels as a "WRONG Bk plot"
- **`cor_FM_index(dend1, dend2, k)`**: the Fowlkes-Mallows index for one k

### Speed

- **Observed speed**: Baker's gamma took 0.04 s at 50 leaves and 0.21 s at 100 leaves, because `cutree` goes through `hclust`. The documentation warns that it "can be quite slow" for larger trees

## Colour, line style, and labels

### The `set()` function

`set(dend, what, value)` is the central styling call. Calls chain with the pipe operator, and `set.dendlist` applies a change to all trees or to those in `which` [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/set.dendrogram.R#L338-L420)]. The `what` values:

- **Branch styles**:
  - **By cluster**: `branches_k_color`, `branches_k_lty`. The documentation of `branches_k_lty` says it "updates the lwd"
  - **Directly**: `branches_col`, `branches_lwd`, `branches_lty`, one value per node in depth-first order
  - **By labels**: `by_labels_branches_col`, `by_labels_branches_lwd`, `by_labels_branches_lty` with extra arguments `type` and `TF_values`
  - **By lists**: `by_lists_branches_col`, `by_lists_branches_lwd`, `by_lists_branches_lty`
  - **Highlights**: `highlight_branches_col`, `highlight_branches_lwd`
- **Labels and points**:
  - **Labels**: `labels`, `labels_colors`, `labels_cex`, `labels_to_character`
  - **Leaf points**: `leaves_pch`, `leaves_cex`, `leaves_col`, `leaves_bg`. A point needs `leaves_pch` first, for example 19
  - **Node points**: `nodes_pch`, `nodes_cex`, `nodes_col`, `nodes_bg`
- **Heights**: `hang_leaves`, `rank_branches`
- **Reset and options**:
  - **Reset**: `clear_branches`, `clear_leaves`
  - **`order_value = TRUE`**: reorders a value vector from data order to leaf order before it is applied

### Colouring by clusters

- **`color_branches(dend, k, h, col, groupLabels, clusters)`**: cuts the tree into k clusters or at height h, then colours each maximal subtree that lies in one cluster. Branches above the clusters keep their colour [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/color_branches.R#L274-L408)]
  - **Palette**: `rainbow_hcl(k, c = 90, l = 50)` by default. A short colour vector is recycled, and a long one is cut, both with a warning
  - **`groupLabels`**: `TRUE` writes the cluster number as `edgetext` on the top branch of each cluster, in a box with a border of the cluster colour. A function or a character vector gives custom texts. The numbers follow the leaf order, which differs from `cutree` numbering ([#65](https://github.com/talgalili/dendextend/issues/65), open)
  - **`clusters`**: a vector of cluster IDs in leaf order replaces the cut. This colours any partition of the leaves, for example a partition computed outside dendextend
  - **Duplicate labels**: they are made unique for the cut and restored afterward, with a warning when warnings are on
- **`color_labels(dend, k, h, labels, col)`**: the same cut for label colours. Without k or h, it colours the labels in leaf order from `col` [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/color_branches.R#L545-L586)]
- **`branches_attr_by_clusters(dend, clusters, values, attr)`**: sets `col`, `lwd`, or `lty` on the branches of each cluster [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/branches_attr_by.R#L141-L248)]
  - **Cluster 0**: means "no cluster"
  - **`branches_changed_have_which_labels = "all"`**: a branch changes only when all its leaves are in the cluster
  - **Overlap**: a node that belongs to two clusters (possible with `"any"`) keeps its old value
- **`branches_attr_by_labels(dend, labels, TF_values = c(2, Inf), attr, type)`**: changes every branch whose leaves are all (`type = "all"`) or any (`"any"`) of the given labels. With `"any"`, the paths from the given leaves to the root are highlighted [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/branches_attr_by.R#L325-L368)]
- **`branches_attr_by_lists(dend, lists, TF_values = c(2, 1))`**: highlights the branches from the root down to each node whose members form one of the given lists [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/branches_attr_by.R#L424-L449)]

### Cluster marks around the tree

- **`rect.dendrogram(tree, k, h, which, border = 2, ...)`**: draws a rectangle around each cluster [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/rect.dendrogram.R#L131-L275)]
  - **Options**: shade the rectangle (`density`, `angle`), add text under each cluster, set the lower and upper edges, and support horizontal trees
  - **Defect**: text placement with `horiz = TRUE` is wrong ([#25](https://github.com/talgalili/dendextend/issues/25), open)
- **`colored_bars(colors, dend, rowLabels)`**: draws one or more rows of coloured boxes under the leaves, one row per categorical variable, with row labels at the side [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/colored_bars.R#L255-L409)]
  - **`sort_by_labels_order = TRUE`**: puts colours given in data order into leaf order
  - **`horiz = TRUE`**: places the bars beside a horizontal tree
- **`colored_dots`**: the same with dots, and a `dot_size` argument

### Labels

- **Label rotation**: labels of a vertical tree are perpendicular (90 degrees), and labels of a horizontal tree are horizontal
- **No font face option**: italic labels need a work-around ([#111](https://github.com/talgalili/dendextend/issues/111), open), and a `labels_font` setting was proposed ([#71](https://github.com/talgalili/dendextend/issues/71), open)
- **`collapse_labels(dend, selected_labels)`**: replaces a subtree of adjacent leaves by one leaf (added in 1.17.0) [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/get_subdendrograms.R#L223-L227)]

## Tree changes

- **`prune(dend, leaves)`**: removes leaves one at a time [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/prune.R#L40-L123)]
  - **Binary node**: the sibling of the removed leaf replaces the node and keeps its own height, so the remaining heights stay true. The style of the removed parent node is lost, because the sibling takes its place
  - **Node with more children**: only the leaf is removed (fixed for non-binary trees in 1.9.0)
  - **Order values**: they are re-ranked to 1..n at the end
- **`intersect_trees(dend1, dend2)`**: prunes both trees to their shared labels [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/prune.R#L286-L311)]
  - **Warning**: "trees were pruned" appears only when `dendextend_options("warn")` is `TRUE`, which is not the default (observed)
  - **No shared labels**: it warns and returns an empty `dendlist`
- **`collapse_branch(dend, tol = 1e-8)`**: merges every internal node whose **height** is below `tol` into its parent, which creates polytomies [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/unbranch.R#L310-L352)]
  - **Test value**: the test uses the node height, so a zero-length internal branch high in the tree is kept
- **`unbranch(dend, branch_becoming_root = 1)`**: makes one child of the root the new root and attaches the other children to it [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/unbranch.R#L125-L175)]
- **`raise.dendrogram`**, **`flatten.dendrogram`**, **`hang.dendrogram`**, **`rank_branches`**: change heights without changing the topology

## Other plot types

- **Circular**: `circlize_dendrogram(dend, facing = "outside", labels = TRUE, labels_track_height = 0.1, dend_track_height = 0.5)` wraps the circlize package [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/circlize.R#L96-L147)]
  - **Labels**: they go on an outer track, and duplicate labels get a number prefix
  - **Label gap**: the gap between leaves and labels cannot be set ([#116](https://github.com/talgalili/dendextend/issues/116), open)
- **ggplot2**: `as.ggdend(dend)` converts a styled dendrogram into data frames of segments, labels, and nodes, and `ggplot()` draws them with identity scales for colour, width, and line type [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/ggdend.R#L182-L343)] [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/ggdend.R#L485-L628)]
  - **Options**: `horiz`, `offset_labels`, `theme = theme_dendro()`, and switches for segments, labels, and nodes
  - **No ggplot tanglegram**: the tanglegram exists only in base graphics
  - **Radial with `coord_polar()`**: label positions are wrong, which an example in the source admits [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/ape.R#L71-L82)], and branches overlap ([#81](https://github.com/talgalili/dendextend/issues/81), open)
  - **Deep trees**: `as.ggdend` hits a recursion error on very wide dendrograms ([#150](https://github.com/talgalili/dendextend/issues/150), open)
  - **Deprecated aesthetic**: ggplot2 3.4 warns that `size` for lines is deprecated (observed)
- **Heat maps**: dendrograms styled with dendextend are accepted by `gplots::heatmap.2`, heatmaply, and others (vignette)

## Scientific semantics

### Height, time, and branch length

- **Heights only**: a dendrogram stores node heights, and a branch length is the difference of two heights
  - **Leaf level**: all leaves sit at height 0 unless the tree was hung
  - **Dated trees**: a dated tree with tips at different times has no natural representation, which is why the phylo conversion demands ultrametric trees
- **Doubled time**: after the ape conversion, the height is twice the node age (see "Conversion from phylogenetic trees"). Axis values of a tanglegram of two time trees are therefore twice the time
- **Per-tree scale**: in a tanglegram each tree has its own height scale (see "Geometry"). `rank_branches` removes heights completely for a topology-only comparison
- **Cut levels from heights**: clusters for k, Baker's gamma, the Bk plot, and the `step1side` visiting order all come from cutting at heights
  - **Ties**: equal heights make some k impossible. The dendrogram `cutree` then returns `NA` or 0, and the hclust `cutree` picks a partition ([#21](https://github.com/talgalili/dendextend/issues/21))

### Clades, partitions, and rooting

- **Rooted clusters**: distinct edges, common subtrees, and the Robinson-Foulds distance all compare leaf sets below nodes, which is the rooted view. Rerooting is not offered
- **Edge style belongs to the lower node**: a branch takes the style of the node below it, which is the same convention for every highlight
- **Support values**: dendrograms have no field for them
  - **Conversion**: ape drops node labels in the conversion [[src](https://github.com/emmanuelparadis/ape/blob/c73e48d8a3493c7896ceebc5e5c44f2f1d4ac0bd/R/as.phylo.R#L106)]
  - **Display**: a user asked how to show bootstrap values ([#86](https://github.com/talgalili/dendextend/issues/86), open), and the pvclust integration is the only support display

### Polytomies and zero-length branches

- **Polytomies**: plotted fine as dendrograms, for example after `collapse_branch`, but a phylo polytomy cannot be converted, and the hclust-based fast paths fail on them
- **Zero-length branches**: allowed in a dendrogram. They pass the conversion when the tree stays binary and ultrametric, and the child is drawn at the height of its parent (observed)

## Defects

Each entry gives the input, the effect, and the evidence. Observed entries were reproduced in the trial.

### Tanglegram

- **`cex_main_right` is ignored** (observed): the right tree's title uses `cex.main = cex_main_left` [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L1062)]. With `cex_main_right = 0.5`, both titles were 15.84 px in the SVG
- **Grey and black for the same meaning** (observed): lines outside common subtrees change colour with one option [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L922-L955)]
  - **Default options**: these lines are grey
  - **`common_subtrees_color_branches = TRUE`**: the same lines are black, because their colour is set to `"black"` before the grey default is applied to `NA` values
- **Per-line widths are dropped** (derived): `lwd` is passed whole to each `arrows()` call, which draws one segment and uses only the first value [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/tanglegram.R#L1044-L1050)]
  - **Requests**: users asked for varying widths in 2016 and again later ([#27](https://github.com/talgalili/dendextend/issues/27), open)
- **Line colour order surprises users** ([#47](https://github.com/talgalili/dendextend/issues/47), open, 16 comments): colours are indexed by the position of the left leaves from the bottom up
  - **Effect**: users who give colours in `tip.label` order or in data order get wrong colours with no warning
  - **Work-around**: a commenter found the right order is the reverse of `tip.label[order.dendrogram(...)]`
  - **Remedy**: a colour map by label would remove the problem
- **Documented line type differs**: line type 3 in code, 2 in the documentation and "dashed" in the paper (see "Highlight: distinct edges")
- **Long labels are cut** (observed, see "Geometry")

### Untangling and entanglement

- **Entanglement above 1** (observed): the normalisation assumes that the reversed order is the worst case. That holds for L of 1 or more, but not below [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/entanglement.R#L390-L392)]
  - **Example**: the trees `(A,(B,C))` and `((B,C),A)` give 1.5 at L = 0 and 1.2071 at L = 0.5
- **`stepBothSides` is missing from the list version** (observed): `untangle(dendlist, method = "stepBothSides")` fails with "'arg' should be one of ...", because the method list of `untangle.dendlist` was not updated [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/untangle.R#L144-L161)]
- **DendSer does not use the other tree** (observed and derived, see "Methods")
- **Silent fallback without DendSer** ([#154](https://github.com/talgalili/dendextend/issues/154))
- **Unused experimental code** (derived): `untangle_best_k_to_rotate_by_2side_backNforth` loops while the entanglement is unchanged, which inverts its stop rule, and the "evolution" functions are unused [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/untangle.R#L1104-L1122)]

### Comparison measures

- **Baker's gamma is a Spearman correlation** (derived, see "Tree comparison measures")
- **Cophenetic correlation of two `hclust` objects ignores labels** ([#49](https://github.com/talgalili/dendextend/issues/49), open): the sort by label is skipped for `hclust` input [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/cor_cophenetic.R#L176-L180)]
  - **Report**: the reporter got 1 for two `hclust` objects with permuted labels, but 0.3125 after conversion to dendrograms or phylo objects
- **`dend_diff` with `which`** ([#59](https://github.com/talgalili/dendextend/issues/59), open): the first part, the first tree drawn twice, was fixed in 2018 by [#80](https://github.com/talgalili/dendextend/pull/80), but the issue is still open

### Scale and robustness

- **Recursion depth**: most functions recurse over the nested list ([#53](https://github.com/talgalili/dendextend/issues/53), open)
  - **Failure**: trees with about 5000 to 10000 leaves fail with "evaluation nested too deeply" or a C stack error in `get_branches_heights` and `color_branches`
  - **Work-around**: a larger operating-system stack
- **Zero-length branches from collapsed clades** ([#92](https://github.com/talgalili/dendextend/issues/92), open): see "Conversion from phylogenetic trees"
- **Warnings off by default**: `dendextend_options("warn")` is `FALSE`, so three changes happen silently
  - pruning to shared leaves
  - recycled colour vectors
  - unique-label fixes

## Performance

Observed in the trial on one core:

- **Entanglement**: 2.9 ms per call at 50 leaves and 6.7 ms at 100 leaves. Every untangling method calls it in its inner loop
- **Untangling**: `step2side` 3.0 s at 50 leaves and 11.2 s at 100 leaves. `stepBothSides` 140 s at 50 leaves. A tanglegram with `step2side` at 100 leaves took 12.6 s, almost all of it untangling
- **Drawing**: the default highlights compute distinct edges and common subtrees, which compare all leaf sets of both trees. `faster = TRUE` turns them off
- **History**: the optional C++ package `dendextendRcpp` once replaced slow functions such as `labels` and `cutree` for dendrograms (`NEWS.md`)
  - **Removal**: it was dropped from tests and suggestions in 1.5.2 (2017), and its code was removed in 1.9.0

## Interaction with TreeKnit output

TreeKnit writes these outputs:

- resolved segment trees (Newick, NEXUS)
- an ARG in extended Newick with `#H` hybrid nodes and `[&segments={0,1}]` annotations ([packages/treeknit-io/src/arg.rs#L12-L32](../../packages/treeknit-io/src/arg.rs#L12-L32))
- `MCCs.json` and `MCCs.dat`
- Auspice JSON
- SVG figures

dendextend reads none of these files itself. ape reads the trees, and dendextend converts them.

- **Binary ultrametric trees work** (observed): the 100-leaf fixture `sim_k2_n100_r0.02` converts directly, and the default tanglegram colours exactly TreeKnit's naive MCCs as common subtrees
- **Resolved trees with polytomies or dated tips fail** (observed): the 50-leaf polytomy fixture `sim_k2_n50_r0.02_poly` (28 zero-length edges, not binary, not ultrametric) stops with "the tree is not ultrametric", both in `as.dendrogram` and in `tanglegram`
  - **Work-around**: a user must resolve polytomies with `ape::multi2di` and force an ultrametric tree, which changes the branch lengths that TreeKnit inferred
- **MCCs as colours** (derived): `MCCs.json` lists leaf sets
  - **Mapping**: converted to a cluster ID per leaf in leaf order, it fits `color_branches(clusters = ...)` for the branches and `color_lines` for the lines
  - **Limit**: MCCs that are not clades colour only their maximal clade subtrees, because `branches_attr_by_clusters` changes a branch only when all its leaves are in one cluster
- **ARG** (observed): dendextend has no network model
  - **`ape::read.tree`**: reads the extended Newick, drops the `[&...]` comments, and keeps `R1#H1` as an ordinary tip and an internal node label. `as.dendrogram` then fails with "the tree is not ultrametric"
  - **`ape::read.evonet`**: reads one reticulation, but the conversion fails with "the tree is not binary"
- **Different leaf sets**: when the two trees of a pair have different leaves, `tanglegram` prunes both to the shared leaves without a warning

## Ideas from the issue tracker and history

### Open requests

- **Tanglegram lines and labels**:
  - **Variable widths of connecting lines** ([#27](https://github.com/talgalili/dendextend/issues/27)): requested to stress chosen links in addition to colour
  - **Colour lines by clades of the left tree** ([#47](https://github.com/talgalili/dendextend/issues/47)): select a clade and colour its lines, similar to the common-subtree colouring
  - **Hide tip labels in a tanglegram** ([#48](https://github.com/talgalili/dendextend/issues/48)): the maintainer offered a `show_labels` parameter if he got an example
  - **Interactive tanglegrams with plotly** (`TODO` file)
- **Layout and rotation**:
  - **Rotate chosen branches in very large trees** ([#110](https://github.com/talgalili/dendextend/issues/110)): rotation by label vector is impractical for 8000 leaves
  - **Leaves on a continuous axis** ([#50](https://github.com/talgalili/dendextend/issues/50)): place leaves at x positions from a variable, as constrained clustering plots do
  - **Plot with node count on the height axis** ([#108](https://github.com/talgalili/dendextend/issues/108))
- **Colouring**:
  - **Colour inner nodes** ([#23](https://github.com/talgalili/dendextend/issues/23))
  - **Colour tips or branches from an external table** ([#77](https://github.com/talgalili/dendextend/issues/77))
- **Comparison and conversion**:
  - **More tree comparison methods** ([#84](https://github.com/talgalili/dendextend/issues/84))
  - **A p-value for `cor.dendlist`** by label permutation ([#19](https://github.com/talgalili/dendextend/issues/19))
  - **Bk plot with only the upper confidence line** ([#68](https://github.com/talgalili/dendextend/issues/68))
  - **Non-ultrametric conversion to phylo** ([#162](https://github.com/talgalili/dendextend/issues/162))

### Declined

- **New features without a pull request**: the maintainer answers several requests with an offer to review a pull request instead of building the feature ([#108](https://github.com/talgalili/dendextend/issues/108), [#120](https://github.com/talgalili/dendextend/issues/120))
- **Questions are sent to Stack Overflow** ([#79](https://github.com/talgalili/dendextend/issues/79), [#110](https://github.com/talgalili/dendextend/issues/110))

### Features found only in the history

- **Untangling experiments**:
  - **Spearman-based entanglement**: an early entanglement was `(1 - cor_spearman(orders)) / 2`, and another was the plain absolute rank sum. Both remain as comments [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/entanglement.R#L406-L426)]
  - **Optimal leaf ordering untangle**: a commented-out `untangle_OLO` used `seriation::seriate(method = "OLO")` on the order matrix of both trees [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/untangle.R#L1128-L1149)]. A 2016 commit records "trying a new untangle method (didn't work)" ([`77e933c`](https://github.com/talgalili/dendextend/commit/77e933c))
  - **Genetic search**: the unused "evolution" functions combine two pairs of trees with `step2side` and keep the better child [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/untangle.R#L1022-L1059)]
  - **Seed search**: a commented loop runs random search plus `step2side` for up to 100,000 random seeds and keeps the best [[src](https://github.com/talgalili/dendextend/blob/0af62aaee571042d82012e444842700890526f73/R/untangle.R#L1348-L1390)]
- **Removed packages**:
  - **d3dendrogram**: an interactive D3 dendrogram, removed in 1.12.0 (2019) because ggdend and plotly made it unnecessary (`NEWS.md`)
  - **dendextendRcpp**: C++ versions of slow functions, removed in 2017 and 2018 (see "Performance")

## Open scientific problems

- **Cost function for untangling**: entanglement measures leaf displacement, while the literature usually counts crossings. Neither one rewards keeping a common subtree contiguous, which is the structure a reassortment display needs
- **Scale of two trees**: independent height scales use the space well but make equal x positions mean different times. A shared scale shows time correctly but can squeeze one tree
- **Node or rank heights**: cut-based measures (Baker's gamma, Bk, cluster colouring, greedy visiting order) depend on the order of node heights, and ties make some cut levels undefined
- **Rooted or unrooted comparison**: the rooted cluster distance and the unrooted split distance answer different questions ([#97](https://github.com/talgalili/dendextend/issues/97))
- **Inference for tree similarity**: correlation values need permutation or bootstrap tests, and the package gives only recipes in the vignette

## Not covered by dendextend

- **Tree and network models**:
  - **Networks and ARGs**: reticulations, hybrid nodes, and segment annotations are absent
  - **Non-ultrametric and dated trees**: tip dates, a time axis offset, and an input path for trees with tips at different heights are absent
  - **Support values on branches**
- **Tanglegram display**:
  - **Curved or bundled connecting lines**: lines are straight segments only
  - **More than two trees in one tanglegram**: a `dendlist` of many trees is shown by pairs, by a correlation matrix, or by several tanglegrams on one page
  - **Interaction**: no zoom, pan, hover, or selection in the tanglegram. `click_rotate` works on a single tree
  - **Shared colour map by label**: colours are given as vectors in leaf order, never as a label-to-colour map

## Method and limits

- **Code**: read commit `0af62aa` in full or in the relevant functions
  - **Files read**: tanglegram, untangling, entanglement, comparison, colouring, pruning, cutting, and conversion
  - **Skipped**: CI files, the pkgdown site in `docs/`, and the pvclust, find_k, and heat-map helpers beyond their names
- **Documentation**: read the main vignette sections on rotation, pruning, `dendlist`, `dend_diff`, tanglegram, untangling, and the comparison measures. Read the paper through its publisher page. The other two vignettes were not read
- **Issue tracker**: listed all 113 issues and 49 pull requests with `gh`. Read the bodies and comments of 27 issues chosen by title (tanglegram, untangling, comparison, conversion, scale)
- **Trial**: ran R 4.6.1 in throwaway `rocker/r-ver` containers with no network access after the package install
  - **Packages**: dendextend 1.19.1, ape 5.8.1, DendSer, colorspace, and svglite, installed from CRAN
  - **Inputs**: hand-written Newick strings and copies of two TreeKnit simulation fixtures (`sim_k2_n50_r0.02_poly`, `sim_k2_n100_r0.02`)
  - **Outputs**: SVG files, inspected as text and as PNG renderings. Timings come from one run each on a shared machine and are rough
- **Not done**: no trial of `click_rotate` (interactive), `circlize_dendrogram`, or `Bk_plot` output. `stepBothSides` was not timed at 100 leaves because of the expected run time
