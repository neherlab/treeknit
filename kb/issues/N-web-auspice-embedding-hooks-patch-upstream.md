# Auspice lacks hooks for apps that embed its components

## Problem

The Auspice view renders Auspice components inside the app and builds the Auspice sources with the app's Vite plugin (`packages/web/build/auspice.ts`). Auspice 3.1.0 assumes its own app and build in several places, which the Bun patch of Auspice, `patches/auspice@3.1.0.patch`, changes:

- **Extensions registry**: Auspice reads its extensions from a build-time `EXTENSION_DATA` variable and loads them through webpack's `require.context("@extensions")`. The patch imports the registry from the `@extensions` module. treeknit registers no extension, so the Vite plugin resolves `@extensions` to a virtual module whose default export is an empty registry. `getExtension()` also returns registered values that are falsy, such as `false`
- **Figure caption**: the SVG download writes the text lines of a Nextstrain dataset below the figure: "Last updated", its address, the data use terms and the CC-BY license, which do not hold for the trees of a TreeKnit user. The patch adds a `caption` parameter to the asynchronous `SVG()` that replaces these lines; the "SVG" download of the Auspice view passes the title, the leaf summary, and the Nextstrain citation
- **Download modal**: the "d" key opens Auspice's download modal, which offers downloads of Nextstrain datasets. The patch skips the key when the `downloadModal` extension is `false`. The Auspice view does not mount the modal
- **Kept view**: once the tree is drawn, Auspice keeps the zoom on the drawn nodes only. The patch lets a new drawing of the same nodes take the zoom from the previous drawing. The Auspice view builds a new store with new nodes for each mount, so it does not use this hook

## Code

- Extensions registry: [util/extensions.ts#L8-L35](https://github.com/nextstrain/auspice/blob/v3.1.0/src/util/extensions.ts#L8-L35)
- SVG download entry: [download/helperFunctions.js#L789](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/download/helperFunctions.js#L789)
- Download modal key: [modal/Modal.jsx#L29](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/modal/Modal.jsx#L29)
- Zoom of a new drawing: [tree/phyloTree/phyloTree.ts#L39](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/tree/phyloTree/phyloTree.ts#L39)

## Fix

Contribute the hooks to [nextstrain/auspice](https://github.com/nextstrain/auspice), then remove them from the patch after upgrading to an Auspice release that contains them. When the registry hook goes, remove the `@extensions` module of the Vite plugin. [N-web-auspice-tree-unmount-patch-upstream.md](N-web-auspice-tree-unmount-patch-upstream.md) describes how to edit the patch.
