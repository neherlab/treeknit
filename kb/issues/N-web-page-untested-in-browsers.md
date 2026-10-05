# Web page has no browser test

`just test-wasm` runs the exported `analyze` function in Node, and the native tests cover the analysis. Nothing runs the page itself (`packages/treeknit-wasm/www/`): the module Web Worker, file loading, the result tables, the download links, the error display, and the replacement of the worker after a WebAssembly trap are untested.

## Fix direction

- Add an end-to-end test that serves `packages/treeknit-wasm/www/` after `just build-wasm`, loads the example trees in a headless browser, and checks the MCC table and the download links, with the browser pinned like the other tools
- Cover the error path with an invalid tree, and the trap path with a request that panics

## Validation

- The test runs in the build container and in CI, and fails when the worker script path or the result shape changes
