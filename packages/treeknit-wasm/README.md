# treeknit-wasm

WebAssembly bindings of TreeKnit, used by the web app in `packages/web`.

```sh
just build-wasm [dev|release|dist]   # into packages/treeknit-wasm/pkg/; dist, the default, adds wasm-opt
```

wasm-bindgen writes the JavaScript module, the WebAssembly binary, and the TypeScript declarations into `pkg/`. The directory is the workspace package `@neherlab/treeknit-wasm`. Only the declarations `pkg/treeknit_wasm.d.ts` are committed, so the TypeScript type checks and lints run without a Rust build: `just gen` rewrites them after a change to the interface, and `just generated-check` fails when they are stale.

## Interface

The request and result types derive their TypeScript declarations with tsify.

```js
await init();
analyze({
  trees: [{ label: 'ha', newick: '((A,B),(C,(D,X)));' }, { label: 'na', newick: '((A,(B,X)),(C,D));' }],
  settings: { gamma: 2, resolve: 'matched', seed: 1 }, // optional; missing fields take the CLI defaults
});
// { pairs: [{ trees: ['ha', 'na'], mccs: [['X'], ['A', 'B', 'C', 'D']] }],
//   arg: { status: 'built', reassortments: 1 } | { status: 'failed', message } | null,
//   files: [{ name: 'MCCs.json', mediaType: 'application/json', text }, ...] }
```

`files` holds the output files of the command line under its file names: `MCCs.json`, the legacy `MCCs.dat` (`MCCs_<a>_<b>.dat` per pair for more than two trees), `<label>_resolved.nwk`, `<label>_imputed.nwk`, and for two trees the ARG files `arg.nwk`, `nodes.dat`, and `<label>_liberal_resolved.nwk`. `defaultSettings()` returns the settings of a request without settings.

Settings: `gamma`, `seqLengths`, `nMcmcIt`, `resolve` (`matched`, `strict`, `liberal`, `none`), `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`, `seed`. Invalid input throws an `Error` with a message. A Rust panic traps the module and leaves the instance unusable.

Analyses run on one thread: browsers give WebAssembly no threads without cross-origin isolation.

## Tests

- `just test-rs`: the native tests, with the rest of the workspace
- `just test-wasm`: the JavaScript interface in WebAssembly, in Node
- `just lint-wasm`: Clippy for the WebAssembly target

The `wasm-bindgen` crate pin in `Cargo.toml` and the `wasm-bindgen` tool pin in `.config/mise.toml` must name the same version, because the command-line tool supports only the crate version it was released with.
