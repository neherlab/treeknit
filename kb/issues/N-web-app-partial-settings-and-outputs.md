# Web app offers a subset of the command-line settings and outputs

`analyze` accepts the settings of the command line (`gamma`, `resolve`, `seed`, `seqLengths`, `nMcmcIt`, `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`), and `defaultSettings()` returns their defaults.

`analyze` returns `MCCs.json`, the legacy `MCCs.dat` files, the resolved and imputed trees, and for two trees the ARG files (`arg.nwk`, `nodes.dat`, the liberally resolved trees). It lacks the auspice JSON (`--auspice-view`), `parameters.json`, and the log; the core's `log` messages go nowhere in the browser. The command line builds `parameters.json` in `packages/treeknit-cli/src/main.rs`, so the WebAssembly bindings cannot reuse it.

## Fix direction

- Give the settings form of the web app an input for every setting; the sequence lengths need one input per tree. Validation stays in Rust, whose messages the app shows
- Return the auspice JSON through `treeknit_io::auspice` as further output files of `analysis::analyze`
- Move the parameter summary of the command line into `treeknit-io`, and return `parameters.json` from both
- Forward `log` records to the browser console, or collect them into the result

## Validation

- `packages/treeknit-wasm/src/analysis.rs` tests cover each new output file; `packages/web` tests cover the mapping of each new input to the request
