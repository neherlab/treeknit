# treeknit-web

WebAssembly bindings for TreeKnit and a static page that runs analyses in the browser.

Requires the `wasm32-unknown-unknown` target and [wasm-pack](https://github.com/wasm-bindgen/wasm-pack).

```sh
rustup target add wasm32-unknown-unknown
wasm-pack build packages/web --target web --release --out-dir www/pkg
python3 -m http.server -d packages/web/www 8000   # any static server; browsers do not load WebAssembly from file://
```

Then open http://localhost:8000.

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

```sh
cargo test -p treeknit-web                # native
wasm-pack test --node packages/web          # in WebAssembly, needs Node
```
