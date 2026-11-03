# Auspice entropy chart crowds its genome axes on narrow screens

## Problem

Auspice 3.2.0 asks d3 for 16 ticks on the main genome axis of the entropy chart and 20 on the navigation axis below it, whatever the chart width. In a narrow chart the labels (such as "28,000") run into each other and cannot be read.

The Auspice view shows no entropy chart, because TreeKnit datasets have no sequences. The Bun patch of Auspice, `patches/auspice@3.2.0.patch`, carries the fix in its general part, which fixes Auspice as a whole: each axis asks for one tick per 60 px of its width, at most the numbers above.

## Code

- Axis ticks: [entropy/entropyD3.js#L627-L630](https://github.com/nextstrain/auspice/blob/db211cc6b44296d92529844502a95a3e68710449/src/components/entropy/entropyD3.js#L627-L630)

## Fix

Contribute the change to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove it from the patch after upgrading to an Auspice release that contains it. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
