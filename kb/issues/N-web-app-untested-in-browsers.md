# Web app has no browser test

`just test-wasm` runs the exported functions in WebAssembly in Node, the native tests cover the analysis and the display data, and `just test-ts` covers the TypeScript logic in Node: the workspace store and search params, the analysis client with its workers, persistence through a storage interface, the canvas view state and layer builders, the Auspice store and selection middleware, and the Content Security Policy. Nothing runs the app in a browser, so these parts are checked only by hand:

- the deck.gl canvas: sizing from the measured box, orientation, the minimap, picking, wheel zoom and drag pan, label fonts
- the Web Workers: loading the module in a Vite-built worker, cancel by terminating the job worker, replacement after a trap
- persistence: IndexedDB writes, restore after reload, coordination between tabs
- the embedded Auspice view: the build plugin's transform of Auspice's source, the patch, two trees with tangle lines, its sidebar controls and downloads
- the Content Security Policy as the browser applies it, and the downloads

A regression of this kind passed every check once: the drawings rendered rotated and blurry after a layout change, because deck.gl mounted before the drawing area had its measured size. Only a browser showed it.

## Decision required

Whether browser tests belong in the test suite is open. A browser runner is slow and resource-heavy, and the core is tested in Rust; without one, a broken build of the app passes every check.

- Browser smoke test: serve the build of `just build-web prod`, load an example in a headless browser pinned like the other tools, run it, and check that the tanglegram canvas, the Auspice view, and a download work
- Component tests in vitest with a DOM implementation and `@testing-library`, without a real browser; this misses the canvas, the workers, and the WebAssembly loading
- No browser test; check the app by hand after changes to `packages/web`

## Validation

- The chosen test runs in the build container and fails when the worker path, the WebAssembly loading, the canvas sizing, or the result shape breaks
