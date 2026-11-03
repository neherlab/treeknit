# Auspice offers a time axis for trees where only some nodes have dates

## Problem

Auspice 3.2.0 offers the time metric, the clock layout and the date controls when the root node of the tree has a date (`num_date`), in `findAvailableDistanceMeasures()`. On a tree where other nodes have no date, such as a dated reference tree with sequences placed onto it, the undated nodes have no position on the time axis and are missing from the drawing.

TreeKnit datasets have no dates, so the Auspice view always shows divergence. The Bun patch of Auspice, `patches/auspice@3.2.0.patch`, carries the fix because its shared part stays identical in the apps that embed Auspice: `findAvailableDistanceMeasures()` offers the time axis only when every node of the tree has a date. With a second tree, Auspice offers the measures that both trees offer, so the time axis also needs a date on every node of the second tree.

## Code

- Time axis availability: [util/computeDistanceMetric.ts#L101-L107](https://github.com/nextstrain/auspice/blob/db211cc6b44296d92529844502a95a3e68710449/src/util/computeDistanceMetric.ts#L101-L107)

## Fix

Contribute the change to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove it from the patch after upgrading to an Auspice release that contains it. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
