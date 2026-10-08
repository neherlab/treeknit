# Usage

`treeknit --help` lists the options, and `treeknit --help-resolve` explains the resolution of the trees. The [command-line reference](cli.md) holds both texts and the list of output files.

## Input

- **Trees**: two or more Newick files, one tree per segment, gzip-compressed or not, or https: addresses
- **Labels**: the file name without its extension labels a tree and names its output files. `HA=seg4.nwk` gives the label `HA`
- **Leaves**: the trees are compared by leaf name, and may have different leaf sets
  - Inference of a pair uses the leaves that both trees share
  - Each pair must share at least two leaves
  - Leaves missing from one tree are attached to the MCC of their neighbors. `--impute` writes the trees with these leaves placed
- **Branch lengths**: optional. They break ties between configurations with the same number of reassortments, together with the segment lengths of `--seq-lengths`

## Web app

- The [web app](https://neherlab.github.io/treeknit/) runs the same analyses in the browser
- The analysis options have the names of the keys of web app links: `--gamma 3` is `gamma=3`
- `--print-link`: print the link that runs the same analysis in the web app
- `--session`, `--example`, `--link`: run a session file, an example, or a link of the web app

## Resolving trees

`--resolve` chooses how trees are resolved. The default is the same for any number of trees.

| Mode                | What happens                                                                                                                                                                      |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `matched` (default) | Resolve during inference and with the inferred MCCs. Then resolve all trees so that, for every pair and MCC, the two trees restricted to the MCC's leaves have the same topology. |
| `strict`            | Resolve during inference and with the inferred MCCs, unambiguous splits only.                                                                                                     |
| `liberal`           | As `strict`, also adding ambiguous splits (the placement of other MCCs is chosen arbitrarily).                                                                                    |
| `none`              | No resolution with MCCs; MCCs then require identical topologies.                                                                                                                  |

`--pre-resolve` adds to each tree, before inference, the splits of other trees that are compatible with _all_ trees. One tree that reassorted in a region therefore blocks resolution there for everyone. It is mostly useful with `--resolve none`.

With `strict` or `liberal` and more than two trees, the MCCs are re-inferred without resolution in a final extra round, since resolving later pairs can invalidate earlier pairs' MCCs (`--no-final-round` skips it). `matched` doesn't need that round, because matching enforces consistency itself.

### How matching works

- The splits each tree has inside an MCC are inserted into the other tree of the pair. Only the MCC's leaves are considered, so branches of other MCCs may attach anywhere
- Pairs are processed in argument order, repeatedly until nothing changes, so splits pass along chains of shared regions
- A split is inserted only if compatible with what the tree already has, so splits of trees given earlier win conflicts. No input split is ever removed
- Conflicting splits from different trees in the same shared region mean the pairwise MCCs are not mutually consistent. Such an MCC is replaced by the maximal clades on which its two trees agree, which adds reassortments, and the log reports it

## Example data

`data/` holds real segment trees, ready as TreeKnit input:

- Influenza A/H3N2: HA and NA pairs, and one set of four segments with 1997 strains
- Influenza A/H5N1: time trees of the PA, PB1, and PB2 segments
- Andes virus: trees of the S, M, and L segments

`treeknit --list-examples` lists them, and `treeknit --example <id>` runs one. The web app offers them in its **Examples** menu.
