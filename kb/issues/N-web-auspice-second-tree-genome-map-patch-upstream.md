# Auspice never colors a second tree by genotype

## Problem

When a second tree is shown, Auspice 3.1.0 computes the genotypes of the second tree only when both trees have the same genome map (`metadata.identicalGenomeMapAcrossBothTrees`). `instantiateSecondTree()` calls `updateMetadataStateViaSecondTree()` with a copy of the metadata (`{...metadata}`) and discards the copy, so the flag is never set, and a coloring by genotype leaves the second tree in the color of missing values even when the genome maps are identical.

TreeKnit datasets have no sequences, so the Auspice view offers no coloring by genotype. The Bun patch of Auspice, `patches/auspice@3.1.0.patch`, carries the fix because its shared part stays identical in the apps that embed Auspice: `instantiateSecondTree()` keeps the copy that the comparison marks and returns it as the metadata.

## Code

- Discarded comparison: [actions/recomputeReduxState.js#L1154](https://github.com/nextstrain/auspice/blob/v3.1.0/src/actions/recomputeReduxState.js#L1154)

## Fix

Contribute the change to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove it from the patch after upgrading to an Auspice release that contains it. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
