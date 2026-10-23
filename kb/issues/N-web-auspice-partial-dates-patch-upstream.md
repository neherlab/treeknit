# Auspice offers a time axis for trees where only some nodes have dates

## Problem

Auspice 3.1.0 offers the time metric, the clock layout and the date controls when the root node of the tree has a date (`num_date`). On a tree where other nodes have no date, such as a dated reference tree with sequences placed onto it, the undated nodes have no position on the time axis and are missing from the drawing.

TreeKnit datasets have no dates, so the Auspice view always shows divergence. The Bun patch of Auspice, `patches/auspice@3.1.0.patch`, carries the fix because its shared part stays identical in the apps that embed Auspice: Auspice offers the time axis only when every node of the main tree and of the second tree has a date.

## Code

- Time axis availability: [actions/recomputeReduxState.js#L476-L480](https://github.com/nextstrain/auspice/blob/v3.1.0/src/actions/recomputeReduxState.js#L476-L480)

## Fix

Contribute the change to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove it from the patch after upgrading to an Auspice release that contains it. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
