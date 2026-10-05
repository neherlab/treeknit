# treeknit-wasm

WebAssembly bindings for TreeKnit and a static page that runs analyses in the browser.

```sh
just build-wasm                                   # into packages/treeknit-wasm/www/pkg/
python3 -m http.server -d packages/treeknit-wasm/www 8000  # any static server; browsers do not load WebAssembly from file://
```

Then open http://localhost:8000. In the build container, prefix `just build-wasm` with `./dev/docker/run`; the server can run on the host.

## Interface

`analyze(request)` takes and returns plain objects, typed in the generated `treeknit_wasm.d.ts`:

```js
analyze({
  trees: [{ label: 'ha', newick: '((A,B),(C,(D,X)));' }, { label: 'na', newick: '((A,(B,X)),(C,D));' }],
  settings: { gamma: 2, resolve: 'matched', seed: 1 }, // optional; missing fields take the CLI defaults
});
// { pairs: [{ trees: ['ha', 'na'], mccs: [['X'], ['A', 'B', 'C', 'D']] }],
//   arg: { status: 'built', reassortments: 1 } | { status: 'failed', message } | null,
//   files: [{ name: 'MCCs.json', mediaType: 'application/json', text }, ...] }
```

`files` holds the output files of the command line under its file names: `MCCs.json`, the legacy `MCCs.dat` (`MCCs_<a>_<b>.dat` per pair for more than two trees), `<label>_resolved.nwk`, `<label>_imputed.nwk`, and for two trees the ARG files `arg.nwk`, `nodes.dat`, and `<label>_liberal_resolved.nwk`. `defaultSettings()` returns the settings of a request without settings.

Settings: `gamma`, `seqLengths`, `nMcmcIt`, `resolve` (`matched`, `strict`, `liberal`, `none`), `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`, `seed`. Invalid input throws an `Error` with a message.

Analyses run on one thread: browsers give WebAssembly no threads without cross-origin isolation.

## Tests

- `just test-rs`: the native tests, with the rest of the workspace
- `just test-wasm`: the JavaScript interface in WebAssembly, in Node
- `just lint-wasm`: Clippy for the WebAssembly target

The `wasm-bindgen` crate pin in `Cargo.toml` and the `wasm-bindgen` tool pin in `.config/mise.toml` must name the same version, because the command-line tool supports only the crate version it was released with.
