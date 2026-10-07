# baltic feature survey: tree rendering, semantics, and edge cases

This report describes baltic (Backronymed Adaptable Lightweight Tree Import Code), a Python library that parses phylogenetic trees into a linked-list model and draws them with matplotlib. Influenza and other reassortment studies use it for segment tanglegrams and reassortment networks. The report is an idea inventory for tree, tanglegram, network, and ARG display. It records what baltic does, how it does it, which scientific conventions it assumes, which inputs break it, and which problems remain open. It does not compare baltic with `packages/web`.

baltic is licensed GPL-3.0. Its behavior and design may be studied, but copying its code needs approval (see the project rules).

- **Source**: [evogytis/baltic](https://github.com/evogytis/baltic) at commit `e357bd5` (2026-09-22), version 1.0 "Cedar", 335 commits since 2016-04-22. Source links point to this commit
- **Other sources**:
  - all 32 issues and 17 pull requests, with the comments of the issues that concern parsing, plotting, and export
  - the retired example notebooks (`austechia`, `curonia`, `galindia`) at commit [`0b8a55f`](https://github.com/evogytis/baltic/commit/0b8a55fcd1ef04e182858167fe56878e5a07dbb7), removed in [`8184aa9`](https://github.com/evogytis/baltic/commit/8184aa96265a8d225c5329b06ff9fbd0d4dbf5c7)
  - the API reference on [baltic.readthedocs.io](https://baltic.readthedocs.io/), which is generated from the docstrings, and the [example gallery](https://phylo-baltic.github.io/baltic-gallery/) with its "As seen in" list of 181 citing works
  - public repositories of studies that draw tanglegrams with baltic (see "Recipes from the retired notebooks and from public repositories")
  - baltic has no paper of its own. The first reassortment study that used it is the influenza B study of [Dudas et al. 2015](https://doi.org/10.1093/molbev/msu287)
- **Evidence labels**:
  - **Observed**: seen in a trial run of the surveyed commit in a throwaway `python:3.12-slim` container (matplotlib 3.10, Python 3.12)
  - **Derived**: read from the code without running it
  - **Docstring conflicts**: where a docstring and the code disagree, the code wins and the difference is noted
- **No application**: baltic has no application and no user interface. Every figure is a script that calls baltic functions on a matplotlib `Axes`
- **Shape of the code**: one Python package of about 14,700 lines, most of them docstrings, in these modules:
  - [`baltic/baltic.py`](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/baltic.py): the regex-driven string parser `make_tree` and the Auspice JSON builder `make_tree_JSON`
  - [`baltic/tree.py`](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py): the `Tree` class with traversal, layout, tree operations, and the drawing methods
  - [`baltic/io.py`](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/io.py): file loaders for Newick, NEXUS, and Auspice JSON
  - [`baltic/bt_utils.py`](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py): dates, time grids, scale bar, colour helpers, untangling, root-to-tip regression
  - [`baltic/curonia.py`](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py): composite figures: tanglegrams, tangled chains, reassortment networks, tree plus matrix, Muller plots, maps, SNP alignments
  - [`baltic/samogitia.py`](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/samogitia.py): parallel processing of BEAST posterior tree sets
- **API renames in version 1.0**: version 1.0 replaces the camel-case API of version 0.3.0 with snake-case names, without compatibility aliases [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/__init__.py#L1-L6)]:
  - `loadNewick`, `loadNexus`, `loadJSON` became `io.load_newick`, `io.load_nexus`, `io.load_JSON`
  - `plotTree` and `plotCircularTree` became `plot_tree(treeType='rectangular' | 'circular' | 'unrooted')`
  - `plotPoints` and `plotCircularPoints` became `plot_points`
  - `addText`, `addTextCircular`, and `addTextUnrooted` became `plot_text`
  - `drawTree` became the private `_assign_tree_coordinates`, and `drawUnrooted` became `_assign_unrooted_tree_coordinates`
  - `x_attr`, `y_attr`, `colour`, `width`, `target`, and `connection_type` became `xCoordinateFxn`, `yCoordinateFxn`, `colour`/`colourFxn`, `width`/`widthFxn`, `targetFxn`, and `connectionType`
  - `sortBranches`, `commonAncestor`, `collapseSubtree`, `collapseBranches`, `reduceTree`, `toString`, `setAbsoluteTime`, `singleType` became `sort_branches`, `find_MRCA`, `collapse_subtree_to_clade`, `collapse_branches`, `reduce_tree`, `to_string`, `set_absolute_time`, `make_single_type`
  - `fixHeights` and `ladderize` do not exist in either version. `sort_branches` is the ladderize operation

## Summary

### Scope

- **Toolkit of drawing functions**: each figure is a script, and every visual choice is a Python callback (`colourFxn`, `widthFxn`, `targetFxn`, `xCoordinateFxn`)
  - The maintainers describe the design as "basic and generally applicable" building blocks in the style of matplotlib [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/docs/source/index.rst?plain=1#L31)]

### Tanglegrams

See "Two trees and tanglegrams".

- **Two built-in layouts**: both were added in 2025 and 2026. Before that every study wrote its own tanglegram loop
  - `plot_tanglegram` draws two trees face to face, the second one mirrored
  - `plot_tangled_chain` draws any number of segment trees end to end, all facing right
- **Connectors**: four-point polylines with horizontal "shoulders" and one diagonal
  - They are coloured by the vertical rank of the tip in the first tree with a desaturated Spectral colour map
  - Curved connectors appear only in user code, such as the Bézier curves of the retired shutter plot
- **Untangling is weak**: it leaves a perfect mirror image and the TreeKnit two-tree example fully tangled (observed)
  - `untangle` never rotates a node that has a leaf child (default `min_shared=2`)
  - It compares absolute mean positions
  - Its permutation step is wrong for polytomies with three or more children (observed)

### Networks

- **Hybrid nodes**: extended Newick hybrid nodes (`#H1`) are parsed into a `Reticulation` leaf-like object that points at the hybrid node
- **Drawing**: `plot_reticulations` draws each reticulation as a dashed straight line with a triangle marker
  - It can add a segment-presence matrix for CoalRe reassortment networks
- **TreeKnit labels fail**: hybrid labels such as TreeKnit's `ARGNode_10#H1` stop the parser with an assertion (observed)

### Scientific conventions

See "Scientific semantics".

- **Missing lengths become 0**: a partly missing set of branch lengths is silently read as zero lengths
  - A tree without any lengths gets invented ultrametric heights of height 1
- **Polytomies and zero-length branches are kept as they are**: nothing is collapsed
  - A parent sits at the mean row of all its children
- **Single-child nodes are kept**: they are valid "multitype" nodes
  - `make_single_type` removes them on request

### Parser fragility

- **Inputs that fail or load wrongly** (see "Defects"):
  - **Syntax**: whitespace, plain Newick comments, and doubled quotes
  - **Names**: `#` in a name and `|` in an internal label
  - **Files**: multi-line Newick, lower-case NEXUS `translate`, and NEXUS tree names without digits

## Tree model

- **Linked objects in a flat list**: a `Tree` holds `Objects`, a flat list of branch objects in parse order. Each object is one node plus the branch above it. There are four classes that share the base class `BranchLike` [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/branchLike.py#L18-L117)]:
  - **`Node`**: an internal node with `children`, the set of descendant tip names `leaves`, and `childHeight`, the height of its highest descendant
  - **`Leaf`**: a tip with a `name`
  - **`Clade`**: a collapsed subtree that behaves like a tip, with a `width` in rows, the hidden `subtree`, and `lastHeight`, the height of its farthest tip [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/clade.py#L55-L106)]
  - **`Reticulation`**: the childless occurrence of a hybrid node, with a `target` (the hybrid node) and a default `width` of 0.5 [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/reticulation.py#L46-L98)]
- **Attributes per object** [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/branchLike.py#L106-L117)]:
  - `length` (default 0.0) and `height` (distance from the root)
  - `absoluteTime` and `absoluteTimeRange`
  - `traits` (a dictionary of annotations) and `index` (the character position in the tree string)
  - the plot coordinates `x` and `y`
- **Hidden super-root**: every tree has an extra parent node with index `Root` above the real root [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L73-L79)]
  - Code checks `k.parent.parent` to detect the real root
  - Several bugs in the history came from this extra node ([#29](https://github.com/evogytis/baltic/issues/29), [`bc54585`](https://github.com/evogytis/baltic/commit/bc54585))
- **Tree type**: each tree is either `'divergence'` or `'time'`. The type selects the default x coordinate: `height` for divergence trees and `absoluteTime` for time trees [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L85-L89)] [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L4613-L4614)]
- **Traversal**: `traverse_tree` is a recursive pre-order walk with an `includeCondition` (what to collect) and a `traverseCondition` (which children to enter) [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L803-L878)]
  - A walk from the root without conditions resets and recomputes `height`, `leaves`, `childHeight`, and `treeHeight`
- **Recursion limit**: importing baltic raises the Python recursion limit to 9001 for the recursive parser and traversal [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/baltic.py#L23)]

## Input

### Tree string parser

`make_tree` scans the string with a moving index and a chain of regular expressions. If no expression advances the index, an assertion stops the parse with "Tree string unparseable" and the next 5000 characters [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/baltic.py#L86-L232)].

#### Syntax and names

- **Format checks**: the string must end with `;` and have balanced parentheses [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/baltic.py#L81-L82)]
- **Tip names**: either integers (BEAST translate keys) or any run of characters other than `(`, `)`, `:`, `[`, `'`, `"`, `#`, and `,` [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/baltic.py#L73-L110)]
  - **Quotes**: quotes are stripped, but a doubled quote inside a quoted name is not unescaped
  - **Unicode** (observed): `Zürich` and `Łódź` load correctly
- **Internal labels**: only letters, digits, `_`, `-`, `.`, and `/`, followed by `:`, `;`, or `[` [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/baltic.py#L216-L222)]
  - **Storage**: the label goes into `traits['label']` as text, so support values stay strings
  - **History**: `/` was added for IQ-TREE labels such as `Node7/50` ([#43](https://github.com/evogytis/baltic/issues/43), [#44](https://github.com/evogytis/baltic/issues/44))
- **Multitype singletons**: a BEAST structured-coalescent integer after `)` (as in `)12[`) is skipped [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/baltic.py#L112-L115)]

#### Branch lengths

- **Syntax**: the expression `[0-9.\-Ee]+` after an optional colon [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/baltic.py#L224-L228)]
  - **Negative lengths** are accepted with no warning (observed: `B:-0.5` gives height 0.5 under a parent at height 1)
- **All lengths zero means "cladogram"**: if the sum of all lengths is 0, baltic logs a warning and invents heights [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/baltic.py#L234-L257)]
  - **Rule**: every tip is at height 1.0, and each parent is `1/maxLvl` below its lowest child, where `maxLvl` is the deepest path length
  - **Observed** with `((A,B),(C,D));`: the internal nodes get height 0.5 and the tips 1.0
- **Partly missing lengths become 0** (observed): in `((A:1,B):1,C:2);` B gets length 0.0 and sits at its parent's height, with no warning [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/branchLike.py#L62-L70)]
  - The cause is the default length of 0.0

#### Annotations

Annotations `[&key=value,...]` are matched by a character class and then split by five more expressions [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/baltic.py#L157-L213)].

- **Numbers**: a value becomes a float only if it is digits after removing one `E` or `e`, one `-`, and one `.`
  - Observed: `y=2E-4` becomes 0.0002, but `x=-1.5e-3` stays the string `'-1.5e-3'`
- **Strings**: quotes are removed, and `true` and `false` become Python booleans
  - A value such as `A+B` (BEAST equiprobable states) keeps only `A`
- **Sets and ranges** `{...}`: keys that contain `set` become lists of strings (or floats for `set.prob`). Other keys become lists of floats
  - Observed: `segments={0,1}` becomes `[0.0, 1.0]`
- **Robust counting histories** (`all_N`, `all_S`): lists of `[site, time, from, to]`
  - Most entries were dropped silently until the 2026 fix of [#47](https://github.com/evogytis/baltic/issues/47)
- **FigTree keys** `!color=...`: recognized and ignored with an info message (observed)

### Hybrid nodes (extended Newick)

- **Leaf occurrence becomes a `Reticulation`**: `,#H1` or `(#H1` creates a `Reticulation` object named `#H1` as a child of the current node [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/baltic.py#L117-L135)]
- **Occurrence with children becomes the hybrid node**: `)#H1` sets `traits['label'] = '#H1'` on the node that has the children [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/baltic.py#L137-L155)]
- **Linking in either order**: whichever occurrence comes second looks up the first by name and sets `reticulation.target = node` and `node.contribution = reticulation`
  - Observed with both orders
  - A second `Reticulation` with the same name raises "Reticulate branch not unique"
- **IDs**: only `#` followed by letters and digits
  - The type letters of Cardona et al. (`R`, `H`, `LGT`) are part of the ID and are not interpreted
- **Labelled hybrid occurrences fail** (observed): a label before `#`, as in `ARGNode_10#H1`, cannot be matched by any expression. See "Interaction with TreeKnit output"
- **The hybrid node keeps one parent**: the node with children has its tree parent as `parent`, and the second parent is reachable only through `contribution.parent`
  - Tree operations such as `traverse_tree`, `leaves`, and heights ignore the reticulation edge, so the network is a tree plus pointers

### File loaders

- **`load_newick`** [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/io.py#L30-L123)]:
  - **One line per tree**: every line that contains `(` is parsed from its first `(`, and each new tree replaces the previous one
    - Observed: a two-tree file loads the second tree with no message
    - A tree split across lines fails with "must end in semicolon" (observed)
  - **Defaults**: `sortBranches=True`, so the tree is ladderized on load. `absoluteTime=False`
- **`load_nexus`** [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/io.py#L125-L284)]:
  - **Tree line**: found with `treestringRegex`, default `tree [A-Za-z_]+([0-9]+)`, which fits BEAST names such as `tree STATE_0`
    - IQ-TREE needs `treestringRegex='tree ([0-9]+)'` ([#44](https://github.com/evogytis/baltic/issues/44))
    - Observed: `tree mcc = ...` fails with "Failed to find tree string using regular expression"
  - **Translate table**: recognized only by the capitalized word `Translate`, and names must match `[A-Za-z\-_/.'0-9 |?]+`
    - Observed: a lower-case `translate` block is ignored and the tips keep the names `1`, `2`, `3`
  - **Dates by default**: `absoluteTime=True` is the default, and the tip-date regex `\|([0-9\-]+)$` must match at least one tip name
    - Observed: a NEXUS file without dates in the names fails with an assertion unless `absoluteTime=False` is passed
  - **Travel-aware phylogeography**: tips named `*_ancestor_taxon` are removed with `reduce_tree`, which leaves single-child nodes ([`c063036`](https://github.com/evogytis/baltic/commit/c063036))
- **`load_JSON`** (Auspice v2) [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/io.py#L286-L455)]:
  - **Sources**: a path, a parsed object, or any string that contains `nextstrain.org`, which is fetched over the network with `requests`
  - **Translation**: a dictionary from baltic attributes to JSON keys, default `name` to `name`, `absoluteTime` to `num_date`, and `height` to `div`
    - For divergence trees without `div`, the height is summed from `length` attributes
    - Branch lengths are differences of the translated heights or times
  - **Traits**: each `node_attrs` entry `{value, confidence}` becomes `traits[key]` and `traits[key + '_confidence']`
  - **Colour scales**: only categorical colourings with a `scale` are kept, in the tree attribute `cmap`
  - **Unnamed internal nodes** get `NODE_0000001` style names in pre-order, as Auspice does [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/baltic.py#L339-L347)]
  - **Returns a tuple** `(tree, meta)`, which surprised users ([#18](https://github.com/evogytis/baltic/issues/18))

### Dates and absolute time

- **Tip dates from names**: `process_tip_dates` applies `tipRegex` and `dateFmt` to every tip name [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/io.py#L458-L545)] [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L156-L195)]
  - **Partial dates**: with `variableDate=True`, a date with only a year or a year and month becomes the midpoint of its range, and the range is stored in `absoluteTimeRange`
  - **Only exact dates anchor the tree**: the most recent date among the exact dates becomes the anchor
- **Absolute time formula**: `absoluteTime = mostRecentSamplingDate - treeHeight + height` [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L497-L503)]
  - **Divergence trees**: the loaders set the time on tips only (`justLeaves=True`)
  - **Anchor is the farthest tip from the root**: the formula assumes that the tip with the largest height is also the most recent one. For a time tree this holds. In the influenza notebooks the most recent tip was found separately from the tip names
  - **`mostRecent`**: the largest non-zero `absoluteTime`. The truthiness test skips a time of exactly 0
- **Decimal years**: `calendar_to_decimal_date` divides the elapsed seconds by the seconds of the year, so leap years are exact [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L148-L151)]

## Layouts

### Coordinate assignment

`_assign_tree_coordinates` (formerly `drawTree`) sets `x` and `y` of every object [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L1769-L1921)].

#### Rows

- **Rows from pre-order**: the leaf-like objects (tips, clades, reticulations) are collected in pre-order [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L1801-L1833)]
  - **Row width**: 1 for a `Leaf` and `width + 1` for any other leaf-like object
  - **Examples**: a `Reticulation` (width 0.5) takes 1.5 rows, and a collapsed clade of `n` tips takes `n + 1` rows by default
- **Top-to-bottom order**: `y = sum(widths from this object to the end) - width/2`, so the first object in pre-order is at the top and the last tip has `y = 0.5`
  - Observed: `ySpan` = 9 for 9 tips
- **`ySpan`**: `max(y) - min(y) + 2 * min(y)`. With rows starting at 0.5 it equals the number of rows [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L1909-L1912)]
- **Unique names required**: two leaf-like objects with the same name stop the layout with "Non-unique names present in tree"
- **Padding around clades** (`padNodes`): a dictionary from node to extra rows [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L1835-L1855)]
  - The rows of tips below the node and of the node's own tips are shifted, which opens a gap above and below the clade
  - The gallery example "pad nodes" shows the effect
- **Custom row widths were removed**: version 0.3.0 accepted a `width_function` per tip. Version 1.0 fixes the widths as above

#### Node positions

- **Parent at the mean of all children**: an internal node is placed at the arithmetic mean of its children's rows, which pulls a polytomy parent toward the side with more children [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L1873-L1881)]
  - Observed: a root with children at rows 2.75, 6.75, and 8.5 sits at row 6.0
  - **`yRange`**: each node also stores the lowest and highest row of its descendants
- **x from height**: `x = height`. The real root gets `x = min(child.x - child.length)`, which is 0 [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L1914-L1921)]

### Rectangular layout

- **Connection types** (`connectionType`) [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L4354-L4374)]:
  - **`baltic`** (default): each branch is a horizontal line from the parent's x to its own x. Each node adds one vertical line from its first child's row to its last child's row
  - **`elbow`**: each branch is its own right-angled path from the parent point to the child point
  - **`direct`**: a straight line from the parent point to the child point, which gives a "triangular" tree
- **Orientation**: `horizontal` (root at the left) or `vertical` (x and y swapped)
  - Root-at-right needs a custom `xCoordinateFxn` such as `treeHeight - k.height`, which is how the second tree of a tanglegram is mirrored
- **Drawing**: all branches go into one matplotlib `LineCollection` with per-segment colours and widths, `capstyle='projecting'`, and `zorder=1` [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L4384-L4389)]
- **Partial trees**: `targetFxn` selects which branches to draw
  - Calling `plot_tree` twice with complementary targets draws a background tree and a highlighted subtree, as in the retired clade-frequency example
- **Axes stay visible**: `plot_tree` calls `ax.autoscale()` and leaves ticks and spines
  - `bt_utils.clean_axes` removes them (observed: the default figures show both axes)

### Circular layout

- **Projection**: the row becomes an angle and the normalized height becomes a radius [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L3304-L3343)]:
  - **`circStart`** (default 0.0) and **`circFrac`** (default 1.0): the start and the fraction of a full turn used by the tree. A value below 1 leaves a gap, which is how the retired "shutter plot" placed eight segment trees around one circle
  - **`inwardSpace`** (default 0.0): extra radius at the centre. A negative value is shifted by `-treeHeight`, which turns the tree outward
  - **`normaliseHeight`**: a function from x to radius. The default maps the smallest and largest x of all objects, including collapsed-clade members, to 0 and 1. Several trees with a shared `normaliseHeight` share one radial scale
- **Arcs**: the `baltic` connection draws each node's vertical line as an arc of `precision` points (default 15). `elbow` draws an arc at the parent [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L4504-L4530)]
- **Equal aspect**: `ax.set_aspect(1)` is set
- **Readable labels**: text rotates with its angle and flips by 180 degrees between 90 and 270 degrees, with the alignment switched to `right` [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L3694-L3701)]

### Unrooted layout

- **Equal-angle algorithm**: each node gets a wedge of the full circle in proportion to its tip count (plus padding), and each branch points to the middle of its wedge with its own length [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L1924-L2016)]
- **Only `direct` connections**: other connection types are ignored with a warning. `circStart` rotates the whole tree
- **Labels along the branch**: text is rotated to the branch angle and flipped to stay upright [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L3598-L3600)]
- **Clades as fans**: a collapsed clade is a trapezoid base plus an arc of `precision` points, `equal` or `skewed`

### Exploded tree

- **Split by a trait**: `explode_tree` cuts the tree at every branch whose trait differs from its parent's (or where a custom function is true) and returns one subtree per piece, by default with the stem branch [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L3006-L3070)]
- **Stacked drawing**: `plot_exploded_tree` sorts the pieces by root time (oldest first) and stacks them vertically with `verticalSpace=2` rows between them, with a point at each piece's origin [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L4902-L4938)]
- **Pieces without tips are dropped**: a piece whose own traversal reaches no tip is logged as an error and skipped
  - Observed with an MCC trait on the TreeKnit example tree: the non-monophyletic MCC `{A, B, C, D}` became three pieces, `{A, B}`, `{C}`, and `{D}`, and the root piece was dropped

## Child order

- **`sort_branches`** (ladderize): sorts the children of every node and redraws [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L1699-L1766)]:
  - **Default key**: tips before internal nodes, then internal nodes by tip count, then by branch length
    - Observed with `descending=True` (default): the order is `A, B, C, D, E, F, G` for `((A,(B,C)),(D,(E,(F,G))))`, so tips and small clades are at the top and the largest clade at the bottom
    - `descending=False` gives the exact reverse
  - **Custom key** (`sortFxn`) or a custom reorder function (`operationFxn`) that receives the whole child list
  - **Bug history**: `descending` had no effect until 2023 ([#33](https://github.com/evogytis/baltic/issues/33))
- **Sorting on load and on draw**: `load_newick` and `load_nexus` sort by default, and `plot_tree` sorts again because `autoSort=True` is its default [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L4652-L4653)]
  - Observed: a manual order `C, A, B, D` became `D, C, A, B` after a plain `plot_tree` call
  - Any untangled order is lost unless every call passes `autoSort=False`
- **Keep tip positions** (history): a 2022 option `sortByHeight=False` sorted only internal nodes and kept tips at their input positions ([#28](https://github.com/evogytis/baltic/pull/28)). It is gone in version 1.0

## Branches, nodes, and marks

### Branch and point style

- **Callbacks or constants**: each style parameter exists twice, as a constant (`colour`, `width`, `pointSize`) and as a function of the branch object (`colourFxn`, `widthFxn`, `pointSizeFxn`) [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L4621-L4639)]
  - Giving both raises `ValueError`
  - Defaults: black, width 2, point size 40
- **Missing colour value**: if `colourFxn` raises `KeyError` (a missing trait), the branch is drawn light grey `(0.7, 0.7, 0.7)` [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L4346-L4349)]
- **Points with outlines**: `plot_points` draws two scatters, the fill on top and an outline behind it [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L3917-L3946)] [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L4052-L4071)]
  - **Outline**: area `((2 + sqrt(s/pi))^2) * pi`, which is a 2-point ring
  - **Return value**: the axes and a dictionary from object to plotted coordinates, which later code can use to attach connectors
- **Text outlines**: `get_path_effects` builds the white-halo stroke used to keep labels readable over branches [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L2877-L2915)]

### Collapsed clades

- **Collapsed clades** [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L4074-L4153)]:
  - **Shape**: a polygon from a narrow base (`cladeBaseWidth` = 0.001 of `ySpan`, so the join is not a sharp point) to the clade's end, `width` rows high. The default colour is light grey with a black edge
  - **`equal` or `skewed`**: `skewed` slants the far edge from the earliest tip to the latest tip, which shows the spread of sampling times
  - **Circular shapes**: `triangle` or `rectangle` (the latter starts with an arc)
- **Gradient clades**: `plot_gradient_clade_tree` replaces designated clades with a fan bounded by two quartic Bézier curves [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2532-L2606)]
  - **Fill**: an alpha gradient from `minAlpha` 0.0 to `maxAlpha` 0.4
  - **Tip branches**: the tip branches inside fade in over the last `tipLen` fraction (default 0.5) of their length

### Node annotations

- **Node height intervals**: `plot_height_95hpds` draws a grey rectangle of 40% opacity for each `height_95%_HPD` pair, converted to calendar time with the last tip date [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2661-L2680)]
- **Node trait summaries**: stacked bars, treemaps, and pie charts of discrete-state probabilities (`trait.set` and `trait.set.prob`) [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L1070-L1365)]
  - States below `otherThres` are merged into "other"

## Labels

- **`plot_text`**: one `ax.text` per target object, tips by default [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L3424-L3465)]
  - **Content**: `textContentFxn` (default `name`)
  - **Offsets**: `xSpace` (default 0.005 of the tree height) and `ySpace`
  - **Order**: `zorder=4`
- **Aligned tip labels**: `plot_aligned_tip_labels` places all tip labels at `treeHeight * (1 + xSpace)` and connects each tip to its label with a dotted line of width 1 in the label's colour [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L3780-L3832)]
  - It works in rectangular and circular layouts and warns in the unrooted layout
- **No overlap handling**: labels are never thinned or moved
  - Observed: in the tanglegram figure the tip labels of the left tree sit on top of the connectors, and a long label runs off the figure in the circular layout
- **Clade labels**: a collapsed clade has a `name`, but `plot_text` targets tips by default
  - The name must be drawn with `targetFxn=lambda k: k.is_leaflike()`
  - Observed: the clade name then sits at the clade's base, over the polygon

## Colour

- **No built-in colour mapping**: colour comes only from the user's `colourFxn`
  - There is no legend, no automatic palette, and no categorical or continuous scale
- **Tip-position colouring for tanglegrams**: both tanglegram functions colour each tip by its rank in the first tree, `Spectral` desaturated to 0.6 from bottom to top [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2336-L2343)] [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2054-L2055)]
  - Observed: five connectors got five distinct colours from purple to orange
  - The same idea colours tree-to-map connectors by row in `connect_tree_to_map`
- **Colour by state history**: the retired reassortment example counted, for each branch, the reassortment events on its path to the root and coloured branches by that count with a five-colour cycle [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L689-L745)]
  - The gallery's WuMV-6 network does the same with `_partition_tree`, which gives each region between reassortment events a random 10-character label
- **Helpers**:
  - **`make_cmap`**: a colour map from a list of mixed colour formats and optional positions [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L2482-L2560)]
  - **`desaturate`** and **`desaturate_cmap`**: scale the HSV saturation (default factor 0.65) [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L2398-L2595)]

## Axes and time

- **Time trees use calendar x directly**: with `treeType='time'` all plots use `absoluteTime` as x, so matplotlib's own axis shows decimal years
- **Time grid**: `generate_calendar_timeline` returns dates at `yearly`, `monthly`, `weekly`, or `n`-day spacing, rounded to the start of a month or year [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L577-L698)] [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L462-L476)]
  - **Deep time**: since 2026 it also accepts deep-time steps (`decadal`, `centennial`, `millennial`, or a tuple such as `(500, 'kyr')`) for years outside 1 to 9999
- **Alternating bands**: `plot_time_grid` shades every second interval, and `format_time_grid` puts tick labels at the start or the middle of each interval [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L1578-L1762)]
- **Scale bar**: `plot_scale_bar` picks a length from the 1-2-5 series near 5% of the tree height [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L814-L892)]
  - **Mutations**: with an alignment length (`alnL`) it picks a round number of mutations and labels the bar in mutations instead of substitutions per site
  - **Styles**: `simple` or `fancy` (with end ticks), horizontal or vertical

## Two trees and tanglegrams

### Face-to-face tanglegram

`curonia.plot_tanglegram(ax, tree1, tree2)` was added in August 2026 ([`adef36c`](https://github.com/evogytis/baltic/commit/adef36c0c9a5c0fa8d09507c3867676707d1fd4f)) [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2263-L2400)].

- **Geometry**: tree 1 grows to the right from x = 0. Tree 2 is mirrored: `x = tree1.treeHeight + treeSpace + (tree2.treeHeight - k.x)`, so its root is at the far right [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2357-L2361)]
  - **Gap**: `treeSpace` defaults to 30% of the taller tree's height
  - **Vertical normalization** (`normaliseY=True`): each tree's rows are divided by its `ySpan`, so trees with different tip counts span the same height
  - **Tips are not aligned**: x is the height, so in a non-ultrametric tree the tips end at different x and the connectors start at the tips. Observed: tip X of the TreeKnit `ha` tree ends at x = 3 while the other tips end at x = 2
- **Connectors**: one four-point polyline per shared tip [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2379-L2398)]
  - **Path**: from the tip horizontally to `tree1.treeHeight + treeSpace * padding`, diagonally to `tree1.treeHeight + treeSpace * (1 - padding)`, and horizontally to the matching tip
  - **`padding`**: defaults to 0.1 and must lie in [0, 0.5]
  - **Drawing**: all connectors are one `LineCollection` at `zorder=0`, under the trees
- **Colours**: a user `colourDict` from tip name to colour, or the default rank colouring of tree 1
  - Tips missing from the dictionary are light grey
- **Unshared tips**: tips present in only one tree are listed in a warning and get no connector [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2370-L2377)]
- **Order is respected**: the function assigns coordinates without sorting and draws with `autoSort=False`, so it keeps an order prepared by `untangle`
- **No untangling inside**: the caller must untangle first. The docstring example calls only `sort_branches`

### Tangled chain

`curonia.plot_tangled_chain(ax, treeList)` draws a sequence of segment trees left to right, all facing right ([`43e484a`](https://github.com/evogytis/baltic/commit/43e484a922dd74ba917a70386d604b32cd106f05), 2025) [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2105-L2260)].

- **Spacing**: each tree is shifted by the cumulative tree heights plus `treeSpace`, default 20% of the first tree's height [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2159-L2162)]
- **Connectors run through the next tree**: a connector goes from a tip of tree `i` to just before the root of tree `i + 1`, then horizontally under tree `i + 1` to the matching tip at its right end
  - Observed: the lines cross the branches of the next tree and are visible only because they are drawn below it
- **Colour**: the rank colouring of the first tree, kept along the whole chain, so a tip keeps its colour from segment to segment
- **Normalization**: rows divided by each tree's `ySpan` ([`90263a4`](https://github.com/evogytis/baltic/commit/90263a402164692e0058214eb04ffeeb54826455))
- **Gallery use**:
  - **WuMV-6**: the example calls `untangle_trees(treeList, iterations=3, bidirectional=False)`, then `plot_tangled_chain(..., lw=8, alpha=0.8, treeSpace=0.01, pointKwargs={'colour': 'w'})`, and labels only the last tree's tips ([gallery](https://phylo-baltic.github.io/baltic-gallery/examples/wumv-6-tangled-chain.html))
  - **Avian influenza**: the example draws eight Nextstrain segment trees without untangling ([script](https://github.com/phylo-baltic/baltic-examples/blob/1a91feefd0649d1e64e5a525fc36d8ff71d955c8/nextstrain/nextstrain-avian-flu-tanglegram.py#L17-L35))

### Untangling

#### Functions

- **`untangle(tree, reference, min_shared=2, maxPolytomy=9)`**: reorders the children of `tree` to follow the tip order of `reference`, in place [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L1853-L1916)]:
  - **Order of visits**: nodes from the highest height to the root (bottom-up)
  - **Score**: for each permutation of the children, the sum over child positions `i` of `|mean current row of child perm[i]'s tips - mean reference row of child i's tips|`. The best permutation is applied and the tree is redrawn after each change
  - **Skips**: a node with more than `maxPolytomy` children, and any node where a child shares fewer than `min_shared` tips with the reference [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L1872-L1884)]
    - With the default of 2, **every node that has a tip as a child is skipped**, which includes all cherries
  - **Absolute rows**: the score compares rows of the two trees directly, without normalizing for different tip counts
- **`untangle_trees(trees, iterations=10, maxPolytomy=8, bidirectional=True)`**: forward passes (tree `i` against tree `i - 1`) and backward passes, a fixed number of times, with no convergence test [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L1960-L1988)]
  - It does not pass `min_shared`, so cherries are never rotated
  - It logs the warning that tangle lines "can be perfectly parallel without much topological congruence"

#### Observed results

See "Defects" for the causes.

- **TreeKnit two-tree example**: `ha` = `((A,B),(C,(D,X)))` and `na` = `((A,(B,X)),(C,D))` keep the orders `A B C D X` and `A B X C D` after `untangle_trees`
  - Two connectors cross, while the rotations `A B X D C` in both trees give zero crossings
- **Mirror image**: an eight-tip tree and its exact mirror stay fully crossed after 10 bidirectional iterations
- **Three-child polytomy**: with reference order `A B C` and tree order `B C A`, `untangle` produces `C A B`

#### Version 0.3.0 algorithm

The algorithm was removed in 2025 [[src](https://github.com/evogytis/baltic/blob/5ddbc5fa0a8d3d857b64abfeaa5158cb4bbbb8c3/baltic/baltic.py#L1744-L1799)].

- **Order of visits**: nodes from the root down
- **Score**: it assigned the node's own sorted rows to the tips of each child permutation and scored the mean squared row difference against the previous tree
- **Cyclic chain**: it looped the chain cyclically (the last tree against the first)
- **Two inversions cancel** (derived): it picked the permutation with the highest cost, but it also assigned rows bottom-up while the layout fills rows top-down
  - For evenly spaced rows and the default squared cost these two inversions cancel

### Recipes from the retired notebooks and from public repositories

#### Original influenza B recipe

The recipe comes from the retired `austechia` notebook ([file](https://github.com/evogytis/baltic/blob/0b8a55fcd1ef04e182858167fe56878e5a07dbb7/docs/notebooks/austechia.ipynb)).

- **Preparation**: load eight segment MCC trees, collapse branches with posterior below 0.5, and reduce every tree to every fifth tip of the PB1 tree to speed up plotting
- **Untangling**: ten iterations. For each node of the next tree, from the tips to the root, sort its children by the mean difference between each tip's row in the previous tree and the rows the node occupies
  - The loop index wraps, so the first tree is also untangled against the last
- **Drawing**: tree 2 mirrored with `x = tree1.treeHeight + skip + tree2.treeHeight - k.height` and `skip = 0.35 * tree1.treeHeight`
  - **Connectors**: four-point lines with shoulders at 15% of the gap, coloured with `cividis` by the tip's row in tree 1
  - **Branches**: coloured by the Victoria or Yamagata lineage of another segment
- **This recipe is the template** of the built-in `plot_tanglegram`. Later studies copied it:
  - Espalier ([script](https://github.com/davidrasm/Espalier/blob/91b6835f7a842c5cc0958791ea72454eb492c222/Espalier/viz/PlotTanglegrams.py#L88-L122), [Rasmussen et al. 2023](https://doi.org/10.1093/sysbio/syad040))
  - Shigella plasmid trees ([notebook](https://github.com/nicfel/Plasmids-Material/blob/20b2ce1b8956238236963c00f63e90b8b9071057/Applications/Shigella/visualisation_code/ChromosomePlasmidTanglegrams.ipynb))
  - mosquito narnavirus segments ([notebook](https://github.com/czbiohub-sf/california-mosquito-study/blob/d4ee624e24aeb149b0b0c3806fde70fa86446d61/scripts/narna-tanglegram.ipynb))

#### Other retired notebook figures

- **Shutter plot**: eight segment trees placed around one circle, each in `circFrac = 1/8` of the turn with a 5% gap, pointing inward
  - One random tip is connected across all trees by quadratic Bézier curves through the centre
  - A second variant lays time along the circle instead of the radius
- **Segment presence on reticulations**: for a CoalRe network, a row of black and white squares to the right of each reticulation shows which segments travel along it, with the posterior as text
  - The function `plot_reticulations(plotSegMatrix=True)` now does this
- **Clonal frame per segment**: the same network drawn once per segment side by side, each copy showing only the branches that carry that segment
  - `markBranches` follows `contribution` when the reticulation carries the segment

#### Public repositories

- **Width equalization** (influenza H5 studies, [script](https://github.com/vjlab/episodic-h5/blob/787fee7d1f39e11fe9e5a819efe708ad5b547d2f/scripts/Tanglegram/tanglegram.py#L76-L90)): per-segment factors (7.5, 5, 1.5) stretch the shallow segment trees to similar widths
  - Connectors have three parts with alpha 0.25 to 0.3 and are coloured by the HA clade
- **Barycentre untangling** (HA and PB2 tanglegram, [script](https://github.com/AlCenZ/tanglegram_tree/blob/aac261d3127aa56a84b478322d987fabb5c1f97e/scripts/plot_ha_pb2_tanglegram.py#L468-L504)): sorts each node's children by the mean row of their tips in the partner tree, alternating between the two trees for a set number of iterations
  - Connectors are coloured by the joint state of both segments (both Victoria-like, HA only, PB2 only, neither) [[src](https://github.com/AlCenZ/tanglegram_tree/blob/aac261d3127aa56a84b478322d987fabb5c1f97e/scripts/plot_ha_pb2_tanglegram.py#L439-L460)]

### Tree plus segment matrix

- **`plot_tree_matrix`**: draws a tree on one axis and a cell matrix on a second axis that shares the rows [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2867-L2919)]
  - **Cells**: one column per key of `labelDict` (for example one column per segment) and one row per tip
  - **Colours**: cell colours come from `colourDict[column][label]`, and a missing label leaves the cell invisible with a warning
- **Influenza B use**: the HA tree with eight columns, each cell coloured Victoria or Yamagata, shows reassortment as colour changes along a row ([gallery](https://phylo-baltic.github.io/baltic-gallery/examples/influenza-b-virus-tree-matrix.html))

## Networks and reticulations

- **`plot_reticulations(ax, tree)`** draws each `Reticulation` that `excludeFxn` does not exclude [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2770-L2820)]:
  - **Line**: dashed, width 2, straight from the reticulation's own point `(absoluteTime, y)` to the hybrid node's point
    - The reticulation's point is at the time of the second parent's branch end, so in a dated network the line is vertical
    - Observed on the TreeKnit ARG: a vertical dashed line at 2019.5 from the bottom row up to the parent of X
  - **Arrow head**: a white triangle on a black triangle at the hybrid node, pointing up when the hybrid node is above the reticulation
  - **Segment matrix** (`plotSegMatrix=True`): reads CoalRe traits `seg0`, `seg1`, ... from the root and draws one square per segment at `root.absoluteTime + treeHeight * segMatrixDist` (default 1.1)
    - A square is black when the reticulation carries the segment
    - Reticulations are numbered from the top (`#1`, `#2`), and the segment columns are labelled above the first one
    - A dashed grey pointer line joins the reticulation to its matrix row
  - **Requires absolute time**: every coordinate is `absoluteTime`, so a divergence network needs `set_absolute_time` first
- **Each reticulation takes its own row**: a `Reticulation` is leaf-like, so it occupies 1.5 rows in the layout, at its pre-order position
  - Observed: in the TreeKnit ARG the reticulation took the last row, below D, and the parent of D drew a vertical stub down to that row
- **The hybrid node is a single-child node**: it is drawn as an ordinary node with one child
  - `plot_tree` draws the reticulation branch like any other branch unless `targetFxn` excludes it
- **Gallery network** (WuMV-6) ([gallery](https://phylo-baltic.github.io/baltic-gallery/examples/wumv-6-reassortment-network.html)):
  - `reduce_tree` keeps reticulations with posterior at least 0.1
  - Branches are coloured by the number of reassortments on their path
  - `plot_reticulations` adds the segment matrix
- **Reticulations break tree operations** (see "Defects"): `untangle`, `to_string`, and `to_auspice_json` fail or lose the reticulation

## Tree operations for display

### Subtrees and reduction

- **`subtree(startingNode, traverseCondition, stem=True)`**: deep-copies the branches below a node into a new tree [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L307-L396)]
  - **Stem**: with `stem=True` the parent of the starting node becomes a single-child root, so the subtree keeps its stem branch
  - **Pruning**: nodes left without children by the traverse condition are removed
  - **Empty result**: it returns `None` when the traversal reaches no tip ([#21](https://github.com/evogytis/baltic/issues/21))
- **`reduce_tree(tipsToKeep)`**: returns a copy with only the paths from the kept tips to the root. Single-child nodes stay [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L2650-L2703)]
  - Observed: reducing `(((A,B),C),D)` to A and D gives `(D,((A)))` with two single-child nodes
- **`make_single_type`**: merges single-child nodes into their child, adding the lengths [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L430-L454)]
- **`state_collapse_tree(tree, switchFxn)`**: keeps one tip per region between state changes (the last or the earliest) and returns a reduced copy [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L373-L455)]
  - The kept tip stores the region size and its members
  - The gallery shows it for Ebola and MERS-CoV host states

### Collapsing

- **`collapse_subtree_to_clade(node, name, widthFunction)`**: replaces a node and its descendants with a `Clade` in place, default width = tip count, then sorts the tree [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L2143-L2218)]
  - `restore_all_collapsed_subtrees` puts all clades back
- **`condense_tree(cutoffs, protectedTips)`**: collapses every outermost node with between 3 and 20% of all tips (defaults), except nodes that contain a protected tip [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L3122-L3137)]
- **`collapse_branches(collapseIfFxn)`**: returns a copy in which each node that satisfies the function (default `posterior <= 0.5`) is removed [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L2281-L2368)]
  - The children of a removed node move up, with the node's length added to theirs
  - Nodes are processed from the tips toward the root

### Rooting and other operations

- **Rerooting**: `reroot(branch, branchFrac=0.5)`, `midpoint_root`, and `root_by_regression` [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L1091-L1217)]
  - **`root_by_regression`**: a closed-form search over every branch position, with uncertain tip dates refined by projection
  - **Time trees**: all three refuse time trees
  - **Check**: `reroot` asserts that the total length is unchanged
- **`find_MRCA(*tips)`**: accepts objects or tip names and returns the last shared node of the root paths [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L2074-L2094)]
- **Other operations**: `rescale(factor)`, `rename_tips(map)`, `count_lineages_at_time(t)`, and `get_all_tip_TMRCAs` (a pairwise matrix of node times)

## Export

- **Figures**: whatever matplotlib can save (SVG, PDF, PNG). There is no baltic export code
  - Observed: with `svg.fonttype = 'none'` the SVG keeps labels as `<text>` elements
  - Observed: each `LineCollection` becomes one group of paths with per-path stroke colours
- **`to_string`**: Newick or a minimal NEXUS (`#NEXUS`, `Begin trees;`, `tree TREE1 = [&R] ...`, `End;`) [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L2441-L2540)]:
  - **Names** are always wrapped in `quoteCharacter` (default `'`) without escaping
  - **Lengths** are written with 15 decimals, the root included (observed: `:0.000000000000000;`)
  - **All traits** become `[&...]` comments
    - Internal labels become `label="n1"` comments instead of Newick labels
    - Integers read as floats are written as floats (`segments={1.0}`)
  - **Reticulations are lost**: see "Defects"
- **`to_auspice_json`**: an Auspice v2 dictionary with `div` or `num_date` per node and `NODE_0000001` names in pre-order [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L4998-L5054)]:
  - **Only some traits become colourings**: keys that end in these suffixes are kept:
    - `_95%_HPD` (continuous, with the HPD as confidence)
    - `.set.prob` (categorical, with the state probabilities as confidence)
    - `_median` and `posterior`
  - **Other traits are dropped**: every other trait is dropped from the colourings and from the nodes (observed with `mcc_ha_na`)
  - **Provenance**: `meta.data_provenance` names baltic and its installed version

## Configuration surface

- **No configuration files and no state**: everything is a function argument
  - The same tree object carries layout coordinates from the last call, so a later call with `recomputeCoordinates=False` reuses them
  - Tanglegram code relies on this to draw several trees with shifted coordinates
- **Parameters that do not apply** (for example `circFrac` in a rectangular plot) produce a `warnings.warn` message instead of an error
- **Logging**: all modules log through the `baltic` logger. Root-to-tip regression installs its own stdout handler

## Statistics and analysis tied to display

- **Tree and sequence context**:
  - **Root-to-tip regression**: `plot_root_to_tip` draws tip divergence against date with the regression line [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L1577-L1786)]
  - **Tree to map**: `connect_tree_to_map` joins tips to their coordinates on a cartopy map axis with `ConnectionPatch` lines, optionally through a "shoulder" x position [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L1956-L2102)]
  - **SNP alignment and genome features**: `plot_snp_alignment` draws variable sites next to the tree, and `plot_seq_features` draws GFF genes as arrows on non-overlapping tracks [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L3209-L3971)]
  - **Clade frequencies and Muller plots**: a port of the Nextstrain frequency estimator (`tree_frequencies`) and `plot_Muller`, which stacks nested clade frequencies [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L983-L1524)]
- **BEAST outputs**:
  - **TMRCA densities**: `plot_tmrca_posterior` draws a kernel density of a TMRCA column of a BEAST log as a full or half violin, optionally attached to a node [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L1367-L1575)]
  - **Skygrid**: `plot_skygrid` draws the median and HPD band of a BEAST skygrid log [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L1789-L1953)]
  - **Posterior tree sets**: `samogitia.process_posterior_trees` streams a BEAST `.trees` file after burn-in and runs worker functions (TMRCA, tree length, lineage trait tracing) in parallel processes [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/samogitia.py#L147-L271)]
- **Tree statistics**: `treeStats` prints a summary of the tree [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L587-L605)]
  - height and length
  - whether the tree is strictly bifurcating, has single-child nodes, or has annotations
  - the object counts

## Scientific semantics

### Branch length and time

- **Height means distance from the root**: x grows from the root (0) to the tips
  - "Tree height" is the largest root-to-tip distance
  - There is no "age before the youngest tip" coordinate. Time trees use calendar time instead
- **Missing lengths are zeros**: a missing length is 0.0, except when all lengths are 0, which triggers the invented ultrametric heights
  - A tree with one real length and many missing ones is therefore drawn with many zero-length branches and no warning (observed)
- **Divergence versus time**: the two tree types differ only in the default x and in which operations are allowed
  - Rerooting and regression need divergence trees
  - `get_all_tip_TMRCAs` needs a time tree

### Polytomies, zero-length branches, and single-child nodes

- **Polytomies are kept**: any number of children is accepted, and the parent sits at the mean row
  - `untangle` brute-forces polytomies up to 9 children (8 in `untangle_trees`) and silently skips larger ones
- **Zero-length branches are kept**: they are drawn with no length
  - A zero-length internal branch makes two vertical node lines coincide, which looks like a polytomy (observed)
  - A zero-length tip sits on its parent's vertical line
- **Single-child nodes are first-class**: they are drawn as plain bends, and `make_single_type` removes them on request. These inputs produce them:
  - structured-coalescent "multitype" trees and sampled ancestors
  - subtrees with a stem
  - reduced trees
- **No collapse of near-zero branches**: there is no threshold option

### Node and branch attributes

- **One trait dictionary per object**: the annotations of a node and of the branch above it share one dictionary, as in BEAST output
  - A colour function therefore colours the branch above a node by the node's state
  - A state change appears on the branch that ends in the new state
- **Support values stay text**: a Newick internal label is stored as the string `traits['label']`, so `95` is not a number until the user converts it
  - Rerooting keeps labels on their nodes, which misplaces branch support on the rerooted path

### Rooting

- **Every tree is rooted**: the root of the Newick string is drawn as the root. Unrooted inputs can be drawn with the unrooted layout, but the internal model keeps the root

## Defects

Each entry gives the input, the effect, and the evidence. Observed entries were reproduced at commit `e357bd5`.

### Parsing

#### Network input

- **TreeKnit hybrid labels** (observed): `(X:0.5)ARGNode_10#H1[...]` and `,ARGNode_10#H1[...]` stop the parser with "Tree string unparseable" [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/baltic.py#L117-L140)]
  - The tip expression stops at `#`
  - The hybrid expressions require `(`, `,`, or `)` directly before `#`

#### Names and syntax

- **`#` in a tip name** (observed): `(Sample#3:1,B:1);` fails with "Tree string unparseable"
- **Whitespace** (observed): `( A:1 , B:1 );` fails at the first space
- **Plain comments** (observed): `(A[note]:1,B:1);` fails, because only `[&...]` comments are recognized
- **Doubled quotes** (observed): `('O''Brien':1,'b c':1);` fails
- **Internal labels with other characters** (observed): `(A,B)n|1:1` fails, because internal labels allow only letters, digits, `_`, `-`, `.`, and `/`
- **Negative exponents in annotations** (observed): `x=-1.5e-3` stays a string, because the numeric test removes only one `-` [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/baltic.py#L177-L181)]

#### Files and multiple trees

- **Two trees on one line** (observed): `(A,B);(C,D);` returns the first tree with no message
- **Multi-tree and multi-line Newick files** (observed): `load_newick` keeps the last tree of a file with one tree per line and fails on a tree split across lines [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/io.py#L105-L110)]
- **NEXUS dialects** (observed) [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/io.py#L242-L260)]:
  - a lower-case `translate` block is ignored
  - a tree name without digits is not found
  - a file without tip dates fails under the default `absoluteTime=True`

### Untangling and tanglegrams

#### Untangling

- **Nodes with a tip child are never rotated** (observed): with `min_shared=2`, a tip child has one shared tip, so its node is skipped [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L1876-L1884)] [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L1972-L1986)]
  - `untangle_trees` cannot change this because it does not pass `min_shared`
  - On the TreeKnit example two connectors stay crossed although a crossing-free order exists
- **Ties for subtrees in a different region** (observed and derived): for two children the score difference between the two orders is zero whenever both reference means lie on the same side of both current means [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L1899-L1907)]
  - A subtree that sits in the opposite half of the other tree is therefore never rotated, because nodes are visited bottom-up before the root swap moves them
  - Observed: an exact mirror stays fully crossed after 10 bidirectional passes
- **Wrong permutation for three or more children** (observed and derived): the score pairs child `i`'s reference mean with child `perm[i]`'s current mean, which describes the inverse permutation, but the code applies `perm` itself [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L1889-L1910)]
  - For two children this is the same, but for a cyclic order of three it is not
  - Observed: reference `A B C` and tree `B C A` give `C A B`
- **Networks crash `untangle`** (observed): a `Reticulation` child has no `leaves`, which raises `AttributeError` [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L1872)]
- **Untangled order lost on redraw** (observed): `plot_tree` sorts by default (`autoSort=True`), so drawing an untangled tree with default arguments restores the ladderized order

#### Tangled chain

- **Tangled chain with tip points** (observed): `plot_tangled_chain(..., pointKwargs={...})` without a point colour raises `NameError: name 'colourMap' is not defined`, because the variable is called `colourDict` [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2188-L2190)]
- **Stale tree spacing in the chain** (derived): the connector loop uses `spaceUnit` from the last iteration of the coordinate loop, so a `treeSpaceFxn` that gives different gaps per tree misplaces the connector shoulders [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2194-L2233)]

### Drawing

#### Clades

- **Per-clade colours ignored** (observed): with `cladeColour` as a function, every collapsed clade gets the colour of the first one [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L4142-L4148)] [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L4269-L4276)]
  - **Cause**: the colour is written into a shared keyword dictionary on the first clade and never replaced
  - **Observed**: clades `AB` (red) and `EF` (blue) were both red
  - **Scope**: the circular and unrooted clade functions have the same pattern
- **Clade polygons drawn once per clade** (observed): `_plot_rectangular_tree` calls the clade function for every `Clade` it meets, and that function draws all clades each time [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L4381-L4382)]
  - Observed: 4 patches for 2 clades. With `n` clades there are `n * n` polygons

#### Layout and order

- **Unrooted wedge widths disagree** (derived): a node's own direction uses `d.width` for a collapsed clade, while its parent allocates `d.width + 1` for the same clade [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L1975)] [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L2001)]
  - Branches toward subtrees with clades or reticulations therefore point slightly off the middle of their wedge
- **Child order destroyed by `make_single_type`** (derived): the grandparent's children pass through a `set`, so their order becomes arbitrary before the final sort [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L449)]

#### Other drawing functions

- **Reticulation colour ignored** (observed): `plot_reticulations(colour='red')` draws black lines, because the line colour list always receives `'k'` [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2789-L2794)]
- **Tree matrix without colours** (observed): `plot_tree_matrix` with the default `colourDict=None` raises `TypeError` [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2893)]
- **Scale bar width** (observed): `lineKwargs={'lw': 0.5}` gives a width of 2.0, because the test uses `or` where it needs `and` [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L801)]

### Tree operations and export

- **Reticulations lost in `to_string`** (observed): a `Reticulation` is neither a node nor a leaf, so only its comment and length are written [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L2487-L2524)]
  - The output `([&segments={0.0}]:1.0,'C':2.0)` has a nameless tip
  - The hybrid label survives only as the comment `label="#H1"`
- **No network or clade export to Auspice JSON** (derived): `branch_to_json` raises `TypeError` for any object that is not a node or a leaf [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L1045-L1047)]
- **Divergence trees cannot be exported to Auspice JSON without a date** (observed): `to_auspice_json()` on a divergence tree fails with "mostRecentDate is of type NoneType, not a float" [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L5004-L5008)]
- **Custom traits dropped from Auspice export** (observed): only the four suffix families listed under "Export" survive [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L5019-L5035)]
- **`collapse_branches(designatedNodes=...)` always fails** (observed): the check asserts that no designated node differs from the root of the copied tree, which is never true [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L2297-L2299)]
  - The error "Root node was designated for deletion" therefore appears for any list
- **Docstring examples that do nothing or fail** [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/curonia.py#L2726-L2732)]:
  - the `untangle` example leaves the tree unchanged (observed)
  - the `plot_reticulations` example uses two childless `#R1` occurrences, which create two unlinked reticulations, and it is marked `doctest: +SKIP` (derived)

## Performance

Timings come from one run in the trial container on a shared machine. They show orders of magnitude only.

- **Parse**: a balanced tree with 2048 tips parses in 0.17 s and a 1000-tip caterpillar in 0.02 s (observed)
- **Layout is quadratic for deep trees** (observed and derived): the coordinate loop repeatedly scans all internal nodes until every node whose children are placed has a row [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/tree.py#L1862-L1907)]
  - Objects are in pre-order, so a caterpillar places one node per scan
  - Observed: sorting and layout take 0.35 s for a 1000-tip caterpillar and 0.04 s for a 2048-tip balanced tree
- **Drawing**: `plot_tree` takes 0.76 s for the 1000-tip caterpillar and 0.17 s for the balanced tree (observed)
  - The cause: every call sorts and lays out again before it builds one `LineCollection`
- **Untangling is slow**: each accepted permutation triggers a full layout, and each node tries up to `k!` permutations
  - Observed: `untangle` with `min_shared=1` on two 512-tip trees took 3.4 s for one pass
- **Copies**: `subtree`, `collapse_branches`, `reduce_tree`, and `state_collapse_tree` deep-copy the tree

## Interaction with TreeKnit output

The shapes below come from `extended_newick` in [packages/treeknit-io/src/arg.rs#L14-L64](../../packages/treeknit-io/src/arg.rs#L14-L64), `auspice_json` in [packages/treeknit-io/src/auspice.rs#L18-L60](../../packages/treeknit-io/src/auspice.rs#L18-L60), and the CLI test outputs in [packages/treeknit-cli/tests/data/outputs/two/expected/](../../packages/treeknit-cli/tests/data/outputs/two/expected/). TreeKnit writes its tree files as Newick text, also when the input extension is `.tree` or `.nex` ([packages/treeknit-io/src/newick.rs#L347-L392](../../packages/treeknit-io/src/newick.rs#L347-L392)).

- **Resolved trees** (observed): `ha_resolved.nwk` and `na_resolved.nwk` load with `load_newick` and keep the internal names `NODE_1`, `NODE_2`, ... as `traits['label']`
  - `plot_tanglegram` draws the pair, but `untangle_trees` leaves two crossings (see "Untangling")
- **ARG in extended Newick** (observed): `ARG/arg.nwk` fails to parse, because every hybrid occurrence carries a label before `#H` ([packages/treeknit-io/src/arg.rs#L59-L62](../../packages/treeknit-io/src/arg.rs#L59-L62))
  - **After removing the labels** before `#H1`, the ARG parses
  - **Annotations**: `[&segments={0,1}]` becomes the float list `[0.0, 1.0]` on each object, and the node names `ARGNode_n` become `traits['label']`. The hybrid node's own name is lost with the removed label
  - **Drawing**: `plot_reticulations` draws the reassortment as one dashed vertical line from a reserved bottom row to the parent of X. The segment matrix does not apply, because it expects CoalRe traits `seg0`, `seg1` on the root instead of a `segments` list per edge
  - **Root**: the ARG root has no length, which baltic reads as 0
- **Auspice JSON** (observed with a file of the same shape):
  - `load_JSON` reads the `div` heights and the `mcc_<a>_<b>` values as strings such as `'1'` in `traits`
  - The `ordinal` colourings carry no `scale`, so `tree.cmap` is empty and colours must come from a user function
  - Re-exporting with `to_auspice_json` drops the MCC attributes
- **MCCs.json and MCCs.dat**: baltic has no reader
  - A script maps each tip to its MCC index and passes it to `colourDict` of `plot_tanglegram` or to a `colourFxn`, as the influenza scripts do with lineage metadata
- **Zero-length branches**: TreeKnit's resolved trees and ARG can contain zero-length branches. baltic keeps them, so they do not break parsing or drawing

## Ideas from the issue tracker and history

### Open requests

- **Magic methods** ([#12](https://github.com/evogytis/baltic/issues/12)): `__repr__`, sorting, and reversing for the tree classes
- **Unit tests and continuous integration** ([#10](https://github.com/evogytis/baltic/issues/10), [#11](https://github.com/evogytis/baltic/issues/11)): a contributor asked for round-trip tests of all formats, including reticulations
  - The test suite still has four parser tests [[src](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/tests/testsuite.py#L5-L56)]
- **Type hints** ([#45](https://github.com/evogytis/baltic/pull/45), open pull request)

### Declined

- **Built-in Auspice export** ([#24](https://github.com/evogytis/baltic/issues/24)): first declined because transferring all traits and building `meta` needed too much customization
  - A notebook was offered instead
  - A minimal export was added later ([`2bf94a1`](https://github.com/evogytis/baltic/commit/2bf94a1e78f0b143d7639273a0570cb1fb6f6082))
- **Tilted (top-down) trees** ([#36](https://github.com/evogytis/baltic/issues/36)): not supported natively
  - The maintainer's workaround swaps the coordinate functions (`x = k.y`, `y = treeHeight - k.height`) with `direct` connections

### Features found only in the history

- **Custom tip row widths** (`drawTree(width_function=...)`, removed in version 1.0): each tip could take its own number of rows [[src](https://github.com/evogytis/baltic/blob/5ddbc5fa0a8d3d857b64abfeaa5158cb4bbbb8c3/baltic/baltic.py#L674-L705)]
- **Sort internal nodes only** (`sortBranches(sortByHeight=False)`, 2022): kept tips at their input positions ([#28](https://github.com/evogytis/baltic/pull/28))
- **Cyclic untangling with a custom cost function** (`untangle(trees, cost_function, iterations)`, removed in 2025): see "Untangling" [[src](https://github.com/evogytis/baltic/blob/5ddbc5fa0a8d3d857b64abfeaa5158cb4bbbb8c3/baltic/baltic.py#L1744-L1799)]
- **Example notebooks** (retired 2026-09-20 in [`8184aa9`](https://github.com/evogytis/baltic/commit/8184aa96265a8d225c5329b06ff9fbd0d4dbf5c7)): these figures exist only there:
  - the shutter plot
  - the Bézier "tree transformation" into trait space
  - clonal frames per segment
  - an animation pipeline for migration maps (`curonia`)
- **AI-assisted rewrite**: since 2025 the code and most documentation were written with AI help, as the README states ([`dff11c1`](https://github.com/evogytis/baltic/commit/dff11c10b2e49f6e730d89d8f6050fa69ff7708c))
  - Several defects above sit in code from this period (the tanglegram functions, `untangle`, `plot_tree_matrix`)

## Open scientific problems

- **Untangling objective**: no rule says what an untangled tanglegram should minimize
  - **Current scores**: baltic's functions score absolute row means, the retired code scored mean squared row differences, and one user script uses barycentres
  - **Crossings**: none counts crossings
  - **Warning**: the maintainer warns that parallel connectors can hide topological disagreement
- **Untangling a chain**: for more than two trees, forward and backward passes between neighbours may oscillate. Neither version tests for convergence
- **Network layout**: a reticulation takes its own row at its pre-order position, which can place it far from its hybrid node and produce long vertical lines
  - No rule chooses rows that keep reticulation edges short
- **Missing versus zero lengths**: baltic cannot tell a missing length from a zero length, which matters for resolved polytomies and for trees with partial lengths
- **Edge or node annotations in networks**: a hybrid node has two incoming edges, but baltic keeps one trait dictionary per object, so the annotation of the second edge must live on the `Reticulation`

## Not covered by baltic

- **Tanglegrams**:
  - **Crossing count**: no measure of tanglegram quality
  - **Connector filtering**: no option to hide connectors of tips whose position agrees in both trees, or to emphasize those in different MCCs
  - **Shared time axis between trees**: tanglegram x is the height from each tree's own root, so two time trees with different root ages are not aligned in time
  - **Curved tanglegram connectors**: only straight four-point connectors are built in
- **Networks**:
  - **ARG layouts**: no layout that places both parents of a hybrid node or keeps reticulation edges short, and no ARG export
- **Interaction and annotation**:
  - **Interaction**: no zoom, pan, selection, hover, or search. Every figure is static
  - **Legends**: no colour legend or size legend
  - **Label collision handling**: labels are never thinned or moved

## Method and limits

- **Code**: the files below were read at `e357bd5`:
  - `baltic.py`, `tree.py`, `io.py`, the class files, the test suite, the tutorials, and the docs sources
  - the tanglegram, network, and matrix functions of `curonia.py`
  - the untangling, date, timeline, scale bar, and export functions of `bt_utils.py`
  - **Read at signature level only**: the frequency estimator, the SNP alignment and GFF code, and the regression internals were read only at the level of their signatures and docstrings
- **History**: the full log of 335 commits, the version 0.3.0 source at [`5ddbc5f`](https://github.com/evogytis/baltic/commit/5ddbc5fa0a8d3d857b64abfeaa5158cb4bbbb8c3), and the retired notebooks at [`0b8a55f`](https://github.com/evogytis/baltic/commit/0b8a55fcd1ef04e182858167fe56878e5a07dbb7)
- **Issues**: listed all 32 issues and 17 pull requests with `gh` and read the 19 that concern parsing, plotting, untangling, and export
- **External recipes**: GitHub code search for "tanglegram", "tangled chain", and "untangling iterations" found nine public repositories that use baltic for tanglegrams
  - Six of them are cited above, and the other three repeat these recipes or draw single trees
  - Multi-word searches returned no results, so more baltic tanglegram code may exist
  - The repositories were read as text only
- **Gallery**: the example index, the "As seen in" list, and three example pages were read through the web. The gallery source repository was not cloned
- **Trials**: three scripts ran in `python:3.12-slim` with matplotlib, numpy, scipy, and requests installed into a mounted directory under `/tmp`, and with networking off for the runs
  - **Renders**: rectangular (`baltic`, `elbow`, `direct`), circular, and unrooted trees, collapsed clades, a face-to-face tanglegram and a tangled chain of the TreeKnit example trees, and the TreeKnit ARG with its hybrid labels removed, to SVG and PNG
  - **Unit trials**: the parser and tree operations on small inputs
  - **Inspection**: the SVG files were inspected as text and the PNG files as images
- **Not tried**: BEAST posterior tree sets, CoalRe network files, cartopy maps, and large real data sets
