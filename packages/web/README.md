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

- `src/main.tsx`: the start-up: fonts, styles, and the root element
- `src/index.css`: the Tailwind CSS theme tokens
- `src/download.ts`: `downloadFile`, the one way the app saves a file: a Blob of the content under an object URL, revoked a minute after the click
- `src/analysis/example.ts`: the examples: a small pair of trees, the real H3N2 tree pairs of `data/`, and the simulated cases of `fixtures/sim/`, each tree as its file name and Newick text, one lazily loaded chunk per tree file
- `build/content-security-policy.ts`: the Content Security Policy of the page; `'wasm-unsafe-eval'` lets the page compile WebAssembly

## Data from Rust

The request and result types are Rust types in `packages/treeknit-wasm/src/analysis.rs`. tsify writes their TypeScript declarations into `packages/treeknit-wasm/pkg/treeknit_wasm.d.ts`, which the app imports from `@neherlab/treeknit-wasm`. Validation, default settings, MCCs per pair, and the output files with their command-line names all come from Rust; the app renders them and does not parse TreeKnit files. After changing the interface, run `just gen` and commit the declarations; `just generated-check` fails when they are stale.

## Tests

`just test-ts` runs the vitest tests in Node over in-memory values: the examples, the download helper, and the Content Security Policy. The analysis itself is tested in Rust (`just test-rs`, `just test-wasm`).
