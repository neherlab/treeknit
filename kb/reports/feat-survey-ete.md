# ETE Toolkit feature survey: tree rendering, semantics, and edge cases

This report describes the ETE Toolkit version 4 (Environment for Tree Exploration), a Python library for tree manipulation, comparison, and visualization, as an idea inventory for tree, tanglegram, network, and ARG display. ETE 4 has two renderers: `smartview`, a web explorer that streams level-of-detail drawings of huge trees from a Python server to the browser, and `treeview`, the older Qt renderer for static figures. The report records what ETE does, how it does it, which scientific conventions it assumes, which inputs break it, and which problems the maintainers left open. It does not compare ETE with `packages/web`.

ETE is licensed GPL-3.0-or-later. Its behavior and design may be studied, but copying its code needs approval (see the project rules).

- **Source**: [etetoolkit/ete](https://github.com/etetoolkit/ete), default branch `ete4`, at commit `9562dfb` (2026-06-02), 4432 commits since 2009. Source links point to this commit. The PyPI release used in the trial is 4.4.0 (2025-09-03)
- **Other sources**:
  - the ETE 3 paper ([Huerta-Cepas et al. 2016](https://doi.org/10.1093/molbev/msw046)), which the project still asks users to cite. No ETE 4 or smartview paper exists. PhyloCloud ([Deng et al. 2022](https://doi.org/10.1093/nar/gkac324)) describes a web platform built on the ETE 4 viewer
  - the documentation at <https://etetoolkit.github.io/ete/> and its sources under `doc/` (tutorials, the ETE 3 to ETE 4 migration table, the drawing internals)
  - the titles and bodies of all 554 issues and 246 pull requests, the full threads of 141 issues and 79 pull requests, the 10 release notes, the project wiki, and the git history
- **Evidence labels**: "Observed" means seen in a trial run of ETE 4.4.0 inside a throwaway Docker container (Python 3.12, PyQt6 6.11.0, Qt offscreen platform). "Derived" means read from the code without running it. Where the documentation and the code disagree, the code wins and the difference is noted
- **Shape of the code**: Python with Cython for the core
  - **Core**: the `Tree` class in [`ete4/core/tree.pyx`](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/tree.pyx), the tree operations (rerooting, ladderize, Robinson-Foulds) in [`ete4/core/operations.pyx`](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/operations.pyx), and the Newick parser in [`ete4/parser/newick.pyx`](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/parser/newick.pyx)
  - **smartview**: a bottle web server in [`ete4/smartview/explorer.py`](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py), the server-side drawer in [`ete4/smartview/draw.py`](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py), and a JavaScript client under [`ete4/smartview/static/js/`](https://github.com/etetoolkit/ete/tree/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js) that turns drawing commands into SVG (and PixiJS rasters for sequences)
  - **treeview**: a PyQt6 scene builder in [`ete4/treeview/`](https://github.com/etetoolkit/ete/tree/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview). Both its tutorial and reference page call it "not the preferred way" in ETE 4 and point to smartview [[doc](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/doc/tutorial/tutorial_treeview.rst#L3-L13)]

## Summary

- **Two trees**: no tanglegram, no side-by-side view, and no links between matching leaves exist in any renderer
  - The oldest open feature request asks for several trees in one scene ([#7](https://github.com/etetoolkit/ete/issues/7), 2011). A pull request that stacked trees was closed unreviewed ([#187](https://github.com/etetoolkit/ete/pull/187))
  - Comparison is numeric or text only: `robinson_foulds()`, `compare()`, and the `ete4 diff` node matcher
  - The `ete4 compare` command fails on every input (observed, [#795](https://github.com/etetoolkit/ete/issues/795))
- **Networks and ARGs are not supported** (see "Networks and reticulations")
  - extended Newick hybrid occurrences such as `R1#H1` become two unrelated nodes with the same name (observed)
  - drawing of horizontal gene transfer links was prototyped and abandoned ([#161](https://github.com/etetoolkit/ete/issues/161), [#247](https://github.com/etetoolkit/ete/pull/247))
- **Risks for TreeKnit users** (see "Interaction with TreeKnit output"):
  - TreeKnit's ARG annotations `[&segments={0,1}]` make the parser fail with every parser setting (observed)
  - the default parser reads internal labels as support values, so TreeKnit's `NODE_k` labels fail unless the user passes `parser=1` (observed)
- **Scientific conventions** (see "Scientific semantics"):
  - **Support is a branch property**: rerooting moves `dist` and `support` together with their branch, which is the correct behavior described by Czech et al. ([`operations.pyx#L114-L134`](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/operations.pyx#L114-L134))
  - **Root consistency check**: rerooting refuses a root whose two child branches carry different supports, because together they are one branch (observed)
  - **Missing lengths count as 1**: in both renderers a node without `dist` gets length 1, and the root gets 0
- **Huge trees by server-side level of detail** (see "smartview"): the browser sends the viewport and zoom, and the server walks the tree and returns only drawing commands for visible nodes
  - **Pixel-size collapsing**: any node whose clade is less than `node_height_min` pixels tall is drawn as a collapsed shape, and runs of small siblings merge into one shape. The browser default is 30 px
  - **Collapsed shapes**: a "skeleton" (a simplified outline of the inner topology) or a box ("outline"), with a summary of up to five names
  - **Requests per view change**: every pan and zoom fetches a new drawing after 50 ms without input, while the old SVG is scaled for immediate feedback
- **Defects with security impact** (see "Defects"):
  - smartview label expressions are evaluated with full Python builtins, so any program or web page that can send a request to the server port can run code on the host (observed)
  - node names and properties go into tooltips as HTML (derived)

## smartview: architecture and API

smartview is started with `t.explore()` or `ete4 explore -t <file>`. It runs an HTTP server in a thread of the Python process, so the user can change the tree from the Python console and from the browser at the same time [[doc](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/doc/tutorial/tutorial_smartview.rst#L63-L78)].

### Server

- **Server stack**: bottle routes served by the multithreaded cheroot server with 100 threads, bound to `127.0.0.1` by default [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L699-L725)]
- **Port**: the first free port from 5000 to 5999 when none is given [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L728-L737)]
- **Remote use**: the documented setup is an SSH tunnel to port 5000 with `open_browser=False`, so a big machine does the layout and only the graphics travel [[doc](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/doc/tutorial/tutorial_smartview.rst#L105-L122)]
- **Global state**: trees, layouts, searches, and the last drawing arguments live in module-level dictionaries, shared by every browser that connects [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L341-L347)]
  - A separation per IP address was added in 2021 ([#546](https://github.com/etetoolkit/ete/pull/546)) for issue [#545](https://github.com/etetoolkit/ete/issues/545). The 2024 rewrite does not have it, and the issue is still open
- **Optional compression**: draw responses can be Brotli-compressed (`compress=True`) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L185-L200)]
- **Self-describing API**: `GET /api` returns every route with its docstring [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L98-L103)]

### Routes

- **Read routes**: `/trees`, and per tree the routes below [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L106-L220)]
  - `size`: width is the depth in branch-length units, height is the leaf count
  - `nodecount`, `properties`, `layouts`, `style`, `draw`, `last_drawing_args`, and `search`
  - `newick`: refused above 2 MB
  - Observed on a 4-leaf tree: `size` returns `{"width": 2.0, "height": 4.0}` and `nodecount` returns `{"nnodes": 7, "nleaves": 4}`
- **Edit routes** (PUT): each applies to a node or to the whole tree [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L228-L320)]
  - `sort`: by a Python key expression
  - `move`: swap with a sibling
  - `remove`: prune a clade
  - `edit`: replace the node properties with a Newick fragment
  - `set_outgroup`, `rename`, `to_dendrogram`, and `to_ultrametric`
- **Tree management**: `POST /trees` adds a tree from JSON or from an upload form, and `DELETE /trees/<id>` removes it [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L322-L336)]
- **Node addresses**: a node is a path of child indices from the root, such as `[1, 0, 1]`
  - A subtree view appends the path to the tree name: `my_tree,1,0,1` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L358-L365)]
  - Rerooting is refused inside a subtree view

### Drawing commands

The `draw` route returns a JSON list of commands. Each is a list that starts with its name, with coordinates in tree units (not pixels). In circular mode the coordinates are radius and angle in radians [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/graphics.py#L1-L15)].

#### Command types

- **Node commands**: `nodebox` (the clade box with name, popup properties, node path, and search hits), `hz-line` (the branch), `vt-line` (the connector between children), `nodedot`, `skeleton`, and `outline` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/graphics.py#L28-L47)]
- **Face commands**: `text`, `textarray`, `line`, `arc`, `circle`, `polygon`, `box`, `rect`, `image`, `heatmap`, `seq`, and `legend` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/graphics.py#L52-L93)]
- **Panel commands**: `panel` switches between the tree (panel 0), aligned panels (1 and up), and their headers (negative numbers). `xmaxs` reports the largest x per panel so the client can place the aligned panels [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/graphics.py#L98-L102)]

#### Requests and output

- **Query parameters**: only `x y w h zx zy za layouts labels collapsed_shape collapsed_ids shape node_height_min content_height_min rmin amin amax` are accepted [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L392-L461)]
  - Anything else returns HTTP 400 "invalid keys" (observed with `min_size`)
- **Observed sizes**: a 200-leaf random tree gave 284 commands for the whole view and 141 for a zoomed viewport
- **Headless image**: `t.render_sm(file)` starts a server and takes a screenshot with Selenium and headless Chrome [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/tree.pyx#L1072-L1131)]
  - The image is 2560 px wide and 50 px per leaf by default
  - The screenshot is taken after a fixed 2-second wait

## smartview: level of detail

The drawer walks the tree once per request in pre- and post-order ([`fn walk()`](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/operations.pyx#L1152-L1167)). It decides per node whether to skip it, collapse it, or draw it [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L129-L168)].

### Node sizes and culling

- **Precomputed node sizes**: each node stores `size = (dx, dy)`, where `dx` is its branch length plus the longest path to a leaf and `dy` is its leaf count [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/operations.pyx#L1172-L1204)]
  - A missing length counts as 1, except at the root
  - Edits call `update_sizes_all` again
- **Viewport culling**: a node whose vertical span misses the viewport is skipped with its whole clade [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L396-L403)]
  - Only the vertical extent is tested, because aligned faces at the right must still be drawn when the branch is outside the view

### Collapsing

- **Collapse rule**: in rectangular mode a node is "small" when `dy * zy < node_height_min`. In circular mode the test uses the outer arc length, `(r + dr) * da * z` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L422-L424)] [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L505-L508)]
  - **Defaults**: the server default is 10 px, but the browser sends 30 px [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L89-L93)] [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/gui.js#L46-L53)]
    - Both values are adjustable in the control panel (1 to 200)
  - **Observed**: at zoom 1, a 4-leaf tree came back as one skeleton with a `(collapsed)` node box. It needed `zx=100&zy=40` to draw in full
- **Runs of small siblings merge**: consecutive small nodes are stacked into one outline box until a node is large or manually collapsed
  - A run that holds only one leaf is drawn as that leaf [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L145-L154)] [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L247-L289)]
- **Skeleton shape**: a polyline that traces the inner topology of the collapsed nodes down to a depth of 30 [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L776-L834)]
  - a child shorter than `content_height_min` pixels becomes its bounding box
  - a child shorter than 20% of that is merged into the next box
  - the alternative "outline" shape is a plain box over the collapsed clade
- **Manual collapse and leaf functions** [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L140-L146)]
  - the user can collapse a node from its context menu (`collapsed_ids`)
  - a layout can set `is-leaf-fn` to treat nodes as leaves, for example `node.level > 4`

### Faces at small sizes

- **Content threshold**: a face row is skipped when its allocated height is below `content_height_min` pixels (server default 5, browser default 4) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L597-L601)] [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L642-L649)]
  - Left, right, aligned, and header positions have no width limit
- **Summaries of collapsed nodes**: a text face on collapsed nodes shows the distinct values of its expression [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/faces.py#L154-L217)]
  - With six or more values it shows the first three plus `[...]` plus the last two
  - Nodes without a value contribute the first value found in their descendants, and `[...]` is then appended

### Open level-of-detail problems

- **Partial collapse**: a collapse that hides only part of a node's children ([#659](https://github.com/etetoolkit/ete/issues/659))
- **Large polytomies**: slow drawing ([#610](https://github.com/etetoolkit/ete/issues/610))
- **Circular mode**: slower than 5 s at low collapse levels ([#665](https://github.com/etetoolkit/ete/issues/665))
- **Aligned panel**: it looks like one band until the first zoom ([#611](https://github.com/etetoolkit/ete/issues/611))

## smartview: layouts and geometry

### Rectangular

- **Coordinates**: x is the distance from the root, y is the leaf index. Each leaf takes one unit of height [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L414-L420)]
- **Parent position**: the branch of a node is placed at the midpoint between the branch positions of its first and last child. In a polytomy the middle children do not pull the parent [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L208-L219)]
- **Zero-length branches**: the horizontal line is drawn only when `dx > 0`, so a zero-length leaf shows only its dot and label at its parent's x (observed) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L225-L231)]
  - Nothing collapses zero-length edges or marks them
- **Initial fit**: the zoom makes the tree use 60% of the window width and 90% of its height, with a 10% left margin [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/gui.js#L547-L580)]
- **Width warning**: when the tree width is 0 or the zoom becomes infinite, a dialog offers to convert the tree to ultrametric or to a dendrogram [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/draw.js#L29-L34)] [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/draw.js#L122-L141)]

### Circular

- **Angle per leaf**: `(angle_end - angle_start) / leaf_count`, from -180 to 180 degrees by default. `angle-span` can replace one of the limits, and an inconsistent triple raises an error [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L450-L474)]
- **Inner radius**: `radius` (`rmin` in the GUI) leaves an empty disc at the centre
- **Clipping to one turn**: all angles are clipped to (-π, π), so a span wider than 360 degrees is not drawn [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L535-L538)]
  - A "spiral" that stacks the overflow was proposed and not built ([#509](https://github.com/etetoolkit/ete/issues/509))
- **Equal zoom required**: the drawer asserts `zx == zy`, and the server answers HTTP 400 otherwise (observed) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L455)]
- **Readable labels**: texts whose rotation is beyond ±90 degrees are turned by 180 degrees
  - The 500 largest use the exact bounding box and the rest an estimate, because `getBBox()` is slow [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/draw.js#L1328-L1372)]
- **Angular zoom**: Ctrl+wheel in circular mode narrows or widens the angle limits around the pointer instead of zooming the radius [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/zoom.js#L49-L67)] [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/zoom.js#L106-L123)]

### Child order

- **Sort**: the default key is `(dy, dx, name)`, that is leaf count, depth, and name. The user can type any Python expression as the key and reverse it [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/operations.pyx#L13-L18)] [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/gui.js#L33-L37)]
- **Move branch up or down**: swaps a node with its neighbour, cyclically [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/operations.pyx#L199-L210)]
- **Ladderize** (API only): sorts children by the longest root-to-leaf path in the clade, then by child count. It uses branch lengths by default, `topological=True` for counts [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/operations.pyx#L367-L412)]

## smartview: faces, labels, and style

### Layouts

A layout has a name, a `draw_tree(tree)` function that returns the tree style and tree-level faces, and a `draw_node(node, collapsed)` function that returns node styles and faces. Several layouts compose, and the GUI has a checkbox per layout [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/layout.py#L70-L156)].

- **Basic layout**: three default faces [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/layout.py#L171-L199)]
  - branch length above the branch (`%.2g`, grey)
  - support below the branch (`%.2g`, light red)
  - the name to the right of leaves and collapsed groups
- **Collapsed-aware faces**: a `draw_node` with two arguments receives the list of collapsed siblings, so it can draw a summary. A one-argument function is called for every collapsed node, and all faces are combined [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/layout.py#L139-L152)]
- **Caching**: the results of `draw_node` are cached per node with `lru_cache`, unbounded by default (`cache_size=None`)
- **Tree style keys** [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/layout.py#L1-L57)]
  - geometry: `shape`, `radius`, `angle-start`, `angle-end`, `angle-span`
  - level of detail: `node-height-min`, `content-height-min`, `collapsed`, `is-leaf-fn`
  - popups: `show-popup-props`, `hide-popup-props`
  - CSS styles: `box`, `dot`, `hz-line`, `vt-line`, plus `aliases` (named styles that faces can reference)
- **Extra explore arguments** become one more layout named "extra arguments", with `_` replaced by `-` in the keys [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L674-L690)]

### Faces and positions

#### Placement

- **Positions** [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L671-L681)]
  - `top`: above the branch
  - `bottom`, `left`, `right`
  - `aligned`: a separate panel to the right
  - `header`: above the aligned panel, tree faces only
- **Columns and anchors**: faces in one position are grouped by column [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L684-L703)]
  - Each face has an anchor from -1 to 1 in x and y, where y = 0 is the branch line for `left` and `right`
- **Font size follows the space**: a text is sized to fit its box, up to `fs_max`, and is dropped below `fs_min` (2 px by default) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/faces.py#L184-L208)]
  - Text width is estimated as `length / 1.5` times the font size
- **Aligned panel**: drawn in its own `div`, by default from 75% of the window width [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/zoom.js#L91-L103)]
  - It has its own horizontal zoom (wheel over the panel) and column headers

#### Face types

- **Face classes**: `TextFace`, `PropFace`, `EvalTextFace`, `CircleFace`, `PolygonFace`, `BoxFace`, `RectFace`, `ImageFace`, `SeqFace`, `HeatmapFace`, `TextArrayFace`, and `LegendFace` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/faces.py#L80-L528)]
  - `SeqMotifFace` and `StackedBarFace` were lost in the 2024 rewrite ([#816](https://github.com/etetoolkit/ete/issues/816), open)
- **Sequences and heatmaps**: drawn as SVG or as PixiJS rasters ("auto", "force raster", "force svg") [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/draw.js#L594-L656)]
  - Only the cells inside the viewport are created
- **Legends**: discrete (a colour map with dots) or continuous (a vertical gradient between the minimum and maximum), shown in a movable box [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/draw.js#L425-L446)]

### Labels added in the browser

- **Expression labels**: "add property" or "add expression" creates a label from a Python expression on the node, such as `p.get('host', '')` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/label.js#L10-L38)]
- **Label settings** [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/label.js#L41-L106)]
  - node type: leaf, internal, or any. Choosing `right` or `aligned` switches the node type to leaf
  - placement: position, column (0 to 20), and anchor
  - text: colour, font family, and maximum size (default 15 px)
- **Palette**: labels cycle through 7 colours, `#0A0 #A00 #00A #550 #505 #055 #000`
- **Server-side text**: the label expression travels in the `labels` query parameter and is evaluated on the server for every visible node (see "Defects")

### Global style controls

- **Controls**: the control panel (Tweakpane) exposes these styles [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/menu.js#L207-L309)]
  - node box and collapsed shape
  - dot: shape (none, circle, or polygon of 3 to 8 sides), radius, opacity, and colour
  - horizontal and vertical lines: colour, width, and for vertical lines a dotted pattern
  - font size (automatic or fixed) and the sequence rendering mode
- **Defaults** [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/gui.js#L90-L121)]
  - 0.5 px black lines
  - a blue dot of radius 2 at 50% opacity
  - a collapsed stroke `#A50` at 10% opacity

## smartview: navigation, search, tags, and editing

### Zoom and pan

#### Zoom

- **Wheel zoom**: the wheel zooms both axes. Ctrl+wheel zooms only the vertical axis, and Alt+wheel only the horizontal axis [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/gui.js#L754-L768)]
- **Smart zoom** (default on, rectangular only): wheel over a node box zooms faster when the box is small on screen, so the box tends to fill the screen [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/zoom.js#L127-L146)]
  - The step is a sigmoid of the screen-to-box size ratio, limited to factors 0.5 to 1.5
- **Sensitivity**: a slider from 0 to 1, default 0.5. The zoom amount is `-0.0005 * deltaY * s / (1 - s)` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/zoom.js#L149-L157)]
  - Trackpads remain hard to tune ([#613](https://github.com/etetoolkit/ete/issues/613))
- **Smooth zoom**: the current SVG is scaled at once, and the new drawing is fetched 50 ms after the last wheel event [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/zoom.js#L160-L179)]
- **Zoom into a node**: double-click or Ctrl+click on a node box fits it with a 10% border [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/zoom.js#L13-L22)]

#### Navigation aids

- **Keys** [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/events.js#L40-L105)]
  - F1 help, `/` search, `r` reset view, `m` minimap, Esc closes menus
  - `+` and `-` zoom
  - arrow keys pan by 4% of the window (20% with Shift)
- **Minimap**: a small whole-tree drawing at the bottom right with a draggable rectangle for the current view, hidden by default [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/minimap.js#L10-L43)]
- **Shareable views**: "share view" copies a URL with `tree`, `subtree`, `shape`, and the viewport `x`, `y`, `w`, `h` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/gui.js#L596-L652)]

### Node interaction

- **Click**: a click without movement opens a small box with the node name and its popup properties (by default `dist` and `support`) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/gui.js#L729-L751)] [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L315-L327)]
- **Shift+click**: opens the clade as a subtree view, and the context menu has "Go back to main tree"
- **Context menu** (right-click) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/contextmenu.js#L50-L158)]
  - view commands: zoom into branch, go to subtree, show node id, tag, collapse or uncollapse
  - output commands: download the branch as Newick, open the NCBI Taxonomy browser when the node has `taxid`
  - editing commands, marked with a warning sign: rename, edit, set as outgroup, move up or down, sort, convert to dendrogram or ultrametric, remove

### Search

- **Simple text**: a name substring, case-insensitive when the query is all lower case and case-sensitive otherwise [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L495-L502)]
- **Commands**: `/r` regular expression on the name, `/e` a Python expression evaluated with a restricted `eval`, and `/t` a topological pattern (a Newick tree whose node names are expressions) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L505-L541)]
  - Observed: `/e __import__("os").getpid()` is refused with "invalid use of '**import**'"
- **Results and parents**: the server stores the matching nodes and all their ancestors except the root
  - Matching node boxes are filled (palette `#FF0 #F0F #0FF #F00 #0F0 #00F`, opacity 0.4)
  - The branches of ancestors are drawn black at width 5 [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L466-L492)] [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/search.js#L71-L89)]
- **Hits inside collapsed groups**: a collapsed group is marked as a hit when any of its nodes is a result or an ancestor of a result, so hits stay visible at any zoom [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L251-L253)]
- **Several searches**: each search gets its own menu folder with colour, opacity, parent line width, and a sort order that decides which colour wins

### Tags and collapse lists

- **Tags**: a named set of node paths with colour and opacity, kept only in the browser. Palette `#00F #0F0 #F00 #0FF #F0F #FF0` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/tag.js#L8-L58)]
- **Collapsed list**: manually collapsed nodes appear in a menu folder as "1 - name" with a remove button [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/collapse.js#L10-L33)]
- **Node selections were removed**: selections with postMessage events to a host page ([#580](https://github.com/etetoolkit/ete/pull/580), [#581](https://github.com/etetoolkit/ete/pull/581)) do not exist at this commit. Tags are the only selection mechanism

### Several trees

- **One tree on screen**: switching trees resets searches, tags, collapsed nodes, and the view [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/gui.js#L416-L439)]
  - There is no side-by-side view and no shared colouring between trees
- **Server holds many trees**: the root page lists them, and the control panel has a tree selector [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L583-L622)]
  - Upload accepts Newick, NEXUS (all trees of the trees block), and `.zip`, `.tar`, `.tar.gz`, `.gz`, `.bz2` archives with one tree per file

## smartview: export

- **Newick**: the whole tree or one branch, written by the server with the default parser (support as internal label, other properties as NHX) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/download.js#L9-L19)]
- **SVG**: a copy of the current view's SVG with the CSS rules inlined, the transparent foreground boxes removed [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/download.js#L22-L32)]
- **PNG**: the same SVG rasterized at the window size, with no resolution option [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/download.js#L35-L53)]
- **Aligned panel missing**: the exports contain only the tree panel
  - A 2021 fix that included it ([#555](https://github.com/etetoolkit/ete/pull/555)) was lost in the 2024 rewrite ([#817](https://github.com/etetoolkit/ete/issues/817), open)

## treeview (Qt): static figures

The Qt renderer builds a `QGraphicsScene` from a `TreeStyle`, per-node `NodeStyle`s, and faces. It needs PyQt6, an optional dependency since 2023 ([`1c3f9d9`](https://github.com/etetoolkit/ete/commit/1c3f9d9946f92d528e2a185ed0413c620ab48523)). The maintainers treat it as legacy ([#733](https://github.com/etetoolkit/ete/issues/733), [#805](https://github.com/etetoolkit/ete/issues/805)).

### TreeStyle

- **Defaults** [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/main.py#L316-L438)]:
  - shape: `mode="r"` (or `"c"`), `orientation=0` (1 draws right to left), `rotation=0`, `arc_start=0` (3 o'clock, clockwise), `arc_span=359`, `root_opening_factor=0.25`
  - scale: `scale=None` (automatic), `optimal_scale_level="mid"`, `tree_width=180` px, `min_leaf_separation=1`, `branch_vertical_margin=0`, `force_topology=False`
  - add-ons: `show_leaf_name=True`, `show_branch_length=False`, `show_branch_support=False`, `show_scale=True`, `scale_length=None`, `draw_guiding_lines=False`, `legend_position=2`, `title`, `legend`, `aligned_header`, `aligned_foot`
  - The docstring says `show_leaf_name` defaults to False and `legend_position` to 4 [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/main.py#L266-L289)]
- **Automatic scale** [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/qt_render.py#L231-L253)]:
  - `mid`: `scale = 180 px / distance to the farthest leaf`
  - `full`: the largest `face_width / dist` over all branches, so every branch is long enough for its top and bottom faces
  - circular: the scale grows until every node fits its angular wedge, with `root_opening_factor` reserving an empty disc
- **Scale bar**: a fixed 50 px bar labelled `"%g" % (50 / scale)`, not rounded to a round number (observed: "0.833333") [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/qt_render.py#L401-L443)]

### NodeStyle

- **Defaults** [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/main.py#L29-L44)]
  - colours: `fgcolor="#0030c1"`, `bgcolor="#FFFFFF"`, line colours black
  - lines: line types 0 (solid, with 1 dashed and 2 dotted), line widths 0 (a 1-px cosmetic pen)
  - node: `size=3`, `shape="circle"` (also `"sphere"`, `"square"`), `draw_descendants=True`
- **Clade background**: a non-white `bgcolor` fills the whole clade (observed) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/qt_render.py#L452-L507)]
  - in rectangular mode from the node to the right edge of the image, including aligned faces
  - in circular mode as an annular sector
- **Collapse with `draw_descendants=False` probably fails** (derived): the leaf test uses `hasattr(node, "_img_style")`, and the Cython `Tree` keeps the style in `props['_img_style']` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/main.py#L479-L481)] [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/tree.pyx#L1897-L1911)]

### Faces

- **Classes**: `TextFace`, `AttrFace`, `ImgFace`, `SequenceFace`, `TreeFace`, `RandomFace`, `DynamicItemFace`, `StaticItemFace`, `CircleFace`, `PieChartFace`, `BarChartFace`, `SeqMotifFace`, `RectFace`, `StackedBarFace`, `SVGFace`, `DiamondFace` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/faces.py#L108-L112)]
- **TreeFace**: renders a whole second tree inside a face, the only "tree next to tree" mechanism
  - The example puts a random tree as an aligned face on each leaf [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/faces.py#L469-L498)]
- **Positions**: `branch-right`, `branch-top`, `branch-bottom`, `float`, `float-behind`, `aligned` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/main.py#L25)]
  - top and bottom faces end at the end of the branch, and a branch shorter than its faces is extended with a grey dotted line [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/qt_render.py#L644-L670)]
  - float faces do not change the layout size. Z order is background, float-behind, tree, float
  - aligned faces exist only on leaves and form one table whose column widths come from all leaves
- **Face cell options**: margins, opacity, `rotable`, `hz_align`, `vt_align`, background, and border per cell [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/faces.py#L114-L168)]
- **Circular labels**: faces at angles between 90 and 270 degrees are turned by 181 degrees when `rotable` is set [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/qt_render.py#L445-L450)]

### Geometry

- **Missing lengths**: a missing `dist` counts as 1 (0 at the root), and `force_topology` sets every branch to 1 [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/qt_render.py#L1012-L1019)]
  - The circular scale search uses `node.dist or 1`, which turns 0 into 1 [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/qt_circular_render.py#L369)]
- **Rectangular**: leaves stack in child order, each with the height of its tallest face or `min_leaf_separation` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/qt_rect_render.py#L17-L71)]
  - A parent is placed at the midpoint of its first and last child
- **Circular**: leaf `i` sits at `arc_start + i * arc_span / n`, and an internal node at the midpoint of its outer children [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/qt_circular_render.py#L195-L224)]
  - The radius where a face box fits its wedge is `(h/2) / tan(angle/2)`, and the gap becomes a dotted extension
- **Half circle canvas** (observed): with `arc_start=-180` and `arc_span=180`, the image is sized for a full circle, so most of it is empty
  - Cropping happens only with `pack_leaves` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/qt_render.py#L264-L274)]

### Rendering

- **API**: `t.render(file, layout=None, w=None, h=None, tree_style=None, units='px', dpi=90)` for SVG, PDF, and PNG, and `t.show()` for the interactive window [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/tree.pyx#L1017-L1052)]
- **Headless**: works with `QT_QPA_PLATFORM=offscreen` (observed for all three formats)
  - The code sets no platform itself, and several issues report display errors ([#491](https://github.com/etetoolkit/ete/issues/491), [#764](https://github.com/etetoolkit/ete/issues/764))
- **SVG output** (observed): SVG Tiny 1.2 from `QSvgGenerator` on one line, with sizes in mm and a pixel view box
  - Each label is a `<text>` inside a transformed group, and font sizes are rewritten from numbers to points [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/main.py#L570-L612)]
- **Size**: without `w` and `h` the scene size in pixels is used. With one of them the aspect ratio is kept [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/main.py#L516-L566)]

### Interactive window

- **Mouse wheel**: Ctrl zooms both axes, Ctrl+Shift only x, Ctrl+Alt only y. Without Ctrl the wheel scrolls [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/qt_gui.py#L730-L761)]
- **Node menu** [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/node_gui_actions.py#L112-L161)]
  - topology: set as outgroup, reverse branches, collapse or expand
  - partitions: copy, cut, and paste a partition, delete node or partition
  - growth and output: populate, add children, show Newick
- **Keyboard tree walk**: arrow keys move the focus to the parent, first child, or siblings [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/qt_gui.py#L780-L835)]

## Two trees and comparison

### Graphical comparison

- **No graphical comparison**: neither renderer draws two trees with links
  - The only multi-tree display work was stacking trees with `TreeFace` containers ([#187](https://github.com/etetoolkit/ete/pull/187), closed unreviewed in 2023)
  - The wiki roadmap names a "side-by-side" diff module, and the code produces text only

### Robinson-Foulds distance

- **`robinson_foulds(t2, ...)`** returns `(rf, rf_max, common, edges_t1, edges_t2, discarded_t1, discarded_t2)` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/tree.pyx#L1226-L1387)]
  - **Rooted mode** (default): edges are clades, the sorted tuple of shared leaf names below each node, leaves and root included
    - A root with more than two children raises "Unrooted tree found!" (observed)
  - **Unrooted mode**: edges are bipartitions, a sorted pair of leaf tuples
  - **rf_max**: the number of non-trivial edges of both trees, minus 2 in rooted mode for the two roots
  - **Different leaf sets**: both trees are restricted to the shared leaves with no warning (observed: `[0, 2, {'A','B','C'}]`)
    - Duplicate names raise an error
  - **Polytomies**: `expand_polytomies=True` tries every binary resolution (3 for size 3, 105 for size 5, limit 5 by default) and returns the minimum
    - `correct_by_polytomy_size` subtracts the extra children instead
  - **Observed** with an 8-leaf pair where one leaf moved: rooted rf 8 of 12, unrooted rf 6 of 10
- **`compare(ref_tree, ...)`** wraps RF [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/tree.pyx#L1389-L1548)]
  - It returns `rf`, `max_rf`, `norm_rf`, `effective_tree_size`, and the edge sets
  - `ref_edges_in_source` and `source_edges_in_ref` are the shared edges as a fraction of each tree's edges, minus the root edge in rooted mode
  - It also returns the TreeKO speciation distance for gene trees with duplications
  - Its unit test is disabled with "TODO: Fix the compare() function" [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/tests/test_tree.py#L1129-L1130)]
- **Second RF implementation**: `operations.robinson_foulds()` counts unrooted partitions over the shared leaves, with an optional normalization by the non-trivial partitions [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/operations.pyx#L954-L1041)]
  - Its comment says it differs from the one in `tree.pyx` and needs review

### Node matching with `ete4 diff`

- **`ete4 diff` node matching**: an idea close to TreeKnit's needs [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/tools/ete_diff.py#L490-L622)]
  - every node of each tree is described by the set of leaf values below it
  - a distance matrix is computed between all node pairs. The default is `1 - |A ∩ B| / max(|A|, |B|)`, and variants add branch-length differences or the RF distance of the subtrees [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/tools/ete_diff.py#L69-L197)]
  - a linear assignment solver (`lapjv`) pairs each node with one node of the other tree, and the report lists pairs with distance above 0
  - the "topology" report prints both matched subtrees as text side by side with differing leaves marked `***` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/tools/ete_diff.py#L753-L828)]

### Other topology tools

These tools are defined in the `Tree` class [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/tree.pyx#L1606-L1691)].

- **`get_topology_id()`**: an MD5 of the sorted bipartitions
- **`check_monophyly()`**: monophyletic, paraphyletic, or polyphyletic, rooted or unrooted
- **`get_monophyletic()`**

## Networks and reticulations

- **Tree model only**: a node has exactly one parent (`up`)
  - A request for several parents per node, for orthologous groups, has no maintainer reply ([#792](https://github.com/etetoolkit/ete/issues/792))
- **Hybrid occurrences are plain names** (observed): with `parser=1`, `((A:1,(B:1)#H1:1):1,(#H1:1,C:2):1);` gives an internal node and a leaf both named `#H1`
  - Both renderers draw them as unrelated nodes
  - With the default parser it fails, because `#H1` is read as a support value
- **Rich Newick annotations are rejected**: only `[&&NHX:...]` blocks are accepted, so `[&...]` comments from BEAST or TreeKnit fail (observed) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/parser/newick.pyx#L264-L274)]
- **Horizontal gene transfer links** ([#161](https://github.com/etetoolkit/ete/issues/161), 2015, open): the proposed API was `nodeA.add_link(nodeB, type=...)`
  - The link types were node-to-node, clade-to-node, and clade-to-clade
  - A prototype for the Qt renderer converted link ends to polar coordinates but attached them to node corners. It was closed unfinished ([#247](https://github.com/etetoolkit/ete/pull/247))

## Input formats

### Newick parser codes

The `parser` argument replaces the ETE 3 `format` argument. Each code says what the two fields `p0:p1` mean for leaves and internal nodes [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/parser/newick.pyx#L123-L157)]:

- **0** (default, alias `support`): leaf `name:dist`, internal `support:dist`
- **1** (alias `name`): leaf `name:dist`, internal `name:dist`
- **2 and 3**: like 0 and 1, with every field required except at the root
- **4 to 9**: variants where some fields must be empty, for example 5 (leaf names and all lengths, no internal labels) and 9 (leaf names only)
- **100**: topology only
- **`multisupport`**: internal labels such as `80/100` become a list of floats
- **Custom parsers**: a dictionary of property descriptions (`pname`, `read`, `write`, `req`), and `make_parser()` changes the number formats for writing [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/parser/newick.pyx#L159-L174)]

### Newick parser behavior

- **Internal names fail by default** (observed): `((A:1,B:1)n1:1,C:2);` fails with "could not convert string to float: 'n1'"
  - A root label is read as support too
  - Users hit this often ([#644](https://github.com/etetoolkit/ete/issues/644), [#668](https://github.com/etetoolkit/ete/issues/668))
- **NHX**: `[&&NHX:k=v:k2=v2]` after the length goes into `props` as strings, never converted to numbers (observed) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/parser/newick.pyx#L264-L274)]
  - A value that contains `=` fails
- **Writing loses data by default**: `t.write()` uses `props=()` and `format_root_node=False`, so all NHX properties and the root name and length are dropped (observed) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/tree.pyx#L705-L727)]
  - Written property values have the characters `:;(),[]=` replaced by `_`, and lists are joined with `|` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/parser/newick.pyx#L179-L193)]
- **Comments**: `[...]` that does not start with `&` is skipped as a comment [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/parser/newick.pyx#L342-L356)]
- **Quoting**: single quotes with `''` as the escape, and double quotes as a non-standard alternative [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/parser/newick.pyx#L87-L103)]
  - Names with spaces or special characters are quoted on writing (observed round trip of `'O''Brien'`)
- **One tree per string**: the text must end with `;` and contain one tree [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/parser/newick.pyx#L290-L306)]

### Other formats

- **NEXUS**: the trees block with an optional translate table, which is applied to leaves only. A leading `[&U]` or `[&R]` is removed and ignored [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/parser/nexus.py#L26-L66)]
- **Format guessing**: from the file extension (`.nw`, `.newick`, `.tree`, `.nex`, `.nxs`, `.nexus`, `.ete`) or a `#NEXUS` start [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/parser/extract.py#L151-L161)]
- **Others**:
  - an indented text format
  - a compact `ete` format
  - PhyloXML and OrthoXML modules
  - NeXML was removed in 2024 ([#756](https://github.com/etetoolkit/ete/pull/756))

## Scientific semantics

### Branch length and time

- **Missing lengths count as 1**: in the layout (`dist()` in smartview, the Qt node box) and in ladderize, a missing length is 1 and a missing root length is 0 [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L837-L840)]
  - A tree with partial lengths is drawn as a mix of real and unit lengths, with no warning
- **No time axis**: neither renderer has a dated axis, node age error bars, or a date offset
  - Time-calibrated trees were requested in 2015 and are still open ([#112](https://github.com/etetoolkit/ete/issues/112))
  - The only scale is a scale bar (Qt) or a 100-pixel scale readout (smartview) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/draw.js#L38-L39)]
- **Ultrametric conversion**: `to_ultrametric()` keeps the root-to-farthest-leaf distance and rescales each branch by `(D - d_above) / size_below`
  - When any length is missing it falls back to unit lengths [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/operations.pyx#L421-L440)]

### Rooting and support values

- **Branch interpretation of support**: rerooting reverses the branches on the path to the new root and swaps `dist` and `support` (plus any `bprops` the caller names) with them [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/operations.pyx#L21-L47)]
  - Observed: in `(((A:1,B:1)90:1,E:1)70:1,(C:1,D:1)70:1);`, rooting on A moves support 90 to the node that now holds {C, D, E}, which is the same bipartition
  - The test suite checks that support, length, and a custom branch property follow their bipartition after random reroots [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/tests/test_tree.py#L974-L1010)]
  - Values read as names with `parser=1` are node properties, so they stay on their nodes and land on wrong branches ([#689](https://github.com/etetoolkit/ete/issues/689))
- **Root consistency check**: before a reroot, the root must have no length and no support, and its two children must have equal support, because the two root branches form one branch of the unrooted tree [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/operations.pyx#L102-L111)]
  - Observed: `((A:1,B:1)90:1,(C:1,D:1)80:1);` cannot be rerooted at all ("inconsistent support at the root: 90.0 != 80.0"), in the API and in smartview
  - IQ-TREE and RAxML trees with a binary root can hit this
- **New root placement**: `set_outgroup(node)` places the root at the midpoint of the node's branch, or at distance `dist` from the node [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/operations.pyx#L137-L159)]
  - The new root branch copies the support of the split branch, so both root branches show it
- **Midpoint rooting**: finds the two farthest leaves and cuts the path at half the diameter, at the exact point [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/operations.pyx#L609-L627)]
- **Unrooting**: a binary root is removed by rooting at one child, which merges the two root branches [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/operations.pyx#L184-L196)]

### Polytomies, zero-length branches, and single-child nodes

- **Polytomies are drawn as they are**: the parent sits at the midpoint of its outer children in both renderers
- **`resolve_polytomy()`** turns each polytomy into a caterpillar of new nodes with length 0 and support 0
  - Observed: it also sets `support=0` on most moved children but not on the first one [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/operations.pyx#L443-L469)]
- **`standardize()`** resolves polytomies and removes single-child nodes, adding their lengths to the child (observed) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/tree.pyx#L1592-L1604)]
- **Zero-length branches are kept**: the parser keeps `dist=0`, and the renderers draw the child at the parent's position with no special mark
  - Nothing collapses them for display
- **Single-child nodes**: drawn as a node on a straight line
  - `delete()` removes a node and, by default, its parent too when it is left with one child [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/tree.pyx#L390-L437)]

### Species and taxonomy

- **PhyloTree species**: a species naming function maps node names to species
  - The docstring says the first three letters are the default
  - The code removes the function when none is given, so `species` is then None [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/phylo/phylotree.py#L272-L344)]
- **NCBI and GTDB annotation**: `annotate_ncbi_taxa()` and `annotate_gtdb_taxa()` add taxonomy names, ranks, and lineages from a local database
  - smartview links nodes with a `taxid` property to the NCBI Taxonomy browser

## Defects

Each entry gives the input, the effect, and the evidence. Observed entries were reproduced with ETE 4.4.0 in a container.

### smartview security and state

- **Label expressions run arbitrary Python** (observed): the `labels` query parameter is compiled and evaluated per node with plain `eval`, which keeps the builtins
  - The request `labels=[["__import__(\"os\").getpid()","leaf","right",0,[null,null],15]]` returned the server's process id as label text
  - The search route uses `safer_eval` and refused the same call
  - A web page open in the same browser can send such a GET request to `127.0.0.1:5000` without reading the answer [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L360-L383)] [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/faces.py#L232-L235)] [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/eval.py#L18-L70)]
- **HTML in names is rendered** (derived): the click box inserts the node name and property values with `innerHTML`, so a crafted name in a tree file can run script [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/draw.js#L517-L521)] [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/gui.js#L740-L749)]
- **Clearing searches of one tree clears all trees** (derived): `PUT /trees/<id>/clear_searches` calls `g_searches.clear()` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L222-L226)]
- **Removed searches stay on the server** (derived): removing a search in the browser deletes only the client entry, and the server keeps the node sets until the process ends [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/search.js#L161-L166)]
- **Node paths go stale after edits** (derived): tags and manual collapses store child-index paths. After sort, move, reroot, or remove, the same paths point to other nodes

### smartview drawing and export

- **List styles break the client** (derived): `add_ns_prefix` calls an undefined `add_prefix` for a style given as a list, although `graphics.py` documents lists as valid styles [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/draw.js#L1180-L1202)]
- **Inconsistent return value** (derived): `draw_content` returns three values for an invisible box and two otherwise, and its callers unpack two [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/draw.py#L197-L204)]
- **Newick download cut at `#`** (derived): the download link is a `data:` URL built with `encodeURI`, which leaves `#` as a fragment marker, so a name such as `R1#H1` ends the file [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/download.js#L56-L65)]
- **SVG and PNG downloads fail on non-Latin-1 text** (derived): the SVG is encoded with `btoa`, which throws for characters such as `Ł` or CJK text [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/static/js/download.js#L28-L29)]
- **Upload error path** (derived): a bad gzip upload raises a `NameError` for `fupload` instead of the intended HTTP 400 [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/smartview/explorer.py#L570-L577)]
- **Open regressions from the 2024 rewrite**: missing faces ([#816](https://github.com/etetoolkit/ete/issues/816)) and the aligned panel missing from downloads ([#817](https://github.com/etetoolkit/ete/issues/817))

### Parser

- **NHX before the colon makes extra leaves** (observed): `((A[&&NHX:x=1]:1,B:1):1,C:2);` gives four leaves `A, 1, B, C`, and A loses its length
  - The child-list loop treats any character after a node as a separator [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/parser/newick.pyx#L326-L339)]
- **Wrong error type for missing fields** (observed): parsers 2, 3, and 5 on `((A,B),C);` raise `UnboundLocalError` for `p1_str` instead of a `NewickError` ([#799](https://github.com/etetoolkit/ete/issues/799)) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/parser/newick.pyx#L242-L250)]

### Comparison and tools

- **Support filters crash RF** (observed): `robinson_foulds(t2, min_support_t1=0.5)` and `compare()` with `min_support_*` raise `TypeError` because the code sorts a tuple of generators [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/tree.pyx#L1332-L1334)]
- **`ete4 compare` fails** (observed, [#795](https://github.com/etetoolkit/ete/issues/795)): it passes the ETE 3 argument `format=` to the tree constructor [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/tools/ete_compare.py#L92-L104)]
- **`ete4 diff` default report** (derived): the topology report calls `get_ascii()`, which ETE 4 renamed to `to_str()` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/tools/ete_diff.py#L799-L800)]
  - In the trial it stopped earlier, because the `lapjv` package is not installed with ETE
- **`Tree._diff(output='table')`** (derived): unpacks seven values from a two-element slice [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/core/tree.pyx#L1566-L1568)]
- **`ete4 explore` without a file** (derived): calls `explore()` without the required tree argument, and its `--face` option is parsed but never used [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/tools/ete_explore.py#L11-L27)]

### treeview

- **Several PyQt6 port leftovers**: `QFont.StyleItalic` ([#777](https://github.com/etetoolkit/ete/issues/777)), `QIODevice.WriteOnly`, and broken toolbar actions (open, save Newick, render region) [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/faces.py#L276-L279)] [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/qt_gui.py#L279-L310)]
- **Undocumented keys fail**: the `NodeStyle` docstring lists `node_bgcolor`, `partition_bgcolor`, and `faces_bgcolor`, which raise `ValueError` [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/main.py#L29-L44)]
- **Scale bar frame** (derived): the image grows by the bar length in height instead of the bar height [[src](https://github.com/etetoolkit/ete/blob/9562dfb6a02795dfda3975be5025f83b8dc884b1/ete4/treeview/qt_render.py#L443)]

## Performance

- **Server-side culling and collapsing** keep the response size proportional to the screen: about 284 commands for 200 leaves at full view in the trial, and the GTDB bacterial tree and the NCBI taxonomy are the maintainers' demonstrations
- **Every view change walks the tree from the root**: subtrees outside the viewport are skipped as a whole, so the cost depends on the visible part plus the path to it
- **Known slow cases**: large polytomies ([#610](https://github.com/etetoolkit/ete/issues/610)) and circular mode at low collapse levels ([#665](https://github.com/etetoolkit/ete/issues/665))
- **Client**: every response replaces the whole SVG. Sequence and heatmap cells can be drawn with PixiJS rasters, and circular label flipping limits exact bounding boxes to 500 texts
- **Python cost**: the tree and the parser are Cython. A Cython parser was tried in 2021 and replaced by the original one because it was faster ([#593](https://github.com/etetoolkit/ete/pull/593))

## Interaction with TreeKnit output

TreeKnit writes resolved segment trees in Newick with labelled internal nodes, an ARG in extended Newick, `MCCs.json` or `MCCs.dat`, Auspice JSON files, and SVG figures.

- **Resolved trees need `parser=1`** (observed with an equivalent tree): TreeKnit names unnamed or numeric internal nodes `NODE_k` and writes a root label ([packages/treeknit-io/src/newick.rs#L316-L333](../../packages/treeknit-io/src/newick.rs#L316-L333))
  - The default parser 0 reads these labels as support values and fails
  - With `parser=1` the trees load and draw, and polytomies stay as they are
  - The zero-length splits that resolution inserts are drawn with no horizontal line and no mark
- **ARG files fail to parse** (observed): every branch carries `[&segments={...}]` before the colon ([packages/treeknit-io/src/arg.rs#L67-L86](../../packages/treeknit-io/src/arg.rs#L67-L86))
  - The parser accepts only `[&&NHX:...]` and reports "invalid NHX format" with parsers 0, 1, and `name`
  - Even with NHX-style annotations the position before the colon would break the file: the length after the annotation becomes an extra leaf (see "Defects")
  - With the annotations removed and `parser=1`, the two occurrences of `R1#H1` become two unrelated nodes, and the reassortment is invisible
- **`MCCs.json`, `MCCs.dat`, and Auspice JSON**: ETE has no reader for these
  - A Python layout could colour leaves by MCC from a dictionary in `draw_node`
  - A `LegendFace` could show the MCC colours
  - Nothing links the two trees
- **Comparing segment trees**: `robinson_foulds(..., unrooted_trees=True)` and the `ete4 diff` matcher work on two resolved trees with the same leaf names
  - The matcher's pairs of nodes with distance above 0 point at incompatible clades. This information is related to the MCC decomposition and differs from it

## Ideas from the issue tracker and history

### Open requests

#### Several trees and networks

- **Several trees in one scene** ([#7](https://github.com/etetoolkit/ete/issues/7), 2011)
- **Horizontal gene transfer links** ([#161](https://github.com/etetoolkit/ete/issues/161)): node-to-node, clade-to-node, and clade-to-clade links drawn over the tree
- **Several parents per node** ([#792](https://github.com/etetoolkit/ete/issues/792)): a `MultiParentTree` subclass was suggested

#### Time and layout

- **Time-calibrated trees** ([#112](https://github.com/etetoolkit/ete/issues/112)): time intervals and credible intervals on nodes
- **Unrooted radial layout** ([#319](https://github.com/etetoolkit/ete/issues/319)): the maintainer called it a lot of work, and users asked again in 2021 and 2024
- **Spiral circular view** ([#509](https://github.com/etetoolkit/ete/issues/509)): angles beyond ±180 degrees stack outwards instead of being clipped
- **Collapsed clades as triangles or outlines** ([#458](https://github.com/etetoolkit/ete/issues/458), [#666](https://github.com/etetoolkit/ete/issues/666))
- **Search hits as an aligned column** ([#516](https://github.com/etetoolkit/ete/issues/516)): matching points in a panel next to the tree, so the distribution of hits is visible in a collapsed view
- **Layouts editable in the browser and exportable as JSON** ([#538](https://github.com/etetoolkit/ete/issues/538))

#### Distances and packaging

- **Information-theoretic tree distances** ([#468](https://github.com/etetoolkit/ete/issues/468)): bindings to the TreeDist C++ code (clustering information distance)
- **Edge-count topological distance** ([#741](https://github.com/etetoolkit/ete/issues/741)): the current topological distance counts nodes
- **Separate Qt and smartview from the core** ([#791](https://github.com/etetoolkit/ete/issues/791)), so servers never import Qt

### Declined

- **Matplotlib backend** ([#362](https://github.com/etetoolkit/ete/issues/362)): too much refactoring
- **Other comparison metrics** ([#417](https://github.com/etetoolkit/ete/issues/417)): not in the roadmap, pull requests welcome
- **Qt widget embedding** ([#566](https://github.com/etetoolkit/ete/pull/566)): closed because the Qt GUI is going unmaintained
- **Tree stacking and HGT links** ([#187](https://github.com/etetoolkit/ete/pull/187), [#247](https://github.com/etetoolkit/ete/pull/247)): closed with four other pull requests on 2023-04-19, at the ETE 3 feature freeze

### Features found only in the history

- **Per-IP separation of server state** ([#546](https://github.com/etetoolkit/ete/pull/546), 2021): each client saw only its own trees and searches
  - The 2024 renderer ([`15ecb03`](https://github.com/etetoolkit/ete/commit/15ecb03ed24f8e7e9ce51aa0c8b3f176a342149e)) does not have it
- **Node selections and embedding events** ([#580](https://github.com/etetoolkit/ete/pull/580), [#581](https://github.com/etetoolkit/ete/pull/581), [#588](https://github.com/etetoolkit/ete/pull/588), 2021): the explorer could run in an iframe and send selection events to the host page with `postMessage`, with selections inside collapsed nodes
  - The rewrite removed them
- **Aligned panel in downloads** ([#555](https://github.com/etetoolkit/ete/pull/555), 2021): lost in the rewrite ([#817](https://github.com/etetoolkit/ete/issues/817))
- **`SeqMotifFace` and `StackedBarFace` in smartview** ([#535](https://github.com/etetoolkit/ete/pull/535), [#676](https://github.com/etetoolkit/ete/pull/676)): lost in the rewrite ([#816](https://github.com/etetoolkit/ete/issues/816)). A downstream tool pins an old version for them
- **Web plugin and `WebTreeApplication`** (ETE 3): a server-side image-map viewer, moved to a separate repository ([#292](https://github.com/etetoolkit/ete/issues/292))
- **NeXML and clustering modules**: removed in 2024 ([#756](https://github.com/etetoolkit/ete/pull/756))

## Open scientific problems

- **Node or branch meaning of internal labels**: ETE decides it at parse time
  - Support read with parser 0 moves with branches on reroot
  - The same values read as names with parser 1 stay on nodes
  - No file records which meaning applies
- **The binary root as one branch**: requiring equal supports on both root children is consistent for unrooted trees
  - It blocks rerooting of rooted inference output whose two root branches carry different values
- **Missing branch lengths**: counting a missing length as 1 mixes units in one drawing
- **Comparing trees with different leaf sets**: RF silently restricts both trees to the shared leaves, so the distance hides the leaves that differ
- **Node matching between two trees**: `ete4 diff` pairs nodes by leaf-set similarity with a linear assignment, which is one answer to "which node of tree 2 corresponds to this node of tree 1" when the trees disagree

## Not covered by ETE

- **Tanglegrams**: no two-tree view, no links between matching leaves, no untangling
- **Networks and ARGs**: no hybrid nodes, no reticulation edges, no extended Newick
- **Time axis**: no dated axis, no node age intervals, no date offset
- **Continuous colour by attribute in the GUI**: colour maps exist only through Python layouts and `LegendFace`
- **Persistent selections**: tags live in the browser and are lost on reload or tree change
- **Export of the whole tree from the explorer**: SVG and PNG cover only the current window, without the aligned panel

## Method and limits

- **Code read**:
  - smartview: the server, drawer, faces, layouts, and the whole JavaScript client
  - core: the `Tree` class, the operations module, and the Newick and NEXUS parsers
  - tools: the comparison and diff tools and the command-line entry points
  - treeview: the module. The Qt treeview inventory was compiled from a full read of `ete4/treeview/`, and the cited lines were checked
- **Documentation read**: the tutorials for smartview, treeview, and trees, the migration table `doc/3to4.rst`, the drawing internals, and the abstract of the ETE 3 paper
  - The online treeview reference page has only headings and no API content
- **GitHub**: all issue and pull request titles and bodies, 220 full threads, all release notes, and the wiki. There were no rate limits or fetch failures
- **Trial**: ETE 4.4.0 from PyPI in a container built from `python:3.12-slim` with PyQt6 6.11.0
  - PyPI has no wheel for ETE 4.4.0, so the image needed `gcc` to build it
  - Tested in the tree API: Newick parsers on 15 test strings, rerooting, ladderize, polytomy resolution, RF and compare
  - Tested in the renderers: headless Qt rendering to SVG, PNG, and PDF in both modes, and the smartview HTTP API with `curl` inside the container
- **Not run**:
  - `ete4 diff` beyond the missing `lapjv` import, because `lapjv` did not build in the image
  - The smartview browser client was not opened in a browser, so client-side behavior is derived from the code
