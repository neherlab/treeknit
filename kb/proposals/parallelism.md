# Parallel computation

The command line infers independent tree pairs on several threads. The web app runs every analysis on one thread. This proposal records which parts of a run can use more threads, which parts cannot, and what threads in the browser would cost.

## Current state

- **Parallel pairs**: in a round without resolution, `rayon` infers all pairs at the same time [packages/treeknit-core/src/pipeline.rs#L231-L255](../../packages/treeknit-core/src/pipeline.rs#L231-L255). `--threads` sets the number of threads, 0 for all cores [packages/treeknit-cli/src/main.rs#L242-L244](../../packages/treeknit-cli/src/main.rs#L242-L244). Each pair seeds its own generator, so the thread count does not change the result
- **Resolving rounds run in sequence**: each pair resolves trees that later pairs read (`fn round_mode()` [packages/treeknit-core/src/pipeline.rs#L90-L93](../../packages/treeknit-core/src/pipeline.rs#L90-L93)). The default resolution `Matched` [packages/treeknit-core/src/options.rs#L79](../../packages/treeknit-core/src/options.rs#L79) resolves in every round. The extra round without resolution exists only for `strict` and `liberal` with more than two trees (`fn schedule()` [packages/treeknit-core/src/pipeline.rs#L82-L86](../../packages/treeknit-core/src/pipeline.rs#L82-L86)). A run with default settings therefore uses one thread. With `--example sim_k3_n50_r0.05`, the three pairs start 15 ms apart with the default settings, and within 0.1 ms of each other with `--resolve none`
- **Two trees**: a round has one pair, so no pair runs in parallel in any mode
- **Upper bound**: a round has $K(K-1)/2$ pairs, where $K$ is the number of trees, and its longest pair bounds its time. Four trees of 1997 leaves with `--better-trees` (no resolution) take 7 to 11 s instead of 29 to 38 s ([`performance.md`](../reports/performance.md#parallel-pairs))
- **Web app**: one thread ([`kb/decisions/web-app.md`](../decisions/web-app.md)). Its Web Workers keep the page responsive and let Cancel stop a run at once. They do not divide an analysis

## Parts that stay sequential

- **Annealing chain**: every Metropolis step starts from the state of the previous step (`fn Chain.mcmc()` [packages/treeknit-core/src/anneal.rs#L95-L121](../../packages/treeknit-core/src/anneal.rs#L95-L121)). One step took 15 to 17 µs on 1997 leaves ([`performance.md`](../reports/performance.md#summary)), which is too short to divide between threads. Most of the inference time is spent here
- **Prune-and-repeat loop**: each iteration of `pub fn infer_pair()` works on the trees that the previous iteration pruned [packages/treeknit-core/src/pair.rs#L51-L81](../../packages/treeknit-core/src/pair.rs#L51-L81)
- **Resolution and sorting**: pre-resolution, resolving rounds, and the polytomy sort change shared trees pair by pair, in the order (0,1), (0,2), ..., (1,2), ... Pairs on separate trees, such as (0,1) and (2,3), are not neighbors in this order, so running them together also changes the result. The port does not resolve shared trees concurrently as TreeKnit.jl does ([`docs/user/treeknit-jl.md`](../../docs/user/treeknit-jl.md#deliberate-differences-from-treeknitjl))
- **Topology matching**: the passes of `pub fn match_topologies()` change shared trees until nothing changes [packages/treeknit-core/src/pipeline.rs#L285-L302](../../packages/treeknit-core/src/pipeline.rs#L285-L302). At 10,000 leaves this step takes 198 s of a 201 s run because its algorithm is cubic ([`M-matched-topologies-slow-on-large-trees.md`](../issues/M-matched-topologies-slow-on-large-trees.md)). A faster algorithm ([`tree-query-indices.md`](tree-query-indices.md)) gains more than threads

## Options

### Command line

- **Read-only steps after the last round**: `fn attach_pair()` [packages/treeknit-core/src/pipeline.rs#L571](../../packages/treeknit-core/src/pipeline.rs#L571) for each pair and the check loop of `pub fn match_topologies()` [packages/treeknit-core/src/pipeline.rs#L303-L324](../../packages/treeknit-core/src/pipeline.rs#L303-L324) only read the trees, so `par_iter` gives the same result. The gain is not measured, and it is probably small because these steps are fast compared with the steps above
- **Annealing repetitions**: the `sa_rep` runs of `pub fn optimize()` are independent chains [packages/treeknit-core/src/anneal.rs#L168](../../packages/treeknit-core/src/anneal.rs#L168). Each run needs its own generator, so the result of a given seed changes. `sa_rep` is fixed at 1 and no option sets it [packages/treeknit-core/src/options.rs#L85](../../packages/treeknit-core/src/options.rs#L85), so this helps only if such an option is added
- **Independent runs**: runs over several seeds or parameter values can run as separate processes, with no code change
- **Rejected**: parallel pairs in resolving rounds and parallel steps inside one chain, for the reasons in [Parts that stay sequential](#parts-that-stay-sequential)

### Web app

- **One thread (recommended for now)**: the current state. Threads would speed up only the rounds that already run in parallel on the command line
- **WebAssembly threads**: shared memory needs a cross-origin isolated page, which sends `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp` [[doc](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Cross-Origin-Opener-Policy#features_that_depend_on_cross-origin_isolation)]. The Rust build needs the nightly toolchain, a standard library rebuilt with atomics, and a thread pool such as `wasm-bindgen-rayon` 1.3.0 (2024-12-21) [[doc](https://github.com/RReverser/wasm-bindgen-rayon/blob/4eea1fa55a965ad516ef9f9e9449704c7eac91c5/README.md?plain=1#L95-L108)]. Costs:
  - The release workflow deploys the app to GitHub Pages [.github/workflows/release.yml#L380-L414](../../.github/workflows/release.yml#L380-L414), which cannot set response headers [[issue](https://github.com/orgs/community/discussions/13309#:~:text=No%20ETA%20at%20the%20moment)]. The app would need another host, or a service worker that adds the headers
  - `same-origin` cuts the link between the app and a page of another origin that opened it [[doc](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Cross-Origin-Opener-Policy#:~:text=any%20references%20between%20the%20new%20document%20and%20its%20opener%20are%20severed)], so the `?from=opener` launch stops working [packages/web/src/help/LinkHelp.tsx#L112-L113](../../packages/web/src/help/LinkHelp.tsx#L112-L113)
- **One worker per pair**: separate workers without shared memory, each with its own module instance. This needs a Rust operation that infers one pair, and it has the same limit as the command line
