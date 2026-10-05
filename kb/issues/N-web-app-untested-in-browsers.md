# Web app has no browser test

`just test-wasm` runs the exported functions in WebAssembly in Node, the native tests cover the analysis, and `just test-ts` covers the TypeScript logic in Node: the mapping from the form to the request, file reading, the worker client with its replacement of a trapped worker, and the Content Security Policy. Nothing runs the app in a browser: the rendering of the form and the results, loading the module in a Vite-built Web Worker, the Content Security Policy as the browser applies it, and the downloads are untested.

## Decision required

Whether browser tests belong in the test suite is open. A browser runner is slow and resource-heavy, and the core is tested in Rust; without one, a broken build of the app passes every check.

- Browser smoke test: serve the build of `just build-web prod`, load the example trees in a headless browser pinned like the other tools, and check the MCC table and a download
- Component tests in vitest with a DOM implementation and `@testing-library`, without a real browser; this misses the worker and the WebAssembly loading
- No browser test; check the app by hand after changes to `packages/web`

## Validation

- The chosen test runs in the build container and in CI, and fails when the worker path, the WebAssembly loading, or the result shape breaks
