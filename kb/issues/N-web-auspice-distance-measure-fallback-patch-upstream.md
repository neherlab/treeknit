# Auspice keeps a branch length measure that the tree does not offer

## Problem

The default branch length of Auspice 3.2.0 is time (`num_date`). When a tree does not offer it, as no TreeKnit dataset does, `modifyControlsStateViaTree()` keeps it, and only `checkAndCorrectErrorsInState()` switches to the first available measure, with the console error "Requested distance measure 'num_date' is not valid. Switching to 'div'" on every load. `createTreeTooState()`, which loads a second tree into a shown tree, never runs that check: a dated main tree shown by time keeps time when an undated second tree arrives, and the undated nodes of the second tree have no position.

The Bun patch of Auspice, `patches/auspice@3.2.0.patch`, switches the measure to the first available one in `modifyControlsStateViaTree()`, which both paths run, and `createTreeTooState()` computes the branch lengths of the main tree again, because the measure may have changed. TreeKnit datasets load by divergence without the error; the view builds every store with `createStateFromQueryOrJSONs()` and never calls `createTreeTooState()`.

## Code

- Available measures: [actions/recomputeReduxState.js#L482-L490](https://github.com/nextstrain/auspice/blob/db211cc6b44296d92529844502a95a3e68710449/src/actions/recomputeReduxState.js#L482-L490)
- Second tree: [actions/recomputeReduxState.js#L1132-L1158](https://github.com/nextstrain/auspice/blob/db211cc6b44296d92529844502a95a3e68710449/src/actions/recomputeReduxState.js#L1132-L1158)

## Fix

Contribute the change to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove it from the patch after upgrading to an Auspice release that contains it. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
