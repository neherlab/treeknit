# Auspice tree clip areas share one element ID

## Problem

In Auspice 3.1.0, every tree object creates a `<clipPath>` with the fixed ID `treeClip`, and all branch, tip, label and grid groups clip to `url(#treeClip)`. Element IDs are unique per document, and the browser resolves a duplicated ID to the first element. The Auspice view shows both trees of a pair as a tanglegram, so the right tree was clipped to the rectangle of the left one. The SVG download also writes both clip areas under one ID.

The Bun patch of Auspice, `patches/auspice@3.1.0.patch`, derives the clip ID from the tree ID (`treeClip-LEFT`, `treeClip-RIGHT`). It also gives each legend its own clip ID, which matters on pages that show several legends, such as Nextclade's tree and map; the Auspice view shows one legend.

## Code

- Clip area creation: [tree/phyloTree/renderers.ts#L524](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/tree/phyloTree/renderers.ts#L524)
- Legend clip area: [legend/legend.tsx#L220](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/legend/legend.tsx#L220), [legend/legend.tsx#L234](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/legend/legend.tsx#L234)
- Clip references: [tree/phyloTree/renderers.ts#L224](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/tree/phyloTree/renderers.ts#L224), [tree/phyloTree/labels.js#L138](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/tree/phyloTree/labels.js#L138), [tree/phyloTree/grid.js#L36-L39](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/tree/phyloTree/grid.js#L36-L39)

## Fix

Contribute the change to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove it from the patch after upgrading to an Auspice release that contains it. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
