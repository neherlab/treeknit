# Auspice tree has no way to hide its panel of a clicked node

## Problem

Auspice 3.1.0 opens a panel with the details of a node when the user clicks a tip or a branch. In the app, a click selects the leaf or node in the workspace, and the inspector shows the selection, so Auspice's panel would show a second copy of it over the tree.

treeknit's own part of the Bun patch of Auspice, `patches/auspice@3.1.0.patch`, adds the prop `showNodeClickedPanel` to the tree component; the app passes `false`.

## Code

- Panel of a clicked node: [tree/tree.tsx#L180](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/tree/tree.tsx#L180)

## Fix

Contribute the prop to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove it from the patch after upgrading to an Auspice release that contains it. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
