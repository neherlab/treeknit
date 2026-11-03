# Auspice keeps the measures of a removed second tree

## Problem

With a second tree, Auspice 3.2.0 offers the branch length measures that both trees offer. Removing the second tree (`REMOVE_TREE_TOO`) keeps that list, so the main tree offers neither time nor the mutation counts that only it has until the page loads again.

The Auspice view never removes a second tree: it builds a new store for each change of the shown trees. The Bun patch of Auspice, `patches/auspice@3.2.0.patch`, carries the fix because its shared part stays identical in the apps that embed Auspice: the thunk `removeSecondTree()` sends the measures of the main tree with `REMOVE_TREE_TOO`, the controls reducer takes them, and the second tree select of Auspice's sidebar dispatches the thunk.

## Code

- Removal: [reducers/controls.ts#L509-L515](https://github.com/nextstrain/auspice/blob/db211cc6b44296d92529844502a95a3e68710449/src/reducers/controls.ts#L509-L515)
- Second tree select: [controls/choose-second-tree.js#L63](https://github.com/nextstrain/auspice/blob/db211cc6b44296d92529844502a95a3e68710449/src/components/controls/choose-second-tree.js#L63)

## Fix

Contribute the change to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove it from the patch after upgrading to an Auspice release that contains it. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
