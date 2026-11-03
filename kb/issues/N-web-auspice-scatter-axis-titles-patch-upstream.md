# Auspice titles scatterplot axes of branch lengths by option label

## Problem

Auspice 3.2.0 titles an axis of a branch length measure with `distanceMeasureText()` ("Divergence" or "Mutations" by a guess from the values, "Date", or the mutation count) in the rectangular and clock layouts. The scatterplot tests the labels of its axes (`scatterVariables.xLabel`, `yLabel`) for the measure names, but the labels are option titles such as "Divergence", so a scatterplot axis of a branch length keeps the option title. The y-axis branch also guesses from the x scale. In the Auspice view, a scatterplot axis of divergence shows "Divergence" where the other layouts show the axis title of the pair, such as "Depth".

The Bun patch of Auspice, `patches/auspice@3.2.0.patch`, tests the axis variables (`scatterVariables.x`, `y`) instead, and titles each axis from its own scale, so an axis of a branch length measure gets the title of that measure in the other layouts.

## Code

- Scatterplot axis titles: [tree/phyloTree/grid.js#L416-L424](https://github.com/nextstrain/auspice/blob/db211cc6b44296d92529844502a95a3e68710449/src/components/tree/phyloTree/grid.js#L416-L424)

## Fix

Contribute the change to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove it from the patch after upgrading to an Auspice release that contains it. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
