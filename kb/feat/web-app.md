# Web app and WebAssembly bindings (new)

TreeKnit.jl has no browser version. `packages/treeknit-wasm` compiles the core and the formats to WebAssembly, and `packages/web` is a React app that runs an analysis in the browser. [`kb/decisions/web-app.md`](../decisions/web-app.md) records the design decisions.

## WebAssembly bindings

`packages/treeknit-wasm` [[src](../../packages/treeknit-wasm/src/lib.rs)] binds the shared modules of `treeknit-io` (`analysis`, `run`, `output`, `summary`), which the command line uses too. `packages/treeknit-wasm/README.md` lists the interface:

- [x] **`Session.run(request, onProgress)`**: validates with the checks of `validate`, runs on one thread, and calls `onProgress` with each `Progress` of the core [[src](../../packages/treeknit-wasm/src/lib.rs#L135-L195)]. An invalid request throws an `Error` named `ValidationError` whose message joins the messages; an error thrown by `onProgress` stops further progress calls and is thrown after the run
- [x] **`summary()`**: per pair the labels, the MCCs as leaf names, and the counts of imputed and ambiguously attached leaves; the ARG outcome; the diagnostics [[src](../../packages/treeknit-io/src/summary.rs#L22-L45)]
- [x] **Output files**: `files()` lists the session file `treeknit_request.json`, the files of `treeknit --impute --auspice-view` (every tree file with the extension `.nwk`), `parameters.json`, and `log.txt`, each with its media type and size in bytes; `fileText(path)` returns the text of a listed file, and `zip()` an archive of all of them under `treeknit_results/` [[src](../../packages/treeknit-wasm/src/lib.rs#L204-L238)]
- [x] **`commandLine()`**: the command that writes the same files from the extracted archive (see [`formats.md`](formats.md#session-file-treeknit_requestjson))
- [x] **Log capture**: the records of level Debug and above of a run become `log.txt`, in the layout of the command-line log without a thread ID, and its warnings and errors become the diagnostics. Every other export discards its records [[src](../../packages/treeknit-wasm/src/log_capture.rs)]
- [ ] **Display data and figures**: `pairView`, `argView`, `constellation`, `figure`, and `argFigure` throw "not implemented"; the file list has no figures yet
- [x] **`defaultSettings()`**: the settings of a request without settings, which are the command-line defaults
- [/] **Settings**: `gamma`, `seqLengths`, `nMcmcIt`, `resolve`, `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`, and `seed` [[src](../../packages/treeknit-io/src/analysis.rs#L42-L71)]. The former method options of the command line and the thread count are not available; analyses run on one thread
- [x] **`validate(request)`**: the checks of the command line [[src](../../packages/treeknit-io/src/analysis.rs#L160-L166)]: at least two trees, labels usable as file names and unique, distinct pair file names, Newick syntax, at least two shared leaves per pair, γ finite and not negative, one finite positive sequence length per tree, at least one round and one MCMC step, and a seed of at most 2^53 - 1. Each error has the camelCase path of its field (`settings.seqLengths[1]`, `trees[0].newick`) and, for a Newick error with a position, the 1-based line and column in characters. A request of the wrong shape (unknown fields, wrong types) throws an error with the JSON position
- [x] **`readRequest(text)`**: the request of a session file, `treeknit_request.json`, checked for its JSON structure and a seed of at most 2^53 - 1 only, so a request with a broken tree or an invalid setting loads; throws with the messages otherwise [[src](../../packages/treeknit-io/src/analysis.rs#L303-L326)]
- [x] **`requestFile(request)`**: the session file of a request, as the output file `treeknit_request.json` (see [`formats.md`](formats.md#session-file-treeknit_requestjson))
- [x] **`treeLabels(fileNames, existingLabels)`**: labels for new trees: the file name without its last extension, `tree` for an empty name, and `_2`, `_3` on a collision with an existing or earlier label, ignoring case as the label check does [[src](../../packages/treeknit-io/src/analysis.rs#L325-L353)]. The command line labels by path instead (see [`cli.md`](cli.md#input-validation))
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

- [x] **Rust**: the native tests of `treeknit-io`, and `just test-wasm`, which runs the exported functions in WebAssembly in Node: the plain-object shapes, a `Session` run of the two-tree example against `fixtures/doc_mccs_1.json` of TreeKnit.jl, its progress, diagnostics, files, and archive, and the parallel path on one thread
- [x] **TypeScript**: `vitest` tests of the examples, the download helper, and the Content Security Policy
- [ ] **Browser**: no test runs the app in a browser ([`N-web-app-untested-in-browsers.md`](../issues/N-web-app-untested-in-browsers.md))
