# Phylo.io feature survey: tree rendering, semantics, and edge cases

This report describes Phylo.io, a browser-based viewer that shows one phylogenetic tree or two trees side by side and colours each branch by how well its clade matches the other tree. It is an idea inventory for tree, tanglegram, network, and ARG display. It records what Phylo.io does, how it does it, which scientific conventions it assumes, which inputs break it, and which problems remain open. It does not compare Phylo.io with `packages/web`.

Phylo.io is licensed MIT. The vendored phyloXML reader `src/phyloxml.js` is LGPL-2.1.

- **Source**: [DessimozLab/phylo-io](https://github.com/DessimozLab/phylo-io) at commit `2b00e3a` (2026-01-30, package version 2.2.5), 929 commits since 2015. Source links point to this commit
  - **Version 1**: the 2016 version (phylo.io 1.0) lives on the branch `v1`, last changed at commit `a68d541` (2023-11-03), and is cited as "v1" where its behavior differs
- **Live application**: <https://phylo.io/viewer/>, operated in Chrome on 2026-10-07
  - **Bundle**: it loads `https://phylo.io/static/phylo.js`, which contains the latest default colour scale of commit `0726a8c`
  - **Match with the repository**: its size differs from `dist/phylo.js` of the repository by 123 bytes, consistent with different sharing URLs, so the observations and the source describe the same program
  - **Website shell**: the sidebar, session loading, and compare toggle are in the repository only as the example page [`Examples/Website/phyloio.html`](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/Examples/Website/phyloio.html), which has the same menu entries
- **Other sources**:
  - The README help section
  - All 13 issues and 5 pull requests with their comments
  - The commit history
  - The paper [Robinson, Dylus and Dessimoz 2016](https://doi.org/10.1093/molbev/msw080) (preprint [arXiv:1602.04258](https://arxiv.org/abs/1602.04258)). No paper about the 2.x version was found
- **Evidence labels**: "Observed" means seen in the live application. "Derived" means read from the code without running it. Where the README and the code disagree, the code wins and the difference is noted
- **Shape of the code**: an ES-module library bundled with webpack, built on d3 v6 and Bootstrap 5
  - **Classes**: `API` holds the containers, a `Container` binds one `div` to one `Viewer` and a list of `Model` trees, and the `Viewer` draws SVG with `d3.cluster` ([`src/index.js`](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/index.js#L1-L13))
  - **Workers**: the comparison runs in two Web Workers, [`src/worker_bcn.js`](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/worker_bcn.js) (best corresponding nodes) and [`src/worker_distance.js`](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/worker_distance.js) (tree distances in [`src/utils.js`](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/utils.js))
  - **User interface**: built in [`src/interface.js`](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js) (4283 lines)

## Summary

### Comparison of two trees

- **Comparison by colour**: compare mode draws two independent trees side by side and colours each branch by the Jaccard similarity between its clade and the best corresponding clade in the other tree
  - **Score**: `|L(v) ∩ L(w)| / |L(v) ∪ L(w)|` over the leaves common to both trees, maximized over candidate nodes `w`. Since version 2 the candidates come from a MinHash LSH forest (top 10), so the best match can be missed (see "Two trees")
  - **Colour scale**: 1 is dark red (`#a50026`), 0.5 is pale yellow, and 0 is dark blue (`#313695`). Observed with small test trees. Version 1 used the reverse direction (1 dark blue, 0 pale yellow)
  - **Navigation**: "Highlight BCN" on a node centres and blinks the best corresponding node in the other tree. "Reroot" and "Reorder" adapt one tree to the other
- **No link lines**: there are no lines between matching leaves. A tanglegram was requested in 2025 ([#15](https://github.com/DessimozLab/phylo-io/issues/15), open)
- **Different leaf sets**: leaves present in only one tree are ignored in the score, so two clades that differ only by such leaves score 1
  - Leaves without a common leaf get no score and stay grey (observed)

### Risks for TreeKnit users

- **ARG annotations break the parse**: both Newick readers split on every comma, so TreeKnit ARG annotations `[&segments={0,1}]` break the parse. The tree is silently not added (observed)
- **Hybrid nodes become labels**: hybrid nodes `#H1` are read as ordinary labels, so an ARG without annotations draws as a tree with duplicated leaves

### Large trees and distances

- **Large trees**: Phylo.io handles large trees with three mechanisms
  - Automatic collapse to about 30 visible tips on load, with collapsed clades as triangles labelled `[first ... last]`
  - Label subsampling
  - A "big tree" mode above 500 leaves that skips label updates during zoom
- **Distances**: all distances use the leaf intersection
  - Robinson-Foulds and a rooted "Clade" distance
  - A branch-length "Euclidean" distance
  - A labelled RF for duplication-annotated gene trees

### Defects

- **Comparison**: two defects affect the comparison (see "Defects")
  - The root never gets a similarity score
  - The "Reorder" button never rotates the root
- **Interaction and colour**: three defects affect the display (see "Defects")
  - Keyboard resizing multiplies the tree size by 21 per key press
  - The "big tree" mode is lost after the similarity computation
  - Categorical palettes give the same colour to different categories

## Layouts

### Coordinate system and layout

- **One layout only**: a rectangular tree with the root at the left. `d3.cluster` gives each leaf one row of `node_vertical_size` (default 30 px) and places each parent at the mean of its children's rows [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L97-L99)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L188-L219)]
  - **Horizontal position**: the distance from the root, scaled linearly from 0 to the largest root-to-node distance. The cladogram mode uses the node depth (edge count) on the same pixel range [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L248-L262)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L407-L414)]
  - **No circular or radial layout**, and no unrooted layout
- **Elbow edges**: each edge is one path from the child to the parent's time, then vertically to the parent row [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L1108-L1120)]
- **Root edge**: the root edge is never drawn. A root with three children counts as "unrooted" and its circle is hidden (radius `1e-6`) [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L158)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L539)]
- **Separate scales per tree**: each tree in compare mode fits its own view, so the scale bars differ even for identical trees. There is no shared time or distance axis
  - **Observed**: 0.3473 and 0.6215 for the same 737-leaf tree

### Tree settings (Settings > Tree)

- **Ignore branch length**: switches to the cladogram. Shown only when the tree has lengths [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L2503-L2506)]
- **Align leaves**: moves tips and triangles to the deepest tip and draws a dashed white line from each tip to its aligned label position [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L669-L757)]
- **Mirror tree**: draws the tree right to left. With the left tree normal and the right tree mirrored, the two trees face each other like a tanglegram without links [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L1709-L1719)]
- **Tree height and width**: buttons -50%, -20%, +20%, +50% multiply the row height or the horizontal node size [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L3867-L3966)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L886-L906)]
- **Autocollapse slider**: from "Off" (0) to the maximum depth. Every node at this edge depth or deeper is collapsed [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L2566-L2579)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L394-L429)]

## Branches, nodes, and collapsed clades

### Node marks

- **Node circles on internal nodes only**: leaves have no mark. A collapsed node has no circle [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L490-L505)]
- **Constant on-screen sizes**: node radius, line width, and font size are divided by the zoom factor `k`, and radius and width are capped at half the row height [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L2263-L2273)]
- **Duplications**: NHX `D=Y` or `Ev=duplication` marks a node as a duplication, and "Show duplications" fills its circle red [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L511-L535)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L540-L543)]

### Collapsed clades

- **Collapsed clade triangle**: the triangle starts at the node, and its length is the mean root-to-leaf distance of the clade minus the node's distance [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L632-L667)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L998-L1006)]
  - **Height**: the half-height is `node_vertical_size * sqrt(leaf count) * 0.2`
  - **Row spacing**: the spacing grows by the same `sqrt(n) * 0.2`, so triangles do not overlap
- **Triangle label**: the internal node name when "Show collapsed subtrees name" is on and the name is not empty, else `[first leaf ... last leaf]` [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L2526-L2547)]
  - **Observed**: Ensembl gene trees show "speciation" and "duplication", and the 737-leaf example shows `[MUSAM01035 ... PHYPA09854]`
- **Triangle colour**: grey `#666` by default. "Color collapsed subtrees by" Leaves or Branches takes the colour from the clade [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L783-L879)]
  - **Categorical data and explicit colours**: the most frequent value
  - **Numbers**: the mean

### Menus and edits

- **Node menu** (click on a circle): the entries depend on the node and the mode [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L1122-L1226)]
  - **Collapse**: Collapse node or Expand, Collapse All, Expand All
  - **Other entries**: Swap subtrees, Hide label or Show label, and in compare mode Highlight BCN
  - **Root**: the root offers only Expand All and Swap subtrees
- **Edge menu** (click on a branch): Reroot, Trim subtree, Open as new tree [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L1228-L1280)]
- **Swap subtrees is a rotation**: the last child moves to the front, so in a polytomy with k children the user must click k-1 times to reach some orders [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L745-L756)]
- **Trim subtree**: removes the clade below the edge. When the parent becomes unary, the sibling takes the parent's place, and its branch length is not extended by the parent's length [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L920-L981)]
- **Undo and redo**: every collapse, swap, reroot, trim, label toggle, and fit is recorded per container [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L42-L74)]

## Labels

- **Leaf labels**: the name by default, or any attribute through Settings > Branches & Labels. Two further positions above-left and below-left of each leaf can show other attributes [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L2506-L2722)]
- **Internal node labels**: off by default. Three positions (right, upper left, lower left) can each show an attribute [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L2275-L2287)]
  - **"Topology"**: this special value prints the similarity score with two decimals
- **Label subsampling**: when on (default), only every m-th visible leaf gets a label, with `m = 1 + floor(fontSize / k / rowHeight)`, so labels never overlap [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L395-L438)]
  - **Triangles**: a collapsed triangle that is higher than the text always keeps its label
- **Screen-space text**: the font size is divided by the zoom factor, so labels keep their pixel size. Turning subsampling off fixes labels to the tree scale instead [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L2229-L2245)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L1761-L1776)]
- **Editing**: a click on a label opens a modal to edit the label value [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L2599-L2612)]
- **Data tooltips**: "Show data tooltips" lists every attribute of the hovered node. Numbers with decimals are rounded to 3 decimals [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L339-L393)]

## Colour and legend

### Colour targets and attribute types

- **Three colour targets**: branches (`node`), leaf labels (`leaf`), and node circles (`circle`). Each takes one attribute [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L50-L66)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L1782-L1803)]
  - **Link branch & leaf coloring**: this option uses the branch attribute for leaf labels too
- **A branch takes the value of the node below it** [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L949-L994)]
- **Three attribute types**: each attribute has one type [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/utils.js#L972-L981)]
  - **Numeric**
  - **Categorical**
  - **Explicit colour**: a column named `color` or `colour`, any capitalization, with CSS colour names, hex, or `rgb()` values
- **Missing values**: branches without a value are dark grey `#555`, circles without a value are transparent grey ([`6501adc`](https://github.com/DessimozLab/phylo-io/commit/6501adcd668640b409424327aff0afbbbb1f3c83))

### Scales and legend

- **Categorical scale**: one shared colour map per attribute name for all trees, so a category gets the same colour in both trees of a comparison [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/api.js#L88-L95)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L3250-L3300)]
  - **Scheme**: the user picks any `d3.interpolate*` scheme
  - **Collision bug**: see "Defects"
- **Numeric scale**: `d3.scaleLinear` from max to min across 2 to 5 user-picked stops [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L262-L364)]
  - **Stops**: new attributes start with 2 stops. The default stops are pure red, pale red, light grey, pale blue, pure blue (`#FF0000` to `#0000FF`), and "Length" uses a blue-to-yellow ramp ([`578e796`](https://github.com/DessimozLab/phylo-io/commit/578e796b2278b47bd32784c2f9a1e5c6ca4a34c5), [`0726a8c`](https://github.com/DessimozLab/phylo-io/commit/0726a8c892bcf761156686b375e050ebabed96fc))
  - **Min and max**: editable, with an "Auto min/max" reset [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L3315-L3520)]
  - **Legend**: a vertical 100-step gradient bar with the maximum at the top, drawn at the left of the viewer [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L1343-L1447)]

### Collapse by colour

- **Collapse by colour**: two buttons collapse clades by their colours ([`4d87e1a`](https://github.com/DessimozLab/phylo-io/commit/4d87e1a406ad44a98e66dfa74677ae39a549daad), 2025) [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L431-L588)]
  - **Collapse uncolored subtrees**: collapses every clade without any coloured leaf (or node)
  - **Collapse monocolored subtrees**: collapses clades whose colours are all equal. For numeric data, colours count as equal when every pair differs by at most 5% of the maximum RGB distance
  - **Idea for TreeKnit**: colour leaves by MCC and collapse every clade that lies inside one MCC

## Zoom and navigation

### View

- **d3 zoom**: wheel zoom around the pointer, drag to pan, and buttons Zoom in (x2) and Zoom out (x0.5). Double-click zoom is off [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L72-L84)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L1952-L1962)]
- **Lock Zoom**: in compare mode the other viewer copies the zoom factor and the x offset, and the y offset is shifted so that the vertical middles of the trees line up [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L1079-L1098)]
- **Fit screen**: scales the view to the current height and stretches the horizontal node size to the width, without collapsing [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L1510-L1559)]
- **Optimise view** (also on first render): increases the collapse depth from 1 until more than `max_visible_leaves` = 30 tips are visible, collapses at that depth, then fits [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L1561-L1638)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L293-L314)]

### Search and tree switching

- **Search**: a text field with case-insensitive substring autocompletion over all node names [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L1462-L1517)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L764-L827)]
  - **Selection**: selecting a name expands the path from the root, colours the path red, and colours the label red
  - **Match**: only the first exact name match is used
  - **Keep queries highlighted**: this option keeps earlier searches
- **Several trees per viewer**: Previous and Next buttons and dots switch between the trees of a container. Each tree keeps its own zoom [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L157-L203)]

### Keyboard shortcuts

- **Keyboard shortcuts**: each viewer has its own keys [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/keyboardManager.js#L19-L56)]. See "Defects"
  - **Left viewer**: `w`/`s` height and `a`/`d` width, `q`/`e` previous and next tree, `r` optimise view, `f` fit
  - **Right viewer**: `i`/`k`, `j`/`l`, `u`/`o`, `z`, `h` do the same
  - **Both viewers**: `t` computes distances and `g` toggles compare mode

## Two trees side by side

Compare mode binds two containers (left and right). When both show a tree, a worker computes the best corresponding node of every node in both directions, and then both viewers redraw [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/api.js#L133-L240)].

### Best corresponding node (BCN) and similarity score

#### Definition

- **Paper definition**: the paper describes "a variation of the Jaccard index" after Munzner et al. (2003, TreeJuxtaposer) and Bremm et al. (2011), with the best corresponding node computed lazily for visible nodes ([Robinson et al. 2016](https://doi.org/10.1093/molbev/msw080))
- **Score in the code**: for a node `v` in tree 1 and a candidate `w` in tree 2, `S(v, w) = |X(v) ∩ X(w)| / |X(v) ∪ X(w)|`, where `X(u)` is the set of leaves below `u` restricted to the leaves common to both trees [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/worker_bcn.js#L18-L144)]
  - **BCN**: the candidate with the largest `S`. The node's score is that maximum
  - **Ties**: the first candidate is kept
  - **Zero**: a score of 0 is not stored
- **Candidates from MinHash**: since 2022 ([`f73472c`](https://github.com/DessimozLab/phylo-io/commit/f73472c20f24cb4ecdcc2d58b542c9a5efcffa2d), [`7c12865`](https://github.com/DessimozLab/phylo-io/commit/7c128650fcf373eac2c264d275a40b9552607197)), only the 10 nearest candidates from a MinHash LSH forest are scored [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L1049-L1106)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/worker_bcn.js#L89-L144)]
  - **Hash input**: every node gets a MinHash of its "deep leaf list" (its leaf names plus one joined string per sub-clade), and all nodes go into the forest
  - **Consequence**: the score is a heuristic. The true best Jaccard match can be outside the 10 candidates, because the hash covers sub-clade strings that the score ignores
  - **Version 1** scored every node of the other tree that shares at least one leaf (the "spanning tree"), so it was exact [[src](https://github.com/DessimozLab/phylo-io/blob/a68d54126234fe7d17bf08a65226779e037e2b87/www/js/treecompare.js#L4639-L4683)]

#### Observed scores and colours

- **Observed scores** with `((A,B),(C,(D,E)))` against `((A,C),(B,(D,(E,F))))`:
  - `AB` scores 0.5 and its BCN is the leaf A, `CDE` scores 0.667 and matches `BDEF`
  - `DE` scores 1.0 against `DEF`, because F is not in the first tree
  - `F` has no score, its branch stays grey, and the root has no score (see "Defects")
- **Colour per branch**: the "Topology" accessor colours the branch above each node by its score [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L52-L62)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L949-L956)]
  - **Scale**: 1 maps to `#a50026` (dark red), 0.75 to `#f46d43`, 0.5 to `#ffffbf`, 0.25 to `#74add1`, and 0 to `#313695` (dark blue)
  - **Observed**: identical trees are entirely dark red, a score of 0.5 is pale yellow, 0.667 is orange. The legend reads "Topology" from 1 at the top to 0 at the bottom
  - **Version 1** used a 6-stop ColorBrewer ramp from dark blue (1.0) to pale yellow (0.0), and coloured branches by support values when internal labels were shown [[src](https://github.com/DessimozLab/phylo-io/blob/a68d54126234fe7d17bf08a65226779e037e2b87/www/js/treecompare.js#L34-L37)]
- **Triangles**: a collapsed clade is coloured by the score only when "Color collapsed subtrees by" is set. By default it stays grey (observed)
- **Progress message**: "Computing similarity... (~ t seconds)" with `t = (2 * n1 + 2 * n2) / (40000/150)` for leaf counts n1 and n2 [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/api.js#L223-L231)]
  - **Observed**: two copies of the 737-leaf example finished in about 5.6 s, against an estimate of 11 s

### Highlight BCN

- **Action**: the node menu entry "Highlight BCN" expands the path to the BCN in the other tree, centres it, and makes its circle blink red six times at 750 ms intervals [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L347-L369)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L577-L600)]
- **Observed**: highlighting `CDE` circled `DEF` in red in the other tree
- **Leaf BCN is invisible** (observed): when the BCN is a leaf, as for `AB` above, nothing blinks, because leaves have no circle
- **Zoom reset** (observed): centring sets the zoom factor to 1, so the other tree jumps to a different size [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L1945-L1950)]
- **History**: in version 1 the selected node was red and its subtree and counterpart green ([Robinson et al. 2016](https://doi.org/10.1093/molbev/msw080))
  - A 2018 report said the highlight did nothing ([#4](https://github.com/DessimozLab/phylo-io/issues/4), closed)

### Equalize trees: reroot and reorder

Settings > Tree > "Equalize trees" has two buttons in compare mode ([`171dfa1`](https://github.com/DessimozLab/phylo-io/commit/171dfa1348e6cd2c698f23cfa0620b72fe64160a), [`82a3b22`](https://github.com/DessimozLab/phylo-io/commit/82a3b22ab193c25e403f4b1688b2830ec6179964)) [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L2582-L2615)].

- **Reroot**: takes the first child of the other tree's root, finds its BCN in this tree, and reroots this tree on the branch above that BCN [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L996-L1014)]
- **Reorder** (tip order adaptation): visits each internal node of this tree in post-order and rotates its children once when its end leaves differ from those of its match [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L1016-L1092)]
  - **Match**: the internal node of the other tree that contains the most of its leaves (an absolute count, the first maximum in post-order wins). The rotation happens when both the first and the last leaf differ from those of that node
  - **Single pass, local rule**: it compares only end leaves, never counts crossings, and for a polytomy tries one rotation only
  - **Observed**: against `((A,B),(C,(D,E)))`, the tree `(((E,D),C),(B,A))` became `((C,(D,E)),(A,B))`, order CDEAB. Each clade matched, but the root was never rotated, so the halves stay crossed (see "Defects")
  - **Version 1** had the same rule, swapped the first two children, and visited the root [[src](https://github.com/DessimozLab/phylo-io/blob/a68d54126234fe7d17bf08a65226779e037e2b87/www/js/treecompare.js#L2680-L2780)]
- **No link lines**: the two trees are separate SVG elements with no connectors between equal leaves. Comparison relies on colour, on aligned order after "Reorder", and on Lock Zoom

### Different leaf sets

- **Score**: restricted to the leaf intersection. Clades that differ only by unshared leaves score 1, and nodes whose leaves are all unshared stay unscored (observed)
- **Distances**: computed on the intersection after removing unnamed leaves and leaves whose name occurs more than once [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/utils.js#L580-L645)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/utils.js#L917-L929)]
  - **Empty intersection**: the message is "No leaves in common to compute distance."
- **History**: the 2016 paper required identical leaf sets and listed partial overlap as future work. Version 2 supports it
  - **Open report**: a 2021 report of a comparison that never finished for 498 and 497 taxa with one unshared taxon is still open ([#13](https://github.com/DessimozLab/phylo-io/issues/13))

### Tree distances

The "Distance" sidebar button computes distances in a worker and shows them in a floating window. Results are cached per tree pair [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/api.js#L365-L419)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/utils.js#L901-L970)].

- **Clade distance**: rooted cluster difference with Day's (1985) table algorithm on the intersection, `n1 + n2 - 2 * common`, where the counts include the root cluster [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/utils.js#L32-L137)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/utils.js#L458-L554)]
- **Robinson-Foulds**: both trees are rerooted at the first common leaf and the same table comparison runs, which gives the unrooted RF [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/utils.js#L946-L958)]
- **Euclidean**: the sum of three parts, shown with 2 decimals [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/utils.js#L486-L543)]
  - `|b1 - b2|` over matched clusters
  - The lengths of unmatched clusters
  - The leaf branch length differences
- **Labeled RF**: computed only when both trees carry duplication labels (NHX `D` or `Ev`), adapted from pylabeledrf [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/utils.js#L869-L899)]
  - **Method**: each island between common edges adds its size, plus one substitution when its labels disagree
- **Display**: "Left tree: c/n (p%)" and "Right tree: c/n (p%)" with the distance as a badge
  - **Observed** for the test pair above: RF 2 with "2/3 (67%)", Clade 4 with "2/4 (50%)", Euclidean 4.00
- **Interpretation is undocumented**: a 2026 user asked how to read RF, Clades, and Euclidean ([#18](https://github.com/DessimozLab/phylo-io/issues/18), open)

## Networks and reticulations

- **No network support**: there is no hybrid node concept, no reticulation edge, and no ARG view. `#H1` is part of a label (observed, see "Interaction with TreeKnit output")
- **Phylostratigraphy stacks**: a separate mode shows bar stacks of gene gains, losses, duplications, and retentions at each node of a JSON species tree, for the OMA browser ("Bar Graph" settings, [`Examples/Phylo Stack`](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/Examples/Phylo%20Stack/stack.html)) [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L2108-L2225)]

## Input formats

### Tree formats

- **Newick**: the `biojs-io-newick` parser splits on `( ) , : ;` with a regular expression and keeps everything else as text [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L670-L693)]
  - **Observed**: quotes stay in labels (`'a b'`, `'O''x'`), comments stay in labels (`C[note]`), and any comma inside brackets starts a new sibling
- **Extended Newick (NHX)**: a variant that also splits on `[ ] =` and stores `key=value` pairs after `[&&NHX` as attributes [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/utils.js#L392-L441)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L500-L549)]
  - **Keys**: `D`, `DD`, `Ev` mark duplications, `B` and `XB` are branch values ([#8](https://github.com/DessimozLab/phylo-io/issues/8), fixed in version 2)
  - **BEAST or FigTree comments**: `[&key=value]` comments are not NHX and break on their commas
- **PhyloXML**: only the first phylogeny is read, with a console message ([`017cf91`](https://github.com/DessimozLab/phylo-io/commit/017cf91049c13f3bbecbc4b5b66ed1f7563a374f)) [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L551-L569)]
  - **Attributes**: name, branch length, the first taxonomy and sequence, and date
  - **Ignored**: confidence and colour elements
- **JSON**: library users can pass a nested `{name, children, branch_length}` object (used by the stack mode) [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L680-L682)]
- **Unsupported inputs**:
  - NEXUS
  - Multi-tree files (one tree per load)
  - Auspice JSON in the dialog

### Dialog, labels, and annotations

- **Add tree dialog**: the format is never detected [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L355-L469)]
  - **Steps**: paste text or pick a file, then choose the format
  - **Options**: tick "Use internal label for branches (e.g. boostrap values)", and add a mapping file
- **Internal labels**: every non-empty name becomes the attribute "Data" [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L473-L498)]
  - **Numeric labels**: if all are numeric, "Data" becomes a numeric colour attribute, so support values can colour branches
- **Mapping file**: CSV or TSV with a header row [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L777-L908)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L1178-L1322)]
  - **Join**: on a chosen column (default `id`) against leaf names or internal node names
  - **Column types**: the user types each column Numerical, Categorical, or Color, with automatic defaults
  - **Scope**: leaves, nodes, or all trees of the container
  - **Walkthrough**: a user documented a colour-by-category walkthrough with a two-column TSV ([#14](https://github.com/DessimozLab/phylo-io/issues/14))
- **Examples**: two "Tree collections" in the empty viewer, and more examples in the dialog [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L24-L37)]
  - **Dialog examples**: small yeast trees, a 737-leaf plant gene tree, NHX gene trees, unrooted trees, and a pair for the labelled RF

## Export, sessions, and sharing

- **Graphics**: per viewer "PNG" (rasterized at 2x the window size) and "SVG", each with the phylo.io logo added [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L2195-L2223)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/utils.js#L328-L390)]
  - **Screenshot session**: this sidebar button puts both viewers side by side with a vertical divider line, PNG at 1x
  - **Content**: the export is the current view, so collapsed clades and zoom are included
  - **README**: the README mentions PDF, which the code does not offer
- **Text**: two text exports [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L1120-L1168)]
  - **"Newick"**: names and lengths through `biojs-io-newick`
  - **"NHX"**: every attribute as `:key=value` inside `[&&NHX...]`, with spaces and `( ) [ ] , : ;` in values replaced by `_`
- **Session file**: "Save session" downloads `Session.phyloio`, a JSON dump of all containers, trees, collapse states, zoom, and colour maps. Loading it replaces the session [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/api.js#L300-L363)]
- **Share link**: posts the session JSON to the phylo.io server and returns `https://phylo.io/viewer/?session=<token>` [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/api.js#L423-L491)]
  - **Size limit**: the server answers 413 for sessions that are too large
  - **Privacy**: the tree data leaves the browser. The 2016 version shared through GitHub Gist
- **Embedding**: the library is on npm (`phyloio`) and jsDelivr, and is used in the OMA browser stack view (README)

## Scientific semantics

### Branch lengths and time

- **Distance from the root only**: positions come from cumulative branch lengths. There is no date axis, no time direction, and no tip dating
- **Missing lengths become 1** (observed): when any child of the root has a length, every missing length is set to 1. `((A:2,B):1,C:3);` places B at distance 2 [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L411-L445)]
  - **Detection checks only the root's children**: a tree whose root children lack lengths becomes a cladogram even if all other branches have lengths
- **Zero-length branches**: drawn as zero-length segments, so a zero-length leaf sits on its parent's vertical line [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L460-L471)]
  - **Length attribute**: the value 0 is not stored as a "Length" attribute, because the test is truthiness (observed: leaves with `:0` have no "Length")
- **Scale bar**: 120 px, labelled with the corresponding branch length to 4 decimals and updated on zoom [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L1300-L1340)]

### Rooting and support values

- **Rooted or unrooted**: a root with exactly three children counts as unrooted (observed with `((A,B,C),(D,E),F)root;`). The flag only hides the root circle
- **Reroot on the midpoint of an edge**: the new root splits the clicked branch into two halves [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L758-L918)]
  - **Edge data**: branch lengths and every key listed in `edge_related_data` move with their branch when the path to the old root is reversed
  - **Old root**: a unary old root is removed, a multifurcating old root stays as a node
  - **Branch interpretation is opt-in**: `edge_related_data` holds `Length`, NHX `B` and `XB`, and the internal labels ("Data") only when the user ticks "Use internal label for branches". Without the tick, support values stay on nodes and are misplaced after a reroot [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L555-L561)]
  - **Fix**: a rerooting bug was fixed and unit tests added in 2025 ([`7990e4f`](https://github.com/DessimozLab/phylo-io/commit/7990e4fb309b731e5c1d423b2de7ba4ef883b44e))
- **Reroot invalidates the comparison**: rerooting or trimming restarts the similarity worker and the distance worker [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L277-L333)]

### Polytomies and similarity

- **Polytomies**: drawn with all children on one vertical line, the parent at the mean row
  - **Score**: the score treats a polytomy clade like any other clade, so a resolved tree and its unresolved version differ only at the resolved splits
  - **Example**: a new split such as `(A,B)` inside `(A,B,C)` scores 2/3 against `ABC`
- **The score is clade-based**: it depends on the rooting of both trees. "Reroot" exists to reduce this effect
- **Asymmetry**: BCN(v) = w does not imply BCN(w) = v. Each direction is computed and coloured separately

## Defects

Each entry gives the input, the effect, and the evidence. Observed entries were reproduced on phylo.io on 2026-10-07.

### Comparison

- **The root never gets a score** (observed and derived): the root shows no score and cannot use "Highlight BCN" [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L138-L157)]
  - **Cause**: the worker sends models back, and the `Model` constructor resets `this.data.elementS = {}` and `elementBCN = {}` even for models rebuilt from worker data
- **"Reorder" never rotates the root** (observed): `traverse(data, null, f)` calls `f` only for children, so the root's children keep their order [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L1028-L1040)]
  - **Observed**: the reference was `ABCDE`, the result was `CDEAB`
- **Stale leaf lists during "Reorder"** (derived): a rotation updates the rotated node's leaf list but not its ancestors' lists, so the end-leaf test of an ancestor uses the old order [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L1079-L1092)]
- **Approximate BCN** (derived): only 10 LSH candidates are scored, so a score below 1 can be an artefact of the search
- **Leaf BCN invisible** and **zoom reset to 1** on "Highlight BCN" (observed, see "Highlight BCN")
- **Workers are not stopped** (derived): `stop_worker` iterates the plain object `this.containers` with `for...of`, which throws, and the empty `catch` hides it. Previous topology workers run to completion [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/api.js#L282-L292)]

### Distances

- **Unary-node suppression drops branch length** (derived, checked by hand): when a leaf outside the intersection is removed, the remaining child replaces its unary parent without adding the parent's length [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/utils.js#L608-L645)]
  - **Example**: for the test pair the Euclidean distance shows 4.00, but the leaf E has length 2 from the clade `DE` in the restricted second tree, so the value should be 5
- **Percentages include trivial clusters** (observed): the RF window shows "2/3 (67%)" for two trees that share one of two non-trivial splits, because the counts include a trivial bipartition
- **Quadratic lookup** (derived): each cluster scans the whole name table of the other tree [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/utils.js#L475-L477)]
- **Duplicated leaf names are dropped** without a message before the distance computation [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/utils.js#L580-L606)]

### Interaction

- **View and size**:
  - **Keyboard resizing multiplies by 21** (observed): every size key passes `20` to a function that multiplies by `1 + percent`, which expects a fraction such as `0.2` [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/keyboardManager.js#L28-L45)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L886-L895)]
    - **Observed**: pressing `w` changed the row height from 30 to 630, then `s` changed it to 13230
    - **Decrease keys**: the "decrease" keys `a`, `s`, `j`, `k` also increase
  - **Big-tree mode lost in compare mode** (observed): models rebuilt from worker data have an empty `leaves` list, so `big_tree` is false [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/model.js#L143-L159)]
    - **Observed**: the 737-leaf example reported 0 leaves after the similarity run, and the "Optimise text" button disappeared
  - **Undo of "Fit screen"** (derived): `render_with_settings` calls `this.container.modify_node_size`, but `this.container` is the DOM element [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L1650-L1654)]
  - **Window resize**: a user advises not to resize the window while working, because the design can be lost ([#14](https://github.com/DessimozLab/phylo-io/issues/14))
- **Settings and switches**:
  - **"Collapse monocolored subtrees" with Nodes or Both** (derived): reads `model.settings.extended_informations`, which does not exist, so the call throws a `TypeError` [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L693)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L743-L745)]
  - **Toggle rooting with distances on** (derived): uses an undefined variable `m` [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/container.js#L856-L866)]
  - **"Show node labels" switch** starts in the state of "Show leaf labels" [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L2546)]

### Colour

- **Categorical colour collisions** ([PR #17](https://github.com/DessimozLab/phylo-io/pull/17), open, confirmed by reading): `d3.scaleSequential` uses only the first two domain values, so the domain is `[0, n/2]` instead of `[0, n]` [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/color_mapper.js#L6-L39)]
  - **Cyclic scheme**: with the cyclic default `interpolateRainbow`, categories i and i + n/2 get the same colour
  - **Non-cyclic scheme**: the upper half of the categories all get the end colour
  - **Name**: the default scheme is named "Viridis" but is Rainbow
- **Numeric 0 counts as missing** (derived): a branch value of 0 is drawn grey `#555`, because the test is `!v` [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L974-L975)]

### Input

- **Failed loads are silent** (observed): an unparseable string throws "Cannot read properties of undefined (reading 'children')" in the console, the dialog closes, and no tree is added
- **HTML in names** (derived): autocomplete entries are built with `innerHTML`, so a crafted leaf name can inject markup [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L1493-L1498)]
  - **Tree labels**: labels in the tree are SVG text, so `<a>` tags do not create links ([#16](https://github.com/DessimozLab/phylo-io/issues/16))
- **Share error message** (derived): a 400 reply is combined with `|` (bitwise or) instead of `||`, so the message is lost [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/api.js#L447-L451)]

## Performance

- **Workers**: the similarity and distance computations run in Web Workers, so the page stays responsive. Each model is serialized without parent links and rebuilt after the worker finishes ([`115e4bc`](https://github.com/DessimozLab/phylo-io/commit/115e4bc), [`3e43f1e`](https://github.com/DessimozLab/phylo-io/commit/3e43f1e))
- **Measured** (observed): about 5.6 s from loading the second 737-leaf tree to finished scores for 1472 nodes in each tree
- **Big-tree mode** (more than 500 leaves): during zoom, label subsampling, circle radius, and line width are not recomputed ([`93e1ee0`](https://github.com/DessimozLab/phylo-io/commit/93e1ee05925ae37be4b8baad33a4964b568c494c), [`ff26941`](https://github.com/DessimozLab/phylo-io/commit/ff26941e4c0ddfc101740e2c75d1cc7ad6a07194)) [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L1021)] [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/interface.js#L1142-L1172)]
  - **Optimise text**: a bottom-right button applies them on demand
- **Collapse as the main scaling tool**: the paper claims 500-taxon trees in "a few seconds on a laptop", because collapsed nodes are not rendered and, in version 1, not compared
- **SVG with transitions**: every render runs a 300 ms d3 transition over all visible nodes and edges [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/viewer.js#L48-L50)]
- **Version 1 limit**: distances were not computed above 100 leaves ([`946d053`](https://github.com/DessimozLab/phylo-io/commit/946d0537081465c0928476b46ad90f07f23c3fe9))

## Interaction with TreeKnit output

Each entry covers one TreeKnit output file type and how Phylo.io handles it.

- **Resolved segment trees (Newick)**: load and compare well
  - **Labels**: internal labels such as `internal_50` or `NODE_7` become the categorical "Data" attribute
  - **Resolution artefacts**: zero-length branches from resolution draw as zero-length segments, and resolved polytomies get scores below 1 only at the inserted splits
  - **Comparison of segment trees is the main use**: the score colours branches by clade agreement between segments, which shows reassortment regions without MCCs. Leaves missing in one segment are ignored
  - **Root order**: run "Reroot" and then "Reorder" on one tree. The root halves can stay crossed (see "Defects")
- **Resolved segment trees (NEXUS)**: not readable. The format choice is Newick, Extended Newick, or PhyloXML
- **ARG in extended Newick**: TreeKnit writes it with `#H` hybrid nodes and `[&segments={0,1}]` annotations ([packages/treeknit-io/src/arg.rs#L12-L33](../../packages/treeknit-io/src/arg.rs#L12-L33)). The file fails to load (observed)
  - **Cause**: the comma inside `{0,1}` splits the annotation into a sibling, and the parser then reads past the end of the tree. Both the Newick and the NHX reader fail with "Cannot read properties of undefined (reading 'children')", and no message is shown
  - **ARG without annotations** (observed with `((A:1.0,(B:0.5)R1#H1:0.5)n1:1.0,(R1#H1:1.0,C:2.0)n2:1.0)GlobalRoot:0.0;`): loads as a tree with one internal node `R1#H1` and one extra leaf `R1#H1`. The reticulation is not visible, and the duplicate leaf name is dropped from the distances
  - **Annotations without braces**: `x[&segments={0,1}]` inside a label produced a spurious leaf named `1}]` (observed)
- **MCC files (MCCs.json), Auspice JSON, SVG figures**: not readable
  - **MCC colouring**: MCC membership can be loaded as a categorical column through a mapping file (`id`, `mcc`), which colours leaves by MCC in both trees with one shared colour map
  - **MCC collapse**: "Collapse monocolored subtrees" then collapses clades inside one MCC

## Ideas from the issue tracker and history

### Open requests

- **Tanglegram between two trees that share some tips** ([#15](https://github.com/DessimozLab/phylo-io/issues/15), 2025, no answer)
- **Hyperlinks on leaves** to a genome browser ([#16](https://github.com/DessimozLab/phylo-io/issues/16)): the maintainer notes that clicks already open the label editor and asked for a contribution
- **Documentation of the distance metrics** ([#18](https://github.com/DessimozLab/phylo-io/issues/18))
- **Comparison that never finishes** for 498 and 497 taxa ([#13](https://github.com/DessimozLab/phylo-io/issues/13), 2021)
- **Distinct categorical colours** ([PR #17](https://github.com/DessimozLab/phylo-io/pull/17), unmerged)
- **Multi-selection of nodes**: a contributor asked in 2019 how to select two nodes. The maintainer said it "would require a lot of work" ([#9](https://github.com/DessimozLab/phylo-io/issues/9))

### Declined

- No request was explicitly declined. Display of speciation and duplication events, requested in 2019 ([#8](https://github.com/DessimozLab/phylo-io/issues/8)), was implemented in version 2

### Features found only in the history

- **Version 1 (2015 to 2018, branch `v1`)**: a jQuery application with these features that version 2 does not have:
  - **Exact BCN search** over all overlapping nodes (see "Two trees")
  - **Branch colouring by support values** with a separate red ramp [[src](https://github.com/DessimozLab/phylo-io/blob/a68d54126234fe7d17bf08a65226779e037e2b87/www/js/treecompare.js#L2148-L2175)]
  - **Ladderize** (branch `Andrei`, 2017, "Ladderization function added")
  - **Sharing through GitHub Gist**, with the tree stored as extended Newick and the view state as metadata
  - **Green highlight** of the selected subtree and its counterpart
- **Branch cutting in compare mode** (branch `remove_branches`, 2017): merged into trimming
- **Transition banner**: the old site was kept with a banner announcing the new version ([`8ed66d4`](https://github.com/DessimozLab/phylo-io/commit/8ed66d453e9a56acb28ac01b590506734afe3899), 2022)
- **Help modal**: the library's built-in help modal still says "Help section is under construction". The help moved to the README ([`7a05abf`](https://github.com/DessimozLab/phylo-io/commit/7a05abf)) [[src](https://github.com/DessimozLab/phylo-io/blob/2b00e3aa3cd03f79913f4f972b30cfb3b4038263/src/api.js#L31-L54)]

## Open scientific problems

- **Clade similarity depends on the root**: Jaccard similarity of clades changes with rooting, so unrooted trees need a matching rooting first. The automatic "Reroot" uses one BCN and gives no guarantee
- **Best matching is local**: each node takes its own best match, so two nodes can share one BCN and the matching is not one-to-one. A global matching (as in MCCs) would avoid double use
- **Approximate search versus exact score**: the LSH speeds up large trees but can lower scores, and the user cannot tell which scores are approximate
- **Leaf order optimization**: the single-pass end-leaf rule does not minimize crossings. Tanglegram untangling (one-sided crossing minimization) is not attempted
- **Partial leaf overlap**: restricting to the intersection hides the clades that differ only by unshared leaves, which can be the biological signal

## Not covered by Phylo.io

- **Two-tree display**:
  - **Link lines between leaves** of the two trees, and any tanglegram layout
  - **Shared axis** for the two trees, and any time or date axis
  - **Comparison of more than two trees** at once
- **Data models and formats**:
  - **Networks, hybrid nodes, and ARGs**
  - **NEXUS, multi-tree files, and BEAST annotations**
- **Layout and rendering**:
  - **Circular or unrooted layouts**
  - **Support-aware rendering**: no support threshold, no collapse of weak branches

## Method and limits

- **Read**: the sources below
  - All non-vendor source files at `2b00e3a` (`src/*.js`), the README, and the website example page
  - The tests (`rerooting.test.js`, `distance.test.js`, `LRF.test.js`) and the version 1 `treecompare.js`
  - The branch list, the commit log, all 13 issues, all 5 pull requests, and the 2016 paper text
- **Ran**: no local trial. The live application was used in Chrome with the built-in examples and small test Newick strings typed into the tool
  - **Parsing tests**: model objects were created through the page's own `phylo._create_model` function to test parsing
  - **Sharing**: no share link was created and no session was uploaded
- **Gaps**:
  - The phylo.io website backend (session storage, the page shell) is not public, so the sharing server limits are known only from the client code
  - No paper about version 2 was found
  - The vendored phyloXML reader was not read in detail
  - The browser tab used for the trials could not be closed: the close command timed out twice
