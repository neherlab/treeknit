# Web page with a WebAssembly build of the core

TreeKnit.jl has no browser version. `packages/web` compiles `treeknit-core` and `treeknit-io` to WebAssembly and adds a static page (`packages/web/www/`) that runs an analysis in the browser and offers the result files for download.

## Decisions

- **One thread.** Browsers give WebAssembly threads only on cross-origin isolated pages (`COOP`/`COEP` headers) and with the standard library rebuilt by nightly Rust. The page runs pairs sequentially (`Options::parallel = false`). Without threads, rayon's global pool falls back to the calling thread, so the parallel path also works; `packages/web/tests/wasm.rs` checks that it gives the same MCCs
- **No OS entropy.** The workspace takes `rand` without default features. The core seeds `rand_xoshiro` generators only, and `getrandom` 0.2, which the default features pull in, does not compile for `wasm32-unknown-unknown`
- **Validated requests.** A panic aborts the WebAssembly instance. `analysis::analyze` rejects the inputs on which `treeknit_core::run` asserts (fewer than two trees, sequence lengths that do not match the trees) and the settings the command line rejects or that have no meaning (zero rounds, negative or non-finite γ, non-positive sequence lengths, duplicate labels) with an error message. The page runs the module in a Web Worker and replaces the worker after a trap, so a panic from a defect costs the analysis but not the page
- **JSON at the boundary.** Requests and results cross the JavaScript boundary as JSON text and are parsed with `serde_json`, whose errors give the expected type and the position. `serde-wasm-bindgen` reports a wrong field type as an opaque `TypeError`
- **Pinned tools.** `wasm-bindgen-cli`, `binaryen` (`wasm-opt`), and Node come from `.config/mise.toml` with locked checksums, and `rust-toolchain.toml` adds the `wasm32-unknown-unknown` target. `wasm-pack` would download unverified `wasm-bindgen` and `wasm-opt` binaries at build time
