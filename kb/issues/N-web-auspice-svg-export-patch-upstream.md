# Auspice SVG download cuts tree labels and misses off-screen charts

## Problem

The SVG download of Auspice 3.1.0, which the Auspice view offers as the "SVG" download, writes each panel as a nested `<svg>` of the panel's size, with `overflow="hidden"` and a `<clipPath>` of the panel's box:

- Tree labels that extend past the tree panel, such as long leaf names or axis labels at its edge, are cut off in the downloaded image, although the view shows them
- The entropy chart computes its bars only while it is on screen, so a chart below the fold is written without bars. The Auspice view shows no entropy chart; the fix is in the patch because the shared part stays identical in the apps that embed Auspice

The Bun patch of Auspice, `patches/auspice@3.1.0.patch`, measures the labels of each tree panel and records the box of the panel and its labels, drops labels that are anchored outside the panel and never visible on screen, gives such a panel `overflow="visible"` and a clip path of that box, widens the image with a `viewBox` that holds the boxes, and computes the entropy bars before writing the image when the chart has none.

## Code

- Panel serialization: [download/helperFunctions.js#L427](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/download/helperFunctions.js#L427)
- Panel clipping: [download/helperFunctions.js#L568-L580](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/download/helperFunctions.js#L568-L580)
- SVG download entry: [download/helperFunctions.js#L789](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/download/helperFunctions.js#L789)

## Fix

Contribute the changes to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove them from the patch after upgrading to an Auspice release that contains them. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
