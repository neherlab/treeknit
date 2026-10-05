# treeknit-wasm

WebAssembly bindings of TreeKnit, used by the web app in `packages/web`.

```sh
just build-wasm <dev|release|prod>   # into packages/treeknit-wasm/pkg/; prod, as shipped, adds wasm-opt
```

wasm-bindgen writes the JavaScript module, the WebAssembly binary, and the TypeScript declarations into `pkg/`. The directory is the workspace package `@neherlab/treeknit-wasm`. Only the declarations `pkg/treeknit_wasm.d.ts` are committed, so the TypeScript type checks and lints run without a Rust build: `just gen` rewrites them after a change to the interface, and `just generated-check` fails when they are stale.

## Interface

Every value that crosses the boundary is a Rust type of `treeknit-io` (modules `analysis`, `inspect`, `schema`, `output`, `summary`, `progress`, `display`, `palette`, `figure`, `version`), which the command line uses too, and derives its TypeScript declaration with tsify. Field names are camelCase and enums are lowercase strings, as in `pkg/treeknit_wasm.d.ts`.

Stateless functions:

- `defaultSettings(): Settings`: the command-line defaults
- `settingsSchema(k, settings): SettingsSchema`: default, range, step, applicability with its reason, and help of each setting for `k` trees; the resolution modes with their effects; the help on tree order
- `inspectTree(label, text): TreeInspection`: leaf, internal node, and polytomy counts, branch lengths (`all`, `some`, `none`), parser warnings, and the parse error with line and column
- `overlap(trees): Overlap`: total leaves, each tree's leaves and missing leaves, each pair's shared leaves and whether it blocks a run, and the trees that do not parse
- `validate(request): ValidationError[]`: every problem with the trees and the settings
- `readRequest(text): AnalysisRequest`: the request of a session file; throws when its JSON structure is invalid
- `requestFile(request): OutputFile`: the session file `treeknit_request.json`
- `treeLabels(fileNames, existingLabels): string[]`: labels for loaded files, unique against the existing labels
- `version(): AppVersion`: the TreeKnit version and the repository URL
- `palette(): Palette`: the drawing colors of the light and the dark theme

`Session`, one run and its results:

- `Session.run(request, onProgress): Session`: validates and runs the request on one thread, calling `onProgress` with each `Progress` (`phase` `pairs`, `matching`, or `done`; `fraction` from 0 to 1, never decreasing, and 1 only at `done`). An error that `onProgress` throws stops further progress calls and is thrown after the run
- `summary(): Summary`: per pair the labels, the MCCs as leaf names, and the counts of imputed and ambiguously attached leaves; the ARG outcome (`status` `built` with the reassortment count, or `failed` with the message; `null` for more than two trees); the diagnostics
- `files(): FileEntry[]`: every output file with its command-line path, media type, and size in bytes: `treeknit_request.json` (the request that ran), the files of `treeknit --impute --auspice-view` with the tree extension `.nwk`, `parameters.json`, and `log.txt`
- `fileText(path): string`: the text of a listed file; throws `no file <path>` for any other path
- `zip(): Uint8Array`: every listed file under `treeknit_results/`
- `commandLine(): string`: the command that writes the same files from the extracted archive

Not built yet; these throw `not implemented`:

- `pairView(pair, version, scale): PairView`: the tanglegram of a pair in version `input`, `resolved`, or `imputed`, laid out with scale `div` or `depth`
- `argView(scale): ArgView | undefined`: the ARG of two trees; `undefined` for more than two trees or a failed ARG
- `constellation(): ConstellationTable`: the MCC of every leaf in every pair
- `figure(pair, version, options): string`: the SVG tanglegram of a pair
- `argFigure(options): string`: the SVG figure of the ARG

`validate` and `Session.run` apply the checks of the command line. `validate` returns each problem with the path of the field it concerns (such as `settings.gamma` or `trees[1].newick`) and, for Newick errors with a position, the 1-based line and column; an empty list means the request runs. `Session.run` throws an `Error` named `ValidationError` whose message joins the messages, one per line, so the caller tells an invalid request from an internal failure, which throws an `Error`. A request that is not of the declared type throws an `Error` that says where it is wrong. A Rust panic traps the module and leaves the instance unusable.

Display data uses normalized units: x from 0 to 1 across the column of a shape (a tree column, or the link zone of a tanglegram), y in leaf rows. `PairView` and `ArgView` carry the drawn shapes (`shapes`): branch elbows, marks, link curves, and ribbon outlines, with curves as cubic Bézier segments `{ from, c1, c2, to }`. The SVG figures draw the same shapes.

Analyses run on one thread: browsers give WebAssembly no threads without cross-origin isolation.

The `log` records of a run, of level Debug and above, become `log.txt` with lines `<time> [LEVEL] <message>`, the layout of the command-line log without its thread ID, and its warnings and errors become the diagnostics of `summary()`. The records of every other export are discarded.

## Tests

- `just test-rs`: the native tests, with the rest of the workspace
- `just test-wasm`: the JavaScript interface in WebAssembly, in Node
- `just lint-wasm`: Clippy for the WebAssembly target

The `wasm-bindgen` crate pin in `Cargo.toml` and the `wasm-bindgen` tool pin in `.config/mise.toml` must name the same version, because the command-line tool supports only the crate version it was released with.
