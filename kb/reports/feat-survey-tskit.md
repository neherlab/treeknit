# tskit feature survey: tree rendering, semantics, and edge cases

This report describes the drawing module of tskit, the Python library that stores genealogies as succinct tree sequences, as an idea inventory for tree, tanglegram, network, and ARG display. A tree sequence is a list of local trees along the genome. It is also the tskit representation of an ancestral recombination graph (ARG). The report records what the drawing functions do, how they do it, which scientific conventions they assume, which inputs break them, and which problems the maintainers left open. It does not compare tskit with `packages/web`.

tskit is licensed MIT. The tutorials are licensed CC BY 4.0.

- **Source**: [tskit-dev/tskit](https://github.com/tskit-dev/tskit) at commit `0e38fc3` (2026-09-14), 3131 commits since 2018. Source links point to this commit
  - The file `python/tskit/drawing.py` is byte-identical to the release 1.0.3 that the trial used
- **Tutorials**: [tskit-dev/tutorials](https://github.com/tskit-dev/tutorials) at commit `6659476` (2026-09-30)
  - The main source is the "Visualization" tutorial `viz.md` (rendered at <https://tskit.dev/tutorials/viz.html>)
  - The "ARGs as tree sequences" tutorial `args.md` covers ARGs
- **Other sources**:
  - The drawing-related issues and pull requests of tskit: the 28 issues labelled "Visualisation" and about 60 others found by title, 27 read with their comments
  - The changelog `python/CHANGELOG.rst` and the commit history of `drawing.py`
  - The Newick importer of [tskit-dev/tsconvert](https://github.com/tskit-dev/tsconvert) at commit `4ed3c0f`
  - The ARG paper the tutorial cites ([Wong et al. 2024](https://doi.org/10.1093/genetics/iyae100))
- **Evidence labels**:
  - "Observed" means seen in a trial with tskit 1.0.3 and msprime 1.4.4 in a throwaway `python:3.12-slim` container, with the SVG files rendered to PNG by `rsvg-convert`
  - "Derived" means read from the code without running it
  - Where the docstrings or tutorials and the code disagree, the code wins and the difference is noted
- **Shape of the code**: one Python module, [`python/tskit/drawing.py`](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py) (2803 lines)
  - The public entry points are `Tree.draw_svg`, `Tree.draw_text`, `Tree.draw`, `TreeSequence.draw_svg`, and `TreeSequence.draw_text` in [`python/tskit/trees.py`](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/trees.py)
  - The classes are `SvgTreeSequence` and `SvgTree` (on a shared base `SvgAxisPlot`), `SvgSkippedPlot`, `TextTreeSequence`, `VerticalTextTree`, and `HorizontalTextTree`
  - A minimal in-house SVG writer replaced the `svgwrite` dependency in 2025 [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L56-L243)]
  - The tests are in [`python/tests/test_drawing.py`](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tests/test_drawing.py) (182 test functions, with known-good SVG files in `python/tests/data/svg/`)

## Summary

### Scope

- **Static output only**: tskit draws static SVG strings and text. It has no interaction, no zoom, and no layout optimization
  - The maintainers move new display features to CSS styling or to separate projects (`tsviz`, `tskit_arg_visualizer`), because the SVG code "quickly turns into a lot of code" that is hard to test ([#798](https://github.com/tskit-dev/tskit/pull/798), [#2330](https://github.com/tskit-dev/tskit/issues/2330), [#2628](https://github.com/tskit-dev/tskit/issues/2628))

### Most relevant ideas for TreeKnit

- **Local trees side by side along a genome axis**: each tree gets an equal-width box, and a shaded trapezoid joins the box to the genomic interval of the tree on a physical axis
  - This is a compact way to show which part of the genome each tree covers (see "Tree sequence layout")
- **Stable leaf order across trees**: the default `minlex` order sorts children by the smallest leaf ID below them
  - The same samples stay in the same place in consecutive trees where the topology allows it
  - There is no untangling (see "Child order")
- **Nested SVG groups with ID classes**: every node is an SVG group that contains its subtree and the edge above it
  - The classes name the node ID (`n7`), parent ID (`a10`), edge ID (`e4`), population, individual, mutations, and sites
  - One CSS rule can therefore highlight a clade, a lineage, or the same edge in all trees (see "CSS classes and highlighting")
- **ARGs as local trees with unary nodes**: an msprime "full ARG" stores each recombination as two nodes at the same time, one for each side of the breakpoint
  - tskit draws them as unary nodes on branches of the local trees and offers no graph view of its own
  - The tutorials use `tskit_arg_visualizer`, networkx, and graphviz for graph views (see "ARGs and networks")
- **Tanglegram recipe**: the visualization tutorial builds a tanglegram of two trees from internal classes, CSS rotations, and straight lines between matching leaves
  - It does not untangle (see "Two trees and tanglegrams")

### Scientific conventions

See "Scientific semantics".

- **Time comes from node times**: a parent must be strictly older than its child
  - Zero-length and negative branches therefore cannot exist, and every node has a time
- **Three time scales**: linear time, `log(t + 1)`, and rank
  - The rank scale spaces distinct node times evenly but labels the axis with real times

### Known weak points

- **Labels and ticks collide**: axis labels and node labels overlap, labels are not measured, and tick labels at the plot edges are skipped or clipped (see "Defects")

## Tree sequence layout

`TreeSequence.draw_svg` draws the local trees of a tree sequence left to right in one SVG [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1331-L1532)].

### Boxes and size

- **One box per tree**: the plot area is split into equal-width boxes, one per drawn tree plus one per block of skipped trees. Each box holds an independent `SvgTree` [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1437-L1471)]
- **Default size**: 200 by 200 user units per box, so the default width grows with the number of trees [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1397-L1398)]
  - **Observed**: 6 trees give `width="1200" height="200"`
  - **Width per leaf was declined**: making the width grow with the sample count was proposed and closed, because it makes it hard for users to compute positions for their own annotations ([#2339](https://github.com/tskit-dev/tskit/issues/2339))
- **Margins**: fixed margins surround the plot and each tree box [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1433-L1436)] [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1654-L1657)]
  - **Tree sequence plot**: 20 units left and right and 10 units at the bottom, plus one line height (16.8 units) at the top for a title
  - **Tree box**: 20 (left, right), 10 (top), and 15 (bottom)
- **`canvas_size`**: changes the SVG width and height without rescaling the drawing [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L880-L908)]
  - It gives room for long labels or for elements that CSS moves outside the plot area
- **`title`**: a centred text at the top, in a group of class `title` [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1473-L1481)]
  - The tutorial uses `<tspan>` elements inside the title string to get two lines, because the string is inserted without escaping [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L326-L335)]

### Genome axis (`x_scale`)

- **`x_scale="physical"`** (default): the axis is linear in genome position [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1571-L1611)] [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1172-L1204)]
  - **Ticks**: they sit at the tree breakpoints and are labelled with the position in bold
  - **Sites**: drawn as tick marks above the axis, with one red chevron per mutation
- **Shaded trapezoids** (physical mode): each tree box continues downward as a polygon whose lower edge spans the tree's genomic interval on the axis [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1275-L1323)] [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L971-L973)]
  - **Alternating shade**: every second polygon is grey at 10% opacity (`.background path:nth-child(odd)`), so the eye can follow each tree to its interval
  - **Observed**: a tree that covers 65 bp still gets a full 200-unit box, and its trapezoid narrows to a short interval on the axis
  - **Curved alternative in a comment**: a code comment gives Bézier path commands as an alternative to the straight diagonals [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1302-L1306)]
- **`x_scale="treewise"`**: each tree boundary is one evenly spaced tick, so the axis is a tree index axis labelled with breakpoint positions [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1612-L1628)]
  - There is no shading and there are no sites
  - `x_regions` raises an error in this mode
- **`x_axis`**: on by default for tree sequences and off for single trees. The default axis title is "Genome position" [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1039-L1041)] [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1403-L1405)]
- **Integer labels**: tick labels have no decimals when all values are integers, otherwise two decimals [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L433-L443)]
- **`x_regions`**: a mapping `{(left, right): label}` draws labelled boxes on the axis, for example genes [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1112-L1130)]
  - The boxes are yellow at 50% opacity, so overlaps show
  - Each region group gets the class `r<i>`

### Genome window (`x_lim`)

- **Cropping**: `x_lim=[left, right]` cuts the tree sequence with `keep_intervals(..., simplify=False)`, so node IDs stay the same [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L500-L528)]
  - Mutations outside the window are dropped
  - Node, site, and mutation classes keep their original IDs through offsets
- **Empty flanks are hidden by default**: without `x_lim`, the axis starts at the first edge and ends at the last edge, unless sites lie outside [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L471-L492)]
  - `x_lim=[0, ts.sequence_length]` forces the whole genome
- **Clipped ends lose their tick**: a breakpoint created by the crop is not labelled [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1630-L1636)]
  - **Observed**: with `x_lim=[200, 700]` on a tree sequence with breakpoints at 196 and 686, the axis shows only the tick "686", and neither 200 nor 700 is labelled

### Many trees (`max_num_trees`)

- **Middle trees are skipped**: when more trees would be drawn than `max_num_trees`, the first `ceil(max/2)` and the last `floor(max/2)` trees are kept [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L494-L541)]
  - `max_num_trees` below 2 is an error
- **Skipped block**: one box with the two-line text "N trees" and "skipped" stands for the skipped trees [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L942-L963)]
- **Broken axis**: in physical mode the axis becomes piecewise linear [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1581-L1597)]
  - The skipped region gets one box width, and that axis segment is dashed (class `ax-skip`, `stroke-dasharray: 4`)
  - **Observed**: with 624 trees and `max_num_trees=10`, the box reads "614 trees skipped", and the axis jumps from 9817 to 993504 over a dashed segment

### How consecutive trees relate

- **Trees are drawn independently**: each tree has its own layout
  - Nothing marks the subtree that moved at a breakpoint
  - The only links between trees are the shared CSS classes (see "CSS classes and highlighting")
- **Shared time scale**: in a tree sequence, `max_time` and `min_time` default to `"ts"`, so all trees use the same vertical scale and node heights are comparable across boxes [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1399-L1402)]
  - A single tree defaults to `"tree"`
- **SPR animation recipe**: the tutorial animates the change from one tree to the next with d3.js [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L2020-L2160)]
  - **Method**: it moves each `.node.n<id>` group of tree `t<i>` to the transform of the same node in tree `t<i+1>`
  - **Recombination nodes**: for a recombination node with no match it uses the next node ID, which is the second msprime recombination node
  - The tutorial calls the code "buggy"

## Single tree layout

`Tree.draw_svg` draws one tree with the same machinery [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1644-L1911)].

### Coordinates

- **Orientation**: the root is at the top and time increases upward
  - There is no option for other orientations in SVG. The tutorial rotates trees with CSS transforms instead
- **Leaf positions**: leaves are placed on an even grid in traversal order, at `1, 2, ..., n`, scaled to the plot width with half a step of space at each side [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2069-L2119)]
- **Parent position is the midpoint of the outer children**: an internal node sits at `(min + max) / 2` of its children's positions, also in a polytomy [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2104-L2110)]
  - A unary node takes its child's position
- **Edge shape**: each edge is one path `M 0 0 V dy H dx` from the child: up to the parent's time, then horizontally to the parent [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2265-L2273)]
  - The horizontal bar of a parent is therefore made of the top parts of its children's edges
  - **Observed**: `<path class="edge e4" d="M 0 0 V -157.144 H -92.9"/>`
- **Rounding**: coordinates are rounded to six significant digits to keep the SVG small [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L558-L569)]
- **Non-recursive drawing**: layout and drawing use explicit stacks since 2025, so deep trees do not hit the Python recursion limit ([#3120](https://github.com/tskit-dev/tskit/pull/3120))

### Child order (`order`)

- **`order="minlex"`** (default since 0.3.0): a postorder traversal that visits the children of each node in increasing order of the smallest leaf ID below them [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/trees.py#L2500-L2539)] [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L363-L383)]
  - **Result**: the leaf IDs read in minimum lexicographic order. Sample 0 is always leftmost
  - **Why**: the order depends only on the leaf IDs below each node, which are the same in every tree. Consecutive trees therefore keep their samples in place except where the topology changes. The proposal showed that the old order shuffled samples between trees that differ by one regraft ([#389](https://github.com/tskit-dev/tskit/issues/389))
  - **Roots are ordered too**: the traversal starts at the virtual root, so several roots are also sorted by their smallest leaf ID
  - **Leaves, not samples**: the key is the leaf ID, so a non-sample leaf takes part and an internal sample does not
  - **Observed**: in a two-segment test tree sequence with trees `((A,B),(C,D))` and `(B,(A,(C,D)))`, minlex gives the leaf orders `A B C D` and `A C D B`. B moves from second to last
- **`order="tree"`**: the child order of the internal linked-list structure, which depends on the order in which edges were inserted
  - **Observed**: the same leaves change places between trees with no topology change (`N1 N3 N0 N2` then `N0 N1 N3 N2`)
- **Undocumented custom order**: `Tree.draw_svg(order=...)` also accepts a list of node IDs in postorder [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1740-L1747)] [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2072-L2073)]
  - It draws only those nodes, so a subtree can be cut off or reordered
  - The code raises an error when the list is not a postorder
- **No rotation and no untangling**: the tutorial says tskit has no method to rotate branches [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L514-L557)]
  - **Workaround**: to get another leaf order, the tutorial renumbers the nodes with `TreeSequence.subset`, so minlex produces the wanted order, and relabels the nodes with the old IDs

### Nodes and symbols

- **Sample nodes are squares, other nodes are circles**: both have `symbol_size` 6 units by default [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1815-L1830)] [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L1267-L1303)]
  - Samples can be internal nodes or at times other than 0, and leaves need not be samples
- **Unary nodes are drawn**: a node with one child appears as a circle on the branch. A full ARG has many of them (see "ARGs and networks")
- **Isolated samples**: a sample must exist in every tree, so a sample without edges in one tree is drawn as an unconnected square in that tree [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L1299-L1303)]
- **Multiple roots**: each root starts its own group, and roots are placed in minlex order
  - **Observed**: a tree with roots `[0, 3, 4, 6]` draws one cherry and three lone leaves side by side, with no common line
- **Root branch** (`force_root_branch`): a stub above each root, of length 1/8 of the time range [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1426-L1431)] [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2012-L2023)]
  - In a tree sequence it is drawn above every root of every tree as soon as any root has a mutation, so that all trees look the same ([#2310](https://github.com/tskit-dev/tskit/issues/2310))
- **Tooltips** (`node_titles`, `mutation_titles`): a `<title>` element on a symbol, which browsers show on mouse-over. It reduces label clutter [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2331-L2332)]

### Collapsed subtrees and polytomy packing

These features came from work on SARS-CoV-2 trees with large polytomies ([#3011](https://github.com/tskit-dev/tskit/issues/3011), [#2372](https://github.com/tskit-dev/tskit/issues/2372)).

- **Multi-sample tips**: when a custom `order` stops the traversal at an internal node, that node is drawn as a tip [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2316-L2352)]
  - **Shape**: a trapezium below its symbol, with the italic label `+N`, where N is its number of descendant samples
  - **Classes**: the trapezium has the class `multi` and the label the class `summary`
- **Tracked-sample traversal**: the private function `_postorder_tracked_minlex_traversal` limits the traversal to tracked samples [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L599-L671)]
  - It descends only into subtrees that contain tracked samples
  - It puts the children with the most tracked samples first
  - It can stop at subtrees whose tracked fraction is at least `collapse_tracked`
- **`pack_untracked_polytomies`** (a keyword passed through `**kwargs`, not in the docstring): in a polytomy, two or more children without tracked samples are replaced by a dotted horizontal line and the label `+n/m` [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2080-L2103)] [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2225-L2249)]
  - **Label**: n samples, m branches, with m in bold Unicode digits
  - **Line length**: an extra length of `1 + ln(m)` leaf steps, so a large polytomy takes a little more space
  - **Single untracked child**: it becomes a normal condensed tip
  - **Observed**: with tracked samples `[0, 4]`, the untracked polytomy node 6 with three sample children is drawn as one triangle labelled `+3`
  - **Unsolved case**: a polytomy of many lineages where nearly all contain tracked samples cannot be collapsed in a meaningful way. The maintainer calls it "an insoluble issue" ([#3011](https://github.com/tskit-dev/tskit/issues/3011))
- **CSS summary triangles**: the tutorial hides a clade with `.n4 > .node {display: none}` and turns the node symbol into a triangle [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L894-L930)] [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L1430-L1552)]
  - The triangle uses `clip-path` and `transform: scale(...)`, with a size proportional to the sample count
  - The hidden clade still takes its horizontal space

## Labels

- **Default labels are IDs**: every node shows its numeric ID and every mutation its ID [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1831-L1839)]
  - `node_labels` and `mutation_labels` replace them
  - A node missing from the mapping gets no label, so `node_labels={}` hides all labels
- **Placement rules** [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2334-L2359)]:
  - **Tips**: centred below the symbol
  - **Roots without a root branch**: centred above the symbol
  - **Other nodes**: above and to the left when the node is the leftmost child of its parent (class `lft`, anchored at the end), otherwise above and to the right (class `rgt`). The label therefore points away from the parent's bar
- **Mutation labels**: to the left of the cross on the leftmost child's branch, otherwise to the right [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2299-L2308)]
- **No label measurement**: SVG cannot measure text before rendering, so labels can overlap each other, the axes, or the canvas edge
  - **Observed**: long tip labels on a 200-unit tree run into each other
  - **Maintainer answer**: CSS rotation (`.leaf > .lab {transform: rotate(90deg) translate(6px)}`) and `canvas_size` ([#26](https://github.com/tskit-dev/tskit/issues/26)) [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L240-L257)]
- **Hover labels**: the tutorial hides labels with CSS and shows them on `:hover` of the symbol. This needs the SVG inline in HTML [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L753-L822)]
- **Text halo**: polytomy labels get a white stroke under the fill (`paint-order: stroke`) so they stay readable over lines [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L992-L993)]

## Mutations and sites

- **Mutation symbol**: a red cross on the branch above the mutation's node. Mutations above a root sit on the root branch [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1840-L1860)]
- **Mutation height**: at the mutation's time when it is known [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1913-L1932)]
  - When times are unknown, or with the rank scale, the mutations of a branch are spaced evenly along it, oldest at the top, sorted by site and then by parent mutation
- **Hidden lineage line**: each mutation group holds an invisible line from the mutation down to its node [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2292-L2294)] [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L1010-L1028)]
  - CSS can show it, so the lineage below a mutation can be coloured from the exact mutation point ([#1155](https://github.com/tskit-dev/tskit/issues/1155))
- **`all_edge_mutations`** (single trees only): also draws the mutations that lie on the same edge outside the tree's interval [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1780-L1811)]
  - They are hot pink with the class `extra`
  - The tree's axis widens to include them
- **`omit_sites`**: removes all sites and mutations, which helps for large trees ([#2516](https://github.com/tskit-dev/tskit/issues/2516))
- **Mutations above nodes outside the drawn tree**: a `UserWarning` lists them, and they are not drawn [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1771-L1777)]

## Time axis

### Scales (`time_scale`)

- **`time_scale="time"`** (default): linear in node time [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L282-L284)]
- **`time_scale="log_time"`**: `log(t + 1)` when the minimum time is 0, else `log(t)`. Negative times raise an error [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L264-L280)]
- **`time_scale="rank"`**: the distinct node times are ranked and spaced evenly. Nodes with equal times share a row [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1953-L1988)] [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1504-L1509)]
  - **Rows match across trees**: with `max_time="ts"` the ranks come from all nodes used in the tree sequence
  - **Real-time labels**: the default ticks are labelled with the real times, because a rank axis "0, 1, 2" carries no information ([#1263](https://github.com/tskit-dev/tskit/issues/1263))

### Range (`max_time`, `min_time`)

- **Values**: `"tree"`, `"ts"`, or a number [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1989-L2011)]
  - **Numeric `max_time`**: it sets the top of the scale. Nothing clips older nodes, and there is no clip path in the module
  - **Single time**: when all nodes have one time, the range is set to 1

### Axis, ticks, and grid lines

- **`y_axis`**: `True` or `"left"` draws the axis on the left, and `"right"` on the right (since 1.0.0) [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1041-L1047)] [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L420-L430)]
  - **Default title**: "Time ago", or "Node time" for the rank scale
  - **Units**: the time units go in parentheses when the tree sequence defines them, for example "Time ago (generations)"
- **One axis for all trees**: a tree sequence Y axis needs one time scale for all trees, so `max_time="tree"` with `y_axis=True` fails [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1494-L1502)]
  - **Observed**: "Can't draw a tree sequence Y axis if trees vary in timescale"
- **`y_ticks`**: a list of values, or a mapping from values to labels, for example `{50: "Introgression event"}` [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L546-L555)] [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1250-L1273)]
  - **Default**: one tick per distinct node time
  - **Out-of-range tick**: a tick outside the axis logs a warning
  - **Rank scale**: the values are ranks ([#2990](https://github.com/tskit-dev/tskit/issues/2990))
- **`y_gridlines`**: a horizontal line behind the trees at each tick, very light by default (`#FAFAFA`) [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1259-L1260)] [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L1345-L1428)]
  - Single grid lines can be styled with `nth-child`, counted from the bottom

## CSS classes and highlighting

All styling goes through CSS classes. Colour parameters exist only in the legacy `Tree.draw`, which turns them into inline styles. Requests for colour parameters in `draw_svg` were declined in favour of CSS ([#2330](https://github.com/tskit-dev/tskit/issues/2330)) [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L717-L733)].

### Structure

- **Nested groups follow the tree**: each node is a `<g>` translated relative to its parent [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2195-L2273)] [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L932-L982)]
  - **Content**: its children's groups, the edge above it, its mutations, its symbol, and its label
  - **Effect**: hiding `.n13` hides the whole subtree with the edge above it
- **Tree sequence structure**: `g.tree-sequence` holds `g.background`, `g.axes` (with `x-axis` and `y-axis`), and `g.plotbox.trees` with one `g.tree.t<i>` per tree
- **One stylesheet**: the built-in style and the user `style` string go into one `<style>` element, because Inkscape 0.92 needs a single stylesheet [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1032-L1034)]

### Class names

The node classes are built by `info_classes` [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2121-L2156)]. Observed node group: `class="a6 i2 leaf node n2 p0 sample"`.

- **Node identity**:
  - **`n<id>`**: the node ID
  - **`a<id>`** or **`root`**: the parent ("ancestor") ID, or `root` for a root. The letter `p` is the population, not the parent
  - **`p<id>`** and **`i<id>`**: the population and the individual of the node
- **Node state**:
  - **`sample`**, **`leaf`**, **`c<k>`**: sample flag, leaf in this tree, or the number of children k of an internal node
  - **`m<id>`** and **`s<id>`**: the mutations above the node and their sites, so a rule can select the subtree below a mutation
- **Trees and edges**:
  - **`t<i>`**: the tree index on the tree group
  - **`e<id>`** or **`root`**: on the edge path, the tskit edge ID, or `root` for a root branch (since 1.0.0, [#557](https://github.com/tskit-dev/tskit/issues/557)) [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2266-L2269)]
- **Element classes**: `sym` (symbol), `lab` (label), `edge`, `mut`, `site`, `extra`, `unknown_time`, `lft`, `rgt`, `polytomy`, `multi`, `summary`, `tick`, `grid`, `title`

### Highlighting patterns from the tutorial

- **Across trees**:
  - **Clade in every tree**: `.n13 .node .edge` colours all edges below node 13, in every tree where node 13 exists. `.n13 .edge` also colours the edge above it [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L984-L1008)]
  - **Same branch across trees**: `.a15.n9 > .edge` colours the branch from 15 to 9 wherever it occurs [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L688-L691)]
    - **Same edge across trees**: `.e<id>` selects one tskit edge, which can span several adjacent trees. A branch with the same parent and child in non-adjacent trees is a different edge [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L746-L751)]
  - **Haplotype sharing around a sweep**: all branches of the central tree are coloured red in all trees, which shows how far shared branches extend along the genome [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L707-L744)]
  - **Edge span as grey level**: one rule per edge ID sets the stroke from the edge span, so edges that persist over long genome regions are darker ([#1265](https://github.com/tskit-dev/tskit/issues/1265)) [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L693-L705)]
- **Within a tree**:
  - **Path to the root**: rules for the node and each of its ancestors from `Tree.ancestors` [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L1030-L1039)]
  - **Colour by population**: `.node.p0 > .sym {fill: red}` and so on, with a legend drawn by hand in the `preamble`. The legend reuses the same classes, so it picks up the styled colours [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L1613-L1648)]
- **On a web page**:
  - **Scoped styles**: an SVG inline in HTML shares its styles with every other SVG on the page, so the tutorial sets a unique `id` through `root_svg_attributes` and prefixes every rule with it [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L1080-L1094)]

### `preamble` and extension

- **`preamble`**: raw SVG inserted right after the opening tag, unchecked [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L910-L917)]
  - Uses: legends, annotations, and whole nested `<svg>` elements ([#3086](https://github.com/tskit-dev/tskit/issues/3086))
- **Internal coordinates**: the tutorial builds `tskit.drawing.SvgTree` directly and reads `node_x_coord` and `timescaling.transform` to place annotations [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L1651-L1699)]
  - The tutorial marks these as internal APIs that may change ([#2699](https://github.com/tskit-dev/tskit/issues/2699))
- **CSS transforms**: skews and staggers give a 3D look of trees along the genome [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L1554-L1610)]
  - `transform` in CSS is an SVG 2 feature that Inkscape and librsvg ignore, so the tutorial converts through headless Chromium to PDF [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L1117-L1131)]

## Two trees and tanglegrams

tskit has no tanglegram function. The tutorial gives two recipes.

- **Side by side**: `draw_svg_side_by_side` draws the second drawing into the `preamble` of the first [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L433-L510)]
  - The second drawing moves right with `root_svg_attributes={"x": offset}`
  - `canvas_size` widens the first drawing
- **Tanglegram** (`tanglegram()` in the tutorial) [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L1701-L1953)]:
  - **Facing trees**: two `SvgTree` objects are rotated with CSS, the left one by -90 degrees and the right one by +90 degrees, so the tips face each other. The right tree's time axis goes on the right with `y_axis="right"`
  - **Left leaf order reversed**: the left tree is renumbered with `subset` so its minlex order is reversed, which puts its first leaf at the top after the rotation. Node labels are remapped to the old IDs, and the function can return the ID maps for styling
  - **Links**: one straight blue `<line>` per leaf between the two trees. `line_gap=None` draws all lines of equal length in the middle. A number starts each line that far from its leaf, which handles tips at different times [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L1920-L1935)]
  - **No untangling**: the docstring says the function plots the default minlex order and that leaf orders from an external program can be passed through `order` [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L1746-L1750)]
  - **Trees from two tree sequences**: the docstring suggests `TreeSequence.concatenate` with a node mapping of the shared samples
- **Shared clades**: a hash of the sorted sample set below each internal node finds the clades that exist in both trees [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L1980-L2018)]
  - CSS marks them with magenta circles

## ARGs and networks

### How tskit stores an ARG

- **Nodes are genomes, edges carry intervals**: a node is an ancestral genome at a time, and each edge `(left, right, parent, child)` says that the child inherits the interval from the parent [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/args.md?plain=1#L322-L346)]
  - A node with different parents on different intervals is a recombinant
  - The tutorial calls this an edge-annotated "genome ARG" (gARG), in contrast to event-based ARGs that store a breakpoint on a recombination node
- **Full ARG from msprime**: `record_full_arg=True`, or `coalescing_segments_only=False` with `additional_nodes=COMMON_ANCESTOR | RECOMBINANT`, keeps the recombination nodes and the common-ancestor nodes without coalescence [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/args.md?plain=1#L40-L78)]
- **Two nodes per recombination**: msprime records the two parental genomes of the gamete, one for the interval left of the breakpoint and one for the right, both flagged `NODE_IS_RE_EVENT` at the same time [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/args.md?plain=1#L95-L104)]
  - **Observed**: `NODE_IS_RE_EVENT` = 131072 (bit 17) and `NODE_IS_CA_EVENT` = 262144 (bit 18). Nodes 4 and 5 are both at time 148.63, node 4 in the tree of [0, 6607) and node 5 in the tree of [6607, 10000)
  - **Why two nodes**: the pair can represent a "diamond", where both parents trace back to the same common ancestor, and the msprime ARG likelihood expects this form [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/args.md?plain=1#L221-L227)]
  - **Open conversion**: converting to one recombination node with metadata is a to-do in the tutorial, with the discussion in msprime issue 1942 [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/args.md?plain=1#L515-L517)]
- **Simplification removes the event nodes**: `simplify()` removes the unary recombination and common-ancestor nodes [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/args.md?plain=1#L229-L264)]
  - The local trees keep their shape and times, but the times and lineages of recombinations are lost
  - **Observed**: 8 nodes become 6, and `simplify(keep_unary=True)` keeps all 8

### Display in tskit

- **Local trees only**: `draw_svg` draws a full ARG as local trees [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/args.md?plain=1#L129-L157)]
  - Recombination nodes appear as unary circles on branches, and the reader matches them across trees by ID or by colour
  - **Tutorial styling**: recombination nodes are red and common-ancestor nodes blue, grid lines mark the recombination times, and only samples and recombination nodes have labels
- **Reading the SPR**: in the local trees, the branch that carries the recombination node is the pruned branch, which tells which side of the regraft moved [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/args.md?plain=1#L180-L199)]
  - The simplified tree sequence loses this
- **Observed overlap**: a recombination node at 148.63 sits almost on a coalescence at 140.11, so the labels "4" and "3" and the two Y tick labels overlap

### Graph views in the tutorials

- **tskit_arg_visualizer**: an interactive D3 drawing of the graph [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L2169-L2227)]
  - **Interaction**: nodes can be dragged horizontally, and hovering over the "genome bar" below the graph highlights a local tree
  - **Edge styles**: `edge_type="ortho"` draws horizontal and vertical edges for full ARGs and shows the two recombination nodes as one point labelled "4/5". The default `edge_type="line"` suits simplified graphs, whose nodes can have several parents and several children
  - **Options in the tutorial**: `sample_order` (chosen by hand for each example), `variable_edge_width` (width by span), `show_mutations`, `label_mutations`
- **networkx and graphviz**: `to_networkx_graph` turns edges into a `MultiDiGraph`, or a `DiGraph` with a list of intervals per parent-child pair [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L2229-L2321)]
  - **Layout**: graphviz `dot` lays it out with the time-0 nodes on one rank, and edge labels list the intervals as `[left,right)`
  - **Limit stated by the tutorial**: `dot` places nodes on discrete levels, so there is "no easy functionality" to place nodes at their times [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L2319-L2321)]
  - **Simplest view**: `nx.multipartite_layout` by topological generation [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/args.md?plain=1#L493-L503)]
- **Other tools named**: ARGscape, Lorax, tsbrowse, and TwisstNTern [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L119-L132)]

## Text drawing

`Tree.draw_text` and `TreeSequence.draw_text` print trees with box-drawing characters [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2362-L2803)].

- **Characters**: Unicode heavy box characters (`┏ ┓ ┻ ╋ ┳ ━ ┃`) [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2655-L2673)]
  - `use_ascii=True` uses `+`, `-`, and `|`, which survive font substitution
  - Some Jupyter setups misalign the Unicode output ([#189](https://github.com/tskit-dev/tskit/issues/189))
- **Orientation** (single trees only): `"top"` (default), `"bottom"`, `"left"`, `"right"` name the side of the root [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2712-L2803)]
  - The left and right forms widen each branch to fit its label
- **Rank layout**: time is always ranked. Each distinct node time takes two rows, one for the label and one for the connector [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2498-L2542)]
- **Labels set the width**: each leaf column is as wide as its label plus one space, and internal labels are pushed right to clear their left neighbour [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2626-L2653)]
- **Custom labels**: with `node_labels`, a node missing from the map is drawn as a plain line character, so the tree shape stays visible [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2585-L2595)]
- **Tree sequences**: trees side by side, separated by `┊` columns [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2376-L2435)]
  - **Axes**: a time column on the left (format `"{:.2f}"`) and breakpoint positions on the bottom row
  - **Shared rows**: all trees use the time rows of the whole tree sequence, so young trees have empty rows above them (observed)
  - Mutations are not shown
- **Observed polytomy and unary node**: the unary node 5 above leaf 0 is printed as its label on the vertical line, and the polytomy uses `╋` where the parent sits over a child

## Input and export

- **No tree file reader**: tskit reads only its own `.trees` format and text tables [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/trees.py#L2640)]
  - It writes Newick (`Tree.as_newick`) and NEXUS (`TreeSequence.write_nexus`) but cannot read them
- **tsconvert Newick import**: `tsconvert.from_newick` reads exactly one tree [[src](https://github.com/tskit-dev/tsconvert/blob/4ed3c0f9bbbb79c7c394b352c58f5386cfef2b90/tsconvert/newick.py#L127-L222)]
  - **Times**: it computes node times from the root downward and shifts them so the youngest node is at time 0
  - **Short edges**: a length of 0 or less is an error unless `min_edge_length` raises it, because tskit forbids such edges
- **SVG only**: the drawing functions return an `SVGString`, which Jupyter renders through `_repr_svg_` [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L296-L303)]
  - `path=` also writes a file
  - There is no PNG or PDF output, by a rule that tskit adds no heavy dependencies ([#791](https://github.com/tskit-dev/tskit/issues/791))
- **Conversion advice**: Inkscape or ImageMagick with librsvg [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L1101-L1131)]
  - librsvg misaligns some labels because it ignores `dominant-baseline` ([#1625](https://github.com/tskit-dev/tskit/issues/1625))

## Configuration surface

`TreeSequence.draw_svg` takes these options [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/trees.py#L7684-L7872)]:

- **Output and styling**: `path`, `size`, `canvas_size`, `root_svg_attributes`, `style`, `preamble`
- **Genome axis**: `x_scale`, `x_axis`, `x_label`, `x_lim`, `x_regions`, `max_num_trees`
- **Time axis**: `time_scale`, `y_axis`, `y_label`, `y_ticks`, `y_gridlines`
- **Labels**: `title`, `node_labels`, `mutation_labels`, `node_titles`, `mutation_titles`
- **Tree content**: `order`, `force_root_branch`, `symbol_size`, `omit_sites`

Other entry points and options:

- **`Tree.draw_svg` differences**: it adds `max_time`, `min_time`, and `all_edge_mutations`, and lacks `x_scale`, `x_lim`, and `max_num_trees` [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/trees.py#L1828-L2012)]
- **Hidden options through `**kwargs`** [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L844-L867)]:
  - `pack_untracked_polytomies`
  - `max_time` and `min_time` for tree sequences
  - `debug_box`, which draws dashed outlines of the plot boxes
- **Deprecated aliases**: `tree_height_scale` and `max_tree_height` (since 0.3.6) still work with a `FutureWarning`
- **Legacy `Tree.draw`**: SVG or text, with `node_colours`, `edge_colours`, and `mutation_colours` [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L674-L779)]
  - A colour of `None` hides the element through zero opacity

## Scientific semantics

### Time and branch length

- **Node times define branch lengths**: every node has a time, and a branch length is the difference of two node times
  - **Strict order**: a parent must be strictly older than its child
  - **Observed**: a parent at the same time as its child fails with "time[parent] must be greater than time[child]"
  - **Consequence**: zero-length branches, negative branches, and missing lengths cannot occur in a drawing
- **Time runs backward**: the axis reads "Time ago", and 0 is the youngest time by default (`min_time`)
  - Negative times are allowed for the linear scale since 0.4.2 ([#2197](https://github.com/tskit-dev/tskit/issues/2197))
- **Polytomies encoded as zero-length splits cannot be stored**: a polytomy is a node with more than two children. A binary resolution would need distinct times
- **Samples at any time**: ancient samples sit above 0, and their squares are drawn at their times

### Rooting

- **Always rooted**: every local tree is rooted by time. The root is the oldest node of each connected part
  - A tree can have several roots when some samples do not coalesce in the stored history
- **Root branch is decoration**: its length is 1/8 of the drawn time range and carries no data, except that mutations above the root sit on it

### Node and edge attributes

- **Attributes live in tables**: population, individual, and metadata belong to nodes. Edges have only intervals
  - Labels and colours come from the user, usually from node metadata, through `node_labels` and CSS
- **Edges span intervals**: an edge in the drawing is one tskit edge, which can be shared by several trees
  - A branch with the same parent and child can belong to different edges in different trees

### Same leaves in all trees

- **Samples exist in every tree**: the leaf set of a tree sequence is fixed, so trees along the genome compare the same samples. A sample can be isolated in some trees

## Defects

### Layout and labels

- **Y axis title overlaps the tick labels** (observed): with `y_axis=True` on a tree sequence, "Time ago (generations)" is drawn across the label "301.96"
  - The maintainers know that labels cannot be measured. The issue was closed for inactivity ([#1654](https://github.com/tskit-dev/tskit/issues/1654))
- **Right Y axis is cut off** (observed): with `y_axis="right"` on a 1200-unit plot, the rotated title "Time ago (generations)" runs past the right canvas edge
  - The tick labels 0.08 and 0.00 overlap
- **Default Y ticks collide**: one tick per distinct node time gives overlapping labels when node times are close
  - **Observed**: 148.63 and 140.11 overlap
- **X tick labels collide** (observed): in the broken axis of `max_num_trees=10`, the labels 9817 and 993504 touch
  - The last label "1000000" is cut at the right edge
  - A 10000-unit axis also cuts its last label
- **Crop hides both window ends** (observed): `x_lim=[200, 700]` labels neither end of the axis (see "Genome window")
- **Long labels overlap** (observed): no truncation, no rotation by default

### Code

- **Literal class name on region boxes** (derived): the `<rect>` of an `x_regions` box gets `class_="r{i}"` without the `f` prefix, so its class is the literal text `r{i}` [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1122-L1123)]
  - The enclosing group has the correct `r<i>` class, so the tutorial's `.x-regions .r1 rect` rule still works
- **Sign error in the `above_right` label offset** (derived): `above_right` computes `-(line_h / 2 + dy)` where the other positions compute `-line_h / 2 + dy` [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2158-L2169)]
  - All current calls pass `dy=0`, so the error has no visible effect yet
- **`TreeSequence.draw_text` ignores unknown keywords** (observed): `ts.draw_text(foo=1)` succeeds, because `**kwargs` is accepted and never used [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/trees.py#L7874-L7924)]
  - `draw_svg(foo=1)` raises a `TypeError`
- **Saved file differs from the returned string** (derived): `draw(path)` writes a pretty-printed copy through `minidom`, while the return value is compact [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L910-L917)]
  - The code carries a "TODO remove the 'pretty'" comment
- **Tick warning uses `logging`**: a tick outside the axis is reported through `logging.warning`, which users of notebooks often do not see [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L1270-L1273)]
  - **History**: issue [#2870](https://github.com/tskit-dev/tskit/issues/2870) moved the unplotted-mutation message from `logging` to `warnings` in 2023, but the tick check added in 2024 uses `logging` again

### Documentation

- **`position_label_format` default** (derived): the docstring says `"{:.2f}"`, but the code prints integers without decimals and other values with two decimals [[src](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/tskit/drawing.py#L2379-L2385)]
- **Site classes in the tutorial**: the text explains `s15 s16` in the example, then names them `s16`, `s17` [[src](https://github.com/tskit-dev/tutorials/blob/66594766d62e170244b8e60ec4ed26aed1e4c2eb/viz.md?plain=1#L647-L659)]
- **Undocumented features**: the custom `order` list and `pack_untracked_polytomies` are in the changelog but not in the docstrings
  - Issue [#2372](https://github.com/tskit-dev/tskit/issues/2372) is open to document them
- **Stale test dependency**: `svgwrite` is still in the test requirements, although no code or test imports it ([`pyproject.toml`](https://github.com/tskit-dev/tskit/blob/0e38fc3d3c98ba35f4c84ed0be359d539bf75724/python/pyproject.toml#L80))

## Performance

All numbers are observed in the trial container, with `node_labels={}`.

- **One tree of 500 tips**: 0.14 s, 233 kB of SVG at `size=(5000, 800)`
- **50 tips, 624 trees**: 8.5 s and 14.7 MB for all trees
  - With `max_num_trees=10`: 0.12 s and 238 kB
- **Size grows with trees times nodes**: each tree is drawn in full with its own groups, labels, and classes
  - Shared edges are drawn again in every tree
- **Advice from the tutorials** ([#2604](https://github.com/tskit-dev/tskit/issues/2604)):
  - `simplify` to a subset of samples (with `filter_nodes=False` to keep IDs)
  - `x_lim`
  - `max_num_trees`
  - `omit_sites`
  - summary triangles

## Interaction with TreeKnit output

TreeKnit writes these files:

- Resolved segment trees (Newick, NEXUS)
- An extended Newick ARG with `#H` hybrid nodes and `[&segments={0,1}]` annotations ([packages/treeknit-io/src/arg.rs#L12-L33](../../packages/treeknit-io/src/arg.rs#L12-L33))
- MCC files
- Auspice JSON
- SVG figures

How these files relate to tskit:

- **No direct reading** (derived): tskit reads none of these files. A converter must build node and edge tables
- **tsconvert loses the reticulations** (derived): `from_newick` reads one tree and makes one node per Newick node [[src](https://github.com/tskit-dev/tsconvert/blob/4ed3c0f9bbbb79c7c394b352c58f5386cfef2b90/tsconvert/newick.py#L184-L222)]
  - Each `label#Hi` occurrence becomes a separate node, and the ARG becomes a tree with duplicated hybrid nodes
- **Mapping an ARG to a tree sequence** (observed with a hand-built test): two segments map to the genome intervals [0, 1) and [1, 2)
  - **Reassortment node**: it becomes a node with one parent edge on [0, 1) and another on [1, 2)
  - **Drawing**: `draw_svg` then draws the two segment trees side by side with a shared time axis and minlex leaf order
  - **Observed**: a two-segment test with the trees `((A,B),(C,D))` and `(B,(A,(C,D)))` draws without error, and the unary node above B in the second tree appears as a circle on B's branch
- **Branch lengths must become times** (derived): TreeKnit trees have divergence branch lengths, so leaves need not be at one time. tskit needs node times with each parent strictly older
  - **Zero-length reassortment edges**: TreeKnit can produce a zero-length branch for a reassortment edge (`min(b1, b2) / 2` when one length is 0, `fn set_branch_lengths()` in [packages/treeknit-core/src/arg.rs#L492-L539](../../packages/treeknit-core/src/arg.rs#L492-L539))
  - tskit rejects such a branch, so it needs a small positive length
- **MCC colouring through CSS** (derived): with node IDs known, one rule per MCC such as `.n12 .node .edge, .n12 > .edge {stroke: ...}` colours the MCC in both segment trees at once
  - This works because the classes are shared across trees

## Ideas from the issue tracker and history

### Open requests

- **Document the summary and collapse features** ([#2372](https://github.com/tskit-dev/tskit/issues/2372)): the thread proposes several summary views
  - Collapse clades by population
  - Show pie charts of sample populations at internal nodes
  - Stop at a sample threshold with a box "X samples", or cut the tree at a lineage count
  - A click-to-toggle prototype in an SVG with embedded JavaScript is in the thread
- **Dark mode** ([#1150](https://github.com/tskit-dev/tskit/issues/1150)): replace `black` with `currentColor` in the stylesheet so the SVG follows the page colour
  - The test showed it works when the OS sets dark mode

### Declined

- **Styling and colour**:
  - **Colour parameters in `draw_svg`** ([#2330](https://github.com/tskit-dev/tskit/issues/2330)): CSS is the preferred route
  - **Edge span as colour** ([#1265](https://github.com/tskit-dev/tskit/issues/1265)): closed, solved by the `e<id>` class and a tutorial example
  - **Global style setting** ([#2362](https://github.com/tskit-dev/tskit/issues/2362)): closed without a change
- **Axes and size**:
  - **Width proportional to sample count** ([#2339](https://github.com/tskit-dev/tskit/issues/2339)): declined, see "Boxes and size"
  - **Kb and Mb axis labels** ([#2345](https://github.com/tskit-dev/tskit/issues/2345)): dropped because a window such as 1,000,123 to 1,000,345 would show identical rounded labels, and genome units are not always base pairs
  - **Round-number Y ticks** ([#1654](https://github.com/tskit-dev/tskit/issues/1654)): a `num_ticks` option after R's `pretty()` was proposed and closed for inactivity
  - **Crop the top of a tree with a small `max_time`** ([#1570](https://github.com/tskit-dev/tskit/issues/1570)): closed for inactivity
- **API and output formats**:
  - **`TreeSequence.draw` with simple options** ([#579](https://github.com/tskit-dev/tskit/issues/579), [#2628](https://github.com/tskit-dev/tskit/issues/2628)): moved to the idea of a separate `tsviz` package
  - **TikZ output** ([#791](https://github.com/tskit-dev/tskit/issues/791), [#798](https://github.com/tskit-dev/tskit/pull/798)): a working `draw_tikz` with mutations and tree sequences was moved to `tsviz`. Reasons: the size of viz code, testing needs a TeX engine, and keeping several viz APIs consistent
  - **Return node positions** ([#2699](https://github.com/tskit-dev/tskit/issues/2699)): closed with the internal `SvgTree` workaround

### Features found only in the history

- **Custom SVG symbols** ([#1289](https://github.com/tskit-dev/tskit/pull/1289), 2021): a draft let users swap node symbols for any SVG element, for example population pie charts
  - It was closed because the API depended on `svgwrite`
  - `svgwrite` itself was removed in 2025 ([#3115](https://github.com/tskit-dev/tskit/pull/3115)), but symbol swapping did not return
- **Original minlex prototype** ([#389](https://github.com/tskit-dev/tskit/issues/389)): a PyGame viewer in `tskit-dev/tsviz` compared sorted and unsorted leaf orders
- **Grouped SVG by element type**: the first SVG version grouped elements by type (all symbols, all labels). The nested tree-shaped groups replaced it in 0.3.0 ([#570](https://github.com/tskit-dev/tskit/pull/570))
- **Pedigree-style views** ([#621](https://github.com/tskit-dev/tskit/issues/621)): requests for one graph that merges all trees led to the networkx and graphviz recipes

## Open scientific problems

- **DAG layout with fixed times**: the maintainers found no algorithm that takes floating-point Y positions (node times) and chooses X positions that minimize line crossings in a DAG ([#621](https://github.com/tskit-dev/tskit/issues/621))
- **One or two recombination nodes**: msprime's two-node form keeps diamonds and the ARG likelihood, while the one-node form is the classical drawing
  - Converting between them is still open
- **Simplified graphs lose event times**: after simplification, the graph changes in three ways
  - Nodes can have several parents and several children
  - The SPR between trees is no longer explicit
  - The "ortho" graph style no longer applies
- **Collapsing polytomies with mixed lineages**: no meaningful summary exists for a large polytomy where nearly all lineages contain the samples of interest ([#3011](https://github.com/tskit-dev/tskit/issues/3011))
- **Ticks on a rank scale**: a date between two ranked node times has no defined position
  - Linear interpolation between ranks is one choice among several ([#2990](https://github.com/tskit-dev/tskit/issues/2990))

## Not covered by tskit

- **Two trees and ARGs**:
  - **Tanglegram and untangling**: only a tutorial recipe, with no crossing minimization
  - **Graph drawing of ARGs**: no built-in network view and no display of reticulations as edges
  - **Tree file import**: no Newick, NEXUS, or extended Newick reader
- **Display**:
  - **Interaction**: no zoom, pan, selection, collapse on click, or tooltips beyond SVG `<title>`
  - **Other layouts**: no radial, circular, unrooted, or horizontal SVG layout without CSS tricks
  - **Label fitting**: no measurement, thinning, or truncation of labels
  - **Legends and colour scales**: none built in. Users draw legends in the `preamble`
- **Export**:
  - **Raster or PDF export**: none

## Method and limits

- **Read**: the code and tutorials below
  - `python/tskit/drawing.py` in full, and the drawing methods and docstrings of `python/tskit/trees.py`
  - The outline of `python/tests/test_drawing.py`
  - The drawing entries of `python/CHANGELOG.rst` and the commit log of `drawing.py`
  - The tutorials `viz.md` and `args.md` in full, and `tsconvert/newick.py`
- **GitHub**: the list of all 1331 issues and 1906 pull requests, filtered by title and by the "Visualisation" label
  - 27 issues and pull requests were read with their comments (truncated to the first 700 characters of each comment)
  - GitHub discussions were not read, including the ladderization discussion that the tutorial links ([#3160](https://github.com/tskit-dev/tskit/discussions/3160))
- **Trial**: tskit 1.0.3 and msprime 1.4.4 from PyPI in `python:3.12-slim`, with only `/tmp/feat-survey-tskit/` mounted
  - **Inputs**: a simplified msprime ARG with mutations (4 samples, 6 trees), a full ARG with two recombination nodes, a hand-built two-segment tree sequence, a polytomy with a unary node, a tree with several roots, a zero-length branch attempt, long labels, and two large simulations for timing
  - **Option sets**: default, treewise with rank scale and `order="tree"`, log scale with a right axis and root branches, `max_num_trees`, and `x_lim`
  - **Inspection**: the SVGs were rendered with `rsvg-convert` and inspected as PNG and as text
- **Not run**: `tskit_arg_visualizer`, networkx, and graphviz, because the tutorials document them and they are separate projects
  - The tutorial tanglegram and SPR animation were read but not run
- **Limits**: the trial used the 1.0.3 release. Its `drawing.py` is identical to the surveyed commit, but `trees.py` has small later changes outside the drawing methods
  - No fetch failed
