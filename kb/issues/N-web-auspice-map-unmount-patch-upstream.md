# Auspice map fails when it is mounted again

## Problem

The Auspice 3.1.0 map assumes that it is mounted once per page load: it registers the export function of the SVG download (`window.L.getMapSvg`) only once, so after a remount the download reads the detached first map; `componentWillUnmount()` reads the Leaflet map without a check, so an unmount before the first update throws; and the unmount leaves the Leaflet map, which holds a WebGL context, and its timers.

The Auspice view shows no map, because TreeKnit datasets have no locations. The Bun patch of Auspice, `patches/auspice@3.1.0.patch`, carries the fix because its shared part stays identical in the apps that embed Auspice: each mount registers the export function of its own instance and removes it on unmount, and the cleanup removes the Leaflet map and clears the timers after the current task, so that React StrictMode's immediate remount keeps the map.

## Code

- Export function: [map/map.js#L106-L140](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/map/map.js#L106-L140)
- Unmount: [map/map.js#L737-L740](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/map/map.js#L737-L740)

## Fix

Contribute the changes to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove them from the patch after upgrading to an Auspice release that contains them. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
