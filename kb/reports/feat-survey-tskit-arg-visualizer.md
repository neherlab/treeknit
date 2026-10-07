# tskit_arg_visualizer feature survey: tree rendering, semantics, and edge cases

This report describes tskit_arg_visualizer, a Python package that draws ancestral recombination graphs (ARGs) from tskit tree sequences as interactive D3.js force layouts, as an idea inventory for the web app. It records what the package draws, how it computes the layout, which scientific conventions it assumes, which inputs break it, and which problems its maintainers left open. It does not compare tskit_arg_visualizer with `packages/web`.

tskit_arg_visualizer is licensed MIT, so its code may be reused with attribution.

- **Source**: [kitchensjn/tskit_arg_visualizer](https://github.com/kitchensjn/tskit_arg_visualizer) at commit `d075303` (2026-05-11), 405 commits since 2023-02-19
  - The repository did not move to the tskit-dev organization
  - Source links point to this commit
  - The last release is 0.1.2 (2026-01-07), so some fixes in this report are not yet released
- **Other sources**:
  - all 166 issues with their comments, the open and closed pull requests that carry a design discussion, the 3 discussions, `CHANGELOG.rst`, and the commit history
  - the documentation in `docs/` (`tutorial.md`, `plotting.md`, `pathing.md`) and `README.md`
  - the paper [Kitchens and Wong 2025](https://doi.org/10.1093/bioadv/vbaf302) (Bioinformatics Advances 5, vbaf302), which describes version 0.1.1
- **Evidence labels**: "Observed" and "Derived" mark how each claim was checked
  - "Observed" means seen in a trial run in a throwaway `python:3.12-slim` container with msprime 1.4.4, tskit 1.0.3, and pandas 3.0.6. The HTML output was rendered to SVG with jsdom 24 and d3 7 in a `node:20-slim` container, without a browser (see "Method and limits")
  - "Derived" means read from the code without running it
  - Where the documentation and the code disagree, the code wins and the difference is noted
- **Shape of the code**: one Python module converts a tree sequence into four pandas tables and writes a JSON payload into a JavaScript template. One D3.js file draws the graph. There is no build step and no JavaScript package
  - [`tskit_arg_visualizer/__init__.py`](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py): class `D3ARG`, conversion, subgraph selection, axis ticks, and the JSON payload (1918 lines)
  - [`tskit_arg_visualizer/visualizer.js`](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js): force simulation, edge paths, genome bar, hover interaction, and export (1382 lines)
  - [`tskit_arg_visualizer/alternative_plots/genome_bar.js`](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/alternative_plots/genome_bar.js): a static genome bar figure

## Summary

### Most relevant for TreeKnit

- **The genome bar is the central interaction**: a bar under the graph shows one block per tree of the tree sequence
  - Hovering a block highlights the edges of that local tree in the graph
  - Hovering an edge highlights the genome intervals that the edge carries
  - For a reassortment ARG with segments mapped to genome intervals, each segment becomes one block, so the bar becomes a segment selector (observed with a two-segment ARG, see "Interaction with TreeKnit output")
- **One graph edge per parent-child pair**: all tskit edges with the same parent and child become one graph edge
  - The graph edge has a list of intervals (`bounds`) and the covered fraction of the genome (`region_fraction`)
  - The edge width can follow this fraction, so edges that carry both segments are thicker than edges that carry one
- **Layout is a force simulation with a fixed time axis**: the vertical position of every node is fixed by its time (rank, linear time, or log time)
  - Sample nodes are fixed on evenly spaced columns in the order of the first tree
  - Only the horizontal position of internal nodes is free, driven by link springs and a many-body repulsion of strength -100
  - The simulation does not count edge crossings, so the user untangles the graph by dragging nodes
- **Two edge shapes**: straight lines (the default) and an orthogonal style ("ortho") with rules for where an edge leaves and enters a node
  - Ortho assumes the msprime two-node encoding of recombination and at most two parents or children per node
  - It misroutes edges for one-node reassortment (observed) and for polytomies
- **Input is tskit only**: the package reads a `tskit.TreeSequence` or its own saved JSON
  - It has no Newick, extended Newick, or NEXUS reader, so TreeKnit output must first be converted into a tree sequence

### Other findings

- **Subgraph view for large ARGs**: `draw_node()` draws the nodes within a given number of edges above and below one or more focal nodes
  - Dashed stubs with a count mark the edges that lead out of the subgraph
  - The paper used it on a SARS-CoV-2 ARG of about 2.7 million nodes
- **Styling through pandas tables**: node size, symbol, fill, stroke, labels, edge colour, mutation styles, and genome bar block colours are columns of the tables
  - There is no colour-by-attribute mapping and no legend
  - A free HTML "preamble" is the suggested way to add a legend
- **Mutations on edges**: mutations can be drawn as small rotated boxes along straight edges, condensed into one box per edge, labelled as `C123T`, and linked to ticks on the genome bar
- **Export**: JSON, SVG, and PNG (at twice the size) from a hover menu, plus `extract_x_positions_from_json()` to replay a hand-made layout
  - Several round-trip defects exist (see "Defects")
- **Experimental collapse ("zoom")**: `draw(zoom=k)` merges the `k` shortest edges into white "summary" nodes
  - It crashes in the surveyed commit (observed)

## Layout

### Coordinate system

- **Time runs down the page**: the oldest node is at the top and the youngest at the bottom [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L144-L179)]
  - The drawing area excludes 50 px at the top and bottom, so `y = (1 - f) * (height - 100) + y_shift`
  - `f` is the scaled time, and `y_shift` is 50 px, or 100 px with a title
- **Vertical position is fixed**: Python sets `fy` for every node, so neither the simulation nor dragging can move a node in time [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L1174-L1187)]
- **Horizontal position is free**: the horizontal axis carries no information. The documentation states that it is only used to untangle the graph [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/docs/pathing.md?plain=1#L11)]
- **Size options**: `width` and `height` (both default 500) set the drawing area [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L1294-L1300)]
  - The SVG grows by 100 px in width for the axis, by 75 px in height for the genome bar, and by 50 px for a title
  - Observed: `width=500, height=500` with axis and genome bar gives a 600 by 575 px SVG
- **Horizontal clamp**: on every tick a node is clamped to `[50, width - 50]`, or `[150, width - 50]` with the axis, so nodes cannot leave the drawing area [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L884-L906)]

### Time scales (`y_axis_scale`)

- **`rank`** (default): every distinct node time gets its own row, evenly spaced. Nodes with equal times share a row [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L167-L171)]
  - Observed: 25 nodes with 23 distinct times give 23 rows 18.2 px apart in a 400 px area
- **`time`**: linear in time between the minimum and the maximum time
  - The minimum may be above 0 since [#140](https://github.com/kitchensjn/tskit_arg_visualizer/pull/140), which matters for subgraphs and serially sampled data ([#122](https://github.com/kitchensjn/tskit_arg_visualizer/issues/122))
- **`log_time`**: `log10(t + 1)`, scaled between the minimum and the maximum [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L172-L177)]
  - Negative times raise an error
  - The `+ 1` offset is arbitrary, as the maintainer notes in #122
- **Mutation times can add rows**: with `show_mutations=True, ignore_mutation_times=False`, mutation times join the node times, so in rank scale each mutation gets its own row [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L1099-L1104)]
- **Discrete times are an open problem**: ARGweaver places nodes on a time grid, with parent and child sometimes at the same time
  - Linear and log scales never settle, and rank scale hides the grid
  - The maintainer prototyped a grouped axis with coloured ticks in Illustrator ([#175](https://github.com/kitchensjn/tskit_arg_visualizer/issues/175), open)

### Time axis

- **Ticks** (`y_axis_labels`): the value selects the tick mode [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L1106-L1141)]
  - `True` gives automatic ticks, and `False` hides the axis
  - A list gives ticks at those times, and a dictionary maps times to text
  - The manuscript figures use the dictionary to label days before a reference date as months ("Dec 2019") [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/docs/manuscript/Figure1/C/large_arg.py#L21-L45)]
- **Automatic ticks per scale**:
  - **Rank**: one tick per distinct time, so ticks are uneven in time but even on the page
  - **Time**: 10 evenly spaced ticks, rounded to integers (see "Defects")
  - **Log time**: the minimum, the powers of ten in between, and the maximum. A power of ten is skipped when it is close to an end
- **Tick label precision**: the number of decimals follows the smallest gap between tick values. Observed in rank scale: labels `0.013`, `0.078`, `0.139` [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L1266-L1286)]
- **Axis title** (`y_axis_title`): defaults to "Time ago (<time_units>)", with the units from the tree sequence ([#213](https://github.com/kitchensjn/tskit_arg_visualizer/issues/213)) [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L1302-L1303)]
  - Observed: "Time ago (unknown)" for a tree sequence built by hand

### Horizontal placement

#### Start positions

- **Sample order** (`sample_order`): samples sit on evenly spaced columns, in the order of the first tree in `minlex_postorder` [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L389-L396)] [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L967-L994)]
  - A partial list puts the listed samples first and appends the others in the default order. A non-sample ID raises an error
  - The README calls the first-tree order a starting point that is not always the best order [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/README.md?plain=1#L13)]
  - Observed: three samples get the columns 150, 350, and 550 px. A single sample is placed in the middle ([#55](https://github.com/kitchensjn/tskit_arg_visualizer/issues/55))
- **Internal nodes start in the middle**: every non-sample node starts at the horizontal centre and moves only through the forces [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L1181-L1182)]
- **Preset positions** (`set_node_x_positions`): a dictionary of node ID to a value between 0 and 1 fixes those nodes [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L913-L936)]
  - Values outside are clipped with a warning, and unknown IDs are ignored with a warning ([#202](https://github.com/kitchensjn/tskit_arg_visualizer/issues/202))

#### Forces and crossings

- **Forces**: d3 `forceLink` with default distance and strength, and `forceManyBody` with strength -100. There is no centring force and no crossing penalty [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L373-L381)]
- **Assisted node positioning**: in ortho mode, some nodes are locked to the horizontal position of their only child [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L537-L545)] [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L894-L899)]
  - A recombination node is locked to its only child, unless the child is also a recombination node. A non-root node with exactly one child is locked to that child too
  - Dragging either node moves both
  - The maintainer added this rule because the repulsion always pushes such nodes apart ([#65](https://github.com/kitchensjn/tskit_arg_visualizer/issues/65))
- **Local optima**: the paper states that the force layout can settle in a local optimum with more crossings than needed, which is why dragging exists ([Kitchens and Wong 2025](https://doi.org/10.1093/bioadv/vbaf302))
- **Graphviz start positions were tried and removed**: in 2023 the maintainer placed nodes with graphviz before the force simulation ([#12](https://github.com/kitchensjn/tskit_arg_visualizer/issues/12), [#28](https://github.com/kitchensjn/tskit_arg_visualizer/issues/28), [`824632b`](https://github.com/kitchensjn/tskit_arg_visualizer/commit/824632b))
  - The result helped some graphs but not others, and the change was reverted

## Edges

### Edge shapes (`edge_type`)

- **`line`** (default): a straight segment from parent to child [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L914-L921)]
  - When two graph edges join the same parent and child (a "diamond", where both parents of a recombination node are the same node), the two edges are drawn as opposite cubic curves with control points 20 px to each side, so both stay visible ([#44](https://github.com/kitchensjn/tskit_arg_visualizer/issues/44))
  - The side of each curve depends on the edge index, so the left curve can lead to the right parent position ([#179](https://github.com/kitchensjn/tskit_arg_visualizer/issues/179), open)
- **`ortho`**: right-angled paths in the style of the classic ARG figure of Griffiths (1991). Each end of an edge gets a connection code [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/docs/pathing.md?plain=1#L13-L55)]:
  - a recombination node has two parents and one child. Parent edges enter it from the left and the right, and the child edge leaves from the bottom
  - a coalescence node has one parent and two children. The parent edge enters from the top, and child edges leave from the left and the right
  - when both children lie on the same side, one edge is rerouted to the other side ("false direction"). The older child keeps its natural side, and for two parents the younger parent keeps it, to reduce crossings
  - the code pair selects one of four d3 path types: `stepAfter`, `stepBefore`, `step`, or a custom vertical-horizontal-vertical "mid" path [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L688-L882)]
  - edges start 20 px to the side of the node centre, and a short vertical stub of up to 20 px is drawn in rank scale (no stub in time scales)
- **Ortho restrictions**: ortho reads the msprime flag `NODE_IS_RE_EVENT` (131072) and one "alternative" parent and child per edge
  - Polytomies, nodes with two parents and two children, and one-node recombination are not handled ([#9](https://github.com/kitchensjn/tskit_arg_visualizer/issues/9), [#7](https://github.com/kitchensjn/tskit_arg_visualizer/issues/7), [#178](https://github.com/kitchensjn/tskit_arg_visualizer/issues/178), all open)
  - The tutorial says ortho "should only be used for full ARGs" [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/docs/tutorial.md?plain=1#L56)]
  - Observed with a four-child polytomy: three child edges leave the right side, and one of them runs down beside the samples and turns left to its child
- **Underlink** (`include_underlink`, default on, ortho only): each edge has a 12 px white path under its 4 px stroke [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L393-L397)] [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.css#L105-L109)]
  - A crossing looks like a bridge with a gap, which tells crossings apart from junctions. Observed in the ortho drawing of a 3-sample ARG
  - The underlink was shortened near nodes after small node symbols showed a white block on the edge ([#170](https://github.com/kitchensjn/tskit_arg_visualizer/issues/170))

### Edge width and colour

- **Fixed width**: 4 px for all edges [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L399-L403)]
- **`variable_edge_width`**: the stroke width is `region_fraction * 7 + 1` px, so an edge that spans the whole genome is 8 px and an edge with a tiny span is 1 px [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L405-L408)]
  - The factor 7 is arbitrary, says the maintainer
  - A user-set range such as `(2, 10)` was discussed but not built ([#43](https://github.com/kitchensjn/tskit_arg_visualizer/issues/43), [#171](https://github.com/kitchensjn/tskit_arg_visualizer/issues/171), open)
- **Edge colour**: the `stroke` column of the edge table, default `#053e4e`, set with `set_all_edge_colors()` or by editing the table
  - `set_edge_colors()` is broken (see "Defects")

## Nodes and labels

### Node symbols and styles

- **Node symbols**: any d3 symbol (`d3.symbolCircle` by default, `d3.symbolSquare`, `d3.symbolStar`, and others) [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L378-L387)]
  - Style fields and defaults: `size` in square pixels (default 150), `fill` (default `#1eebb1`), `stroke` (default `#053e4e`), and `stroke_width` (default 4)
  - `from_ts(default_node_style=...)` changes the defaults
- **Styling API**: `set_node_styles({id: {"fill": ...}})` changes single nodes, and `set_all_node_styles()` all nodes [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L798-L826)]
  - The key format was changed from a list of dictionaries to a dictionary keyed by node ID for consistency with labels ([#181](https://github.com/kitchensjn/tskit_arg_visualizer/issues/181))
- **CSS classes for groups**: each node symbol gets the classes `node`, `n<id>`, `flag<flags>`, and `sample` for samples [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L516-L540)]
  - The `styles` argument of `draw()` adds CSS rules scoped to one drawing. So all recombination nodes can be styled with `.flag131072` ([#30](https://github.com/kitchensjn/tskit_arg_visualizer/issues/30), [#132](https://github.com/kitchensjn/tskit_arg_visualizer/issues/132))
  - The flag class uses the whole bit field, so a sample that also has another flag gets `flag3` and not `flag1`

### Labels

- **Label text**: the default label is the node ID [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L499-L515)]
  - A merged recombination pair gets `u/u+1` (observed: `3/4`, `11/12`)
  - `set_node_labels({id: text})` sets labels, and `\n` splits a label into lines ([#128](https://github.com/kitchensjn/tskit_arg_visualizer/issues/128))
- **Label placement**: decided on every tick [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L1010-L1068)]
  - **Tips** (no children): centred below the node ([#29](https://github.com/kitchensjn/tskit_arg_visualizer/issues/29))
  - **Recombination nodes and roots**: centred above the node
  - **Other nodes**: above, on the side away from the parent, so the label does not sit on the parent edge ([#33](https://github.com/kitchensjn/tskit_arg_visualizer/issues/33))
  - **Offsets**: half the symbol width plus half the font size, or the CSS variables `--offset` and `--tipoffset`
- **Rotated tip labels** (`rotate_tip_labels`): tip labels turn by 90 degrees for long sample names ([#106](https://github.com/kitchensjn/tskit_arg_visualizer/issues/106)) [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L663-L668)]
  - The rotation anchor is the label start, so labels start at different distances from the symbol ([#174](https://github.com/kitchensjn/tskit_arg_visualizer/issues/174), open)
- **Label switch in the page**: the "Node Labels" menu switches between DEFAULT, `#ID`, and NONE without a redraw [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L281-L323)]
  - Turning labels off also frees nodes that labels cover for dragging ([#115](https://github.com/kitchensjn/tskit_arg_visualizer/issues/115), [#124](https://github.com/kitchensjn/tskit_arg_visualizer/issues/124))
- **No label thinning**: every label is drawn, and no overlap test exists

## Recombination and the D3ARG data model

### Conversion from a tree sequence (`D3ARG.from_ts`)

The conversion follows the documented "D3ARG" representation [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/docs/tutorial.md?plain=1#L368-L382)].

#### Nodes

- **Two-node recombination is merged into one node**: msprime with `record_full_arg=True` writes each recombination as two nodes flagged `NODE_IS_RE_EVENT`, one per parent [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L397)] [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L504-L508)]
  - The visualizer takes every second flagged node and merges it into the node before it, so the pair becomes one node with two parents, labelled `u/u+1`
  - Observed: 35 tskit nodes with 20 flagged nodes give 25 graph nodes
  - The pairing assumes adjacent IDs and is not checked. A tree sequence redated by tsdate, which can split a pair in time, crashes the conversion ([#243](https://github.com/kitchensjn/tskit_arg_visualizer/issues/243), open, workaround: convert to one-node recombination first)
- **One-node recombination stays as is**: a node with two parents and no flag is drawn as one node with two parent edges
  - Line mode handles it, ortho mode does not ([#178](https://github.com/kitchensjn/tskit_arg_visualizer/issues/178), open)
  - The maintainer suggests detecting recombination by counting parents instead of reading flags
- **Unattached nodes**: `ignore_unattached_nodes=True` drops nodes without edges but keeps the IDs of the others, which helps teaching examples simplified with `filter_nodes=False` ([#69](https://github.com/kitchensjn/tskit_arg_visualizer/issues/69))
- **Unary nodes are drawn**: the visualizer does not simplify or collapse unary nodes [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/docs/manuscript/Figure1/B/view.py#L27-L73)]
  - The manuscript script for the ARGweaver figure removes non-recombination unary nodes with its own `simplify_with_recombination()` before drawing

#### Edges

- **Edges with the same parent and child become one graph edge**: their intervals are kept as a space-separated `bounds` string such as `"0.0-27.0 34.0-85.0"`, and `region_fraction` holds the covered fraction [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L583-L651)]
  - **Rationale**: tskit edges are a denormalised encoding of a graph whose edges carry interval sets, as Jerome Kelleher put it in [#42](https://github.com/kitchensjn/tskit_arg_visualizer/issues/42). The graph then shows one line per path of inheritance and its total share of the genome
  - **Diamonds are kept as two edges**: edges from the two members of a recombination pair to the same parent stay separate, because they are two lineages (observed: two edges `16 -> 11` with different bounds)
    - With one-node recombination, a diamond collapses into one edge and the information is lost, as the maintainer notes in #42
  - **Adjacent intervals are not squashed** (observed: `"0.0-27.0 27.0-34.0"`), so the genome bar shows a boundary at 27 that is no breakpoint of this edge ([#209](https://github.com/kitchensjn/tskit_arg_visualizer/issues/209), open)
- **Pathing hints per edge**: `alt_parent` is the other parent of a recombination child, and `alt_child` is one other child of the parent [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L606-L626)]
  - Both are `-1` when absent
  - For a polytomy only the first other child is stored (observed: all four children of a polytomy point to child 0 or 1)

#### Speed

- **Array accessors**: conversion was rewritten from per-row `ts.tables` access to array accessors, about 36,000 times faster in one test ([#75](https://github.com/kitchensjn/tskit_arg_visualizer/issues/75))
  - Observed: 0.08 s for 2,733 nodes and 0.32 s for 10,146 nodes

### Tables of a `D3ARG`

- **Nodes**: `id`, `ts_flags`, `time`, `child_of` (parents), `parent_of` (children), `x_pos_reference`, `label`, and the style columns
  - The column names `child_of` and `parent_of` read in the opposite direction of their content
- **Edges**: `id`, `source` (parent), `target` (child), `source_time`, `target_time`, `bounds`, `alt_parent`, `alt_child`, `region_fraction`, `stroke`
- **Mutations**: edge, time, plotting time, site, position, ancestral, inherited, and derived state, and style columns [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L653-L692)]
  - The inherited state is the derived state of the parent mutation, so a label shows the actual change at this edge ([#111](https://github.com/kitchensjn/tskit_arg_visualizer/issues/111))
- **Breakpoints**: one row per tree with start, stop, scaled position, scaled width, and fill [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L695-L726)]

### Other constructors

- **`D3ARG.from_json()`**: rebuilds the object from a JSON file downloaded from the page, with dragged positions fixed. See "Defects" for two failures
- **`draw_D3(json)`**: draws a saved JSON again without a `D3ARG` object
- **No other formats**: there is no constructor from Newick, extended Newick, networkx, or a plain edge list
  - The `D3ARG` constructor takes the four tables directly, so a caller can build them by hand

## Interaction

### Genome bar (`tree_highlighting`, default on)

#### Hover highlighting

- **Blocks**: one dark block per tree, with 1 px white borders, below the graph. Labels show the genome start and end [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L1129-L1168)]
  - Very short trees can become invisible, so the border was reduced from 5 px to 1 px ([#32](https://github.com/kitchensjn/tskit_arg_visualizer/issues/32))
- **Hover a block: highlight the local tree**: every edge whose intervals overlap the block turns `#1eebb1` and moves to the front. The block shows its start and stop positions [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L1170-L1231)]
  - **Mutations follow the tree** ([#212](https://github.com/kitchensjn/tskit_arg_visualizer/issues/212)): the mutation display depends on the edge and the position
    - mutations on highlighted edges and inside the block stay normal
    - mutations on highlighted edges but outside the block fade to opacity 0.2
    - all other mutations are hidden
- **Hover an edge: show its intervals**: every block covered by the edge turns `#1eebb1`, and position labels appear at the outer ends of each interval [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L410-L460)]
  - Mutation ticks of this edge appear on the bar ([#112](https://github.com/kitchensjn/tskit_arg_visualizer/issues/112))
- **Hover is off while dragging**: the SVG gets the class `no-hover` during a drag, which stops flicker ([#153](https://github.com/kitchensjn/tskit_arg_visualizer/issues/153))
- **No click or lock**: the highlight lasts only while the pointer is on the block
  - An exported figure cannot show a highlighted tree, as the maintainer notes in [#59](https://github.com/kitchensjn/tskit_arg_visualizer/issues/59)

#### Other genome bars

- **Subgraph genome bar**: in `draw_node()`, regions that no shown edge covers are grey and not interactive [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L1672-L1700)]
  - Adjacent blocks that the shown edges do not separate are merged ([#85](https://github.com/kitchensjn/tskit_arg_visualizer/issues/85))
  - Observed: a 3-node subgraph of a 10-tree ARG shows 3 blocks
- **Standalone genome bar**: `draw_genome_bar(windows=[[a, b]], show_mutations=True)` draws a static bar with framed windows and mutation ticks numbered by site, alternating above and below ([#91](https://github.com/kitchensjn/tskit_arg_visualizer/issues/91)) [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/alternative_plots/genome_bar.js#L61-L136)]

### Dragging and the dashboard

- **Drag**: a node moves only horizontally, and after release it stays fixed. Other nodes keep reacting to it [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L1105-L1127)]
- **Dashboard**: a row of icon buttons above the SVG, hidden until the pointer is over the figure ([#26](https://github.com/kitchensjn/tskit_arg_visualizer/issues/26)). Each icon has a hover tip
  - **Download As**: JSON, SVG, or PNG
  - **Reheat Simulation**: frees all nodes except samples and nodes locked to a reference, puts the samples back on even columns in their current left-to-right order, and restarts the simulation with `alphaTarget(0.3)` [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L246-L263)]
  - **Space Samples**: puts samples on even columns in their current order, so the user can reorder samples by dragging and then tidy the spacing ([#48](https://github.com/kitchensjn/tskit_arg_visualizer/issues/48)) [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L265-L279)]
    - Only in the full view, because a subgraph has no defined tips ([#93](https://github.com/kitchensjn/tskit_arg_visualizer/issues/93), open)
  - **Node Labels**: DEFAULT, `#ID`, or NONE
- **No zoom or pan**: the SVG has a fixed size
  - A large ARG needs a large `width` and `height` (the manuscript used 2000 by 3000 px) and the page scrolls

### Mutations (`show_mutations`, line mode only)

- **Placement** [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L1189-L1264)]:
  - **`ignore_mutation_times=True`** (default): the mutations of an edge are spread evenly between the two ends of the edge
  - **`ignore_mutation_times=False`**: each mutation sits at its own time. A mutation with unknown time gets a random plotting time within the middle part of its edge
  - **`condense_mutations=True`**: one pink box per edge, labelled `⨉n`, and a tooltip lists all mutations ([#99](https://github.com/kitchensjn/tskit_arg_visualizer/issues/99)). It forces `ignore_mutation_times=True`
- **Shape**: a box three times wider than high (`size` 5 gives 15 by 5 px), rotated to the edge direction [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L611-L661)]
  - With `label_mutations=True` the box fits the text `C123T` at a font size of twice `size` ([#95](https://github.com/kitchensjn/tskit_arg_visualizer/issues/95), [#162](https://github.com/kitchensjn/tskit_arg_visualizer/issues/162))
- **Colours**: gold for unknown times, orange for known times, pink for condensed boxes, all with stroke `#053e4e` [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L26-L42)]
- **Hover**: shows a tooltip with the change and draws the site tick on the genome bar [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L542-L609)]
  - It also outlines all other mutations at the same site, so recurrent mutations and reversions are easy to find ([#157](https://github.com/kitchensjn/tskit_arg_visualizer/issues/157))
- **Ortho mode**: mutations are skipped with a printed warning, because the box placement assumes a straight line from parent to child (observed)
  - The same assumption misplaces mutations on the curved edges of a diamond ([#183](https://github.com/kitchensjn/tskit_arg_visualizer/issues/183), open)
- **Mutations above a root** are not drawn [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L661)]

## Large ARGs

### Subgraph around focal nodes

`draw_node` (alias `draw_nodes`) collects nodes up to `depth` edges older and younger than each seed node. `depth=[2, 0]` gives two levels of parents and no children ([#82](https://github.com/kitchensjn/tskit_arg_visualizer/issues/82), [#114](https://github.com/kitchensjn/tskit_arg_visualizer/issues/114)) [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L1581-L1704)].

- **The walk goes only up or only down**: it does not reach siblings or cousins through a shared parent, which the maintainer calls "easiest to first implement" ([#13](https://github.com/kitchensjn/tskit_arg_visualizer/issues/13))
- **Last-level edges are closed**: at the depth limit, edges between nodes already in the set are added, so a node reached by two routes is connected ([#88](https://github.com/kitchensjn/tskit_arg_visualizer/issues/88))
- **Edges leaving the subgraph**: a grey dashed stub 30 px above or below the node, with the count of missing parents or children [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L470-L497)]
  - One stub with a count replaced one stub per edge, which looked misleading for polytomies of 100 or more children ([discussion #113](https://github.com/kitchensjn/tskit_arg_visualizer/discussions/113))
- **No fixed node**: no node gets a fixed horizontal position ([#79](https://github.com/kitchensjn/tskit_arg_visualizer/issues/79), open)
  - Observed in jsdom: a 4-node chain stays on one vertical line, so the two edges into the merged recombination node `3/4` lie on top of each other
  - Samples at different times overlap
- **Always straight lines**: `draw_node()` has no `edge_type` and no `variable_edge_width` parameter (observed: `TypeError`)
- **Selection without drawing**: `subset_graph()` returns the selected tables without drawing, for scripts that need the node IDs ([#126](https://github.com/kitchensjn/tskit_arg_visualizer/issues/126))

### Collapse by edge length

- **`draw(zoom=k)`** (experimental, undocumented in the tutorial): merges the endpoints of the `k` shortest edges whose child is not a sample into "summary" nodes `S0`, `S1`, ..., at the mean time of the merged nodes, drawn white ([discussion #107](https://github.com/kitchensjn/tskit_arg_visualizer/discussions/107)) [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L1351-L1437)]
  - The mean time can place a parent below its child with serial samples ([#176](https://github.com/kitchensjn/tskit_arg_visualizer/issues/176), open)
  - Mutations stay on edges that now loop from a summary node to itself ([#156](https://github.com/kitchensjn/tskit_arg_visualizer/issues/156), open)
  - A time-bin variant, where all connected nodes in one time bin merge, was prototyped in #13 and not merged

### Notebook size

- **Each drawing embeds everything**: each drawing embeds the whole JavaScript, CSS, and JSON ([#108](https://github.com/kitchensjn/tskit_arg_visualizer/issues/108), [PR #146](https://github.com/kitchensjn/tskit_arg_visualizer/pull/146))
  - A notebook with about 25 subgraph drawings was 26 MB
  - A 40-node subgraph of a 0.5 million node ARG once took 25 MB per cell until the sample positions of the full ARG were no longer sent

## Export

- **JSON**: the payload of the drawing with the current node positions, after fixing every node at its current position [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L205-L214)]
  - The file is named `<save_filename>.json`, default `tskit_arg_visualizer.json` ([#214](https://github.com/kitchensjn/tskit_arg_visualizer/issues/214))
- **Replay a layout**: `set_node_x_positions(extract_x_positions_from_json(json))` applies a saved layout to a new `D3ARG`, for example one with new labels [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L230-L263)]
  - Positions are stored as fractions between 0 and 1 ([#137](https://github.com/kitchensjn/tskit_arg_visualizer/issues/137), [#202](https://github.com/kitchensjn/tskit_arg_visualizer/issues/202))
- **SVG**: the SVG node with every CSS rule of the page copied into a `<style>` element [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L125-L162)]
  - Illustrator and Inkscape ignore part of this CSS, so mutation ticks moved and hidden labels showed ([#127](https://github.com/kitchensjn/tskit_arg_visualizer/issues/127), [#155](https://github.com/kitchensjn/tskit_arg_visualizer/issues/155)). Version 0.1.1 moved many styles into SVG attributes for this reason
  - Observed: without the stylesheet, the underlink paths have no `fill="none"` attribute and render as black filled shapes, because only CSS sets their fill
- **PNG**: the SVG rasterized at twice the width and height [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L221-L229)]
- **No export without a browser**: the layout runs only in the browser, so a script cannot produce an SVG ([#215](https://github.com/kitchensjn/tskit_arg_visualizer/issues/215), open)

## Configuration surface

### Parameters

- **`draw()` parameters**: `width`, `height`, `tree_highlighting`, `y_axis_labels`, `y_axis_title`, `y_axis_scale`, `edge_type`, `variable_edge_width`, `include_underlink`, `sample_order`, `title`, `show_mutations`, `ignore_mutation_times`, `label_mutations`, `condense_mutations`, `is_notebook`, `rotate_tip_labels`, `zoom`, `styles`, `preamble`, `save_filename` [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L1439-L1462)]
- **Names that do not exist**: some names in use or in the documentation have no implementation [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/docs/tutorial.md?plain=1#L321-L328)]
  - `include_mutations` is `show_mutations`
  - `force_notebook` was renamed `is_notebook` in 0.1.1 ([#220](https://github.com/kitchensjn/tskit_arg_visualizer/issues/220))
  - There is no `set_edge_styles` (only `set_edge_colors` and `set_all_edge_colors`). The tutorial still documents `force_notebook` and a nonexistent `reset_all_edge_strokes()`
- **Title** (`title`): a 20 px text centred above the graph, multi-line with `\n`
  - Text in the title is drawn as text, so quotes and `<b>` stay literal (observed)
- **Preamble** (`preamble`): any HTML put above the SVG, meant for legends
  - It is inserted as HTML, so it is the user's responsibility ([#199](https://github.com/kitchensjn/tskit_arg_visualizer/pull/199))

### Output

- **Output target**: in a notebook (Jupyter, Colab, JupyterLite) the HTML goes into the cell. Otherwise it is written to a temporary file and opened in the default browser [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L75-L99)] [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L213-L221)]
- **Return value**: `draw()` returns a `DrawInfo` with the size, the element ID `arg_<random>` for scoped CSS, and the IDs of the drawn nodes ([#232](https://github.com/kitchensjn/tskit_arg_visualizer/pull/232))
- **Third-party hosts**: the page loads require.js from cdnjs and D3 from d3js.org at run time, so it does not work offline [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L13-L27)] [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L1374-L1381)]
  - Two open pull requests add a local D3 and remove require.js ([PR #246](https://github.com/kitchensjn/tskit_arg_visualizer/pull/246), [PR #248](https://github.com/kitchensjn/tskit_arg_visualizer/pull/248))

## Scientific semantics

### Time and branch length

- **Node times only**: positions come from tskit node times. There are no branch lengths, so there is no cladogram fallback and no notion of missing lengths
- **Strict time order is inherited from tskit**: tskit rejects a parent that is not strictly older than its child (observed: `TSK_ERR_BAD_NODE_TIME_ORDERING` for an edge between two nodes at time 1)
  - So a zero-length branch cannot reach the visualizer
- **Rank scale hides time but keeps order**: rank scale gives even spacing between distinct times, which keeps dense recent events readable
  - It also places nodes from unrelated parts of the graph on different rows even when their times differ by a tiny amount
- **Unknown mutation times**: drawn at a random plotting time, which makes two drawings of the same data differ [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L664-L670)]

### Recombination node encodings

- **Two-node encoding** (msprime `record_full_arg`): one node per parent of a recombination, both flagged and at the same time
  - The visualizer merges them and keeps two edges where both lead to the same parent (a diamond)
  - This is the only encoding that ortho mode handles
- **One-node encoding**: one node with two parents. Common in the literature and in reassortment ARGs
  - Drawn correctly in line mode only
  - A diamond cannot be represented, because the two edges merge into one graph edge
- **Detection by flags, not by structure**: the visualizer trusts the `NODE_IS_RE_EVENT` flag
  - The maintainer agrees that counting parents and children would be more robust ([#178](https://github.com/kitchensjn/tskit_arg_visualizer/issues/178), [#7](https://github.com/kitchensjn/tskit_arg_visualizer/issues/7))

### Edge intervals

- **Highlighting does not check connectivity**: a local tree is the set of edges whose intervals overlap the block ([#40](https://github.com/kitchensjn/tskit_arg_visualizer/issues/40))
  - In an unsimplified ARG, an edge can carry an interval that its parent edge does not, so the highlighted set can be disconnected
  - The maintainer asked whether this is wanted and closed the issue without a decision
- **Overlap versus containment**: a block highlights edges that overlap it, but an edge highlights only blocks that it fully contains [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L420-L428)] [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L1182-L1189)]

### Polytomies

- **Line mode** draws all children of a polytomy as separate straight lines from one point
- **Ortho mode** has no polytomy rule (see "Edge shapes")

## Defects

Each entry gives the input, the effect, and the evidence. Observed entries were reproduced in the trial at commit `d075303`.

### Conversion and data model

- **Split recombination pair** ([#243](https://github.com/kitchensjn/tskit_arg_visualizer/issues/243), open): `from_ts()` fails with `IndexError` or merges unrelated nodes when the two members of a recombination pair are not adjacent in ID order or not at the same time [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L590-L592)]
  - Cause: pairs are taken blindly as every second flagged node
- **Exact flag comparisons** (derived): a recombination node or a sample that carries one more flag bit is treated as a plain node [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L537)] [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L955-L962)]
  - The label `u/u+1` is given only when `ts_flags == 131072`
  - The sample check of `sample_order` accepts only `ts_flags == 1`
  - In the same check, a duplicate node ID creates a `ValueError` without raising it
- **`zoom` crashes** (observed): `draw(zoom=3)` fails with `ValueError: 'S0' is not in list` [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L1415-L1429)]
  - Cause: summary nodes get `ts_flags = 99`, and `99 & NODE_IS_SAMPLE` is 1, so they are treated as samples without a column
- **Unused helper** (derived): `_get_summary_node_subs()` calls `get_summary_descendants()`, which does not exist [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L1342-L1349)]

### Styling API

- **`set_edge_colors()` always fails** (observed): `set_edge_colors({0: "red"})` raises `TypeError: cannot unpack non-iterable int object`, because the loop unpacks dictionary keys [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L863-L876)]
  - The tutorial shows exactly this call
- **`reset_all_node_labels()` always fails** (observed): it iterates over a DataFrame as if it were a list of dictionaries and raises `TypeError` [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L751-L758)]
- **Mutation label colour** (derived): only an almost pure black fill gets white text [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L618-L629)]
  - The code picks white text when `0.2126 r + 0.7152 g + 0.0722 b <= 0.5`, but `r`, `g`, and `b` are 0 to 255
  - A dark fill such as `#053e4e` (lightness 51) keeps the dark text that [#218](https://github.com/kitchensjn/tskit_arg_visualizer/issues/218) wanted to avoid
- **Node `symbol` is evaluated as code** (derived): a crafted JSON file can run script in the page [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L520)] [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L588-L594)]
  - The symbol string `"d3.symbolCircle"` goes through `eval()`
  - The mutation tooltip inserts its content as HTML

### Axes

- **Time ticks are rounded to integers** (observed): with `y_axis_scale="time"` and node times from 0 to 2.49, the axis shows only the ticks 0, 1, and 2 instead of 10 ticks [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L1126-L1129)] [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L101-L124)]
  - Cause: `calculate_evenly_distributed_positions()` rounds to 0 decimals by default
- **Log-time end label is rounded** (observed): with `y_axis_scale="log_time"` and a maximum time of 2.494, the top tick is labelled `2`, because log-time labels get no decimals [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L1270-L1271)]
- **Tick positions are truncated** (derived): tick positions are parsed with `parseInt`, so each tick is up to 1 px above its node row [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L331-L333)]

### Layout and edges

- **Ortho with one-node recombination** (observed, [#178](https://github.com/kitchensjn/tskit_arg_visualizer/issues/178)): in a two-segment ARG with an unflagged hybrid node, ortho mode misroutes two edges
  - The edge from one parent ends in empty space beside the hybrid node
  - The child edge leaves the side of the hybrid node and turns back to the child
- **Subgraph nodes stack** (observed, [#79](https://github.com/kitchensjn/tskit_arg_visualizer/issues/79)): see "Large ARGs"
- **Mutations in diamonds** ([#183](https://github.com/kitchensjn/tskit_arg_visualizer/issues/183), open): placed on the straight line between the nodes while the edge is curved
- **Tooltip position** ([#222](https://github.com/kitchensjn/tskit_arg_visualizer/issues/222), open): the tooltip sometimes appears far below the pointer in Safari and Chrome
- **`NaN` in transforms** ([#229](https://github.com/kitchensjn/tskit_arg_visualizer/issues/229), open): the console shows `translate()` calls with `NaN`. The jsdom trial did not reproduce it

### Saving and loading

- **`from_json()` fails without mutations** (observed): a JSON saved from a drawing with the default `show_mutations=False` has an empty mutation list, and `from_json()` raises `KeyError: 'time'` [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L450-L452)]
- **`from_json()` shifts positions** (observed): with the axis shown, a node saved at 0 is reloaded at 0.111 [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/__init__.py#L436-L442)]
  - Cause: it uses an offset of 100 px and a width of `width - 150`, while the drawing uses 150 px and `width - 200`
  - `extract_x_positions_from_json()` uses the correct formula since [`9703417`](https://github.com/kitchensjn/tskit_arg_visualizer/commit/9703417) (2026-05-05), which is not released
- **SVG and PNG files have no extension** (derived): the save calls pass `save_filename` without `.svg` or `.png`, while JSON gets `.json` [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L215-L229)]
  - The tutorial says files are saved as `tskit_arg_visualizer.<format>`
- **Condensed mutations are lost on reload** ([#125](https://github.com/kitchensjn/tskit_arg_visualizer/issues/125)): the JSON holds the condensed table, so `from_json()` cannot restore the mutations

### Documentation and examples

- **`example.py` fails** (observed): it passes `return_included_nodes=True`, which was removed after [#148](https://github.com/kitchensjn/tskit_arg_visualizer/issues/148), and raises `TypeError`
- **`plotting.md` lists removed fields**: `include_label`, `sample_symbol`, `subset_nodes`, and `include_labels` per node do not exist any more [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/docs/plotting.md?plain=1#L44)] [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/docs/plotting.md?plain=1#L129-L143)]
- **`pathing.md` says the vertical axis is always rank**, which predates the time scales [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/docs/pathing.md?plain=1#L11)]
- **The standalone genome bar** (derived) has two script defects [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/alternative_plots/genome_bar.js#L139)]
  - It calls `draw_genome_bar()` once without D3 at the end of its script, which throws in the console before the real call runs
  - Its HTML loads three scripts from third-party hosts

## Performance

- **Conversion is fast**: observed `from_ts()` times of 0.08 s for 2,733 nodes and 3,321 edges, and 0.32 s for 10,146 nodes and 10,660 edges
- **JSON preparation is slow**: `_prepare_json()` loops over nodes with `iterrows()`
  - Observed `draw()` times of 0.5 s for 695 nodes, 2.1 s for 2,488 nodes, and 9.7 s for 10,146 nodes
  - A vectorised version was proposed ([#136](https://github.com/kitchensjn/tskit_arg_visualizer/issues/136), open)
- **Payload size**: observed HTML of 0.4 MB for 242 nodes and 1.0 MB for 695 nodes
  - The JSON is embedded twice, once as data and once as the escaped source for the JSON download
  - A compact column format was prototyped ([#187](https://github.com/kitchensjn/tskit_arg_visualizer/issues/187), [PR #194](https://github.com/kitchensjn/tskit_arg_visualizer/pull/194), open)
- **Rendering**: every tick updates every node, edge, label, and mutation, and the ortho path code calls `getElementById` per edge
  - The paper recommends subgraphs instead of whole-graph drawings for ARGs with many reticulations
- **Subgraph drawing is cheap**: observed `draw_node()` with depth 2 in 0.13 s on the 10,146-node tree sequence

## Interaction with TreeKnit output

TreeKnit writes its ARG as extended Newick with `#H` hybrid nodes and `[&segments={0,1}]` annotations ([packages/treeknit-io/src/arg.rs#L12-L33](../../packages/treeknit-io/src/arg.rs#L12-L33)). tskit_arg_visualizer cannot read this file, MCCs files, or Auspice JSON. It reads only tree sequences.

- **Mapping a reassortment ARG to a tree sequence** (observed): segment `k` becomes the genome interval `[k, k+1)`
  - A hybrid node becomes one node with one parent edge per segment, and a shared branch becomes one edge over `[0, 2)`
  - The trial built `((A,B),C)` and `((A,C),B)` with a hybrid node above C
- **Trial result**: the mapped ARG draws correctly in line mode
  - `from_ts()` produced 6 graph nodes and 7 graph edges. Shared edges got the bounds `"0.0-1.0 1.0-2.0"` (region fraction 1.0), and segment-specific edges `"0.0-1.0"` or `"1.0-2.0"` (0.5)
  - The genome bar showed two blocks, one per segment. Hovering a block highlights the edges of that segment tree, and hovering an edge shows which segments it carries
  - Ortho mode misrouted the hybrid edges (see "Defects")
- **Times are needed**: tskit needs node times, while TreeKnit stores branch lengths
  - Times must be derived from root-to-node distances
  - Every zero-length branch must be lengthened first, because tskit rejects a parent at the same time as its child (observed)
  - This port's `fn set_branch_lengths()` gives a reassortment node edges of `min(b1, b2) / 2`, which is 0 when one of the matching branches has length 0 ([packages/treeknit-core/src/arg.rs#L490-L540](../../packages/treeknit-core/src/arg.rs#L490-L540))
- **Segment roots**: when the two segment trees have different roots, TreeKnit adds a `GlobalRoot` with a zero-length branch
  - In a tree sequence each segment interval can keep its own root, so this artificial node is not needed
- **MCC colouring**: there is no attribute mapping
  - MCC membership must be written into the `fill` column per node, for example with `set_node_styles()`
  - A legend must be added through `preamble`

## Ideas from the issue tracker and history

### Open requests

#### Colour and linked views

- **Population column for colouring** ([#238](https://github.com/kitchensjn/tskit_arg_visualizer/issues/238)): a `p<id>` class per node so CSS can colour by population
- **Linked figures** ([discussion #60](https://github.com/kitchensjn/tskit_arg_visualizer/discussions/60)): a map or other plot next to the ARG, where hovering an edge highlights the same lineage in both views. Proposed for spatial ARGs
- **Highlight colour option** ([#195](https://github.com/kitchensjn/tskit_arg_visualizer/issues/195)): the hover colour `#1eebb1` is fixed

#### Genome bar and hover

- **Hover a node to show its genome extent and inherited mutations** ([#186](https://github.com/kitchensjn/tskit_arg_visualizer/issues/186)): highlight on the genome bar where a node carries known sequence
  - The maintainer suggests an interactivity menu that turns hover features on and off
- **Node hover with metadata** ([#50](https://github.com/kitchensjn/tskit_arg_visualizer/issues/50))
- **Restrict genome coordinates** ([#70](https://github.com/kitchensjn/tskit_arg_visualizer/issues/70)): an `x_lim` or automatic trimming of empty flanking trees, as tskit `draw_svg` does
- **Mutation tooltip content** ([#105](https://github.com/kitchensjn/tskit_arg_visualizer/issues/105)): a user function that maps a mutation row to its tooltip text

#### Layout and labels

- **Space samples per time point** ([#119](https://github.com/kitchensjn/tskit_arg_visualizer/issues/119)): evenly space only samples of the same time, for serial samples
- **Label inside large nodes** ([#224](https://github.com/kitchensjn/tskit_arg_visualizer/issues/224)): a `node_label_offset` option or CSS positioning

#### Output and packaging

- **Download as HTML** ([#89](https://github.com/kitchensjn/tskit_arg_visualizer/issues/89)): a self-contained interactive page with the current layout [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/visualizer.js#L231-L244)]
  - It was built and then commented out because it broke in some environments
- **Programmatic SVG download** ([#215](https://github.com/kitchensjn/tskit_arg_visualizer/issues/215))
- **Load shared code once per notebook** ([#39](https://github.com/kitchensjn/tskit_arg_visualizer/issues/39), [PR #169](https://github.com/kitchensjn/tskit_arg_visualizer/pull/169)): a `setup_notebook()` in the style of Bokeh's `output_notebook()`
  - Not merged, because SVG export needs the styles inside the drawing
- **anywidget output** ([#245](https://github.com/kitchensjn/tskit_arg_visualizer/issues/245), [PR #244](https://github.com/kitchensjn/tskit_arg_visualizer/pull/244)) and **a PyScript version** ([#63](https://github.com/kitchensjn/tskit_arg_visualizer/issues/63))

### Declined

- **Position scaling in JavaScript** ([#23](https://github.com/kitchensjn/tskit_arg_visualizer/issues/23)): needed for an interactive time scale or interactive collapse, declined because it would change how the package works
- **Expand nodes by right-click** ([#13](https://github.com/kitchensjn/tskit_arg_visualizer/issues/13), [#114](https://github.com/kitchensjn/tskit_arg_visualizer/issues/114)): the browser would need the whole graph or a channel back to Python
- **Compressed JSON** ([#109](https://github.com/kitchensjn/tskit_arg_visualizer/issues/109)): closed as won't-fix after the sample-position bloat was removed
- **Time-slice view** ([#51](https://github.com/kitchensjn/tskit_arg_visualizer/issues/51)): closed in favour of the subgraph view
- **Snapping horizontal positions to a grid** ([#21](https://github.com/kitchensjn/tskit_arg_visualizer/issues/21)): closed without a change, with the concern that snapping makes crossings worse
- **Automatic resizing to the window width** ([#26](https://github.com/kitchensjn/tskit_arg_visualizer/issues/26)): declined to keep figures reproducible

### Features found only in the history

- **Graphviz initial positions** ([`49df278`](https://github.com/kitchensjn/tskit_arg_visualizer/commit/49df278), reverted in [`824632b`](https://github.com/kitchensjn/tskit_arg_visualizer/commit/824632b), 2023)
- **Subset highlighting** (`subset_nodes`, [#13](https://github.com/kitchensjn/tskit_arg_visualizer/issues/13)): chosen nodes and the edges between them at full opacity, the rest faint
  - Removed with the move to pandas tables in [`c3dd151`](https://github.com/kitchensjn/tskit_arg_visualizer/commit/c3dd151)
- **Copy layout to the clipboard** ([#11](https://github.com/kitchensjn/tskit_arg_visualizer/issues/11)): replaced by the download menu in [`ff4963d`](https://github.com/kitchensjn/tskit_arg_visualizer/commit/ff4963d)
- **Click to lock a mutation** ([#14](https://github.com/kitchensjn/tskit_arg_visualizer/issues/14)): each click gave the mutation a random colour so several could be compared
  - The mutation tooltip replaced it, because the two interactions conflicted
- **Edge span plot**: `alternative_plots/edge_spans.js` draws one horizontal line per edge over its genome interval, but no Python function calls it ([`5ea77f2`](https://github.com/kitchensjn/tskit_arg_visualizer/commit/5ea77f2)) [[src](https://github.com/kitchensjn/tskit_arg_visualizer/blob/d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee/tskit_arg_visualizer/alternative_plots/edge_spans.js#L1-L49)]

## Open scientific problems

- **Untangling a graph with a fixed time axis**: the force model knows nothing about crossings, and graphviz starts helped only some graphs. The paper names local optima as the reason for manual dragging
- **Encoding of recombination**: the two-node and one-node encodings carry different information (diamonds), and tools that redate or simplify ARGs can break the two-node pairing
- **What a local tree is in an unsimplified ARG**: interval overlap can select a disconnected set of edges ([#40](https://github.com/kitchensjn/tskit_arg_visualizer/issues/40))
- **Discrete and tied times**: neither linear nor rank scale shows ARGweaver's time grid well ([#175](https://github.com/kitchensjn/tskit_arg_visualizer/issues/175))
- **Collapsing a graph for overview**: summary nodes at mean times can invert time order, and trees can no longer be extracted from a collapsed graph (discussion #107)
- **Which nodes anchor a subgraph layout**: samples are a good anchor for the full graph, but a subgraph has no natural tips ([#93](https://github.com/kitchensjn/tskit_arg_visualizer/issues/93))

## Not covered by tskit_arg_visualizer

- **Input and comparison**:
  - **Tree file formats**: no Newick, extended Newick, NEXUS, or Auspice input
  - **Side-by-side trees and tanglegrams**: local trees appear only as highlights inside the graph
- **Display**:
  - **Attribute-driven colour and legends**: no colour scale, no categorical palette, and no generated legend
  - **Branch-length or support display**: no branch labels, support values, or error bars
  - **Zoom, pan, and label thinning**: the SVG has a fixed size, and all labels are drawn
- **Editing and navigation**:
  - **Rerooting, rotation of children, and collapsing clades**
  - **Search and selection**: no find box and no persistent selection
- **Deployment**:
  - **Offline use**: D3 and require.js come from third-party hosts

## Method and limits

- **Code**: read all of `__init__.py`, `visualizer.js`, `visualizer.css`, both files in `alternative_plots/`, the tests, `example.py`, and the manuscript scripts at commit `d07530334eb8eac6c1a7f3b89fe8d4e3893a62ee`
  - CI files, lockfiles, and packaging files other than `pyproject.toml` were skipped
- **Documentation**: read `README.md`, `docs/tutorial.md`, `docs/plotting.md`, `docs/pathing.md`, `docs/notes.md`, and `CHANGELOG.rst`
  - The paper was read through its publisher page, as summarised by a fetch tool, so statements from the paper are limited to its abstract-level claims
- **History**: read all 166 issues with comments through `gh`, the pull requests with design discussions (#19, #45, #56, #64, #66, #138, #146, #169, #173, #194, #244, #246, #248), and the 3 discussions
- **Trial**: ran the cloned package (on `PYTHONPATH`, not installed) in `python:3.12-slim` with msprime 1.4.4, tskit 1.0.3, numpy 2.5.3, and pandas 3.0.6. `webbrowser.open` was replaced to capture the HTML files
  - Inputs:
    - an msprime full ARG with 3 samples and 10 trees, and the same ARG with mutations
    - a hand-built two-segment reassortment ARG
    - a four-child polytomy and a zero-length edge
    - msprime ARGs with 50, 200, 1,000, and 5,000 samples
- **Rendering without a browser**: the HTML was loaded in jsdom 24 with d3 7 injected in place of the CDN, the force simulation ran for 6 s, and the SVG was converted to PNG with `rsvg-convert` after adding the package stylesheet. Limits:
  - jsdom has no text measurement, so drawings with `label_mutations=True` failed in the trial
  - pointer interaction (hover, drag, download) was not exercised, so all interaction claims are derived from the code
  - jsdom computes no layout for HTML, so tooltip positions were not checked
- **No injected instructions** were found in the repository, issues, or web pages
