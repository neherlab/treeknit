# Auspice entropy chart draws twice under React StrictMode

## Problem

In Auspice 3.1.0, `componentWillUnmount()` of the entropy chart destroys the chart only `if (this.state.chart)`, while the chart is drawn first and stored later through `setState({chart})`. React StrictMode unmounts and mounts each component once more right after the first mount, before that state update applies, so the first drawing stays in the DOM and the second mount draws a second chart over it.

The Auspice view shows no entropy chart. The Bun patch of Auspice, `patches/auspice@3.1.0.patch`, carries the fix because its shared part stays identical in the apps that embed Auspice: `componentWillUnmount()` stops the resize observer and removes the drawing from the DOM (`select(this.d3entropy).selectAll("*").remove()`), which does not depend on the state.

## Code

- Chart stored in the state: [entropy/index.js#L211](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/entropy/index.js#L211)
- Unmount: [entropy/index.js#L337-L341](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/entropy/index.js#L337-L341)

## Fix

Contribute the change to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove it from the patch after upgrading to an Auspice release that contains it. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
