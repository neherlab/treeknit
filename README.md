# TreeKnit (Rust)

A port of [TreeKnit.jl](https://github.com/PierreBarrat/TreeKnit.jl), which infers reassortment from segment trees:
- **maximally compatible clades** (MCCs) for every pair of trees;
- **polytomies resolved** using the other trees;
- for two trees, an **ancestral reassortment graph** (ARG) in extended Newick.

The algorithm is described in Barrat-Charlaix, Vaughan & Neher, *PLoS Comput Biol* 18(8): e1010394 (2022).

Compared with the Julia version:
- The trees may have different leaf sets. Inference uses the leaves a pair shares. Afterwards, leaves missing from one tree are attached to the MCC of their neighbours and imputed into that tree.
- Runs are reproducible (`--seed`), and independent tree pairs run in parallel with identical results.
- Every command-line option takes effect. In TreeKnit.jl 0.5.8, `--gamma`, `--n-mcmc-it`, `--no-likelihood`, `--seq-lengths` and `--parallel` are silently ignored.

## Usage

Releases and nightly builds of `main` are on the [releases page](https://github.com/neherlab/treeknit-rs/releases): the CLI for Linux, macOS, and Windows, with the changes of each release in `CHANGELOG.md`. The [web app](https://neherlab.github.io/treeknit-rs/) runs the same analyses in the browser, and a link to it can name trees, settings, a run, and a view, such as [`?example=h3n2-2017&run&view=tanglegram`](https://neherlab.github.io/treeknit-rs/?example=h3n2-2017&run&view=tanglegram); its help page lists the keys, which the analysis options of the command line share. To build the CLI from source:

```sh
cargo build --release
target/release/treeknit ha.nwk na.nwk [more.nwk ...] -o results
```

Main options:

| Option | Effect |
|---|---|
| `-g/--gamma` | cost of a reassortment (default 2) |
| `HA=seg4.nwk`, `https://...` | trees: files (gzip-compressed or not) or https: addresses, with an optional label |
| `--seq-lengths 1700,1400` | segment lengths for the branch-length tie-break and the drawn length of a split that some trees lack |
| `--resolve matched\|strict\|liberal\|none` | how trees are resolved (default `matched`, see below); `--help-resolve` explains the modes |
| `--pre-resolve`, `--rounds`, `--no-final-round` | further control of tree resolution; each flag has its opposite (`--no-pre-resolve`, `--final-round`, `--likelihood`, `--no-naive`) |
| `--naive` | naive MCCs (γ → ∞) |
| `--impute` | also write trees with missing leaves placed |
| `--auspice-view` | auspice JSON for tanglegrams |
| `--plot` | SVG figures: a tanglegram per pair and, for two trees, the ARG |
| `--seed`, `--threads` | reproducibility and parallelism |
| `-v`, `--verbosity-level` | logging detail |
| `--session treeknit_session.json` | run the trees and settings of a session file saved by the web app, a path or an https: address; analysis options change its settings |
| `--example h3n2-2017`, `--list-examples` | run a built-in example of the web app, or list them |
| `--link <url>`, `--print-link` | run a link of the web app, or print the link that runs the analysis in the browser |

Output in the results directory:

| File | Content |
|---|---|
| `MCCs.json` | MCCs per pair, in the `MCC_dict` format of TreeKnit.jl. If any leaves were missing from one tree of a pair, an `imputed` list gives each such leaf, its source tree, the index of the MCC it joined, and whether the placement was ambiguous. |
| `MCCs.dat` | the same MCCs in the text format of TreeKnit.jl before 0.5: one MCC per line, leaves separated by commas. With more than two trees: `MCCs_<a>_<b>.dat` per pair. |
| `<tree>_resolved.nwk` | input trees after resolution, with polytomies sorted for tanglegrams |
| `<tree>_imputed.nwk` | with `--impute`: resolved trees with missing leaves placed |
| `ARG/arg.nwk` | two trees only: ARG in extended Newick, `[&segments={0,1}]` annotations, hybrids `#Hi` |
| `ARG/nodes.dat` | ARG node ↔ tree node table |
| `ARG/<tree>_liberal_resolved.nwk` | the trees the ARG was built from |
| `tanglegram_<a>_<b>.svg` | with `--plot`: the tanglegram of the resolved trees of each pair, with MCC colors and reassortment branches marked |
| `ARG/arg.svg` | with `--plot`, two trees with a built ARG: the ARG, colored by segment, with reassortments as dashed curves |
| `parameters.json`, `log.txt` | parameters and log of the run |
| `treeknit_session.json` | with `--session`, `--example`, `--link`, or `--print-link`: the trees and settings that ran |

## Resolving trees

`--resolve` chooses how trees are resolved. The default is the same for any number of trees.

| Mode | What happens |
|---|---|
| `matched` (default) | Resolve during inference and with the inferred MCCs. Then resolve all trees so that, for every pair and MCC, the two trees restricted to the MCC's leaves have the same topology. |
| `strict` | Resolve during inference and with the inferred MCCs, unambiguous splits only. |
| `liberal` | As `strict`, also adding ambiguous splits (the placement of other MCCs is chosen arbitrarily). |
| `none` | No resolution with MCCs; MCCs then require identical topologies. |

`--pre-resolve` adds to each tree, before inference, the splits of other trees that are
compatible with *all* trees. One tree that reassorted in a region therefore blocks resolution
there for everyone. It is mostly useful with `--resolve none`.

With `strict` or `liberal` and more than two trees, the MCCs are re-inferred without resolution
in a final extra round, since resolving later pairs can invalidate earlier pairs' MCCs
(`--no-final-round` skips it). `matched` doesn't need that round, because matching enforces
consistency itself.

**How matching works**
- The splits each tree has inside an MCC are inserted into the other tree of the pair.
  Only the MCC's leaves are considered, so branches of other MCCs may attach anywhere.
- Pairs are processed in argument order, repeatedly until nothing changes, so splits pass
  along chains of shared regions.
- A split is inserted only if compatible with what the tree already has, so splits of trees
  given earlier win conflicts. No input split is ever removed.
- Conflicting splits from different trees in the same shared region mean the pairwise MCCs are
  not mutually consistent. Such an MCC is replaced by the maximal clades on which its two trees
  agree, i.e. more reassortments, and this is logged.

**Former options** are still accepted, with a deprecation warning and their TreeKnit.jl
meaning, and reproduce its results. With them:
- the method preset depends on the number of trees (`--better-MCCs` for two, `--better-trees`
  for more);
- `--rounds` counts all rounds; with `--better-MCCs` and more than two trees the default 2 means
  one resolving round and a final one without;
- `--resolve-all-rounds` makes the final round resolve too.

They cannot be mixed with `--resolve`, `--pre-resolve` or `--no-final-round`. Closest current
equivalents:

| Former | Now |
|---|---|
| `--better-trees` | `--resolve none --pre-resolve` |
| `--better-MCCs` | `--resolve strict --pre-resolve` |
| `--liberal-resolve` | `--resolve liberal` (in the `--better-MCCs` preset) |
| `--no-resolve` | `--resolve none` |
| `--no-pre-resolve` | the default |
| `--resolve-all-rounds` | resolve in the final round too |

## Layout

| Package | Content |
|---|---|
| `treeknit-core` | Algorithms, no IO. Arena trees and bitset clades (`tree`, `bits`), naive MCCs, split graph energy and likelihood, simulated annealing, pair inference, resolution, the K-tree pipeline, imputation of missing leaves, ARG construction. |
| `treeknit-io` | Newick, MCC JSON, extended Newick, node table, auspice JSON. |
| `util-newick` | Newick and NEXUS reader and writer in the common dialects, a copy of the crate of TreeTime (`packages/util-newick/README.md`). |
| `treeknit-cli` | The `treeknit` binary. |
| `treeknit-wasm` | WebAssembly bindings of the core, with TypeScript declarations (`packages/treeknit-wasm/README.md`). |
| `web` | The web app: React, runs analyses in the browser (`packages/web/README.md`). |

`data/` holds real segment trees, ready as TreeKnit input and offered in the **Load example** menu of the web app: influenza A/H3N2 (HA and NA pairs, and one set of four segments with 1997 strains), influenza A/H5N1 time trees of the PA, PB1, and PB2 segments, and Andes virus trees of the S, M, and L segments.

## Development

[`docs/dev/developer_guide.md`](docs/dev/developer_guide.md) describes the build container, the `just` recipes, the checks that must pass before merging, and the build profiles.

## Testing against the Julia implementation

`ref/` holds Julia scripts that write reference outputs of TreeKnit.jl 0.5.8 to `fixtures/`, using the branch `fix/issues-from-rust-port` of the Julia code, which fixes the bugs found during the port. The cases come from the docs, the test suite, real data (NY H3N2, the examples) and ARGTools simulations; `ref/README.md` has the commands.

- `cargo test` checks every deterministic function exactly against these fixtures (about 5,000 checks): naive MCCs, K-tree resolution, strict and liberal resolution with MCCs, node→MCC maps, polytomy sorting, energies and likelihoods of given configurations, and ARGs. It also compares annealing outcomes with the 20 seeded Julia runs per case.
- `cargo run --release -p treeknit-io --example accuracy [drop]` compares accuracy against the true MCCs of the simulated cases, for Julia and Rust, measured as scaled variation of information. It also drops a fraction of leaves per tree and reports how often the dropped leaves are placed with their true MCC.

## Deliberate differences from TreeKnit.jl

These differ from the released TreeKnit.jl 0.5.8. The first two are also fixed on the Julia branch `fix/issues-from-rust-port`, against which the fixtures are generated.

- **Different defaults.** The defaults are `--resolve matched` without pre-resolution, for any number of trees. TreeKnit.jl's defaults are `--better-MCCs` (two trees) and `--better-trees` (more), which remain available.
- **Strict resolution adds certain splits that 0.5.8 rejects.** A polytomy sister holding leaves of an MCC that also has leaves outside the polytomy must attach at the polytomy node, since MCCs are connected. 0.5.8 decides this only from the node→MCC map, which often assigns no MCC at such polytomies. Splits stay rejected when a sister consists only of MCCs inside the polytomy, which may be nested in the new clade.

- **Missing branch lengths contribute 0 to the likelihood tie-break.** In Julia a single missing length makes the likelihood `missing`. Julia then prefers those configurations, because `maximum` over a vector containing `missing` is `missing`. This happens whenever resolved nodes, which get length 0, meet input trees without lengths.
- **Negative and non-finite branch lengths count as missing in the likelihood tie-break.** The Poisson model of the tie-break has no likelihood for them. Such lengths occur in time trees and neighbor-joining trees. Julia stops with `DomainError` from `log` of a negative number.
- **Node labels are deterministic counters** (`ARGNode_17`, `Singleton_3`) instead of random strings.
- **The ARG is built from the resolved output trees,** not from the raw inputs. The liberally resolved trees written next to it are exactly the ones the ARG was built from, including inserted singletons. In Julia, `nodes.dat` refers to singleton nodes that are absent from those files.
- **An invalid branch length is a parse error.** TreeKnit.jl reads a length that is not a number, such as the root length `:0.R` of its own tests, as missing. The port reports the error with its line and column, because a broken length usually means a broken file. The comparison with TreeKnit.jl reads the trees of `test_srg_2` without that root length, which both implementations ignore.
- **ARG labels are quoted.** `ARG/arg.nwk` quotes labels that Newick reads differently without quotes, such as `'B,1'`. TreeKnit.jl writes them as they are, which makes the file unreadable for such labels.
- **Parallelism is used only where it is race-free:** independent pairs, and rounds without resolution. TreeKnit.jl's parallel mode resolves shared trees concurrently.
