# TreeViewer feature survey: tree rendering, semantics, and edge cases

This report describes TreeViewer, a cross-platform desktop program (C#, Avalonia) for drawing phylogenetic trees, as an idea inventory for the web app. TreeViewer builds every plot from small modules that run in a fixed pipeline, and it has a command-line version for trees too large for the interactive view. The report records what TreeViewer does, how it does it, which scientific conventions it assumes, which inputs break it, and which problems its maintainer left open. It does not compare TreeViewer with `packages/web`.

TreeViewer is licensed AGPL-3.0, and its tree library TreeNode is licensed GPL-3.0. Their behavior and design may be studied, but copying their code needs approval (see the project rules).

- **Source**: [arklumpus/TreeViewer](https://github.com/arklumpus/TreeViewer) at commit `e2ba62a` (2026-10-03), 412 commits since 2021-02
  - The latest program release is v2.2.0 (2023-10-26). Later changes ship only as module updates from the module repository
  - Source links point to commit `e2ba62a644a6e749304489ff0e03172c27a86e4e`
- **Tree library**: [arklumpus/TreeNode](https://github.com/arklumpus/TreeNode) at tag v1.5.4 (commit `928695b`, 2023-09-30), the NuGet version that TreeViewer pins
  - It holds the Newick, NEXUS, and binary parsers, rerooting, splits, and consensus
  - TreeNode HEAD (`7733e4b`, 2025-08-10) differs only where noted
- **Other sources**:
  - all 72 issues and the single pull request with their comments (discussions are off), all release notes, and the commit history
  - the wiki at commit `74605b4` (2025-04-10), in particular [Comparing trees](https://github.com/arklumpus/TreeViewer/wiki/Comparing-trees), [Working with large trees from the command-line interface](https://github.com/arklumpus/TreeViewer/wiki/Working-with-large-trees-from-the-command%E2%80%90line-interface), [Command-line interface](https://github.com/arklumpus/TreeViewer/wiki/Command%E2%80%90line-interface), and [Tree statistics](https://github.com/arklumpus/TreeViewer/wiki/Tree-statistics)
  - the paper [Bianchini & Sánchez-Baracaldo 2024](https://doi.org/10.1002/ece3.10873), read from Europe PMC (PMC10834882)
  - the module manuals, which are XML doc comments in each module source file
- **Evidence labels**:
  - "Observed" means seen in a trial run of the v2.2.0 Linux command-line program in a throwaway container, with the 102 modules of the module repository installed
  - "Derived" means read from the code without running it
  - Where the documentation and the code disagree, the code wins and the difference is noted
- **Shape of the code**: a .NET 7 desktop program
  - The core ([`src/TreeViewer/CoreClasses/`](https://github.com/arklumpus/TreeViewer/tree/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/CoreClasses)) runs the pipeline, renders through VectSharp and SkiaSharp, and handles selection and undo
  - All tree features live in 100 module files in [`src/Modules/`](https://github.com/arklumpus/TreeViewer/tree/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules). A module is C# source that the program downloads signed and compiles on the user's machine
  - The command-line program is [`src/TreeViewerCommandLine/`](https://github.com/arklumpus/TreeViewer/tree/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewerCommandLine)

## Summary

### Two trees and networks

- **One tree on screen**: a file with several trees is reduced to one tree (see "Two trees and comparisons")
  - The result is either a consensus (the default when the first two trees share their leaves) or a tree chosen by index
  - There is no side-by-side view, no tanglegram, and no link between trees
- **Tree comparison as attributes**: the "Compare trees" step marks each branch of the shown tree with whether its split is present in, or compatible with, another tree or a set of trees
  - The user then colours branches by this attribute
  - A statistics window gives the Robinson-Foulds distance of two trees and a tree-space map of many trees
- **No networks**: neither TreeViewer nor TreeNode gives `#` a meaning
  - A TreeKnit ARG loads as a tree with an extra tip per reassortment and with an invisible one-child node (Observed, see "Interaction with TreeKnit output")

### Scientific conventions

These conventions are described in "Scientific semantics".

- **Missing lengths count as 0**: a single missing length places the node on its parent. Only a tree without any length becomes a cladogram
- **Branch reading of support on reroot**: rerooting moves labels and attributes with their branch
  - Support values stay correct, while node values such as names shift by one node
- **A slash splits labels**: an unquoted `/` starts a support field, so an unquoted influenza strain name loses most of its text (Observed)
- **Ages are measured from the farthest tip**: axes, node bars, and computed ages all count time back from the tip farthest from the root

### Colour, scale, and architecture

- **Colour by attribute**: string attributes must hold CSS colours, and numeric attributes map onto 17 built-in gradients over a range the user types
  - There is no automatic categorical palette and no automatic legend for a gradient ([#57](https://github.com/arklumpus/TreeViewer/issues/57), open)
- **Large trees**: the command-line program plots trees with more than 100,000 tips
  - File loaders read huge files from disk lazily
  - The maintainer reports that the 107,235-tip GTDB tree works in the interactive view ([#49](https://github.com/arklumpus/TreeViewer/issues/49))
- **Architecture**: one tree flows through a fixed pipeline of nine module types (see "Module architecture")
  - Every user edit, including a click on "reroot here", becomes a stored pipeline step with parameters
  - The plot is therefore a reproducible recipe that can be saved, applied to another tree, or run from a script

## Module architecture

### Pipeline

The paper ([Figure 3](https://doi.org/10.1002/ece3.10873)) and the code describe one pipeline.

- **Module types**: the enum has nine values: `FileType`, `LoadFile`, `Transformer`, `FurtherTransformation`, `Coordinate`, `Plotting`, `SelectionAction`, `Action`, `MenuAction` [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/CoreClasses/Modules.cs#L1291)]
- **Data flow**: the modules run in this order [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/CoreClasses/Modules.cs#L1114-L1199)]
  1. File type module: parses the file
  2. Load file module: keeps the trees in memory or on disk
  3. One Transformer: many trees in, one tree out
  4. A list of Further transformations: edit the one tree in order
  5. One Coordinates module: a position for every node
  6. A list of Plot actions: draw in list order, so a module higher in the list draws behind the ones below it
- **Module signatures**: a Coordinates module returns `Dictionary<string, Point>` keyed by node ID [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/CoreClasses/Modules.cs#L1156-L1180)]
  - A Plot action receives the tree, its parameters, the coordinates, and a VectSharp `Graphics`, and returns its bounding box
- **Coordinate system markers**: each Coordinates module also stores a marker point under its own module ID [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Branches.cs#L179-L193)]
  - Plot actions read the marker to detect the layout (elbow, arc, or straight branches) and to get the units per branch length
  - Before v2.2.0 every plot action had a "Branch reference" parameter. Reviewers of the paper criticised it, and it was replaced by this detection ([`f407de6`](https://github.com/arklumpus/TreeViewer/commit/f407de6))

### Incremental recomputation

- **Cache per step**: before each Further transformation runs, the program stores a clone of the tree. A change to step `i` reruns only steps `i` to the end [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/CoreClasses/TreeDrawingLogic.cs#L301-L374)]
- **A failing step does not stop the chain**: an exception marks the step with a warning icon and a tooltip [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/CoreClasses/TreeDrawingLogic.cs#L391-L401)]
  - The next steps run on the tree as it was before the failing step
- **One layer per plot action**: each plot action renders into its own layer, and a parameter change redraws only that layer [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/CoreClasses/TreeDrawingLogic.cs#L622-L664)]
  - A background thread merges requests and limits redraws to 30 per second
- **Selection survives edits**: after a recomputation the program finds the selected node by ID, else by the last common ancestor (LCA) of the names below it [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/CoreClasses/TreeDrawingLogic.cs#L165-L189)]

### Nodes are addressed by leaf names

- **Node parameters**: a parameter that points at a node stores names, and the module finds the LCA of these names at run time [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Reroot.cs#L262-L273)]
  - The default value is the first and the last leaf name, which is the root
- **Unnamed nodes cannot be targeted**: a selection action refuses a node whose names do not identify it uniquely, with the message "The requested node cannot be uniquely identified!" [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Reroot_selection_action.cs#L111-L132)]
- **Consequence**: a stored step survives changes earlier in the pipeline and can be applied to another tree with the same leaf names
  - Renaming a leaf breaks every step that names it, and the error message says so

### Selection and interaction

- **Every drawn item carries a node ID**: the renderer tags each path and text with the ID of the node that drew it [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/CoreClasses/TreeDrawingLogic.cs#L758-L813)]
  - A click on any item (branch, label, shape, highlight) selects that node
- **Selection highlight**: the selected node is drawn in the selection colour (default `#237FFF`) and its descendants in a lighter tint [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/CoreClasses/SelectionLogic.cs#L63-L133)]
  - The descendant highlight stops after 500 ms on large clades
- **Selection actions become steps**: each action adds or updates a Further transformation or Plot action with the selected node as parameter
  - Tree edits: "Root tree on selection", "Collapse selection", "Cartoon selection", "Polytomise selection", "Prune selection", "Switch selection"
  - Display: "Highlight selection" (key `I`)
- **Undo and redo**: Ctrl+Z, Ctrl+Y, and Ctrl+Shift+Z restore whole pipeline states (v2.1.0, [#10](https://github.com/arklumpus/TreeViewer/issues/10))
- **Autosave**: every 10 minutes by default
  - The interface blocks while it saves, which freezes the program on large trees ([#70](https://github.com/arklumpus/TreeViewer/issues/70))

### Code in files and module signatures

- **Formatters are C# source**: every attribute-to-colour, attribute-to-number, and attribute-to-text conversion is a small C# function that the user can edit and that is stored in the tree file
  - The paper calls a tree file "essentially a small program" ([paper](https://doi.org/10.1002/ece3.10873), "Digital signatures")
- **Trust prompt**: files are signed with a key per user. Opening a file from another user shows a warning before its code compiles [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/Readme.md?plain=1#L98)]
- **Custom scripts**: Further transformation, Plot action, Action, and Coordinates modules each accept user C# code
  - The command-line `csc` command compiles and runs a C# file
- **Module repository**: modules are downloaded as signed source (RSA, SHA-512)
  - `TreeViewer -I` installs all modules headless, and an offline archive can be installed for machines behind a firewall ([#29](https://github.com/arklumpus/TreeViewer/issues/29), [#48](https://github.com/arklumpus/TreeViewer/issues/48))

## Module inventory

Each entry gives the module name, the main parameters with their defaults, and notable behavior. `n` is the number of leaves and `t` the longest root-to-tip length. Parameter lines are in the dump of each source file.

### File type and Load file modules

#### File types

- **Newick**: extensions `tree`, `tre`, `nwk`, `nwka`, `treefile`. Detection score 0.01 when the first non-blank character is `(` [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Newick_filetype.cs#L47-L62)]
- **NEXUS**: extension `nex`, score 0.5 when the file starts with `#NEXUS` [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Nexus_filetype.cs#L51-L76)]
  - It also reads TreeViewer's own `begin treeviewer;` block (saved modules) and `begin attachment;` blocks (base64 files)
- **Binary tree** (`.tbi`): TreeViewer's own format, score 0.8
  - It keeps all attributes, attachments, and modules, and it supports random access to single trees
- **NCBI ASN.1** (`.asn`, `.asnb`): text and binary, first tree only
- **RevBayes trace** (`.trees`): score 0.5 when the last tab column holds a tree. The other columns become numeric attributes
- **Stochastic map**: phytools `write.simmap` output, score 0.85 when an attribute looks like `{state,length:...}`

#### Load file modules

- **Memory loader**: score 0.5 up to the "Large file threshold" of 25 MiB (26214400 bytes) [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Memory_loader.cs#L68-L95)]
  - Above the threshold, a dialog asks which trees to skip, keep every k-th, or stop at
- **Compressed memory loader**: score 0.75 above 25 MiB. It re-encodes the trees in the binary format in memory [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Compressed_memory_loader.cs#L76-L99)]
- **Disk loader**: score 0.8 above the "Huge file threshold" of 1 GiB (1073741824 bytes) [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Disk_loader.cs#L79-L106)]
  - It converts other formats to a temporary binary file and reads trees from it lazily

### Transformer

- **Consensus** (the only Transformer): returns the single tree of a one-tree file. For several trees it computes a consensus or returns tree number `Tree #` (default 1) [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Consensus.cs#L68-L150)]
  - `Consensus tree`: default on when the leaf set of tree 1 contains that of tree 2 or the reverse
  - `Threshold`: default 0, which keeps all compatible splits (a greedy consensus). 0.5 gives majority rule and 1 strict consensus
  - `Branch lengths`: Median (default) or Mean of the matching branches
  - `Treat trees as clock-like`: default on when the first two trees are clock-like. Clock-like trees are aligned at the tips, others at the root
  - `Skip`, `Every`, `Until`: burn-in and thinning. `Every` defaults to `max(1, N / 100)`

### Further transformations

#### Rooting

- **Reroot tree**: moves the root to an outgroup or to the midpoint [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Reroot.cs#L179-L275)]
  - Parameters: `Rooting mode` Outgroup (default) or Mid-point, `Outgroup` node, `Position` on the outgroup branch 0.5
  - Midpoint rooting finds the two most distant tips with two farthest-tip searches and roots at half their distance
- **Unroot tree**: no parameters. It joins the two root children into a trifurcation

#### Clade display

- **Collapse node**: replaces a clade by two children that the Branches module draws as a triangle [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Collapse_node.cs#L178-L256)]
  - `Equalise lengths` (default on) gives both children the mean root-to-leaf depth, otherwise they get the shortest and the longest depth
  - `Fill colour` default `#F0F0F0`
- **Cartoon node**: keeps the clade but marks it for triangle drawing [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Cartoon_node.cs#L173-L219)]
  - `Equalise lengths` (default on) rescales the branch lengths inside the clade so that all leaves end at the mean depth

#### Topology edits

- **Polytomise node**: removes a node and attaches its children to its parent [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Polytomise_node.cs#L252-L396)]
  - The node's length is added to each child, so leaf depths stay the same
  - `Mode`: Single node or Attribute match (for example all nodes with `Support` smaller than a value)
  - `Apply recursively to all children`: off
- **Resolve polytomy**: groups two siblings of a polytomy under a new node. The new branch gets `min(l1, l2) * Position`, default 0.5 [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Resolve_polytomy.cs#L125-L175)]
- **Prune node**: removes the selected clade or keeps only the selected clade [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Prune_node.cs#L162-L197)]
  - Parameters: `Action` Prune selection or Keep only selection, `Mode` Single node or Attribute match, `Position` 0.5, `Leave one-child parent` off, `Keep pruned node names` off (stores names in `UnderlyingNodes`)
  - Position 0 removes the node and its branch, a larger value keeps a stub of that fraction
  - The selection action uses 0
- **Prune and regraft**: moves a subtree next to `New sibling` at `Position` 0.5 of the sibling's branch
- **Sort nodes**: `Order` Ascending (default) or Descending. Children are ordered by subtree depth in edges, then by the first leaf name [[src](https://github.com/arklumpus/TreeNode/blob/928695ba66085a8c96296e2e64c1e787892e4bf1/CSharp/TreeNode/TreeNode.cs#L921-L943)]
- **Switch children**: reverses the children of one node, optionally recursively
- **Subsample tree**: reduces the number of tips [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Subsample_tree.cs#L149-L250)]
  - `Type`: Relative (default, `Threshold` 0.5 of the leaves) or Absolute (default `n/2`), minimum 3
  - It repeatedly finds the closest pair of remaining tips and removes the one with the shorter terminal branch (`Criterion` Shortest branch) or the longer one

#### Branch lengths

- **Transform lengths**: `All equal` sets every length to 1. `Cladogram` sets all lengths to 1 and then stretches branches so that all tips align [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Transform_lengths.cs#L108-L144)]
- **Scale tree**: multiplies all lengths by `Scaling factor` 1
- **Compute node ages**: writes attribute `Age`. "Until tips" (default) is `t - depth`, "Since root" is the depth [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Compute_node_ages.cs#L136-L157)]

#### Attributes

- **Compare trees**: see "Two trees and comparisons"
- **Add attribute**, **Change attribute**, **Replace attribute**: set an attribute on a node, change an existing one, or replace values that match a string, regular expression, or numeric comparison
- **Add attribute from attachment**: reads a taxon list and marks the taxa
  - `Apply to`: Specified taxa, taxa and all ancestors, taxa and ancestors up to the LCA, LCA, or LCA and all children
- **Parse node states**: joins a table attachment to the tree
  - Defaults: `Separator` `\s` as a regular expression, `Match column` 1, `Match attribute` Name, new attribute `State`, type Auto
- **Propagate attribute**: copies values along the tree [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Propagate_attribute.cs#L122-L146)]
  - From tips to root: Average (default), Minimum, or Maximum of the children
  - From root to tips: Preserve (default), Divide equally, or Subtract
- **Linear transformation**: `value * a + b` on a numeric attribute, written to a new attribute
  - Users copy `Length` with it before a cladogram transform ([#25](https://github.com/arklumpus/TreeViewer/issues/25))
- **Add index**: writes the leaf order as attribute `Index`, starting at 1
- **Custom script**: user C# code that edits the tree

#### Age distributions and stochastic maps

- **Set up age distributions** and **Set up age distributions (attachment)**: collect node ages from all loaded trees or from a tree list, a table, or MCMCTree output
  - Defaults: `Compute mean` on, `Credible interval` Highest-density (default) or Equal-tailed, `Threshold` 0.89
- **Parse age distributions**: reads age samples stored in a node attribute
- **Set up stochastic map** and **Set up stochastic map (attachment)**: sample character histories along branches. `Resolution` 0.01 of the total tree length

### Coordinates

- **Rectangular**: the standard rooted layout [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Rectangular_coordinates.cs#L69-L201)]
  - Size: `Width` default `20 * t / (shortest positive length)`, `Height` `14 * n`
  - `Rotation` 0, with buttons for 0, 90, 180, 270 degrees
  - `Coordinate shift` None, Relative, or Absolute from node attributes, and a custom script that can move every node
- **Circular**: the rooted layout on a circle [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Circular_coordinates.cs#L54-L156)]
  - `Outer radius` `max(20 n / 2pi, 200)`, `Inner radius` at least 20, `Rotation` 0
  - `Sweep angle` 360 (range 1 to 360, for half fans, added in [`03f3700`](https://github.com/arklumpus/TreeViewer/commit/03f3700))
  - Shifts and script as in Rectangular
- **Radial** (unrooted): `Width` and `Height` `min(10000, 14 n)`, `Preserve aspect ratio` off, `Start angle` 0, `Sweep angle` 360 [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Radial_coordinates.cs#L86-L157)]

### Plot actions

#### Tree drawing

- **Branches**: draws the edges
  - `Root branch` on, `Rounding` 0, `Line cap` Round, `Auto colour by node` off
  - `Colour` from attribute `Color` (default black), `Line weight` from attribute `Thickness` (default 1)
- **Labels**: draws text from an attribute
  - `Attribute` Name, `Show on` Leaves, `Exclude cartoon nodes` on
  - `Anchor` Node, Mid-branch, Centre of leaves, or Origin, `Position` (5, 0), `Orientation` 0 relative to the Branch
  - Font Helvetica 10, optional background box and border
- **Node shapes**: `Show on` Leaves, `Shape` Polygon with 5 sides drawn as a star (or Circle), `Size` 8 from attribute `ShapeSize`, fill `#00A2E8`
- **Branch extensions**: dotted or plain lines from leaves to a common edge, for aligned labels

#### Clade marking

- **Highlight node**: a box (rectangular), wedge (circular), or convex hull (radial) around one clade. Fill `#80B9D9` or a gradient, margins 5
- **Highlights**: the same shapes for every node whose attribute `Highlight` holds a colour
- **Group labels**: bars with text beside clades, see "Labels and annotation"

#### Scale and legend

- **Scale axis**, **Scale bar**: see "Axes, time, and scale"
- **Legend**: hand-written Markdown, see "Colour and legend"

#### Character, age, and sequence data

- **Node states**: pie chart (default), rectangle, or wedge per node from a state attribute such as `{A:0.5,B:0.5}`
- **Stochastic mapping branches**: branch segments coloured by character state
  - `Style`: All states, Most probable states (`Dominance threshold` 0.6, `Exclusion threshold` 0.10), or Maximum a posteriori
- **Node bars**: a bar for a two-value range attribute such as a 95% HPD. `Show on` Internal nodes, `Thickness` 2, whiskers of size 10
- **Age distributions**: histogram or envelope (default) of node ages at each node, height 10
- **Age distributions timeline**: the same distributions stacked above or below the tree on a shared time axis
- **Plot alignment**: a FASTA alignment as a block beside the tree or as sequences at the nodes
  - A click on a sequence selects its tip, and the reverse

#### Free annotation and export

- **Text element**, **Rectangle**, **Draw image**, **Node images**: free annotations anchored to a node or to the origin
  - Images can be SVG, PDF, PNG, JPEG, and other formats from attachments
  - Node images (2025, [`86a5993`](https://github.com/arklumpus/TreeViewer/commit/86a5993)) take one image per node from an archive, keyed by node name
- **Crop region**: a named rectangle relative to a node that the export can use. Guides are on by default
- **Custom script**: user C# drawing code

### Actions, selection actions, and menu actions

#### Layout and style

- **Style buttons**: "Rooted tree style", "Circular tree style", and "Unrooted tree style" delete all plot actions and add Branches and Labels with default settings
  - They ask first when the plot was customised ([#3](https://github.com/arklumpus/TreeViewer/issues/3))
- **Reshape tree**: changes only the Coordinates module and keeps the plot actions [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Reshape_tree.cs#L41)]
  - Keys: Ctrl+Alt+R (rectangular), Ctrl+Alt+U (radial), Ctrl+Alt+Q (circular)
- **Branch score style**: sets up layered branches for a numeric score, see "Colour and legend"
- **Apply modules to other tree**: copies the whole pipeline (or only coordinates and plot actions, or only plot actions) to another tree file or to another open window [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Apply_modules_to_other_tree.cs#L59)]

#### Search, selection, and editing

- **Search** (Ctrl+F): finds nodes by attribute value (substring, regular expression, or greater/smaller than a number)
  - Matches are highlighted in `#FFFF98`
  - The user can copy the names or start "Replace attribute" on the matches
- **Lasso selection** (Ctrl+L): the user draws a polygon, and the program copies an attribute of the enclosed tips to the clipboard
- **Spreadsheet editor**: edits node attributes in a table
- **New tree**: a random tree, or a neighbour-joining or UPGMA tree from an alignment
- **Custom action script**: user C# code that acts on the window

#### Menu entries

- **Files**: Open (Ctrl+O), Open advanced (Ctrl+Shift+O, choose the File type and Load file modules), Paste tree (Ctrl+V), Save (Ctrl+S), Export (Ctrl+P)
- **Selection and editing**: Copy selected node (Ctrl+C), Select root node (Ctrl+A), Colour picker, Undo, Redo, Apply crop
- **Help**: Online manual (F1)

## Layouts

### Rectangular

- **Horizontal position**: the depth of the node (sum of lengths from the root) divided by `t` and multiplied by the width [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Rectangular_coordinates.cs#L350-L387)]
- **Vertical position**: leaves are evenly spaced at `(index + 0.5) / n` [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Rectangular_coordinates.cs#L405-L432)]
  - An internal node takes the **midpoint of its first and last child**, so in a polytomy the middle children do not pull the parent
- **Cartoon clades**: the root of a cartooned clade takes the midpoint of its first and last leaf instead
- **Default aspect ratio**: the default width is clamped so that width over height stays at most 4:3 (global setting "Maximum default aspect ratio", 1.333) [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Rectangular_coordinates.cs#L69-L100)]
  - The parameter manual says the ratio stays between 9:16 and 16:9 [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Rectangular_coordinates.cs#L145)]
  - Observed: a 50-tip TreeKnit fixture gets 933 by 700
- **Rotation**: any angle, applied to all node coordinates
- **Root stub**: the root's own length, or `0.1 * t` when the root has no length [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Rectangular_coordinates.cs#L455)]

### Circular

- **Polar mapping**: radius `r = depth / t * (outer - inner) + inner`, angle `theta = y * sweep + rotation`, with the same vertical rule as Rectangular [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Circular_coordinates.cs#L392-L393)]
- **Gap**: with a 360 degree sweep, the first and the last leaf are one leaf spacing apart
- **Root length**: the root's own length is added to every node, which differs from Rectangular
- **Branches**: an arc around the centre at the parent's radius, then a radial line to the child

### Radial (unrooted)

- **Equal-angle algorithm**: ported from FigTree's `RadialTreeLayout.java`, as the manual states [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Radial_coordinates.cs#L294-L356)]
  - Each child gets a wedge of its parent's angle in proportion to its number of leaves
  - The node lies at its length along the wedge centre
- **No daylight step**: there is no iterative optimisation that spreads clades apart, so large clades can overlap
- **Fit to box**: the layout is scaled to the width and the height separately unless `Preserve aspect ratio` is on [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Radial_coordinates.cs#L363-L390)]
  - This distorts branch lengths by default
- **Automatic choice**: a tree whose root has three or more children counts as unrooted, and the program suggests Radial for it (Observed)

### Custom placement

- **Coordinate shift**: Relative adds node attributes X and Y to the computed position, Absolute replaces the position [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Rectangular_coordinates.cs#L459-L506)]
  - The module picks attributes named like `X` and `Y` automatically
- **Coordinate script**: a C# method `GetCoordinates(TreeNode tree, ref Dictionary<string, Point> coordinates)` can rewrite every position
  - The maintainer used it to give several phylograms the same scale ([#41](https://github.com/arklumpus/TreeViewer/issues/41)) and to align highlight boxes ([#32](https://github.com/arklumpus/TreeViewer/issues/32))

## Branches and nodes

- **One path per edge**: each edge runs from the parent to a corner and then to the child. There is no separate vertical connector per parent [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Branches.cs#L294-L320)]
- **Several Branches modules**: a plot can hold any number of Branches modules with different attributes, colours, and widths
  - The "Branch score style" and early clade highlighting ([#5](https://github.com/arklumpus/TreeViewer/issues/5)) use this
- **Cartoon triangle**: a polygon from the clade root through every leaf of the clade in order, filled with the colour stored by Collapse or Cartoon node [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Branches.cs#L447-L475)]
- **Root warning**: the Branches module warns when it draws a root branch on a tree whose root has more than two children, or in the Radial layout [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Branches.cs#L483-L523)]
- **Invisible edges**: an edge with width 0 or a transparent colour is not drawn [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Branches.cs#L281)]
- **Rounded corners**: `Rounding` above 0 replaces the corner by a cubic Bezier curve [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Branches.cs#L340-L350)]

## Labels and annotation

### Labels

- **Label anchors**: Node, Mid-branch, Centre of leaves, or Origin [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Labels.cs#L791-L818)]
  - Origin projects the node onto the line through the root perpendicular to the growth direction
  - This gives labels aligned at the root in rectangular trees and at the centre in circular trees
- **No overlap avoidance**: every label is drawn, with no thinning at small sizes [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/Readme.md?plain=1#L114)]
  - The maintainer advises removing tip labels for large trees, also because the Linux interface draws text as paths and uses much memory
- **Labels on collapsed clades**: a second Labels module with `Exclude cartoon nodes` off and anchor Centre of leaves ([#28](https://github.com/arklumpus/TreeViewer/issues/28))
- **Readable angles**: text whose angle is more than 90 degrees from horizontal is turned by 180 degrees [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Labels.cs#L878-L912)]
- **Direction on zero-length branches**: for a mid-branch label, the module walks up through ancestors at the same position to find a direction [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Labels.cs#L628-L632)]
- **Rich text**: labels accept inline `<i>`, `<b>`, and `<#RRGGBB>` tags ([#22](https://github.com/arklumpus/TreeViewer/issues/22))
  - Line breaks are not supported ([#7](https://github.com/arklumpus/TreeViewer/issues/7))

### Group labels

- **Purpose**: a bar with text beside each clade that has a value of an attribute [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Group_labels.cs#L117-L256)]
- **Defaults**:
  - distance 100 (or per node through `LabelDistance`), height 20
  - `Prevent overlap` on, `Gravity` Bottom
  - `Overflow` Expand label, Clip, or Compress text
- **Row packing**: labels are sorted by start and placed greedily in the first row where they fit [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Group_labels.cs#L1531-L1625)]
  - `Preserve nesting` keeps a descendant's label in a row at or above its ancestor's row
- **Only on last ancestor**: draws a label only where the value differs from the parent's value, which pairs with "Propagate attribute"
- **History**: gravity, inverted arrangement, overflow, nesting, and manual rows were added in 2024 after a report of stacking problems ([#46](https://github.com/arklumpus/TreeViewer/issues/46))

### Highlights

- **Highlights in three layouts**: a rectangle in rectangular trees, an annular wedge in circular trees, and a convex hull with margin in radial trees [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Highlight_node.cs#L509-L641)]
  - Gradients go from root to leaves or from leaf to leaf, with the midpoint colour interpolated in CIELAB
  - In radial trees the gradient axis comes from a principal component analysis of the hull points

## Colour and legend

### Formatters

- **String to colour**: the default converter reads the attribute as a CSS colour (`red`, `#ff0000`) [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/CoreClasses/Modules.cs#L399-L409)]
  - Any other string gives no colour, and the default colour is used
- **No categorical palette**: to colour by arbitrary category names (for example a segment or an MCC index), the user writes a C# `switch` in the formatter or replaces values by colours with "Replace attribute"
- **Auto colour by node**: an option of most plot actions. It hashes the names of all nodes in the subtree and picks one of 9 fixed colours [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/CoreClasses/Modules.cs#L493-L516)]
  - The colours stay the same across runs since v2.2.0 ([`0b9f4b5`](https://github.com/arklumpus/TreeViewer/commit/0b9f4b5))
  - The hash depends on the order of the names and includes internal names, so the same clade in two trees with a different child order can get different colours
  - With 9 colours, collisions are frequent
- **Number to colour**: a linear map from a range the user types (default 0 to 1) onto a gradient (default transparent to black), clamped at both ends [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/CoreClasses/Modules.cs#L410-L431)]
  - The range does not come from the data
- **Gradients**: 17 presets, and a gradient editor changes the stops [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/CoreClasses/Modules.cs#L518-L551)]
  - Presets: `TransparentToBlack`, `WhiteToBlack`, `RedToGreen`, `Rainbow`, `Viridis`, `RedYellowGreen`, `WongRainbow`, `WongDiscrete`, `Muted`, `MutedDiscrete`, `Magma`, `Inferno`, `Plasma`, `Cividis`, `Rocket`, `Mako`, `Turbo`
- **Default colour names**: the default colour attribute is `Color` (US spelling)
  - An attachment with a `Colour` column silently does nothing ([#67](https://github.com/arklumpus/TreeViewer/issues/67))

### Legend

- **Hand-written Markdown**: the Legend module renders Markdown with image URLs for symbols [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Legend.cs#L41-L130)]
  - Symbol URLs: `circle://size,colour`, `square://`, `rect://`, `ellipse://`, `poly://`, `star://`, and `attachment://name`
  - The default text shows three symbols in Wong colours `#CC79A7`, `#0072B2`, `#009E73` [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Legend.cs#L205)]
- **No automatic legend**: a colour formatter or gradient does not produce a legend ([#57](https://github.com/arklumpus/TreeViewer/issues/57))
  - A continuous gradient legend was requested in 2025 and is still open
  - The maintainer suggests drawing it elsewhere and adding it as an image
- **Two generated legends**: "Branch score style" builds an SVG gradient bar, and "Stochastic mapping branches" has an "Add legend" button

### Branch score style

- **Purpose**: shows a numeric score such as a BLAST score on branches [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Branch_scores.cs#L378-L402)]
  - A dialog asks for the attribute, its minimum and maximum, the number of layers (default 10, minimum 2), and a gradient (default Viridis)
- **Layered drawing**: it adds one Branches module per layer, each with a step gradient and a smaller width [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Branch_scores.cs#L730-L781)]
  - A branch shows every layer whose threshold lies below its score
- **Cost**: each branch is drawn about 10 times ([#49](https://github.com/arklumpus/TreeViewer/issues/49))
  - A 120,000-tip tree took more than 30 minutes to export as PDF
  - The fix was a single Branches module with a gradient

## Axes, time, and scale

### Scale axis

- **Layouts**: works in Rectangular and Circular coordinates only. Radial gives the error "The coordinates module is not supported!" [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Scale_axis.cs#L255-L265)]
- **Defaults** [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Scale_axis.cs#L96-L220)]:
  - top and bottom axes on
  - `Reverse axes` on: the farthest tip is at 0 and values grow toward the root
  - `Negative ages` off, `Offset` 0
  - `Tick spacing` `0.05 t`, `Start` 0, `End` `t`, `Labels every` 2 ticks, `Digits` 2
  - `Grid type` Shading
- **Tick labels**: `start + i * spacing * sign + offset`, with fixed spacing and no rounding to steps of 1, 2, or 5 [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Scale_axis.cs#L487)]
- **Calendar dates**: `Offset` and `Negative ages` turn ages into dates ([`ebb8b6a`](https://github.com/arklumpus/TreeViewer/commit/ebb8b6a), [#17](https://github.com/arklumpus/TreeViewer/issues/17))
  - For example, offset 2020 with negative ages gives a forward calendar axis
- **Circular trees**: the axis runs along a radius. Left-only or right-only axes were added at HEAD ([`e2ba62a`](https://github.com/arklumpus/TreeViewer/commit/e2ba62a))

### Other time and scale elements

- **Scale bar**: length `round(0.2 t, 2 digits)`, centred 15 units below the tree. It works in all three layouts [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Scale_bar.cs#L196-L216)]
- **Node bars**: a bar for a two-element range attribute at age `t - depth`, in Rectangular and Circular coordinates only [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Node_bars.cs#L345-L348)]
- **Age distributions**: the full distribution of a node's age across a tree sample, as a violin-like envelope or histogram at the node
  - By default it shows the mean and an 89% highest-density interval (the paper follows McElreath for 89%)

## Two trees and comparisons

TreeViewer shows one tree at a time. It compares trees through attributes and reports.

### Compare trees (Further transformation)

- **What it computes**: for every branch of the shown tree, whether its split is present in the other tree and whether it is compatible with it. The results go into the attributes `<prefix>Present` and `<prefix>Compatible` [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Compare_trees.cs#L239-L299)]
  - **Other tree**: a loaded tree by index (prefix `Tree<i>_`) or an attachment (prefix `<name>_`)
  - **One other tree**: the values are the strings `Yes` and `No`
  - **Several other trees**: the values are the fractions of trees in which the split is present or compatible, computed in parallel [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Compare_trees.cs#L301-L394)]
- **Attribute transfer**: with `Store attributes on equivalent splits` (default on) and one other tree, all attributes of the matching branch are copied with the prefix
  - Examples: `Tree415_Length`, `Tree415_Support`
- **Rooted splits**: when a tree is rooted, every split gets a pseudo-leaf `@Root` on the side away from the node, so a split becomes a clade [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Compare_trees.cs#L198-L203)] [[src](https://github.com/arklumpus/TreeNode/blob/928695ba66085a8c96296e2e64c1e787892e4bf1/CSharp/TreeNode/TreeNode.cs#L1158-L1189)]
- **Different leaf sets**: compatibility is tested on splits restricted to the shared leaves. Presence is tested on the full splits
- **Compatibility test**: two splits are compatible when at least one of the four intersections of their sides is empty [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Compare_trees.cs#L481-L534)]
- **Display recipe** (wiki, "Comparing trees"): search for `No` in `Tree415_Present`, or colour or thicken branches by the attribute
  - The wiki notes that compatibility with a gene-tree sample resembles the gene concordance factor of IQ-TREE, but counts non-decisive trees as compatible

### Statistics window

- **Two trees**: the report prunes both trees to their shared leaves [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/Stats/TwoTreesReport.cs#L122-L176)] [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/Stats/Comparisons.cs#L306-L460)]
  - **Distances**: the Robinson-Foulds distance, the weighted Robinson-Foulds distance (sum of the lengths of unmatched splits), and the numbers of shared and unique splits
  - **Same topology**: the report gives the edge-length distance, the square root of the summed squared length differences of matched splits
  - **Split length differences**: a histogram with the mean and an 89% highest-density interval
  - **Tree shape**: Sackin and Colless indices and cherry counts of both trees, each tested against simulated Yule (YHK) and uniform (PDA) null distributions
- **Many trees**: the report analyses a whole tree sample [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/Stats/MultipleTreesReport.cs#L280-L347)] [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/Stats/PointClustering.cs#L475)]
  - a distance matrix (RF, weighted RF, or edge length)
  - a two-dimensional multidimensional scaling (MDS) map
  - a Duda-Hart test for clustering, and K-medoids with at most 12 clusters
  - A click on a cluster opens its trees in a new window
- **Command line**: `distances RF|wRF|EL <file>` writes the matrix for the loaded trees

### Side-by-side work

- **Separate windows**: "Apply modules to other tree" styles a second tree like the first, in its own window. There is no shared canvas and no links between leaves
- **Shared scale**: two plots get the same scale only through a coordinate script that rescales by the scale marker ([#41](https://github.com/arklumpus/TreeViewer/issues/41), open)

## Networks and reticulations

- **No support**: no file of TreeViewer or TreeNode treats `#` or hybrid nodes specially
  - A search of both repositories for `#H`, hybrid, network, reticulation, tanglegram, and cophylogeny finds nothing in the code
- **Extended Newick in the docs**: the TreeNode format notes state that Extended Newick files should parse but leave the meaning to the consumer
  - The command-line `enwk` command ("Extended Newick") writes Newick with attributes, without hybrid nodes [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewerCommandLine/Commands/EnwkCommand.cs#L292-L301)]
- **Issue tracker**: no issue asks for networks, tanglegrams, or ARGs

## Input parsing

The parsers are in TreeNode v1.5.4.

### Newick tokens

- **Several trees per line**: a tree ends at a `;` outside quotes [[src](https://github.com/arklumpus/TreeNode/blob/928695ba66085a8c96296e2e64c1e787892e4bf1/CSharp/TreeNode/NWKA.cs#L285-L345)]
  - Text before the first `(` becomes the attribute `TreeName`
  - A tree that fails to parse ends the file silently
- **Field separators**: `:`, `/`, and `,` outside brackets end a field [[src](https://github.com/arklumpus/TreeNode/blob/928695ba66085a8c96296e2e64c1e787892e4bf1/CSharp/TreeNode/NWKA.cs#L442)]
  - `:x` sets `Length`. A second `:y` becomes `Length2`, and so on
  - `/x` sets `Support`, then `Support2`. A non-numeric value becomes `Unknown`, then `Unknown2` [[src](https://github.com/arklumpus/TreeNode/blob/928695ba66085a8c96296e2e64c1e787892e4bf1/CSharp/TreeNode/NWKA.cs#L499-L563)]
  - Observed: unquoted `A/Texas/50/2012` becomes the name `A`, `Unknown=Texas`, `Support=50`, `Support2=2012`
  - TreeNode HEAD adds an option to turn the slash rule off ([`0a76dfa`](https://github.com/arklumpus/TreeNode/commit/0a76dfa), 2025). TreeViewer still pins v1.5.4
  - Multi-value support such as `99/1.0/99` therefore becomes three attributes ([#73](https://github.com/arklumpus/TreeViewer/issues/73), open)
- **Name or support**: a bare token is a name unless it starts with a digit and parses as a number, then it is support [[src](https://github.com/arklumpus/TreeNode/blob/928695ba66085a8c96296e2e64c1e787892e4bf1/CSharp/TreeNode/NWKA.cs#L570-L625)]
  - A leaf always takes its first token as its name, so leaves such as `123` or `37_0` keep their names (Observed)
- **Quotes**: `'` and `"` both quote, and `\` escapes the next character [[src](https://github.com/arklumpus/TreeNode/blob/928695ba66085a8c96296e2e64c1e787892e4bf1/CSharp/TreeNode/Extensions.cs#L333-L411)]

### Annotations and attributes

- **Annotations**: `[&key=value,...]` blocks, with a leading `&` or `!` removed [[src](https://github.com/arklumpus/TreeNode/blob/928695ba66085a8c96296e2e64c1e787892e4bf1/CSharp/TreeNode/NWKA.cs#L455-L497)]
  - FigTree `!color=#ff0000` becomes the string attribute `color` (Observed)
  - A value is a number when it parses as one, otherwise a string
  - Vectors such as `{0,1}` stay strings
- **Duplicate keys get suffixes**: a TreeAnnotator file with both `length=` and `:length` stores the second as `Length2` ([#20](https://github.com/arklumpus/TreeViewer/issues/20), [#66](https://github.com/arklumpus/TreeViewer/issues/66))
  - TreeViewer then plots the mean length and misaligns an ultrametric tree
- **MrBayes support**: an attribute `prob` becomes the support when no support was read [[src](https://github.com/arklumpus/TreeNode/blob/928695ba66085a8c96296e2e64c1e787892e4bf1/CSharp/TreeNode/NWKA.cs#L708-L711)]

### NEXUS and unsupported formats

- **NEXUS**: only `trees` blocks are read, with `tree` and `utree` [[src](https://github.com/arklumpus/TreeNode/blob/928695ba66085a8c96296e2e64c1e787892e4bf1/CSharp/TreeNode/NEXUS.cs#L350-L356)]
  - The translate table renames every node with a non-empty name
  - Numeric internal labels are already support values, so they are not renamed
- **Unsupported**: MEGA `.mtsx` ([#9](https://github.com/arklumpus/TreeViewer/issues/9)) and Newick files with a PHYLIP-style count line ([#48](https://github.com/arklumpus/TreeViewer/issues/48))

## Export and saving

- **Export dialog** (Ctrl+P): PDF, SVG, and PNG or TIFF [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Export.cs#L214-L219)]
  - **PDF**: page size from 29 presets (ISO A0 to A8, B0 to B10, Letter, Legal, Ledger, Executive, ANSI C to E) or custom, orientation, and margins [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Export.cs#L366-L392)]
  - **SVG text**: Embed full font, Embed subsetted font, Convert into paths, or Convert into paths using glyphs (the default, which keeps files small but some editors do not support it) [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Export.cs#L1129-L1131)]. Observed: a command-line SVG contains no `<text>` elements
  - **PNG and TIFF**: width and height in pixels or physical units with a resolution in dpi. The dialog recommends raster output for very large trees, because the file size depends on the resolution only
  - **Crop region**: the entire plot or a named crop region
- **Tree files**: the loaded trees, the transformed tree, or the final tree, in one of these formats [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Save_tree.cs#L186-L212)]
  - binary (`.tbi`) or NEXUS: all attributes, attachments, and modules
  - Newick: basic attributes
  - Newick with attributes (`.nwka`): all attributes
  - NCBI ASN.1: one tree
- **Newick output**: every name in single quotes, attributes as `[key=value]` after the length
  - Observed: leaves get `Support=NaN`

## Command-line program

### Usage

- **A command loop on standard input**: `TreeViewerCommandLine` ignores its arguments and reads commands interactively, or from a redirected file as a script [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewerCommandLine/Program.cs#L118-L146)] [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewerCommandLine/Program.cs#L264)]
- **Commands**: `attachment`, `background`, `binary`, `csc`, `distances`, `enwk`, `exit`, `file`, `help`, `load`, `module`, `newick`, `nexus`, `node`, `open`, `option`, `pdf`, `png`, `region`, `resolution`, `svg`, `system`, `tiff`, `update`
- **Workflow** (wiki tutorial with a 21,235-tip tree): changes take effect only after `update`
  1. `open file`, then accept the suggested modules with `y`
  2. `module enable Labels`, `module select #i`
  3. `option select`, `option set`
  4. `update`
  5. `pdf out.pdf`
- **Colour**: honours the `NO_COLOR` environment variable

### Pitfalls and tips

- **Bare plot after load** (Observed): `load` enables only the Transformer and the Coordinates module
  - Without `module enable Branches`, the SVG is an empty 1 by 1 page
- **Modules required**: the command line has no modules until `TreeViewer -I` installs them, which needs the network (Observed)
- **Large-tree trick** (wiki, large trees tutorial): plot without labels in the command line, save as `.tbi`, and open that file in the interface
  - The interface then does not try to draw tip labels

## Scientific semantics

### Branch lengths and time

- **Missing lengths count as 0** (Derived and Observed): a node without a length is placed at its parent's position [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Rectangular_coordinates.cs#L355-L384)]
  - Only when every length is missing does each edge count as 1
  - Observed with `((A:1,B):1,C:2)`: B sits on its parent with no warning
- **Layouts disagree on missing lengths**: the Radial layout counts a missing length as 1 [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Radial_coordinates.cs#L302)]
- **Negative lengths**: accepted without warning, so the node is drawn behind its parent
- **Time from the farthest tip**: "Compute node ages", the Scale axis, and Node bars measure age as `t - depth`
  - In a serially sampled tree only the farthest tip has age 0, and the other tips have positive ages
- **Clock-like test**: tips within a relative tolerance of 0.001 of each other count as contemporaneous [[src](https://github.com/arklumpus/TreeNode/blob/928695ba66085a8c96296e2e64c1e787892e4bf1/CSharp/TreeNode/TreeNode.cs#L1000)]

### Rooting and support values

- **Rooted test**: a tree counts as rooted when its root has fewer than three children [[src](https://github.com/arklumpus/TreeNode/blob/928695ba66085a8c96296e2e64c1e787892e4bf1/CSharp/TreeNode/TreeNode.cs#L389-L392)]
- **NEXUS rooting flags are ignored**: `[&R]` and `[&U]` are skipped only when the comment text equals them exactly [[src](https://github.com/arklumpus/TreeNode/blob/928695ba66085a8c96296e2e64c1e787892e4bf1/CSharp/TreeNode/NEXUS.cs#L364-L370)]
  - With the usual surrounding spaces they become a root attribute `Unknown='&R'` (Observed)
- **Reroot uses the branch reading**: on the path from the old to the new root, each reversed node takes the name, length, support, and all attributes of its former child [[src](https://github.com/arklumpus/TreeNode/blob/928695ba66085a8c96296e2e64c1e787892e4bf1/CSharp/TreeNode/TreeNode.cs#L520-L531)]
  - Support values therefore stay on the correct branch, the behavior that [Czech et al. 2017](https://doi.org/10.1093/molbev/msx055) recommend
  - Values that belong to nodes (names of internal nodes, ages) move by one node along the path, and there is no option to declare which attributes are node values
- **Unrooting**: one root child is grafted onto the other with the summed length [[src](https://github.com/arklumpus/TreeNode/blob/928695ba66085a8c96296e2e64c1e787892e4bf1/CSharp/TreeNode/TreeNode.cs#L398-L436)]
  - The new root takes the old root's name, and the support of the removed edge is lost

### Node and branch attributes

- **One dictionary per node**: annotations before and after the length go into the same attribute dictionary
  - A branch attribute and a node attribute with the same key overwrite each other
- **Attribute types**: every attribute is a string or a number. Modules declare which type they expect, and the formatters convert

### Polytomies and zero-length branches

- **Polytomies are kept**: the parser and the layouts accept any number of children. The rectangular parent sits at the midpoint of its outer children
- **Explicit operations**: "Polytomise node" collapses low-support branches into polytomies by an attribute test, and "Resolve polytomy" splits two siblings off
- **Zero-length branches are kept**: nothing collapses them
  - A zero-length tip sits on its parent node and its label overlaps the branches (Observed with `(E:0,F:1):0`)
- **Zero-length fixes in the history** (2021, [`0e7253f`](https://github.com/arklumpus/TreeViewer/commit/0e7253f), [`8483354`](https://github.com/arklumpus/TreeViewer/commit/8483354), [`45f7b27`](https://github.com/arklumpus/TreeViewer/commit/45f7b27)):
  - a tree of total length 0 is treated as length 1
  - divisions by zero length in Branches and Radial were fixed

### Consensus

- **Default greedy consensus**: threshold 0 keeps every split that is compatible with the splits already chosen, in order of frequency
  - Observed with `((A,B),(C,D))` and `((A,C),(B,D))`: the result keeps split AB with support 0.5 and drops the equally frequent AC, and the root becomes a trifurcation
  - The choice between equally frequent splits is arbitrary

## Defects

### Parsing

- **Slash splits names** (Observed): unquoted labels with `/` lose text into `Support` and `Unknown` attributes, see "Input parsing"
  - Influenza strain names are the common case
- **`[&R]` stored as an attribute** (Observed): see "Rooting and support values"
- **Silent stop on a bad tree** (Derived): a tree that fails to parse ends the file without a message, so later trees are lost
- **Escape flag overwritten** (Derived): the Newick file splitter passes `out escaping` where the tokenizer expects `out escaped` [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Newick_filetype.cs#L86)]
  - `\` escapes therefore do not work when the file is split into trees
- **Pipe characters in names** crash the interface during "Transforming trees" without a message
  - The issues were closed as user errors ([#21](https://github.com/arklumpus/TreeViewer/issues/21), [#40](https://github.com/arklumpus/TreeViewer/issues/40))

### Transformations

- **Compare trees across rooting** (Derived): presence uses full splits that include `@Root` only for a rooted tree [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Compare_trees.cs#L242-L280)]
  - A rooted tree compared with an unrooted tree therefore reports every split as absent
  - Compatibility is still correct
- **Consensus defaults read two trees** (Derived): the leaf-set test and the clock-like test use only the first two trees [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Consensus.cs#L72-L77)]
  - The manual says "if the trees all contain the same taxa"
- **Discordant trees in consensus**: opening a full BEAST tree file crashed the consensus ([#17](https://github.com/arklumpus/TreeViewer/issues/17), open)
  - The maintainer suspects very divergent trees
- **Collapse node marks only one child** (Derived): with `Equalise lengths` on, the code sets the "Collapsed" attribute twice on the first child and never on the second [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Collapse_node.cs#L205-L211)]
- **Polytomise "Equal" is a substring test** (Derived): for string attributes, the comparison uses `Contains`, so `Equal` to `A` also matches `AB` [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Polytomise_node.cs#L356-L361)]
  - Prune node has the same code

### Display

#### Labels

- **Negative numbers get no label** (Derived): the default number formatter returns nothing for values below 0 [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewer/CoreClasses/Modules.cs#L361-L378)]
  - Negative lengths and other negative values are therefore not shown
- **Centre of leaves in circular trees** (Derived): the centre is the mean of the minimum and maximum angle from `atan2` [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Labels.cs#L700-L713)]
  - A clade that crosses the negative x axis gets its label on the opposite side
- **Group label sort order** (Derived): with `Preserve nesting`, the comparator mixes ancestry with start positions [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Group_labels.cs#L1537-L1551)]
  - This is not a consistent order, so the sort can misplace labels or throw
- **"Only on last ancestor"** has no effect according to a 2025 report ([#63](https://github.com/arklumpus/TreeViewer/issues/63), open)
- **Label border uses the background formatters** (Derived): the border colour calls the background colour formatter, and the border thickness calls the background width formatter [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Labels.cs#L569)] [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Labels.cs#L615)]

#### Layout and images

- **Branch extensions on zero-length branches** point in the wrong direction
  - The workaround is a length of `1e-5` ([#47](https://github.com/arklumpus/TreeViewer/issues/47), closed without a fix)
- **Inner radius never 0 by default** (Derived): the default inner radius is clamped to at least 20, so the root never sits at the centre by default [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/Modules/Circular_coordinates.cs#L56-L60)]
  - The manual describes 0 as an option
- **Node images** draws nothing with the official example archive on Windows ([#68](https://github.com/arklumpus/TreeViewer/issues/68), open)

### Command line and documentation

- **Wrong module ID** (Derived): the command line's coordinates-module lookup uses the Transformer ID [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/src/TreeViewerCommandLine/Program.cs#L199-L210)]
- **Removed parameter in the wiki**: the "Comparing trees" page tells users to set the "Branch reference" of Branch extensions, a parameter removed in v2.2.0
- **Doc and code limits differ**: the Rectangular aspect ratio (see "Layouts")

## Performance

- **Interactive limit**: the maintainer reports that the 107,235-tip GTDB tree works in the interface ([#49](https://github.com/arklumpus/TreeViewer/issues/49)). The paper gives no benchmark
- **Labels dominate**: drawing many labels is slow in the interface, and on Linux text is drawn as paths, which uses more memory [[src](https://github.com/arklumpus/TreeViewer/blob/e2ba62a644a6e749304489ff0e03172c27a86e4e/Readme.md?plain=1#L114)]
- **Incremental work**: cached pipeline steps and one render layer per plot action limit recomputation (see "Incremental recomputation")
- **Lazy loading**: the Disk loader reads trees on demand from a binary file
  - A tree sample larger than memory can still feed the consensus or a single-tree view
- **Quadratic and cubic steps** (Derived):
  - "Compare trees" compares every split of the shown tree with every split of the other tree using arrays of names, which is cubic in the number of leaves
  - `GetSplit` removes leaves with `List.Contains`, quadratic per node
  - "Subsample tree" stores a full `n` by `n` distance matrix and scans it for each removed tip
  - "Sort nodes" recomputes subtree depths inside the sort comparison
- **Memory**: about 400 MB at idle according to the maintainer, 1.4 GB on one macOS report ([#26](https://github.com/arklumpus/TreeViewer/issues/26), open)

## Interaction with TreeKnit output

TreeKnit writes resolved segment trees, an ARG in extended Newick, `MCCs.json`, `MCCs.dat`, `ARG/nodes.dat`, Auspice JSON files, and SVG figures (`const ARG_NEWICK` and the other names in [packages/treeknit-io/src/output.rs#L29-L40](../../packages/treeknit-io/src/output.rs#L29-L40)).

### ARG

- **ARG extended Newick** (Observed with a file written to the shape of `fn extended_newick()` in [packages/treeknit-io/src/arg.rs#L14-L86](../../packages/treeknit-io/src/arg.rs#L14-L86)):
  - The file loads as a tree. The occurrence of `R1#H1` with children becomes a one-child internal node, which is invisible in the rectangular layout
  - The childless occurrence becomes an extra tip labelled `R1#H1` beside its other parent's clade
  - No reassortment edge is drawn, and nothing tells the user that the file is a network
  - Zero-length edges, which `fn set_branch_lengths()` can produce ([packages/treeknit-core/src/arg.rs#L492-L539](../../packages/treeknit-core/src/arg.rs#L492-L539)), load and draw at length 0 without error
  - `[&segments={0,1}]` becomes the string attribute `segments='{0,1}'`. A formatter could colour by it only through custom C# code
  - Export as Newick quotes all labels and keeps `#H1` in the names, so a round trip keeps the hybrid occurrences as plain names
- **Strain names with slashes**: TreeKnit writes labels unquoted ([packages/treeknit-io/src/arg.rs#L59](../../packages/treeknit-io/src/arg.rs#L59)), so labels such as `A/Texas/50/2012` break in TreeViewer as described under "Input parsing"

### Segment trees and MCCs

- **Resolved segment trees** (Observed): `fixtures/sim/sim_k2_n50_r0.05/tree1.nwk` loads correctly, and labels such as `37_0` and `internal_3` stay names
- **Two segment trees in one file** (Observed): the default Transformer replaces them by their consensus
  - The user must turn off "Consensus tree" and pick a tree by index to see a segment tree
- **MCCs as attributes**: `MCCs.json`, `MCCs.dat`, and the Auspice JSON have no file type module
  - A user can join an MCC table (taxon, MCC index) through "Parse node states" and then colour by it with a hand-written formatter
  - "Compare trees" between two segment trees marks the branches whose clade is present in the other tree, which is a visual approximation of shared clades
- **Auspice JSON**: not readable

## Ideas from the issue tracker and history

### Open requests

#### Display

- **Continuous gradient legend** ([#57](https://github.com/arklumpus/TreeViewer/issues/57)): an automatic legend for a numeric colour mapping
- **Shared scale for several trees** ([#41](https://github.com/arklumpus/TreeViewer/issues/41)): one scale for phylograms plotted separately
- **Multi-value support labels** ([#73](https://github.com/arklumpus/TreeViewer/issues/73)): show `99/1.0/99` as one label
- **Aligned highlight boxes** ([#32](https://github.com/arklumpus/TreeViewer/issues/32)): highlights that end at one right edge

#### Editing and discoverability

- **Spreadsheet editing and attribute tables** ([#36](https://github.com/arklumpus/TreeViewer/issues/36), [#44](https://github.com/arklumpus/TreeViewer/issues/44)): edit annotations in place and export labels as a table
  - A spreadsheet editor exists since v2.2.0, but the export is still open
- **Search inside many steps** ([#35](https://github.com/arklumpus/TreeViewer/issues/35)): a plot with more than 300 collapse steps is hard to manage
- **Attribute-driven node shapes are hard to find** ([#69](https://github.com/arklumpus/TreeViewer/issues/69))

### Declined

The maintainer declines nothing outright. He answers most requests with a workaround through existing modules or a custom script:

- **Line breaks in labels** ([#7](https://github.com/arklumpus/TreeViewer/issues/7)): only the Markdown legend supports them
- **Rotated group-label text** ([#7](https://github.com/arklumpus/TreeViewer/issues/7)): use Labels with the Origin anchor
- **Pipe characters in names** ([#21](https://github.com/arklumpus/TreeViewer/issues/21)): closed as a user error without a parser change
- **Keep lengths in a cladogram** ([#25](https://github.com/arklumpus/TreeViewer/issues/25), [#59](https://github.com/arklumpus/TreeViewer/issues/59)): copy `Length` into another attribute first

### Features found only in the history

- **Tree-space clustering**: pairwise RF distances with clickable clusters were added in 2023 ([`41fdf6c`](https://github.com/arklumpus/TreeViewer/commit/41fdf6c)), with the `distances` command ([`c0b26d3`](https://github.com/arklumpus/TreeViewer/commit/c0b26d3))
- **Branch reference parameter**: removed in v2.2.0, replaced by automatic layout detection ([`f407de6`](https://github.com/arklumpus/TreeViewer/commit/f407de6))
- **Parse tip states**: replaced in 2021 by "Parse node states", which also matches internal nodes ([`39ca2d0`](https://github.com/arklumpus/TreeViewer/commit/39ca2d0))
  - Its reference file is still in the repository
- **Separate PDF and SVG export modules**: merged into "Export" in v2.0.0 ([`14d0e76`](https://github.com/arklumpus/TreeViewer/commit/14d0e76))
- **Up and down arrows for module order**: replaced by drag and drop, still available through the global setting `ShowLegacyUpDownArrows` ([`7b231f0`](https://github.com/arklumpus/TreeViewer/commit/7b231f0))

## Open scientific problems

- **Node or branch meaning of attributes**: rerooting moves all attributes with their branch, which is correct for support and wrong for node ages and internal names
  - No format records which reading applies
- **Shared clades between trees**: "Compare trees" tests each split separately
  - It does not find maximal shared subtrees
  - Its presence test fails when the trees differ in rooting or leaf set
- **Arbitrary choice in greedy consensus**: equally frequent conflicting splits are resolved by order
  - The consensus of two discordant trees therefore shows one tree's split with support 0.5
- **Time origin**: ages from the farthest tip suit contemporaneous samples. For serially sampled trees the user must supply an offset by hand
- **Colour by category**: there is no stable palette for arbitrary categories, and the automatic colour depends on child order

## Not covered by TreeViewer

- **TreeKnit data**:
  - **Two trees on one canvas**: no tanglegram, no links between matching leaves, no untangling, and no shared colour between trees
  - **Networks**: no hybrid nodes, reticulation edges, or ARG display
  - **Auspice JSON and TreeKnit MCC files** as input
- **Display**:
  - **Automatic legends** for colour gradients and categorical attributes
  - **Label thinning** or overlap avoidance for tip labels
  - **Zoom-dependent detail**: the plot is a static figure with pan and zoom, and no level-of-detail rendering
- **Platform**:
  - **Web or browser version**: the program is a desktop application only

## Method and limits

- **Read**:
  - the TreeViewer core classes for the pipeline, rendering, selection, and statistics
  - all 100 module files: parameter lists in full, and the algorithms of the coordinate, branch, label, axis, highlight, transformation, consensus, and comparison modules
  - the command-line program and its command help texts
  - the TreeNode v1.5.4 parsers, rerooting, and split code
- **History**: all 72 issues, the single pull request, all release notes, and the commit log. Discussions are disabled
- **Paper**: the Wiley full text returned HTTP 403. The text was read from Europe PMC (PMC10834882)
- **Trial**: the v2.2.0 Linux release (`TreeViewer-Linux-x64.tar.gz`) ran in a throwaway `debian:bookworm-slim` container with only a temporary directory mounted
  - It needed `libicu72`, `libfontconfig1`, and `libssl3`
  - `TreeViewer -I` downloaded 102 modules from the current module repository, which can be newer than the v2.2.0 release
  - Test inputs: a polytomy with a zero-length tip, a missing length, long labels, two conflicting trees, a BEAST-style NEXUS file, a TreeKnit-style ARG, and a TreeKnit fixture tree. Outputs were inspected as PNG images and as Newick text
- **Not tested**:
  - the interactive interface was not run
  - setting internal-node labels through the command-line `option` command failed with "Unknown action", so internal labels were not checked in a trial
  - the TreeKnit fixtures contain no ARG text file, so the ARG input was written by hand to the shape of the TreeKnit writer
