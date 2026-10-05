# Web app and WebAssembly bindings (new)

TreeKnit.jl has no browser version. `packages/treeknit-wasm` compiles the core and the formats to WebAssembly, and `packages/web` is a React app that runs an analysis in the browser. [`kb/decisions/web-app.md`](../decisions/web-app.md) records the design decisions.

## WebAssembly bindings

`packages/treeknit-wasm` [[src](../../packages/treeknit-wasm/src/lib.rs#L1-L41)] binds the shared module `treeknit_io::analysis` [[src](../../packages/treeknit-io/src/analysis.rs#L1-L6)], which the command line uses too:

- [ ] **Run**: no export runs an analysis yet
- [x] **`defaultSettings()`**: the settings of a request without settings, which are the command-line defaults
- [/] **Settings**: `gamma`, `seqLengths`, `nMcmcIt`, `resolve`, `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`, and `seed` [[src](../../packages/treeknit-io/src/analysis.rs#L41-L70)]. The former method options of the command line and the thread count are not available; analyses run on one thread
- [x] **`validate(request)`**: the checks of the command line [[src](../../packages/treeknit-wasm/src/lib.rs#L18-L22)]: at least two trees, labels usable as file names and unique, distinct pair file names, Newick syntax, at least two shared leaves per pair, γ finite and not negative, one finite positive sequence length per tree, at least one round and one MCMC step, and a seed of at most 2^53 - 1. Each error has the camelCase path of its field (`settings.seqLengths[1]`, `trees[0].newick`) and, for a Newick error with a position, the 1-based line and column in characters. A request of the wrong shape (unknown fields, wrong types) throws an error with the JSON position
- [ ] **Output files**: no export returns output files ([`N-web-app-partial-settings-and-outputs.md`](../issues/N-web-app-partial-settings-and-outputs.md))
- [x] **TypeScript declarations**: tsify derives them from the Rust types; the generated file is committed and `just generated-check` detects stale declarations
- [x] **Panics**: a panic hook writes the message to the browser console. A panic traps the module

## Web app

`packages/web` [[src](../../packages/web/src/main.tsx)] is being rebuilt: the page shows the wordmark only, on the design tokens and fonts of the new visual identity [[src](../../packages/web/src/index.css)].

- [ ] **Tree input**: no file, paste, or drop input
- [/] **Examples**: `src/analysis/example.ts` provides a small pair of trees, the real H3N2 tree pairs of `data/`, and the simulated cases of `fixtures/sim/` as file names with Newick texts. Vite bundles each tree file as its own chunk and fetches it when its case is loaded. No menu offers them yet [[src](../../packages/web/src/analysis/example.ts)]
- [ ] **Newick text input**: there is no field to paste a tree
- [ ] **Tree list editing**: the label, order, or presence of a single tree cannot be changed
- [ ] **Settings form**: no settings inputs
- [ ] **Run in a Web Worker**: no analysis client or worker
- [ ] **Results**: no MCC tables, ARG summary, or downloads. `src/download.ts` saves a file through a Blob and an object URL [[src](../../packages/web/src/download.ts)]
- [ ] **Errors**: no analysis errors to show
- [ ] **Drawing**: no tree, tanglegram, or ARG drawing (see [`visualization.md`](visualization.md#arg-viewers))
- [x] **Local data**: the trees stay in the browser
- [x] **Self-hosted assets**: the fonts, styles, scripts, and the WebAssembly module are bundled. The Content Security Policy allows WebAssembly compilation with `'wasm-unsafe-eval'`
- [x] **Hosting**: the workflow `.github/workflows/release.yml` deploys the app to GitHub Pages every night, from relative asset paths that work under any path. `just run-web prod` builds and serves it locally

## Tests

- [x] **Rust**: the native tests of `analysis.rs`, and `just test-wasm`, which runs the exported functions in WebAssembly in Node, including the parallel path on one thread
- [x] **TypeScript**: `vitest` tests of the examples, the download helper, and the Content Security Policy
- [ ] **Browser**: no test runs the app in a browser ([`N-web-app-untested-in-browsers.md`](../issues/N-web-app-untested-in-browsers.md))
