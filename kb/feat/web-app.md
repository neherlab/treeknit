# Web app and WebAssembly bindings (new)

TreeKnit.jl has no browser version. `packages/treeknit-wasm` compiles the core and the formats to WebAssembly, and `packages/web` is a React app that runs an analysis in the browser. [`kb/decisions/web-app.md`](../decisions/web-app.md) records the design decisions.

## WebAssembly bindings

`packages/treeknit-wasm` [[src](../../packages/treeknit-wasm/src/lib.rs#L9-L32)]:

- [x] **`analyze(request)`**: runs the pipeline on labeled Newick texts and returns the MCCs of each pair, the ARG outcome for two trees (hybrid count or failure message), and the output files [[src](../../packages/treeknit-wasm/src/analysis.rs#L9-L48)]
- [x] **`defaultSettings()`**: the settings of a request without settings, which are the command-line defaults
- [/] **Settings**: `gamma`, `seqLengths`, `nMcmcIt`, `resolve`, `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`, and `seed` [[src](../../packages/treeknit-wasm/src/analysis.rs#L67-L113)]. The former method options of the command line and the thread count are not available; analyses run on one thread
- [x] **Validation**: at least two trees, non-empty and unique labels, γ finite and not negative, at least one round, one positive sequence length per tree, and no unknown fields [[src](../../packages/treeknit-wasm/src/analysis.rs#L180-L222)]. Errors name the field or position. The command line does not check the same ranges ([`H-cli-accepts-invalid-settings.md`](../issues/H-cli-accepts-invalid-settings.md))
- [/] **Output files**: `MCCs.json`, `MCCs.dat` or `MCCs_<a>_<b>.dat`, `<label>_resolved.nwk`, `<label>_imputed.nwk`, and for two trees `arg.nwk`, `nodes.dat`, `<label>_liberal_resolved.nwk`, under the command-line names with the extension `.nwk` [[src](../../packages/treeknit-wasm/src/analysis.rs#L224-L264)]. The Auspice JSON files, `parameters.json`, and the log are missing ([`N-web-app-partial-settings-and-outputs.md`](../issues/N-web-app-partial-settings-and-outputs.md))
- [x] **TypeScript declarations**: tsify derives them from the Rust types; the generated file is committed and `just generated-check` detects stale declarations
- [x] **Panics**: a panic hook writes the message to the browser console. A panic traps the module

## Web app

`packages/web` [[src](../../packages/web/src/analysis/AnalysisPage.tsx#L14-L58)]:

- [x] **Tree input**: one or more Newick files (`.nwk`, `.newick`, `.tre`, `.tree`, `.txt`). The label is the file name without its last extension. A new selection replaces the list [[src](../../packages/web/src/analysis/TreeInputs.tsx#L12-L55)]
- [x] **Example**: a button loads two example trees
- [ ] **Newick text input**: there is no field to paste a tree
- [ ] **Tree list editing**: the label, order, or presence of a single tree cannot be changed
- [/] **Settings form**: γ, the resolution mode, and the seed. The other settings keep their defaults ([`N-web-app-partial-settings-and-outputs.md`](../issues/N-web-app-partial-settings-and-outputs.md)) [[src](../../packages/web/src/analysis/SettingsFields.tsx#L23-L34)]
- [x] **Run in a Web Worker**: comlink calls the module in a worker. `AnalysisClient` replaces the worker after a trap or a failed start, so the page stays usable [[src](../../packages/web/src/analysis/client.ts#L10-L47)]
- [x] **Results**: the ARG summary with the number of reassortments or the failure message, one MCC table per pair, and one download button per output file [[src](../../packages/web/src/results/Results.tsx#L8-L88)]
- [x] **Errors**: analysis errors appear above the results; a failed start shows "TreeKnit could not start."
- [ ] **Drawing**: no tree, tanglegram, or ARG drawing (see [`visualization.md`](visualization.md#arg-viewers))
- [x] **Local data**: the trees stay in the browser
- [x] **Self-hosted assets**: the fonts, styles, scripts, and the WebAssembly module are bundled. The Content Security Policy allows WebAssembly compilation with `'wasm-unsafe-eval'`
- [ ] **Hosting**: no workflow deploys the app. A user builds and serves it with `just run-web prod`

## Tests

- [x] **Rust**: the native tests of `analysis.rs`, and `just test-wasm`, which runs the exported functions in WebAssembly in Node, including the parallel path on one thread
- [x] **TypeScript**: `vitest` tests of the form-to-request mapping, file reading, the worker client, and the Content Security Policy
- [ ] **Browser**: no test runs the app in a browser ([`N-web-app-untested-in-browsers.md`](../issues/N-web-app-untested-in-browsers.md))
