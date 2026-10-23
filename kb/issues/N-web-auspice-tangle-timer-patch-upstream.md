# Auspice tangle lines keep a stale redraw timer

## Problem

When the tanglegram of Auspice 3.1.0 changes size, `componentDidUpdate()` removes the lines between the two trees and draws them again after 1 s. It removes the lines only when no redraw is pending, and it never resets `this.timeout` after the timer fires, so after the first resize the old lines stay on screen for 1 s after each later resize. No `componentWillUnmount()` clears the timer, so a redraw that is pending when the Auspice view mounts again draws into an SVG that is gone.

The Bun patch of Auspice, `patches/auspice@3.1.0.patch`, resets `this.timeout` when the timer fires, and clears a pending timer on unmount.

## Code

- Redraw timer: [tree/tangle/index.js#L72-L81](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/tree/tangle/index.js#L72-L81)

## Fix

Contribute the change to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove it from the patch after upgrading to an Auspice release that contains it. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
