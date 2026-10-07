# Taxonium feature survey: tree rendering, semantics, and edge cases

This report describes Taxonium, a web viewer for phylogenetic trees with up to tens of millions of tips, as an idea inventory for the web app. It records:

- **Drawing**: what Taxonium draws and how
- **Semantics**: which scientific conventions its data model assumes
- **Faithfulness**: where the picture can mislead
- **Edge cases**: which inputs break it
- **History**: how its maintainers found and fixed problems, and which problems stay open

The report does not compare Taxonium with `packages/web`.

Taxonium is licensed GPL-3.0. Its behavior and design may be studied. Copying its code needs approval (see the project rules).

- **Source**: [theosanderson/taxonium](https://github.com/theosanderson/taxonium) at commit `ae4bceb` (2026-09-04), 2530 commits since 2021. Source links point to this commit
- **Live application**: <https://taxonium.org>, operated in Chrome on 2026-10-07 with the public Mpox clade IIb and H5N1 B3.13 trees, and with test trees loaded through the `treeUrl` parameter
- **Other sources**: the survey also used these sources
  - All 270 issues, the 559 pull requests, the 6 discussions, and commit diffs
  - The Taxonium paper ([Sanderson 2022](https://doi.org/10.7554/eLife.82392)) and its peer review
  - The Treenome Browser paper ([Kramer et al. 2023](https://doi.org/10.1093/bioinformatics/btac772)) and the Chronumental documentation
  - Issues in other repositories that cite Taxonium (Pango designation, UShER, CoV-Spectrum)
- **Evidence labels**: "Observed" means seen in the live application. "Derived" means read from the code without running it. Where the documentation and the code disagree, the code wins and the difference is noted
- **Shape of the code**: Taxonium has five parts
  - A React component (`taxonium_component`) that draws with deck.gl in WebGL
  - A Next.js website around the component
  - A Node backend for very large trees
  - An Electron desktop app
  - The Python package `taxoniumtools`, which converts UShER and Newick trees to the Taxonium JSONL format
  - The filtering and search code in `taxonium_data_handling` is shared by the in-browser web worker and the Node backend

## Summary

- **Scale first**: positions are computed once, at conversion or load time, and nodes are stored sorted by y. The viewer never computes layout again. For this reason, re-rooting, collapsing, and radial layouts are declined ([#447](https://github.com/theosanderson/taxonium/issues/447))
- **Level of detail by binning**: in each viewport query, only the topmost tip per cell of a grid of 400 columns and 2000 rows is kept, and then all its ancestors are added
  - Vertical extent stays faithful to tip counts
  - Rare tips, point proportions, and the legend depend on the zoom level
  - Nothing on screen says that tips were omitted
- **Mutation-annotated trees are the native format**: genotype colouring, mutation and genotype search, and the Treenome genome view need branch mutations. Only UShER (through `taxoniumtools`) and Nextstrain JSON provide them. Genotypes are parsimony-inferred states, never observed data
- **No units on screen**: each tree is rescaled by its own 95th or 99th percentile x, and the units are not stored. No axis, scale bar, or node distance is shown. This request has been open since 2021 ([#86](https://github.com/theosanderson/taxonium/issues/86))
- **Colour by hash**: category colours come from a string hash
  - The hash keeps colours stable across datasets
  - The hash gives sibling names such as `BA.1` and `BA.4` nearly identical colours
  - Numeric fields use a fixed log10 plasma scale, and their legend shows wrong colours
- **Shareable state in the URL**: searches, colour field, and x axis are in the query string. The camera and display settings are not
- **Many silent failures**: Taxonium never reports parse errors
  - Nexus files with a TRANSLATE block or with several trees hang on "Laying out the tree" (observed)
  - BEAST annotation values keep stray quotes and brackets (observed)
- **No support for two trees**: no issue, pull request, or discussion asks for a tanglegram, side-by-side trees, or linked selection. The architecture (one worker per page, module-level caches) breaks when a second tree is loaded in the same page

## Architecture and data model

- **Taxonium JSONL**: the first line is a header with the config and one shared mutation table. The root is the node whose `parent_id` equals its own `node_id` [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxoniumtools/src/taxoniumtools/utils.py#L219-L266)]. Each following line is one node with these fields:
  - `name`, `node_id`, and `parent_id`
  - `x_dist`, optional `x_time`, and `y`
  - `mutations` (indices into the mutation table) and `num_tips`
  - `meta_*` fields and optional `clades`
- **Nodes sorted by y, `node_id` equal to the array index**: a viewport query finds the visible y range with two binary searches and slices the array. Parent lookups are direct array accesses [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_data_handling/filtering.js#L75-L133)]
- **Two interchangeable backends**: a web worker serves files loaded in the browser, and a Node server serves large trees. Both have the same query, search, details, and export interface. The desktop app starts the Node server as a child process with a heap limit of three quarters of system RAM, and embeds the viewer with `backendUrl` [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useBackend.ts#L11-L43)] [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_electron2/main.js#L36-L134)]
- **Mutations as indices**: nodes carry integer indices, and the client replaces them with mutation objects after each query. The server streams the mutation table in chunks of 10,000, because one JSON response with all mutations exceeded V8's string limit ([#613](https://github.com/theosanderson/taxonium/issues/613)) [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_backend/server.js#L233-L272)]
- **Root genome as pseudo-mutations**: `taxoniumtools` stores the root sequence on the root node as one mutation per genome position and one amino-acid mutation per codon. The docstring calls this a "Hacky way of recording the root sequence". Genotype colouring, genotype search, revertant search, and Treenome all depend on it [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxoniumtools/src/taxoniumtools/ushertools.py#L301-L338)]
- **Config from four sources**: later sources win. Without a config file, search types and colour options are generated from the fields of the first node [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useConfig.tsx#L40-L84)] [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_data_handling/importing.js#L430-L566)]. The order is:
  1. Backend or file config
  2. The `config` URL parameter
  3. A file from `configUrl`
  4. The `configDict` component property

## Layout

### Coordinates

- **Rectangular layout only**: the root is at the left, tips are to the right, and each tip has one row. There is no radial, circular, or unrooted layout. The Taxonium paper names this a trade-off for scale
- **Equal tip spacing**: tips get consecutive ranks. The vertical extent of a clade is therefore proportional to its tip count [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/utils/jstree.js#L517-L527)] [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_data_handling/importing.js#L373-L383)]
  - In the browser importers, `y = rank / (n_tips - 1) * 2000`
  - JSONL files are rescaled at load to `y * 2400 / N` (N = all nodes, or `0.6666 N` below 10,000 nodes) and rounded to 6 decimals
- **Internal node y**: the y of an internal node is the midpoint of its first and last child. The mean of the children is not used. In a polytomy with three single tips and one large clade, the parent sits halfway to the large clade [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/utils/jstree.js#L520-L526)]
- **x scaling per tree**: the original unit is not stored [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxoniumtools/src/taxoniumtools/utils.py#L145-L157)] [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/utils/processNewick.ts#L132-L144)]
  - `taxoniumtools` maps the 95th percentile of all node x values to 600
  - The browser importers map the 99th percentile to 450
  - Two trees of the same samples are therefore on different scales
  - The same Newick file gets different x scales when it is loaded in the browser and when it is converted with `taxoniumtools`
- **Division by zero is not guarded** (derived): if the percentile value is 0, for example in a star tree, Python raises an error and the browser gets an infinite scale
- **Two x axes**: a node can have `x_dist` (divergence) and `x_time` (time). A "Tree type" selector switches between them when both exist, and switching refits only the x axis. If the first node lacks one of them, the viewer switches to the other and rewrites the URL (observed: `xType=x_time` on the Mpox tree became `x_dist`) [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/Taxonium.tsx#L172-L212)]
- **Initial view fits a robust maximum**: since 2026, the initial x zoom fits `min` to `min + 1.3 (q99 - min)`, where q99 comes from a 4096-bin histogram. The goal is that a few very divergent sequences do not push the tree off screen [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_data_handling/importing.js#L114-L217)]
  - The motivating case was a dengue tree with x up to 2805, while 99% of nodes were below 620 ([#833](https://github.com/theosanderson/taxonium/pull/833))
  - Branches beyond the fit are off screen in the main view and in the minimap, with no marker

### Child order

- **Ladderize in opposite directions**: the same tree loaded in the two ways is mirrored [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/utils/processNewick.ts#L207-L228)] [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxoniumtools/src/taxoniumtools/usher_to_taxonium.py#L91-L93)]
  - The browser importers sort children by ascending tip count (small clades on top) when the per-file "ladderize" option is on. This option is the default on the website
  - `taxoniumtools` sorts by descending tip count (large clades on top)
- **Nextstrain child order reversed** (derived): without ladderize, children come out in reverse file order, because the pre-order walk pops them from a stack [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/utils/processNextstrain.js#L314-L369)]
- **The URL parameter and the registry disagree**: `ladderizeTree` is honored only as the string `"true"`. The dataset registry stores a JSON boolean for the full NCBI taxonomy, so that tree is not ladderized [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_website_next/src/hooks/useInputHelper.jsx#L276)]

### Branch lengths, polytomies, and zero-length edges

- **Polytomies are kept**: no input path binarizes the tree
- **Zero-length edges are not collapsed or marked**: a node draws its vertical segment at the parent's x, so a zero-length internal node lies on the parent's bar. A resolved binary split with length 0 and a polytomy look the same. Observed with `data/h3n2-new-york-1999-2004/ha.nwk` (50 zero-length branches): zero-length tips sit directly on their parent's bar, and their labels overlap
- **Missing and negative lengths become 0**: the rule is `x = parent.x + (d >= 0 ? d : 0)`. Only when every length is missing does the layout fall back to a cladogram with tips aligned at the maximum depth. In that case the "Tree type" label still says "Distance" [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/utils/jstree.js#L529-L553)]
- **History**: tips with zero-length branches disappeared because they shared a bin with their parent. Binning was then restricted to tips ([#168](https://github.com/theosanderson/taxonium/pull/168))

## Level of detail and sampling

The client asks for the visible box padded by one view on each side (three times the view in each axis). The backend thins the tips in that box, adds ancestors, and returns the result. A coarse thinned copy of the whole tree fills the screen while new data loads.

### Binning

- **Grid**: the precision is `2000 / (max - min)` per axis of the query box, and the x precision is divided by 5. This gives 2000 rows and 400 columns. Each tip is snapped to `round(v * p) / p` [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_data_handling/filtering.js#L44-L73)] [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_data_handling/filtering.js#L135-L178)]
- **Winner is the first tip in array order**: arrays are sorted by y, so the topmost tip of each cell is kept and all others are dropped. There is no random choice, no majority vote, and no aggregation of attributes
- **Ancestors of every kept tip are added**, also when they are far outside the box, so the drawn tree is always connected. Internal nodes in the box whose tips all lost their cells are not drawn
- **Local and server modes thin differently** (derived): the worker clamps only y to the tree extent, and the server clamps x and y. The grid and the surviving tips therefore differ for the same file [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/webworkers/localBackendWorker.js#L61-L109)] [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_backend/server.js#L274-L332)]
- **Worked numbers** (derived): on a canvas of 900 by 1500 px, a grid row is 1.35 px and a column is 11.25 px, while points have a 6 px diameter
  - In a whole-tree view of a 1,000,000-tip JSONL file in local mode, one row holds about 500 consecutive tips and keeps one tip per distinct root-to-tip distance class
  - Every tip is drawn only when fewer than about 667 tips are visible

### Refetching and fill-in

- **Refetch rule**: a new query is sent when the view comes within half a view of the box edge, or when the y zoom changes by more than 0.5 (log2). Requests are debounced by 100 ms, with one request in flight. A pure x zoom does not refetch, so tips hidden by the coarse columns stay hidden after a horizontal zoom-in, although there is room to draw them [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useGetDynamicData.tsx#L46-L160)]
- **Fill-in layers**: while the view is outside the loaded box, the coarse whole-tree data is drawn in the main view, so panning never shows blank space [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useGetDynamicData.tsx#L76-L88)]
  - A truthiness test (`vs.min_x &&`) disables fill-in when the left edge of the view is exactly 0
  - The fix ([#699](https://github.com/theosanderson/taxonium/pull/699)) was merged into a side branch (`new-react`) and never reached the main line

### Design statement

The Taxonium paper says that the filtered view shows "essentially the same tree" as the full one. The eLife reviewers asked which tips are rendered and whether the choice could depend on metadata. This question was not answered ([#437](https://github.com/theosanderson/taxonium/issues/437) is open).

## Branches, nodes, and marks

- **Two line layers per tree**: each node draws its own elbow [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useLayers.tsx#L247-L290)]
  - A horizontal segment goes from `(x, y)` to `(parent_x, y)`
  - A vertical segment goes from `(parent_x, y)` to `(parent_x, parent_y)`
  - All branches have one colour. There is no colour or width by attribute
- **Hover and selection by width** (observed): other branches are 1 px wide
  - The hovered branch is drawn 3 px (horizontal) and 2 px (vertical) wide, and the hovered node gets a ring of 4 px at 30% opacity
  - The selected branch is drawn 3.5 px and 2.5 px wide, and the selected node gets a black ring of radius 6 px
- **Points for tips only**: internal points are hidden unless "Display points for internal nodes" is on. Point outlines are drawn only when fewer than 3000 nodes are loaded [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useLayers.tsx#L166-L245)]
- **Pretty stroke**: an optional second point layer underneath, slightly larger and in a dark colour, gives every point an outline. The settings hint says that it looks best at opacity 1. It reuses the layer id of the main point layer
- **Draw order**: branches, fill-in branches, pretty stroke, points, fill-in points, clade labels, selected ring, hovered ring, tip labels, search rings, minimap
- **Picking radius**: hover and click use a radius of 10 px [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/Deck.tsx#L163-L167)]

## Labels

- **Tip labels by global density**: labels are drawn when `num_tips / 2^zoomY < 0.8 * 10^threshold` (default threshold 2.9, about 636). The rule uses the total tip count of the tree instead of the tips in view, so in a sparse region of a large tree labels appear late [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useLayers.tsx#L425-L459)]
- **Font and placement**: tip labels use Roboto weight 100 at 9.5 px (12 px when fewer than 200 nodes are loaded), in light grey `(180,180,180)`, 10 px right of the tip. At the zoom where labels switch on, a Newick tree has about 3 px per tip. Each label then overlaps the next 3 to 5 labels and can be paired with the wrong tip (observed on the Mpox tree; derived numbers)
- **No collision handling** for tip or clade labels
- **Clade labels**: Taxonium labels the 10 nodes with the most tips among the loaded nodes that carry a `clades.pango` value. The labels are bold, 11 px, and anchored left of the node [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useLayers.tsx#L134-L148)]
  - The accessor is the constant `"pango"`, so Nextstrain clade annotations are not shown
  - Ancestors of every loaded tip are loaded, so the large ancestral clades on the path to the root take the slots, even when they are off screen
- **Internal labels as names**: with "Display labels for internal nodes", Newick internal labels are drawn verbatim. Support values are not separated from names. The H3N2 tree shows labels such as `98__iesFJ` (observed)
- **Non-ASCII labels** (derived): no `TextLayer` sets `characterSet`, and the default deck.gl glyph atlas covers ASCII only. Names with letters such as `é` or `Ś` therefore do not render completely
- **Label colour cannot come from config**: label colour is a display setting, and display settings are not configurable ([discussion #716](https://github.com/theosanderson/taxonium/discussions/716))

## Colour and legend

### Colour modes

- **Colour by a metadata field, by genotype at a position, or none**: the URL stores the field, gene, and position as `color={"field":...,"gene":...,"pos":...}` [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useColorBy.tsx#L21-L120)]
  - The default field is `config.defaultColorByField`, else `meta_pangolin_lineage`
  - The genotype default is gene `S` (else `nt`) at position 484, a SARS-CoV-2 heritage
- **Only points are coloured**: branches keep one colour, so colour shows tip attributes and never the inferred state of an internal branch
- **Colour precedence**: Taxonium takes the first rule that applies [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useColor.tsx#L14-L150)]
  1. A configured continuous ramp (`colorRamps`)
  2. JavaScript numbers on a log10 plasma scale
  3. The user and config `colorMapping`
  4. A fixed amino-acid palette
  5. Greys for missing values
  6. Hard-coded values (countries, continents, sequencing platforms, `B.1.2`)
  7. The string hash

### Category colours by hash

- **Algorithm**: reverse the string, compute a Java-style 32-bit hash, and take bytes 0, 1, and 2 as red, green, and blue. If the sum is below 150 or above 500, hash again with `_` appended [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useColor.tsx#L134-L150)]
- **Why a hash**: colours stay the same across datasets and over time. Users reported three region names in near-identical purples. The maintainer kept the hash and added hard-coded colours for the reported values ([#297](https://github.com/theosanderson/taxonium/issues/297), [#620](https://github.com/theosanderson/taxonium/pull/620))
- **The retry undoes the reversal** (derived): the retry hashes the reversed string plus `_`, and reversing that again puts the original string last. The last character, which distinguishes sibling lineage names, then changes only the red byte by a few units. About 23% of names take the retry
  - Computed with a re-implementation of the function: `BA.1` is `(33,179,89)` and `BA.4` is `(36,179,89)`
  - `JN.1.2`, `JN.1.6`, and `JN.1.7` differ by at most 5 in red
  - Observed on the Mpox tree: B.1, G.1, and F.1 got almost the same yellow
- **Collision rate** (Monte Carlo estimate, CIELAB difference below 10): at least one hard-to-separate pair occurs among 10 categories in 26% of random name sets, and among 20 categories in 73%. Under simulated deuteranopia, the rate for 10 categories is 81%
- **Single letters take amino-acid colours**: a category named `A` gets the alanine colour from the residue palette
- **Global cache keyed by value** (derived): one module-level cache serves all fields. A numeric string coloured by hash therefore keeps that colour when a ramp field has the same value

### Numeric values

- **Fixed log10 plasma scale**: `t = clamp(log10(v) / 10, 0, 1)`, so the scale spans 1 to 1e10 whatever the data [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useColor.tsx#L42-L54)]
  - Values from 0 to 100 use one fifth of the palette
  - The years 2020 and 2024 get the same colour
  - 0 and values below 1 get the colour of 1
- **Negative values throw** (derived): `log10` of a negative number is NaN, and the plasma function indexes its palette with NaN and fails
- **Numbers only from `taxoniumtools`**: pandas keeps numeric columns as JSON numbers, while metadata joined in the browser stays text. The same column is therefore continuous in one path and categorical in the other
- **Configured ramps**: `colorRamps` gives stops per field, which d3 interpolates linearly. Values outside the stops are extrapolated, and text values become grey `(120,120,120)`
  - Observed on the H5N1 tree: "HA Cell entry" uses a blue-grey-red ramp with a gradient legend from 1.27 to -5.82
  - The ramps were added for deep-mutational-scanning scores, whose high escape values were invisible as categories ([#603](https://github.com/theosanderson/taxonium/issues/603))

### Missing values and genotype colours

- **Six greys with different meanings**: "No metadata row" and the value `None` cannot be told apart. The six greys are:
  - Empty, `undefined`, or `unknown`: `(200,200,200)`
  - `None`: `(220,220,220)`
  - `N/A` and `NA`: `(180,180,180)`
  - Residue `X`: `(128,128,128)`
  - Unparseable ramp value: `(120,120,120)`
  - Treenome reference residue: `(245,245,245)`
- **Nucleotides use the amino-acid palette**: genotype colouring of `nt` positions colours A, C, G, T as alanine, cysteine, glycine, threonine, and `N` as asparagine yellow. Treenome draws the same bases in grey levels
- **Residue palette and colour-vision deficiency**: E and V become nearly identical under deuteranopia. E is a residue of the default genotype view S:484 (derived, simulated)

### Legend

- **Counts over the loaded set**: the legend counts values over the thinned tips of the query box (three times the view) plus all their ancestors [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useLayers.tsx#L23-L49)]
  - Internal nodes count
  - The order changes while panning
  - The counted set is neither the visible set nor the whole tree (observed: zooming to the F.1 lineage reordered the legend)
- **Top 10, no numbers**: entries are sorted by count and cut at 10 with "...". Then the empty value is removed, so often only 9 entries show. No counts or percentages are printed [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/components/Key.tsx#L133-L152)]
- **Wrong colours for numeric fields** (derived): legend entries are `String(value)` and go through the hash, while points get the number and the plasma scale. Every legend swatch is therefore wrong when no ramp is configured
- **Swatches are opaque**: points have opacity 0.6 over white
- **Click to recolour a value**: clicking an entry opens a colour picker for that value (observed). The choice is kept in memory only and is lost on reload. The minimap ignored such edits until [#741](https://github.com/theosanderson/taxonium/pull/741)
- **Collapsible**: the legend is collapsed by default below 800 px window width
- **No legend hover highlight**: hovering a legend entry only enlarges its dot. An earlier highlight of matching nodes filled the genotype cache with wrong values (see "Bug history")

## Zoom, pan, and minimap

### Zoom and pan

- **Independent x and y zoom**: the view state holds `zoom: [zoomX, zoomY]`. The mouse wheel zooms y only, four buttons zoom each axis by 0.6 (log2), and drag pans (observed) [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useView.tsx#L72-L186)]
- **Ctrl+wheel horizontal zoom is gone**: it existed from 2022 ([#231](https://github.com/theosanderson/taxonium/pull/231)) in a custom controller. It was deleted in the TypeScript and React 19 migration ([#700](https://github.com/theosanderson/taxonium/pull/700), 2025-05)
  - The documentation still describes it
  - Observed: Ctrl+wheel has no effect
  - The commented-out zoom-axis toggle still says "you can also hold Ctrl key"
- **Keyboard breaks the view** (observed, reproducible): after "Reset zoom", one Right-arrow press makes the view jump to about the default zoom and removes the minimap viewport frame. `+` and `-` do nothing. Mouse drag and wheel keep the zoom. The likely cause is the stock deck.gl controller working on the `[x, y]` zoom introduced in #700 (not confirmed)
- **No zoom or pan limits**: the view can be panned off the tree ([#439](https://github.com/theosanderson/taxonium/issues/439), an eLife reviewer request, open)
- **Click versus drag**: a click selects only if the pointer moved at most 10 px since pointer-down
- **Touch**: pinch zoom comes from deck.gl
  - In the old horizontal-zoom mode, pinch was disabled because it was "super confusing" ([#251](https://github.com/theosanderson/taxonium/issues/251))
  - Map-like trackpad gestures were declined as "the depths of Deck.GL" ([#250](https://github.com/theosanderson/taxonium/issues/250))

### Fitting

- **Fit modes per axis**: each axis has the mode "force", "if unmoved", or "skip". "Reset zoom" returns to the last fitted view [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/Taxonium.tsx#L214-L279)]
  - A new dataset refits both axes
  - Switching the x axis or Treenome refits only x
  - A window resize refits an axis only if the user has not moved it
- **Zoom to search results**: this action sets the y zoom to `9 - log2(span + 50000 / num_nodes)` and centres on the results, keeping the x zoom. The formula ignores the canvas size. The `zoomToSearch` URL parameter triggers it on load [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useSearch.tsx#L282-L340)]

### Minimap

The minimap is a second orthographic view in the top-right corner (20% of the width, 35% of the height). It shows the coarse whole tree and the search results [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useLayers.tsx#L461-L539)].

- **Shroud**: a polygon with a hole greys the area outside the main view
- **Navigation**: pressing or dragging on the minimap moves the main view there. The minimap itself never zooms
- **Treenome**: the minimap is hidden when Treenome is on

## Hover, selection, and node panel

- **Hover tooltip**: the tooltip shows three parts. It flips to the other side beyond 66% of the width or height. It never shows a branch length or a distance (observed) [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/components/NodeHoverTip.tsx#L75-L197)]
  - The tip name in bold, or "Internal node"
  - The fields of `keys_to_display`, with the colour-by value in its colour, and values longer than 100 characters cut
  - The mutations on the branch, filtered by type (amino acid by default)
- **Hovering a branch hovers its child node**, so long branches are easy targets
- **Stale tooltip** (observed): after the pointer left a branch, the tooltip stayed on screen through zoom, reset, and Treenome toggles
- **Click selects a node or branch** and opens a side-panel section (observed) with these parts:
  - The name or "Internal node" with the id, and a button to select the parent
  - All metadata and the "Number of descendants"
  - "Mutations at this node" with `aa` and `nt` check boxes. The root's list is hidden, because the root carries the pseudo-mutations
- **Clade actions** for an internal node (observed) [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/components/SearchPanel.tsx#L164-L256)]:
  - "List all tips" opens a text box with one attribute of every tip below (choice of attribute, no download)
  - "Download Nextstrain JSON" exports the subtree. It is offered below 1,000,000 tips and not for Newick input
  - "View clade in Nextstrain" links the server export below 20,000 tips
  - CoV-Spectrum links build a variant query from the node's genotype when configured
- **Inconsistent empty message** (derived): with only `nt` mutations and `nt` hidden, the tooltip says "No coding mutations" and the panel shows an empty list
- **Selection survives panning** ([#241](https://github.com/theosanderson/taxonium/issues/241)): in one observed case, the panel closed after several zoom-button clicks

## Search and highlighting

### Search methods

- **Several searches at once**: each search has its own row, check box, result count, colour, and zoom button. The URL stores the specification as JSON (`srch`, `enabled`). The URL update is debounced by 500 ms after a 300 ms input debounce
- **Methods**: there is no regex, no date range, and no "at least k of n" [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_data_handling/filtering.js#L228-L400)]. The available methods are:
  - Text contains, and exact text (the `x` check box)
  - A pasted list of names, one per line (the `m` check box, exact only)
  - Number comparison
  - Mutation (gene, position, residue or any, minimum descendants)
  - Genotype (inferred residue at a position) and revertant
  - Boolean AND, OR, and NOT with nested sub-searches
- **Permalink**: each search offers a link with `zoomToSearch=<index>`, shown only when the tree comes from a URL

### Result display

- **Ring display**: results get unfilled rings of radius `5 + 2i` px for search `i`, so rings of several searches nest. A toggle switches to filled points (observed). Colours cycle by list position through red, blue, green, magenta, cyan, yellow, so deleting a search recolours the later ones
- **Results above 10,000**: only results in the current y range are returned, thinned with the same grid as the tree. The client repeats the search for each new box. Rings and tip points are thinned independently, so ring density does not show hit density [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_data_handling/filtering.js#L401-L484)]

### Counting and matching defects

- **Counts are node counts**: both counts below are printed as "results"
  - A mutation search counts branches that carry the mutation (independent origins). It does not count tips that inherit the mutation
  - A genotype search counts tips and internal nodes, which is about twice the tip count on a binary tree
- **Text match is substring**: "F.1" also matches F.1.x and other names that contain the text (observed: 319 results)
- **Minimum descendants is off by one**: the label says "at least N", and the test is `num_tips > N`
- **Searching for 0 returns nothing** (derived): the empty-input test `spec.number == ""` is true for 0 in JavaScript

### Discoverability and history

- **Discoverability**: the `x` and `m` check boxes appear only for some fields. Users learned the multi-name mode from a maintainer of another project ([discussion #648](https://github.com/theosanderson/taxonium/discussions/648))
- **History**: blinking search results replaced the fading of non-matching points in 2021 ([#40](https://github.com/theosanderson/taxonium/issues/40)). Blinking is no longer in the code

## Treenome Browser and genome panel

Treenome draws the mutations of each clade beside the tree, aligned with the tree rows, under an embedded JBrowse 2 genome view. Observed on the Mpox tree: the tree takes 40% of the width, the minimap disappears, and coloured vertical bars fill the rest under a coordinate ruler.

- **One bar per mutation per branch**: a mutation on a branch is drawn as one vertical bar at its genome position, spanning the y range of the clade below the branch. The display is a summary by clade. It is not a matrix of tip genotypes [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/webworkers/treenomeWorker.js#L1-L139)]
- **Zoom sync**: the bars follow the tree's y zoom and y position. The genome range follows the JBrowse window
- **Single-tip mutations are invisible** (derived): a tip's span is one point, and the bar padding is 0.0001 world units, under a thousandth of a pixel
  - A tip's private mutations never appear in its row
  - A single reverted tip inside a clade bar looks like it carries the clade's mutation
- **Reversions to the root residue are drawn in the background colour**, so a reverted sub-clade shows as a gap ([#347](https://github.com/theosanderson/taxonium/pull/347)). Nucleotide reversions outside SARS-CoV-2 trees are not marked
- **Zoomed out**: with more than 600 visible bases, a bar is 2 px wide
  - On a 29,903-base genome in 900 px, positions 51 bases apart overlap
  - Bars spanning tiny clades are dropped when 10,000 or more nodes are loaded
  - The Treenome paper states that rare mutations appear only on zoom-in
- **Availability**: Treenome is available only when a root genome exists, so only for `taxoniumtools` output built with a GenBank file. SARS-CoV-2 trees get a built-in gene track and UCSC tracks. Other trees get the reference only, and users can add tracks
- **Reference versus root**: the paper's figure legend says that bars show mutations relative to the reference genome. The code compares with the tree root

## Settings and state

- **Settings dialog** with four tabs (observed):
  - "Toggle": minimap, labels for internal nodes, points for internal nodes, mutation types
  - "Appearance": label density 2.9, maximum clade labels 10, node size 3, node opacity 0.6, pretty stroke
  - "Search": display searches as points, point size 3
  - "Colour": default node colour, label colour, branch colour, clade label colour
- **Not persisted**: display settings and legend colour edits live in memory and reset on reload. Only `xType`, `color`, `srch`, `enabled`, `zoomToSearch`, `treenomeEnabled`, and `mutationTypesEnabled` go to the URL. A shared link therefore restores searches, colouring, and axis, but not the camera
- **Not configurable**: config cannot set display settings. The maintainer calls this "a mess", and a unification is in progress ([#594](https://github.com/theosanderson/taxonium/issues/594), [#773](https://github.com/theosanderson/taxonium/pull/773))
- **URL update contract**: the component calls `updateQuery` with partial objects. Two defects came from this contract:
  - The desktop app once passed React's `setState`, which replaced the whole state on each call, so zoom-to-search erased the searches ([#718](https://github.com/theosanderson/taxonium/issues/718))
  - Adding a search did nothing under router transitions until the update became one atomic call that reads `window.location` ([#820](https://github.com/theosanderson/taxonium/issues/820))
- **Long name lists break the URL**: multi-line searches are stored in the query string ([#533](https://github.com/theosanderson/taxonium/issues/533), open)
- **Malformed URL JSON** (derived): it is parsed without error handling and caught only by the global error boundary

## Input formats

### Tree formats

- **Taxonium JSONL** (optionally gzip): this is the only format with precomputed layout, mutations, clade labels, and time. `usher_to_taxonium` and `newick_to_taxonium` produce it. A single JSONL file launches at once
- **Newick** (browser): a vendored copy of Heng Li's jstreeview parser reads Newick. Branch lengths accept scientific notation [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/utils/processNewick.ts#L90-L172)]
  - `[&key=value,...]` comments become `meta_<key>` fields
  - Only the last bracket block per node is kept
  - NHX is not parsed ([#304](https://github.com/theosanderson/taxonium/issues/304))
  - Values stop at the first comma
  - Observed with a BEAST tree: `[&deme="USA"]` became a "Deme" colour field with the values `"USA"]`, `"New Zealand"]`, `"Hong Kong"]`
- **Quoted labels are not supported**: the tokenizer has no quote state, and all single quotes are then deleted from names. Observed with a double-quoted ARG file: labels kept their double quotes
- **Parse errors are silent**: the parser sets error flags that no caller reads, so malformed input renders as a partial tree
- **Nexus** (browser, "experimental"): regular expressions extract the first `tree` statement and the `Translate` block [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/utils/nexusToNewick.ts#L1-L46)]. Defects:
  - Translate keys also replace branch-length tokens, because every token without `:`, `,`, `(`, `)` is looked up
  - Translate lines must be split by exactly one space; tab-separated lines are dropped
  - With the `s` flag, the tree regex can start inside a comment that contains the word "tree" and span several trees
  - A tree with nothing between its last `)` and `;` gives an empty Newick
  - Observed: a 15-taxon file with a TRANSLATE block and several trees, and a 23-taxon file with a tab-separated TRANSLATE block and a comment containing "tree", both stayed on "Laying out the tree" with no error. A plain single-tree Nexus file without TRANSLATE loaded
- **Extended Newick (networks)**: not supported. Observed with an ARG file: each `#N` hybrid reference became an extra tip (34 "sequences" for 20 taxa)
- **Nextstrain JSON** (Auspice v2 only): the importer gives `x_dist` from `div`, `x_time` from `num_date`, branch mutations, and node attributes as fields [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/utils/processNextstrain.js#L237-L401)]. Limits:
  - `meta.colorings` is ignored, so Auspice colour scales are lost
  - Confidence intervals are dropped
  - The first leaf alone decides whether an axis exists
  - A root `div` of 0 is treated as missing, which shifts each root child subtree left (derived)
  - Parent links are keyed by node name, so duplicate names attach children to the wrong parent
- **Deep trees**: recursive traversals overflowed the stack on a 10,492-tip Newick tree and were made iterative ([#780](https://github.com/theosanderson/taxonium/issues/780)). The Nextstrain path is still recursive

### Metadata

- **Metadata CSV or TSV** (browser): the first column (or `taxonColumn`) is the key, matched exactly to the node name. No join statistics are shown [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/utils/processNewick.ts#L272-L395)]
  - CSV is split on every comma, with all double quotes deleted, so quoted fields with commas shift columns
  - CRLF line ends and a byte-order mark are not handled
  - Nodes without a row get empty values

### File detection and loading

- **File type by extension**: wrong or unknown types fail without a clear error
  - A Taxonium file named `.json` is parsed as Nextstrain and hangs ([#572](https://github.com/theosanderson/taxonium/issues/572))
  - A zip archive hangs at "Finalising" with no error ([discussion #703](https://github.com/theosanderson/taxonium/discussions/703))
  - A v1 `.pb` file redirects to a deleted deployment ([#830](https://github.com/theosanderson/taxonium/issues/830))
- **Loading by URL**: the parameters are `protoUrl` (JSONL), or `treeUrl`, `treeType` (`nwk`, `nexus`, `nextstrain`), `ladderizeTree`, `metaUrl`, `metaType`, and `taxonColumn`. URL inputs give a shareable link, and local files do not. A "Use Proxy" option routes URLs through a CORS proxy

## Export

- **PNG**: a copy of the WebGL canvas at screen resolution, with the minimap and without the legend, tooltip, buttons, or JBrowse panel. A legend in exports was declined as "pretty hard", because the legend is HTML and the tree is WebGL ([#626](https://github.com/theosanderson/taxonium/issues/626))
- **SVG**: Taxonium generates the SVG from the layer descriptions with the same thinned data. WebGL output is not used [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/utils/deckglToSvg.ts#L21-L183)]. Defects (derived):
  - Hidden searches are exported, because `visible` is ignored
  - Opacity is ignored
  - `alignment-baseline="center"` is not a valid SVG value, so labels shift about one row up in most renderers
  - Names are not XML-escaped
  - Nothing is clipped to the view
  - The selected and hovered rings get `stroke-width="undefined"`
- **Nextstrain subtree JSON**: divergence is the cumulative count of nucleotide mutations without gaps, and every metadata field becomes a categorical colouring [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_data_handling/exporting.js#L1-L138)]
  - The default colouring is always undefined (derived)
  - Only one of two `description` keys survives (derived)
  - Dates are exported as categories ([#381](https://github.com/theosanderson/taxonium/issues/381), open)
- **No Newick export** ([#328](https://github.com/theosanderson/taxonium/issues/328), open)
- **Tip list**: one attribute per line in a text box, with no download or copy button ([#431](https://github.com/theosanderson/taxonium/issues/431), open)

## Embedding and application shell

- **React component**: there is no iframe or `postMessage` interface. The component was designed for nearly full-screen use, and the minimap does not cover the tree in a smaller frame ([#734](https://github.com/theosanderson/taxonium/issues/734)). The properties are:
  - The data source (`sourceData`, `backendUrl`) and config
  - Query state with a partial-update callback
  - Title and overlay callbacks
  - `onNodeSelect`, `onNodeDetailsLoaded`, and `sidePanelHiddenByDefault`
- **One worker per page** (derived): the web worker and its callbacks are module-level, so two components with local data on one page would share one tree
- **Website**: the website has these parts:
  - A gallery of example trees
  - A browse page of 65 registered datasets plus hundreds of monthly "Viral UShER" trees
  - Drag and drop anywhere in the window, and a text-entry box
  - An "about" overlay with dataset provenance from the config (observed on H5N1)
  - Pages to build a tree, or to place sequences on an existing tree with UShER. The build page links its result with a search that rings the user's placed samples and zooms to them
- **Desktop app**: the app opens one file, starts the Node backend, and shows a URL so that the local tree can be opened in the web app. Four config keys that it passes (`showSidebar`, `showLegend`, `showMinimap`, `showTipLabels`) are read nowhere
- **Host-specific code**: special behavior exists for the domains `cov2tree.org`, `big-tree.ucsc.edu`, `epicov.org`, and `visualtreeoflife.taxonium.org`

## Scientific semantics

### Mutations on branches

- **Source of mutations**: mutations come from UShER mutation-annotated trees, through `usher_to_taxonium`, and from Nextstrain JSON, where Augur already translated them. Mutation search, genotype colouring, and Treenome therefore work only for these two sources
  - Newick input carries no mutations
  - `[&mutations=...]` comments become a text field only
  - A separate mutation input is requested ([#738](https://github.com/theosanderson/taxonium/issues/738))
- **UShER model**: UShER records substitutions only, with no indels and no ambiguity codes [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxoniumtools/src/taxoniumtools/ushertools.py#L389-L402)]
  - UShER resolved missing bases during placement, so a sample with `N` at a site shows its parent's base
  - Taxonium stores no missing-data mask and cannot show "unknown" for a tip
  - Only the first alternative of each UShER mutation record is read
- **Translation from a GenBank file** (`--genbank`, one record only): a codon map is built per CDS by walking the feature parts [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxoniumtools/src/taxoniumtools/ushertools.py#L102-L178)]
  - Joined features and ribosomal slippage work (the duplicated base of ORF1ab gets two codon slots)
  - Minus-strand genes are read in descending order and complemented
  - All nucleotide changes of one branch in one codon are applied together and translated once, so `GAT` to `GCC` gives one D to A change ([#88](https://github.com/theosanderson/taxonium/pull/88))
  - The codon context includes all ancestral changes
  - Synonymous changes give no amino-acid record
- **Overlapping genes**: one nucleotide change gives one amino-acid record per overlapping codon, so a change in ORF1a shows as both `ORF1a:L3829F` and `ORF1ab:L3829F`. A user reported this as a duplicate ([#597](https://github.com/theosanderson/taxonium/issues/597), closed as not planned)
- **Same gene name on two CDS features** (open, acknowledged by the maintainer in [#499](https://github.com/theosanderson/taxonium/issues/499)): the codon map is keyed by gene name, so a second CDS with the same name overwrites codon slots of the first. In the standard SARS-CoV-2 GenBank file, ORF1a and ORF1ab both carry `gene="ORF1ab"`. The public tree avoids this with a custom GenBank file [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxoniumtools/src/taxoniumtools/ushertools.py#L350-L372)]
- **Standard genetic code only**: no `transl_table` is read, so mitochondrial and Mycoplasma genes get wrong amino acids (derived). Ambiguous reference bases translate to `X` since 2026 ([#784](https://github.com/theosanderson/taxonium/pull/784)). Before that change, they crashed the conversion
- **Homoplasy**: the mutation table is deduplicated, so the same change on many branches is one entry, and a mutation search finds every independent origin. Pango designers use this to judge whether a mutation arose once or many times ([pango-designation#1503](https://github.com/cov-lineages/pango-designation/issues/1503))
- **Display default hides nucleotide changes**: `mutationTypesEnabled` defaults to amino acids only, so a branch with only synonymous changes shows "No coding mutations"

### Genotype, root, and reversions

- **Genotype of a node is inferred**: genotype colouring takes the residue of the nearest mutation at the position on the path to the root, else the root's pseudo-mutation, else `X`. Genotype search walks up in the same way. For files built with a GenBank file, it matches nodes that keep the root residue [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/hooks/useColorBy.tsx#L85-L117)] [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_data_handling/filtering.js#L534-L604)]
- **Root genome reconstruction**: the root sequence is rebuilt from the parent bases of all mutations, because the GenBank reference need not equal the tree root (mpox, [#226](https://github.com/theosanderson/taxonium/issues/226))
  - Shearing before reconstruction gave the root a wrong residue at S:501, so reconstruction now runs first ([#469](https://github.com/theosanderson/taxonium/issues/469))
  - The root's own UShER mutations (reference to root) are not used (derived), so a site changed only on the root branch keeps the reference base
- **The root appears in mutation searches**: its pseudo-mutations exist at every site, so a mutation search with residue "any" always returns the root ([#269](https://github.com/theosanderson/taxonium/issues/269), open). Nucleotide pseudo-mutations have parent residue `X`, so the root also passes the revertant filter (derived)
- **"Revertant" means back to the root state**: the paper and the Treenome figure say "reference". The two agree only when the root equals the reference
- **Reversions are not marked on the tree**: only the revertant search finds them. Tree builders ask to colour reverted branches, because a concentration of reversions is "usually a really bad sign" of a wrong placement ([#85](https://github.com/theosanderson/taxonium/issues/85), [#434](https://github.com/theosanderson/taxonium/issues/434), open)
- **Nextstrain imports have no root genome**: the reference state shows as `X` [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxonium_component/src/utils/processNextstrain.js#L270-L298)]
  - The importer also returns `rootId: 0` after renumbering nodes by y, so the topmost tip is treated as the root (derived)
  - That tip's own mutations vanish from colouring, hover, and Treenome, while search still sees them (derived)
- **Historic genotype bugs**: a refactor left the root without amino-acid records, so every unmutated node showed `X` in versions 2.0.108 to 2.0.110. Trees built with those versions stay broken until they are rebuilt ([#523](https://github.com/theosanderson/taxonium/issues/523), [#516](https://github.com/theosanderson/taxonium/pull/516))

### Condensed nodes and shearing

- **Identical sequences**: UShER condensed nodes are expanded into one zero-length sibling tip per sample, with no marker that they were identical. Collapsing identical sequences into one larger point was declined ([#11](https://github.com/theosanderson/taxonium/issues/11))
- **Shearing** (`--shear`, threshold 1000): at each node, a child with more than 1000 times fewer tips than its largest sibling is pruned with its whole subtree [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxoniumtools/src/taxoniumtools/ushertools.py#L263-L299)]
  - The help text says that such branches are usually sequencing errors "but it also could represent recombinants, or a real, unfit branch"
  - The public SARS-CoV-2 tree used a threshold of 5000. This removed all of lineage BA.3 and made it look absent and paraphyletic to Pango designers ([pango-designation#1087](https://github.com/cov-lineages/pango-designation/issues/1087))
  - The maintainer kept pruning because "increased interpretability to new end-users is worth it"
  - Protecting annotated clades is open ([#387](https://github.com/theosanderson/taxonium/issues/387))
- **Shear merge keeps wrong parent bases** (derived): when pruning leaves a parent with one child, the parent's nucleotide mutations move to the child only where the child has none at that site
  - A parent change A to G followed by a child change G to T stays recorded as G to T
  - A child reversion G to A stays as a spurious change
  - Amino-acid records are recomputed afterwards. Nucleotide records and the mutation-count branch length are not
- **No record of removed samples**: sheared samples are absent from search, counts, and lists

### Branch lengths and time

- **Units per input**: all units are rescaled per tree and stored without units (see "Coordinates")
  - UShER branch length is the number of nucleotide substitutions on the branch
  - Newick lengths are whatever the file holds
  - Nextstrain gives divergence and decimal years
  - Chronumental gives time
- **No axis, no scale bar, no distance in the tooltip**: users have requested these since 2021 ([#86](https://github.com/theosanderson/taxonium/issues/86), [#268](https://github.com/theosanderson/taxonium/issues/268)). A draft pull request is open since 2024 ([#612](https://github.com/theosanderson/taxonium/pull/612))
  - A y scale ("sequences per centimetre") was also requested ([#599](https://github.com/theosanderson/taxonium/issues/599))
  - A student asked whether there really is one dot per sequence. At 10 million sequences, there is one dot per 10 nanometres
- **Vertical order carries no meaning**: ladderizing an UShER tree puts large recent clades on top, so early and late cannot be read from y
- **Chronumental time trees**: `--chronumental` runs the external Chronumental program on the mutation-count tree with the metadata dates. Then it copies the time branch lengths back by walking both trees in pre-order, with no name check [[src](https://github.com/theosanderson/taxonium/blob/ae4bceba28723d0b122330a7a8e1491601a97e2f/taxoniumtools/src/taxoniumtools/utils.py#L47-L131)]
  - Chronumental anchors the dates on the earliest-dated sample unless a reference node is given. A single wrong early date "tends to go badly" ([#103](https://github.com/theosanderson/taxonium/issues/103))
  - Dates such as `1969-12-31` (Unix epoch placeholders) gave a negative clock rate ([#649](https://github.com/theosanderson/taxonium/issues/649))
  - Chronumental, and not Taxonium, places partial dates such as `2021-05` at the middle of the month with a matching uncertainty
- **Chronumental gotchas** (derived): Taxonium does not pin a Chronumental version
  - Leaving out `--chronumental_steps` passes `--steps None`, and Chronumental exits
  - The inferred-date table is always tab-separated, but Taxonium reads it with the separator of the metadata file, which fails for CSV
  - `--key_column` is not passed on, so Chronumental looks for its own name column
- **Nextstrain time axis**: `x_time` is relative to the root and rescaled, so calendar dates cannot be read from the picture (derived)
  - If the first leaf lacks `num_date`, the whole tree gets no time axis
  - If other tips lack it, their branches silently become length 0
- **Dates are text**: there is no date type, no date range filter ([#253](https://github.com/theosanderson/taxonium/issues/253), open), and no continuous colour by date

### Metadata joining

- **Exact match on the name**: `taxoniumtools` joins metadata to node names by exact match
  - The key column is cast to text, because integer IDs failed to join ([#600](https://github.com/theosanderson/taxonium/issues/600))
  - Duplicate keys are an error
  - A ragged TSV makes pandas take the first column as the index, so keys look missing or duplicated ([#576](https://github.com/theosanderson/taxonium/issues/576), open)
  - `newick_to_taxonium` deletes every space in the Newick text, so `A B` becomes `AB` and no longer matches the metadata
  - `--remove_after_pipe` cuts names at the first `|` after the join
- **Unmatched rows are invisible**: nodes without metadata are grey, and no count of unmatched names in either direction is shown. Users found join failures only by noticing grey nodes ([#10](https://github.com/theosanderson/taxonium/issues/10), [#467](https://github.com/theosanderson/taxonium/issues/467))

## Faithfulness of the picture

Taxonium is faithful where it uses geometry and unfaithful where it summarizes. These points are derived from the code with worked examples, and the colour points were also observed.

- **Faithful**:
  - Vertical extent of a clade is proportional to its tip count, also after thinning
  - Search results below 10,000 hits are all drawn, also outside the view
  - Precision: deck.gl 9 splits positions into high and low parts for Cartesian coordinates and works relative to the view centre. 10 million tips (2.4e-4 world units apart at y near 2400, about one float32 step) therefore do not jitter
- **Misleading without warning**:
  - A rare tip inside a dense row vanishes in zoomed-out views, together with its branch, hover target, and legend entry. Even a surviving rare point is drawn under about four overlapping points at opacity 0.6
  - The colour of a cell is the topmost tip's colour, so the ladderize direction decides the composition of mixed regions. Abundance does not
  - Point proportions follow the number of distinct root-to-tip distances instead of tip counts. A clonal expansion of 100,000 identical genomes keeps one point per row, while 1,000 diverse genomes keep up to about 20 per row
  - Legend order follows the loaded box including internal nodes
  - Numeric legends show wrong colours
  - Sibling lineages share colours
  - Search counts are node counts
  - Treenome hides private mutations and single-tip reversions
  - Long branches beyond the robust fit are off screen with no marker
- **Stated limitations in the Taxonium paper**:
  - Tip counts do not reflect outbreak size, because sampling density differs by country
  - Clades can be built from shared sequencing errors
  - Topology uncertainty is not communicated
  - The tool is for exploration and not for publication figures

## Bug history and how it was resolved

### Zoom model

From 2021 to 2025, deck.gl had one zoom value for both axes, and x was scaled separately through a model matrix.

- **Costs of the old model**: it needed hand-computed query bounds, disabled smooth x zoom, and made animated horizontal zoom impossible ([#265](https://github.com/theosanderson/taxonium/pull/265), open since 2022)
- **Reason**: the maintainer avoided deck.gl's native per-axis zoom because it "had low precision in the past in our hands"
- **Failed simplifications**: six simplification pull requests were closed unmerged in 2025 ([#668](https://github.com/theosanderson/taxonium/pull/668) and others)
- **Migration defect**: the migration to native `[x, y]` zoom in [#700](https://github.com/theosanderson/taxonium/pull/700) broke tip labels and zoom-to-search ([#706](https://github.com/theosanderson/taxonium/issues/706)). Code still computed `2 ** zoom` on what was now an array, hidden by `as number` casts, so `2 ** [0, -2]` gave NaN
- **Resolution**: the website was rolled back twice, then fixed in [#712](https://github.com/theosanderson/taxonium/pull/712). The desktop app began to bundle the component, so that a website deployment cannot break it
- **Side effect**: Ctrl+wheel x zoom was lost in the same change

### Other defects

- **Coordinate constants**: the fixed y ranges (2000, 2400) and x scaling leaked into a dozen constants (initial target `[1400, 1000]`, the minimap shroud ring of ±1e5 to 1e6, the zoom-to-search formula)
  - The minimap shroud was not a rectangle and showed a slanted edge on large screens. The fix only made the ring larger ([#582](https://github.com/theosanderson/taxonium/issues/582), [#584](https://github.com/theosanderson/taxonium/pull/584))
  - Only the initial view was derived from data extents and canvas size, in 2026 ([#833](https://github.com/theosanderson/taxonium/pull/833))
- **Numbers tested for truthiness**: there were at least six defects, among them:
  - `zoomToSearch=0` ignored ([#776](https://github.com/theosanderson/taxonium/issues/776))
  - Root `div: 0`
  - Mutation id 0 never deduplicated
  - Metadata value 0 turned into empty
  - Two bounds of exactly 0
- **Impure deck.gl accessors**: two caches gave wrong colours
  - A point-radius accessor added for legend-hover emphasis called the genotype function. This filled a per-node cache while parent nodes were not loaded yet, so parts of the tree turned grey depending on zoom. The maintainer found it by bisecting deployed previews and removed the feature ([#523](https://github.com/theosanderson/taxonium/issues/523), [#524](https://github.com/theosanderson/taxonium/pull/524))
  - A cache cleared in `useEffect` made the genotype legend show the previous position ([#462](https://github.com/theosanderson/taxonium/issues/462))
- **Missing update triggers**: colours did not update unless the data array changed identity. Minimap colours ignored legend edits until `colorHook` was added to the update triggers ([#739](https://github.com/theosanderson/taxonium/issues/739))
- **Three designs for bracket comments**: the first annotation parser found the owner of each block by text search and crashed on unnamed internal nodes ([#461](https://github.com/theosanderson/taxonium/issues/461)). The three designs were:
  1. Strip a literal `[&R]` prefix (2022, [#337](https://github.com/theosanderson/taxonium/issues/337))
  2. Delete every bracket block (2022, [#357](https://github.com/theosanderson/taxonium/pull/357))
  3. Keep blocks as metadata and restore the prefix rule (2023, [#497](https://github.com/theosanderson/taxonium/pull/497), [#503](https://github.com/theosanderson/taxonium/pull/503))
- **Newlines in names**: trees from the MAFFT web service had a newline after every node, so names ended with `\n` and no metadata matched. All newlines are now deleted before parsing ([#467](https://github.com/theosanderson/taxonium/issues/467)). The parser's whitespace test `c < "!" && c > "~"` can never be true, a bug inherited from jstreeview
- **Huge single lines**: a 212,000-node M. tuberculosis tree with amino-acid mutations had a 1.1 GB header line, beyond V8's string limit. The header is now parsed with a streaming JSON parser, and the mutations are sent in chunks ([#613](https://github.com/theosanderson/taxonium/issues/613)). A full streaming parser was too slow for SARS-CoV-2 trees and was not merged
- **Stale tree after navigation**: the worker answered queries from the previous tree while the next one downloaded ([#828](https://github.com/theosanderson/taxonium/issues/828)). The fix resets the loaded data. The filtering and search caches are module-level and survive (derived), so a second tree in the same page can reuse node ids and the children array of the first
- **Hook order**: the component returned early before its other hooks while the data source was unknown, which crashed on slow connections (React error 310, [#817](https://github.com/theosanderson/taxonium/issues/817))
- **Idle CPU of 8 to 15%**: traced to deck.gl's continuous redraw loop (upstream issue visgl/deck.gl#6961), open ([#432](https://github.com/theosanderson/taxonium/issues/432))

## Performance and scale

- **Browser memory caps tree size, not machine RAM**: the Chrome web worker stops at about 2 GB on macOS and 4 GB on Windows
  - A 5.85 million-node tree loads on an 8 GB laptop, and a tree above 10 million nodes does not ([#372](https://github.com/theosanderson/taxonium/issues/372))
  - The component warns above 6 million nodes and suggests the desktop app
  - Firefox has no such cap on macOS, which is why the Firefox warning was disabled
  - Safari discards memory-heavy background tabs, which users saw as random reloads ([#384](https://github.com/theosanderson/taxonium/issues/384))
- **Each node is a JavaScript object** with all metadata fields as properties, plus a mutation index map and a copy of all y values (derived). Load time of the public SARS-CoV-2 tree is about a minute. The maintainer names building the objects as the bottleneck, and decompression is not ([#596](https://github.com/theosanderson/taxonium/issues/596))
- **Viewport queries are O(N)** even when zoomed in, because ancestor collection ends with a filter over the whole tree. A narrow query on 4.2 million nodes takes 89 ms. Open work addresses this:
  - Fetch small selections by node id (0.46 ms, [#834](https://github.com/theosanderson/taxonium/pull/834))
  - Move binning and search into 64-bit WebAssembly ([#835](https://github.com/theosanderson/taxonium/pull/835))
- **Search is a linear scan** over all nodes, by design, to allow substring matches. An index was declined ([#410](https://github.com/theosanderson/taxonium/issues/410)). The descendant array is built on first use, so the first tip list on a large tree is slow ([#408](https://github.com/theosanderson/taxonium/issues/408))
- **Transfer by structured clone**: the worker returns arrays of node objects, and no transferable buffers are used (derived)

## Interaction with TreeKnit output

Observed by loading files through the `treeUrl` parameter:

- **Newick trees with polytomies and zero-length branches load and draw** (`data/h3n2-new-york-1999-2004/ha.nwk`, 154 tips). Splits added by resolution would have length 0 and would be invisible as horizontal branches, as in any zero-length case. Polytomies are not distinguishable from zero-length binary splits
- **Internal labels with support values** are shown as names. TreeKnit-style labels such as `98__iesFJ` are shown verbatim when internal labels are on
- **ARG output in extended Newick is not supported**: hybrid references become extra tips. A two-tree or network view does not exist. The only multi-tree request is several desktop windows ([#429](https://github.com/theosanderson/taxonium/issues/429)), and the maintainer prefers separate processes
- **MCC colouring** would need a metadata column per tip. MCC identifiers would get hash colours, with the sibling-name collisions described above, and the legend would show at most 9 or 10 MCCs
- **Two trees in one page** are not supported by the component (one worker, module-level caches)

## User stories

- **Pango lineage designation**: designers use Taxonium in several ways
  - They paste the full names of a proposal into the multi-name search and look for "a cloud of circles" ([auto-pango-designation#194](https://github.com/jmcbroome/auto-pango-designation/issues/194))
  - They colour by a site and ring branches with a change at that site to judge homoplasy
  - They ring representative sequences to decide the parent of a new branch ([pango-designation#3175](https://github.com/cov-lineages/pango-designation/issues/3175))
  - They read the order of mutations on a branch from colour plus rings
  - For close study, they export the subtree to Auspice: Taxonium is "the best if you must look at large trees, but when you want to study a potential new lineage, it's not got the features I love from Auspice" ([usher#340](https://github.com/yatisht/usher/issues/340))
- **Tree quality control**: the UShER tree builder removes sequences that attract wrong placements, "a bit of a game of whack-a-mole. Couldn't do it without taxonium" ([#76](https://github.com/theosanderson/taxonium/issues/76)). The builder also uses the revertant search to find runs of reversions that indicate sequencing errors
- **Surveillance dashboards** open Taxonium with an encoded search (lineage, mutation, country). Boolean searches were built for this ([#87](https://github.com/theosanderson/taxonium/issues/87))
- **Other pathogens and data**: users also apply Taxonium to these data:
  - H5N1 with deep-mutational-scanning scores, which led to colour ramps
  - M. tuberculosis (127,000 genomes): loads take 10 minutes on a 16 GB laptop, "long enough for a PI to get tired of waiting" ([#445](https://github.com/theosanderson/taxonium/issues/445))
  - Mpox, monthly viral trees built with UShER, and bacterial genomics
  - The NCBI taxonomy with Wikipedia thumbnails
  - Placement-uncertainty research with MAPLE ([#496](https://github.com/theosanderson/taxonium/issues/496))
- **Embedding**: in-browser analysis apps embed the component next to their own tools. Their requests (settings from config, re-rendering, sizing) form most of the open component issues

## Ideas from the issue tracker and history

### Open requests

- **Axes and scales**:
  - X axis and scale bar ([#86](https://github.com/theosanderson/taxonium/issues/86), [#268](https://github.com/theosanderson/taxonium/issues/268))
  - Y scale ([#599](https://github.com/theosanderson/taxonium/issues/599))
- **Mutations along paths**:
  - All mutations from the root to a tip, net of reversions ([#380](https://github.com/theosanderson/taxonium/issues/380), [#217](https://github.com/theosanderson/taxonium/issues/217))
  - Mutation labels on branches when there is room, filterable by gene ([#435](https://github.com/theosanderson/taxonium/issues/435))
  - Colour reverted branches ([#434](https://github.com/theosanderson/taxonium/issues/434))
  - Colour by the number of Spike-protein mutations ([#377](https://github.com/theosanderson/taxonium/issues/377))
- **Subtree summaries**:
  - Count of tips under a node that match a search ([#378](https://github.com/theosanderson/taxonium/issues/378))
  - Distribution of colourings below a node ([#426](https://github.com/theosanderson/taxonium/issues/426))
  - An Auspice-style entropy panel below a node ([#427](https://github.com/theosanderson/taxonium/issues/427))
  - Search for nodes that share the selected node's value, to check monophyly ([#593](https://github.com/theosanderson/taxonium/issues/593))
- **Uncertainty**:
  - Branch width or colour by a numeric property such as placement probability or support ([#496](https://github.com/theosanderson/taxonium/issues/496))
  - Bootstrap display ([#478](https://github.com/theosanderson/taxonium/issues/478))
- **More data beside the tree**:
  - Several metadata colour columns at once ([#477](https://github.com/theosanderson/taxonium/issues/477))
  - Heatmaps and gene-arrow plots aligned with tips ([discussion #476](https://github.com/theosanderson/taxonium/discussions/476)). The maintainer asked how these would look when zoomed far out
- **Filter, then colour**: hide tips that do not match a filter and colour the rest ([#778](https://github.com/theosanderson/taxonium/issues/778), [#825](https://github.com/theosanderson/taxonium/pull/825) open)
- **Sampling**:
  - Sparsify with a preference for undersampled countries ([#437](https://github.com/theosanderson/taxonium/issues/437))
  - Do not shear annotated clades ([#387](https://github.com/theosanderson/taxonium/issues/387))
- **Export**:
  - Legend in PNG and SVG ([#526](https://github.com/theosanderson/taxonium/issues/526), [#626](https://github.com/theosanderson/taxonium/issues/626))
  - Newick ([#328](https://github.com/theosanderson/taxonium/issues/328))
  - Dates as continuous values in the Auspice export ([#381](https://github.com/theosanderson/taxonium/issues/381))
  - Root-to-subtree mutations in the export ([#383](https://github.com/theosanderson/taxonium/issues/383))
- **Interaction**:
  - Metadata on hover over tip names ([#256](https://github.com/theosanderson/taxonium/issues/256), demo in [#261](https://github.com/theosanderson/taxonium/pull/261))
  - Scroll to pan ([#379](https://github.com/theosanderson/taxonium/issues/379))
  - Circle the selected node in the minimap ([#288](https://github.com/theosanderson/taxonium/issues/288))
  - Zoom to all searches ([#335](https://github.com/theosanderson/taxonium/issues/335))
  - Configurable legend length ([#508](https://github.com/theosanderson/taxonium/issues/508))
  - Readable labels on light colours ([#585](https://github.com/theosanderson/taxonium/issues/585))
  - A metadata field as tip label ([#742](https://github.com/theosanderson/taxonium/issues/742))

### In progress or parked

- **Map view**: a geographic map with a heat map of sample origins, with pie charts per country as the next step ([#536](https://github.com/theosanderson/taxonium/pull/536), open since 2023)
- **Label size under two-axis zoom** ([#710](https://github.com/theosanderson/taxonium/pull/710))
- **Minimap zoom fix** ([#737](https://github.com/theosanderson/taxonium/pull/737))
- **X axis draft** ([#612](https://github.com/theosanderson/taxonium/pull/612))
- **Hover highlight of a Treenome mutation** ([#342](https://github.com/theosanderson/taxonium/pull/342))
- **SNP-distance search**: samples within N SNPs of a node, with TSV download ([#592](https://github.com/theosanderson/taxonium/pull/592))

### Declined

- **Re-rooting**: positions are precomputed for 10-million-node trees ([#447](https://github.com/theosanderson/taxonium/issues/447))
- **Collapse identical sequences into larger points**: the maintainer answered "No" ([#11](https://github.com/theosanderson/taxonium/issues/11))
- **Load only a subtree**: this should be done with matUtils before conversion ([#372](https://github.com/theosanderson/taxonium/issues/372))
- **Map-like trackpad gestures**: declined as "the depths of Deck.GL" ([#250](https://github.com/theosanderson/taxonium/issues/250))
- **Find mutations that spread frequently**: this is out of scope, and the UShER team points to PyR0 ([#575](https://github.com/theosanderson/taxonium/issues/575))
- **Inference of ancestral traits**: lineage labels on internal branches should come from UShER. Fitch parsimony for ancestral states is an "ongoing hope" ([#598](https://github.com/theosanderson/taxonium/issues/598))

### Features found only in the history

- **Blinking search results**: added in 2021, replacing the fading of non-matching points
- **Ctrl, Alt, or Meta with the wheel for horizontal zoom**: available from 2022 to 2025
- **Hover over a legend entry to highlight matching nodes**: the genotype cache bug led to its removal
- **Branch colouring by the colour field**: the `colourLines` parameter is visible in 2021 URLs

## Open scientific problems

A user of Taxonium cannot answer these questions in the tool today:

- **How far apart are two nodes?** There is no axis, scale bar, or printed distance
- **Which mutations separate a tip from the root, net of reversions?** There is no path view, and reversions are only searchable
- **What is below a node?** There is no composition of metadata, matching-tip count, or mutation entropy for a subtree
- **Is a clade or a placement supported?** There are no support values or alternative placements, and topology uncertainty is not communicated
- **Is a lineage monophyletic, and is a rare clade really absent?** Shearing and thinning both remove small clades without a trace
- **Is the sample representative of the outbreak?** There is no downsampling or geography-aware sparsification, and tip counts follow sequencing effort
- **Does the result depend on the root?** Re-rooting is not available
- **Do two trees agree?** There is no comparison of trees, no linked view, and no tanglegram
- **What did the ancestors carry for a trait?** There is no ancestral-state inference for metadata
- **Is a tip's genotype observed or inferred?** Missing data is not stored, so inherited states look observed

## Not covered by Taxonium

- **Layouts**: radial, circular, or unrooted layouts; a cladogram as a user choice (only as a fallback)
- **Tree editing**: collapsing clades into triangles, re-rooting, rotating nodes
- **Branch styling**: branch colour or width by attribute, support values
- **Axes**: axes, scale bars, grid lines
- **Multiple trees**: networks and extended Newick, tanglegrams, and any second tree
- **Publication output**: legend in exports, publication styling
- **Persistence**: persisted display settings and colour edits
