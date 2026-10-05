# treeknit-web

The TreeKnit web app: a React page that runs analyses in the browser on the WebAssembly build of the core (`packages/treeknit-wasm`). The trees never leave the browser.

```sh
just build-web dev    # unminified build with source maps, into dist/
just build-web prod   # build as shipped
just run-web dev      # Vite dev server with hot reload
just run-web prod     # build as shipped, then serve it
```

The commands are the same in the main checkout and in a worktree; `run-web` prints the URL on the port of the checkout. The developer guide (`docs/dev/developer_guide.md`, section "Web app") describes the modes, the ports, and the use of the build container.

## Structure

- `src/analysis/worker.ts`: the Web Worker that loads the WebAssembly module and exposes `analyze` and `defaultSettings` through [comlink](https://github.com/GoogleChromeLabs/comlink)
- `src/analysis/client.ts`: `AnalysisClient`, the owner of the worker. A WebAssembly trap (a Rust panic) leaves the module unusable, so the client replaces the worker after a trap or a failed start; the analysis fails, the page stays
- `src/analysis/example.ts`: the examples of the **Load example** menu: a small pair of trees, the real H3N2 tree pairs of `data/`, and the simulated cases of `fixtures/sim/`, one lazily loaded chunk per tree file
- `src/analysis/`: the form (`react-hook-form`) with the trees and the settings, and the TanStack Query hooks that call the client
- `src/results/`: the MCC tables, the ARG summary, and the downloads of the output files
- `src/ui/`: the controls, built on React Aria Components and styled with Tailwind CSS
- `build/content-security-policy.ts`: the Content Security Policy of the page; `'wasm-unsafe-eval'` lets the page compile WebAssembly

## Data from Rust

The request and result types are Rust types in `packages/treeknit-wasm/src/analysis.rs`. tsify writes their TypeScript declarations into `packages/treeknit-wasm/pkg/treeknit_wasm.d.ts`, which the app imports from `@neherlab/treeknit-wasm`. Validation, default settings, MCCs per pair, and the output files with their command-line names all come from Rust; the app renders them and does not parse TreeKnit files. After changing the interface, run `just gen` and commit the declarations; `just generated-check` fails when they are stale.

## Tests

`just test-ts` runs the vitest tests in Node over in-memory values: the mapping from the form to the request, file reading, the worker client over a `MessageChannel`, and the Content Security Policy. The analysis itself is tested in Rust (`just test-rs`, `just test-wasm`).
