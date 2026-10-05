# Web app offers a subset of the command-line settings and outputs

The form of the web app sets γ, the resolution mode, and the seed. `analyze` accepts the other settings of the command line (`seqLengths`, `nMcmcIt`, `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`), and the app sends their defaults from `defaultSettings()`, but the form has no inputs for them.

The app offers `MCCs.json`, the legacy `MCCs.dat` files, the resolved and imputed trees, and for two trees the ARG files (`arg.nwk`, `nodes.dat`, the liberally resolved trees). It lacks the auspice JSON (`--auspice-view`), `parameters.json`, and the log; the core's `log` messages go nowhere in the browser. The command line builds `parameters.json` in `packages/treeknit-cli/src/main.rs`, so the WebAssembly bindings cannot reuse it.

## Fix direction

- Add form inputs for the remaining settings to `packages/web/src/analysis/SettingsFields.tsx`; the sequence lengths need one input per tree. Validation stays in `analysis::analyze`, whose messages the app shows
- Return the auspice JSON through `treeknit_io::auspice` as further output files of `analysis::analyze`
- Move the parameter summary of the command line into `treeknit-io`, and return `parameters.json` from both
- Forward `log` records to the browser console, or collect them into the result

## Validation

- `packages/treeknit-wasm/src/analysis.rs` tests cover each new output file; `packages/web` tests cover the mapping of each new input to the request
