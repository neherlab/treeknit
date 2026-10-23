# Auspice entropy chart crowds its genome axes on narrow screens

## Problem

Auspice 3.1.0 asks d3 for 16 ticks on the main genome axis of the entropy chart and 20 on the navigation axis below it, whatever the chart width. In a narrow chart the labels (such as "28,000") run into each other and cannot be read.

The Auspice view shows no entropy chart, because TreeKnit datasets have no sequences. The Bun patch of Auspice, `patches/auspice@3.1.0.patch`, carries the fix because its shared part stays identical in the apps that embed Auspice: each axis asks for one tick per 60 px of its width, at most the numbers above.

## Code

- Axis ticks: [entropy/entropyD3.js#L627-L630](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/entropy/entropyD3.js#L627-L630)

## Fix

Contribute the change to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove it from the patch after upgrading to an Auspice release that contains it. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
