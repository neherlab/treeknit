# treeknit-wasm

WebAssembly bindings of TreeKnit, used by the web app in `packages/web`.

```sh
just build-wasm <dev|release|prod>   # into packages/treeknit-wasm/pkg/; prod, as shipped, adds wasm-opt
```

wasm-bindgen writes the JavaScript module, the WebAssembly binary, and the TypeScript declarations into `pkg/`. The directory is the workspace package `@neherlab/treeknit-wasm`. Only the declarations `pkg/treeknit_wasm.d.ts` are committed, so the TypeScript type checks and lints run without a Rust build: `just gen` rewrites them after a change to the interface, and `just generated-check` fails when they are stale.

## Interface

The request and its types come from `treeknit_io::analysis`, which the command line uses too, and derive their TypeScript declarations with tsify.

```js
await init();
defaultSettings();
// { gamma: 2, seqLengths: null, nMcmcIt: 50, resolve: 'matched', preResolve: false, rounds: 1,
//   finalRound: true, likelihood: true, naive: false, seed: 1 }
validate({
  trees: [{ label: 'ha', newick: '((A,B),(C,(D,X)));' }, { label: 'na', newick: '((A,(B,X)),(C,D);' }],
  settings: { gamma: -1 }, // optional; missing fields take the CLI defaults
});
// [{ field: 'trees[1].newick', message: "tree na: Newick parse error: expected ',' or ')' at byte 16", line: 1, column: 17 },
//  { field: 'settings.gamma', message: 'gamma must be a non-negative number, got -1', line: null, column: null }]
```

`validate` applies the checks of the command line and returns every problem, with the path of the field it concerns and, for Newick errors with a position, the 1-based line and column. An empty list means the request runs. Settings: `gamma`, `seqLengths`, `nMcmcIt`, `resolve` (`matched`, `strict`, `liberal`, `none`), `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`, `seed`. A request that is not of the declared type throws an `Error` that says where it is wrong. A Rust panic traps the module and leaves the instance unusable.

Analyses run on one thread: browsers give WebAssembly no threads without cross-origin isolation.

## Tests

- `just test-rs`: the native tests, with the rest of the workspace
- `just test-wasm`: the JavaScript interface in WebAssembly, in Node
- `just lint-wasm`: Clippy for the WebAssembly target

The `wasm-bindgen` crate pin in `Cargo.toml` and the `wasm-bindgen` tool pin in `.config/mise.toml` must name the same version, because the command-line tool supports only the crate version it was released with.
