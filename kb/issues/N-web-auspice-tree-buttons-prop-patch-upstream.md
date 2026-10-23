# Auspice tree has no way to hide its zoom buttons

## Problem

Auspice 3.1.0 draws a group of zoom buttons for each tree in the title row of the tree card. The header of the Auspice view draws one zoom group for both trees, because it zooms the matching clades of both trees together, and puts the leaf summary, the filter badges, and the downloads in the same title row.

treeknit's own part of the Bun patch of Auspice, `patches/auspice@3.1.0.patch`, adds the prop `showTreeButtons` to the tree component; the app passes `false`, and the CSS utility `auspice-card` (`packages/web/src/index.css`) reshapes the title row for the app's header.

## Code

- Zoom buttons of the trees: [tree/tree.tsx#L215-L219](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/tree/tree.tsx#L215-L219)

## Fix

Contribute a title row extension to [nextstrain/auspice](https://github.com/nextstrain/auspice): a left slot for content such as the summary and the filter badges, and a right slot for the app's buttons before the zoom buttons. It would replace the prop and the CSS overrides of the title row. Remove the prop from the patch after upgrading to an Auspice release that contains the extension. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
