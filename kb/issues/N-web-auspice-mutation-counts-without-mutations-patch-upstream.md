# Auspice offers mutation counts for genes without mutations

## Problem

Auspice 3.2.0 offers a branch length of mutation counts ("Mutation counts: nuc", "Mutation counts: <CDS>") for the genome and for every CDS of the genome map (`meta.genome_annotations`), whether the tree has mutations there or not. A gene or CDS without any valid mutation on the tree places every node at 0, so the tree draws as a vertical line. With a second tree, the genome map of the main tree also names the CDSs of the second tree, so a second tree with another genome map (for example flu HA, then NA) offers counts for CDSs that it has no mutations in.

TreeKnit datasets have no genome map, so the Auspice view offers no mutation counts. The Bun patch of Auspice, `patches/auspice@3.2.0.patch`, carries the fix in its general part, which fixes Auspice as a whole: `findAvailableDistanceMeasures()` offers a mutation count only for the genome and the CDSs with at least one valid mutation (`validMutation()`: no `N` or gap in nucleotides, no `X` in amino acids) on the tree.

## Code

- Measures of the genome map: [util/computeDistanceMetric.ts#L110-L118](https://github.com/nextstrain/auspice/blob/db211cc6b44296d92529844502a95a3e68710449/src/util/computeDistanceMetric.ts#L110-L118)

## Fix

Contribute the change to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove it from the patch after upgrading to an Auspice release that contains it. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
