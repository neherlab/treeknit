# Auspice reads the coordinates of dropped metadata wrongly

## Problem

Auspice 3.1.0 turns a dropped CSV or TSV file with `latitude` and `longitude` (or `__latitude` and `__longitude`) columns into a map of the samples. Its parser groups samples by the joined strings of their coordinates, so `(1, 23)` and `(12, 3)` are one location; reads an empty coordinate as 0; refuses a file whose only other columns are the coordinates; and gives a map that the new metadata makes available no legend placement, so its legend fails.

The Auspice view accepts no dropped metadata and shows no map. The Bun patch of Auspice, `patches/auspice@3.1.0.patch`, carries the fix because its shared part stays identical in the apps that embed Auspice: coordinates alone make a valid file, samples are grouped by their parsed coordinates, rows with an empty coordinate and rows of names that are not in the tree are skipped, and the metadata update adds the legend placement of a map that it makes available.

## Code

- Column check: [actions/filesDropped/parseCsv.ts#L81](https://github.com/nextstrain/auspice/blob/v3.1.0/src/actions/filesDropped/parseCsv.ts#L81)
- Coordinate grouping: [actions/filesDropped/parseCsv.ts#L313-L318](https://github.com/nextstrain/auspice/blob/v3.1.0/src/actions/filesDropped/parseCsv.ts#L313-L318)
- Legend placements: [util/metadataJsonParsing.ts#L113](https://github.com/nextstrain/auspice/blob/v3.1.0/src/util/metadataJsonParsing.ts#L113)
- Geographic metadata update: [actions/updateMetadata/updateMetadata.ts#L182-L189](https://github.com/nextstrain/auspice/blob/v3.1.0/src/actions/updateMetadata/updateMetadata.ts#L182-L189)

## Fix

Contribute the changes to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove them from the patch after upgrading to an Auspice release that contains them. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
