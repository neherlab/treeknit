# treeknit-wasm

WebAssembly bindings for TreeKnit and a static page that runs analyses in the browser.

```sh
just build-wasm                                   # into packages/treeknit-wasm/www/pkg/
python3 -m http.server -d packages/treeknit-wasm/www 8000  # any static server; browsers do not load WebAssembly from file://
```

Then open http://localhost:8000. In the build container, prefix `just build-wasm` with `./dev/docker/run`; the server can run on the host.

## Interface

`analyze(request)` takes and returns plain objects:

```js
analyze({
  trees: [{ label: 'ha', newick: '((A,B),(C,(D,X)));' }, { label: 'na', newick: '((A,(B,X)),(C,D));' }],
  settings: { gamma: 2, resolve: 'matched', seed: 1 }, // optional; missing fields take the CLI defaults
});
// { mccs: { MCC_dict: ... }, resolved: [{ label, newick }], imputed: [...],
//   arg: { status: 'built', newick, nodes, reassortments, trees } | { status: 'failed', message } | null }
```

Settings: `gamma`, `seqLengths`, `nMcmcIt`, `resolve` (`matched`, `strict`, `liberal`, `none`), `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`, `seed`. Invalid input throws an `Error` with a message.

Analyses run on one thread: browsers give WebAssembly no threads without cross-origin isolation.

## Tests

- `just test-rs`: the native tests, with the rest of the workspace
- `just test-wasm`: the JavaScript interface in WebAssembly, in Node
- `just lint-wasm`: Clippy for the WebAssembly target

The `wasm-bindgen` crate pin in `Cargo.toml` and the `wasm-bindgen` tool pin in `.config/mise.toml` must name the same version, because the command-line tool supports only the crate version it was released with.
