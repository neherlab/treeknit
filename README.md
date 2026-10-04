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

```sh
cargo build --release
target/release/treeknit ha.nwk na.nwk [more.nwk ...] -o results
```

Main options:

| Option | Effect |
|---|---|
| `-g/--gamma` | cost of a reassortment (default 2) |
| `--better-trees` / `--better-MCCs` | method presets; `--help-defaults` explains them |
| `--seq-lengths "1700 1400"` | segment lengths for the branch-length tie-break |
| `--no-resolve`, `--liberal-resolve`, `--no-pre-resolve`, `--resolve-all-rounds`, `--rounds` | control tree resolution |
| `--match-topologies` | resolve all trees so that their topologies match within every MCC; earlier trees take precedence where splits conflict, and MCCs that cannot match are split (see below) |
| `--naive` | naive MCCs (γ → ∞) |
| `--impute` | also write trees with missing leaves placed |
| `--auspice-view` | auspice JSON for tanglegrams |
| `--seed`, `--threads` | reproducibility and parallelism |
| `-v`, `--verbosity-level` | logging detail |

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
| `parameters.json`, `log.txt` | parameters and log of the run |

## Matching topologies

With `--match-topologies`, the output trees agree within shared regions. For every pair of
trees and every MCC, the two trees restricted to the MCC's leaves have the same topology.

- **How:** after inference (with resolution in every round), the splits each tree has inside
  an MCC are inserted into the other tree of the pair. Only the MCC's leaves are considered,
  so branches of other MCCs may attach anywhere. Pairs are processed in argument order,
  repeatedly until nothing changes, so splits pass along chains of shared regions.
- **Precedence:** a split is inserted only if compatible with what the tree already has, so
  splits of trees given earlier win conflicts. No input split is ever removed.
- **Conflicts:** conflicting splits from different trees in the same shared region mean the
  pairwise MCCs are not mutually consistent. Such an MCC is replaced by the maximal clades on
  which its two trees agree, i.e. more reassortments, and this is logged.
- **Compared with pre-resolution:** a tree that has reassorted relative to the others in some
  region doesn't block resolution there, since only trees sharing the region take part.
  Combining it with `--no-pre-resolve` removes pre-resolution's veto entirely.

## Layout

| Crate | Content |
|---|---|
| `treeknit-core` | Algorithms, no IO. Arena trees and bitset clades (`tree`, `bits`), naive MCCs, split graph energy and likelihood, simulated annealing, pair inference, resolution, the K-tree pipeline, imputation of missing leaves, ARG construction. |
| `treeknit-io` | Newick, MCC JSON, extended Newick, node table, auspice JSON. |
| `treeknit-cli` | The `treeknit` binary. |

## Testing against the Julia implementation

`ref/` holds Julia scripts that write reference outputs of TreeKnit.jl 0.5.8 to `fixtures/`, using the branch `fix/issues-from-rust-port` of the Julia code, which fixes the bugs found during the port. The cases come from the docs, the test suite, real data (NY H3N2, the examples) and ARGTools simulations; `ref/README.md` has the commands.

- `cargo test` checks every deterministic function exactly against these fixtures (about 5,000 checks): naive MCCs, K-tree resolution, strict and liberal resolution with MCCs, node→MCC maps, polytomy sorting, energies and likelihoods of given configurations, and ARGs. It also compares annealing outcomes with the 20 seeded Julia runs per case.
- `cargo run --release -p treeknit-io --example accuracy [drop]` compares accuracy against the true MCCs of the simulated cases, for Julia and Rust, measured as scaled variation of information. It also drops a fraction of leaves per tree and reports how often the dropped leaves are placed with their true MCC.

## Deliberate differences from TreeKnit.jl

These differ from the released TreeKnit.jl 0.5.8. The first two are also fixed on the Julia branch `fix/issues-from-rust-port`, against which the fixtures are generated.

- **Strict resolution adds certain splits that 0.5.8 rejects.** A polytomy sister holding leaves of an MCC that also has leaves outside the polytomy must attach at the polytomy node, since MCCs are connected. 0.5.8 decides this only from the node→MCC map, which often assigns no MCC at such polytomies. Splits stay rejected when a sister consists only of MCCs inside the polytomy, which may be nested in the new clade.

- **Missing branch lengths contribute 0 to the likelihood tie-break.** In Julia a single missing length makes the likelihood `missing`. Julia then prefers those configurations, because `maximum` over a vector containing `missing` is `missing`. This happens whenever resolved nodes, which get length 0, meet input trees without lengths.
- **Node labels are deterministic counters** (`ARGNode_17`, `Singleton_3`) instead of random strings.
- **The ARG is built from the resolved output trees,** not from the raw inputs. The liberally resolved trees written next to it are exactly the ones the ARG was built from, including inserted singletons. In Julia, `nodes.dat` refers to singleton nodes that are absent from those files.
- **Parallelism is used only where it is race-free:** independent pairs, and rounds without resolution. TreeKnit.jl's parallel mode resolves shared trees concurrently.
