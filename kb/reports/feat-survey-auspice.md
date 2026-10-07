# Auspice feature survey: tree rendering, semantics, and edge cases

This report describes how Auspice, the tree viewer of Nextstrain, renders phylogenetic trees, as an idea inventory for the web app. It records what Auspice does, how it does it, which scientific conventions it assumes, which inputs break it, which defects its maintainers met and how they resolved them, and which problems remain open. It does not compare Auspice with `packages/web`.

Auspice is licensed AGPL-3.0. Its behavior and design may be studied. Copying its code needs approval (see the project rules).

- **Source**: [nextstrain/auspice](https://github.com/nextstrain/auspice) at commit `dbf2875` (2026-10-07), which is Auspice 3.0.0 (2026-09-02) plus unreleased fixes. Source links point to this commit
- **Live application**: <https://nextstrain.org>, operated in Chrome on 2026-10-07 with these datasets:
  - Zika and seasonal H3N2 HA
  - the H3N2 HA/NA tanglegram and an HA/HA tanglegram
- **Other sources**:
  - the git history and commit messages of the cited lines
  - open and closed issues and pull requests of `nextstrain/auspice`, with related items of `nextstrain/augur` and `nextstrain/ncov`
  - the [Nextstrain discussion forum](https://discussion.nextstrain.org), the Auspice and Augur documentation, and the `augur export v2` schema
  - Nextstrain workflows that build tanglegram datasets, and the tanglegram literature
- **Evidence labels**: "Observed" means seen in the live application. Other statements are read from the code without running it, or come from the cited issue, pull request, or forum thread. Where the documentation and the code disagree, the code wins and the difference is noted
- **Shape of the code**: a React and Redux app. The tree is drawn by `PhyloTree`, a d3 class that renders SVG and updates it through one `change()` entry point [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/change.ts#L299-L388)]
  - layout: `src/components/tree/phyloTree/layouts.ts`
  - drawing: `src/components/tree/phyloTree/renderers.ts`
  - state: `src/reducers/` and `src/actions/recomputeReduxState.js`

## Summary

### Geometry and data

- **Zero-length branches get no special treatment**: a polytomy that a tree builder or TreeTime resolved into zero-length bifurcations looks exactly like a real polytomy. Nothing in the dataset format marks a split as added by resolution
  - only "zoom out" knows about such chains. It climbs to the first branch with a nucleotide mutation
- **The divergence unit is guessed from the zoomed axis**: a largest value of at most 5 means "per site". Zooming can therefore flip the axis label and the clock rate unit
- **Gray means five things**: unknown value, value missing from the color scale, low confidence, filtered out, and deliberate background. Users cannot tell them apart (open since 2024)

### Two trees

- **The tanglegram shows two trees, matched by exact tip name**:
  - unmatched tips are dropped without a warning
  - lines take the left tip's color and thin to 0.25 px above 750 links
  - untangling has been off since 2018, because it hid the clade structure of the second tree
  - maintainers state that three trees will probably never be supported
- **The left dataset owns the coloring**: observed, the NA tree of an HA/NA tanglegram colored by HA subclade is entirely gray. Genotype coloring of the right tree is lost through a shallow-copy defect, even when both trees share a genome map

### Display conventions

- **A branch takes the inferred state of its child along its whole length**: this implies when a trait change happened. Parent-to-child gradients were built and reverted within days in 2020, because branches disappeared in some browsers
- **Filters select tips, then trim to their common ancestor**: trait filters ignore internal-node values. Genotype filters select nodes instead and draw no connecting path
- **Tip labels appear below 75 visible tips and never avoid each other**: observed, a filter that keeps 13 tips of a 1037-tip tree draws their labels at 8 px in an unreadable pile
- **SVG with d3 limits the tree to a few thousand tips**: maintainers aim at about 5000 tips, and animation is off above 4000 tips

### Defects found during this survey

See "Defects" for the full list. The main ones:

- an inverted axis for very small spans
- a temporal color scale that ignores the second tree's tips and sorts dates as text
- second-tree legend counts that ignore how often a value occurs
- JSON legends discarded for discrete colorings
- a zoom label on the root that never matches

## Data model that the renderer depends on

The dataset is an Auspice v2 JSON with `meta` and `tree`. Every renderer decision below follows from these fields.

### Positions

- **Divergence is cumulative**: `node_attrs.div` is the divergence from the root, not a branch length. A branch length is `div(child) - div(parent)` [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeMiscHelpers.js#L52-L58)]
  - users who parse the JSON miss this repeatedly [[forum](https://discussion.nextstrain.org/t/1314)]
  - a zero-length branch is therefore a child with the same `div` as its parent, and nothing marks it as introduced by polytomy resolution [[augur#908](https://github.com/nextstrain/augur/pull/908)]
- **Root divergence is ignored on export**: `augur export v2` requires a value on the root, ignores it, and starts every tree at 0. Without `mutation_length` or `branch_length` on the root, no divergence is exported at all (open since 2021) [[augur#734](https://github.com/nextstrain/augur/issues/734)]
- **Dates are decimal years at noon**: `num_date.value` follows the TreeTime `numeric_date` convention, with a day at noon and leap years of 366 days [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/dateHelpers.js#L7-L90)]
  - Auspice wrote its own conversion in 2019, after `2019-12-01` showed as `2019-11-30` [[#839](https://github.com/nextstrain/auspice/pull/839)]
- **Date confidence and provenance**: `num_date.confidence` is `[lower, upper]`. Since 2025, `num_date.inferred` and `num_date.raw_value` (for example `2020-XX-XX`) tell a reconstructed tip date from a provided one [[#1943](https://github.com/nextstrain/auspice/pull/1943)] [[augur#1760](https://github.com/nextstrain/augur/pull/1760)]
  - before, Auspice guessed "inferred" from a non-degenerate interval
  - the first proposal, "omit the confidence for known dates", was rejected because TreeTime does not always compute confidences [[augur#386](https://github.com/nextstrain/augur/issues/386)]
- **Metric availability is decided on the root alone**: if the root has `div` and `num_date`, both metrics are offered. If only one is present, the other metric and its controls (date slider, animation, clock layout, URL key `m`) are hidden [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L475-L486)] [[#670](https://github.com/nextstrain/auspice/pull/670)]
- **A node missing the active metric vanishes silently**: its depth is `undefined` and its path strings contain `NaN`, so the browser drops them without a warning [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L272-L285)]
  - a user with `num_date` on tips only saw tips and no branches [[forum](https://discussion.nextstrain.org/t/510)]
  - a tip without a date sits at the far left and drags the axis back [[forum](https://discussion.nextstrain.org/t/454)]
- **Several trees in one dataset**: `tree` may be an array. Auspice puts the roots under a synthetic `__ROOT` node with `hidden: "always"` and the minimum `div` and `num_date` of the roots, so all single-tree code keeps working [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeJsonProcessing.ts#L71-L87)] [[#1442](https://github.com/nextstrain/auspice/pull/1442)]
  - genotype coloring then assumes one coordinate system for all trees, which is wrong for trees from different alignments [[forum](https://discussion.nextstrain.org/t/1506)]
- **Each dataset root is its own parent**: the root stem has zero length and stays invisible unless the view is zoomed [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeJsonProcessing.ts#L116-L153)]

### Hidden branches

- **`node_attrs.hidden` is an enum**: `"always"` hides the branch in every view, `"timetree"` only on the time metric, `"divtree"` only on divergence [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/renderers.ts#L253-L270)] [[#676](https://github.com/nextstrain/auspice/pull/676)]
  - the Ebola all-outbreaks tree uses it to drop long undated backbone branches
  - a maintainer once suggested `"hidden": true` on the forum, which the schema rejects and the code ignores [[forum](https://discussion.nextstrain.org/t/510)]
- **Only the branch is hidden**: a tip with `hidden` is still drawn (open since 2019) [[#742](https://github.com/nextstrain/auspice/issues/742)]
- **Hidden nodes still define the x domain**: long hidden internal branches leave empty space [[#1950](https://github.com/nextstrain/auspice/issues/1950)]
  - a draft fix excludes them in the rectangular layout [[#1951](https://github.com/nextstrain/auspice/pull/1951)]
  - the maintainer advises against the same change for unrooted layouts, where ignoring basal nodes pulls visible clades together and makes them look more closely related

### Traits and uncertainty

- **Trait objects**: `node_attrs.<trait>` holds `value` and optionally `confidence`, `entropy`, `url`, and `raw_value` [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeMiscHelpers.js#L28-L50)]
  - `confidence` is a `{value: probability}` map for discrete traits and `[lower, upper]` for numbers
- **Observed and inferred values look identical**: a tip whose location came from metadata and an internal node inferred at 100 % both export `confidence: {X: 1.0}` (open since 2019, "high priority") [[augur#386](https://github.com/nextstrain/augur/issues/386)] [[augur#1505](https://github.com/nextstrain/augur/issues/1505)]
  - Auspice treats a tip with one value above 0.99 as observed
  - augur exports an entropy of about `-1e-12` for known tips
- **Missing values**: these count as missing, case-insensitive [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/globals.js#L173-L194)]
  - `unknown`, `?`, `nan`, `na`, `n/a`, the empty string, and `unassigned`
  - non-primitive values such as JSON `null`
  - `0` and `false` are valid. Strings such as `"null"`, `"none"`, `"missing"`, and `"-"` are ordinary categories with their own color
- **`?` and the empty string mean different things upstream**: in `augur traits`, `?` means "infer this". Such tips receive reconstructed values and inflate value counts in the filter list. An empty string is not inferred [[#1173](https://github.com/nextstrain/auspice/issues/1173)]
- **No field for branch support**: the schema description of `branch_attrs` mentions "support values", but only `labels` and `mutations` are defined [[augur#666](https://github.com/nextstrain/augur/issues/666)] [[forum](https://discussion.nextstrain.org/t/1202)]
  - bootstrap values can only be shown as a coloring or a label string
  - IQ-TREE support labels such as `75.6/72` replace internal node names in augur [[augur#856](https://github.com/nextstrain/augur/issues/856)]

### Branch attributes

- **Mutations are strings per gene, relative to the parent**: `branch_attrs.mutations` is `{gene: ["A123T", ...]}`, 1-based. Auspice reads the first character, the middle, and the last character [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeMiscHelpers.js#L134-L167)]
  - insertions and multi-base indels cannot be expressed
  - a 2022 discussion preferred the Nextclade insertion notation over VCF style. Insertions are still missing [[#1444](https://github.com/nextstrain/auspice/issues/1444)] [[#1962](https://github.com/nextstrain/auspice/issues/1962)]
- **Branch length and mutation list can disagree**: divergence comes from branch-length estimation and the mutation list from ancestral reconstruction [[augur#1689](https://github.com/nextstrain/augur/issues/1689)] [[augur#1690](https://github.com/nextstrain/augur/pull/1690)]
  - without an outgroup the root state is not identifiable, so one root child can get zero length and no mutations while the other gets all of them
  - augur now samples the root state from the profile
- **Branch labels are data, cast to strings**: `branch_attrs.labels.<key>` defines the label keys (`clade`, `aa`, any custom key). Values become strings at load, because URL queries are strings and `"3" === 3` is false [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeJsonProcessing.ts#L51-L68)] [[#1157](https://github.com/nextstrain/auspice/pull/1157)]
- **No unit for divergence**: the dataset never says whether `div` is in substitutions per site or in mutation counts (`augur refine --divergence-units`). See "Divergence units" [[#1238](https://github.com/nextstrain/auspice/issues/1238)]

### Names

- **Duplicate and missing names are repaired with a console warning only**: an unnamed node gets a random 6-character name, and a duplicate gets `_<random>` appended [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeJsonProcessing.ts#L28-L39)] [[#1514](https://github.com/nextstrain/auspice/pull/1514)]
  - this was an error once and became a warning, because some pipelines use duplicate display names on purpose [[#1541](https://github.com/nextstrain/auspice/issues/1541)]
  - the user sees nothing in the interface, and tip matching between trees then fails silently
- **Internal node names collide across trees**: independent inference names nodes `NODE_0000001` in every tree. A draft augur PR makes them unique for pipelines that export several trees [[augur#1451](https://github.com/nextstrain/augur/pull/1451)]
- **DOM ids are derived from names**: runs of non-word characters become `-`, so `A/B` and `A.B` collide in hover lookups [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/helpers.ts#L11-L21)]

## Layouts

### Layout set

- **Five layouts**: rectangular (`rect`, default), `radial`, `unrooted`, `clock`, and `scatter`, chosen in the sidebar or with URL key `l` [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L16-L54)]
- **Clock is a fixed scatterplot**: date on x and divergence on y. The user's scatter variables survive a trip through other layouts [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L35-L42)]
- **Two distance metrics only**: `div` and `num_date`. Any other value falls back to `div` [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L256-L268)]
  - requests for more axes (APOBEC3 versus non-APOBEC3 divergence for mpox, amino-acid divergence) stay open [[#1769](https://github.com/nextstrain/auspice/issues/1769)] [[#265](https://github.com/nextstrain/auspice/issues/265)]
  - they are blocked because the two-value enum is assumed everywhere, and because the dataset first needs a divergence unit

### Rectangular

- **Elbow drawn as two paths**: each node draws its horizontal branch (the "stem", from the parent's x to its own x). An internal node also draws a vertical bar (the "tee") that spans its children, in its own color [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L524-L534)]
- **Who owns the vertical bar is debated since 2017**: [[#200](https://github.com/nextstrain/auspice/issues/200)] [[#167](https://github.com/nextstrain/auspice/issues/167)] [[#2025](https://github.com/nextstrain/auspice/issues/2025)] [[#508](https://github.com/nextstrain/auspice/issues/508)]
  - Neher: the bar is the internal node itself, and child-owned L-shapes make the branches of a multifurcation overlap
  - Bedford: when a trait changes at a node, the bar shows the ancestor's color, and uniform bars hide the line of descent
  - an open 2025 proposal attaches the bar to the start of each child branch, so that filtered trees drop dangling bar segments. The unsolved part is how long the bars should be for children of polytomies
- **Stem offset for thick parents**: a child stem starts `0.5 * (parentWidth - childWidth)` px left of the parent's x, so that a thin child tucks into a thick parent bar without a notch. Branch width follows visible tip counts, so the offset changes with filtering [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L526)] [[#183](https://github.com/nextstrain/auspice/pull/183)]
- **Left overshoot (open)**: stems use `stroke-linecap: round`, which extends each stem by half its width past its start, on top of the stem offset. Users report branches that stick out to the left [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/renderers.ts#L340-L364)] [[#1481](https://github.com/nextstrain/auspice/issues/1481)]
- **A 0.01 px leftover**: every horizontal branch ends 0.01 px below its start, because an SVG linear gradient cannot apply to a path with a zero-height bounding box [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L528-L534)] [[#1005](https://github.com/nextstrain/auspice/issues/1005)] [[#1022](https://github.com/nextstrain/auspice/pull/1022)]
  - the gradients were removed in 2020 [[#1042](https://github.com/nextstrain/auspice/pull/1042)]
  - the jitter, an empty `<defs>` group, and the commented gradient code remain
- **Room for the zoomed clade's stem**: when zoomed, the x domain extends 5 % to the left [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L410-L413)] [[#1392](https://github.com/nextstrain/auspice/pull/1392)]
  - the clip mask added in 2021 had hidden the stem of a zoomed clade, which users click to zoom out
  - the commit notes that hard-to-find root branches remain a problem
- **Mirroring**: `params.orientation` flips the x or y scale. The right tree of a tanglegram uses `[-1, 1]` [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L326-L338)]

### Radial

- **Geometry**: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L220-L235)]
  - tips cover 95 % of the circle (`angle = 2 * 0.95 * π * displayOrder / maxDisplayOrder`), which leaves an 18° gap
  - angle 0 points down, and display order runs counterclockwise on screen
  - a branch starts on the child's ray at the parent radius. A `1e-15` term avoids a division by zero for zero-length root branches
  - an internal node draws an SVG arc at its own radius across its children's angles
- **Square domains**: radial and unrooted use square pixel ranges and square data domains, because branch length must mean the same in every direction [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L316-L325)]
- **Gaps between subtrees become angular gaps**: the denominator is the maximum display order since exploded trees came. The commit notes that angular separation says little when a subtree root is near the centre [[`455bc3ba`](https://github.com/nextstrain/auspice/commit/455bc3ba)]
- **Zoom is a camera move, not a re-layout**: [[#752](https://github.com/nextstrain/auspice/pull/752)] [[#1345](https://github.com/nextstrain/auspice/issues/1345)]
  - an earlier version unfurled a zoomed clade to fill the circle and was reverted as disorienting
  - a user request to bring it back is open
  - maintainers also abandoned a whole-tree view that squashed out-of-view nodes, which worked in y but not in x
- **Tip labels stay horizontal**: on the left half they run back over the tree (open request to rotate them) [[#1423](https://github.com/nextstrain/auspice/issues/1423)]

### Unrooted

- **Equal-angle layout**: each clade gets a wedge proportional to its leaf weight, and its branch points to the wedge centre with length `depth - parentDepth` [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L141-L160)]
- **Filtered tips keep 15 % weight**: leaf weight is `tipCount + 0.15 * (fullTipCount - tipCount)` [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L657-L659)]
  - added in 2019 so that a zoomed clade unfurls into more angle [[#754](https://github.com/nextstrain/auspice/pull/754)]
  - a 2022 rewrite counts visible tips, so filters and the date slider now also reshape the layout. This looks like an unplanned side effect [[`32593b4e`](https://github.com/nextstrain/auspice/commit/32593b4e)]
- **Single-child nodes**: an open code TODO asks to check that unary nodes do not widen the layout [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L163-L165)]
- **Polytomies collapse to one point**: samples at a polytomy, and identical samples, share one position. The root is not drawn, so a large root polytomy becomes overlapping dots (open) [[#1522](https://github.com/nextstrain/auspice/issues/1522)]
  - proposals: fan labels at angles, scale the dot by sample count, or a small swarm plot
  - zero-length branches from resolution look identical to unresolved polytomies here too
- **Clade zoom cannot pick a side of an edge**: clicking near the root zooms to the side away from the root. Bedford noted that a click cannot specify a side without a rooting [[#754](https://github.com/nextstrain/auspice/pull/754)]
- **No grid and no scale bar** [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/grid.js#L222-L226)]

### Clock and scatter

- **Any two variables**: each axis is `div`, a genotype, display order ("node order"), or any trait. Temporal traits are converted with `numDate` [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L79-L122)] [[#1310](https://github.com/nextstrain/auspice/pull/1310)]
- **Categorical axes**: a point scale in legend order [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L591-L628)] [[#1346](https://github.com/nextstrain/auspice/pull/1346)]
  - jitter of up to a quarter band applies only when a band is wider than 50 px, so it disappears in small panels
  - the jitter uses `Math.random` and changes on every render
- **No jitter on continuous axes**: jitter would misstate values, so identical points hide each other (open) [[#1585](https://github.com/nextstrain/auspice/issues/1585)]
- **Missing values are moved off-screen**: nodes without a value sit 100 px outside the range, and their branches are skipped. The header count still includes them (open) [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L476-L498)] [[#1318](https://github.com/nextstrain/auspice/issues/1318)]
- **Node order as an axis**: display order against a trait recreates a tree-like plot that shows per-clade trait distributions. Subtree roots get no parent value, so exploded subtrees stay unjoined [[#1494](https://github.com/nextstrain/auspice/pull/1494)]
- **Axes ignore filters**: filtering only restyles points, and the domains keep the full data range (open, tied to a redesign of zoom versus filter) [[#1317](https://github.com/nextstrain/auspice/issues/1317)] [[#1351](https://github.com/nextstrain/auspice/issues/1351)]

### Tip order and vertical spacing

- **No ladderization in the viewer**: the display order is the child order of the JSON, and ladderizing is left to the pipeline [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/helpers.ts#L99-L128)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/helpers.ts#L225-L236)]
  - children are visited last to first, so the last child is at the top
  - the subtrees under `__ROOT` are visited first to last
- **Internal node y is the mean of its children's y**: it is not the midpoint of the outer children, so a polytomy's node is pulled toward the side with more children [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/helpers.ts#L124-L127)]
- **Gaps between subtrees**: in exploded and multi-tree datasets each subtree is followed by a gap of `numTips / 20` tips, or none for small trees. The commit calls the value "based off testing a few datasets" [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/helpers.ts#L135-L147)] [[`937c68cf`](https://github.com/nextstrain/auspice/commit/937c68cf)]
- **Empty subtrees take no space** [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/helpers.ts#L226-L229)]
- **5 % vertical padding below 30 tips**, so that small clades do not touch the panel edge [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L427-L432)] [[#556](https://github.com/nextstrain/auspice/pull/556)]

## Branches and tips

### Branch width and tip size

- **Width encodes the visible share of tips**: visible branches get `sqrt((tipCount + 5) / (maxTipCount + 5))` scaled to 1 to 10 px [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeVisibilityHelpers.js#L93-L114)] [[#330](https://github.com/nextstrain/auspice/pull/330)] [[#638](https://github.com/nextstrain/auspice/pull/638)]
  - `tipCount` counts visible descendant tips, and the maximum is the whole tree's count, so a zoomed small clade keeps thin branches
  - non-visible branches get 0.5 px
  - the `+5` pseudo-count (2017) keeps small clades from vanishing
  - the width is recomputed on every filter and date change [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeVisibilityHelpers.js#L266-L283)]
- **Tip radius**: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/tipRadiusHelpers.ts#L53-L89)]
  - 4 px by default
  - 7 px for tips that match a hovered legend entry or map location
  - plus 4 px for the hovered tip
  - a selected-tip radius of 10 px exists in code that nothing calls [[#1891](https://github.com/nextstrain/auspice/issues/1891)]
- **Tip stroke width is fixed at 1 px**: branch width never applies to tips [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/renderers.ts#L217-L251)]
- **Vaccine strains are drawn as 6 px crosses**: a vaccine entry with only `serum` does not count [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L499-L506)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeJsonProcessing.ts#L187-L190)]

### Zero-length branches and polytomies

Resolved polytomies produce this case, and Auspice has met it many times.

- **Drawing**: in the rectangular layout a zero-length stem is a dot as long as the stem offset, with a round cap, and the child's bar lies on top of the parent's bar [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L524-L534)]
  - TreeTime encodes polytomies as bifurcations with zero-length branches during coalescent reconstruction
  - such resolved polytomies therefore look like polytomies on the divergence metric
- **The divergence view shows what resolution added**: TreeTime must date ancestors even when they are genetically identical to their descendants. The time view shows structure that the divergence view collapses into polytomies [[#1125](https://github.com/nextstrain/auspice/issues/1125)]
  - identical tips on zero-length branches sit exactly at their parent's x [[forum](https://discussion.nextstrain.org/t/576)]
- **Arbitrary resolution creates confident-looking structure**: with many identical genomes, the tree builder resolves the multifurcation arbitrarily, and TreeTime then dates the new nodes and infers traits on them. The result is over-confident ancestral country calls "from zero SNVs" [[augur#537](https://github.com/nextstrain/augur/issues/537)]
  - option discussed: collapse zero-mutation branches in the time view only. This is cheap, but trait inference stays biased
  - option discussed: keep polytomies with `augur refine --keep-polytomies` [[augur#345](https://github.com/nextstrain/augur/pull/345)]
  - Bedford's tradeoff: kept polytomies make "all the clades look explosive", so the team switches between the divergence and time views instead
- **Polytomy resolution is stochastic**: TreeTime trims branches without mutations, then resolves polytomies greedily by temporal order and clock fit. The result depends on the rooting and the input tree [[augur#345](https://github.com/nextstrain/augur/pull/345)]
- **Zoom out skips resolved polytomies**: a one-parent step "did nothing" visibly when the parent was a zero-length bifurcation [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/helpers.ts#L274-L352)] [[#936](https://github.com/nextstrain/auspice/issues/936)] [[#1001](https://github.com/nextstrain/auspice/pull/1001)]
  - first rule: climb while the ancestor is within 1/50 of the axis span. On an mpox tree with an in-view span of 563 mutations it skipped branches of 1 and 12 mutations and jumped two levels [[#1529](https://github.com/nextstrain/auspice/issues/1529)]
  - since 2022 on the divergence metric with mutations present: climb to the first branch with a nucleotide substitution. Gaps, Ns, and undeletions do not count [[#1552](https://github.com/nextstrain/auspice/pull/1552)]
  - the 1/50 rule remains for the time metric and for trees without mutations
  - the 1/50 tolerance depends on the view: on time it uses the date range of the current slider selection, on divergence the current in-view x domain [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/helpers.ts#L278-L295)]
- **A floor for flat polytomies**: since 2020 the x domain is at least 1e-8 wide [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L397-L409)] [[#901](https://github.com/nextstrain/auspice/issues/901)] [[#909](https://github.com/nextstrain/auspice/pull/909)] [[`0446278c`](https://github.com/nextstrain/auspice/commit/0446278c)]
  - before, zooming into a clade that is flat in divergence spread a fake topology across the screen, because augur had exported resolved branches of about 3e-12
  - the commit calls the value arbitrary and warns that it "will become problematic if we get eukaryotic genomes"
  - the code sets `maxX = 1e-8 - minX` instead of `minX + 1e-8`, which reverses the axis when `minX` is positive (see "Defects")
  - an exactly zero span is first widened by 0.005 data units on each side, which is large for substitutions per site and tiny for years
- **Hit area**: no invisible enlarged hit path exists, so a zero-length stem is clickable only on its round cap, a circle as wide as the stroke. Clicking it zooms into one arbitrary resolved sub-split [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/renderers.ts#L344-L363)]
- **Branch labels collide**: when a parent and child joined by a zero-length branch both pass the label threshold, their labels sit at the same x and adjacent y [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/labels.js#L82-L98)]
- **Downstream tools reject polytomies**: Nexus exported from Auspice keeps them, and BEAST 1 rejects non-bifurcating trees. The advice is TreeTime without `--keep-polytomies`, or `ape::multi2di` [[forum](https://discussion.nextstrain.org/t/1496)]
- **Single-child nodes**: drawn as a straight line with an invisible joint that can still be hovered and clicked. Filters hide unary ancestors above the common ancestor [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/helpers.ts#L124-L127)]

### Negative and inconsistent lengths

- **No clamping anywhere**: a negative branch length, or a child dated before its parent, is drawn as is [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L141-L147)]
  - rectangular: the branch runs leftwards
  - radial: the branch start lies beyond the child
  - unrooted: the branch points the opposite way
  - maintainers treat these as data problems (wrong root, non-clock-like branches) to inspect with `treetime clock` [[forum](https://discussion.nextstrain.org/t/2045)]
- **Implausibly old roots are a frequent report**: one misdated tip or a wrong clock rate drags the root back (SARS-CoV-2 rooted in 1961, Chikungunya in 1777). The viewer shows the root date without any warning [[forum](https://discussion.nextstrain.org/t/603)] [[forum](https://discussion.nextstrain.org/t/211)] [[forum](https://discussion.nextstrain.org/t/1967)]

## Axes, units, and dates

### Divergence units

- **The unit is guessed**: `guessAreMutationsPerSite` returns "per site" when the largest value of the current scale domain is at most 5 [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/helpers.ts#L361-L366)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/grid.js#L393-L405)]
  - the guess sets the axis title: "Divergence" or "Mutations"
  - the guess sets the clock rate text: "subs per site per year" or "subs per year"
- **The guess follows the zoom**: the domain is fitted to the in-view nodes. Not reproduced live, because no public dataset offered such a clade within easy reach
  - a clade near the root of a mutation-count tree flips to "Divergence"
  - a deep per-site tree above 5 substitutions per site is labelled "Mutations"
- **History**: [[#582](https://github.com/nextstrain/auspice/pull/582)] [[#858](https://github.com/nextstrain/auspice/issues/858)] [[#860](https://github.com/nextstrain/auspice/pull/860)] [[#1238](https://github.com/nextstrain/auspice/issues/1238)] [[#1683](https://github.com/nextstrain/auspice/issues/1683)]
  - until 2018 the clock text said "subs per year", which was wrong for per-site divergence
  - in 2020 SARS-CoV-2 builds switched to mutation counts, and the rate became wrong by a factor of the genome length. This led to the guess
  - Bedford rejected "subs per genome per year", because a count tree may cover one gene
  - declaring the unit in the dataset is planned and open since 2020
- **Count-mode divergence upstream**: count mode first multiplied per-site divergence by the genome length, which gave fractional counts. Augur now counts reconstructed mutations [[augur#618](https://github.com/nextstrain/augur/pull/618)] [[augur#1467](https://github.com/nextstrain/augur/issues/1467)]
  - sites masked in every sequence once produced thousands of spurious mutations, because the root state was drawn from a flat distribution [[augur#1779](https://github.com/nextstrain/augur/pull/1779)]
- **Fractional gridlines for integer counts**: the numeric grid puts minor lines at 0.2 mutations. Fixing it needs the unit (open) [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/grid.js#L49-L73)] [[#1359](https://github.com/nextstrain/auspice/issues/1359)]
- **Divergence is not a count of differences from the reference**: users expect a count against Wuhan-Hu-1 and see different values across runs. The model accounts for reversions and multiple hits, and phylogenetic uncertainty changes divergence between runs [[forum](https://discussion.nextstrain.org/t/728)]
- **Gaps do not count as divergence**: two strains looked identical in divergence view although one showed "Gaps: 7". Tree builders treat `-` as missing data, so a genome with a large deletion looks closer to the root [[#1124](https://github.com/nextstrain/auspice/issues/1124)] [[#1500](https://github.com/nextstrain/auspice/issues/1500)]

### Grid and ticks

- **Numeric step**: `10^floor(log10(range))`, divided by 5 or 2 for short ranges, with 4 minor ticks per major step (5 for steps of exactly 5 or 10) [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/grid.js#L49-L106)]
- **Calendar-aware temporal grid (2020)**: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/grid.js#L108-L216)] [[#846](https://github.com/nextstrain/auspice/pull/846)] [[#1229](https://github.com/nextstrain/auspice/pull/1229)]
  - major lines are a whole number of calendar units: day, week, month, year, 5 years, decade, or century
  - the unit is chosen from the available pixels, with at least 130 px between lines below 1000 px axis width and 180 px above
  - some counts are disallowed: no 4-year, 7-month, or 2-week steps
  - the real time between lines is slightly uneven because months differ, which was judged intuitive
- **Calendar dates, not epi weeks**: a 2019 proposal for epidemiological week labels found that "week" is ambiguous (ISO week versus CDC MMWR week), and that situation reports show no consensus. Labels stay `YYYY`, `YYYY-Mon`, or `YYYY-Mon-DD` [[#804](https://github.com/nextstrain/auspice/pull/804)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/dateHelpers.js#L180-L210)]
- **Browser freeze on narrow axes**: an axis narrower than 130 px got zero intervals, an infinite step, and a date loop that never ended. Small windows, the grid layout, and long tip labels triggered it. It was open from 2021 to 2026 and is fixed by forcing at least one interval [[#1363](https://github.com/nextstrain/auspice/issues/1363)] [[#2098](https://github.com/nextstrain/auspice/pull/2098)]
- **Radial grid**: full circles centred on the root at `position - rootDepth`. The first circle could fall off-centre until a 2023 fix, whose author did not know why the position fell below the domain [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/grid.js#L265-L325)] [[#1693](https://github.com/nextstrain/auspice/pull/1693)]
- **Axis titles exist for rect, clock, and scatter only**: an unlabelled axis was called a common source of misreading [[#942](https://github.com/nextstrain/auspice/pull/942)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/grid.js#L390-L428)]
- **Date window shading**: on the time metric in the rectangular layout, `#EEE` rectangles shade the time outside the date slider [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/grid.js#L444-L531)]
  - each rectangle is drawn only if wider than 30 px
  - hidden in focus mode, mirrored for the right tree

### BCE dates and early CE dates

- **BCE dates**: negative decimal years are supported for deep phylogenies (TB, plague, ancient DNA) [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/dateHelpers.js#L27-L59)] [[#627](https://github.com/nextstrain/auspice/pull/627)]
  - they show as a rounded year without month or day
  - axis labels read "-undefined" until 2021 [[#1297](https://github.com/nextstrain/auspice/pull/1297)]
  - native date pickers cannot go before `0001-01-01`, so BCE bounds show as read-only text [[#1412](https://github.com/nextstrain/auspice/pull/1412)]
- **Years 0-99 CE become 1900-1999**: JavaScript `new Date(y, ...)` maps two-digit years to the 20th century [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/dateHelpers.js#L125-L140)]
  - a CCHF tree with root date 17.6 broke. The root date was fixed with `setFullYear` in 2024 [[#1892](https://github.com/nextstrain/auspice/issues/1892)] [[#1909](https://github.com/nextstrain/auspice/pull/1909)]
  - the slider still jumps 1900 years in that range (open) [[#1476](https://github.com/nextstrain/auspice/issues/1476)]
  - the grid helper still uses `new Date(year, ...)` for month, year, five-year, and decade steps
- **Negative fractions**: for negative values `%1` is negative, so `-500.3` resolves into year -501 [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/dateHelpers.js#L15-L25)]
- **Local time versus UTC**: one direction of the conversion uses local time and the other UTC. The noon anchor absorbs the daylight-saving hour [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/dateHelpers.js#L83-L90)]
- **Dual date state**: Redux stores each bound as a calendar string and as a number, and the two drift. CE URL dates must be full `YYYY-MM-DD`, so `?dmin=1000` selects nothing (open) [[#1477](https://github.com/nextstrain/auspice/issues/1477)]

### Clock regression

- **Forced through the root, by design since 2016**: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/regression.ts#L8-L52)] [[`b4b74a86`](https://github.com/nextstrain/auspice/commit/b4b74a86)]
  - slope `Σ y(x - x0) / Σ (x - x0)²` over visible tips, with `x0` the root date
  - the line passes through date = root date and divergence = 0, which assumes a root divergence of 0 (true for augur output)
  - no R² is reported. The scatter layout fits a free intercept and reports R²
- **The rate is Auspice's own, not TreeTime's**: Bedford proposed to show the rate from `augur refine`, with this regression as a fallback. Only the fallback exists [[#858](https://github.com/nextstrain/auspice/issues/858)]
  - augur now exports the rate's standard deviation, which Auspice does not read [[augur#1284](https://github.com/nextstrain/augur/pull/1284)]
  - a forum maintainer calls the clock view "quick and dirty", not for reporting [[forum](https://discussion.nextstrain.org/t/1455)] [[forum](https://discussion.nextstrain.org/t/227)]
- **Follows the filter since 2022**: R² and slope use visible tips only [[#1483](https://github.com/nextstrain/auspice/issues/1483)] [[#1484](https://github.com/nextstrain/auspice/pull/1484)]
  - Bedford used it to show clock signal dropping from R² 0.82 (2 years) to 0.33 (6 months): temporal signal depends on the sampling window
  - with zero visible tips the line disappears but the toggle stays on
  - the root anchor stays at the full tree's root date
- **No log scales** exist anywhere in the tree
- **Clock deviation as a coloring**: root-to-tip residuals, to spot misdated or misassigned samples, are an open augur request [[augur#1065](https://github.com/nextstrain/augur/issues/1065)]

### Confidence intervals

- **Date intervals as bars**: on the time metric in the rectangular layout, each node with `num_date.confidence` gets a semi-transparent bar from lower to upper date in the branch color [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/confidence.js#L41-L56)] [[#327](https://github.com/nextstrain/auspice/pull/327)]
  - width: 0 for 1 px branches, `2 * width` up to 6 px, and `width + 6` above
  - bars ignore the pointer, so they never steal branch hover
  - all bars are drawn at once, because figures need them
- **Hovering a branch shows its interval** when the global toggle is off [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/reactD3Interface/callbacks.ts#L112-L123)]
- **Toggle state bugs recur**: [[#1051](https://github.com/nextstrain/auspice/issues/1051)] [[#1577](https://github.com/nextstrain/auspice/pull/1577)] [[#1799](https://github.com/nextstrain/auspice/issues/1799)] [[#1978](https://github.com/nextstrain/auspice/issues/1978)]
  - the toggle appeared for divergence trees
  - it vanished after a layout round trip (open)
  - its presence was once detected on `nodes[0]` only, which broke for multi-tree datasets where node 0 is the synthetic root. The lesson recorded: derive "has intervals" from all nodes of all trees
- **Tight intervals are not trustworthy**: rate underestimation and time-dependent rates make deep extrapolation "pretty meaningless" despite narrow intervals [[forum](https://discussion.nextstrain.org/t/554)]
- **Node dates differ between gene trees**: HIV gag, pol, and env trees date the same ancestral events decades apart, which maintainers attribute to selection and time-dependent rates. This open science problem applies to comparing node dates across segment trees [[forum](https://discussion.nextstrain.org/t/1596)]
- **Trait intervals are never drawn**: numeric trait confidences appear only as text [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/infoPanels/hover.js#L117-L137)]

## Zoom, focus, and navigation

### Clade zoom

- **Click a branch to zoom into its clade**: zoom sets an in-view root, the axes refit to the in-view nodes, and the rest is clipped [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/reactD3Interface/callbacks.ts#L61-L98)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L362-L414)]
  - coordinates do not change in rect and radial. Unrooted recomputes its layout
  - observed: clicking a 22-tip Zika clade rescaled the time axis to 2014-2018, showed tip labels, kept the clade's stem at the left, and changed the header to "Showing 22 of 1037 genomes"
- **Zoom on a terminal branch** zooms to its parent's clade, except for a single-tip exploded subtree, whose parent is the synthetic root [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/tree.ts#L26-L70)]
- **Zoom out**: click the in-view root's branch, or the magnifier-minus button. The button is enabled only when zoomed and has no tooltip [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/treeButtons.tsx#L36-L75)]
- **"Zoom to selected"**: zooms to the common ancestor of the filtered tips [[#1132](https://github.com/nextstrain/auspice/issues/1132)] [[#1257](https://github.com/nextstrain/auspice/pull/1257)]
  - it cannot help when that ancestor is the root (early samples, basal clades)
  - it does not work for date filters (open) and does nothing for genotype filters [[#1556](https://github.com/nextstrain/auspice/issues/1556)] [[#1583](https://github.com/nextstrain/auspice/issues/1583)]
  - auto-zoom on every filter change was rejected as too jumpy, because users toggle country filters quickly to compare distributions in a fixed view [[#1240](https://github.com/nextstrain/auspice/issues/1240)]
- **"Zoom to root"**: formerly "Reset layout" [[#1952](https://github.com/nextstrain/auspice/pull/1952)]
- **Free pan and zoom were removed**: early Auspice had SVG pan and zoom and removed it as confusing. A request for arbitrary zoom was answered with "write a proposal", because of the edge cases [[#936](https://github.com/nextstrain/auspice/issues/936)]
- **Two sources of truth**: the zoom root lives in Redux and in `PhyloTree`. The two diverged when a zoomed URL loaded, so zoom out failed [[#1156](https://github.com/nextstrain/auspice/pull/1156)]
  - a 2026 issue lists five overlapping state concepts (`idxOfInViewRootNode`, `idxOfFilteredRoot`, `inView`, `visibility`, `focusNodes`) and proposes unifying them [[#2036](https://github.com/nextstrain/auspice/issues/2036)] [[#2027](https://github.com/nextstrain/auspice/issues/2027)]
- **No zoom history**: a "back to previous zoom" PR stalled in 2021, because history with two trees is ambiguous. Bedford preferred browser back and forward as a general undo [[#1403](https://github.com/nextstrain/auspice/pull/1403)] [[#1393](https://github.com/nextstrain/auspice/issues/1393)]
- **Very long names break zoom**: a tip name of several hundred characters made the label margin wider than the panel. The x range inverted and the tree flipped or vanished (open since 2022) [[#1497](https://github.com/nextstrain/auspice/issues/1497)]

### Zoom in the URL

- **Only labelled clades can be shared**: the zoom is written as `label=<key>:<value>`, from the first non-`aa` branch label of the in-view root, or as `treeZoom=selected`. Any other zoom is missing from the URL [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeVisibilityHelpers.js#L68-L90)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/middleware/changeURL.js#L198-L220)]
  - observed: after zooming into an unlabelled Zika clade, the URL query stayed empty
- **Why not node names**: a request to share any internal node (30 comments, open since 2020) was refused for names like `NODE_0002547`, because they change between builds and a stale link would silently show another clade [[#1036](https://github.com/nextstrain/auspice/issues/1036)] [[#1138](https://github.com/nextstrain/auspice/issues/1138)]
  - `label=mrca(tipA,tipB)`: always a clade in a rooted tree, but the tips may vanish
  - labelling nodes in the dataset
  - single-use node ids, so that stale links are ignored
- **Label lookup must be unique**: a missing or duplicated value shows a warning and does not zoom [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeVisibilityHelpers.js#L25-L64)] [[#886](https://github.com/nextstrain/auspice/pull/886)]
  - `aa` labels are excluded, because almost every branch has one and their values move between runs
- **`display_defaults.label`** sets an initial zoom (2025). As a side effect, no URL loads the full tree unless the root has a label [[#1952](https://github.com/nextstrain/auspice/pull/1952)]

### Focus on selected

- **An accordion y axis**: a toggle that gives visible tips 80 % of the height and squeezes the rest [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/helpers.ts#L179-L207)] [[#1373](https://github.com/nextstrain/auspice/pull/1373)]
  - the share is capped at `nVisible / 5` for fewer than four tips, and raised to `nVisible / nTotal` when most tips are visible
  - the tip after each visible tip also gets the focused spacing, because the allocation looked wrong with few selected tips
- **Terminology and design**: [[#1368](https://github.com/nextstrain/auspice/issues/1368)] [[#1993](https://github.com/nextstrain/auspice/pull/1993)] [[#2028](https://github.com/nextstrain/auspice/pull/2028)]
  - maintainers agreed that "zoom" changes the axis range and "focus" changes the scale within it
  - a toggle that always follows the active filters was chosen over a button, which created confusing intermediate states
  - non-linear time was judged a bad idea, but since 2026 focus also fits both axes to the focused nodes, so "in view" no longer means "on screen"
- **Origin**: proposed in 2018 as accordion drawing in the sense of Munzner, closed unmerged in 2021, and merged in 2024 [[#500](https://github.com/nextstrain/auspice/pull/500)]
- **Availability**: rect and radial only, mutually exclusive with stream trees, URL `focus=selected` [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/controls/toggle-focus.tsx#L14-L58)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L225-L236)]
- **Forum use**: focus is the maintainers' answer to badly scaled trees, for example a 1978 root above 2017 samples [[forum](https://discussion.nextstrain.org/t/1872)]

## Visibility, filters, and the date range

### Visibility model

- **Three states per node**: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/globals.js#L169-L171)]
  - not visible: thin branch, not on the map
  - visible to the map only: thin branch, counted on the map
  - visible: tree and map
- **Rule**: a node must be in view and pass the filters. Then: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeVisibilityHelpers.js#L216-L264)]
  - a node or parent without `num_date`, or any node of a divergence-only tree, is visible. Tips without dates are therefore never filtered out by date
  - a node whose own date is inside the window is visible
  - a node whose branch interval overlaps the window is map-only
  - every other node is not visible
- **Why map-only exists**: a branch that crosses the window boundary is one node object, and drawing only its in-window part was judged hard. The map needs the overlap rule, so that inferred migrations along a branch still animate [[#311](https://github.com/nextstrain/auspice/issues/311)] [[#639](https://github.com/nextstrain/auspice/pull/639)]
- **Non-visible parts are not grayed**: tips are hidden, and branches keep their color at 0.5 px without hover or click [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/reactD3Interface/callbacks.ts#L84-L108)]
- **The date filter disables backbone clicks**: with a minimum date, ancestral branches before the window cannot be clicked to zoom. Huddleston named the tension: accurate representation of the filter versus the click interface users rely on (open) [[#1405](https://github.com/nextstrain/auspice/issues/1405)]
- **Precision-aware date filtering was proposed in 2016**: filter on each node's date interval and display on the point estimate, by analogy with geographic precision. It was never built [[#6](https://github.com/nextstrain/auspice/issues/6)] [[#9](https://github.com/nextstrain/auspice/issues/9)]
- **Partially dated trees mislead**: undated tips always stay visible, so after a date filter a visible tip may be in range or undated. Hadfield's example: mpox filtered to before 2013 still shows many undated tips [[#1519](https://github.com/nextstrain/auspice/pull/1519)]

### Filters

- **Semantics**: values of one trait combine with OR, different traits with AND [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeVisibilityHelpers.js#L116-L214)] [[#1248](https://github.com/nextstrain/auspice/pull/1248)]
  - only tips are tested, and their ancestors become visible
  - ancestors above the common ancestor of the selection are then hidden again
  - Bedford's argument: "filtered" needs firm semantics, and a thick branch above the common ancestor would suggest that off-screen tips are selected. Neher asked to keep the branch leading into it
- **Internal-node traits are ignored**: filtering to a value annotated on internal nodes (a basal state lost repeatedly) shows only matching tips. Values that occur only on internal nodes cannot be filtered at all (open) [[#1275](https://github.com/nextstrain/auspice/issues/1275)] [[#1865](https://github.com/nextstrain/auspice/issues/1865)]
- **Genotype filters select nodes, not paths**: filtering by `HA1 198P` shows only the nodes that carry the state, without connecting branches (open) [[#1281](https://github.com/nextstrain/auspice/issues/1281)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeVisibilityHelpers.js#L285-L380)]
  - one maintainer called this "more truthful", since a visible root would imply that the root carries the state
  - another argued that the connecting subtree shows the transitions
  - two meanings of filtering coexist: evolutionary paths to selected tips, or the parts of the tree that match a query
  - states at one position combine with OR and positions with AND. "Mutation" was renamed "genotype", because a wild-type state is not a mutation [[#1265](https://github.com/nextstrain/auspice/pull/1265)] [[#1276](https://github.com/nextstrain/auspice/pull/1276)]
- **Continuous traits are not filterable**: range filters are open requests [[#1748](https://github.com/nextstrain/auspice/issues/1748)] [[#1402](https://github.com/nextstrain/auspice/issues/1402)]
- **Clicking a tip is a filter**: it opens the detail modal and adds a strain filter for that tip. Closing the modal restores the earlier filter state exactly [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/reactD3Interface/callbacks.ts#L96-L104)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/reducers/controls.ts#L458-L468)]
  - a 2021 fix left a trail of inactive filters. The 2024 fix records the state before the click [[#1357](https://github.com/nextstrain/auspice/issues/1357)] [[#1749](https://github.com/nextstrain/auspice/pull/1749)]
  - observed: clicking Zika tip ZJ03 showed "Showing 1 of 1037" and added `?s=KU820899` to the URL. Escape restored "22 of 1037" and an empty query
- **Strain search is a filter**: the sidebar search lists every tip as a "sample" entry, and the URL key `s` holds several strains [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/controls/filter.js#L116-L138)] [[#1200](https://github.com/nextstrain/auspice/pull/1200)]
- **Highlighting without filtering**: maintainers recommend a boolean coloring from a dropped TSV plus legend hover, because filtering hides the rest of the legend [[#1400](https://github.com/nextstrain/auspice/issues/1400)] [[forum](https://discussion.nextstrain.org/t/309)]
- **Context around a scattered selection**: public-health users want genetically close context for each separate introduction at once, not one zoom per introduction (open) [[#1241](https://github.com/nextstrain/auspice/issues/1241)]

### Date slider and animation

- **Range**: from the root date minus 0.01 years to the latest date plus 0.01 years. In a tanglegram the range covers both trees [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L32-L51)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L397-L422)]
- **Slider**: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/controls/date-range-inputs.js#L34-L115)] [[#1412](https://github.com/nextstrain/auspice/pull/1412)]
  - two handles that push each other, with a minimum span of 7.5 % of the range
  - updates at most every 100 ms while dragging, in a cheap "quickdraw" mode, and a full redraw on release
  - a native date picker per bound since 2022
- **Animation**: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/framework/animationController.js#L48-L115)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/middleware/changeURL.js#L221-L237)]
  - a 50 ms tick and a window of 7.5 % of the whole date range
  - durations of 15, 30, or 60 s, with optional cumulative mode and loop
  - tip labels are removed while it plays
  - URL `animate=start,end,loop,cumulative,duration`
- **Map circles appear before tree tips**: when the geographic trait is inferred on internal nodes, map circles count internal nodes and appear when the branch becomes visible (unresolved) [[forum](https://discussion.nextstrain.org/t/729)]

## Color, legend, and uncertainty

### Scale types

- **Dispatch**: the coloring's JSON `type` picks the scale: `temporal` (or `num_date`), `continuous`, a JSON-provided `scale` map, `categorical`, `ordinal`, or `boolean`. Genotype colorings are categorical [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L22-L135)]
  - any error during scale construction makes every node gray with a single "unknown" legend entry, so data errors show only in the console
- **Continuous**: a linear ramp over the minimum and maximum of all nodes of both trees [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L260-L306)]
  - outliers compress everything else into one or two colors (S1 mutations: most tips 5 to 10, a few at 50)
  - quantile scales are an open request [[#1454](https://github.com/nextstrain/auspice/issues/1454)]
- **Anchored continuous scales**: `scale: [[value, "#hex"], ...]` fixes colors to values, so that colors mean the same across builds. The first version was `vmin` and `vmax` for dengue titers in 2017 [[#412](https://github.com/nextstrain/auspice/pull/412)] [[#1340](https://github.com/nextstrain/auspice/pull/1340)]
- **Temporal**: the stops are the root date plus tip dates at equal rank spacing, so most color resolution goes where sampling is dense [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L309-L368)] [[#544](https://github.com/nextstrain/auspice/pull/544)]
  - before 2018, deep trees (dengue, Lassa) showed almost every tip in the last red box
  - Bedford called the sampling-weighted domain bespoke and asked to make it an option
- **Ordinal**: only for all-integer values [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L210-L258)] [[#726](https://github.com/nextstrain/auspice/issues/726)]
  - every integer between minimum and maximum gets a swatch, observed or not
  - above 37 values it falls back to continuous, and any non-integer value makes it categorical
  - ordered string categories ("small", "medium", "large") need a provided `scale`
- **Boolean**: `true`, `1`, and `yes` are blue. Every other valid value is yellow, typos included [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L500-L504)]
- **String numbers**: continuous values are cast to numbers at load since 2023, after negative ACE2 values stored as strings vanished from scatterplots. Non-numeric strings then disappear from info boxes [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/castJsonTypes.js#L1-L57)] [[#1626](https://github.com/nextstrain/auspice/issues/1626)] [[#1655](https://github.com/nextstrain/auspice/pull/1655)]
- **Hard-coded trait scales were removed**: LBI had a built-in range. The lesson recorded: do not special-case trait names in the viewer [[#1842](https://github.com/nextstrain/auspice/pull/1842)]

### Palette and stability

- **Hand-tuned ramps**: `colors[n]` holds an n-color ramp from purple and blue through green and yellow to red, for n up to 36. Above 36 values the colors repeat [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/globals.js#L82-L121)]
  - exactly 37 values once crashed the scale (fixed 2021) [[#1355](https://github.com/nextstrain/auspice/pull/1355)]
- **Colors depend on rank and count**: the categorical legend order assigns the colors [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/sortedDomain.js#L1-L15)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L400-L415)]
  - the order is by count over all nodes, ties alphabetical, and `clade_membership` alphabetical
  - a value therefore changes color between datasets, and internal-node reconstructions influence which value gets the first colors
  - a Firefox and Chrome difference once reversed the legend order, and users complained because they had learned the country colors [[#973](https://github.com/nextstrain/auspice/issues/973)] [[#979](https://github.com/nextstrain/auspice/pull/979)]
- **Colors stay fixed under filtering**: scales come from the whole tree, and only the legend shrinks [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L443-L485)]
- **No color-blind-safe palette**: the rainbow is hard to read with red-green color blindness. ColorBrewer was suggested, and nothing changed (open since 2020) [[#932](https://github.com/nextstrain/auspice/issues/932)] [[#974](https://github.com/nextstrain/auspice/issues/974)]
  - a "new colors" button that maximizes contrast among visible values is also open [[#1290](https://github.com/nextstrain/auspice/issues/1290)]
- **Rare values are drawn under common ones**: an open request asks to draw tips in inverse-frequency order [[#1362](https://github.com/nextstrain/auspice/issues/1362)]

### Branch color

- **A branch takes its child's color**: the stem of node n uses n's inferred state along its whole length, and the vertical bar of an internal node uses that node's state [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/renderers.ts#L272-L291)]
- **This implies timing**: a long red branch reads as "arrived at the start of the branch". Mumps in British Columbia looked like a 2013 arrival [[#896](https://github.com/nextstrain/auspice/issues/896)]
  - parent-to-child gradients were implemented in March 2020 [[#947](https://github.com/nextstrain/auspice/pull/947)]
  - they were removed in April 2020, because gradient branches did not render at all on Windows Firefox, Brave, and in narratives [[#1005](https://github.com/nextstrain/auspice/issues/1005)] [[#1042](https://github.com/nextstrain/auspice/pull/1042)]
- **Branches are always grayed a little**: the branch color is the node color mixed 60 % toward `#BBB`, and tips use a brighter fill of the node color [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorHelpers.js#L69-L120)]

### Uncertainty

- **Entropy turns branches gray**: when a coloring has confidences, branches blend toward gray by a power scale of the trait entropy [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorHelpers.js#L69-L115)] [[#256](https://github.com/nextstrain/auspice/issues/256)]
  - originally a branch was fully gray at 80 % confidence, because the reconstruction was over-confident
  - after augur added sampling-bias correction, full gray moved to about 50 %, described as "an honest accounting" [[#898](https://github.com/nextstrain/auspice/pull/898)]
  - tips got the same treatment in 2024, with a mapping that leaves certain tips unchanged [[#1796](https://github.com/nextstrain/auspice/pull/1796)]
- **No toggle**: confidence coloring turns on whenever any node carries a confidence for the current coloring [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L437-L467)]
- **Gray is overloaded**: these all look gray, and maintainers accepted the overlap (open) [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L164-L177)] [[#1797](https://github.com/nextstrain/auspice/issues/1797)]
  - unknown values (`#ADB1B3`)
  - values missing from a provided scale (grays from `#BDC3C6` to `#868992`)
  - low confidence (blend toward `#BBB`)
  - deliberate background categories
- **Marginals do not compose**: a prototype animated trait states resampled from the exported confidences (open draft) [[#1163](https://github.com/nextstrain/auspice/pull/1163)]
  - Bedford and Neher noted that the confidences are per-node marginals, so independent sampling yields impossible histories, such as French Polynesia to Thailand to French Polynesia
  - a correct display needs joint distributions of adjacent nodes, exported as branch attributes
- **HPD lists cannot be shown**: BEAST MCC trees carry per-node HPD sets that `augur export` cannot represent. Only `<trait>_confidence` and `<trait>_entropy` are recognized [[forum](https://discussion.nextstrain.org/t/1078)]

### Legend

- **Visible values only** for categorical and ordinal scales: since 2021 the legend lists the values present on visible tips of both trees [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L443-L485)] [[#1109](https://github.com/nextstrain/auspice/issues/1109)] [[#1150](https://github.com/nextstrain/auspice/pull/1150)]
  - the trigger: a user had hand-edited a country legend for a tweet
  - continuous and boolean legends stay complete, because a partial ramp would mislead
  - whether reconstructed ancestors should count was left open. Only tips count
- **Bins are half-open `(a, b]`**: a continuous legend entry owns the interval between the midpoints to its neighbors [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L506-L516)] [[#1643](https://github.com/nextstrain/auspice/pull/1643)] [[#1657](https://github.com/nextstrain/auspice/pull/1657)]
  - the outer bins reach to minus and plus infinity since 2023
  - hard-coded bounds of 0 and 10 000 had made negative fitness values never match the first swatch
- **Custom legends**: JSON `legend` entries with `display` text and `bounds`, for example a gray "no change" band from -0.1 to 0.1 [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L535-L619)] [[#1340](https://github.com/nextstrain/auspice/pull/1340)]
  - overlapping bounds are replaced by automatic ones
  - temporal colorings reject custom legends
- **Legend bins outside the data get the extreme color**, because the ramp domain is the observed range. Two datasets with the same config then color a bin differently (open, draft fix) [[#2014](https://github.com/nextstrain/auspice/issues/2014)] [[#2016](https://github.com/nextstrain/auspice/pull/2016)]
- **Interaction**: hover enlarges matching tips in both trees. Shift-click opens a color editor when the dataset editor is enabled [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/legend/item.tsx#L33-L41)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/legend/legend.tsx#L179-L193)]
- **Layout**: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/legend/legend.tsx#L76-L146)]
  - closed by default above 32 entries or below 600 px panel width
  - one or two columns, scrollable since 2026
  - no tip counts per entry (open request) [[#1990](https://github.com/nextstrain/auspice/issues/1990)]

### Genotype coloring

- **Reconstructed from branch mutations**: a walk from the root sets each node's state at the chosen positions [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/setGenotype.js#L3-L68)]
  - the ancestral state is the "from" allele of the first mutation seen, or the root sequence when present
  - the root sequence is needed for positions without mutations. It is a few kB for viruses and about 2 MB for bacteria, so it is a separate fetch [[#1197](https://github.com/nextstrain/auspice/pull/1197)] [[#1688](https://github.com/nextstrain/auspice/pull/1688)]
- **Legend order**: by the earliest date at which a genotype appears [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/setGenotype.js#L70-L104)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L180-L208)]
  - gap, N (nucleotides only, because N is asparagine in amino acids), and X come last in fixed grays
  - ten colors, repeated beyond ten alleles
- **Fixed residue colors versus per-view colors (unresolved)**: framed as "reading out genotypes" versus "evolutionary observation" (PR open since 2019) [[#739](https://github.com/nextstrain/auspice/pull/739)]
  - one side wanted each residue in the same color at every site (Taylor 1997)
  - the other preferred colors by abundance, so that the major clade keeps its color when stepping through sites
- **Multi-position genotypes** join states with `" / "`, so a combination that contains a gap is not gray [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/setGenotype.js#L60-L66)]

## Labels

### Tip labels

- **Threshold**: labels appear only when fewer than 75 tips are in view and visible. The numbers were tuned by eye in 2018 [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/labels.js#L17-L45)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/defaultParams.ts#L37-L45)] [[#557](https://github.com/nextstrain/auspice/pull/557)]
  - font size: 12 px below 25 tips, 10 px below 50, else 8 px, counted over all in-view tips
- **Visible versus in-view**: a filter that left a few tips of a big tree showed no labels, so the show rule now counts visible tips. Font size and the reserved margin still count all in-view tips [[#915](https://github.com/nextstrain/auspice/issues/915)] [[#1043](https://github.com/nextstrain/auspice/pull/1043)]
  - observed: `zika?f_country=Fiji,China` keeps 13 of 1037 tips. The labels appear at 8 px, and the 12 Chinese tips, which share nearly the same y, pile into an unreadable clump
- **No collision handling**: overlap was reported in 2020. Only the cheap fix landed: labels below tips, ignoring the pointer [[#1082](https://github.com/nextstrain/auspice/issues/1082)] [[#1085](https://github.com/nextstrain/auspice/pull/1085)]
- **Users think labels are broken**: they pick a label field on a large tree and see nothing. A "show all" toggle or a message about the limit is open since 2020 [[#916](https://github.com/nextstrain/auspice/issues/916)] [[forum](https://discussion.nextstrain.org/t/1676)]
- **Reserved margin**: `0.65 * longest label in characters * font size` px, an assumed glyph width. Text is never measured [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L630-L655)] [[#1689](https://github.com/nextstrain/auspice/issues/1689)]
  - until 2026 the code followed the first tip's name and cut off longer ones [[#2099](https://github.com/nextstrain/auspice/pull/2099)]
  - it also measured strain names when another label field was shown
- **Label by any attribute**: a dropdown since 2021, for any node attribute since 2024 [[#1246](https://github.com/nextstrain/auspice/pull/1246)] [[#1668](https://github.com/nextstrain/auspice/pull/1668)]
  - pipelines can store a unique accession in `name` and show a readable strain name via `display_defaults.tip_label`. The hover box then shows both
  - `none` hides labels [[#1618](https://github.com/nextstrain/auspice/pull/1618)]
- **One field only**: a format string such as `{region} / {location}` was suggested. The single-key state and URL design block it [[#1369](https://github.com/nextstrain/auspice/issues/1369)] [[#1963](https://github.com/nextstrain/auspice/issues/1963)]
- **Same position in every layout**: 8 px right and 2 px down from the tip, never rotated, and not mirrored on the right tree of a tanglegram [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/labels.js#L38-L39)]

### Branch labels

- **Show rule**: a label shows when its node is visible and either of these holds. The rule went through four versions from 2018 to 2022 [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/labels.js#L82-L98)] [[#908](https://github.com/nextstrain/auspice/pull/908)] [[#1249](https://github.com/nextstrain/auspice/pull/1249)] [[#1501](https://github.com/nextstrain/auspice/pull/1501)] [[#1503](https://github.com/nextstrain/auspice/pull/1503)]
  - its visible tips exceed 3 % of the visible tips under the in-view root, so zooming in reveals the labels of smaller clades
  - the node roots a dataset tree or an exploded subtree (not for `aa`)
- **Styles**: `aa` labels 10 px weight 500, others 14 px weight 700, anchored 5 px left of the node above the branch [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/labels.js#L55-L71)]
- **"Show all labels"**: added for reference trees where every lineage must be visible [[#1554](https://github.com/nextstrain/auspice/pull/1554)]
  - since 2025 it applies only to visible branches, because users could drop filters to see everything but not the reverse [[#2004](https://github.com/nextstrain/auspice/pull/2004)]
  - no middle setting exists: default shows 2 labels, "all" about 100, and users want 10 to 20 [[#1624](https://github.com/nextstrain/auspice/issues/1624)]
  - it cannot be set from the dataset [[#1676](https://github.com/nextstrain/auspice/issues/1676)]
- **Hidden on filtered-out branches** since 2021: one maintainer used them as landmarks, another found text "hovering in thin air". Hiding won, matching the legend [[#1335](https://github.com/nextstrain/auspice/issues/1335)] [[#1370](https://github.com/nextstrain/auspice/pull/1370)]
- **Labels ignore the pointer**, after label text made branches unclickable [[#1580](https://github.com/nextstrain/auspice/issues/1580)] [[#1584](https://github.com/nextstrain/auspice/pull/1584)]
- **Auto-generated nucleotide labels** (like `aa`) and mask-aware labels are open ideas. Maintainers prefer computing labels from `branch_attrs.mutations` in the viewer [[#1736](https://github.com/nextstrain/auspice/issues/1736)]

## Hover, click, and detail views

### Pointer handling

- **Mouse only**: `mouseover`, `mouseout`, and `click` on tips, stems, bars, and vaccine crosses. No touch handling, drag, wheel, or pan exists [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/renderers.ts#L154-L363)]
- **Hit area equals the drawn stroke**: dense regions are hard to hit, for example a thin branch in a dense SARS-CoV-2 region (open) [[#1111](https://github.com/nextstrain/auspice/issues/1111)] [[#1407](https://github.com/nextstrain/auspice/issues/1407)]
  - remedies discussed: hiding clades, vertical zoom, rectangle zoom, lasso
  - the team chose "zoom to selected" and focus instead
- **Non-visible nodes ignore the pointer**, and labels and confidence bars never capture it [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/reactD3Interface/callbacks.ts#L84-L133)]
- **Narrative mode** locks clicks and hides the zoom buttons. Hover still works [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/reactD3Interface/callbacks.ts#L98)]

### Hover

- **Highlight**: tip hover enlarges the tip by 4 px. Branch hover recolors the hovered stem and bar (saturation times 1.8, lightness divided by 1.2). No ancestor or descendant path is highlighted [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/reactD3Interface/callbacks.ts#L84-L128)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorHelpers.js#L126-L131)]
- **Box placement**: in the half of the panel opposite the node, 200 to 280 px wide by panel width. The box never clamps to the panel, so boxes near the bottom are cut off (open) [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/infoPanels/hover.js#L319-L376)] [[#1356](https://github.com/nextstrain/auspice/issues/1356)]
- **Branch box**: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/infoPanels/hover.js#L178-L270)]
  - descendant count, divergence, inferred date with interval, and the current coloring with confidence
  - mutations on the branch by category, at most 5 nucleotide mutations and 3 per protein for 7 proteins
  - the hints "Click to zoom into clade" and "Shift + Click to display more info"
  - observed on a Zika branch: "Number of descendants: 22", "No nucleotide mutations", "Divergence: 7.999e-3", "Inferred Date: 2013-05-23", "Date Confidence Interval: (2013-01-19, 2013-07-11)", and "country (confidence): French Polynesia (100%)"
- **Tip box**: root-to-tip change counts instead of lists. Observed: "Nucleotide changes: 119 + 1 reversions to root", "Amino Acid changes: 11", and "Divergence: 0.0106" [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/infoPanels/hover.js#L145-L168)]
- **Hover content follows the coloring**: coloring by an amino acid hides the clade line. Configurable hover fields are an open request [[forum](https://discussion.nextstrain.org/t/1490)] [[#1729](https://github.com/nextstrain/auspice/issues/1729)]
- **Divergence formatting**: 3 decimals above 1, 4 above 0.01, else 3-digit exponential. A divergence of 0 is hidden because it is falsy [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/helpers.ts#L257-L263)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/infoPanels/hover.js#L43-L49)]

### Click and shift-click

- **Branch click zooms, shift-click opens the branch modal** with the full mutation table [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/reactD3Interface/callbacks.ts#L135-L168)] [[#1420](https://github.com/nextstrain/auspice/pull/1420)]
  - shift-click was "the simplest for now" in 2021, after users could not read all mutations of long branches [[#1417](https://github.com/nextstrain/auspice/issues/1417)]
- **Terminal branch and tip are different objects for users**: branch hover summarises the branch's own mutations, and tip hover summarises the sample relative to the root [[#1752](https://github.com/nextstrain/auspice/issues/1752)] [[#1753](https://github.com/nextstrain/auspice/pull/1753)]
  - a 2024 refactor merged the two, and users who triage singleton mutations daily lost hours. It was reverted within days
- **Modal rows**: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/infoPanels/click.js#L57-L297)]
  - dates (collection or inferred, interval, provided date) and publication
  - every remaining trait with its confidence, and links (GISAID, GenBank, per-trait URLs)
  - the mutation table with a copy-as-TSV button
  - the internal node name, always, because users cross-reference node-data files [[#1849](https://github.com/nextstrain/auspice/pull/1849)]
- **Clickable mutations (2025)**: click colors by that position, shift-click adds or removes the position, and cmd-click filters by the state [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/infoPanels/MutationTable.js#L69-L111)] [[#2018](https://github.com/nextstrain/auspice/pull/2018)]

### Mutation semantics

- **Categories**: undeletion, gap, N, homoplasy, and unique, where the first match wins. "Reversion to root" overlaps them [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeMiscHelpers.js#L170-L213)] [[#1444](https://github.com/nextstrain/auspice/issues/1444)] [[#1449](https://github.com/nextstrain/auspice/pull/1449)]
  - motivated by SARS-CoV-2 reference backfilling, where missing coverage filled with reference bases creates false reversions
  - users expected exclusive categories. Maintainers kept the overlap and added an explainer
- **Homoplasy is an exact string match**: `gene:A123T` counts as homoplasic when it appears on more than one branch of the left tree [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeJsonProcessing.ts#L155-L173)]
  - `G123T` at the same site does not count
  - counts are computed once at load, so filtering does not change them
  - in a tanglegram the right tree's branches are classified with the left tree's counts [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/tree.tsx#L174)]
- **Root-to-tip collapsing**: A>B then B>C shows as A>C, and A>B then B>A as a reversion. A real flu reversion (K171N then N171K) had shown both mutations [[#1280](https://github.com/nextstrain/auspice/pull/1280)]
- **Undeletions** (gap to base) get their own category, because they are almost always artefacts. Any `-` to base counts, even when the base differs from the ancestor [[#1542](https://github.com/nextstrain/auspice/pull/1542)]
- **Gaps and Ns** are excluded from nucleotide entropy and events but shown in hover. Amino-acid X is dropped in hover but not in the modal [[#552](https://github.com/nextstrain/auspice/pull/552)] [[#895](https://github.com/nextstrain/auspice/pull/895)]
- **Ns are never shown in the tip box**: the code counts them as `ns` and reads `nt` [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/infoPanels/hover.js#L148-L161)]
- **Entropy and events disagree when zoomed**: mutations on the zoomed subtree's root branch count as events but give zero entropy (open) [[#2010](https://github.com/nextstrain/auspice/issues/2010)]

## Two trees: second tree and tanglegram

### Loading and pairing

- **URL syntax**: `<datasetA>:<datasetB>`, both full paths, for example `flu/seasonal/h3n2/ha/2y:flu/seasonal/h3n2/na/2y`. A third part is ignored [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/loadData.js#L15-L29)] [[#792](https://github.com/nextstrain/auspice/pull/792)]
  - since 2019. The earlier `ha:na/2y` form could not compare `ha/2y` with `na/3y`
- **Each side is an ordinary dataset**: no second-tree format exists [[#534](https://github.com/nextstrain/auspice/pull/534)] [[augur#168](https://github.com/nextstrain/augur/issues/168)]
  - the pipeline must subsample both segments to the same strains and copy labels across, such as flu HA clade labels onto NA
  - a tanglegram is only as informative as the overlap of its tip sets
- **Candidate list**: `auspice view` offers datasets with the same first path part and the same number of parts that differ in exactly one part [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/cli/server/getDatasetHelpers.ts#L114-L143)]
  - on nextstrain.org, 149 datasets list candidates: seasonal flu HA and NA, avian flu segments, Lassa, Oropouche, hantaviruses, dengue genome against E, and enterovirus D68 genome against VP1
  - `rabies` lists itself three times as its own candidate
  - community and Groups builds get no list, so users type the URL by hand [[forum](https://discussion.nextstrain.org/t/732)] [[forum](https://discussion.nextstrain.org/t/771)] [[forum](https://discussion.nextstrain.org/t/1823)]
- **The concept is broader than segments**: maintainers kept pairing general, because comparing time windows or dataset versions (Zika in June versus September) is useful. The feature was renamed from "tangle tree", because candidates may share no tips [[#776](https://github.com/nextstrain/auspice/issues/776)] [[#792](https://github.com/nextstrain/auspice/pull/792)]
- **The left dataset's metadata governs both trees**: colorings, titles, panels, and display defaults of the right JSON are ignored [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L1149-L1164)]

### Tip matching

- **Exact name equality, visible in both trees**: a map from right-tree tip names to indices pairs each left tip [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeTangleHelpers.js#L3-L24)]
  - internal nodes are never linked
  - unmatched tips get no line and no warning
  - for duplicate names the last right tip wins
- **Case matters**: a user saw lines on only a few tips, because one tree had lower-case strain names [[forum](https://discussion.nextstrain.org/t/1615)]
- **Filters and the date slider remove lines**: only fully visible tips are linked, so tips outside the time window (thin map-only branches) lose their line [[#639](https://github.com/nextstrain/auspice/pull/639)]
- **No matched or unmatched count**: the right tree's tip count is a commented TODO [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L1160-L1161)]

### Lines

- **Z-shaped**: horizontal from the left tip to the gap, diagonal across a fixed 100 px gap, and horizontal to the right tip, so crossings stay inside the gap. The first version (2018) drew straight tip-to-tip lines [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/tangle/index.js#L6-L11)]
- **Width by link count**: 1 px, 0.5 px above 100 links, 0.25 px above 750 [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/tangle/index.js#L34)]
  - observed: the H3N2 HA/NA view with about 3000 links shows a handful of visible lines, and the rest are nearly invisible
- **Color from the left tip**: a tip with different values in the two trees shows the left value on its line [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/tangle/index.js#L45)]
- **Not interactive**: the line layer ignores the pointer, and hovering a tip highlights neither its line nor its partner [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/tangle/index.js#L95)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/reactD3Interface/callbacks.ts#L14-L24)]
- **Full redraw on every tree change, without transitions**: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/tree.tsx#L118-L121)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/tangle/index.js#L32-L81)] [[`f0e7b4c2`](https://github.com/nextstrain/auspice/commit/f0e7b4c2)]
  - the tree component tells the tangle to redraw whenever either tree changes, and the transition code is commented out
  - the lines therefore jump to their final positions while tips still move for 500 ms. This was chosen in 2018 so the lines cannot drift out of sync
  - after a resize the lines are removed and redrawn 1 s later
- **Toggle**: "Show Tanglegram" in the sidebar, with no URL key [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/controls/toggle-tangle.js#L10-L22)]

### Layout of the two trees

- **Each tree gets `(width - 100) / 2`**, and the right tree is mirrored, with its root on the right and tips facing the gap [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/tree.tsx#L22-L24)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/reactD3Interface/initialRender.ts#L35-L44)]
- **Independent axes**: each tree fits its own x and y domains [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L376-L414)] [[#524](https://github.com/nextstrain/auspice/issues/524)]
  - the same date sits at different x positions, and tip spacing differs when tip counts differ
  - observed: in the HA/NA view the left time axis starts at 2017, and the mirrored right axis runs from 2023 at the gap to 2015 at the right edge
  - the 2018 design shared only the slider range ("modify absoluteMinDate if tree2 root is older")
- **Forced settings**: rectangular layout, full panel width, and no map, entropy, or frequencies panels. The layout, explode, stream-tree, and confidence controls are hidden [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L1170-L1186)]
  - observed: the sidebar showed "Streamtrees unavailable" and no layout buttons
- **Tip labels are not mirrored**: on the right tree they extend rightward over its own branches, and they captured the pointer until 2024. The margin reserved for them sits on the root side [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/labels.js#L38)] [[#1783](https://github.com/nextstrain/auspice/pull/1783)]
- **Titles**: dataset names above each tree at a hand-measured offset. Long names cover the narrative button (open) [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/tangle/index.js#L82-L92)] [[#1447](https://github.com/nextstrain/auspice/issues/1447)]

### Interaction across trees

- **Zoom is per tree**: clicking a branch zooms only its tree [[#1772](https://github.com/nextstrain/auspice/issues/1772)] [[#2026](https://github.com/nextstrain/auspice/pull/2026)] [[#1965](https://github.com/nextstrain/auspice/issues/1965)]
  - since 2026 each tree has its own zoom buttons, which accepts two clicks for a joint zoom
  - zooming one tree can break or freeze the other (open)
  - only the left tree's zoom can be stored in the URL
  - no action zooms the other tree to the corresponding clade
- **Filters and tip clicks act on both trees by name**: clicking a tip filters both trees to that strain [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/reactD3Interface/callbacks.ts#L26-L34)]

### Coloring across trees

- **One scale over both trees**: discrete values and numeric ranges come from the union [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L22-L76)]
  - a 2018 bug recomputed the scale with an extra "unassigned" value from the second tree but did not repaint the left tree, so linked tips differed slightly in color [[#663](https://github.com/nextstrain/auspice/issues/663)] [[#665](https://github.com/nextstrain/auspice/pull/665)]
- **Values present only in the right tree become gray**: when the left dataset provides a JSON scale, extra values receive the grays used for unlisted values [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L164-L177)]
  - observed: `flu/seasonal/h3n2/ha/2y:flu/seasonal/h3n2/na/2y` colored by subclade shows every NA tip gray, because NA subclades (B.4.3 and others) are missing from the HA scale
- **Genotype coloring of the right tree is lost**: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L817-L852)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L1154)]
  - a 2024 fix colors the right tree when both genome maps are identical [[#1785](https://github.com/nextstrain/auspice/pull/1785)]
  - one day later a refactor passed a shallow copy of the metadata and discarded it, so the flag never reaches the state [[#1788](https://github.com/nextstrain/auspice/pull/1788)]
  - observed: `flu/seasonal/h3n2/ha/2y:flu/seasonal/h3n2/ha/6y?c=gt-HA1_158` colors the left tree in five genotypes and the whole right tree gray. The lines carry the left colors, and the console shows no "different genome_annotations" warning
- **Segments cannot share genotype coloring** by nature: each segment has its own coordinates, so a shared coloring must be a tip trait [[#1785](https://github.com/nextstrain/auspice/pull/1785)]

### Untangling

- **Switched off since 2018**: `attemptUntangle = false`, so crossings in the deployed app are not minimized at all [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/globals.js#L34)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/tree.tsx#L58-L66)]
- **The dormant heuristic (Neher)**: one greedy pass over the right tree's internal nodes [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/tangle/untangling.js#L11-L84)] [[#573](https://github.com/nextstrain/auspice/pull/573)]
  - reverse the children, recompute display order below the node, and keep the flip if the Pearson correlation of matched tips' positions does not drop
  - about 500 ms for a flu tree. H3N2 HA versus NA correlation rose from 0.75 to 0.82
  - polytomies get only full reversal. A better polytomy sort was tried "without much success"
  - it mutates the child arrays of the shared state in place
- **Why it was disabled**: [[#577](https://github.com/nextstrain/auspice/pull/577)]
  - Bedford: untangling distorts the second tree's own clade structure (the sister relation of NA clade A2/re to A1a was no longer visible) and still leaves many crossings
  - the objective should be the number of crossings, cleaning up clades near the tips rather than the root
  - a sidebar toggle defaulting to off was suggested and never built
- **Abandoned alternative**: a commented-out barycenter approach, where each node gets the mean normalized y of its tips in the other tree [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/tangle/unusedUntanglingCode.js#L1-L69)]
- **The docs still say crossings "usually" indicate reassortment**. With untangling off, many crossings are layout artifacts [[doc](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/docs/advanced-functionality/second-trees.rst#L11)]

### Untangling in the literature

- **Complexity**:
  - with one tree fixed, crossing minimization is solvable exactly in O(n log n), and the displacement (Spearman footrule) objective in O(n²) ([Venkatachalam et al. 2010](https://doi.org/10.1109/TCBB.2010.57))
  - with both trees free, the problem is NP-complete ([Fernau et al. 2010](https://doi.org/10.1016/j.jcss.2009.10.014)), with no constant-factor approximation for binary trees under the Unique Games Conjecture ([Buchin et al. 2012](https://doi.org/10.1007/s00453-010-9456-3))
  - Auspice has the easy one-sided case, yet used a greedy proxy
- **Displacement-optimized tanglegrams** ([Huson 2026](https://doi.org/10.1093/molbev/msag066)), implemented in SplitsTree 6:
  - minimize taxon displacement and, for networks, the vertical span of reticulation edges
  - try all permutations at nodes with up to 8 children, and swaps under simulated annealing above
  - one-sided and two-sided modes, with multifurcations and missing taxa supported
  - keep clades contiguous
- **Other implementations**:
  - Dendroscope's NN-tanglegram orders both sides from a Neighbor-Net circular order and works for networks ([Scornavacca et al. 2011](https://doi.org/10.1093/bioinformatics/btr210))
  - `phytools::cophylo` minimizes squared rank differences with greedy rotations, tries all child orders at polytomies with `rotate.multi`, and accepts partial and one-to-many associations ([source](https://github.com/liamrevell/phytools/blob/221cb9aab06c15bf7ba4f7765b4d28c5168fdd71/R/cophylo.R))
  - `dendextend::untangle` offers one-sided and alternating two-sided steps, measured by "entanglement" ([docs](https://rdrr.io/cran/dendextend/man/untangle.html))
  - shuffle methods reach zero entanglement where stepwise methods stall ([Nguyen et al. 2022](https://doi.org/10.1093/bioadv/vbac014))
- **Chains of N trees**: baltic untangles a list of trees against their neighbours ([source](https://github.com/evogytis/baltic/blob/e357bd55ec835d258074865d4b44d45aae2359db/baltic/bt_utils.py#L1813-L1988), [8-segment influenza B example](https://phylo-baltic.github.io/baltic-gallery/examples/influenza-b-virus-tangled-chain.html))
  - it tries all child permutations up to 8 children and normalizes each tree's y to [0, 1]
  - it draws lines only between neighbouring trees
  - nodes with a child that shares fewer than 2 tips are skipped, so cherries keep their order
  - it warns that parallel lines do not imply congruence
- **Crossings are a weak congruence signal**: after untangling, entanglement explains about 40 % of the variance in Robinson-Foulds distance between 20-leaf trees, and very different trees can draw with zero crossings. The author recommends colors and line styles that mark agreement ([de Vienne 2019](https://doi.org/10.1093/molbev/msy196))
- **Edge bundling** joins two facing hierarchies with bundled edges that emphasize splits, joins, and relocated subtrees ([Holten and van Wijk 2008](https://doi.org/10.1111/j.1467-8659.2008.01205.x)). Auspice only thins its lines

### More than two trees

- **Refused**: users asked for all eight influenza segments and three bunyavirus segments. The answer was "Three trees isn't possible, and probably never will be", with a pointer to static baltic figures [[forum](https://discussion.nextstrain.org/t/1501)] [[forum](https://discussion.nextstrain.org/t/771)]
- **No issue asks for MCC or reassortment coloring** in the Auspice tracker (searched 2026-10-07)

### Reassortment display in Nextstrain workflows

- **seasonal-flu (2023, now unused)**: a rule stack runs TreeKnit and `treetime arg` on the first two segments only ([rules](https://github.com/nextstrain/seasonal-flu/blob/f1af50727c0e846b9f9d623efeb7ed0bec477a12/workflow/snakemake_rules/treeknit.smk), [PR 105](https://github.com/nextstrain/seasonal-flu/pull/105))
  - it ran in the Europe profile and was removed from it in November 2024, so no public build uses it
  - the PR records that a Bioconda package of the Julia tool could not be built
- **MCCs as a categorical trait**: the workflow exports an `mcc` attribute, so both Auspice trees can be colored by MCC ([source](https://github.com/nextstrain/seasonal-flu/blob/48e29d6dfc0d0bf0f4466ecb2bf2dbe5e8302f8a/scripts/make-branch-length-json.py#L50-L77))
  - MCCs are sorted by size. Those with more than two tips get letters, and all smaller ones share `--`
  - `treetime.arg.assign_mccs` pushes the labels onto internal nodes
  - letters are not stable between runs, and the test `mi<=n_large_mccs` gives one extra MCC a letter
- **MCCs as metadata, not as structure**: for Omicron recombinants, maintainers preferred a "potential recombinant" coloring that users can explode on demand over separate subtrees, which would assert a specific set of events (open) [[ncov#915](https://github.com/nextstrain/ncov/issues/915)]
  - one maintainer proposed the same for TreeKnit MCCs of H3N2, because MCCs vary with run parameters
- **avian-flu avoids reassortment** ([README](https://github.com/nextstrain/avian-flu/blob/58bb3ca8dbf727dc340dc0fa6b297fd0e5b09317/README.md?plain=1)):
  - genome builds use GenoFLU constellations to partition data into non-reassorting sets
  - segment builds subsample each segment independently by default, so tip sets differ and many tips have no line
- **Oropouche**: the L/M tanglegram on nextstrain.org shows at least two recombination events ([blog](https://nextstrain.org/blog/2024-10-22-oropouche-analysis-and-resources))
- **Concatenated genomes need care**: frequent reassortment makes a whole-genome flu tree meaningless "without accounting for reassortment" [[forum](https://discussion.nextstrain.org/t/1807)]
- **TreeTime ARG mode**: `treetime arg` dates both segment trees jointly, using the concatenated alignment on branches inside an MCC, and writes one timetree per segment without a joint ARG file ([source](https://github.com/neherlab/treetime/blob/85a26f33f13dcf20e9f31ebf852731a386762d29/treetime/arg.py)). The shared time scale inside an MCC supports aligned time axes

## Exploded trees, multiple trees, and stream trees

### Exploded trees

- **Split by a trait**: every child whose trait value differs from its parent's becomes a new subtree under `__ROOT`, including changes to or from undefined [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/tree.ts#L391-L506)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/helpers.ts#L368-L381)]
  - original children are stored for undo
  - subtrees are sorted by legend order
  - each subtree's stem keeps its length from the original parent, so subtrees appear shifted rather than rooted at zero
- **Origin**: phylogeographers wanted to read the number and size of introductions [[#1008](https://github.com/nextstrain/auspice/issues/1008)] [[#1442](https://github.com/nextstrain/auspice/pull/1442)]
  - Ebola West Africa: many small Guinea subtrees versus one large Sierra Leone subtree
  - MERS: camel-to-human jumps that never spread further
  - the design discussion also proposed collapsing single-child chains and dropping "hanging" intermediate states without sampled tips
- **Limits (open)**: [[#1460](https://github.com/nextstrain/auspice/issues/1460)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/tree.ts#L439-L461)]
  - zoom to selected fails across subtrees, there is no URL state, and explode resets zoom
  - internal branches whose children all moved are drawn as extra tips
  - subtrees in radial and unrooted layouts overlap when their origins are similar
  - explode and tanglegram exclude each other
  - the subtree comparator is not a consistent ordering
- **Major and minor parents, designed, not built**: the multi-tree PR proposed a major parent (where a subtree attaches) and a minor parent (a recombination donor), drawn as solid and dashed connectors. Only the major parent for stems exists [[#1442](https://github.com/nextstrain/auspice/pull/1442)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeJsonProcessing.ts#L142-L153)]
- **Dashed branch styles** for recombinant branches are an open request, because line type is more accessible than color and marks a different mode of descent [[#1650](https://github.com/nextstrain/auspice/issues/1650)]

### Stream trees (2025, experimental)

- **A clade as a streamgraph**: a branch label partitions the tree, and each partition becomes stacked kernel-density ribbons, one per color category, over tip dates or divergences. The goal is trees with more tips than pixels [[doc](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/docs/advanced-functionality/streamtrees.rst#L7-L29)] [[#1902](https://github.com/nextstrain/auspice/pull/1902)]
- **One bandwidth for all streams**: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeStreams.ts#L91-L130)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeStreams.ts#L252-L288)]
  - reviewers objected that per-stream bandwidths made widths incomparable, so the final design uses one grid of 500 evaluation points and one sigma
  - a per-stream weight `exp(-(m - 4)/4) + 1` inflates streams with few tips, so areas are not comparable across streams, as the docs state
  - the docs say the grid extends 3 sigma, and the code uses 5
- **Stream order**: child streams are ordered by where they leave the parent and laid out post-order, to avoid connector crossings [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeStreams.ts#L297-L371)] [[#1969](https://github.com/nextstrain/auspice/issues/1969)]
  - the order differs between the time and divergence views
  - reviewers preferred fewer crossings over stable positions
- **Limits**: rect only, and no focus, explode, or second tree [[#1972](https://github.com/nextstrain/auspice/issues/1972)] [[#1970](https://github.com/nextstrain/auspice/issues/1970)]
  - a label whose clade has no own tips leaves a visual break
  - filtering leaves streams where their unfiltered siblings were
- **Collapsing identical sequences** (one tip per haplotype, with radius by count) was discussed as an alternative way to scale [[#1902](https://github.com/nextstrain/auspice/pull/1902)]

## Export

- **Newick**: "TimeTree (Newick)" or "Tree (Newick)" by metric [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/download/helperFunctions.js#L22-L54)]
  - branch lengths are differences of `num_date` (years) or `div`, without rounding, so values like `0.30000000000000004` occur
  - the export starts at the filtered root or the zoomed root, drops invisible nodes, and keeps unary nodes
  - internal names are omitted, and the root length is 0
  - names are not quoted, so names with spaces, commas, colons, or parentheses give invalid Newick
- **Disjoint selections fail**: date and genotype filters can leave several subtrees without a visible common ancestor, and the export then shows "Error saving tree!" [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/download/helperFunctions.js#L394-L408)] [[#1591](https://github.com/nextstrain/auspice/pull/1591)]
- **A pruned tree is not a re-inferred tree**: when filtered export was added for downstream tools, the decision recorded was that users must know this difference [[#1235](https://github.com/nextstrain/auspice/issues/1235)]
- **Nexus**: internal names plus BEAST-style `[&key=value]` annotations [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/download/helperFunctions.js#L64-L111)] [[#1245](https://github.com/nextstrain/auspice/pull/1245)]
  - annotations cover every coloring, confidence ranges as `{a,b}`, the current genotype, and divergence on time trees
  - characters `[]{}=,` and non-Latin text are stripped for FigTree
- **Only the left tree**: Newick, Nexus, metadata TSV, and the dataset JSON use the left tree, and the JSON export warns. The SVG screenshot is the only output with both trees and the lines [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/download/downloadButtons.js#L56-L84)] [[#630](https://github.com/nextstrain/auspice/pull/630)]
- **SVG fidelity (open)**: [[#1540](https://github.com/nextstrain/auspice/issues/1540)] [[#1337](https://github.com/nextstrain/auspice/issues/1337)] [[#2038](https://github.com/nextstrain/auspice/issues/2038)] [[#2102](https://github.com/nextstrain/auspice/issues/2102)] [[forum](https://discussion.nextstrain.org/t/1151)]
  - panels are serialized from the DOM without external CSS, so fonts differ
  - hidden elements stay in the file with inline `visibility: hidden`, which Inkscape ignores, so filtered tips and hidden labels reappear
  - the radial clip rectangle cuts circles, and legends are missing
  - the grid layout is never used, because the caller reads `panelLayout.panelLayout`, which is undefined [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/download/downloadButtons.js#L115)]
- **Data-use debate**: export of SVG and Newick in 2017 raised the concern that it would ease publishing trees from unpublished data. The decision was that Newick reveals no more than the JSON [[#296](https://github.com/nextstrain/auspice/issues/296)]
- **Dataset JSON rebuilt from state (2026)**: the current view becomes `display_defaults`. Whether filters should prune tips or be stored as state was discussed ("scientifically I prefer" keeping the tree) [[#2055](https://github.com/nextstrain/auspice/pull/2055)] [[#2000](https://github.com/nextstrain/auspice/issues/2000)] [[#2075](https://github.com/nextstrain/auspice/issues/2075)]

## Performance

- **SVG with d3 is the bottleneck**: a 2020 audit request says trees slow down at about 2000 tips on a laptop, the aim is about 5000, and showing more is a visualisation problem rather than a JavaScript one [[#955](https://github.com/nextstrain/auspice/issues/955)] [[#968](https://github.com/nextstrain/auspice/issues/968)]
  - painting and d3 updates dominate the cost
  - branches cost about 6 times as much as tips
- **Canvas and WebGL were never adopted**: [[#968](https://github.com/nextstrain/auspice/issues/968)]
  - a canvas prototype with an R-tree index for hit testing and a PixiJS prototype (about 7 times faster for solid branches) were not merged
  - maintainers valued d3 events and transitions and needed IE11 at the time
  - they noted that the change-diffing code exists only because overlapping d3 transitions are fragile
- **Animation off above 4000 tips**: the threshold was chosen by testing. Zooming a 23 000-tip tree went from about 2 s to about 1 s [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/middleware/performanceFlags.js#L23-L28)] [[#1880](https://github.com/nextstrain/auspice/pull/1880)]
  - transitions also drop to 0 ms when renders come less than 1 s apart, which the commit calls buggy, because a slow update looks like a quick follow-up [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/change.ts#L340-L345)]
- **Overlapping transitions corrupt the drawing**: [[#1904](https://github.com/nextstrain/auspice/issues/1904)] [[#1907](https://github.com/nextstrain/auspice/pull/1907)] [[#2077](https://github.com/nextstrain/auspice/pull/2077)]
  - skipping transitions entirely was faster, but a still-scheduled 500 ms transition then overwrote later updates (mis-scaled trees, stale genotype colors until hover)
  - 0 ms transitions with a forced flush came back
  - a legend hover during a zoom starts a transition that interrupts the zoom on those tips and freezes them (open fix)
- **Selective redraw (open PR)**: redraw only nodes whose visibility changed, which cut the work per animation tick from 383 ms to 84 ms on 35 000 tips [[#2079](https://github.com/nextstrain/auspice/pull/2079)] [[#2078](https://github.com/nextstrain/auspice/pull/2078)]
  - children of branches whose width changed must also be redrawn, because their start point depends on the parent's width
  - a render-equivalence suite checks that the incremental DOM equals a full redraw byte for byte
- **Creating SVG elements is costly**: branch hover adds and removes a confidence path, which made hover slow on a 23 000-tip tree [[#1878](https://github.com/nextstrain/auspice/issues/1878)]
- **Stack limits**: spreading a 277 000-element array into `push` overflowed Chrome's stack on a TB dataset [[#1293](https://github.com/nextstrain/auspice/pull/1293)]
- **Large files**: 600 to 750 MB pretty-printed datasets loaded in Firefox but not in Chrome. Augur now minifies JSON by default, and a size warning in Auspice is still open [[#1622](https://github.com/nextstrain/auspice/issues/1622)] [[#1926](https://github.com/nextstrain/auspice/issues/1926)]
- **Practical size**: the displayed tree is a subsample (about 4000 of 52 000 dengue genomes). Users with millions of sequences are sent to UShER and Taxonium [[forum](https://discussion.nextstrain.org/t/1902)] [[forum](https://discussion.nextstrain.org/t/1313)]
- **A password manager workaround**: the 1Password extension called `innerText` on up to four ancestors per element, so the tree is wrapped in four extra `<g>` levels [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/tree.tsx#L141-L154)] [[#1919](https://github.com/nextstrain/auspice/issues/1919)]

## Configuration surface

### URL keys for the tree

- **View**: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L54-L243)]
  - `l` (layout), `m` (metric), `c` (coloring, including `gt-<gene>_<positions>`), `ci` (confidence bars)
  - `tl` (tip label key), `branchLabel`, `showBranchLabels=all`
  - `label=<key>:<value>` (zoom), `treeZoom=selected`, `focus=selected`, `streamLabel`
  - `scatterX`, `scatterY`, `branches=hide`, `regression=show|hide`
- **Selection**: `f_<trait>=a,b`, `s=<strains>`, `gt=<gene>.<state>,...`, `dmin`, `dmax`, `animate` [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/middleware/changeURL.js#L102-L114)]
- **Page**: `d` (panels), `p` (full or grid, called "buggy" by the docs), `sidebar`, `legend`, `onlyPanels`, `lang`, `n` (narrative slide) [[doc](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/docs/advanced-functionality/view-settings.rst#L119)]
- **Absent**: no keys exist for explode, tip radius, branch width, the tanglegram toggle, the right tree's zoom, or an unlabelled zoom [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/middleware/changeURL.js#L41-L283)]
- **Self-healing URL**: a key is written only when it differs from the default, and invalid values are removed from the URL at load [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L511-L771)]
- **Commas break filters**: a filter value that contains a comma cannot be expressed (open) [[#1846](https://github.com/nextstrain/auspice/issues/1846)]

### Dataset defaults

- **`meta.display_defaults`**: `color_by`, `geo_resolution`, `distance_measure`, `layout`, `branch_label`, `label`, `tip_label`, `stream_label`, `transmission_lines`, `language`, `sidebar`, `panels`, and `map_triplicate` [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/metadataJsonParsing.ts#L77-L96)]
  - `layout` rejects `scatter`, although the docs list it
- **Not settable from the dataset**: focus, "show all labels", scatter axes (a code TODO), confidence bars, `treeZoom` (although the docs list it), and explode [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L751)]
- **Other meta fields**: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/metadataJsonParsing.ts#L38-L195)]
  - `colorings`, with `scale` and `legend`
  - `filters` (footer only), `panels`, `stream_labels`
  - `sharing`: download switches. GISAID data disables JSON and TSV downloads
  - `genome_annotations` and `root_sequence`
- **Precedence**: hard-coded defaults, then values derived from the tree, then `display_defaults`, then the URL, then validation with fallbacks [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L893-L924)]
  - initial zoom: `label`, then `treeZoom`, then `display_defaults.label`
  - the URL never changes a default, except `sidebar`
- **Fallback for an invalid coloring**: the dataset default, then `country`, then the first coloring, then a gray "none" [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L518-L563)]
- **Each narrative slide is a full view specification**: queryable state resets to the defaults before the slide's query applies. Slides can show tanglegrams, and tree clicks are locked [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L245-L283)] [[#1305](https://github.com/nextstrain/auspice/pull/1305)]

## Interaction with TreeKnit output

- **TreeKnit's `--auspice-view` JSON**: one `mcc_<a>_<b>` coloring per tree pair, with type `ordinal` and string values (see [TreeKnit in practice](treeknit-ecosystem.md)) [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L220-L258)]
  - the two labels are sorted, so both trees of a pair use the same key
  - strings fail the integer test, so Auspice warns and draws a categorical scale
  - nodes without an MCC get the string `"null"`, which is an ordinary category with its own color
- **Colors by rank, not identity**: the categorical legend orders MCC ids by node count [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/sortedDomain.js#L1-L15)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L487-L498)]
  - the largest MCC takes the first color, and colors change when the tree changes
  - above 36 MCCs the colors repeat
- **Tanglegram of two TreeKnit JSONs**: tip names are identical across trees, so every tip links
  - both files use the same coloring key and the same MCC numbers, so the shared scale gives one MCC the same color in both trees and on its lines
  - the right file's coloring metadata is ignored, which is harmless here
- **Resolved polytomies**: TreeKnit's added splits have zero length and draw as described in "Zero-length branches and polytomies". Nothing marks them
- **Ordering**: TreeKnit sorts the polytomies of the second tree so that leaves of one MCC face each other. Auspice keeps the JSON child order and does not untangle, so this ordering survives in the tanglegram

## Defects

Each item was found by reading the code at `dbf2875`. No upstream report was found unless one is linked. Items marked "observed" were reproduced in the live application.

### Geometry and axes

- **Inverted axis for tiny spans**: the minimum-span guard sets `maxX = 1e-8 - minX`, so a positive `minX` with a span below 1e-8 reverses the domain [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L402-L409)]
- **Clock regression through divergence 0**: the line passes through (root date, 0) instead of the root's divergence. This is harmless for augur output, which always starts at 0, and wrong for other inputs [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/regression.ts#L19-L29)]
- **Years 0-99 CE in the grid**: month, year, five-year, and decade grid starts still use `new Date(year, ...)` [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/dateHelpers.js#L125-L134)]
- **Tip zoom differs between Redux and the drawing class**: one handles single-tip exploded subtrees, and the other always zooms to the parent [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/tree.ts#L45-L52)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/change.ts#L430-L434)]
- **Exploded-away internal nodes become tips**: a node whose children all moved gets a tip circle on top of the previous tip [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/tree.ts#L428-L431)]
- **Margin and label rules count different tips**: the reserved margin uses all in-view tips and the label rule only visible ones, so labels can be drawn without a margin [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/layouts.ts#L640-L652)]

### Two trees

- **Right-tree genotype coloring lost** (observed): the shallow copy `{...metadata}` is discarded [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/recomputeReduxState.js#L1154)]
- **Temporal scale ignores the right tree's tips and sorts dates as text**: [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L332-L340)]
  - `vals.concat(...)` is not assigned
  - `vals.sort()` has no comparator, which misorders years below 1000 and BCE years
- **Right-tree legend counts ignore frequency**: each right-tree value adds one to the count, whatever its frequency, so the left tree alone orders the legend [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L406-L412)]
- **Right-tree branch labels flip side after the first update**: the update path always uses `xTip - 5` [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/labels.js#L112-L119)]
- **Right-tree hover box 50 px too far right**: the offset adds the whole 100 px gap instead of half [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/infoPanels/hover.js#L341-L344)]
- **Right-tree homoplasies use the left tree's counts** [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/tree.tsx#L174)]
- **Both trees define `id="treeClip"`**: document-wide lookup likely gives the right tree the left clip rectangle (small effect, not verified) [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/phyloTree/renderers.ts#L521-L528)]

### Color and legend

- **JSON legends dropped for discrete colorings**: `_validateLegendArray` returns `true` for every continuous entry and `false` for every other kind, so categorical, ordinal, and boolean legends are discarded at parse time [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/metadataJsonParsing.ts#L268-L295)]
  - introduced on 2026-05-25
  - no public dataset checked uses such a legend, so the effect was not observed
- **Continuous ramp off by one**: 10 domain stops face 9 colors, so the top 11 % of the range extrapolates past the last color [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/globals.js#L37)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L284-L287)]
- **Ordinal palette off by one**: spans of 36 or 37 give 37 or 38 values for a 36-color palette [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L233-L236)]
- **Single-entry continuous legend**: its bounds become `[NaN, Infinity]` [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/colorScale.ts#L506-L516)]

### Interaction and state

- **Zoom label on the root never matches**: `found = 0` doubles as "not found" [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeVisibilityHelpers.js#L34-L57)]
- **Falsy zeros**: a `num_date` of 0 counts as missing in the date filter, and a common ancestor at index 0 counts as not found [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeVisibilityHelpers.js#L139-L141)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/treeVisibilityHelpers.js#L240-L244)]
- **Ns never shown in the tip hover box**: the code reads `nucCounts.nt` instead of `ns` [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/infoPanels/hover.js#L161)]
- **Trait confidence at exactly 0.99**: hover treats `> 0.99` as observed, and the modal treats `< 0.99` as uncertain [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/infoPanels/hover.js#L126)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/components/tree/infoPanels/click.js#L216)]
- **Metadata updates do not refilter**: changing values of a filtered trait updates badge counts but not visibility, as a code comment documents [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/actions/updateMetadata/updateMetadata.ts#L53-L62)]

### Documentation

- **Stream grid width**: the docs say 3 sigma, and the code uses 5 [[doc](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/docs/advanced-functionality/streamtrees.rst#L104-L110)]
- **Display defaults**: the docs list `layout: "scatter"` and `treeZoom`, which the parser rejects or ignores [[doc](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/docs/advanced-functionality/view-settings.rst#L41)]

## Ideas from the issue tracker and history

### Open requests

- **Tree structure**:
  - tee at the start of each child branch [[#2025](https://github.com/nextstrain/auspice/issues/2025)]
  - re-rooting [[#1862](https://github.com/nextstrain/auspice/issues/1862)]
  - declared divergence units [[#1238](https://github.com/nextstrain/auspice/issues/1238)], and more metrics [[#1769](https://github.com/nextstrain/auspice/issues/1769)]
  - radial unfurl on zoom [[#1345](https://github.com/nextstrain/auspice/issues/1345)], and an unrooted polytomy display [[#1522](https://github.com/nextstrain/auspice/issues/1522)]
  - hiding the deep backbone below a depth threshold for public-health users [[#1451](https://github.com/nextstrain/auspice/issues/1451)]
  - excluding hidden nodes from the domain [[#1950](https://github.com/nextstrain/auspice/issues/1950)]
- **Navigation**:
  - shareable zoom to any node [[#1036](https://github.com/nextstrain/auspice/issues/1036)]
  - zoom history [[#1403](https://github.com/nextstrain/auspice/pull/1403)]
  - coordinated zoom in tanglegrams [[#1772](https://github.com/nextstrain/auspice/issues/1772)]
  - vertical zoom for dense branches [[#1111](https://github.com/nextstrain/auspice/issues/1111)]
- **Labels**:
  - show all tip labels [[#916](https://github.com/nextstrain/auspice/issues/916)]
  - multi-field tip labels [[#1963](https://github.com/nextstrain/auspice/issues/1963)]
  - rotated radial labels [[#1423](https://github.com/nextstrain/auspice/issues/1423)]
  - a middle density for branch labels [[#1624](https://github.com/nextstrain/auspice/issues/1624)]
  - branch labels in info boxes [[#1782](https://github.com/nextstrain/auspice/issues/1782)]
- **Color**:
  - color-blind palettes [[#932](https://github.com/nextstrain/auspice/issues/932)], quantile scales [[#1454](https://github.com/nextstrain/auspice/issues/1454)], and a distinct uncertainty gray [[#1797](https://github.com/nextstrain/auspice/issues/1797)]
  - legend counts [[#1990](https://github.com/nextstrain/auspice/issues/1990)], and several colorings at once [[#509](https://github.com/nextstrain/auspice/issues/509)]
  - dashed branch styles [[#1650](https://github.com/nextstrain/auspice/issues/1650)]
  - homoplasy coloring computed in the browser [[#1812](https://github.com/nextstrain/auspice/issues/1812)], and reversion highlighting [[#1470](https://github.com/nextstrain/auspice/issues/1470)]
  - tips drawn in inverse-frequency order [[#1362](https://github.com/nextstrain/auspice/issues/1362)]
- **Two trees**: the same tree twice with two colorings [[#1589](https://github.com/nextstrain/auspice/issues/1589)], and fewer panels in tanglegram mode [[#1939](https://github.com/nextstrain/auspice/issues/1939)]
- **Filters**: continuous ranges [[#1748](https://github.com/nextstrain/auspice/issues/1748)], internal-node traits [[#1865](https://github.com/nextstrain/auspice/issues/1865)] [[#1275](https://github.com/nextstrain/auspice/issues/1275)], and a date filter that keeps ancestral branches clickable [[#1405](https://github.com/nextstrain/auspice/issues/1405)]
- **Uncertainty**: sampling histories from joint distributions [[#1163](https://github.com/nextstrain/auspice/pull/1163)], and observed versus inferred flags for traits [[augur#386](https://github.com/nextstrain/augur/issues/386)]

### Declined or abandoned

- **Untangling by default** (2018): it hid the second tree's clade structure [[#577](https://github.com/nextstrain/auspice/pull/577)]
- **Branch gradients** (2020): browsers failed to render them [[#1042](https://github.com/nextstrain/auspice/pull/1042)]
- **Free pan and zoom**: confusing, and replaced by clade zoom [[#936](https://github.com/nextstrain/auspice/issues/936)]
- **Auto-zoom on filter change**: too jumpy [[#1240](https://github.com/nextstrain/auspice/issues/1240)]
- **Node-name zoom URLs**: names change between builds [[#1036](https://github.com/nextstrain/auspice/issues/1036)]
- **Three or more trees**: "probably never will be" [[forum](https://discussion.nextstrain.org/t/1501)]
- **Canvas or WebGL rendering**: d3 events and transitions were valued more [[#968](https://github.com/nextstrain/auspice/issues/968)]
- **Epi-week axis labels**: no agreed week definition [[#804](https://github.com/nextstrain/auspice/pull/804)]
- **Smaller DOM** (integer coordinates, group-level styles): small memory gains and unwelcome visual changes [[#1128](https://github.com/nextstrain/auspice/pull/1128)] [[#1136](https://github.com/nextstrain/auspice/pull/1136)]

### Features found only in the history

- **Tree panning** existed until Auspice 1.19.0 (2018-05-09). The review of the tanglegram PR had flagged that the two trees panned independently [[#537](https://github.com/nextstrain/auspice/pull/537)] [[#544](https://github.com/nextstrain/auspice/pull/544)]
- **Vaccine strains at their use date**, with dotted lines to the collection-date tip (2018), later reduced to crosses [[#498](https://github.com/nextstrain/auspice/pull/498)] [[#529](https://github.com/nextstrain/auspice/pull/529)]
- **LBI computed in the browser** (2018). Two unused constants remain, and the hard-coded LBI scale was removed in 2024 [[#491](https://github.com/nextstrain/auspice/pull/491)] [[#1842](https://github.com/nextstrain/auspice/pull/1842)] [[src](https://github.com/nextstrain/auspice/blob/dbf287559125f24a66c3207bdf1eb30fa46806b4/src/util/globals.js#L32-L33)]
- **Partial-match strain search highlighting** (2020), not carried over when strain search became a filter [[#930](https://github.com/nextstrain/auspice/pull/930)] [[#1200](https://github.com/nextstrain/auspice/pull/1200)]

## Open scientific problems

- **Marking resolution**: neither the dataset format nor the viewer tells splits that a tree builder or TreeTime resolved from inferred structure. The team switches between divergence and time views instead [[augur#537](https://github.com/nextstrain/augur/issues/537)]
- **Inferred versus observed states**: the format cannot say whether a tip value was observed or reconstructed. Dates gained `inferred` and `raw_value` in 2025, and traits have not [[augur#386](https://github.com/nextstrain/augur/issues/386)]
- **Joint uncertainty**: per-node marginal confidences cannot be sampled into consistent histories. That needs joint distributions on branches [[#1163](https://github.com/nextstrain/auspice/pull/1163)]
- **Where a change happened on a branch**: coloring a whole branch by its child state implies a timing that the reconstruction does not give [[#896](https://github.com/nextstrain/auspice/issues/896)]
- **Divergence units and comparable axes**: without declared units, labels and rates are guesses. Segment trees with different rates cannot share a divergence axis [[#1238](https://github.com/nextstrain/auspice/issues/1238)] [[#1769](https://github.com/nextstrain/auspice/issues/1769)]
- **Node dates across gene trees** can differ by decades for the same events [[forum](https://discussion.nextstrain.org/t/1596)]
- **Crossings versus reassortment**: crossings also arise from uncertain topology, polytomies, and resolution order, and untangling is off. A tanglegram alone cannot separate them ([de Vienne 2019](https://doi.org/10.1093/molbev/msy196))
- **More than two trees**: which order of trees to show, and how to show relations between non-neighbouring trees, has no implemented answer beyond baltic's neighbour chain

## Method and limits

- **Code**: every behavior with a source link was read at commit `dbf2875`. History comes from `git log -L`, `git log -S`, and the linked pull requests
- **Live checks** (2026-10-07, nextstrain.org):
  - checked: branch and tip hover, clade zoom and URL state, tip click and Escape, the HA/NA tanglegram (coloring, line width, axes, forced settings), HA/HA genotype coloring, and filtered tip labels
  - not reproduced: the zoom-dependent unit label, the inverted tiny-span axis, and the discarded JSON legend
- **GitHub search** hit its rate limit several times. The issue research then listed all 1044 issues and 1042 pull requests and filtered titles locally. Issues that no title or commit message describes may be missing
- **Forum**: the Discourse JSON API redirects to `nextstrain.discourse.group`. Parallel fetches hit rate limits and succeeded one at a time. The "exporting data for Auspice" docs page returns 404
- **Literature**: Scornavacca et al. 2011 was read from its abstract and secondary summaries only
