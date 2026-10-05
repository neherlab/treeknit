/* tslint:disable */
/* eslint-disable */
/**
 * A cubic Bézier segment.
 */
export interface Bezier {
    from: Point;
    c1: Point;
    c2: Point;
    to: Point;
}

/**
 * A figure of a run, by what it shows.
 */
export type Figure = { kind: "pair"; pair: number } | { kind: "arg" };

/**
 * A labeled tree in Newick format.
 */
export interface TreeText {
    label: string;
    newick: string;
}

/**
 * A log record of a run.
 */
export interface Diagnostic {
    level: Level;
    message: string;
    /**
     * RFC 3339 time of the record.
     */
    time: string;
}

/**
 * A node of a `DrawTree`.
 */
export interface DrawNode {
    name: string;
    /**
     * `name` as a label shows it: shortened in the middle to `DRAWING_RULES.label_max_chars`
     * characters (Unicode scalar values) with an ellipsis.
     */
    shortName: string;
    /**
     * Index of the parent; `None` for the root.
     */
    parent: number | null;
    /**
     * Indices of the children, in display order.
     */
    children: number[];
    /**
     * Branch length as parsed; `None` when the tree gives none or the length is not finite.
     */
    branchLength: number | null;
    /**
     * Divergence from the root; a missing or negative length counts as 0.
     */
    xDiv: number;
    /**
     * Cladogram position in branch steps from the root: the height of the root minus the height
     * of the node, where a height is the largest number of branches from a node down to a leaf.
     * Every leaf is at the height of the root.
     */
    xDepth: number;
    /**
     * Leaf rank 0 to n-1 in display order; an internal node sits at the midpoint of its first and
     * last child.
     */
    y: number;
    leaf: boolean;
    /**
     * Number of leaves at or below the node: 1 for a leaf.
     */
    cladeSize: number;
    /**
     * An internal node that the parsed input tree lacks, added by resolution or imputation.
     */
    added: boolean;
    /**
     * A leaf that the input tree lacks, placed by imputation.
     */
    imputed: boolean;
    /**
     * Index of the node's MCC in `PairView.mccs`; `None` for a node without an MCC.
     */
    mcc: number | null;
    /**
     * The branch above the node is a reassortment branch: the node is not the root, has an MCC,
     * and its parent's MCC is a different one or none.
     */
    mccBreak: boolean;
}

/**
 * A node of an `ArgView`.
 */
export interface ArgNodeView {
    label: string;
    /**
     * `label` as a label shows it, shortened as `DrawNode.short_name`.
     */
    shortLabel: string;
    /**
     * Parent index per segment; `None` for the top root, where the node lacks the segment, and
     * where it is the root of the segment without the synthetic `GlobalRoot` above it.
     */
    parents: [number | null, number | null];
    /**
     * Indices of the children, in display order.
     */
    children: number[];
    /**
     * Length of the branch to the parent, per segment.
     */
    tau: [number | null, number | null];
    /**
     * The node has different parents in the two segments.
     */
    hybrid: boolean;
    leaf: boolean;
    /**
     * The segments the node carries (0, 1, or both), ascending.
     */
    segments: number[];
    /**
     * Distance from the top root along the parent chain that leads to it; a missing or negative
     * length counts as 0.
     */
    xDiv: number;
    /**
     * Cladogram position in branch steps from the top root, as `DrawNode.x_depth`: the height of
     * the top root minus the height of the node, where a height is the largest number of edges
     * from a node down to a leaf over the children of both segments.
     */
    xDepth: number;
    /**
     * Leaf rank in display order; an internal node sits at the midpoint of its children.
     */
    y: number;
}

/**
 * A numeric setting.
 */
export interface NumberSetting {
    default: number;
    /**
     * Smallest accepted value, or, with `min_exclusive`, the bound that values must exceed.
     */
    min: number;
    /**
     * Values must be greater than `min`, not equal to it.
     */
    minExclusive: boolean;
    /**
     * Largest accepted value; `None` without an upper bound.
     */
    max: number | null;
    /**
     * Step of the input control, or none for any value, as `step="any"` of an HTML number input.
     * A step also restricts the valid values to `min + n * step`, so real-valued settings have
     * none.
     */
    step: number | null;
    /**
     * Only whole numbers are accepted, such as a count or a seed.
     */
    integer: boolean;
    /**
     * The setting changes the result of a run with the current settings.
     */
    applies: boolean;
    /**
     * Why the setting does not apply; `None` when it applies.
     */
    reason: string | null;
    /**
     * One-sentence help text.
     */
    help: string;
}

/**
 * A point `[x, y]` in normalized units.
 */
export type Point = [number, number];

/**
 * A point mark on a drawing.
 */
export interface Mark {
    kind: MarkKind;
    /**
     * Index of the node the mark belongs to.
     */
    node: number;
    /**
     * MCC of the node in a tanglegram, its `DrawNode.mcc`; `None` for a node without an MCC and
     * on the ARG.
     */
    mcc: number | null;
    /**
     * Color slot of `mcc`.
     */
    slot: number | null;
    at: Point;
}

/**
 * A problem with a request, at the field it concerns.
 */
export interface ValidationError {
    /**
     * Path of the field in the request, such as `settings.gamma`, `settings.seqLengths[1]`, or
     * `trees[0].newick`; `None` for the request as a whole.
     */
    field: string | null;
    message: string;
    /**
     * 1-based line of a parse error in the Newick text.
     */
    line: number | null;
    /**
     * 1-based column of a parse error, in Unicode characters.
     */
    column: number | null;
}

/**
 * A resolution mode with its display name and its effect.
 */
export interface ModeInfo {
    mode: ResolveMode;
    /**
     * Display name, such as "Matched".
     */
    name: string;
    /**
     * One-line effect of the mode on the trees.
     */
    effect: string;
}

/**
 * A result file with its text, at its path in the results directory of the command line.
 */
export interface OutputFile {
    /**
     * Path relative to the results directory, with `/` separators, such as `ARG/arg.nwk`.
     */
    path: string;
    /**
     * Media type of the text, such as `application/json`.
     */
    mediaType: string;
    /**
     * The bytes the command line writes, newline rule included.
     */
    text: string;
}

/**
 * A set of leaves of one MCC, consecutive in both trees.
 */
export interface Block {
    /**
     * Index of the MCC in `PairView.mccs`.
     */
    mcc: number;
    /**
     * First and last leaf row of the block in the left tree.
     */
    left: [number, number];
    /**
     * First and last leaf row of the block in the right tree, in left order: the first value
     * belongs to the first leaf of the block in the left tree.
     */
    right: [number, number];
}

/**
 * A tree laid out for drawing, with the MCCs of one pair.
 */
export interface DrawTree {
    label: string;
    /**
     * One node per tree node, indexed from 0 in preorder; node 0 is the root.
     */
    nodes: DrawNode[];
}

/**
 * An MCC of a pair.
 */
export interface MccInfo {
    /**
     * Index of the MCC in `PairView.mccs`, the order of `MCCs.json`.
     */
    index: number;
    /**
     * Number of leaves, attached leaves included.
     */
    size: number;
    /**
     * Leaf names.
     */
    leaves: string[];
    /**
     * Names of the members that imputation attached (leaves in one tree of the pair only).
     */
    imputedLeaves: string[];
    /**
     * Names of the attached members whose attachment is ambiguous: their attachment point is in
     * no MCC, so they joined the MCC whose root is closest to it.
     */
    ambiguousLeaves: string[];
    /**
     * Color slot, 0 to 7, the same in every version of the pair.
     */
    slot: number;
}

/**
 * An edge of an `ArgView`, from parent to child.
 */
export interface ArgEdge {
    parent: number;
    child: number;
    /**
     * The segments the edge carries, ascending.
     */
    segments: number[];
    /**
     * The child is a hybrid node and the edge is not on the child's chain to the top root, along
     * which the node's x is measured. Each hybrid node has one reticulation edge.
     */
    reticulation: boolean;
}

/**
 * An entry of the file list of a run, without its text.
 */
export interface FileEntry {
    /**
     * Path relative to the results directory, as in `OutputFile`.
     */
    path: string;
    /**
     * The last segment of `path`, the name of the file as a download: `arg.nwk` for
     * `ARG/arg.nwk`.
     */
    fileName: string;
    mediaType: string;
    /**
     * Size in bytes; `None` for a figure not rendered yet.
     */
    size: number | null;
    /**
     * The figure the file holds; `None` for the other files.
     */
    figure: Figure | null;
}

/**
 * An on-off setting.
 */
export interface ToggleSetting {
    default: boolean;
    /**
     * The setting changes the result of a run with the current settings.
     */
    applies: boolean;
    /**
     * Why the setting does not apply; `None` when it applies.
     */
    reason: string | null;
    /**
     * One-sentence help text.
     */
    help: string;
}

/**
 * Branch scale of a drawing.
 */
export type Scale = "div" | "depth";

/**
 * Branches and marks of one tree.
 */
export interface TreeShapes {
    /**
     * One elbow per non-root node.
     */
    elbows: Elbow[];
    marks: Mark[];
    /**
     * One leader per leaf, in node order.
     */
    leaders: Leader[];
}

/**
 * Drawing colors of both themes.
 */
export interface Palette {
    light: ThemeColors;
    dark: ThemeColors;
}

/**
 * Drawing colors of one theme, each as `#rrggbb`.
 */
export interface ThemeColors {
    /**
     * The eight MCC color slots.
     */
    mcc: [string, string, string, string, string, string, string, string];
    /**
     * Branches of nodes without an MCC.
     */
    noMcc: string;
    ground: string;
    ink: string;
    inkMuted: string;
    /**
     * Reassortment, and nothing else.
     */
    signal: string;
    focus: string;
    /**
     * Segment A (the first tree) of the ARG.
     */
    segmentA: string;
    /**
     * Segment B (the second tree) of the ARG.
     */
    segmentB: string;
}

/**
 * Leaf overlap of the input trees and of each pair, before a run.
 */
export interface Overlap {
    /**
     * Number of distinct leaves over the trees that parse.
     */
    totalLeaves: number;
    /**
     * The trees that parse, in input order.
     */
    trees: TreeOverlap[];
    /**
     * Pairs of trees that parse, in pipeline order (0,1), (0,2), ..., (1,2), ...
     */
    pairs: PairOverlap[];
    /**
     * Input indices of the trees that do not parse.
     */
    failed: number[];
}

/**
 * Leaves of one tree against all leaves.
 */
export interface TreeOverlap {
    /**
     * Input index of the tree.
     */
    index: number;
    label: string;
    /**
     * Number of leaves of the tree.
     */
    leaves: number;
    /**
     * Number of leaves that other trees have and this tree lacks.
     */
    missing: number;
}

/**
 * Leaves shared by the trees `i` and `j` (input indices, `i < j`).
 */
export interface PairOverlap {
    i: number;
    j: number;
    /**
     * Number of shared leaves.
     */
    shared: number;
    /**
     * The pair shares fewer than `analysis::MIN_SHARED_LEAVES` leaves, so the request does not
     * validate.
     */
    blocked: boolean;
}

/**
 * MCCs of one pair of trees.
 */
export interface PairSummary {
    /**
     * Index of the pair in pipeline order.
     */
    index: number;
    /**
     * Indices of the two trees, `i < j`.
     */
    trees: [number, number];
    /**
     * Labels of the two trees.
     */
    labels: [string, string];
    /**
     * Number of MCCs.
     */
    mccCount: number;
    /**
     * The MCCs as leaf-name lists, as in `MCCs.json`.
     */
    mccs: string[][];
    /**
     * Number of leaves in one tree of the pair only, attached to an MCC of the pair.
     */
    imputedCount: number;
    /**
     * Number of these attached leaves whose attachment is ambiguous.
     */
    ambiguousCount: number;
}

/**
 * Meaning of a mark.
 */
export type MarkKind = "reassortment" | "imputed" | "hybrid";

/**
 * Outcome of building the ARG of two trees.
 */
export type ArgOutcome = { status: "built"; reassortments: number } | { status: "failed"; message: string };

/**
 * Parse error of a Newick text.
 */
export interface TreeError {
    message: string;
    /**
     * 1-based line of the error; `None` for an error without a position.
     */
    line: number | null;
    /**
     * 1-based column of the error, in Unicode characters.
     */
    column: number | null;
}

/**
 * Part of a run that is in progress.
 */
export type Phase = "pairs" | "matching" | "done";

/**
 * Path of an ARG edge: an elbow, or a dashed S-curve for a reticulation edge.
 */
export type EdgePath = { kind: "elbow"; points: [Point, Point, Point] } | { kind: "curve"; curve: Bezier };

/**
 * Presence of branch lengths in a tree.
 */
export type BranchLengths = "all" | "some" | "none";

/**
 * Progress of a run: the completed fraction, and the round and pair in progress.
 */
export interface Progress {
    phase: Phase;
    /**
     * Completed fraction of the run, from 0 to 1, never decreasing during a run.
     */
    fraction: number;
    /**
     * Round in progress, 1-based.
     */
    round: number;
    /**
     * Number of rounds, including the final round without resolution.
     */
    rounds: number;
    /**
     * Pair in progress, 1-based, in pipeline order.
     */
    pair: number;
    /**
     * Number of pairs.
     */
    pairs: number;
}

/**
 * Results of a run at a glance.
 */
export interface Summary {
    /**
     * Pairs in pipeline order (0,1), (0,2), ..., (1,2), ...
     */
    pairs: PairSummary[];
    /**
     * Outcome of the ARG; `None` for more than two trees.
     */
    arg: ArgOutcome | null;
    /**
     * The run shows that the trees have no reassortment: for two trees, the ARG was built with
     * no reassortment (one MCC whose ARG failed proves nothing); for more trees, every pair has
     * one MCC.
     */
    noReassortment: boolean;
    /**
     * Warnings and errors of the run, in the order they occurred.
     */
    diagnostics: Diagnostic[];
}

/**
 * Rules and help of the settings for a request with a given number of trees and settings.
 */
export interface SettingsSchema {
    /**
     * One entry per setting, named as the field of `Settings`.
     */
    settings: SettingFields;
    /**
     * The resolution modes, in the order the web app lists them.
     */
    modes: ModeInfo[];
    /**
     * Help on why the order of the trees matters.
     */
    treeOrderHelp: string;
}

/**
 * Settings of the command line; a missing field takes the command-line default.
 */
export interface Settings {
    /**
     * Cost γ of a reassortment, removing an MCC (`--gamma`).
     */
    gamma?: number;
    /**
     * Sequence lengths of the segments, in the order of the trees, used by the likelihood
     * tie-break (`--seq-lengths`).
     */
    seqLengths?: number[] | null;
    /**
     * MCMC steps per leaf (`--n-mcmc-it`). A fixed-width integer, so that a session file reads
     * the same on 32-bit WebAssembly and on 64-bit hosts, and the check against [`MAX_MCMC_IT`]
     * reports a value that no `usize` of WebAssembly holds.
     */
    nMcmcIt?: number;
    /**
     * How trees are resolved (`--resolve`).
     */
    resolve?: ResolveMode;
    /**
     * Before inference, add to each tree the splits of other trees compatible with all trees
     * (`--pre-resolve`).
     */
    preResolve?: boolean;
    /**
     * Rounds of pair inference (`--rounds`), a fixed-width integer as `n_mcmc_it`.
     */
    rounds?: number;
    /**
     * With strict or liberal resolution and more than two trees, run a final round that
     * re-infers MCCs without resolution (the opposite of `--no-final-round`).
     */
    finalRound?: boolean;
    /**
     * Break ties between configurations with branch lengths (the opposite of
     * `--no-likelihood`).
     */
    likelihood?: boolean;
    /**
     * Naive MCCs, γ → ∞ (`--naive`).
     */
    naive?: boolean;
    /**
     * Seed of the random number generator (`--seed`), at most 2^53 - 1.
     */
    seed?: number;
}

/**
 * Severity of a diagnostic.
 */
export type Level = "error" | "warn" | "info" | "debug";

/**
 * Shape of one input tree, or why it does not parse.
 */
export interface TreeInspection {
    label: string;
    /**
     * Number of leaves; 0 when the tree does not parse.
     */
    leaves: number;
    /**
     * Number of internal nodes, the root included.
     */
    internalNodes: number;
    /**
     * Number of internal nodes with more than two children.
     */
    polytomies: number;
    /**
     * Which branches have a length.
     */
    branchLengths: BranchLengths;
    /**
     * Warnings of the Newick parser, such as "more than one tree in file, using the first".
     */
    warnings: string[];
    /**
     * Why the Newick text does not parse; `None` when it parses.
     */
    error: TreeError | null;
}

/**
 * Size and content of a figure; a missing field takes its default.
 */
export interface FigureOptions {
    /**
     * Width in px.
     */
    width?: number;
    /**
     * Height of a leaf row in px.
     */
    rowHeight?: number;
    /**
     * Branch scale.
     */
    scale?: Scale;
    /**
     * When leaf labels are drawn.
     */
    labels?: LabelMode;
}

/**
 * The ARG of two trees laid out for drawing, in one tree column. Segment 0 (A) is the first
 * tree and segment 1 (B) the second.
 */
export interface ArgView {
    /**
     * The ARG nodes, with the synthetic `GlobalRoot` last when `root_case` is `synthetic`.
     */
    nodes: ArgNodeView[];
    edges: ArgEdge[];
    /**
     * Index of the top root.
     */
    root: number;
    /**
     * How the top root relates to the segment roots, as in `ARG/arg.nwk`.
     */
    rootCase: RootCase;
    /**
     * The shapes of the drawing for the requested scale.
     */
    shapes: ArgShapes;
}

/**
 * The MCC of every leaf in every pair.
 */
export interface ConstellationTable {
    /**
     * Every taxon: first the leaves of the first tree in the leaf order of its final tree, the
     * order of its `_resolved` output file, then the leaves it lacks, in the order of the first
     * tree that has them. The pair views can order the first tree differently, because they sort
     * both trees of a pair for that pair.
     */
    leaves: string[];
    /**
     * Labels of the two trees of each pair, in pipeline order.
     */
    pairs: [string, string][];
    /**
     * `cells[leaf][pair]`; `None` when the leaf is in neither tree of the pair.
     */
    cells: (ConstellationCell | null)[][];
}

/**
 * The MCC of one leaf in one pair.
 */
export interface ConstellationCell {
    /**
     * Index of the MCC in the pair's `PairView.mccs`.
     */
    mcc: number;
    /**
     * Number of leaves of the MCC.
     */
    size: number;
    /**
     * Color slot of the MCC.
     */
    slot: number;
}

/**
 * The S-curve of one link, with control points at x = 0.5.
 */
export interface LinkCurve {
    /**
     * Index of the link in `PairView.links`.
     */
    link: number;
    /**
     * The link's MCC, its `Link.mcc`.
     */
    mcc: number;
    /**
     * Color slot of the link's MCC.
     */
    slot: number;
    curve: Bezier;
}

/**
 * The TreeKnit version and the source repository.
 */
export interface AppVersion {
    /**
     * Version as the command line reports it, such as `0.5.0` or `0.5.0-dev`.
     */
    version: string;
    /**
     * URL of the source repository, from the workspace `repository` field.
     */
    repository: string;
}

/**
 * The dotted line from a leaf tip to the label edge of its tree column, at x = 1, so that the
 * labels align when the tips do not. A leaf at the label edge has a leader of length 0.
 */
export interface Leader {
    /**
     * Index of the leaf.
     */
    node: number;
    /**
     * The tip of the leaf.
     */
    from: Point;
    /**
     * The label edge, at the leaf's row.
     */
    to: Point;
}

/**
 * The drawing rules that the consumer applies, because they depend on the drawn size: the
 * thresholds of the row height, the columns of a drawing, and the strokes and marks in px. The
 * label width that the columns take is measured by the consumer: the interactive views measure
 * the rendered text, the SVG figures estimate it.
 *
 * A drawing has a margin of `margin_px` on each side; the inner width is the rest. A label
 * column is as wide as its longest label plus `label_gap_px` on each side, at most a share of
 * the width it labels. The tanglegram has, from left to right, the left tree, its labels, the
 * link zone, the right labels, and the mirrored right tree. Its link zone takes
 * `link_zone_share` of the inner width minus both label columns, at least `link_zone_min_share`
 * of the inner width, and the two trees share the rest equally. The ARG has its tree, then its
 * labels.
 */
export interface DrawingRules {
    /**
     * In the label mode `auto`, leaf labels are drawn from this many px per row.
     */
    labelAutoMinRowPx: number;
    /**
     * From this many px per row, each link is an S-curve; below it, each block is a ribbon.
     */
    linkMinRowPx: number;
    /**
     * A longer leaf label is shortened in the middle to this many characters (Unicode scalar
     * values); `DrawNode.short_name` and `ArgNodeView.short_label` hold the shortened labels.
     */
    labelMaxChars: number;
    /**
     * Space around a drawing, in px.
     */
    marginPx: number;
    /**
     * Space on each side of a label column, between it and the tree and the link zone, in px.
     */
    labelGapPx: number;
    /**
     * Share of the inner width that the link zone of a tanglegram takes, minus the label columns.
     */
    linkZoneShare: number;
    /**
     * Smallest share of the inner width that the link zone of a tanglegram takes.
     */
    linkZoneMinShare: number;
    /**
     * Largest share of half the inner width that each label column of a tanglegram takes.
     */
    tanglegramLabelColumnMaxShare: number;
    /**
     * Largest share of the inner width that the label column of an ARG takes.
     */
    argLabelColumnMaxShare: number;
    /**
     * Stroke width of a branch, in px.
     */
    branchWidthPx: number;
    /**
     * Stroke width of a reassortment branch, in px.
     */
    reassortmentWidthPx: number;
    /**
     * Stroke width of a link, in px.
     */
    linkWidthPx: number;
    /**
     * Stroke width of a leader from a leaf tip to its label, in px.
     */
    leaderWidthPx: number;
    /**
     * Opacity of a leader.
     */
    leaderOpacity: number;
    /**
     * Radius of a mark ring, in px.
     */
    markRadiusPx: number;
    /**
     * Stroke width of a mark ring, in px.
     */
    markLinePx: number;
    /**
     * Opacity of the fill of a ribbon.
     */
    ribbonOpacity: number;
    /**
     * Dash and gap of a dashed line (added nodes, reticulations), in px.
     */
    dashPx: [number, number];
    /**
     * Dash and gap of a dotted line (leaders), in px.
     */
    dotPx: [number, number];
}

/**
 * The rectangular branch above a node: from (parent x, parent y) to (parent x, node y) to
 * (node x, node y).
 */
export interface Elbow {
    /**
     * Index of the node below the branch.
     */
    node: number;
    points: [Point, Point, Point];
    /**
     * The node's MCC, its `DrawNode.mcc`.
     */
    mcc: number | null;
    /**
     * Color slot of the node's MCC; `None` for a node without an MCC.
     */
    slot: number | null;
    /**
     * The branch is a reassortment branch.
     */
    mccBreak: boolean;
    /**
     * The node is `added`; its branch is dashed.
     */
    added: boolean;
}

/**
 * The ribbon of one block: its left and right y ranges, each extended by half a row, joined by
 * two S-curves.
 */
export interface Ribbon {
    /**
     * Index of the block in `PairView.blocks`.
     */
    block: number;
    /**
     * The block's MCC, its `Block.mcc`.
     */
    mcc: number;
    /**
     * Color slot of the block's MCC.
     */
    slot: number;
    /**
     * Closed outline: each segment starts where the previous one ends, and the last ends where
     * the first starts.
     */
    outline: Bezier[];
}

/**
 * The schema of each field of `Settings`, except `resolve`, which `SettingsSchema.modes`
 * describes.
 */
export interface SettingFields {
    gamma: NumberSetting;
    /**
     * One sequence length per tree; `default` is the length a newly added tree gets.
     */
    seqLengths: NumberSetting;
    nMcmcIt: NumberSetting;
    rounds: NumberSetting;
    seed: NumberSetting;
    preResolve: ToggleSetting;
    finalRound: ToggleSetting;
    likelihood: ToggleSetting;
    naive: ToggleSetting;
}

/**
 * The shape of one ARG edge.
 */
export interface ArgEdgeShape {
    /**
     * Index of the edge in `ArgView.edges`.
     */
    edge: number;
    /**
     * The segments of the edge, its `ArgEdge.segments`.
     */
    segments: number[];
    /**
     * The edge is a reticulation edge, its `ArgEdge.reticulation`; its path is a curve.
     */
    reticulation: boolean;
    path: EdgePath;
}

/**
 * The shapes of a tanglegram.
 */
export interface PairShapes {
    /**
     * Branches and marks of the left tree, x across the left tree column.
     */
    left: TreeShapes;
    /**
     * Branches and marks of the right tree, x across the right tree column, unmirrored.
     */
    right: TreeShapes;
    /**
     * One S-curve per link, x across the link zone: from (0, left y) to (1, right y).
     */
    links: LinkCurve[];
    /**
     * One outline per block, x across the link zone.
     */
    ribbons: Ribbon[];
}

/**
 * The shapes of an ARG drawing, x across the tree column.
 */
export interface ArgShapes {
    /**
     * One shape per edge, in the order of `ArgView.edges`.
     */
    edges: ArgEdgeShape[];
    /**
     * Hybrid rings.
     */
    marks: Mark[];
    /**
     * One leader per leaf, in node order.
     */
    leaders: Leader[];
}

/**
 * The tanglegram of one pair of trees in one version.
 */
export interface PairView {
    left: DrawTree;
    right: DrawTree;
    /**
     * One link per leaf in both drawn trees that has an MCC of the pair, in the left display
     * order.
     */
    links: Link[];
    /**
     * Runs of consecutive links of one MCC, in the left display order.
     */
    blocks: Block[];
    /**
     * The MCCs of the pair; `DrawNode.mcc`, `Link.mcc`, and `Block.mcc` index this list.
     */
    mccs: MccInfo[];
    /**
     * The shapes of the drawing for the requested scale.
     */
    shapes: PairShapes;
}

/**
 * The three top-root cases of the extended Newick writer.
 */
export type RootCase = "shared" | "synthetic" | "oneShared";

/**
 * The two copies of one leaf.
 */
export interface Link {
    /**
     * Node index of the leaf in the left tree.
     */
    left: number;
    /**
     * Node index of the leaf in the right tree.
     */
    right: number;
    /**
     * Index of the leaf's MCC in `PairView.mccs`.
     */
    mcc: number;
}

/**
 * Trees to compare, and the settings of the analysis.
 */
export interface AnalysisRequest {
    trees: TreeText[];
    settings?: Settings;
}

/**
 * Version of the trees of a pair: input, resolved, or imputed.
 */
export type TreeVersion = "input" | "resolved" | "imputed";

/**
 * When leaf labels are drawn.
 */
export type LabelMode = "auto" | "on" | "off";

export type ResolveMode = "none" | "strict" | "liberal" | "matched";


/**
 * One run of TreeKnit and its results, kept for later queries.
 */
export class Session {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * The SVG figure of the ARG with `options`. With the scale `div`, an ARG where a segment has no
     * branch lengths is drawn as a cladogram. Throws an `Error` named `ValidationError` when
     * `options` are invalid, and an `Error` for more than two trees or a failed ARG.
     */
    argFigure(options: FigureOptions): string;
    /**
     * The ARG laid out with `scale`; `undefined` for more than two trees or a failed ARG.
     */
    argView(scale: Scale): ArgView | undefined;
    /**
     * The command that reproduces the file set of the run from the extracted archive.
     */
    commandLine(): string;
    /**
     * The MCC of every leaf in every pair.
     */
    constellation(): ConstellationTable;
    /**
     * The SVG tanglegram of pair `pair` (pipeline order) in `version` with `options`. With the
     * scale `div`, a pair where a tree has no branch lengths is drawn as cladograms, as in the
     * figure files. Throws an `Error` named `ValidationError` when `options` are invalid. The
     * figure files of `files()` keep their default options.
     */
    figure(pair: number, version: TreeVersion, options: FigureOptions): string;
    /**
     * The text of the listed file at `path`. A figure is rendered with the default options on
     * first use and kept.
     */
    fileText(path: string): string;
    /**
     * Every output file of the run, without its text. A figure has the size `null` until its
     * text is first read, and names the figure it holds.
     */
    files(): FileEntry[];
    /**
     * The tanglegram of pair `pair` (pipeline order) in `version`, laid out with `scale`.
     */
    pairView(pair: number, version: TreeVersion, scale: Scale): PairView;
    /**
     * Validate and run `request`, calling `onProgress` with each `Progress`. Throws an `Error`
     * named `ValidationError` when the request does not validate, the error that `onProgress`
     * throws (after the run completes without further progress calls), and an `Error` otherwise.
     */
    static run(request: AnalysisRequest, onProgress: (progress: Progress) => void): Session;
    /**
     * The MCCs of each pair, the ARG outcome, and the diagnostics.
     */
    summary(): Summary;
    /**
     * A ZIP archive of every listed file, under `treeknit_results/`, figures included. A figure
     * not yet read is rendered into the archive and not kept, so the archive needs the memory of
     * one figure at a time.
     */
    zip(): Uint8Array;
}

/**
 * The settings that a request without settings uses: the defaults of the command line.
 */
export function defaultSettings(): Settings;

/**
 * The thresholds of the drawing rules that depend on the drawn row height.
 */
export function drawingRules(): DrawingRules;

/**
 * Leaf, node, and polytomy counts, branch lengths, warnings, and parse error of one tree.
 */
export function inspectTree(label: string, text: string): TreeInspection;

/**
 * Leaf overlap of the trees and of each pair, with the pairs that block a run.
 */
export function overlap(trees: TreeText[]): Overlap;

/**
 * The drawing colors of the light and the dark theme.
 */
export function palette(): Palette;

/**
 * The request of a session file (`treeknit_request.json`); throws with the messages when its
 * JSON structure is invalid.
 */
export function readRequest(text: string): AnalysisRequest;

/**
 * The session file of a request, `treeknit_request.json`.
 */
export function requestFile(request: AnalysisRequest): OutputFile;

/**
 * Defaults, ranges, applicability, and help of every setting, for `k` trees and `settings`.
 */
export function settingsSchema(k: number, settings: Settings): SettingsSchema;

/**
 * Set up the module: panics go to the console, and the log of the Rust code is captured for the
 * diagnostics and `log.txt` of each run. Throws when another logger is installed.
 */
export function start(): void;

/**
 * Labels for trees loaded from `fileNames`: the file name without its last extension, with the
 * characters that a label must not hold replaced by `_`, and `_2`, `_3`, ... where it collides
 * with `existingLabels` or an earlier new label. Every label passes the label check.
 */
export function treeLabels(fileNames: string[], existingLabels: string[]): string[];

/**
 * Every problem with the trees and the settings of the request; none when it runs.
 */
export function validate(request: AnalysisRequest): ValidationError[];

/**
 * The TreeKnit version and the source repository.
 */
export function version(): AppVersion;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_session_free: (a: number, b: number) => void;
    readonly defaultSettings: () => [number, number, number];
    readonly drawingRules: () => [number, number, number];
    readonly inspectTree: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly overlap: (a: number, b: number) => [number, number, number];
    readonly palette: () => [number, number, number];
    readonly readRequest: (a: number, b: number) => [number, number, number];
    readonly requestFile: (a: any) => [number, number, number];
    readonly session_argFigure: (a: number, b: any) => [number, number, number, number];
    readonly session_argView: (a: number, b: any) => [number, number, number];
    readonly session_commandLine: (a: number) => [number, number];
    readonly session_constellation: (a: number) => [number, number, number];
    readonly session_figure: (a: number, b: number, c: any, d: any) => [number, number, number, number];
    readonly session_fileText: (a: number, b: number, c: number) => [number, number, number, number];
    readonly session_files: (a: number) => [number, number, number, number];
    readonly session_pairView: (a: number, b: number, c: any, d: any) => [number, number, number];
    readonly session_run: (a: any, b: any) => [number, number, number];
    readonly session_summary: (a: number) => [number, number, number];
    readonly session_zip: (a: number) => [number, number, number, number];
    readonly settingsSchema: (a: number, b: any) => [number, number, number];
    readonly start: () => void;
    readonly treeLabels: (a: number, b: number, c: number, d: number) => [number, number];
    readonly validate: (a: any) => [number, number, number, number];
    readonly version: () => [number, number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __externref_drop_slice: (a: number, b: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
