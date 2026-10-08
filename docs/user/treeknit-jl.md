# Differences from TreeKnit.jl

TreeKnit is a port of [TreeKnit.jl](https://github.com/PierreBarrat/TreeKnit.jl) 0.5.8. This page is for users of TreeKnit.jl.

## Additions

- **Trees with different leaf sets**: TreeKnit.jl requires the same leaves in every tree
  - Inference of a pair uses the leaves that both trees share
  - Each pair must share at least two leaves
  - Leaves missing from one tree are attached to the MCC of their neighbors and imputed into that tree (`--impute`)
- **Reproducible runs**: `--seed` sets the seed, and the results do not depend on the number of threads
- **Options that take effect**: TreeKnit.jl 0.5.8 accepts these options but does not use them. The port applies them
  - `--gamma`: TreeKnit.jl always uses γ = 2
  - `--n-mcmc-it`: TreeKnit.jl always uses 50 steps per leaf
  - `--no-likelihood`: TreeKnit.jl always uses the likelihood tie-break
  - `--seq-lengths`: TreeKnit.jl always uses sequence lengths of 1
- **`--parallel`**: accepted and without effect, because independent pairs run in parallel wherever that gives the same result
- **Web app**: the [web app](https://neherlab.github.io/treeknit/) runs the same analyses in the browser. `--session`, `--example`, `--link`, and `--print-link` move an analysis between the web app and the command line
- **Figures**: `--plot` draws a tanglegram per pair and, for two trees, the ARG

## Former options

The method options of TreeKnit.jl are still accepted, with a deprecation warning and their TreeKnit.jl meaning, and reproduce its results when `--gamma`, `--n-mcmc-it`, `--no-likelihood`, and `--seq-lengths` keep their defaults. With them:

- the method preset depends on the number of trees: `--better-MCCs` for two, `--better-trees` for more
- `--rounds` counts all rounds. With `--better-MCCs` and more than two trees, the default 2 means one resolving round and a final one without
- `--resolve-all-rounds` makes the final round resolve too

They cannot be mixed with `--resolve`, `--pre-resolve`, `--no-final-round`, or `--final-round`. The closest current equivalents:

| Former                 | Now                                                 |
| ---------------------- | --------------------------------------------------- |
| `--better-trees`       | `--resolve none --pre-resolve`                      |
| `--better-MCCs`        | `--resolve strict --pre-resolve`                    |
| `--liberal-resolve`    | `--resolve liberal` (in the `--better-MCCs` preset) |
| `--no-resolve`         | `--resolve none`                                    |
| `--no-pre-resolve`     | the default                                         |
| `--resolve-all-rounds` | resolve in the final round too                      |

## Deliberate differences from TreeKnit.jl

These differ from the released TreeKnit.jl 0.5.8. The TreeKnit.jl branch `fix/issues-from-rust-port`, which writes the reference outputs of the tests ([`ref/README.md`](../../ref/README.md)), also fixes the strict resolution and the missing branch lengths.

- **Defaults**: the defaults are `--resolve matched` without pre-resolution, for any number of trees. TreeKnit.jl's defaults are `--better-MCCs` (two trees) and `--better-trees` (more), which remain available
- **Strict resolution**: it adds certain splits that 0.5.8 rejects. A polytomy sister holding leaves of an MCC that also has leaves outside the polytomy must attach at the polytomy node, since MCCs are connected. 0.5.8 decides this only from the node-to-MCC map, which often assigns no MCC at such polytomies. Splits stay rejected when a sister consists only of MCCs inside the polytomy, which may be nested in the new clade
- **Missing branch lengths**: they contribute 0 to the likelihood tie-break. In Julia a single missing length makes the likelihood `missing`. Julia then prefers those configurations, because `maximum` over a vector containing `missing` is `missing`. This happens whenever resolved nodes, which get length 0, meet input trees without lengths
- **Negative and non-finite branch lengths**: they count as missing in the likelihood tie-break. The Poisson model of the tie-break has no likelihood for them. Such lengths occur in time trees and neighbor-joining trees. Julia stops with `DomainError` from `log` of a negative number
- **Node labels**: deterministic counters (`ARGNode_17`, `Singleton_3`) instead of random strings
- **Input of the ARG**: the resolved output trees, not the raw inputs. The liberally resolved trees written next to it are exactly the ones the ARG was built from, including inserted singletons. In Julia, `nodes.dat` refers to singleton nodes that are absent from those files
- **Invalid branch lengths**: a parse error. TreeKnit.jl reads a length that is not a number, such as the root length `:0.R` of its own tests, as missing. The port reports the error with its line and column, because a broken length usually means a broken file. The comparison with TreeKnit.jl reads the trees of `test_srg_2` without that root length, which both implementations ignore
- **ARG labels**: `ARG/arg.nwk` quotes labels that Newick reads differently without quotes, such as `'B,1'`. TreeKnit.jl writes them as they are, which makes the file unreadable for such labels
- **Parallelism**: only where it is race-free, for independent pairs in rounds without resolution. TreeKnit.jl's parallel mode resolves shared trees concurrently
