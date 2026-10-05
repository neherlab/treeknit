/* tslint:disable */
/* eslint-disable */
/**
 * A labeled tree in Newick format.
 */
export interface TreeText {
    label: string;
    newick: string;
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
     * MCMC steps per leaf (`--n-mcmc-it`).
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
     * Rounds of pair inference (`--rounds`).
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
 * Trees to compare, and the settings of the analysis.
 */
export interface AnalysisRequest {
    trees: TreeText[];
    settings?: Settings;
}

export type ResolveMode = "none" | "strict" | "liberal" | "matched";


/**
 * The settings that a request without settings uses: the defaults of the command line.
 */
export function defaultSettings(): Settings;

export function start(): void;

/**
 * Every problem with the trees and the settings of the request; none when it runs.
 */
export function validate(request: AnalysisRequest): ValidationError[];

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly defaultSettings: () => [number, number, number];
    readonly start: () => void;
    readonly validate: (a: any) => [number, number, number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
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
