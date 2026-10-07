# IcyTree feature survey: tree rendering, semantics, and edge cases

This report describes IcyTree, a browser-based viewer for rooted phylogenetic trees and networks, as an idea inventory for the web app. It records what IcyTree does, how it does it, which scientific conventions it assumes, which inputs break it, and which problems its maintainer left open. It does not compare IcyTree with `packages/web`.

IcyTree is licensed GPL-3.0. Its behavior and design may be studied, but copying its code needs approval (see the project rules).

- **Source**: [tgvaughan/icytree](https://github.com/tgvaughan/icytree) at commit `af1836d` (2026-06-01), 529 commits since 2013. Source links point to this commit
- **Live application**: <https://icytree.org>, operated in Chrome on 2026-10-07. Its JavaScript files are byte-identical to commit `af1836d`, so the observations and the source describe the same program
- **Other sources**: all 56 issues and 3 pull requests with their comments, the commit diffs, the IcyTree paper ([Vaughan 2017](https://doi.org/10.1093/bioinformatics/btx155)), and the papers it relies on for rerooting ([Czech et al. 2017](https://doi.org/10.1093/molbev/msx055)) and for networks ([Cardona et al. 2008](https://doi.org/10.1186/1471-2105-9-532))
- **Evidence labels**: "Observed" means seen in the live application. "Derived" means read from the code without running it. Where the manual and the code disagree, the code wins and the difference is noted
- **Shape of the code**: a single-page app on jQuery UI. The whole tree is one SVG element that is rebuilt on each style change. Layout, drawing, and interaction live in [`js/treelayouts.js`](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treelayouts.js), [`js/treedrawing.js`](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js), and [`js/icytree.js`](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js)

## Summary

- **Scope**: the maintainer defines IcyTree as a fast viewer for rooted time trees and networks, and declines publication-figure and inference features ([#50](https://github.com/tgvaughan/icytree/issues/50), [#55](https://github.com/tgvaughan/icytree/issues/55), [#57](https://github.com/tgvaughan/icytree/pull/57))
- **Scientific conventions** (see "Scientific semantics"):
  - **Time comes only from branch lengths**: one missing length turns the whole tree into a cladogram. Date or height annotations are never used for positions
  - **One attribute map per node**: node attributes, edge attributes, and Newick labels share one namespace, and an edge takes the attributes of the node below it. Rerooting therefore misplaces clade support, a problem the maintainer documents but will not fix
  - **Zero-length edges are collapsed by default**: this shows sampled ancestors and polytomies, but it overwrites the parent's annotations and hides zero-length tips
- **Risks for TreeKnit users**: networks with a zero-length hybrid edge fail or lose the reassortment (issue #53, still broken), and TreeKnit's ARG output can contain such edges (see "Interaction with TreeKnit output")
- **Display and interaction**:
  - **Colour**: attribute-driven and categorical only. Every distinct value gets its own hue, also for numbers
  - **Zoom**: Shift+wheel zooms time only, Ctrl+wheel zooms the leaf axis only, and labels keep their pixel size
  - **Edge actions**: hover shows a statistics table. A click collapses the clade, Shift+click reroots, and right-click rotates the children
- **Silent failures on real inputs**: FigTree colour annotations, non-Latin-1 attribute values, empty annotation values, and some NEXUS dialects fail or render wrongly without a useful message (see "Defects")

## Layouts

### Coordinate system

- **Orientation**: a node height maps to the horizontal position, `x = (1 - h) * width`. The root is at the left and the youngest node at the right. Leaf order maps to the vertical position [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L107-L112)]
- **Heights relative to the youngest node**: heights are computed from the root by subtracting edge lengths, then shifted so that the node farthest from the root has height 0. Serially sampled trees therefore draw correctly, and "age" always means "time before the youngest node", not before the present [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/tree.js#L145-L166)]
- **Time tree detection**: a tree counts as a time tree when every edge has a length. Otherwise the cladogram layout is forced with the notification "Switching to Cladogram Layout" (observed with `((A,B),(C,D));`), and the lengths that are present are ignored too. These menu entries are then disabled [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L747-L767)]:
  - the whole "Tree layout" submenu
  - the axis, the log scale, and the error bars
  - the lineages-through-time and skyline plots
- **Root edge stub**: the root edge is drawn to the left border. A root length of exactly 0 is treated as missing, and a missing root length becomes a stub of 1% of the root height [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treelayouts.js#L44-L52)] [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treeparsing.js#L63-L65)]
- **Layout on a copy**: each redraw copies the tree and applies zero-length-edge collapsing and sorting to the copy. Reroot and rotation change the loaded tree itself, so they also change exports [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treelayouts.js#L28-L42)]

### Layout modes

The Style > "Tree layout" submenu selects the mode. The `'` key cycles forward and `"` backward.

- **Standard time tree** (default): leaves sit on evenly spaced rows. A parent is placed at the **mean** of the rows of its children, so in a polytomy it is pulled toward the side with more children. Observed with `((A:1,B:1):1,C:2,D:2);`: the root is at 0.611 of the height, the mean of 0.167, 0.667, and 1.0. The midpoint of the outer children would be 0.583. The manual says "mid-way between each child", which holds only for two children [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treelayouts.js#L282-L309)]
- **Transmission tree**: the parent takes the row of its first child, so a horizontal line follows one host (the infector lineage) through time. This layout disables node sorting, because the child order of the input carries meaning [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treelayouts.js#L312-L338)]
- **Cladogram**: the horizontal position is the rank of the node. A leaf has rank 0 and an internal node one more than its highest child, so all leaves align at the right. Ranks are divided by `1.01 * rootRank`, so the root edge is ignored. In networks, hybrid nodes that share a rank are spread by fractions of a rank. This rule took five rewrites in 2016 and 2017 [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treelayouts.js#L349-L449)]

### Child order

- **Sorted (descending)** (default): children with larger clades are placed below children with smaller clades, so a serially sampled time tree descends from top left to bottom right. Clade size counts **all nodes**, internal nodes included. Observed: in `((A,B,C,D,E),((F,G),(H,I)))` the four-leaf clade (7 nodes) is sorted as larger than the five-leaf polytomy (6 nodes) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/tree.js#L285-L313)]
- **Sorted (ascending)**: the reverse order
- **Unsorted**: input order. NeXML defines no child order, so this order is arbitrary for NeXML trees
- **Manual rotation**: a right-click on an edge moves the first child of the node below the edge to the end of its children. Observed: in sorted mode the click has no visible effect, but the rotation is stored and appears later when the user switches to Unsorted [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L1117-L1126)]
- **Sort setting persists across files**: the sort mode is a global menu state, so a new file opens in the mode of the previous one (observed)

### Topology simplification before layout

- **Collapse zero-length edges** (default on): a child at exactly the same height as its parent is merged into the parent, and its children move up. This shows sampled ancestors and polytomies that are encoded as zero-length edges in a binary tree. See "Zero-length edges" for the consequences [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/tree.js#L359-L387)]
- **Mark singletons** (`m`): a node with exactly one child (an ancestral migration or a sampled ancestor) is drawn as a filled circle. Without the mark, such a node is invisible when its two adjacent edges have the same colour [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L854-L859)]

## Edges, nodes, and marks

- **Edge shape**: each edge is one SVG path that runs horizontally from the child to the time of the parent, then vertically to the parent row (an "elbow" edge). There is no separate vertical connector per parent [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L485-L514)]
- **Non-scaling strokes**: edges, error bars, and axis lines use `vector-effect: non-scaling-stroke`, so their width stays constant at any zoom level. Inkscape ignores this property, so exported SVGs of a stretched view show distorted line widths ([#43](https://github.com/tgvaughan/icytree/issues/43), open)
- **Edge width** (`+`/`-`): uniform width in steps of 1 px, minimum 1 px [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L1188-L1193)]
- **Relative edge width by attribute** (`o`/`O`): width = `minWidth + (value / max) * (lineWidth - minWidth)`, with `minWidth` = 1 px. The manual says values must lie between 0 and 1, but the code divides by the maximum. This option replaced "edge opacity by attribute" in 2018 ([`6f39ca2`](https://github.com/tgvaughan/icytree/commit/6f39ca2)), but the keyboard help still calls `o` and `p` "opacity" keys [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L707-L717)] [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/index.html#L465-L466)]
- **Node age error bars** (`b`/`B`): any internal-node attribute with two elements (for example a BEAST `height_95%_HPD`) can be drawn as a horizontal bar on the time axis. The bar is three times the edge width, has 40% opacity, is drawn under the edges, and follows the log scale. Bars are disabled in the cladogram ([#37](https://github.com/tgvaughan/icytree/issues/37)) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L672-L689)]
- **Node marks**: circles with a radius of `0.5 * lineWidth + sqrt(1.5 * lineWidth)` pixels, resized by hand on every zoom so they keep their size ([#30](https://github.com/tgvaughan/icytree/issues/30)). A mark is drawn for a singleton when singleton marking is on, for each node that has a value of the node colour attribute, and at both ends of recombinant edges [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L1457-L1469)]
- **Anti-alias toggle**: edges use `shape-rendering: crispEdges` by default for sharp lines. Anti-aliasing helps when there are more leaves than vertical pixels [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L1796-L1797)]

## Labels

- **Tip text** (`t`/`T`): None, Label (the default: Newick label or NEXUS taxon name), or any attribute. A leaf without the attribute gets an empty label [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L870-L894)]
- **Internal node text** (`i`/`I`): off by default. A singleton gets its label above the edge, and a node with several children gets it to the right of the node, both at an offset of 2.5 edge widths that stays constant under zoom ([#31](https://github.com/tgvaughan/icytree/issues/31)). Empty values are skipped [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L922-L959)]
- **Internal labels stay text**: a Newick internal label such as `95` is never interpreted as a support value. IcyTree has no notion of support, so it cannot treat support specially on reroot or collapse
- **Label precision limit**: None, or 2, 3, 4, or 5 significant figures for numeric labels. Trailing zeros are removed and non-numeric text is kept [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L28-L44)]
- **Angled labels** (`v`): all node labels rotate by -45 degrees, which helps when singleton labels overlap the edges. Very long angled labels are cut off, because the fit measures labels without their rotation ([#44](https://github.com/tgvaughan/icytree/issues/44), open) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L565-L568)]
- **Font size** (`[`/`]`): steps of 2 px, minimum 5 px, default 11 px. The legend has its own font size (`(`/`)`, [#57](https://github.com/tgvaughan/icytree/pull/57)) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L1195-L1207)]
- **No label culling**: every label is drawn at every zoom level. Labels of dense trees overlap until the user zooms in vertically (observed with the H3N2 example, 980 text elements)
- **Fit includes labels**: the initial view widens the bounding box so that the longest label at the right fits: `W' = (x_text - x0) / (1 - w_text / W)`, because labels keep their pixel width while the tree scales ([#21](https://github.com/tgvaughan/icytree/issues/21)) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L1300-L1321)]

## Colour and legend

- **Colour edges by** (`c`/`C`) and **colour nodes by** (`k`/`K`): independent selections. Each offers None, Label, and every attribute. An edge takes the value of the node below it. Edges and nodes without a value stay black (light grey in dark mode) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L447-L463)]
- **Palette** [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L153-L231)]:
  - **Order**: the distinct values are sorted numerically when all are numeric, otherwise lexicographically
  - **Hues**: each value gets a hue on an even spacing, with a step of `min(0.33, 1/N)`, counting down from red
  - **Legibility**: yellow and green hues (0.1 to 0.5) are drawn darker (lightness 0.30 instead of 0.45) to stay readable on white. Observed for three values: `#e50000`, `#0400e5`, and the darkened green `#009906`
  - **Dark mode**: the saturation is halved
- **Colours are not stable**: a value's colour depends on how many distinct values exist, so the same value gets different colours in different trees of one file
- **Numeric detection is all-or-nothing**: one non-numeric value such as `NA` makes the whole attribute lexicographic, so `10` sorts before `9`, and `1.0` and `1` are separate values. An 18-month bug made every attribute count as numeric until [`b47394a`](https://github.com/tgvaughan/icytree/commit/b47394a) (2016)
- **Categorical only**: numeric attributes also get one hue per distinct value. Observed: colouring the H3N2 example by Label gives a legend with 2044 entries and 1191 distinct stroke colours, which the eye cannot tell apart. A mapping that follows the value distribution was proposed in 2018 and is still open ([#46](https://github.com/tgvaughan/icytree/issues/46))
- **Legend** (`g`): drawn in the bottom-left corner of the current view and rebuilt on every zoom. Edge entries are short bars and node entries are circles, each label in the colour of its value. The title is "Edge colour: <Attribute>" (observed: "Edge colour: Deme"). When both are active, the node legend stacks above the edge legend [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L265-L362)]

## Time axis and scale

- **Axis modes** (`a`/`A`): None (default), Age (values increase to the left), or Forward time (values increase to the right). Observed on H3N2: Age shows 0 to 7, Forward time shows 0 to -7, because the youngest tip is 0 by default [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L364-L438)]
- **Axis offset**: a dialog sets the age or date of the youngest node. Observed: offset 2008.5 with Forward time gives ticks at the round years 2008, 2007, and so on, not at 2008.5 ([#39](https://github.com/tgvaughan/icytree/issues/39), fixed in [`afc1e0a`](https://github.com/tgvaughan/icytree/commit/afc1e0a)). Label precision was raised to 10 digits for year-like offsets ([`7737e0d`](https://github.com/tgvaughan/icytree/commit/7737e0d))
- **Viewport-aware ticks**: ticks are recomputed on every pan and zoom for the visible time range only, with at most 20 ticks. Observed: zooming from 1 to 5.6 changed the ticks from whole years to steps of 0.1 year. The spacing is `10^ceil(log10(range / 19))`, always a power of ten, with no steps of 2 or 5 ([`9c888db`](https://github.com/tgvaughan/icytree/commit/9c888db), [#12](https://github.com/tgvaughan/icytree/issues/12)) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L390-L405)]
- **Grid lines**: each tick is a grey vertical line across the visible height, with its label at the bottom
- **Offset log scale** (`s`): magnifies the recent part of coalescent-like trees [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treelayouts.js#L54-L63)] [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L1803-L1820)]
  - **Formula**: the scaled height is `ln((h + c) / c) / ln((T + c) / c)`, where `T` is the total height and `c` is a fraction of `T` (default 0.001)
  - **Strength**: **Alt + mouse wheel** multiplies or divides the fraction by 1.5 and redraws (observed: 0.001 to 0.00067)
  - **Ticks**: evenly spaced on screen and labelled in exponential notation
  - **Serially sampled trees** (observed): the magnified region is the time just before the single youngest tip. On the H3N2 example almost the whole tree is squeezed into the left third, and only one tip's branch uses the rest of the width
  - **Labels with an offset** (observed): with offset 2008.5, the five-significant-figure exponential labels repeat (`2.0085e+3` three times), so the axis cannot be read

## Networks (recombinant edges)

IcyTree reads extended Newick hybrid nodes (`#H1`). The occurrence that has children is the source node, and each leaf occurrence is a destination whose parent is an additional parent of the source. IcyTree calls the additional parental edges "recombinant edges" [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/tree.js#L222-L279)].

- **Dashed drawing**: a recombinant edge runs from the source node to the row of the destination leaf, then horizontally to the time of the additional parent, then vertically to it. The dash pattern scales with the edge width (observed: `6, 2` at width 2). Dashing tells real coalescences apart from crossings of lines [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L516-L551)]
- **Each recombination reserves a row**: the destination leaf keeps its own row. Observed on the ARG example: 34 rows for 20 sampled tips and 14 recombinations
- **End marks**: a circle marks the source node and, in inline mode, the additional parent [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L820-L829)]
- **Display recombinant edges** (`d`, default on): hides all recombinant edges, which is useful when the primary parent carries the meaning (for example the clonal frame in Bacter output). Observed: the 14 reserved rows stay as empty gaps, and the primary edges keep their detours
- **Recombinant edge text** (`n`/`N`) and **recombinant edge width** (`p`/`P`): label or width from an attribute of the destination leaf, the same as for normal edges
- **Inline recombinant edges** (`w`, default off): a parent of a destination leaf is placed on the row of its non-recombinant child instead of between its children. A node whose children are all destinations takes its first child's row [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treelayouts.js#L289-L301)]
- **Minimize recombinant edge length** (`f`, default on): each destination leaf is moved to the side of its parent that faces the source node, which shortens the dashed line [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/tree.js#L337-L357)]
- **Collapse in networks**: "clade" is not well defined in a network, so a collapsed clade hides only descendants that are not reached through recombinant edges. A destination leaf inside a collapsed clade whose source is outside it gets its own row next to the triangle. A node whose children are all destinations cannot be collapsed [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treelayouts.js#L123-L162)]
- **Network reroot**: rerooting a network can turn nodes into new hybrid pairs with fresh IDs. Destination lengths are then changed so that each destination ends at its source's height, which can make lengths negative, as the warning dialog says. Rerooting on the edge above a destination is blocked ([`65ddda5`](https://github.com/tgvaughan/icytree/commit/65ddda5)) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/tree.js#L389-L508)]

## Zoom and pan

- **Wheel zoom around the pointer**: each wheel event changes the zoom by a factor of 1.1, and the point under the pointer stays fixed. The zoom cannot go below 1, the fitted view (observed: 40 zoom-out steps end at exactly 1) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L1471-L1523)]
- **Single-axis zoom**: observed with synthetic wheel events, **Shift** + wheel changes only the horizontal (time) zoom, and **Ctrl** + wheel only the vertical (leaf) zoom. Vertical-only zoom spreads the leaves of a dense tree apart without stretching time. The keys were swapped once ([`ae35b0c`](https://github.com/tgvaughan/icytree/commit/ae35b0c), 2018)
- **Drag to pan**: the view centre is clamped so that the view never leaves the tree bounding box [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L1369-L1394)]
- **Reset zoom** (`z`)
- **Constant-size text and marks**: the SVG uses `preserveAspectRatio="none"`, so the zoom stretches the geometry. SVG has no non-scaling text, so each label gets a counter-transform that keeps its pixel size and its pixel offset from its node. Observed: a tip label stays 12 px high at every zoom [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L1416-L1469)]
- **Persistent view**: zoom and pan survive style changes and switching between the trees of a file (observed). A window resize redraws at the new size [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L1343-L1360)]
- **Full redraw per wheel event in Chrome**: Chrome 67 ignored `non-scaling-stroke` after view changes ([#49](https://github.com/tgvaughan/icytree/issues/49)), so every wheel event in Chrome reruns the whole layout and SVG build. The "temporary" workaround from 2018 ([`ae7f2f9`](https://github.com/tgvaughan/icytree/commit/ae7f2f9)) is still active [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L1520-L1522)]

## Edge interaction

All tree modifications start from an edge. The edge identifies the node below it.

- **Hover highlight and statistics**: the hovered edge is drawn at three times the edge width (observed: 2 px to 6 px). A table shows the branch length, parent age, child age, child label, and all child attributes. It opens on the side of the pointer that faces the window centre, and the wheel does not zoom while the pointer is over it [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L1131-L1258)]
- **Stale statistics table** (observed): after a click collapses or reroots, the table stays on screen with the old values, because the edge under the pointer was removed by the redraw
- **Click: outline the clade**: the clade is replaced by a grey triangle that spans the rows of all its leaves, from the clade root to the youngest descendant (observed on the primates example) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treelayouts.js#L218-L257)]
- **Alt+click: collapse the clade**: the same triangle in one leaf row. Observed: Alt+click on an outline switches it to the one-row form, and a plain click restores the clade [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L1090-L1109)]
- **Collapsed clades have no label**: the triangle shows no tip text, no count, and no name. Automatic collapse of large clades was proposed with the feature and never built ([#38](https://github.com/tgvaughan/icytree/issues/38))
- **Shift+click: reroot**: a new root is placed at the midpoint of the edge, and all collapsed clades are expanded. Observed on primates: the edge of length 0.1207492 became two edges of 0.0603746, with no dialog. A dialog with Abort and Continue appears only when the tree has `[&...]` annotations (see "Rerooting and support values") [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L1022-L1088)]
- **Right-click: rotate the children** (see "Child order")

## Search and highlighting

Search > "Find nodes" opens a non-modal dialog [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L428-L524)].

### Query

- **Several terms at once**: a comma-separated list. Each term marks its matches with its own value (1, 2, and so on), so each term gets its own colour
- **Attribute choice**: Label or any leaf attribute
- **Match options**: whole value or substring, and case-sensitive (default on)
- **Terms are regular expressions**: the substring test uses `String.search`. Observed: `^M_.*a$` matches only `M_mulatta`. A term such as `A.1` also matches `AB1`, and a term with an unbalanced `(` throws

### Results

- **Scope**: one of three [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/tree.js#L566-L613)]:
  - the matching nodes only
  - the matching nodes plus every ancestor whose descendants all match (monophyletic groups)
  - the matching nodes plus all their ancestors (paths to the root)
- **Shared ancestors take the last term** (observed): with "Include all ancestors" and the terms `Homo,Pongo`, the path above the common ancestor of both is coloured as Pongo
- **Results as an attribute**: matches are written into an annotation of the loaded tree (default key `HIGHLIGHT`, editable), and edge colouring switches to it (observed). The highlight therefore appears in the legend and in NEXUS, PhyloXML, and NeXML exports
- **Taxon names are not highlighted**: on very short terminal branches the coloured edge is hard to find. The maintainer agrees, but taxon names have no styling system ([#58](https://github.com/tgvaughan/icytree/issues/58), open)
- **Clear**: removes the annotation and resets edge colouring to None

## Multiple trees

- **Tree navigator**: a semi-transparent box shows "Tree number: i of N" when the file holds more than one tree. On hover it shows first, previous, next, and last buttons and a number field [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L1237-L1278)]
- **Keys**: `.` and `,` step one tree, and `>` and `<` jump 10% of the trees. Observed on 100 trees: the `>>` and `<<` buttons also jump 10 trees, despite labels that suggest "last" and "first" [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L1209-L1221)]
- **Shared style**: all trees use the same style and view. The attribute menus are rebuilt for the tree on screen, and the selection is kept when the new tree has the same attribute [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L1079-L1186)]
- **No side-by-side view**: one tree is on screen at a time. There is no tanglegram or comparison of two trees

## Styles, shortcuts, and feedback

- **Single-key shortcuts**: one key per menu item. For a submenu, the lowercase key selects the next entry and the uppercase key the previous one. Help > "Keyboard shortcuts" lists them, and each menu entry shows its key. Keys work only while the page body has focus, so after a dialog or a page load the user must click an empty area first (observed) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L1825-L2131)]
- **Change notification**: each style change shows a centred message for one second, for example "Colour edges by: deme" or "Log scale: ON" (observed) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L957-L965)]
- **Disabled entries**: menu entries that do not apply (the axis in a cladogram, sorting in a transmission tree) are greyed out instead of hidden [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L730-L777)]
- **Named styles**: "Save current style" stores the style under a name in `localStorage`, so it persists across sessions. `{` and `}` cycle through saved styles. The layout and the sort order are not saved. See "Defects" for the restore bug [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L1479-L1613)]
- **Dark mode** (`x` or the toolbar button): persisted in `localStorage`. Edges, labels, and error bars become light grey, and the palette loses half of its saturation, because the light-mode palette avoids light hues that dark mode needs ([#54](https://github.com/tgvaughan/icytree/pull/54))
- **Self-hosted assets**: a user behind a firewall that blocked CDNs could not load the site ([#51](https://github.com/tgvaughan/icytree/issues/51)), so all libraries are served from the site since [`703553c`](https://github.com/tgvaughan/icytree/commit/703553c) (2022). The paper stresses that no tree data leaves the browser

## Input

- **Load paths**: file dialog (`l`), drag and drop of a file or of dragged text, a paste dialog (`e`), a URL dialog (`u`), and the page parameter `?url=<file>`, which makes shareable links. When a URL fails for a reason other than 404, IcyTree retries through a public CORS proxy. An HTML error page from the proxy was once parsed as a tree ([#47](https://github.com/tgvaughan/icytree/issues/47)) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L805-L833)]
- **Format detection**: `#NEXUS` at the very first byte, then the XML root element (`phyloxml`, `nexml`), else Newick [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treeparsing.js#L651-L687)]
- **Large inputs**: an input over 500,000 characters shows a "Loading..." screen first. Parsing in a Web Worker was dropped, because SVG rendering, not parsing, dominates the time for big trees ([#36](https://github.com/tgvaughan/icytree/issues/36)) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L1294-L1313)]
- **Parse errors**: a modal dialog with the message. Newick errors include the position and the surrounding text ([#42](https://github.com/tgvaughan/icytree/issues/42)) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treeparsing.js#L175-L203)]
- **Attach metadata from CSV**: the user picks the key column, and each other column becomes a string attribute of every node, internal nodes included, whose label equals the key. The join applies to all loaded trees and is repeated after a reload ([#52](https://github.com/tgvaughan/icytree/pull/52)) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L1333-L1353)]

## Export

- **SVG of the current view**: the export contains exactly what the window shows, including zoom, pan, axis, and legend [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L1355-L1374)]
- **PNG and JPEG**: the SVG is rasterized at the size of the window. There is no resolution option [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L1391-L1432)]
- **Tree files**: Newick (topology, labels, and lengths only), NEXUS with `[&key=value]` annotations, PhyloXML, and NeXML. Networks export only to Newick and NEXUS. The export holds the tree on screen with its reroots, rotations, and search annotations, but without sorting or collapsing [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L1434-L1477)]

## Statistics

The manual marks these features as experimental.

- **Lineages through time**: a step plot of the lineage count against age. The count changes by `1 - (number of children)` at each node, which handles polytomies, singletons, and sampled ancestors [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/tree.js#L615-L633)]
- **Skyline plot**: the classic skyline ([Pybus et al. 2000](https://doi.org/10.1093/genetics/155.3.1429)) extended to non-ultrametric trees, with a minimum interval size. The size can be optimized over 100 grid values by maximizing an AICc-corrected likelihood ([`b778ce6`](https://github.com/tgvaughan/icytree/commit/b778ce6)). "Smooth" only draws a spline through the steps. See "Defects" for an interval error [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treeplots.js#L57-L176)]
- **Tree statistics**: counts of leaves, internal nodes, and nodes with out-degree 1, 2, and more than 2, root height, tree length, cherry count, and Colless imbalance. Colless counts binary nodes only and is not normalized. In networks, destination leaves count as leaves [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treestats.js#L27-L125)]

## Scientific semantics

### Node attributes and edge attributes

- **One attribute map per node**: every `[&...]` block of a node goes into one map, whether it describes the node or the edge above it. The maintainer calls this "one set of annotations per node+(parent edge)" ([#56](https://github.com/tgvaughan/icytree/issues/56)). A node key and an edge key with the same name overwrite each other
- **Three annotation positions**: after the label (BEAST node annotations), after the branch length (BEAST 1 edge annotations), and directly after the colon (MrBayes edge annotations, added in 2026 by [`eef1172`](https://github.com/tgvaughan/icytree/commit/eef1172) for [#56](https://github.com/tgvaughan/icytree/issues/56)). MrBayes uses different keys for node and edge values (`prob...` and `length_...`), so its files do not collide
- **All values are text**: numbers stay strings until a feature converts them. Vectors `{a,b}` become arrays, and an empty value becomes `null`

### Rerooting and support values

- **The problem**: support values describe branches (bipartitions), but Newick stores them as internal node labels. Rerooting reverses every branch on the path between the old and the new root, so the labels on that path must move to the other adjacent branch. [Czech et al. (2017)](https://doi.org/10.1093/molbev/msx055) tested 20 tools and found 8 that reroot wrongly. A published study had wrong support values because of it
- **IcyTree's behavior**: labels and annotations stay on their nodes (the "node interpretation"). An IQ-TREE tree rerooted in IcyTree shows support on the wrong branches, while FigTree shows it correctly ([#59](https://github.com/tgvaughan/icytree/issues/59))
- **Maintainer's position**: a fix needs "assumptions about the meaning" of each annotation, so the issue was closed as won't-fix and the manual warning was extended to name clade support ([`af1836d`](https://github.com/tgvaughan/icytree/commit/af1836d))
- **Warning gap** (observed and derived): the warning dialog appears only when the tree has `[&...]` annotations. Support values stored as plain internal labels, which is the IQ-TREE and RAxML convention, are rerooted with no warning, which is exactly the case of #59 [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L1028-L1037)]
- **What the paper recommends**: viewers should let the user declare whether labels belong to nodes or to branches, ideally before display or reroot, as Dendroscope 3.5 and later, Archaeopteryx, and ETE do. When the new root lands on a branch that has a support value, both new root branches should show it. Some labels, such as clade names, are neither node nor branch values and no format can say so

### Zero-length edges

- **What zero length means is ambiguous**: it can encode a sampled ancestor (a sampled tip that is a direct ancestor), a polytomy resolved into a binary tree, or a real zero-length branch. IcyTree cannot tell these apart. Two BEAST tutorials give conflicting advice: the FBD tutorial uses the collapsed degree-two-node view, but the Total-Evidence tutorial turns the option off to see sampled ancestors on zero-length branches ([Taming the BEAST](https://taming-the-beast.org/tutorials/))
- **Design reversals**: the maintainer changed this behavior four times:
  - 2016: zero-length leaves were removed at parse time
  - 2018-03: the removal was dropped, because a viewer "should not attempt to interpret trees at a deeper level than what's explicitly there" ([`48927f7`](https://github.com/tgvaughan/icytree/commit/48927f7))
  - 2018-06: it returned as a display option, on by default ([`faaa45c`](https://github.com/tgvaughan/icytree/commit/faaa45c))
  - 2018-11: it was extended to internal edges ([`0b676b9`](https://github.com/tgvaughan/icytree/commit/0b676b9)) and to chains of zero-length edges ([`77d2354`](https://github.com/tgvaughan/icytree/commit/77d2354))
  - 2019: the parent takes over the child's label ([`e652567`](https://github.com/tgvaughan/icytree/commit/e652567))
- **The parent's annotations are lost**: the merge replaces the parent's whole annotation map and label with the child's. A polytomy encoded as a zero-length internal edge therefore shows the sub-clade's support on the parent, and the parent's own posterior and HPD values disappear. A source comment asks whether this is right for polytomy dummy nodes [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/tree.js#L368-L373)]
- **Zero-length tips vanish** (observed): `((A:0,B:1):1,C:2);` draws no tip "A". A becomes the label of a singleton node over B, visible only with internal node text on
- **Exact comparison**: the test is `child.height == node.height` on floating-point heights. A length of `1e-8` is kept, but a length below the floating-point resolution of the root height is collapsed
- **Networks**: see issue #53 under "Defects"

### Time, dates, and rooting

- **Branch lengths only**: date and height annotations (for example from MASTER, the very first issue [#1](https://github.com/tgvaughan/icytree/issues/1)) are never used for positions. A single missing length makes all heights NaN, and the tree becomes a cladogram (observed with `((A:1,B):1,C:2);`). Before 2016 a missing length defaulted to 1 ([`e5a1a66`](https://github.com/tgvaughan/icytree/commit/e5a1a66))
- **Negative lengths are accepted silently** (observed): `((A:1,B:-0.5):1,C:2);` draws B to the left of its parent, with no warning
- **Unrooted input**: Newick always implies a root, and IcyTree draws it without a hint that it may be arbitrary. Only the manual warns against reading meaning into it. Unrooted PhyloXML and NeXML trees are skipped on load. The NEXUS `[&U]` flag is ignored. Midpoint rooting was declined as an inference method ([#55](https://github.com/tgvaughan/icytree/issues/55))
- **Hybrid copies at different times**: in a timed network, the two occurrences of a hybrid node can end at different implied times, for example in PhyloNet output. IcyTree then draws diagonal recombinant edges. The maintainer offered to force one copy to the other's time, but "it's not clear which one should be treated as correct" ([#48](https://github.com/tgvaughan/icytree/issues/48)). The cladogram layout is the workaround
- **Rich Newick inheritance probabilities are discarded**: PhyloNet's extra `:support:probability` fields are skipped. Turning them into attributes was rejected because another tool might use the same syntax with a different meaning ([`898acdc`](https://github.com/tgvaughan/icytree/commit/898acdc))

### Extended Newick conventions

- **Cardona et al. (2008)**: a hybrid node with k parents is written k times. One occurrence carries the children, and the others are childless. Each occurrence is `label#[type]i:length`, where the optional type is `R` (recombination), `H` (hybridization), or `LGT` (lateral gene transfer), and each length is measured from that occurrence's own parent. For LGT, the occurrence with the children must be the target of the vertical (non-transfer) edge
- **IcyTree's reading**: the occurrence with children is the source, wherever it appears in the string, and leaf occurrences are destinations. The ID is the whole text after `#`, so `#H1` and `#LGT1` are different IDs and the type is not interpreted. If all occurrences are leaves, the first one in preorder becomes the source. A source without a destination is an error
- **What "primary" means is up to the file author**: IcyTree draws the edge to the source's own parent solid and the others dashed. In Bacter output, solid is the clonal frame and dashed is gene conversion. In other tools the distinction may carry no meaning (manual, "Recombinant edges")

## Defects

Each entry gives the input, the effect, and the evidence. Observed entries were reproduced on icytree.org on 2026-10-07.

### Networks

- **Zero-length hybrid edge** ([#53](https://github.com/tgvaughan/icytree/issues/53), closed in 2025 without a fix): reported with TreeKnit ARG output. Observed with the issue's own string, where the second occurrence of `ARGNode_jnx1q48i#H1` has length `0`: collapsing removes the leaf occurrence, `getRecombEdgeMap` throws the string "hybrid nodes must come in groups of 2 or more", the paste dialog stays open, no error is shown, and the previous tree stays on screen [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/tree.js#L255-L260)]
- **Zero-length edge above a hybrid source** (observed): `((A:2,(B:1)#H1:0):1,(#H1:1,C:2):1);` loads, but the source is merged into its parent without its hybrid ID. The recombinant edge disappears, and the leaf occurrence is drawn as an extra unlabelled tip. Turning "Collapse zero-length edges" off restores the dashed edge
- **`#` in a label** (observed): `(Sample#3:1,B:1);` gives the label `Sample` and the hybrid ID `3`. Two tips ending in the same `#n` become a recombination pair

### Annotations and colour

- **FigTree colour annotations** (observed): `[&!color=#ff0000]` fails with "Expected token CLOSEA but found HASH", because `#` is a token everywhere. Trees saved from FigTree with colours cannot be loaded [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treeparsing.js#L85)]
- **Non-Latin-1 attribute values** (observed): colouring by an attribute with the value `Αθήνα` throws `InvalidCharacterError` from `btoa`, which builds CSS class names, and the window goes blank. Place names such as `Łódź` and any CJK text trigger it, but `Zürich` does not [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L497)]
- **Empty annotation value on an internal node** (observed): `((A:1,B:1)[&x=]:1,C:2);` fails to display with "Cannot read properties of null (reading 'length')", because the error-bar menu filter reads `.length` of the `null` value. The same filter accepts any two-character string as an interval [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L1119-L1124)]
- **Array-valued colour attributes** (observed): colouring by `v={1,2}` leaves those edges without any stroke, so they become invisible, and the legend is empty

### Newick and NEXUS syntax

- **Plain Newick comments** (observed): `(A[note],B);` fails with "Error reading character [". Only the NEXUS reader removes comments
- **Two trees on one line** (observed): `(A,B);(C,D);` loads one tree with no message, because trees are split on `;` followed by a newline [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treeparsing.js#L566)]
- **Quote escapes** (observed): `'O''Brien''s'` becomes `O'Brien''s`, because only the first doubled quote is unescaped. Unquoted labels keep trailing spaces: `(A ,B)` gives the label `A ` [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treeparsing.js#L123)]
- **NEXUS dialects**: inputs that load wrongly or not at all [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treeparsing.js#L586-L640)]:
  - **Translate table applied to internal labels** (observed): with `translate 3 Gorilla`, an internal support label `3` is renamed to `Gorilla` [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/tree.js#L544-L551)]
  - **PAUP-style `utree`** (observed): `utree t1=(A:1,B:1);` in a NEXUS trees block produces a one-node tree, because the tree-line regex is not anchored and the slice ignores the match position [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treeparsing.js#L629-L633)]
  - **Header after a byte-order mark or whitespace** (derived): `#NEXUS` is not detected, and the file is parsed as Newick
  - **Tree names** (derived): names other than word characters and `.` (for example `'Tree 1'` or `rep-1`) are skipped silently
  - **Line breaks** (derived): lines are joined without a space, so a label split across lines loses its space
  - **Several trees blocks** (derived): only the first one is read

### Numbers and NeXML

- **Unusual branch lengths** (observed): `0x1A` is read as 26, and `Infinity` is accepted and makes all heights NaN while the tree still counts as a time tree
- **NeXML** (observed and derived): edge lengths stay strings, so Statistics > "Tree length" shows a concatenated string such as `013.230930.947...` for the NeXML example (observed). Edge `<meta>` annotations are attached to the wrong node (derived) [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treeparsing.js#L489-L505)]

### Display and state

- **Saved styles do not restore everything**: the redraw reads some settings from the menus, and the restore does not update those menus [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L1530-L1571)]
  - **Angled labels and label precision** (observed): a style saved with both stores `angleText: true` and `labelPrec: 3`, but applying it leaves both off
  - **Error bars and legend font size** (derived): not restored, for the same reason
  - **Sort order** (derived): the key is saved as `sortNodesDecending` but read as `sortNodesDescending`
- **Stale hover table** after collapse or reroot (observed, see "Edge interaction")
- **HTML in labels is rendered** (observed with `<i>x</i>`): the hover table and the error dialogs insert labels, attribute values, and parse-error context as HTML. A tree file with a crafted label can therefore run script in the viewer [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treedrawing.js#L1220-L1239)] [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/icytree.js#L934-L954)]

### Export and statistics

- **Exported cladograms get zero lengths** (derived): a missing length is written as `:0.0`. On re-import the tree is a time tree with all heights 0, and the default collapse merges every edge [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treewriting.js#L65-L68)]
- **Labels are written in double quotes** without escaping an embedded `"` (derived). Newick and NEXUS quote with single quotes, so other tools may misread the export [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treewriting.js#L40-L41)]
- **Skyline intervals are shifted by one event** (derived, checked by hand): each interval uses the lineage count after its closing event and counts the next event's coalescence. For `((A:1,B:1):1,C:2);` the code gives one interval ending at 1 with Ne = 0.5, but the classic skyline gives Ne = 3 on (0, 1] and Ne = 1 on (1, 2] [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treeplots.js#L97-L135)]
- **Two definitions of tree length** (derived): the statistics dialog sums branch lengths, while `Tree.getLength` sums height differences. They differ for networks and for negative lengths

## Performance

- **Rendering dominates**: the maintainer measured that SVG rendering, not parsing, takes most of the time for big trees ([#36](https://github.com/tgvaughan/icytree/issues/36)). The paper claims trees with tens of thousands of tips stay responsive, without benchmarks
- **Every change rebuilds everything**: each style change copies the tree, collapses, sorts, lays out, and rebuilds the whole SVG. In Chrome each wheel event does too (see "Zoom and pan")
- **Fixes in the history**:
  - **Lexer**: anchoring the token regexes with `^` made lexing ten times faster ([`0f137d0`](https://github.com/tgvaughan/icytree/commit/0f137d0))
  - **Node lookup**: lookup by ID scanned all nodes until a cached map was added ([`7d06f3a`](https://github.com/tgvaughan/icytree/commit/7d06f3a))
  - **Labels**: building labels in a document fragment sped up Firefox ([`5dc592a`](https://github.com/tgvaughan/icytree/commit/5dc592a))
- **Remaining quadratic lexing** (derived): the `#` regex is still unanchored and each token copies the rest of the input, so lexing time grows with the square of the file size [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treeparsing.js#L85)]

## Interaction with TreeKnit output

TreeKnit's documentation recommends opening its extended Newick ARG in IcyTree ([TreeKnit.jl visualization](https://pierrebarrat.github.io/TreeKnit.jl/visualization/)). Issue #53 was reported with that output.

- **Zero-length hybrid edges are possible** (derived): in this port, a reassortment node's edges get `min(b1, b2) / 2` and `b - min(b1, b2) / 2`, where `b1` and `b2` are the lengths of the matching branches in the two trees (`fn set_branch_lengths()` in [packages/treeknit-core/src/arg.rs#L492-L539](../../packages/treeknit-core/src/arg.rs#L492-L539)). When one of them is 0, for example after resolution inserts a split of length 0, the ARG contains a zero-length hybrid edge, which IcyTree's default collapse breaks as described under "Defects"
- **Attributes**: the `[&segments={0,1}]` annotations are vectors, so IcyTree cannot colour by them (array values are invisible) but shows them in the hover table

## Ideas from the issue tracker and history

### Open requests

- **Highlight the taxon names of search hits** ([#58](https://github.com/tgvaughan/icytree/issues/58))
- **Hue spacing that follows the value distribution** ([#46](https://github.com/tgvaughan/icytree/issues/46)): a mapping by quantiles for skewed numeric attributes
- **Bounding box with angled labels** ([#44](https://github.com/tgvaughan/icytree/issues/44))
- **Line widths in exported SVG** ([#43](https://github.com/tgvaughan/icytree/issues/43)): the export would have to bake the stroke counter-scaling into the geometry

### Declined

- **Aligned metadata columns to the right of the tree** ([#50](https://github.com/tgvaughan/icytree/issues/50)): requested for public health epidemiology (contact tracing, cluster definition). Declined because it breaks the assumptions of the viewport code, which the maintainer calls "a huge headache to write"
- **Midpoint rooting** ([#55](https://github.com/tgvaughan/icytree/issues/55)): declined as an inference method
- **Correct support after rerooting** ([#59](https://github.com/tgvaughan/icytree/issues/59)): declined, see "Rerooting and support values"
- **Publication-quality figures** ([#57](https://github.com/tgvaughan/icytree/pull/57)): the maintainer merged separate legend font sizes but warned that further work in this direction "opens a bottomless can of worms"

### Features found only in the history

- **Edge opacity by attribute** ([`eaa0929`](https://github.com/tgvaughan/icytree/commit/eaa0929), 2015): replaced by relative edge width in 2018. It was used by Bacter summaries to show conversion support
- **File polling** ([`f47ff50`](https://github.com/tgvaughan/icytree/commit/f47ff50), 2015): the view followed a file that another program updated, for example a running inference. Removed in [`f032d5f`](https://github.com/tgvaughan/icytree/commit/f032d5f) as unused
- **File reload** ([`02e3a8c`](https://github.com/tgvaughan/icytree/commit/02e3a8c), removed 2020): browsers no longer re-read a changed file, so the command stopped working
- **Big trees in several SVG pages** ([`3802904`](https://github.com/tgvaughan/icytree/commit/3802904), 2014): one export split a tall tree into pages with rescaled text. Removed without a stated reason in [`b9c9882`](https://github.com/tgvaughan/icytree/commit/b9c9882)
- **Shuffle child order** ([`32d67cc`](https://github.com/tgvaughan/icytree/commit/32d67cc), 2018): `Tree.shuffleNodes()` still exists, but no menu entry or key calls it [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/tree.js#L315-L335)]

## Open scientific problems

- **Node or branch meaning of labels**: no file format records whether a label belongs to a node or to the branch above it, and rerooting, collapsing, and edge colouring all depend on it
- **Meaning of zero-length edges**: a zero length can be a sampled ancestor, an encoded polytomy, or a real zero length, and each needs a different display
- **Times of hybrid occurrences**: when the occurrences of a hybrid node end at different times, no rule says which time is correct
- **Clades in networks**: collapse, search scopes, and statistics assume clades, which networks do not define
- **Colour for continuous and skewed values**: evenly spaced categorical hues do not work for hundreds of values or skewed distributions

## Not covered by IcyTree

- **Two trees side by side**: no tanglegram, no links between matching leaves, and no shared colour between two trees
- **Continuous colour scale**: no gradient legend
- **Label thinning**: no hiding of overlapping labels at low zoom
- **Selection**: no persistent selection of nodes or clades beyond search highlighting
- **Circular or radial layouts**: only rectangular layouts exist
- **Collapsed clade labels**: a triangle shows no name or leaf count
- **Export resolution**: raster export has no scale factor, and the SVG export cannot cover the whole tree while the view is zoomed in
