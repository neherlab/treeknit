# Auspice shows the Branch Length control with one option

## Problem

Auspice 3.2.0 always shows the "Branch Length" select in the sidebar, also for a tree that offers one measure, such as a tree with divergence only or with dates only. Every TreeKnit dataset has divergence only, so the sidebar would show a select whose only option is "Divergence". Auspice 3.1.0 showed its branch length buttons only for a tree with both divergence and dates.

The Bun patch of Auspice, `patches/auspice@3.2.0.patch`, hides the control while the tree offers fewer than two measures.

## Code

- Branch length control: [controls/choose-metric.tsx#L64-L66](https://github.com/nextstrain/auspice/blob/db211cc6b44296d92529844502a95a3e68710449/src/components/controls/choose-metric.tsx#L64-L66)

## Fix

Contribute the change to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove it from the patch after upgrading to an Auspice release that contains it. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
