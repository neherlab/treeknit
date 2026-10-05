# Web page offers a subset of the command-line settings and outputs

The page form sets γ, the resolution mode, and the seed. `analyze` accepts the other settings of the command line (`seqLengths`, `nMcmcIt`, `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`), but the page has no inputs for them. Of the command-line outputs, the page offers `MCCs.json`, the resolved and imputed trees, and the ARG files. It lacks the auspice JSON (`--auspice-view`), the legacy `MCCs.dat`, `parameters.json`, and the log; the core's `log` messages go nowhere in the browser.

## Fix direction

- Add form inputs for the remaining settings, with the validation messages of `analysis::analyze`
- Return the auspice JSON from `analyze` through `treeknit_io::auspice`, and offer it with `MCCs.dat` and `parameters.json` for download
- Forward `log` records to the browser console, or collect them into the result

## Validation

- `packages/treeknit-wasm/src/analysis.rs` tests cover each new setting and output
