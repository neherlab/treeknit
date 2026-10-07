# Dendroscope feature survey: tree rendering, semantics, and edge cases

This report describes Dendroscope 3, a Java desktop program for rooted phylogenetic trees and rooted phylogenetic networks, as an idea inventory for tree, tanglegram, network, and ARG display. It records what Dendroscope does, how it does it, which scientific conventions it assumes, which inputs break it, and which problems its authors left open. It does not compare Dendroscope with `packages/web`.

Dendroscope is licensed GPL-3.0, and so is the `jloda` library that holds its graph, Newick, and drawing code. Their behavior and design may be studied, but copying their code needs approval (see the project rules).

- **Source**: [husonlab/dendroscope3](https://github.com/husonlab/dendroscope3) at commit `c2d3555` (2025-04-28, version 3.8.11). Source links point to this commit
  - **History**: 156 commits since the repository was created in 2015. The program itself dates from 2012
- **Library**: [husonlab/jloda](https://github.com/husonlab/jloda) at commit `27ec9d8` (2024-01-26)
  - **Content**: Dendroscope gets its graph classes, the Newick parser and writer, the graph view, the magnifier, the label thinning, and the image export from `jloda`
  - **Pairing**: a successful build confirms the pairing of the two commits (see "Method and limits")
- **Other sources**:
  - **Manual**: the user manual in [`tex/manual/manual.tex`](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex)
  - **Issues**: all 22 issues with their comments. The repository has no pull requests
  - **History**: the commit history
  - **Papers**: on Dendroscope 3 ([Huson and Scornavacca 2012](https://doi.org/10.1093/sysbio/sys062)), on the tanglegram method ([Scornavacca, Zickmann, and Huson 2011](https://doi.org/10.1093/bioinformatics/btr210)), and on hybridization networks ([Albrecht et al. 2012](https://doi.org/10.1093/bioinformatics/btr618))
- **Evidence labels**:
  - **Observed**: seen in a trial run of Dendroscope built from the surveyed source, in a throwaway Docker container under a virtual X display
  - **Derived**: read from the code without running it
  - **Conflicts**: where the manual and the code disagree, the code wins and the difference is noted
- **Shape of the code**: a Java Swing application of about 370 files and 76,000 lines. Every menu item is a command object with a text syntax, so the command language and the menus are one surface. The main parts:
  - **Window**: a grid of tree panels, each a `jloda` graph view ([`window/TreeGrid.java`](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeGrid.java), [`window/TreeViewer.java`](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeViewer.java))
  - **Layouts**: one drawer class per layout in [`drawer/`](https://github.com/husonlab/dendroscope3/tree/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer)
  - **Network embedding and tanglegrams**: [`embed/EmbeddingOptimizerNNet.java`](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java), [`tanglegram/TanglegramUtils.java`](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/tanglegram/TanglegramUtils.java), and the connector lines in [`window/Connector.java`](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/Connector.java)
  - **Network construction**: the Autumn hybridization algorithm in [`autumn/`](https://github.com/husonlab/dendroscope3/tree/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/autumn), and cluster, galled, and level-k networks in [`algorithms/`](https://github.com/husonlab/dendroscope3/tree/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/algorithms) and [`consensus/`](https://github.com/husonlab/dendroscope3/tree/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/consensus)
  - **Parsing**: extended Newick in [`jloda/phylo/PhyloTree.java`](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/phylo/PhyloTree.java)

## Summary

### Tanglegrams

Details are in "Two trees and tanglegrams".

- **Method**: a NeighborNet circular ordering of all taxa gives a target leaf order, and each tree is then rotated bottom-up to reduce crossings against the other tree
  - **Order source**: the clusters of both trees
  - **Rounds**: five alternating rounds
  - **Scope**: the same method works for networks, for multifurcating trees, and for different taxon sets
- **Quality** (observed): on two test pairs the result had 17% to 20% more crossings than a simple alternating one-sided optimization found in seconds
  - 27 against 23, and 723 against 619
- **Speed** (observed): 67 ms for 17 leaves, 17 s for 200 leaves, and more than 300 s for 1,000 leaves
  - **Cause**: a leaf-set recomputation inside a recursion
- **Connectors**: straight light grey lines connect every pair of nodes with the same label, internal nodes included
  - **Start point**: a line starts at the edge of the label box
  - **Selection**: both connected nodes selected make the line pink
- **Export defects** (observed): in exported images the connectors are displaced by about one leaf row
  - In PDF export they are drawn in the wrong panel

### Networks

Details are in "Networks and reticulations".

- **Layout on the LSA tree**: each reticulation node is placed under its lowest stable ancestor (LSA), and that tree decides the leaf order and the vertical positions
- **Reticulate edges**: drawn as blue curves
  - **Branch lengths**: discarded at parse time
  - **Phylogram position**: a reticulation node is placed half an average edge length to the right of its rightmost parent
- **Transfer edges**: the `##` prefix marks the one tree edge into a transfer node, and the other parental edges are then drawn as straight arrows
- **Hybridization networks**: the Autumn algorithm computes all minimum hybridization networks of two multifurcating trees with overlapping taxa
  - **Source tree labels**: each reticulate edge carries the number of the input tree it comes from, an idea close to TreeKnit's segment annotation
  - **Removed algorithms**: the binary-tree algorithm of Albrecht et al. (2012) and the Hybroscale algorithm were removed from the code in 2021, but the manual still lists them

### TreeKnit files

Details are in "Interaction with TreeKnit output".

- **ARG as Newick** (observed): TreeKnit's ARG file fails to load as Newick
  - **Cause**: an annotation before a branch length is a syntax error for the parser
- **ARG in NEXUS** (observed): wrapped in a NEXUS trees block, the ARG loads
  - All annotations and all reticulate branch lengths are lost

### Status and scale

- **Status**: the last code change is from 2023
  - The 2025 commit only changes the version string and the build script
  - 19 of the 22 issues are open, most without an answer
- **Large trees**: a subtree with at least 250 leaves whose drawing is smaller than about one pixel per child is drawn as a filled polygon
  - Overlapping labels are hidden by default
  - The manual claims trees with hundreds of thousands of taxa

## Layouts

### Layout modes

The Layout menu and the command `set drawer=<name>` select one of eight layouts ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L424-L447)).

- **Rectangular phylogram and cladogram**: one drawer class serves both. A new panel starts as a rectangular cladogram [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeViewer.java#L111-L114)]
- **Slanted cladogram**: a parent sits at `x = -0.5 * (yMax - yMin)` of its leaf span, so edges form triangles [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerAngled.java#L104-L157)]
  - **Networks**: the layout shows the alert "This visualization is not well-defined for networks" once per session
- **Circular phylogram and cladogram, inner circular cladogram**: leaves on a circle, with the leaves on the inside for the inner variant
- **Radial phylogram and cladogram**: unrooted-style drawings of the rooted tree
  - **Rotation**: Shift and the left or right arrow key rotate circular and radial drawings ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L676-L697))
- **Last layout persists**: the drawer name is stored as a global program property on every change, so the next session starts with the last layout used [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeViewer.java#L1033-L1036)]
- **Auxiliary parameter**: `auxiliaryparameter change={increment|decrement}` adjusts a per-layout number
  - **Network offset**: the rectangular drawer sets it to 5 as the "percent offset in phylogram view of network" [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerParallel.java#L48-L52)]
  - **Ignored**: the phylogram code ignores it and uses a fixed 50% (see "Defects")

### Rectangular coordinates

- **Leaf rows**: leaves get the rows 1, 2, 3, and so on, in the order of a traversal of the LSA tree (see "LSA tree") [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerBase.java#L917-L928)]
- **Parent position**: a parent sits at the **midpoint of its first and last child** [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerBase.java#L935-L952)]
  - **Middle children**: the position ignores them, so it differs from the mean of all children
  - **Observed**: with the polytomy `(A:1,B:1,C:1,D:2)` the parent is drawn halfway between A and D
- **Network row spacing**: in a network, real leaves keep a spacing of exactly 1 [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerBase.java#L954-L981)]
  - **Reticulation nodes**: those that act as leaves of the LSA tree get fractional rows between their neighbors, at steps of `1 / (count + 1)`
- **Cladogram depth**: the horizontal position is minus the level, so all leaves align at the right [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerBase.java#L983-L1027)]
  - **Level**: the largest number of edges from the node down to a leaf
  - **Special cases**: a transfer edge adds no level, and the LSA children are counted as children
- **Phylogram depth**: breadth-first from the root, a node with one parent sits at the parent's position plus the edge length [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerParallel.java#L190-L262)]
  - A node is put back in the queue until its parent has a position
- **Fit margins**: "zoom to fit" leaves 300 px of width and 200 px of height free for labels, which makes small trees look small in exports (observed) [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeViewer.java#L1041-L1067)]

### Child order

- **Input order by default**: a tree without reticulations keeps the child order of the file [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L55-L72)]
- **Ladderize left, right, random**: the code sorts children by subtree **height**, the largest number of edges to a leaf [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeViewer.java#L763-L803)]
  - **Manual**: the manual says the largest clades go to the top or bottom, which describes a sort by clade size ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L435-L437))
- **Swap, rotate, reorder subtrees**: three commands change the child order of the selected nodes ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L265-L270))
  - **Swap**: reverses the children
  - **Rotate**: moves the children cyclically
  - **Reorder Subtrees**: opens a drag-and-drop list of the children of one node
- **Flip horizontally**: `set hflip=true` mirrors the drawing, and the labels move to the other side. The tanglegram uses this for the right tree

## Edges, nodes, and labels

### Colour and format

- **Formatting is manual and per element**: colour, width, font, label colour, and label fill colour apply to the selected nodes and edges (`set color=r g b`, `set edgewidth=`, `set labelcolor=`) ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L939-L965))
  - **No attribute colouring**: there is no colouring by an attribute
- **Edge shapes**: straight, curved, or angular per edge, set in the Format panel or with `set edgeshape=` for the selected edges ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L526-L547))
- **Node shapes**: none (the default), rectangle, or oval, with size, line colour, and fill colour [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeViewer.java#L102-L111)]

### Labels

- **Sparse labels** (default on): a label is drawn only when it does not overlap the first label drawn or any of the last 100 labels drawn [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerBase.java#L279-L284)] [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/swing/graphview/LabelOverlapAvoider.java#L66-L124)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeGrid.java#L181)]
  - **Selected nodes**: they always get their label
  - **Observed**: on 200 leaves about one label in eight is drawn
- **Edge labels and weights**: two switches show text on edges
  - **Edge labels**: `show edgelabels=true` shows the text of `[...]` edge labels
  - **Edge weights**: `show edgeweights=true` writes each branch length as an edge label
  - **Observed**: on short edges the length labels overlap the node labels
- **Radial labels**: leaf labels rotate to follow the direction of their edge
- **Movable labels**: labels can be dragged [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerParallel.java#L327-L383)]
  - "Reposition Labels" resets them, and dragged labels keep their place on relayout

### Images and scale bar

- **Taxon images**: "Load Taxon Images" matches image file names to taxon names and shows each image next to its leaf, at north, south, east, west, or radial positions ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L333-L339))
- **Scale bar**: a short bar with its length in the top-left corner (observed)

## Large trees and navigation

- **Proxy shapes**: for a subtree with at least 250 leaves, the drawer stores its bounding box [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerParallel.java#L265-L321)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerParallel.java#L385-L406)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerBase.java#L196-L217)]
  - **Off screen**: when the box is off screen, the subtree is skipped
  - **Small box**: when the box is less than 2 px wide and 2 px high, or less than about 1 px per child, the subtree is drawn as one filled polygon in the colour of its parent edge
  - **Users see branches disappear**: a user with a 10,000-taxon tree reported vanishing branches during vertical zoom. The author explained the optimization and offered a switch, which was never added ([#1](https://github.com/husonlab/dendroscope3/issues/1))
  - **Threshold setting**: `set approxthreshold=<int>` changes the 250-leaf limit ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L949))
- **Zoom and scroll**: the mouse wheel zooms around the pointer, and Shift with the wheel zooms horizontally only. Arrow keys scroll, and Shift with arrow keys stretches one axis ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L676-L697))
- **Magnifier**: a band over rectangular and slanted layouts, a disk over circular and radial ones [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/swing/graphview/Magnifier.java#L35-L43)] [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/swing/graphview/Magnifier.java#L310-L334)]
  - **Mapping**: a point at distance `d` from the centre moves to `r * d / (d + m) * (1 + m / r)`, with `m = 0.5 * r * (1 - δ) / (δ - 0.5)`
  - **Displacement**: `δ` (default 0.75) is the new distance of a point that was at half the radius
  - **Radius**: the default radius is 90% of the view
  - **Magnify all mode**: maps the whole tree into the magnifier
- **Collapse**: a collapsed node is drawn as a grey outline polygon from the node to the extreme leaves below it [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerParallel.java#L408-L422)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/CollapsedShape.java#L54-L70)]
  - **Commands**: collapse the selected nodes, their complement, or all nodes at a given distance from the root (`collapse level=<n>`)
- **Click selection levels**: each extra click selects a larger part [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeGrid.java#L314-L335)]
  - **One click**: selects a node
  - **Double click**: selects the part below it that is reachable without crossing reticulate edges
  - **Triple click**: selects the whole subnetwork, or inverts the selection in a tree

## Multiple trees in a grid

- **Panel grid**: a document holds any number of trees, and the window shows them in a grid of panels whose size follows the number of trees [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/MultiViewer.java#L776-L793)]
  - Up to 3 trees: one row
  - 4 trees: 2x2
  - Up to 6 trees: 2x3
  - Up to 8 trees: 2x4
  - 9 trees: 3x3
  - Larger sets: at most 7x7
- **Paging**: a scroll bar and the arrow keys page through the trees. Left and right go to the previous and next tree, up and down to the first and last [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeGrid.java#L81-L120)]
- **Panel selection**: a click selects a panel and colours its background light green [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeGrid.java#L597-L611)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeGrid.java#L835-L846)]
  - **Scope of commands**: commands such as tanglegram, consensus, and align act on the selected panels, or on all panels when none is selected
- **Linked selection**: when several panels are selected, selecting nodes in one panel selects the nodes with the same labels in the other selected panels [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeGrid.java#L255-L285)]
- **Select from previous window**: copies the selection of labels from the previous window to the current one ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L292))

## Two trees and tanglegrams

### Workflow

- **Preconditions**: the command needs exactly two selected panels, or a grid of two panels [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/commands/TanglegramNeighborNetCommand.java#L115-L117)]
  - **Steps from the author** ([#15](https://github.com/husonlab/dendroscope3/issues/15)):
    - Put both trees in one document
    - Set a 1x2 grid
    - Choose Algorithms > Tanglegram
- **Result in a new window**: the two trees are copied into a new document with a 1x2 grid and the title suffix `-tanglegram` [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/commands/TanglegramNeighborNetCommand.java#L188-L254)]
  - **Layout**: both are drawn as rectangular cladograms
  - **Flip**: the second tree is flipped so that its leaves face the first
- **Command-line mode reuses the document**: in `-g` mode the current document is emptied and refilled, so the grid must be set to `1 x 2` before the command (observed)
- **Crossing count**: the number of crossings is printed to the console as "The minimal crossing number found is 27" (observed) [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/tanglegram/TanglegramUtils.java#L45-L70)]
  - **Definition**: the number of leaf pairs whose order differs between the two sides, counted in O(n²)

### The NN-tanglegram algorithm

The paper ([Scornavacca, Zickmann, and Huson 2011](https://doi.org/10.1093/bioinformatics/btr210)) defines a tanglegram of two rooted networks as an embedding of the forest that remains after the reticulate edges are removed, with a leaf order that does not interleave the trees of the forest. The code does the following.

#### Target order

- **Formal outgroup**: each tree gets a new root with the old root and a dummy leaf `rho****` as children [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L97-L118)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L517-L539)]
  - The dummy leaf later cuts the circular order into a linear one
- **Distance matrix**: all hardwired clusters of both networks become splits against the full taxon set [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/tanglegram/TanglegramUtils.java#L225-L289)]
  - **Distance**: the number of splits that separate two taxa, summed over the two networks
  - **Shared splits**: a split present in both networks counts twice
- **Different taxon sets**: when the trees differ in taxa, both are first restricted to the common taxa [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L634-L733)]
  - The circular order is then restricted to the common taxa too
  - A fix in 2022 corrected the index of this restriction ([`154283d`](https://github.com/husonlab/dendroscope3/commit/154283d))
- **Circular order**: NeighborNet on the matrix gives a circular order of all taxa [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L711-L715)]
  - **Guarantee from the paper**: if a drawing with zero crossings exists, the method finds it (Theorem 3.5 of the paper). Observed: two trees that differ by one moved leaf gave 0 crossings
  - **Two distance variants**: the paper reports that a shortest-path distance clearly beats the split distance on networks. The code has both, but the tanglegram command always uses the split distance [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/commands/TanglegramNeighborNetCommand.java#L155-L156)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L127-L133)]

#### Rotation and embedding

- **Forest decomposition**: for each network, the subtree below each reticulation node and the subtree below the root, all without reticulate edges, form a forest [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L209-L241)]
- **Bottom-up rotation**: in each forest tree, the children of every node are swapped pairwise when the swap lowers the crossing count against the target order [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/tanglegram/TanglegramUtils.java#L393-L519)]
  - **Binary nodes**: one test
  - **Polytomies**: adjacent swaps repeat until no swap helps or an order repeats
- **Leaf insertion for networks**: the leaf orders of the forest trees are merged one leaf at a time [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L243-L372)]
  - Each leaf goes to the position with the fewest crossings that does not interleave the forest trees already placed
- **Alternating rounds**: the code always runs five rounds and keeps the last result (see "Defects") [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L194-L391)]
  - **Round one**: optimizes both trees against the NeighborNet order
  - **Later rounds**: optimize both trees against the order of the first or the second tree in turn
  - **Paper**: recommends one swap round ("+1S") and also tests five ("+5S")
- **Applying the order**: an embedder takes the final leaf order and fixes the child order of the network, including LSA children, so that the drawn leaves follow it [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbedderForOrderPrescribedNetwork.java#L39-L107)]

#### Observed quality

The comparison method is a throwaway alternating one-sided optimization: each node takes the better of its two child orders against the other tree, repeated from random starts.

- **17 leaves with 4 moved taxa**: Dendroscope 27 crossings, the simple method 23
- **200 leaves with 10 swapped leaf pairs**: Dendroscope 723 crossings, the simple method 619 after 20 restarts

### Align taxa for more than two trees

- **Align Taxa**: for any number of selected trees, the circular order of all their clusters is computed once. Then each tree's children are sorted by the smallest position of the taxa below them. No crossing count is involved [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/commands/AlignTaxaCommand.java#L80-L108)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L463-L514)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L558-L613)]
- **Connect Taxa**: `connect what=taxa panels=<ids>` adds connectors between every pair of listed panels [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/commands/ConnectTaxaCommand.java#L80-L103)]
  - A grid of trees can therefore show connectors between neighbors and also across the grid
  - "Disconnect All" removes them

### Connector lines

#### Matching

- **Matching rule**: two nodes are connected when their trimmed labels are equal, **internal nodes included** [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeGrid.java#L758-L790)]
  - **Comparison**: every node of the first panel is compared with every node of the second panel
  - **Repeated labels**: a label that occurs several times gives several connectors
  - **Selection**: when nodes are selected, only selected nodes are connected
- **Internal labels in tanglegrams**: a user asked how to stop lines between inner nodes ([#10](https://github.com/husonlab/dendroscope3/issues/10), no answer)
  - **Observed**: two trees with internal labels `NODE_0` to `NODE_3` got connectors between equal internal labels, root to root included, mixed with the leaf connectors
- **Support values are skipped**: when internal labels are declared as edge labels (support values), numeric internal labels are not connected unless selected

#### Drawing

- **Line style**: a straight line in light grey (`#C0C0C0`, observed in SVG), 1 px wide [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/Connector.java#L58-L156)]
  - **Start point**: the line starts just outside the label box on the side that faces the other panel, so it does not cross the label text
- **Clipping by visibility**: a connector is drawn only when both end points lie inside the visible area of their panels (with a 5 px inset) [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/Connector.java#L77-L127)]
  - **Zoom and scroll**: when the user zooms or scrolls one panel, connectors to leaves outside the view disappear instead of running to the border
- **Selection and format**: a connector whose two end nodes are both selected is drawn pink [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/Connector.java#L120-L127)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/FormatterHelper.java#L314-L327)]
  - Colour and width of such connectors change together with the edge format of the selection
- **Paint layer**: connectors are painted over the whole grid in the coordinates of the window, after the panels [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/Connector.java#L52-L63)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeGrid.java#L675-L681)]
  - **Coordinate correction**: coordinates go through screen coordinates with a hard-coded correction of 20 pixels, which the source comment calls "off by this amount" (see "Defects")

#### Storage

- **Saved in NeXML**: connectors are stored in an extra tree block named `InterTreeConnectors`, by tree name and node label [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/io/nexml/ConvertTreeDataToNexmlDoc.java#L150-L156)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/core/Connectors.java#L78-L105)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeViewer.java#L1343-L1350)]
  - **Reload**: each label maps to the last node with that label

### Tanglegrams of networks

- **Networks on either side**: the same command accepts networks
  - **Observed** with a one-reticulation network against a tree with an extra taxon: 0 crossings, the reticulation drawn as two blue curves, and the extra taxon without a connector
- **Reticulation crossings**: the paper defines the optimum as fewest connector crossings first, then fewest crossings of reticulate edges
  - The code minimizes connector crossings only, and the paper leaves reticulation crossings to later work

## Networks and reticulations

### Extended Newick parsing

Dendroscope follows the extended Newick of [Cardona et al. (2008)](https://doi.org/10.1186/1471-2105-9-532): every node whose label ends in `#H1` is the same node ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L733-L757)).

- **Hybrid marker**: the text after the **last** `#` counts as a hybrid ID when it starts with `H`, `L`, or `R` in either case [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/phylo/PhyloTreeNetworkIOUtils.java#L27-L80)]
  - **Node label**: the text before the **first** `#`
  - **Examples**: `Sample#3` stays an ordinary label (observed), but a label such as `flu#H3N2` becomes a hybrid node with ID `H3N2` (derived)
- **Merge of occurrences**: all occurrences with the same ID are merged into one node [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/phylo/PhyloTree.java#L736-L815)]
  - Children and parents of the other occurrences move to the first one
  - Different labels of the occurrences are joined with a comma
- **Reticulate branch lengths are dropped**: every edge into a hybrid occurrence gets length 0, whatever the file says [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/phylo/PhyloTree.java#L679-L707)]
  - **Observed**: input lengths 0.5 and 0.2 on the two hybrid edges are exported as `:0`
- **Transfer acceptor `##`**: an occurrence written `##H1` marks its incoming edge as the tree edge of a transfer [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/phylo/PhyloTree.java#L689-L701)] [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/phylo/PhyloTree.java#L796-L812)]
  - **Tree edge**: gets a small positive length, 0.000001 if none is given
  - **Other parental edges**: get the length -1, which marks them as transfer edges
- **Rich Newick fields off**: the second and third `:` fields (support and probability) are parsed only in SplitsTree 6 mode. Dendroscope reads only the branch length [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/phylo/PhyloTree.java#L38)] [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/phylo/PhyloTree.java#L640-L677)]
- **Hint message**: any input that contains `#` prints "Input contains the special character '#', will try to interpret as extended-Newick" [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/io/Newick.java#L81-L84)]

### LSA tree

- **Definition**: the lowest stable ancestor (LSA) of a reticulation node is the lowest node that lies on every path from the root to it [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/consensus/LSATree.java#L247-L275)]
  - **LSA children**: Dendroscope builds a map from each node to its LSA children, which are its tree children plus the reticulation nodes whose LSA it is
- **Layout on the LSA tree**: all drawers traverse the LSA children for leaf order and vertical position [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerParallel.java#L150-L155)]
  - A reticulation node is therefore drawn in the subtree of its LSA, between the subtrees of its parents when the embedding is good
- **Debug views**: the hidden program property `showlsa` draws the LSA edges in green, and `showids` appends node IDs to labels [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerBase.java#L261-L270)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerParallel.java#L88-L99)]

### Network embedding

- **Embedding algorithms**: `set layouter=` selects how the child order of a network is optimized [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/LayoutOptimizerManager.java#L28-L63)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/MultiViewer.java#L73)]
  - `Unoptimized`
  - `Algorithm2008`: Kloepper and Huson 2008
  - `Algorithm2009`: Huson 2009
  - `Algorithm2010`: the NeighborNet method, default
  - `Algorithm2010Dist`
  - **Manual**: the manual also lists `AlgorithmLSA`, which the code does not have ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L943-L944))
- **Single network with the default**: the clusters of the network give a circular order, and children are sorted by the first taxon position below them, the same as "Align Taxa" [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L55-L72)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L457-L460)]

### Drawing reticulate edges

#### Shape and position

- **Colour and shape**: reticulate edges with length 0 or less are drawn **blue** as curves [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeViewer.java#L895-L918)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerParallel.java#L157-L183)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerRadial.java#L376-L393)]
  - **Rectangular layouts**: quadratic curves from the parent's position to the reticulation node
  - **Radial and circular layouts**: curves too
  - **Observed**: in all three layouts
- **Transfer edges**: an edge with length -1 is drawn straight and with an arrow head [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeViewer.java#L904-L906)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerParallel.java#L172-L177)]
- **Reticulation position in the phylogram**: the node is placed at the largest horizontal position of its parents plus 50% of the mean length of all edges [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerParallel.java#L191-L247)]
  - Its own branch lengths play no role, so the time of a reticulation is not shown

#### Editing and selection

- **Edges become reticulate when edited**: when a user draws a second edge into a node, both edges become blue reticulate edges with length 0 [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeViewer.java#L125-L153)]
  - Deleting one makes the other a tree edge with length 1
- **Selection of reticulate edges**: "Select Special" selects all edges into reticulation nodes. "Select subpart" selects what is reachable from the selected nodes without crossing reticulate edges ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L301), [manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L913))

#### Hidden and dormant views

- **Hidden weight view**: with the program property `scaleconfidence`, a reticulate edge gets the width `10 * weight` and a matching grey value, a way to show how much a reticulation is supported [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeViewer.java#L907-L914)]
- **Dormant transfer view**: a class `TransferVisualization` exists but is called behind `if (false)` [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerBase.java#L131-L133)]

## Networks computed from trees

### Hybridization networks (Autumn)

- **Input**: two rooted trees without reticulations, multifurcating allowed, with overlapping taxon sets [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/commands/autumn/ComputeHybridizationNetworksCommand.java#L116-L121)]
- **Steps** [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/autumn/hybridnetwork/ComputeHybridizationNetwork.java#L63-L198)]:
  - **Upper bound**: the hybridization number of a cluster network gives the first bound (observed: "Computing upper bound using cluster network: 3")
  - **Taxa in one tree only**: removed before the search and reattached to every result network afterward
  - **Common refinement**: multifurcations are refined against the other tree
  - **Search**: a recursive search with subtree reduction, cluster reduction, and a lookup table. It runs in parallel threads (observed: 16 worker threads)
  - **Duplicates**: networks with the same agreement forest are removed
- **Output**: all minimum networks go into a new document, one per panel, with a message "Hybridization number: 1, Number of networks: 1" (observed) [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/commands/autumn/ComputeHybridizationNetworksCommand.java#L61-L89)]
- **Edges tell their source tree**: each reticulate edge gets the label `1` or `2`, the number of the input tree that contains that parent edge [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/autumn/PostProcess.java#L80-L99)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/autumn/hybridnetwork/AddHybridNode.java#L83)]
  - **Observed** in the Newick export: `(X)#H1:0[1]` and `#H1:0[2]`
- **No branch lengths**: result networks are topological [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/autumn/PostProcess.java#L39-L64)]
  - Reticulate edges get length 0, and a root with several children gets an extra root edge
- **Related commands**:
  - **Hybridization number**: computes the number only
  - **Reroot By Hybridization Number**: a rooting that minimizes the number
  - **`orderNetworks`**: sorts or filters result networks by whether named taxa sit below a reticulation [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/commands/autumn/OrderNetworksByHybridTaxa.java#L255-L258)]

### Consensus networks

- **Methods**: consensus trees and consensus networks [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/commands/consensus/ConsensusCommand.java#L42-L66)]
  - **Trees**: strict, loose, majority, LSA, primordial, and "distortion 1"
  - **Cluster network**: all clusters, hardwired
  - **Galled network**: softwired with topological limits
  - **Level-k network**: minimum level
- **Threshold**: a cluster enters the network when at least `threshold` percent of the trees contain it (dialog default 20%) [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/commands/consensus/ClusterNetworkConsensusCommand.java#L46-L52)]
- **Partial taxon sets**: the Z-closure method merges trees with different taxa before the network is built [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/consensus/ComputeNetworkConsensus.java#L77-L162)]
  - When the clusters are compatible, a tree is built instead of a network
- **Multi-labeled trees**: "Network for Multi-Labeled Tree" turns a tree with repeated labels into a network with each label once ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L384-L391))
  - By clusters
  - By the exact method of Huber and others
  - By level-k

### Comparison measures

The Advanced Algorithms menu computes the following for two trees or networks ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L405-L421)). Results are printed as text only.

- **Distances**:
  - Hardwired cluster
  - Softwired cluster
  - Displayed trees
  - Tripartition
  - Nested labels
  - Path multiplicity
- **DTL reconciliation**: duplication, transfer, and loss, for two binary trees, with cost options
- **Simplistic network**: the "simplistic" network from triplets

## Input formats

- **Newick**: one tree per line ending in `;`. Lines without `;` are joined [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/io/Newick.java#L67-L99)]
  - **Edge labels**: one `[...]` block directly after the label or the branch length becomes the edge label. A `[` inside it is an error [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/phylo/PhyloTree.java#L709-L731)]
  - **Quotes**: a `'` toggles quoting and is dropped. Doubled quotes are not unescaped: `'O''Brien'` becomes `OBrien` (observed) [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/phylo/PhyloTree.java#L587-L604)]
  - **Duplicate labels allowed**: multi-labeled trees are accepted by default, which the multi-labeled network methods need [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/phylo/PhyloTree.java#L43)]
- **NEXUS**: the first trees block, with `translate`, `properties rooted=`, and the PAUP `tree *` form [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/io/Nexus.java#L78-L193)]
  - Other blocks are skipped
  - **Comments**: the NEXUS tokenizer removes `[...]` comments, so annotations are lost there
- **NeXML**: Dendroscope's project format, with formatting, drawer, and connectors
  - **DendroPy colours**: node colour metadata written by DendroPy fails with "Command failed: 1" ([#11](https://github.com/husonlab/dendroscope3/issues/11), open)
  - **Version 3.8.7**: saved files failed with "color is null", fixed in 3.8.8 ([#21](https://github.com/husonlab/dendroscope3/issues/21))
- **Old `.dendro` files**: read for compatibility ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L718-L720))
- **Internal label dialog**: when the first tree has any internal node label, the GUI asks how to treat internal labels [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/commands/OpenFileCommand.java#L155-L180)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/util/SupportValueUtils.java#L37-L46)]
  - **Choices**: as node labels, as edge labels (support values), or delete them
  - **Manual**: the manual says the dialog appears only when all internal labels are numbers ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L222-L226))

## Export

- **Images**: PNG, GIF, JPG, BMP, SVG, PDF, and EPS [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/commands/ExportImageCommand.java#L82-L145)]
  - **EPS**: works in code but is missing from the syntax text
  - **Options**: `visibleonly` (only the visible region), `textasshapes`, and `replace`
  - **Size**: the image has the size of the window panel. Observed: 1300x804 px for one panel and 800x502 px for a 1x2 grid in the default command-line window
  - **SVG content** (observed): every edge segment is a `<line>`, every label a `<text>` with a font family, and the panel titles such as `[2]Tree2` are part of the image
  - **Grid export**: with several panels, PDF and EPS use a separate painting path that paints each panel and the connectors again. The other formats paint the live grid [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeGrid.java#L874-L928)]
- **Trees**: `save format={newick|nexus|nexml}` [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/io/Newick.java#L107-L117)] [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/phylo/PhyloTreeNetworkIOUtils.java#L82-L92)]
  - **Content**: Newick and NEXUS keep topology, labels, edge labels in brackets, and branch lengths
  - **Collapsed subtrees**: hidden on write
  - **Networks**: reticulations are written as `#H<n>`, and transfer acceptor edges as `##H<n>`
- **Print and clipboard**: print, copy image, and copy of the trees as text

## Command language and batch mode

- **One command per menu item**: every action has a text syntax such as `set drawer=RadialPhylogram;` or `compute tanglegram method=nnet;`. Window > Command Input runs commands interactively, with an "Apply to Every Tree in File" button ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L804-L1019))
- **Command-line mode** `-g`: runs the command given with `-x`, then reads commands from the file of `-c` or from standard input with the prompt `DENDROSCOPE> ` [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/main/Dendroscope.java#L84-L233)]
  - **Still needs a display**: the mode creates Swing windows, so a server needs `xvfb-run` (manual) and the Java desktop libraries. Observed: without `libXtst` the program fails at start-up ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L1045-L1057))
  - **Quiet start** (added in 3.8.2 after [#18](https://github.com/husonlab/dendroscope3/issues/18) and [#12](https://github.com/husonlab/dendroscope3/issues/12)):
    - `+s` hides the splash screen
    - `+w` hides the message window
    - `-q` hides the quit confirmation
  - **One document**: commands that compute new trees (tanglegram, consensus, hybridization) refill the current document instead of opening a window
- **Apply to all trees**: `apply-all-begin <command>; ... apply-all-end;` runs commands on each tree of the file [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/commands/ApplyToAllCommand.java#L80-L81)]
- **Use as an API**: a user asked for an API to render trees in a standard way without the GUI ([#17](https://github.com/husonlab/dendroscope3/issues/17), no answer). The command language with `-g` is the only route
- **Colour by script**: a script can select taxa with `find searchtext=<regex> target=Nodes all=true regex=true;` and then colour them with `set color=r g b;` ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L927-L932))
  - This is the only way to colour groups such as MCCs

## Search and editing

- **Find and replace**: a toolbar for node or edge labels ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L550-L608))
  - **Options**: case sensitivity, whole words, Java regular expressions, and scope (global or selection)
  - **Actions**: find first, next, or all
  - **Collapsed subtrees**: a hit inside a collapsed subtree selects the collapsed node
- **Edit topology**: "Unlock Edge Lengths" lets the user drag nodes and internal edge points
  - Taxa can be deleted
  - Subtrees can be extracted below a node, induced by a selection, or rooted at the LSA of a selection
- **Contract edges**: by support value below a threshold (dialog default 75), or the selected edges [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/commands/ContractEdgeBySupportValueCommand.java#L133-L138)]

## Scientific semantics

### Branch length and time

- **Branch lengths are the only time source**: positions come from edge lengths only. There is no time axis, no date annotation, and no age display. The scale bar shows a length unit
- **Missing lengths become 1**: an edge without a length has the default length 1 [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/phylo/PhyloGraph.java#L34)] [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/phylo/PhyloGraph.java#L141-L146)]
  - **Observed**: in `(G,H)` both edges are drawn as long as the sibling edges of length 1, with no hint that the lengths are missing
  - **Mixed trees**: a tree with some missing lengths is still a phylogram, unlike viewers that switch to a cladogram
- **Negative lengths become 0**: the parser clamps every length to at least 0, silently [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/phylo/PhyloTree.java#L654)]
  - **Observed**: `J:-0.5` is drawn at its parent and exported as `J:0`
- **Zero-length edges are kept**: no collapsing. Observed: the tip `E:0` is drawn on its parent node, and its label overlaps the parent edge
- **Reticulations have no time**: see "Drawing reticulate edges"

### Rooting and support values

- **Rooted by construction**: every input is treated as rooted at the outermost node
- **Reroot on an edge**: the new root is placed so that the mean distance to the leaves is equal on both sides [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/util/RerootingUtils.java#L78-L94)]
  - **Range**: the position is clamped to between 5% and 95% of the edge length
  - **Midpoint**: this rule differs from placement at the midpoint of the edge
- **Support values move with their branch**: when internal labels are declared as edge labels, the reroot copies them onto edges, reroots, and copies them back [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/util/RerootingUtils.java#L64-L76)]
  - Supports therefore stay on the correct branches
  - This is the behavior that [Czech et al. (2017)](https://doi.org/10.1093/molbev/msx055) recommend
- **Outgroup reroot**: with several selected labels, the program finds the tightest rooting that puts the outgroup below the root. Not available for networks ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L261-L264))
- **Network reroot**: a dialog warns "Rerooting networks has major bugs, try anyway?". Rerooting on a reticulate edge or below a reticulation is refused [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/util/RerootingUtils.java#L51-L62)]
- **Midpoint rooting**: available as a command, with a ranking of all edges by a midpoint score [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/util/RerootingUtils.java#L333-L370)]

### Polytomies and labels

- **Polytomies**: drawn with the parent at the midpoint of the first and last child (see "Rectangular coordinates")
  - **Tanglegrams**: the method handles them with repeated adjacent swaps
  - **Hybridization networks**: the method refines them against the other tree
- **Unlabeled leaves in tanglegrams**: a leaf without a label gets a generated label `null<node id><edge id>` before the tanglegram, which then counts as a taxon of one tree only [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/tanglegram/TanglegramUtils.java#L364-L374)]

## Defects

### Tanglegrams

- **Connectors displaced in exported images** (observed): in PNG and SVG exports of a tanglegram, every connector runs about one leaf row below its leaves [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/Connector.java#L52-L63)]
  - One extra line appears below the last leaf
  - **Cause**: the connector code converts coordinates through the screen position of the live window and subtracts a hard-coded 20 pixels, which the source comment describes as a guess
  - Whether the offset is right depends on the window decoration
- **Connectors in the wrong panel in PDF export** (observed): the PDF of a 1x2 tanglegram draws all connectors inside the right panel, starting at its left border [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeGrid.java#L878-L891)]
  - **Cause**: the PDF path paints connectors with window coordinates into a panel that has other offsets
- **The early stop of the rounds never fires** (derived): the variable `best` starts at `Integer.MAX_VALUE` and is never updated, so `score == best` is never true [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L194-L196)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L388-L391)]
  - **Effect**: the method always runs five rounds and returns the last orders, even when an earlier round had fewer crossings
- **Label identity comparison** (derived): when trees are restricted to common taxa, leaves are found with `getLabel(node) == taxon`, which compares object identity [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L655-L668)]
  - It works only while both strings are the same object, which holds for labels copied by `clone` (the trial with a missing taxon worked)
- **Out-of-range loop in Align Taxa** (derived): the loop over the linear order runs to `z <= bestOrdering.length` and reads one element past the end when a leaf is not found [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L495-L508)]
- **Tanglegram fails with a Java error** for some IQ-TREE trees ([#19](https://github.com/husonlab/dendroscope3/issues/19), open, no answer)

### Networks and parsing

- **Annotation before a branch length is an error** (observed): `X#H1[&segments={0}]:0.5` fails with "Unexpected ':' at position 20" [[src](https://github.com/husonlab/jloda/blob/27ec9d8cccf7ecfa64a64a30ad2c6a6243406a2f/src/jloda/phylo/PhyloTree.java#L716-L731)]
  - **Cause**: after a `[...]` block the parser accepts only `,`, `)`, or `;`
  - **Effect**: BEAST-style node annotations fail in Newick files
- **Doubled quotes** (observed): `'O''Brien'` becomes `OBrien`
- **Phylogram offset parameter ignored** (derived): the offset of reticulation nodes is fixed at 50% with the parameter call commented out, while the constructor sets the parameter to 5 for this purpose [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerParallel.java#L191)]
- **Leaf count in the proxy computation** (derived): a source comment notes a "slight bug": a child reached a second time through a reticulation counts as one leaf [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/drawer/TreeDrawerParallel.java#L291-L294)]
- **Random ladderize skips unary nodes** (derived): the recursion treats a node with one child as a leaf, so the order below a unary node is not shuffled [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeViewer.java#L809-L812)]
- **Build** (observed): `tripletMethods/CombinationGenetator.java` holds the class `CombinationGenerator`. A build that compiles only reachable sources fails, and all files must be passed to the compiler

### Display, export, and commands

- **Export title ignored** (derived): `exportimage` parses `title=` but never uses it [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/commands/ExportImageCommand.java#L93)]
- **Some lengths of 1 are not written** (observed): the Newick export omits the length of some edges with length 1
  - **Example**: `(A:1,B:1,C:1,D:2)` is exported as `(A,B,C,D:2)`
  - **Round trip**: Dendroscope reads a missing length as 1, so its own round trip is exact, but other tools read a missing length
- **Failed open still exports** (observed): after a parse error, `exportimage` reported a panel size of -2147482448 x -2147482895 and wrote a 1000x1000 empty image
- **Issues reported by users**: all of these are open
  - Label alignment at the baseline in circular and radial layouts ([#7](https://github.com/husonlab/dendroscope3/issues/7))
  - Broken PDF export with radial labels ([#6](https://github.com/husonlab/dendroscope3/issues/6))
  - A 250,000-node tree with many zero-length branches that hangs on open ([#5](https://github.com/husonlab/dendroscope3/issues/5))
  - Commands in the command window that start before the previous one finishes, so exports miss labels ([#2](https://github.com/husonlab/dendroscope3/issues/2))
  - Zoom limits in radial phylograms after an update ([#14](https://github.com/husonlab/dendroscope3/issues/14))
  - Save disabled after deleting a taxon ([#9](https://github.com/husonlab/dendroscope3/issues/9))

## Performance

- **Tanglegram cost** (observed, one run each, 2 GB heap): 17 leaves in 67 ms, 200 leaves in 17 s, 1,000 leaves not finished after 300 s
- **Cause** (derived): the cost is roughly O(n³) per round, for five rounds and two trees [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/tanglegram/TanglegramUtils.java#L120-L129)] [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/tanglegram/TanglegramUtils.java#L426-L466)]
  - **Leaf order**: the recursion calls `computeSetOfLeaves()` at every node it visits, so one order costs O(n²)
  - **Swap tests**: every adjacent swap test rebuilds orders and counts crossings in O(n²)
- **Network leaf insertion** (derived): each leaf tries every position with a full crossing count, which is O(n³) per leaf and O(n⁴) per network [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/embed/EmbeddingOptimizerNNet.java#L264-L366)]
- **Connector construction** (derived): all node pairs of the two panels are compared, O(nm) for panels with n and m nodes [[src](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/src/dendroscope/window/TreeGrid.java#L762-L790)]
- **Drawing of large trees**: proxy shapes and off-screen skipping keep repaint cost near the number of visible subtrees. The author called 10,000 taxa "a small tree size" for Dendroscope ([#1](https://github.com/husonlab/dendroscope3/issues/1))

## Interaction with TreeKnit output

### TreeKnit files

TreeKnit writes the following files ([packages/treeknit-io/src/output.rs#L206-L264](../../packages/treeknit-io/src/output.rs#L206-L264)):

- **Resolved trees**: `<label>_resolved.<ext>`
- **ARG**: the ARG in extended Newick (`ARG/arg.nwk`), the ARG node table, and the ARG segment trees
- **MCCs**: `MCCs.json` and `MCCs.dat`
- **Figures and other formats**: Auspice JSON and SVG figures

In the ARG every branch carries `[&segments={0,1}]`, and the annotation comes **before** the branch length, as in `label#H1[&segments={0}]:0.5` ([packages/treeknit-io/src/arg.rs#L12-L87](../../packages/treeknit-io/src/arg.rs#L12-L87)).

### Results in Dendroscope

- **ARG as Newick fails** (observed): a hand-written ARG in exactly this shape failed to open with "Unexpected ':' at position 20". No tree is loaded
- **ARG wrapped in NEXUS loads** (observed): the same string inside a NEXUS trees block opens, because the NEXUS tokenizer removes the brackets. The result:
  - **Segments lost**: no annotation survives, so Dendroscope cannot tell which segment a branch carries
  - **Reassortment branch lengths lost**: both edges into each hybrid node get length 0, and the phylogram places the hybrid node at its rightmost parent plus half a mean edge length
    - TreeKnit's reassortment times, set from the matching branches of the two trees ([packages/treeknit-core/src/arg.rs#L492-L539](../../packages/treeknit-core/src/arg.rs#L492-L539)), are not shown
  - **Zero-length hybrid edges are harmless**: Dendroscope has no zero-length collapse, so a hybrid edge of length 0 draws normally (observed with `R1#H1:0.0`)
- **Hybrid labels**: TreeKnit hybrid occurrences carry the same label on both occurrences, so the merged node keeps one label (derived from the merge rule)
- **Resolved trees in a tanglegram**: resolved trees are plain Newick and load. Two derived risks:
  - **Internal labels get connectors**: resolution inserts nodes named `RESOLVED_<n>`, numbered in each tree on its own ([packages/treeknit-core/src/resolve.rs#L140-L142](../../packages/treeknit-core/src/resolve.rs#L140-L142)). Equal names in the two trees would be connected even when the clades differ
  - **Internal label dialog**: in the GUI, any internal label in the first tree opens the dialog about node labels and support values
- **MCCs**: Dendroscope has no input for clade groups or colours. A command script with `find` and `set color` per MCC is the only way to colour them
- **TreeKnit's own reticulation semantics**: Dendroscope's hybridization networks label each reticulate edge with the input tree it comes from (`[1]` or `[2]`)
  - This label corresponds to TreeKnit's segment set of a reassortment edge
  - Dendroscope cannot read that label back as data, but its Newick export of computed networks shows the convention

## Ideas from the issue tracker and history

### Open requests

- **Connectors only between leaves** ([#10](https://github.com/husonlab/dendroscope3/issues/10)): a user wants to suppress lines from inner nodes
- **Node colour from NeXML or NEXUS annotations** ([#11](https://github.com/husonlab/dendroscope3/issues/11)): a user writes `[&color=#ff0000]` with DendroPy and finds it ignored
  - The request is for a standard format instead of Dendroscope-specific commands
- **An API for standardized rendering** ([#17](https://github.com/husonlab/dendroscope3/issues/17))
- **Old zoom behavior** ([#14](https://github.com/husonlab/dendroscope3/issues/14)): deep zoom into small clades of huge trees, which the user names as the main reason to load large trees
- **A switch to turn off proxy shapes**, offered by the author in [#1](https://github.com/husonlab/dendroscope3/issues/1) and never added
- **Release tags** ([#8](https://github.com/husonlab/dendroscope3/issues/8))
- **File associations** ([#4](https://github.com/husonlab/dendroscope3/issues/4))

### Declined

No issue was closed as declined. Most open issues have no answer.

### Features found only in the history

- **Removed algorithms**: three features were removed in [`fb101ad`](https://github.com/husonlab/dendroscope3/commit/fb101ad) (2021-03-11, "removed problematic code", 171 files and 41,527 lines)
  - **Hybridization networks for binary trees**: Albrecht et al. 2012, command `method=ASCH2011`
  - **rSPR distance**
  - **Hybroscale**: all minimum networks for several multifurcating trees, Albrecht 2015
  - **Manual**: the manual still describes the first two in its menu list and command summary ([manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L392-L411), [manual](https://github.com/husonlab/dendroscope3/blob/c2d35555003a9261120a1d3223fd00e3e180f46b/tex/manual/manual.tex#L885-L896))
- **Transfer visualization**: the class is still present but switched off with `if (false)` (see "Drawing reticulate edges")
- **Shortest-path tanglegram distance**: implemented and preferred by the paper for networks, but no command uses it
- **Networks with both transfers and hybridizations**: supported in the unoptimized layout since [`78192d7`](https://github.com/husonlab/dendroscope3/commit/78192d7) (2021)

## Open scientific problems

- **Reticulation crossings in tanglegrams**: the paper defines a second optimization level for crossings of reticulate edges and leaves it open. The code ignores it
- **Times of reticulations**: Dendroscope draws a reticulation node at a fixed offset from its parents because extended Newick gives each hybrid edge its own length, and the lengths need not agree. No rule says which length is the time of the event
- **Optimal tanglegrams**: the NN method guarantees zero crossings when possible, but gives no bound otherwise. On the test pairs a simpler method found fewer crossings
- **Labels as node or branch data**: Dendroscope asks the user once per file, and its connectors treat equal internal labels as the same object in both trees, which is only true for some label kinds

## Not covered by Dendroscope

- **Data on the tree**:
  - **Colour by attribute or group**: no annotation parsing, no colour by clade, partition, or segment, and no legend
  - **Time axis**: no axis, no dates, no ages
  - **Annotations**: `[&key=value]` comments are dropped or break parsing
- **Several trees**:
  - **Ribbons or grouped connectors**: connectors are single straight lines without colour by group, without bundling, and without curves
  - **Untangling of more than two trees**: "Align Taxa" orders many trees by one shared circular order, without a crossing objective
- **Platform**:
  - **Headless rendering**: batch mode still needs an X display
  - **Web use**: desktop only, and the download links in the README broke in 2026 ([#22](https://github.com/husonlab/dendroscope3/issues/22))

## Method and limits

- **Read**: the sources below, with the depth stated
  - **Code**: the manual, the tanglegram, embedding, connector, grid, drawer, parser, reroot, export, consensus, and Autumn code listed above, and the `jloda` parser, label thinning, and magnifier
  - **History**: all 22 issues, the commit log, and the removal commit `fb101ad`
  - **Papers**: read through their abstracts and a summary of the full text of the tanglegram paper
  - **Not read in detail**: the algorithmic core of Autumn, the galled and level-k network algorithms, and the distance computations
- **Built and ran**: Dendroscope from the surveyed source and `jloda` at `27ec9d8`, compiled with `javac` in `eclipse-temurin:17-jdk`
  - **Environment**: a derived image with `xvfb`, without network and with all capabilities dropped. Inputs and outputs stayed under `/tmp/feat-survey-dendroscope/`
  - **Test inputs**:
    - two 17-leaf trees, two 200-leaf trees, and two 1,000-leaf trees for tanglegrams
    - a one-reticulation network with and without TreeKnit annotations, as Newick and as NEXUS
    - a network against a tree with an extra taxon, and two trees with equal internal labels
    - a tree with a polytomy, a zero-length tip, missing lengths, a negative length, a `#` in a label, a doubled quote, and a long label
    - two trees for the Autumn hybridization network
- **Comparison heuristic**: the crossing numbers of the alternative method come from a throwaway Python script, run on the host on generated data only. They show that better orders exist, not what the optimum is
- **Not done**:
  - **GUI**: no interactive GUI session, so mouse behavior, the magnifier, the Format panel, and on-screen connector placement were not observed
  - **Connector offset**: observed only in exports under a virtual display
  - **Installers**: the release installers from the university site were not downloaded
