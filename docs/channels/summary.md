# TreeKnit

TreeKnit infers reassortment from segment trees:

- maximally compatible clades (MCCs) for every pair of trees
- trees with polytomies resolved using the other trees
- for two trees, an ancestral reassortment graph (ARG)

The [README](https://github.com/neherlab/treeknit#readme) explains its use, and the [web app](https://neherlab.github.io/treeknit/) runs the same analyses in the browser.

The x86_64 builds need a Haswell (2013) or newer CPU with AVX2, and the Linux ARM64 builds an ARMv8.2-A CPU.
