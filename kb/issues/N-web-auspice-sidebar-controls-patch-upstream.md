# Auspice sidebar controls need local fixes for keyboard use and border-box pages

## Problem

- **Keyboard**: Auspice 3.1.0 hides the checkbox of each toggle with `display: none`, so the keyboard skips all toggles of the Auspice controls, such as "Show all labels", "Focus on selected", the tangle switch, and the stream tree switch. Panel section headers and the buttons of filter badges react to clicks only. Selects (color by, branch labels, tip labels, the filter query, the branch label of stream trees), panel toggles and filter badge buttons have no accessible names
- **Toggle height on border-box pages**: a toggle container has a fixed height of 21 px, and some toggles add padding through their `style` prop. A page that sizes all boxes as `border-box`, as Tailwind's base styles of the web app do, shrinks the toggle by the padding, and the toggle label overlaps the next control, for example "Show all labels" over "Tip Labels"

The Bun patch of Auspice, `patches/auspice@3.1.0.patch`, keeps the checkbox of a toggle in place, invisible and focusable, with a focus ring on the slider; panel headers and filter badge buttons react to Enter and Space; the controls get names (`Hide filter <name>`, `Show filter <name>`, `Remove filter <name>` for the badge buttons); and the toggle container keeps `content-box` sizing.

## Code

- Hidden checkbox: [controls/toggle.js#L17-L19](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/controls/toggle.js#L17-L19)
- Toggle container: [controls/toggle.js#L6-L10](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/controls/toggle.js#L6-L10)
- Panel header: [controls/panelHeader.tsx#L45-L49](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/controls/panelHeader.tsx#L45-L49)
- Filter badge buttons: [info/filterBadge.js#L109-L117](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/info/filterBadge.js#L109-L117)

## Fix

Contribute the changes to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove them from the patch after upgrading to an Auspice release that contains them. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
