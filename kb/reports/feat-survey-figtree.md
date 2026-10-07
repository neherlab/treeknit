# FigTree feature survey: tree rendering, semantics, and edge cases

This report describes FigTree, a Java desktop viewer for rooted phylogenetic trees that is widely used to draw summarized BEAST trees, as an idea inventory for the web app. It records what FigTree does, how it does it, which scientific conventions it assumes, which inputs break it, and which problems its maintainer left open. It does not compare FigTree with `packages/web`.

FigTree is licensed GPL-2.0-or-later (source file headers). Its tree and file library jebl2 carries LGPL headers and no license file ([jebl2 #7](https://github.com/rambaut/jebl2/issues/7)). Behavior and design may be studied, but copying code from either project needs approval (see the project rules).

- **Source**: [rambaut/figtree](https://github.com/rambaut/figtree) at commit `24c51ad` (2026-05-16), 326 commits since 2007. Source links point to this commit
  - The `dev` branch adds packaging commits and carries the tag `v1.5.0-beta1` (2026-05-26). Its source differs from `master` only in version handling
- **Library**: [rambaut/jebl2](https://github.com/rambaut/jebl2) at commit `51da3b1` (2023-11-05), 50 commits
  - FigTree ships it as the binary `lib/jebl.jar`, last updated in [`d7eec05`](https://github.com/rambaut/figtree/commit/d7eec05) (2021-02-26)
  - The jar predates two jebl2 commits, one of which only adds the position of a missing bracket to a Newick error message
- **Status**: the README says that FigTree "has been superseded by PearTree" ([`82504b6`](https://github.com/rambaut/figtree/commit/82504b6), 2026-03-20) [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/README.md?plain=1#L5)]
- **Other sources**:
  - all 210 issues and 13 pull requests of FigTree with their comments
  - all jebl2 issues and pull requests, and the commit history of both repositories
  - the version notes in `release/common/README.txt`
  - the web page <http://tree.bio.ed.ac.uk/software/figtree/>
  - the rerooting study [Czech et al. 2017](https://doi.org/10.1093/molbev/msx055)
  - FigTree has no paper and no manual beyond the web page and [`doc/large_trees.md`](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/doc/large_trees.md?plain=1)
- **Evidence labels**: two labels mark how a claim was checked
  - "Observed" means seen in a trial run of the release `v1.5.0-beta1` in its command-line graphic mode (see "Method and limits")
  - "Derived" means read from the code without running it
  - Where the documentation and the code disagree, the code wins and the difference is noted
- **Shape of the code**: a Swing application of about 31,000 lines
  - A `TreePane` lays out one tree through a `TreeLayout` and draws it with a stack of painters (labels, node shapes, node bars, scales, legend) and decorators (colour, width)
  - Each control panel ("controller") reads and writes its settings as key-value pairs, which are also the format of the `begin figtree;` block
- **Key files**:
  - [`TreePane.java`](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java): view pipeline, selection, fitting, and drawing order
  - [`RectilinearTreeLayout.java`](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RectilinearTreeLayout.java), [`PolarTreeLayout.java`](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/PolarTreeLayout.java), [`RadialTreeLayout.java`](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RadialTreeLayout.java): the three layouts
  - [`FigTreeFrame.java`](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java) and [`FigTreeApplication.java`](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeApplication.java): file input and output, export, and the command line
  - jebl2 [`NexusImporter.java`](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusImporter.java), [`NewickImporter.java`](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NewickImporter.java), [`ReRootedTree.java`](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/trees/ReRootedTree.java): parsing and rerooting

## Summary

### Scope and risks for TreeKnit users

See "Interaction with TreeKnit output".

- **Scope**: a figure tool for one rooted tree at a time, built around BEAST maximum clade credibility (MCC) trees
  - A file can hold many trees, but only one is on screen
  - There is no tanglegram, no network model, and no live comparison of trees
- **The ARG file fails to open**: `arg.nwk` is plain Newick with `[&segments={0,1}]` comments, and the Newick reader stops with "duplicate taxon 1}]" (Observed)
- **A NEXUS wrapper works but shows a tree**: each reassortment appears as an extra tip named like `ARGNode_2#H1`
  - The `segments` vectors cannot drive colour

### Scientific conventions

See "Scientific semantics".

- **Support moves with branches on reroot**: rerooting gives each reversed node the attributes of the node on the other side of its branch, which keeps support values on their bipartitions (Observed)
  - Node-specific values such as height intervals move too, and become meaningless (Observed)
- **Discrete or continuous colour is chosen by type**: an attribute is continuous only when every value is a number and at least one value is non-integer
  - Integer codes are therefore always discrete
- **Colour scales cover all trees of a file**, so one value has one colour in every tree

### Annotation model

See "Annotation model and file state".

- **One attribute map per node**: NEXUS `[&key=value]` comments before or after the branch length go into the same map
  - A branch is drawn with the values of the node below it
- **Display state lives in the tree**: user colours, highlights, collapsed clades, rotations, and clade names are stored as node attributes with a `!` prefix
  - The keys are `!color`, `!hilight`, `!collapse`, `!cartoon`, `!rotate`, and `!name`
  - They are written back into the NEXUS file
- **Panel state lives in a `begin figtree;` block**: `set key=value;` lines restore layout, labels, colours, scales, and legend
  - The two bundled examples have 59 and 66 such lines
- **Plain Newick gets none of this**: the Newick reader has no comment syntax
  - It sets every missing branch length to 1.0 (Observed)

### Display ideas worth noting

- **Node bars** for any two-element numeric interval, drawn along the time axis
- **Gradient branch colour** from the parent's colour to the child's colour along each branch
- **Cartoon and collapse** of clades, with the clade name from `!name` as the label of a collapsed triangle
- **Highlight** boxes that cover a clade and its tip labels
- **Calendar axis** from a scale factor, an offset, and axis reversal, extended to cover node bars older than the root

## Annotation model and file state

### Parsing of `[&...]` annotations

The NEXUS reader treats `[` and `]` as comment delimiters and `[&` as the start of a metacomment [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusImporter.java#L118-L122)].

- **Positions**: node and branch metacomments fill one map
  - A metacomment after a node label goes to the node [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusImporter.java#L1312-L1313)]
  - A metacomment after the branch length goes to the branch [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusImporter.java#L1259-L1261)]
  - The branch object forwards every attribute to its child node, so both positions fill one map [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/trees/SimpleRootedTree.java#L879-L897)]
- **Key-value syntax**: `key=value` pairs separated by commas [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusImporter.java#L1357-L1383)]
  - quoted keys for names with spaces
  - `{a,b}` lists and `{{a,b},{c,d}}` nested lists
  - a key without a value becomes `true`
- **Value types**: the value parser tries, in this order [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusImporter.java#L1396-L1466)]:
  - `{...}` becomes an array, element by element
  - `#rrggbb` becomes a colour. A value of the form `#-123` is an older signed-decimal colour. Hexadecimal output replaced it in 2014 ([`56efd61`](https://github.com/rambaut/figtree/commit/56efd61))
  - a quoted string becomes a string without the quotes
  - `true` and `false` become booleans
  - then integer, then double, then the trimmed string
- **Integer or double depends on the text**: `1` becomes an integer and `1.0` a double
  - A support column with an exact `1` among decimals once crashed the continuous colour scale ([#92](https://github.com/rambaut/figtree/issues/92), fixed in [`fba50b3`](https://github.com/rambaut/figtree/commit/fba50b3))
- **Internal node labels**: a plain Newick label after `)` becomes the attribute `label`, parsed with the same value rules [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusImporter.java#L1306-L1310)]
  - When any tree has such labels, the desktop application asks the user to name the attribute, for example `posterior` or `bootstrap` [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L897-L939)]
  - The command-line graphic mode skips this dialog
- **Tree-level comments**: comments before the tree set tree attributes [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusImporter.java#L1192-L1211)]
  - `[&U]` marks the tree as unrooted
  - `[&R]` is ignored
  - a MrBayes `[W 0.5]` becomes the attribute `weight`
  - other pairs become tree attributes

### Display annotations with `!` keys

FigTree stores interactive display state as node or taxon attributes whose names start with `!`. The attribute menus hide these names [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/AttributeComboHelper.java#L240-L244)], and the NEXUS writer saves them like any other attribute [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusExporter.java#L398-L441)].

- **Colour and names**:
  - **`!color`**: the "User selection" branch colour. The default branch decorator reads `!color` for the paint and `!stroke` for the line [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreeAppearanceController.java#L83-L86)]
    - On a taxon it colours the tip label (Observed: a taxon with `[&!color=#ff0000]` in the taxa block got a red label)
  - **`!name`**: a clade or taxon name. The label option "Names" shows it for internal nodes, and it replaces the taxon name on tips [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/BasicLabelPainter.java#L85-L113)]
- **Clade shapes**:
  - **`!hilight={tipCount, minTipHeight, #colour}`**: a filled box behind a clade
    - The box runs from the middle of the clade's stem branch to the time of the most recent tip of the whole tree, and spans the clade's rows plus half a row on each side [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RectilinearTreeLayout.java#L606-L630)]
    - Its outline is the fill colour made darker, and an option draws it as a white-to-colour gradient [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1677-L1706)]
  - **`!cartoon={tipCount, minTipHeight}`**: the clade becomes a triangle that keeps one row per tip, with the tip labels at the triangle's edge [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RectilinearTreeLayout.java#L441-L498)]
  - **`!collapse={"collapsed", minTipHeight}`**: the clade becomes a triangle in a single row. The node's `!name` becomes its tip label (Observed: "CD clade") [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RectilinearTreeLayout.java#L536-L604)]
  - **Stored heights**: cartoon, collapse, and highlight values store the minimum tip height of the clade at the time of the action
    - The function that would refresh them after a reroot or transform is empty [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L915-L917)]
- **Child order**:
  - **`!rotate=true`**: the node's children are drawn in reverse order
    - Rotation is a flag on the node, applied after automatic ordering, so it survives a change of the ordering option [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L231-L246)]

### The `begin figtree;` block

- **Syntax**:
  - **Format**: one `set key=value;` line per setting, for example `set appearance.branchColorAttribute="rate";`
    - Saving writes every key of every panel, sorted by name [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeNexusExporter.java#L52-L61)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L942-L955)]
  - **Value types**: `true`/`false`, `#` colours, integers, doubles, and strings [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeNexusImporter.java#L89-L120)]
    - Each panel casts its value to a fixed Java type. A hand-written `nodeBars.barWidth=4` therefore aborts with a `ClassCastException` because the panel expects `4.0` (Observed)
    - [#222](https://github.com/rambaut/figtree/issues/222) reports the same failure in the other direction
  - **Prefixes**: `layout`, `trees`, `appearance`, `colour`, `scale`, `tipLabels`, `nodeLabels`, `branchLabels`, `nodeShapeInternal`, `nodeShapeExternal`, `nodeBars`, `scaleBar`, `scaleAxis`, `legend`, `rectilinearLayout`, `polarLayout`, `radialLayout` [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreePanel.java#L61-L141)]
  - **Colour schemes**: each attribute's scheme is saved as `colour.scheme.<attribute>="<attribute>:HSBDiscrete{hue,1,0.0,1.0,0.6,0.6,0.8,0.4}"` or a continuous or interpolating variant [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/AttributeColourController.java#L282-L307)]
    - A user-reordered list of discrete values is saved as `colour.order.<attribute>`
- **Load and save behaviour**:
  - **Missing values**: an attribute name that the current file lacks is ignored without a message
    - The control keeps its previous selection (Observed: `legend.attribute="segments"` left the legend on another attribute)
  - **What the block does not restore**: the position of a user root
    - The block stores `trees.rooting` and `trees.rootingType` only [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreesController.java#L268-L269)], so a saved file reopens with the original root ([#81](https://github.com/rambaut/figtree/issues/81), [#208](https://github.com/rambaut/figtree/issues/208))
  - **Unsaved files**: a file without a FigTree block opens as modified and without a file name, so it cannot be overwritten without a new "Save as" [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L788-L794)]

### Taxon annotations and annotation import

- **Two attribute owners for tips**: a tip has node attributes and, separately, attributes of its taxon object (for example from the NEXUS taxa block)
  - Annotating selected tip labels writes to the taxon [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1068-L1077)]
  - Tip labels look on the taxon first and on the node second [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/BasicLabelPainter.java#L143-L154)]
- **Annotation import**: File > "Import Annotations" reads a CSV or tab-separated table [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L999-L1106)]:
  - the first column holds taxon names and the header line holds attribute names. Lines starting with `#` are skipped
  - the separator is a tab when the header contains one, else a comma. Quoted fields are split at commas
  - each column becomes boolean, integer, real, or string, depending on whether all its values parse
  - a row with a different number of fields stops the import with its row number
- **Annotate nodes from tips**: copies a taxon attribute to the tips and reconstructs internal values with Fitch parsimony
  - It refuses non-binary trees with "The Fitch algorithm can only reconstruct ancestral states on binary trees" [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/ExtendedTreeViewer.java#L173-L202)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/Parsimony.java#L82-L88)]
- **Annotate tips from nodes**: copies a node attribute of each tip to its taxon [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/ExtendedTreeViewer.java#L204-L216)]

## Layouts

The Layout panel selects Rectangle, Polar, or Radial [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreeViewerController.java#L49-L53)].

### Rectangular

- **Geometry**:
  - **Coordinates**: distance from the root maps to x, and tips take evenly spaced rows from top to bottom with spacing `1 / (n - 1)` of the height [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RectilinearTreeLayout.java#L141-L168)]
  - **Parent position**: an internal node sits at the mean of its children's rows, so a polytomy pulls the parent toward the side with more children [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RectilinearTreeLayout.java#L207-L208)]
    - Observed with `((A:1,B:1):1,C:2,D:2);`: the root sits at y = 154.72, the mean of 45.83, 168.33, and 250
  - **Root stub**: a horizontal line from the left edge to the root, with a length of "Root Length" (default 0.01) times the root height [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RectilinearTreeLayout.java#L49)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RectilinearTreeLayout.java#L151-L166)]
    - A root branch length in the file is discarded [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusImporter.java#L1181-L1186)]
- **Branch and label style**:
  - **Curvature** (slider, 0 to 1): 0 gives square elbows, 1 gives straight diagonal lines from parent to child, and values in between round the elbow with a quadratic curve [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RectilinearTreeLayout.java#L227-L273)]
    - Segment colouring along branches is off whenever curvature is above 0 [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RectilinearTreeLayout.java#L137-L139)]
  - **Align tip labels**: all labels move to the x of the most distant tip, and a dotted "callout" line of 0.5 px joins each tip to its label [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RectilinearTreeLayout.java#L304-L321)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L2597)]
- **Magnification and clade areas**:
  - **Fisheye**: a slider magnifies the rows around a point of interest
    - The transform is `c = 1 - d / (s + |d|)` with `s = 1 / (fishEye * n)` and `d` the distance from the point, rescaled to [0, 1] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RectilinearTreeLayout.java#L650-L663)]
    - The point follows the pointer while the Command key (Ctrl elsewhere) is held [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePaneSelector.java#L247-L251)]
  - **Clade areas**: the layout builds a polygon per internal node that covers its part of the tree
    - "Background" colouring fills these areas by an attribute (Observed in the bundled influenza example, coloured by height) [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RectilinearTreeLayout.java#L340-L439)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1658-L1674)]

### Polar

- **Coordinates**: distance from the centre is distance from the root, and the angle is the tip row
  - Tips use a spacing of `1 / n` of the angular range, so the first and last tips do not overlap on a full circle [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/PolarTreeLayout.java#L162-L190)]
- **Branches**: a radial line from each child to the parent's radius, plus an arc at the parent's radius [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/PolarTreeLayout.java#L308-L322)]
- **Options** [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/PolarTreeLayout.java#L42-L54)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/PolarTreeLayout.java#L168)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/PolarTreeLayoutController.java#L62-L139)]:
  - root angle, default 180°
  - angle range, default 360°
  - root length, default 10 * 0.01 = 10% of the root height
  - show root, and align tip labels
- **Aspect ratio**: kept fixed, so the tree is centred and scaled to the smaller window side [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/PolarTreeLayout.java#L76-L78)]
- **Time grid**: grid lines become concentric circles [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/PolarTreeLayout.java#L94-L97)]
  - The labelled axis is drawn only for the rectangular layout [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/ScaleAxisPainter.java#L103-L110)] (Observed: circles without tick labels)
- **Horizontal tip labels**: an enum value exists but throws "Not implemented yet" [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/PolarTreeLayout.java#L394-L397)]
  - The request was closed as won't-fix ([#37](https://github.com/rambaut/figtree/issues/37))

### Radial

- **Equal-angle algorithm**: each child receives a wedge of its parent's wedge in proportion to its tip count, and the branch points along the middle of the wedge [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RadialTreeLayout.java#L114-L171)]
- **Spread** (0 to 100): widens every non-root wedge by `1 + spread / 1000` before it is divided, which opens dense regions [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RadialTreeLayout.java#L145-L151)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RadialTreeLayoutController.java#L69-L76)]
- **No root stub, no axis, no highlight**:
  - the axis methods throw [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RadialTreeLayout.java#L77-L87)]
  - highlighting is commented out with the note "Not too clear how to do hilighting for radial trees" [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RadialTreeLayout.java#L129-L132)] (Observed: no box in the radial view)
- **Labels**: tip labels continue the direction of their branch and turn at 90° (Observed)

### Child order, rotation, and branch transforms

The view is a chain of wrapper trees: reroot, then order, then transform [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L108-L138)]. The loaded tree is not changed.

- **Order nodes**: "increasing" puts the child with more tips first (top), and "decreasing" the reverse
  - The sort is stable, so ties keep the input order [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/trees/SortedRootedTree.java#L29-L70)]
  - Observed on the bundled carnivore example: with "increasing", the single-tip outgroup is drawn at the bottom
- **Rotate**: reverses the children of the selected nodes (`!rotate`, see above)
  - "Clear rotations" removes the flag from the selection or from all nodes [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1041-L1059)]
- **Transform branches**: three options [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/trees/TransformedRootedTree.java#L41-L72)]:
  - **cladogram**: node height = the largest number of edges to a tip, so all tips align
  - **proportional**: node height = the number of tips below minus 1
  - **equal**: every branch has length 1, so tips at different depths do not align (Observed)
- **Automatic transform**: a tree without branch lengths is shown with the transform on [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L79-L85)]
  - Only NEXUS trees can lack lengths, because the Newick reader fills them with 1.0 (see "Scientific semantics")
- **Transform hides time**: with a transform on, these elements are hidden (Observed):
  - the scale bar, the axis, and the grid [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1639-L1655)]
  - the node bars [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1815-L1821)]
  - Hiding the scales dates from [`8a5b4fd`](https://github.com/rambaut/figtree/commit/8a5b4fd) (2023, [#167](https://github.com/rambaut/figtree/issues/167))

### Rooting

- **Reroot on a branch**: the root is placed at the midpoint of the first selected branch [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1022-L1031)]
  - A "Root" tool mode does the same on a click [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePaneSelector.java#L126-L130)]
- **Midpoint root**: finds the longest tip-to-tip path through cached subtree distances and roots at its middle [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/trees/ReRootedTree.java#L598-L649)]
  - It applies to every tree in the file
- **User rooting blocks tree switching**: the user root refers to a node of the current tree, so FigTree refuses to show another tree with "Cannot switch trees when user rooting option is on" [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/DefaultTreeViewer.java#L186-L193)]
- **Old root removed**: a degree-two root is skipped on reroot and its two edges are joined [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/trees/ReRootedTree.java#L67-L78)]
  - The new root has no attributes [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/trees/ReRootedTree.java#L770-L779)]
- **Attribute transfer**: see "Rooting and support values"

## Branches, nodes, and clade marks

### Branch colour and width

- **Colour by attribute**: a branch takes the colour of the value on the node below it. Without a value it is black [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1787-L1811)]
- **Gradient**: colours each branch with a linear gradient from the parent's colour to the child's colour along the drawn path, including the elbow (Observed with a `rate` attribute) [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1791-L1803)]
- **Segment colouring along a branch**: an attribute of the form `{state, interval, state, interval, ..., state}` splits a branch into coloured pieces, for structured-coalescent "lineage colourings" [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RectilinearTreeLayout.java#L228-L252)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1748-L1786)]
  - The menu lists such attributes with a `*` suffix, but selecting one returns no colour, marked "todo reinstate branch colouring" [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/AttributeColourController.java#L167-L169)]
  - The feature is dormant
- **Width by attribute**: width = `minWeight + value * lineWeight`, with the value scaled to [0, 1] over the range of the **first** tree only [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreeAppearanceController.java#L256-L265)]
  - The default line weight is 1.0 px, adjustable from 0.01 to 48 [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreeAppearanceController.java#L151-L157)]
- **Drawing order**: from back to front [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1616-L1923)]:
  - legend, scales, grid, clade backgrounds, highlights, and collapsed triangles
  - branches, node bars, and node shapes
  - tip labels with callouts, node labels, and branch labels
  - Node bars are therefore drawn over branches

### Node bars

- **Purpose**: draw a height interval, typically `height_95%_HPD` from TreeAnnotator, at every internal node
- **Eligible attributes**: only internal-node attributes that are arrays of exactly two numbers [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/AttributeComboHelper.java#L166-L179)]
- **Geometry**: the bar runs from height `values[0]` to `values[1]` along the node's local time direction, so it is horizontal in the rectangular layout and radial in the polar layout [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/NodeBarPainter.java#L57-L129)]
  - Its thickness is "Bar width" (default 4.0 px) and its fill is translucent blue `(24, 32, 228, 128)` with a 0.5 px black outline [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreePanel.java#L111)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/NodeBarPainter.java#L151-L165)]
- **Axis extension**: when the root's bar is older than the root, the time axis and the layout grow to include it [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1984-L2001)]
  - Observed: a root bar of {3.5, 6.0} on a tree of height 4.5 gave an axis to 2014 for tips at 2020
- **Absolute heights**: the bar uses the stored heights, so it only fits the tree drawn from the file's branch lengths
  - After a reroot the bars float away from their nodes (see "Defects")

### Node shapes

- **Shapes**: circle, rectangle, or diamond, with separate panels for tips and internal nodes since 2015 [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/NodeShapePainter.java#L50-L62)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreePanel.java#L51)]
- **Size by attribute**: two scaling modes [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/NodeShapePainter.java#L238-L251)]
  - "Width" scaling gives `size = minSize + maxSize * value`
  - "Area" scaling interpolates the shape area between `minSize` and `maxSize + minSize`
  - The default size is 4 px and the outline 0.5 px black [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/NodeShapePainter.java#L47)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/NodeShapePainter.java#L308-L309)]
- **Colour by attribute**: uses the shared colour schemes (Observed: internal circles coloured by a discrete `state`)

### Collapse, cartoon, and highlight

- **Actions** [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/menus/TreeMenuFactory.java#L58-L133)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L840-L1020)]:
  - Tree > Collapse (Cmd+1) and Cartoon (Cmd+2) toggle the mark on selected clades
  - Highlight (Cmd+L) asks for a colour
  - "Clear" removes marks from the selection or from all nodes
- **Nested marks**: the actions stop at the first selected node on each path from the root [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L969-L990)]
  - A collapse of a clade hides any marks below it
  - "Clear" uncovers nested marks one level at a time
- **Triangle tip**: the triangle reaches the clade's youngest tip, using the stored minimum tip height [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RectilinearTreeLayout.java#L536-L571)]
- **Extract subtree**: since 2023, the selected clade or tips can be opened as a new tree in a new window [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L1233-L1238)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1357-L1428)]
  - Unary nodes that the selection creates are removed

## Labels

### Label content

- **Three label painters**: tip labels (shown by default), node labels, and branch labels (both hidden by default) [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreePanel.java#L80-L120)]
- **Display choices**: "Names", "Solid box", "Node ages", "Node heights (raw)", "Branch times", "Branch lengths (raw)", and every attribute [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/LabelPainter.java#L39-L45)]:
  - **Node ages** apply the time scale (factor and offset), and **Branch times** apply the factor only [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/BasicLabelPainter.java#L126-L138)]
  - **Solid box** draws a filled box in the label colour instead of text, so coloured boxes can stand for a trait at each tip ([PR #134](https://github.com/rambaut/figtree/pull/134), 2019) [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/BasicLabelPainter.java#L115-L117)]
- **Array values**: an array is shown as `[a,b]`, and a one-element array as its element (Observed: `{0,1}` as `[0,1]`, `{0}` as `0`) [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/BasicLabelPainter.java#L159-L181)]
- **Number formats**: Decimal (`#.####`), Scientific (`0.###E0`), Percent, and Roman [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/LabelPainterController.java#L76-L80)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/LabelPainterController.java#L176-L192)]
  - Only `Double` values go through the format. Integers, floats, and strings are printed as they are [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/BasicLabelPainter.java#L159-L162)]
- **"Sig. Digits"**: the spinner sets the maximum number of **fraction** digits, default 2 [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/LabelPainterController.java#L196-L203)]

### Label style and placement

- **No overlap handling**: every label is drawn at its fixed point size, so dense trees overlap until the user enlarges the canvas (Observed on the 1,000-tip influenza example)
  - [`doc/large_trees.md`](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/doc/large_trees.md?plain=1#L4-L8) recommends a small font plus vertical "Expansion"
  - It names 2 pt as the smallest font, but the code allows 0.01 pt
- **Colour by**: each label painter has its own "Colour by" attribute
  - Tip labels default to "User selection", that is `!color` on the taxon (Observed)
- **Placement**: labels sit 10 px from their anchor (`labelXOffset`) [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L2571)]
  - Branch labels are centred above the middle of the branch [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L2315-L2349)] (Observed with "equal" lengths: a `1` above every branch)
- **Fonts**: default `sansserif` 8 pt for labels, 10 pt for the scale bar, size range 0.01 to 72 [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/LabelPainterController.java#L72-L74)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/LabelPainterController.java#L155)]
  - A size of 72 crashed until [PR #202](https://github.com/rambaut/figtree/pull/202), merged in 2026

## Colour and legend

All colour choices go through one per-attribute scheme, shared by branches, backgrounds, labels, and node shapes [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/AttributeColourController.java#L148-L190)].

### Value collection and type

- **Values from all trees**: the scheme collects the attribute from every node and taxon of every tree in the file, so colours agree across trees [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/AttributeColourController.java#L151-L161)]
- **Discrete or continuous**: an attribute is continuous when all its values are numbers and at least one is non-integer. Everything else is discrete [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/decorators/ColourDecorator.java#L165-L191)]
- **Arrays cannot be colours**: attributes whose values are arrays are left out of all colour menus [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/AttributeComboHelper.java#L252-L280)]

### Discrete schemes

- **Discrete scheme "HSB"**: values are sorted (numbers numerically, strings lexicographically), and value `i` of `N` gets hue `i / N`, saturation 0.6, and brightness 0.8 [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/decorators/DiscreteColourDecorator.java#L96-L134)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/decorators/HSBDiscreteColourDecorator.java#L109-L172)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/decorators/HSBDiscreteColourDecorator.java#L280-L287)]
  - Observed for three states: red, green, blue
  - **Secondary axis**: the dialog can step saturation or brightness within groups of hues ("secondary count"), which gives more distinguishable colours for many values
- **Fixed palette**: a "FixedDiscrete" scheme uses 26 preset colours and cycles when there are more values [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/decorators/DiscreteColourDecorator.java#L47-L76)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/decorators/DiscreteColourDecorator.java#L165-L169)]
- **Manual order**: the user can reorder values in the discrete dialog, and the order is saved in the file

### Continuous schemes

- **Continuous scheme "HSB"**: the value is scaled to [0, 1] over the data range, then hue runs 0 -> 1, saturation stays 0.6, and brightness runs 0.4 -> 0.8 [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/decorators/HSBContinuousColourDecorator.java#L97-L119)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/decorators/HSBContinuousColourDecorator.java#L204-L211)]
  - Hue 0 and hue 1 are both red, so the default scale starts at dark red and ends at light red (Observed in the legend of `rate`)
- **Continuous scheme "Interpolating"**: linear RGB interpolation between two or three user colours [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/decorators/InterpolatingColourDecorator.java#L109-L145)]
  - The defaults are a red and a blue [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/decorators/InterpolatingColourDecorator.java#L44-L45)]
- **Continuous scale options** [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/decorators/ContinuousScale.java#L135-L186)]:
  - a log scale, which throws for values <= 0
  - "normalize", which makes the range symmetric around 0 or applies user bounds
  - A diverging scale with a fixed midpoint was declined ([#181](https://github.com/rambaut/figtree/issues/181))

### Fills and legend

- **Fill colours**: shapes and backgrounds use the same colour at half alpha [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/decorators/ColourDecorator.java#L236-L238)]
- **Legend**: drawn in a column at the left of the tree for one chosen attribute [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L2392-L2399)]
  - Discrete legends show a swatch and the value per line, in the swatch colour
  - Continuous legends show a vertical colour bar with the maximum at the top and the minimum at the bottom (Observed) [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/LegendPainter.java#L150-L197)]

## Axes and time scale

- **Time scale panel**: two modes [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TimeScale.java#L51-L73)]:
  - **scale by factor with an offset**: `age = height * factor + offset`. The bundled influenza example uses factor -1 and offset 2005.25 to show calendar years
  - **scale the root to an age**: `factor = rootAge / rootHeight`
- **Reverse axis**: multiplies the factor by -1 and runs the axis from right to left [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TimeScale.java#L55)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L306-L313)]
  - Observed: offset 2020, factor 1, and "reverse axis" gave a forward calendar axis from 2014 to 2020
- **Root age field**: changing the root age lengthens the root stub so that it starts at the given age [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L372-L377)]
- **Scale bar** (shown by default): a bar with its length as text
  - The automatic length is one tenth of the root height, rounded up at its first significant digit [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/ScaleBarPainter.java#L193-L217)]
  - Observed: a tree of height 2 gets a bar labelled "0.3"
- **Scale axis** (hidden by default): a ruler under the tree with major and minor ticks
  - The automatic major spacing starts at the power of ten below the range and divides it into halves, quarters, fifths, or tenths until enough ticks appear [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/ScaleAxis.java#L393-L447)]
  - The user can set the spacing, the origin, the line width, and a grid of vertical lines at the ticks

## Zoom, fit, and navigation

- **Fit to window**: the layout reserves the label widths in pixels, then scales the tree to the remaining width and height [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L2120-L2198)]
  - A very long label shrinks the tree (Observed: a 69-character label left 82 of 400 px for the tree)
- **Zoom and Expansion**: zoom enlarges the canvas in both directions and Expansion only vertically
  - The canvas grows by the factor `1 + (s * max(n, 50) * 0.02)^1.2` for a slider value `s` in [0, 1] and `n` tips, so the range grows with the tree [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/DefaultTreeViewer.java#L227-L274)]
  - Labels keep their point size, so a larger canvas separates them
  - Expansion is disabled in the polar and radial layouts [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/DefaultTreeViewer.java#L241-L243)]
- **Keys**: Cmd+= and Cmd+- zoom, Cmd+Alt+= and Cmd+Alt+- expand, Cmd+0 resets [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreeViewerController.java#L217-L231)]
- **Pan**: hold Space and drag to scroll [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePaneSelector.java#L277-L301)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePaneSelector.java#L315-L318)]

## Selection, search, and editing

### Selection

- **Selection modes**: Node, Clade, Tip, and Taxa (tip labels) [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePaneSelector.java#L43-L48)]:
  - a click on a branch selects the node below it, and a click on a label selects that tip [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1285-L1311)]
  - **Shift** extends the selection to every node on the paths to the most recent common ancestor of the old selection and the new node [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L616-L639)]
  - **Cmd/Ctrl** toggles one item, and **Alt** swaps between Node and Clade for one click [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePaneSelector.java#L141-L172)]
  - a drag draws a grey rectangle and selects everything it touches [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePaneSelector.java#L186-L232)]
  - the selected branches are drawn under a 6 px stroke in the selection colour, default `(45, 54, 128)` [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L2598)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreeAppearanceController.java#L73)]
- **Conversions**: selected tips can become one of two node sets [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L753-L784)]
  - the clade of their common ancestor
  - the set of internal nodes on their paths that do not lead to all of them
- **Hover read-out**: the status bar shows `Subtree: n tips [height = h, length = l]` or `Tip: "name" [height = h, length = l]` for the branch under the pointer [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePaneRollOver.java#L56-L82)]
  - It shows the whole tree when the pointer is elsewhere

### Search

- **Find**: a panel searches taxon labels, branch lengths, node ages, any annotation, or one annotation [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L1508-L1540)]:
  - text tests: contains, starts with, ends with, matches, and regular expression, with a case option [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreeViewer.java#L175-L180)]
  - number tests: equals, not equals, greater, at least, less, at most [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreeViewer.java#L193-L199)]
  - results become the selection
- **Toolbar filter**: a search field in the toolbar matches the tip labels as displayed (case-insensitive) [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L388-L403)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/DefaultTreeViewer.java#L301-L311)]
  - It selects the matches and scrolls to the first one

### Editing and clipboard

- **Annotate**: Cmd+' sets a user-defined annotation on the selected nodes and taxa [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/ExtendedTreeViewer.java#L150-L154)]
- **Copy**: copies selected taxon names as text, or the selected subtree as NEXUS ([`5a7c9c4`](https://github.com/rambaut/figtree/commit/5a7c9c4)) [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L1356-L1373)]
- **Paste**: adds the trees of the clipboard to the open file [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L1375-L1404)]
- **No undo**: the maintainer states "There is no undo" ([#150](https://github.com/rambaut/figtree/issues/150))

## Multiple trees and two-tree display

- **No tanglegram**: there is no side-by-side view, no lines between matching tips, and no untangling
  - A request to draw two trees at the same scale ([#106](https://github.com/rambaut/figtree/issues/106)) was closed in 2026 without a code change
- **Tree navigation**: a file can hold any number of trees [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/MultipleTreesController.java#L46-L91)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/menus/TreeMenuFactory.java#L48-L53)]
  - The "Current Tree" panel shows the tree name, a number spinner, and the tree length
  - Cmd+] and Cmd+[ step through the trees
- **Shared settings**: all trees share one style
  - Colour schemes cover all trees, but the width scale reads the first tree only (see "Branch colour and width")
- **Dormant multi-pane viewer**: `MultiPaneTreeViewer` stacks up to eight tree panes vertically on one page, each with `1 / treesPerPage` of the height [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/MultiPaneTreeViewer.java#L111-L124)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/MultiPaneTreeViewer.java#L233)]
  - Nothing outside its two files uses it
  - It has no links between panes
- **Command-line graphics draw the first tree**: both readers stop after one tree [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeApplication.java#L126-L144)] (Observed with a two-tree Newick file)

## Networks and reticulations

FigTree has no network model. No source file, issue, or commit of FigTree or jebl2 mentions extended Newick, hybrid nodes, or reticulations.

- **Hybrid occurrences become tips**: in a NEXUS file, `X#H1` with children is an ordinary internal node with the label `X#H1`, and each childless `X#H1` is an ordinary tip with that taxon name (Observed)
  - The reticulation edge is not drawn
- **Duplicate tips fail**: a hybrid with three or more parents writes the same childless name twice, and the reader stops with a duplicate-taxon error [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusImporter.java#L1344-L1354)]
- **Unary nodes are drawn**: a node with one child is a corner in the branch with no mark (Observed with `((A:1,(B:1)U:1):1,C:3);`)
  - Trees with many such nodes, for example Markov-jump or transmission trees, load slowly or crash
  - The issue was closed as not planned ([#72](https://github.com/rambaut/figtree/issues/72))

## Input

- **Format detection**: the first non-empty line must contain `#NEXUS`, else the file is read as Newick [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L720-L728)]
- **NEXUS**: reads the taxa block (with taxon annotations), the translate table, `tree` and `utree` commands, and all trees of all trees blocks, then the FigTree block [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L747-L764)] [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusImporter.java#L1075-L1124)]
  - Commands before the first tree, such as Mesquite's `TITLE` and `LINK`, are skipped since 2012 ([#45](https://github.com/rambaut/figtree/issues/45))
  - An unknown command between two trees stops the import with "Unknown command" [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusImporter.java#L1219-L1223)]
- **Newick**: reads all trees in the file, with labels that may contain spaces [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L765-L771)]
  - Quoted labels work (Observed: `'O''Brien'` became `O'Brien`)
  - For comments and missing lengths see "Defects"
- **Empty and duplicate names** [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NewickImporter.java#L208-L223)]:
  - an empty tip name stops the import with "Emtpy node names are not allowed." [sic]
  - a repeated tip name stops it with a duplicate-taxon error
- **Other inputs**:
  - `-url` reads a tree from a URL [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeApplication.java#L405-L419)]
  - paste adds trees from the clipboard
  - annotation tables add taxon attributes
- **Large files**: a progress dialog appears after 1 s [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L684-L689)]

## Export and command-line graphics

### Tree files

- **Export Trees dialog**: NEXUS, Newick, or JSON [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/ExportTreeDialog.java#L44-L67)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L1435-L1480)]
  - options: "Save as currently displayed", "Save all trees", "Include FigTree block", and "Include Annotations"
- **As displayed**: applies the current reroot, order, and transform to every tree [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/DefaultTreeViewer.java#L161-L167)]
- **NEXUS writer** [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusExporter.java#L33)] [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusExporter.java#L223)] [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusExporter.java#L444-L469)]:
  - writes `[&R]` or `[&U]`
  - writes every attribute, including the `!` display keys, with strings in double quotes and colours as `#rrggbb`
  - writes branch lengths with `%.6g`, that is six significant digits
  - never writes the root length [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusExporter.java#L387-L394)]
- **Internal labels**: the writer saves `label` as `[&label=...]`, so support values leave the Newick label position
  - No option writes a chosen attribute as the Newick label ([#100](https://github.com/rambaut/figtree/issues/100), closed in 2026 without a code change)
- **JSON writer**: a nested `root` object per tree, with fields left over from one project [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/JSONTreeExporter.java#L57-L61)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/JSONTreeExporter.java#L93-L160)]
  - a fixed `"origin":"2013.34520547945"`
  - value lists for a fixed set of attribute names (`location`, `host`, `country`, and others)

### Graphics

- **Formats**: PDF (iText), SVG (Batik), PNG, and JPEG [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L1273-L1294)]
  - PNG keeps a transparent background, and JPEG is filled white [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L1296-L1313)]
  - The image has the size of the window, or of the enlarged canvas after zoom
- **SVG output**: Batik's Graphics2D generator writes each branch as one `<path>` in an elbow shape and each label as a `<text>` in a translated `<g>` (Observed)
  - The `viewBox` was added in 2016 ([#102](https://github.com/rambaut/figtree/issues/102))
- **Command line**: `figtree -graphic <PDF|SVG|PNG|JPEG> [-width W] [-height H] <tree-file> [<graphic-file>]`, with a default size of 800 * 600 [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeApplication.java#L244-L309)]:
  - without an output file, the image goes to standard output
  - the FigTree block of a NEXUS file controls all settings, so a saved FigTree file is a reproducible figure recipe
  - only the first tree is drawn, and the label-naming dialog is skipped
  - GIF is still handled in the code but missing from the option list, because its text was broken ([#93](https://github.com/rambaut/figtree/issues/93))
  - it runs headless with `-Djava.awt.headless=true` (Observed in `eclipse-temurin:21-jre`)
- **`-fast`**: a mode for big trees that turns off the hover read-out ([`d7eec05`](https://github.com/rambaut/figtree/commit/d7eec05)) [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/DefaultTreeViewer.java#L97-L100)]

## Configuration surface

- **Panels and keys** [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreePanel.java#L61-L141)]:
  - Layout: `layout.layoutType`, `zoom`, `expansion`
  - Trees: `trees.rooting`, `rootingType`, `order`, `orderType`, `transform`, `transformType`
  - Appearance: `appearance.*` colours, line weight, width attribute, gradient
  - Time Scale: `scale.*`
  - Tip, Node, and Branch Labels, Tip and Node Shapes, Node Bars, Scale Bar, Scale Axis, Legend, and the three layout panels
- **Preferences**: default fonts, colours, and line weight are kept in Java preferences and seed new windows [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreeAppearanceController.java#L88-L96)]
- **Defaults** [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreePanel.java#L83-L139)]:
  - on: tip labels and scale bar
  - off: node labels, branch labels, node shapes, node bars, axis, grid, and legend

## Scientific semantics

### Branch lengths, heights, and time

- **Heights from lengths**: node heights are computed from branch lengths with the youngest tip at height 0, so ages count back from the most recent tip
- **Missing lengths**: the Newick reader sets each missing length to 1.0 [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NewickImporter.java#L143-L148)]
  - Observed: in `((A:1,B:0):0,(C,D):1,E:2);`, C and D were drawn at distance 2 from the root, and the scale bar showed units that the file does not contain
  - A NEXUS tree without lengths is drawn as a cladogram instead
- **Negative lengths**: accepted and drawn as a branch that goes back in time (Observed: in `((A:1,B:-0.5):1,C:2);` B lies left of its parent)
  - The maintainer explains such pictures as bad input ([#124](https://github.com/rambaut/figtree/issues/124))
- **Height annotations are not positions**: `height` and `height_95%_HPD` drive labels and bars only. Positions come from branch lengths

### Rooting and support values

- **The problem**: support values describe branches (bipartitions), but Newick stores them as node labels
  - A reroot reverses the branches on the path to the new root, so their labels must move to the other end
  - [Czech et al. (2017)](https://doi.org/10.1093/molbev/msx055) found tools that get this wrong
  - A reporter of [IcyTree #59](https://github.com/tgvaughan/icytree/issues/59) found that FigTree puts rerooted IQ-TREE support values on the correct branches
- **FigTree's rule**: when a reroot walks from an old child to its old parent, the new node takes the attributes of the old child [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/trees/ReRootedTree.java#L80-L94)]
  - Every attribute therefore stays with its branch
- **Observed**: midpoint rooting of `((A:1,B:1)90:1,(C:1,(D:1,E:5)70:1)80:1);` moved the root onto E's branch
  - Label 70 moved to the node that now joins C with (A,B), which is the same bipartition {D,E}|{A,B,C}
  - Label 90 stayed
  - Label 80 belonged to one of the two root edges that were joined, and it disappeared
- **Side effect on node values**: height intervals, ages, and ancestral states also move one node along the path
  - Observed in the same run: the {6.5, 7.5} bar of (A,B) was drawn to the left of the new root, detached from every node
- **Open reports**:
  - labels next to the root are still misplaced in some trees ([#10](https://github.com/rambaut/figtree/issues/10))
  - a reroot doubled the length of the old outgroup branch in one report ([#159](https://github.com/rambaut/figtree/issues/159))

### Node attributes and branch attributes

- **One map**: node and branch metacomments share one attribute map per node (see "Parsing of `[&...]` annotations")
  - A node key and a branch key with the same name overwrite each other
- **Branch colour from the child**: a branch takes the colour of its lower node. This is the BEAST convention for rates and discrete states on branches
- **Gradient as a compromise**: the gradient option shows the change from the parent's value to the child's value along a branch, which suits node values such as reconstructed states

### Zero-length branches, polytomies, and unary nodes

- **Zero-length branches**: drawn as they are, with no collapsing
  - Observed: B with length 0 in `((A:1,B:0):0,...)` sits on its parent's x, and its label still appears
- **Polytomies**: drawn with the parent at the mean of the child rows (see "Rectangular")
  - Fitch parsimony refuses them
- **Unary nodes**: drawn as a corner with no mark
  - Extracting a subtree removes the unary nodes that the extraction creates
- **Unrooted trees**: the `[&U]` flag is read and written back, but the drawing is the same rooted drawing (Observed)

## Defects

Each entry gives the input, the effect, and the evidence.

### Parsing and file state

#### Newick and table input

- **Newick comments are label text**: the Newick reader sets no comment delimiters, so `[&x=1]` becomes part of a tip name [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NewickImporter.java#L33-L36)]. Observed:
  - `((A[&x=1]:1,B:1):1,C:2);` draws a tip named `A[&x=1]`
  - a comment that contains a comma splits the tip, so `arg.nwk` from TreeKnit fails with "duplicate taxon 1}]"
  - the maintainer first called Newick comments undefined, then agreed the reader should "either complain or parse them" ([#170](https://github.com/rambaut/figtree/issues/170), open)
  - the open jebl2 [PR #8](https://github.com/rambaut/jebl2/pull/8) parses them
- **Missing Newick lengths become 1.0** (Observed, see "Scientific semantics")
  - jebl2 PR #8 also removes this rule
- **Mixed attribute types** (derived): discrete colouring sorts values in a `TreeSet`, which throws when one attribute mixes numbers and strings [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/decorators/DiscreteColourDecorator.java#L100-L112)]
  - Non-numeric support labels such as `n/a` crash loading ([#56](https://github.com/rambaut/figtree/issues/56), closed as not planned)
- **Annotation tables with quoted commas** (derived): the CSV reader splits on every comma [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L1010-L1013)]

#### NEXUS output

- **Locale**: the NEXUS writer formats lengths with the default locale, so some systems write `0,000830000` ([#200](https://github.com/rambaut/figtree/issues/200)) [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusExporter.java#L374)]
- **Unescaped strings** (derived): string attribute values are written between double quotes without escaping an embedded quote [[src](https://github.com/rambaut/jebl2/blob/51da3b156f11e5f464cb4584fce1cfbd4368d97a/src/jebl/evolution/io/NexusExporter.java#L463-L465)]

#### FigTree block and settings

- **Typed values in the FigTree block**: an integer where a panel expects a double, or the reverse, aborts loading with a `ClassCastException` (Observed with `nodeBars.barWidth=4`, [#222](https://github.com/rambaut/figtree/issues/222))
- **User root not saved**: see "The `begin figtree;` block" ([#81](https://github.com/rambaut/figtree/issues/81), [#208](https://github.com/rambaut/figtree/issues/208))
- **Settings keys with spaces** (derived): `flattenName` discards the results of its `replaceAll` calls, so colour-scheme keys keep spaces and tabs of attribute names [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/AttributeColourController.java#L309-L315)]
- **"Import Colour Scheme..." reads an alignment** (derived): the menu item calls a copy of the character import [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L1178-L1191)]
  - Users still ask for colour-scheme import ([#212](https://github.com/rambaut/figtree/issues/212))

### Layout and drawing

#### Tip labels and tip count

- **Clipped tip labels** (observed): in the NEXUS-wrapped ARG, labels such as `ARGNode_86#H3` ran past the right edge of a 700 px image
  - The fit adds label widths in pixels to tree bounds in branch-length units [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L2421-L2446)], which reserves space only for the label of the most distant tip
  - A long label on a slightly shorter tip is cut
- **Single-tip tree** (observed): `(A:1);` gives `NaN` coordinates in the SVG, because the row spacing is `1 / (n - 1)` [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/RectilinearTreeLayout.java#L149)]

#### Reroot and transform

- **Node bars after reroot** (observed): bars keep their absolute heights and float away from their nodes (see "Rooting and support values")
- **Stale triangle and box sizes** (derived): cartoon, collapse, and highlight store a tip height when they are made, and the refresh function is empty [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L915-L917)]
  - After a transform or reroot the shapes can end at the wrong time
  - Related reports: [#113](https://github.com/rambaut/figtree/issues/113), [#141](https://github.com/rambaut/figtree/issues/141), [#198](https://github.com/rambaut/figtree/issues/198)

#### Axis

- **Axis extension from bars** (derived): the bar painter resets its maximum for every node, so the axis grows only for the bar of the last internal node visited, which is the root in parsed trees [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/NodeBarPainter.java#L57-L61)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1997-L2000)]
- **Reverse axis grid**: grid lines do not follow the reversed axis ([#149](https://github.com/rambaut/figtree/issues/149), [#218](https://github.com/rambaut/figtree/issues/218), both open)
  - The axis origin setting has no effect ([#145](https://github.com/rambaut/figtree/issues/145), open)

#### Selection and painters

- **Drag selection rectangle** (derived): the hit tests pass `height` as the width and `width` as the height, so a wide, flat drag tests a tall, narrow area [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1313-L1335)]
- **Polar rotation** (derived): with `!rotate`, the arc and the branch label of child `i` use the coordinates of the mirrored child, so arc colours swap between siblings [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/treelayouts/PolarTreeLayout.java#L296-L347)]
  - Rotation in the polar layout distorted branches in an older report ([#88](https://github.com/rambaut/figtree/issues/88))
- **Node shape backgrounds** (derived): the internal-node background pass checks the tip painter's setting [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1824)]

### Colour and controls

- **Default continuous scale wraps** (observed): hue runs from 0 to 1, so the minimum and the maximum are both red and differ only in brightness
- **All-negative attributes** (derived): the scale starts its maximum at `Double.MIN_VALUE`, the smallest positive number, so a range of negative values keeps a maximum near 0 [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/decorators/ContinuousScale.java#L262-L263)]
- **Width scale from one tree** (derived): the width range comes from the first tree only [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreeAppearanceController.java#L258)]
  - The "Min Weight" control is enabled by the colour attribute menu instead of the width menu [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreeAppearanceController.java#L202-L210)]
- **Large annotated trees**: PDF export and display fail with an `UnsupportedOperationException` in the discrete colour code ([#120](https://github.com/rambaut/figtree/issues/120), open)
- **Digits spinner** (derived): its minimum equals its default, so a user cannot go below 2 digits, or below a preferred default [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/painters/LabelPainterController.java#L196)]

## Performance

- **Observed timings** of the command-line SVG export, random binary trees, including about 0.4 s of container and JVM start:
  - 1,000 tips: 1.4 s, 0.46 MB of SVG
  - 10,000 tips: 4.0 s, 4.6 MB (PNG: 2.7 s)
  - 50,000 tips: 23.8 s, 22.9 MB
- **Full layout per change**: each repaint after a change runs the whole layout and recomputes every label bound [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/TreePane.java#L1925-L2405)]
- **Fixes in the history**:
  - a progress dialog for big files ([`d701ee8`](https://github.com/rambaut/figtree/commit/d701ee8))
  - a zoom range proportional to the tip count ([`5936b57`](https://github.com/rambaut/figtree/commit/5936b57))
  - cached tip counts in jebl2 ([`f459803`](https://github.com/rambaut/jebl2/commit/f459803))
  - `-fast`
- **Known slow cases**:
  - unary nodes ([#72](https://github.com/rambaut/figtree/issues/72))
  - more than 500 annotated taxa before 2014 ([#69](https://github.com/rambaut/figtree/issues/69))
  - slider lag on big trees ([#143](https://github.com/rambaut/figtree/issues/143))

## Interaction with TreeKnit output

This port writes all trees as Newick text ([packages/treeknit-io/src/output.rs#L276-L289](../../packages/treeknit-io/src/output.rs#L276-L289)) and writes no NEXUS file.

### ARG file

`ARG/arg.nwk` is extended Newick with a `[&segments={...}]` comment on every branch and `label#Hi` for each reassortment node ([packages/treeknit-io/src/arg.rs#L11-L87](../../packages/treeknit-io/src/arg.rs#L11-L87)).

- **As written, it fails** (observed): the import stops with "duplicate taxon 1}]"
  - the file has no `#NEXUS` header
  - the Newick reader splits `{0,1}` at the comma
- **In a NEXUS wrapper, it loads** (observed with `#NEXUS begin trees; tree ARG = [&R] <arg.nwk> end;`): the ARG is drawn as a tree with extra tips
  - Each reassortment node with its child is an internal node labelled `ARGNode_2#H1`, and its second occurrence is an extra tip with the same name
  - A 50-tip ARG with 6 reassortments was drawn as a tree with 56 tips
  - Some long labels were clipped
- **Segment semantics match**: TreeKnit annotates the branch above each node, and FigTree colours a branch with the value of the node below it
  - The vector form `{0,1}` cannot be used, because array attributes are left out of the colour menus
  - A scalar copy, for example `seg="01"` or an integer code, would colour branches by segment set at once
- **Root**: the `:0.0` root length of `GlobalRoot` is discarded, which is harmless

### Resolved trees and other outputs

- **Resolved trees (`*_resolved.nwk`)**: load and draw (observed)
  - Internal labels such as `internal_44` become the `label` attribute, and the desktop application asks for a name for it
  - A tree with missing lengths in the input keeps them missing in the output, and FigTree then draws every missing length as 1.0
- **Two trees**: FigTree draws one tree at a time
  - Both resolved trees can share one NEXUS trees block to step between them with consistent colours
- **MCC colouring idea**: three steps show the clades
  - an annotation table with columns `taxon,mcc` adds an MCC index to each tip
  - "Annotate nodes from tips" spreads it to internal nodes with Fitch parsimony
  - branch colour by `mcc` then shows the clades. Fitch requires binary trees, so a resolved tree with a polytomy is refused
- **`MCCs.json`, `MCCs.dat`, Auspice JSON, SVG figures**: FigTree reads none of these

## Ideas from the issue tracker and history

### Open requests

- **Input**:
  - **Newick importer parity** ([jebl2 PR #8](https://github.com/rambaut/jebl2/pull/8), 2026): parse comments, keep missing lengths missing, read the root length
  - **Better import errors** with line and position ([#118](https://github.com/rambaut/figtree/issues/118), [#138](https://github.com/rambaut/figtree/issues/138))
- **Colour**:
  - **Diverging colour scale** ([#181](https://github.com/rambaut/figtree/issues/181)): a white midpoint at a chosen value. The maintainer answered "No, sorry" and suggested rescaling the attribute
  - **Proportional colours inside cartoon triangles** ([#158](https://github.com/rambaut/figtree/issues/158)): show the trait mix of a collapsed clade. "Unlikely for the near future"
  - **Custom colours and colour-scheme import** ([#212](https://github.com/rambaut/figtree/issues/212), [#223](https://github.com/rambaut/figtree/issues/223))
- **Labels and command line**:
  - **Polar trees and dates from the command line** ([#207](https://github.com/rambaut/figtree/issues/207), [#209](https://github.com/rambaut/figtree/issues/209))
  - **Multi-line branch labels** ([#139](https://github.com/rambaut/figtree/issues/139)) and node numbers ([#109](https://github.com/rambaut/figtree/issues/109))

### Declined

- **Non-numeric support labels** ([#56](https://github.com/rambaut/figtree/issues/56)) and **unary-node performance** ([#72](https://github.com/rambaut/figtree/issues/72)): closed as not planned in 2026
- **Newick export with rotations** ([#81](https://github.com/rambaut/figtree/issues/81)): not planned
- **Undo** ([#150](https://github.com/rambaut/figtree/issues/150)): "very few options are irreversible"
- **Taxonomic labels in Newick** ([#97](https://github.com/rambaut/figtree/issues/97)): called "non-standard NEWICK format"

### Features found only in the history

- **Lineage colourings along branches**: the `{state, interval, ...}` attribute form still draws segments in the layouts, but the menu path returns nothing ("todo reinstate branch colouring") [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/AttributeColourController.java#L167-L169)]
- **Several trees per page**: `MultiPaneTreeViewer` (2007-2008) stacks trees vertically and is unused
- **Character alignment beside the tips**: `CharactersPainter` replaces tip labels with coloured alignment columns, but its menu action is commented out [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/application/FigTreeFrame.java#L98)] [[src](https://github.com/rambaut/figtree/blob/24c51adc9b4a61760b828d99511cf3449a7ba9d3/src/figtree/treeviewer/ExtendedTreeViewer.java#L122-L125)]
- **Phylogeography and KML export**: moved to BEAST in 2010 ([`04f4441`](https://github.com/rambaut/figtree/commit/04f4441))
- **Web and applet versions**: both were deleted in [`b628f31`](https://github.com/rambaut/figtree/commit/b628f31) (2023)
  - an applet ([`442ae2f`](https://github.com/rambaut/figtree/commit/442ae2f), 2007)
  - a JWt web version ([`1e6139e`](https://github.com/rambaut/figtree/commit/1e6139e), 2010)
- **GIF export**: dropped from the command line in [`8cfca46`](https://github.com/rambaut/figtree/commit/8cfca46) (2015) because text was missing

## Open scientific problems

- **Node values on reroot**: moving every attribute with its branch is right for support and wrong for heights, ages, and node states
  - No file format says which attribute is which
- **Joined root edges**: when a reroot removes the old root, the two root edges become one branch, and their two support values compete for one place
- **Hybrid occurrences**: without a network model, the second occurrence of a hybrid is a tip, and the reassortment edge is invisible
- **Intervals and transforms**: height intervals have no meaning on a cladogram or on a rerooted tree, so FigTree hides or misplaces them
- **Colour by integer codes**: the type rule makes every integer attribute discrete, which suits state codes and fails for counts

## Not covered by FigTree

- **Networks and ARGs**: no extended Newick semantics and no reticulation edges
- **Two trees**: no tanglegram, no tip links, no untangling, no shared layout of two trees
- **Colour by vector attributes**: arrays such as `{0,1}` cannot drive colour
- **Plain Newick annotations**: not read
- **Label thinning**: no hiding or culling of overlapping labels
- **Undo**: none

## Method and limits

- **Code read**:
  - FigTree `master` at `24c51ad`: the application, panels, layouts, painters, decorators, selection, search, export, and command line
  - jebl2 at `51da3b1`: the NEXUS and Newick readers, the NEXUS writer, `ReRootedTree`, `SortedRootedTree`, `TransformedRootedTree`, and `SimpleRootedTree`
  - The `dev` branch was compared and differs only in packaging and version handling
- **History read**: all 210 FigTree issues and 13 pull requests, all jebl2 issues and pull requests, and the commit logs of both repositories, through `gh` on 2026-10-07
  - The web page <http://tree.bio.ed.ac.uk/software/figtree/> was fetched
  - About 90 issues imported from Google Code in 2015, and 23 issues closed in bulk on 2026-04-01, have no comments or commits
  - Spot checks of four of the 2026 "completed" closures found the feature absent at the surveyed commit
- **Trial**: the release `FigTree_v1.5.0-beta1.tgz` from GitHub ran in throwaway `eclipse-temurin:21-jre` containers with no network, mounting only a scratch directory
  - All observations come from `-graphic` exports to SVG and PNG of about 30 test files
  - small Newick and NEXUS trees: polytomy, zero and negative lengths, missing lengths, a unary node, quotes, long labels, a single tip, two trees
  - NEXUS files with FigTree blocks that set each layout, transform, node bars, colours, legend, axis, rerooting, cartoon, and collapse
  - the two example files of the 1.4.4 release, random trees of 1,000 to 50,000 tips, and the output of a TreeKnit run on `fixtures/sim/sim_k2_n50_r0.05`
- **Not tried**: the desktop window
  - Selection, zoom, fisheye, dialogs, find, and the label-naming dialog are derived from the code
  - User rerooting on a chosen branch cannot be set from a FigTree block, so only midpoint rooting was observed
- **Version caveat**: the trial used `v1.5.0-beta1`, whose source matches the surveyed commit apart from packaging
