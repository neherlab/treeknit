# Auspice guesses the name of the divergence axis

## Problem

Auspice 3.1.0 names the divergence axis and the divergence line of the hover panel "Divergence" or "Mutations" by a guess from the values (`guessAreMutationsPerSite()`: "Divergence" when the largest value is at most 5). TreeKnit's branch scales are divergence and depth (the Cladogram scale), which the guess cannot tell apart, so a deep cladogram is labeled "Mutations".

treeknit's own part of the Bun patch of Auspice, `patches/auspice@3.1.0.patch`, adds the prop `axisTitle` to the tree component, which replaces the guess on the axis and in the hover panel; the app passes the axis title that Rust writes into the pair (`axis_title`). auspice.us, which opens the JSON downloads of the view, still guesses, because only a field of the dataset reaches it.

## Code

- Axis titles: [tree/phyloTree/grid.js#L396-L404](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/tree/phyloTree/grid.js#L396-L404)
- Hover panel: [tree/infoPanels/hover.js#L48](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/tree/infoPanels/hover.js#L48)
- Guess: [tree/phyloTree/helpers.ts#L361](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/tree/phyloTree/helpers.ts#L361)

## Fix

Contribute an axis title field of the dataset to [nextstrain/auspice](https://github.com/nextstrain/auspice), which fixes the downloaded datasets too, and write it from Rust. Remove the prop from the patch after upgrading to an Auspice release that reads the field. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
