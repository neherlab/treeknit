# Changelog

Each release has a section `## <version>`, which becomes the notes of its release on the releases page. Changes that are not released yet go under `## Unreleased`; `dev/release <version>` renames that heading when it prepares the release.

## Unreleased

First release of the Rust port of [TreeKnit.jl](https://github.com/PierreBarrat/TreeKnit.jl), which infers reassortment from segment trees.

- **MCCs**: maximally compatible clades for every pair of trees, compatible with the `MCCs.json` and `MCCs.dat` output of TreeKnit.jl
- **Resolution**: polytomies resolved using the other trees (`--resolve matched|strict|liberal|none`)
- **ARG**: for two trees, an ancestral reassortment graph in extended Newick
- **Different leaf sets**: inference uses the leaves a pair shares; leaves missing from one tree are imputed into it
- **Input checks**: the command line and the web app check the trees and settings with the same rules and report every error before a run: a negative or non-finite γ, sequence lengths that are not positive, zero rounds or MCMC steps, a seed above 2^53 - 1, tree labels that cannot be file names, pairs whose output files would get the same name, and pairs that share fewer than two leaves
- **Reproducible runs**: `--seed` fixes the result, and independent tree pairs run in parallel with identical results
- **Every option takes effect**: in TreeKnit.jl 0.5.8, `--gamma`, `--n-mcmc-it`, `--no-likelihood`, `--seq-lengths` and `--parallel` are silently ignored
- **Prebuilt binaries**: the CLI for Linux (glibc and static musl), macOS, and Windows
- **Web app**: the same analyses in the browser, on the WebAssembly build of the core
