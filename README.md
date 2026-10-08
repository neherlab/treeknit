# TreeKnit

<p align="center">
  <a href="https://github.com/neherlab/treeknit/actions/workflows/ci.yml">
    <img src="https://github.com/neherlab/treeknit/actions/workflows/ci.yml/badge.svg" alt="CI" />
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/github/license/neherlab/treeknit" alt="License" />
  </a>
  <a href="https://doi.org/10.1371/journal.pcbi.1010394">
    <img src="https://img.shields.io/badge/DOI-10.1371%2Fjournal.pcbi.1010394-blue" alt="DOI" />
  </a>
</p>

<p align="center">
  <a href="https://github.com/neherlab/treeknit/releases">
    <img src="https://img.shields.io/github/v/release/neherlab/treeknit?logo=github&label=release" alt="GitHub release" />
  </a>
  <a href="https://pypi.org/project/treeknit/">
    <img src="https://img.shields.io/pypi/v/treeknit?logo=pypi&logoColor=white&label=pypi" alt="PyPI" />
  </a>
  <a href="https://anaconda.org/bioconda/treeknit">
    <img src="https://img.shields.io/conda/vn/bioconda/treeknit?logo=anaconda&label=bioconda" alt="Bioconda" />
  </a>
  <a href="https://hub.docker.com/r/neherlab/treeknit">
    <img src="https://img.shields.io/docker/v/neherlab/treeknit?logo=docker&label=docker&sort=semver" alt="Docker" />
  </a>
</p>

<p align="center">
  <a href="https://github.com/neherlab/treeknit/commits">
    <img src="https://img.shields.io/github/last-commit/neherlab/treeknit?logo=github" alt="GitHub last commit" />
  </a>
  <a href="https://github.com/neherlab/treeknit/commits">
    <img src="https://img.shields.io/github/commit-activity/w/neherlab/treeknit" alt="GitHub commit activity" />
  </a>
  <a href="https://github.com/neherlab/treeknit/graphs/contributors">
    <img src="https://img.shields.io/github/contributors/neherlab/treeknit?logo=github&label=developers" alt="GitHub contributors" />
  </a>
</p>

<p align="center">
  <a href="https://github.com/neherlab/treeknit/releases">
    <img src="https://img.shields.io/github/downloads/neherlab/treeknit/total?logo=github&label=github%20downloads" alt="GitHub downloads" />
  </a>
  <a href="https://pypi.org/project/treeknit/">
    <img src="https://img.shields.io/pypi/dm/treeknit?logo=pypi&logoColor=white&label=pypi%20downloads" alt="PyPI downloads" />
  </a>
  <a href="https://anaconda.org/bioconda/treeknit">
    <img src="https://img.shields.io/conda/dn/bioconda/treeknit?logo=anaconda&label=bioconda%20downloads" alt="Bioconda downloads" />
  </a>
  <a href="https://hub.docker.com/r/neherlab/treeknit">
    <img src="https://img.shields.io/docker/pulls/neherlab/treeknit?logo=docker&label=docker%20pulls" alt="Docker pulls" />
  </a>
  <a href="https://github.com/neherlab/treeknit/stargazers">
    <img src="https://img.shields.io/github/stars/neherlab/treeknit?style=flat&logo=github" alt="GitHub stars" />
  </a>
</p>

TreeKnit infers reassortment from the trees of the segments of a virus, such as the HA and NA trees of influenza:

- **Maximally compatible clades** (MCCs) for every pair of trees: the largest sets of leaves whose subtrees have the same topology in both trees. Leaves of one MCC share their history in both segments, and a reassortment separates two MCCs
- **Resolved trees**: polytomies (nodes with more than two children) resolved with the splits of the other trees
- For two trees, an **ancestral reassortment graph** (ARG) in extended Newick: one network that holds the histories of both segments, with a hybrid node for each reassortment

This is a Rust port of [TreeKnit.jl](https://github.com/PierreBarrat/TreeKnit.jl).

## Quick start

Install TreeKnit and run it on two trees:

```sh
pip install treeknit
treeknit ha.nwk na.nwk -o results
```

The results directory then holds:

- `MCCs.json`: the MCCs
- `ha_resolved.nwk`, `na_resolved.nwk`: the resolved trees
- `ARG/arg.nwk`: the ARG
- with `--plot`: SVG figures of the tanglegram and the ARG

## Install

The links download the binaries of the latest [release](https://github.com/neherlab/treeknit/releases).

| Channel | Install |
|---|---|
| Linux x86_64 | [glibc](https://github.com/neherlab/treeknit/releases/latest/download/treeknit-x86_64-unknown-linux-gnu), [musl](https://github.com/neherlab/treeknit/releases/latest/download/treeknit-x86_64-unknown-linux-musl)\* |
| Linux ARM64 | [glibc](https://github.com/neherlab/treeknit/releases/latest/download/treeknit-aarch64-unknown-linux-gnu), [musl](https://github.com/neherlab/treeknit/releases/latest/download/treeknit-aarch64-unknown-linux-musl)\* |
| macOS | [Apple Silicon](https://github.com/neherlab/treeknit/releases/latest/download/treeknit-aarch64-apple-darwin), [Intel](https://github.com/neherlab/treeknit/releases/latest/download/treeknit-x86_64-apple-darwin) |
| Windows | [x86_64](https://github.com/neherlab/treeknit/releases/latest/download/treeknit-x86_64-pc-windows-gnu.exe) |
| pip | `pip install treeknit` |
| Conda | `conda install -c bioconda treeknit`\*\* |
| Docker | `docker pull neherlab/treeknit` |
| From source | `cargo build --profile dist -p treeknit-cli`, writes `target/dist/treeknit` |
| Docs | [usage](docs/user/usage.md), [command-line reference](docs/user/cli.md), [for users of TreeKnit.jl](docs/user/treeknit-jl.md), [changelog](CHANGELOG.md) |

<sub>\* glibc builds need glibc 2.17 or newer. musl builds are static and run on any Linux distribution.</sub>

<sub>\*\* Bioconda releases need manual approval and can be delayed.</sub>

## Citation

The method is described in:

> Barrat-Charlaix P, Vaughan TG, Neher RA (2022). TreeKnit: Inferring ancestral reassortment graphs of influenza viruses. _PLOS Computational Biology_ 18(8): e1010394. https://doi.org/10.1371/journal.pcbi.1010394

## Contributing

Report bugs and ask questions in the [issue tracker](https://github.com/neherlab/treeknit/issues). [`CONTRIBUTING.md`](CONTRIBUTING.md) explains how to build, test, and change the code.

## License

[MIT](LICENSE)
