# Changelog

Each release has a section `## <version>`, which becomes the notes of its release on the releases page. Changes that are not released yet go under `## Unreleased`; `dev/release <version>` renames that heading when it prepares the release.

## Unreleased

First release of the Rust port of [TreeKnit.jl](https://github.com/PierreBarrat/TreeKnit.jl), which infers reassortment from segment trees.

- **MCCs**: maximally compatible clades for every pair of trees, compatible with the `MCCs.json` and `MCCs.dat` output of TreeKnit.jl
- **Resolution**: polytomies resolved using the other trees (`--resolve matched|strict|liberal|none`)
- **ARG**: for two trees, an ancestral reassortment graph in extended Newick
- **Different leaf sets**: inference uses the leaves a pair shares; leaves missing from one tree are imputed into it
- **Input checks**: the command line and the web app check the trees and settings with the same rules and report every error before a run: a negative or non-finite γ, sequence lengths that are not positive, zero rounds or MCMC steps, more than 2^32 - 2 rounds or 2^32 - 1 MCMC steps per leaf (the counts of the 32-bit WebAssembly build of the web app, so both surfaces accept the same settings), a seed above 2^53 - 1, tree labels that cannot be file names, pairs whose output files would get the same name, and pairs that share fewer than two leaves
- **Reproducible runs**: `--seed` fixes the result, and independent tree pairs run in parallel with identical results
- **Seed range (breaking)**: `--seed` accepts at most 2^53 - 1 (9007199254740991). Earlier builds of the port accepted any 64-bit value, so a script that passes a larger seed, such as a nanosecond timestamp, now stops with an error. The web app keeps the seed in its session file, and JavaScript numbers hold integers exactly only up to 2^53 - 1, so a larger seed would change on its way through the app
- **Tree label characters (breaking)**: a tree label names the output files of its tree, so on every platform it must not contain a character that Windows reserves (`<>:"|?*`), and labels that differ only in case count as one label. The command line labels a tree by its file name, so a Linux file such as `2024-01-01T10:00.nwk`, which earlier builds of the port accepted, now stops the command until it is renamed. Files whose names differ only in case, such as `a/HA.nwk` and `b/ha.nwk`, get the name of their directory in the label, as files with equal names do
- **Output names follow tree labels**: every output file of a tree is named after its label. Earlier builds of the port named the trees of input files with the same name, such as `a/ha.nwk` and `b/ha.nwk`, by their labels `ha_a` and `ha_b` in `MCCs.json`, but wrote both resolved trees to `ha_resolved.nwk`, so the second overwrote the first; they are now `ha_a_resolved.nwk` and `ha_b_resolved.nwk`
- **Session files**: `treeknit --request treeknit_request.json` runs the trees and settings of a session file that the web app saves; with `--impute --auspice-view --plot`, as the web app shows the command, it writes the same output files as the web app
- **Figures**: `--plot` writes an SVG tanglegram of each pair and, for two trees, an SVG figure of the ARG
- **Every option takes effect**: in TreeKnit.jl 0.5.8, `--gamma`, `--n-mcmc-it`, `--no-likelihood`, `--seq-lengths` and `--parallel` are silently ignored
- **Prebuilt binaries**: the CLI for Linux (glibc and static musl), macOS, and Windows
- **Web app**: the same analyses in the browser, on the WebAssembly build of the core
