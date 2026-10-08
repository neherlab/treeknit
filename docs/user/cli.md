<!-- Written by `just gen` from `treeknit --help-markdown`. Edit the command-line definitions in `packages/treeknit-cli/src/main.rs` and the file list in `packages/treeknit-io/src/output.rs` instead of this file. -->

# Command-line reference

This page holds the help texts of `treeknit` and the list of the files that it writes. `treeknit --help` and `treeknit --help-resolve` print the same texts.

## Options

```text
Infer reassortment between segment trees: maximally compatible clades (MCCs) for every pair of trees, resolved trees, and for two trees an ancestral reassortment graph (ARG)

Usage: treeknit [OPTIONS] [TREE]...

Options:
      --threads <THREADS>
          Worker threads for independent tree pairs (0: all cores). Pairs run in parallel only in rounds without resolution: with `--resolve none`, and in the final round of `strict` and `liberal` with more than two trees [default: 0]
      --verbosity-level <VERBOSITY_LEVEL>
          Verbosity: -1 silent, 0 normal, 1 detailed, 2 debug [default: 0]
  -v, --verbose
          Set verbosity to 1
      --help-resolve
          Explain the --resolve modes and the former method options
  -h, --help
          Print help
  -V, --version
          Print version

Input:
      --session <FILE>  Run the trees and settings of a session file (`treeknit_session.json`, saved by the web app), a path or an https: address. Analysis options change its settings
      --example <ID>    Run a built-in example (see --list-examples). Analysis options change its settings
      --link <URL>      Run the trees and settings of a link of the web app. Analysis options change its settings
      --list-examples   List the built-in examples
  [TREE]...             Trees, one per segment (at least two): Newick files, gzip-compressed or not, or https: addresses. `<label>=<tree>` labels a tree; by default, the file name labels it

Output:
  -o, --outdir <OUTDIR>  Output directory [default: treeknit_results]
      --impute           Write trees with leaves missing from them placed by imputation (`*_imputed.nwk`)
      --auspice-view     Write auspice JSON files for tanglegram visualisation
      --plot             Write SVG figures: a tanglegram of the resolved trees of each pair (`tanglegram_<a>_<b>.svg`) and, for two trees, the ARG (`ARG/arg.svg`)
      --print-link       After the run, print the link of the web app that runs the same analysis, and write the session file into the output directory

Analysis:
  -g, --gamma <GAMMA>          Cost γ of a reassortment, that is of removing an MCC. [default: 2]
      --seq-lengths <LENGTHS>  Sequence lengths of the segments, e.g. 1500,2000 (used by the likelihood tie-break and to weigh the drawn length of a split that some trees lack)
      --n-mcmc-it <N_MCMC_IT>  MCMC steps per leaf of the simulated annealing. [default: 50]
      --resolve <MODE>         How trees are resolved: matched, strict, liberal or none (see --help-resolve) [possible values: none, strict, liberal, matched]
      --pre-resolve            Before inference, add to each tree the splits of other trees compatible with all trees
      --no-pre-resolve         Do not pre-resolve the trees (the default)
      --rounds <ROUNDS>        Rounds of pair inference. [default: 1]
      --no-final-round         With strict or liberal resolution and more than two trees, skip the final round that re-infers MCCs without resolution
      --final-round            Run the final round without resolution (the default)
      --seed <SEED>            Seed of the random number generator, so that a run can be repeated. [default: 1]
      --naive                  Naive MCCs (γ → ∞)
      --no-naive               Infer MCCs (the default)
      --no-likelihood          Do not break ties between configurations with branch lengths
      --likelihood             Break ties between configurations with branch lengths (the default)

Use --help-resolve for details on how trees are resolved. The analysis options have the names of the keys of web app links: --gamma 3 is gamma=3.
```

## Resolution of the trees

```text
Resolution of the trees (--resolve):
  matched  (default) Resolve during inference and with the inferred MCCs, then resolve all trees so that their topologies match within every MCC; where splits of different trees conflict, earlier trees win, and MCCs that cannot be matched are split.
  strict   Resolve during inference and with the inferred MCCs, adding unambiguous splits only.
  liberal  As strict, also adding ambiguous splits.
  none     Do not resolve with MCCs, so MCCs require identical topologies.

Final round (--no-final-round skips it): With strict or liberal resolution and more than two trees, re-infer the MCCs without resolution in a final extra round, because resolving later pairs can invalidate the MCCs of earlier pairs.

--pre-resolve: Before inference, add to each tree the splits of other trees that are compatible with all trees, which is mostly useful without resolution.

Former options are still accepted with their TreeKnit.jl meaning, and reproduce its
results: the method preset depends on the number of trees (--better-MCCs for two,
--better-trees for more), and --rounds counts all rounds (with --better-MCCs and more than
two trees, the default 2 means one resolving round and a final one without). They cannot be
mixed with --resolve, --pre-resolve, --no-final-round or --final-round. Closest current
equivalents:
  --better-trees         --resolve none --pre-resolve
  --better-MCCs          --resolve strict --pre-resolve
  --liberal-resolve      --resolve liberal (in the --better-MCCs preset)
  --no-resolve           --resolve none
  --match-topologies     --resolve matched
  --resolve-all-rounds   resolve in the final round too
```

## Output files

The files go into the results directory, `treeknit_results` unless `--outdir` names another one. In the names, `<tree>` stands for the label of a tree, and `<a>` and `<b>` for the labels of the two trees of a pair.

| File | Content |
|---|---|
| `MCCs.json` | The MCCs of every pair, in the `MCC_dict` format of TreeKnit.jl. When leaves are missing from one tree of a pair, the list `imputed` gives each such leaf, its source tree, the index of the MCC it joined, and whether the placement was ambiguous |
| `MCCs.dat`, `MCCs_<a>_<b>.dat` | The MCCs of a pair in the text format of TreeKnit.jl before 0.5: one MCC per line, leaves separated by commas. `MCCs.dat` for two trees, one file per pair for more |
| `<tree>_resolved.nwk` | The tree after resolution, with polytomies sorted for tanglegrams. Tree files keep the extension of their input file |
| `<tree>_imputed.nwk` | With `--impute`: the resolved tree with the leaves that only other trees have, placed by imputation |
| `auspice_<tree>.json` | With `--auspice-view`: the resolved tree in the JSON format of Auspice, colored by MCC, for tanglegrams in Auspice |
| `ARG/arg.nwk` | Two trees: the ARG in extended Newick, with `[&segments={0,1}]` annotations and the hybrid nodes `#H<i>` |
| `ARG/nodes.dat` | Two trees: the table of the ARG nodes and the tree nodes they stand for |
| `ARG/<tree>_liberal_resolved.nwk` | Two trees: the liberally resolved trees that the ARG was built from |
| `tanglegram_<a>_<b>.svg` | With `--plot`: the tanglegram of the resolved trees of a pair, with MCC colors and the reassortment branches marked |
| `ARG/arg.svg` | With `--plot`, two trees with a built ARG: the ARG, colored by segment, with reassortments as dashed curves |
| `parameters.json` | The options and the seed of the run |
| `log.txt` | The log of the run |
| `treeknit_session.json` | With `--session`, `--example`, `--link`, or `--print-link`: the trees and settings that ran, which the web app opens and `--session` runs |
