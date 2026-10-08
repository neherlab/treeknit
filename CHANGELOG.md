# Changelog

## 1.0.0

First release of TreeKnit, a Rust port of [TreeKnit.jl](https://github.com/PierreBarrat/TreeKnit.jl), which infers reassortment from segment trees.

- **MCCs**: maximally compatible clades for every pair of trees, in the `MCCs.json` and `MCCs.dat` formats of TreeKnit.jl
- **Resolution**: polytomies resolved using the other trees (`--resolve matched|strict|liberal|none`)
- **ARG**: for two trees, an ancestral reassortment graph in extended Newick
- **Different leaf sets**: inference uses the leaves a pair shares, and leaves missing from one tree are imputed into it (`--impute`)
- **Figures**: `--plot` writes an SVG tanglegram of each pair and, for two trees, an SVG figure of the ARG
- **Reproducible runs**: `--seed` fixes the result, whatever the number of threads
- **Input checks**: every error in the trees and settings is reported before a run, one per line
- **Web app**: the same analyses in the browser at [neherlab.github.io/treeknit](https://neherlab.github.io/treeknit/). `--session`, `--example`, `--link`, and `--print-link` move an analysis between the web app and the command line
- **Install**:
  - prebuilt binaries for Linux, macOS, and Windows
  - `pip install treeknit`
  - `conda install -c bioconda treeknit`
  - `docker pull neherlab/treeknit`
- **For users of TreeKnit.jl**: the [page for users of TreeKnit.jl](https://github.com/neherlab/treeknit/blob/main/docs/user/treeknit-jl.md) lists every difference. The main ones:
  - the default resolution is `matched` for any number of trees
  - `--gamma`, `--n-mcmc-it`, `--no-likelihood`, and `--seq-lengths` take effect
  - the former method options still work
- **Documentation**: the [command-line reference](https://github.com/neherlab/treeknit/blob/main/docs/user/cli.md) lists every option and output file
