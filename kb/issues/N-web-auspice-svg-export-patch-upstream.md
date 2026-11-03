# Auspice SVG download cuts tree labels, misses off-screen charts, and misplaces caption text

## Problem

The SVG download of Auspice 3.2.0, which the Auspice view offers as the "SVG" download, writes each panel as a nested `<svg>` of the panel's size, with `overflow="hidden"` and a `<clipPath>` of the panel's box:

- Tree labels that extend past the tree panel, such as long leaf names or axis labels at its edge, are cut off in the downloaded image, although the view shows them
- The entropy chart computes its bars only while it is on screen, so a chart below the fold is written without bars. The Auspice view shows no entropy chart; the fix belongs to the general part of the patch, which fixes Auspice as a whole
- Text after an inline element starts one space too far left: the download writes each HTML text node of the caption as an SVG `<text>` at the position of its first character, and SVG drops a leading space. In the citation of the figure caption, "18:e1010394." after the italic journal name "PLOS Computational Biology" touched the name

The Bun patch of Auspice, `patches/auspice@3.2.0.patch`, measures the labels of each tree panel and records the box of the panel and its labels, drops labels that are anchored outside the panel and never visible on screen, gives such a panel `overflow="visible"` and a clip path of that box, widens the image with a `viewBox` that holds the boxes, computes the entropy bars before writing the image when the chart has none, and starts each exported text line at its first visible character.

The tests `packages/web/src/auspice/__tests__/test_svg_text.test.ts` cover the text positions of a caption line, on one line and on several lines.

## Code

- Panel serialization: [download/helperFunctions.js#L429](https://github.com/nextstrain/auspice/blob/db211cc6b44296d92529844502a95a3e68710449/src/components/download/helperFunctions.js#L429)
- Panel clipping: [download/helperFunctions.js#L570-L582](https://github.com/nextstrain/auspice/blob/db211cc6b44296d92529844502a95a3e68710449/src/components/download/helperFunctions.js#L570-L582)
- Text lines: [download/domToSvg.js#L100-L145](https://github.com/nextstrain/auspice/blob/db211cc6b44296d92529844502a95a3e68710449/src/components/download/domToSvg.js#L100-L145)
- SVG download entry: [download/helperFunctions.js#L791](https://github.com/nextstrain/auspice/blob/db211cc6b44296d92529844502a95a3e68710449/src/components/download/helperFunctions.js#L791)

## Fix

Contribute the changes to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove them from the patch after upgrading to an Auspice release that contains them. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
