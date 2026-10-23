# Auspice tree keeps drawing after it unmounts

## Problem

Auspice 3.1.0's tree component removes its drawing when it unmounts, but the tree objects keep their pending drawing steps: the staged transitions of an update (`modifySVGInStages`) and the timers that draw tip labels (`updateTipLabels`). The Auspice view mounts again for each new set of datasets and each change of the shown trees, and React `StrictMode` mounts it twice on the same DOM. Without the fix, a pending step of the old tree draws into the SVG of the new one.

The Bun patch of Auspice, `patches/auspice@3.1.0.patch` (registered in `patchedDependencies` of the root `package.json`), fixes this: `componentWillUnmount()` calls a `dispose()` step on each tree object before the drawing is removed, and the disposed tree stops its staged transitions and tip label timers.

## Code

- `TreeComponent.componentWillUnmount()`: removes the drawing ([tree/tree.tsx#L81-L86](https://github.com/nextstrain/auspice/blob/v3.1.0/src/components/tree/tree.tsx#L81-L86))

## Fix

Contribute the change to [nextstrain/auspice](https://github.com/nextstrain/auspice). After upgrading to an Auspice release that contains it, remove the change from the patch. The patch file also holds the changes in the other `N-web-auspice-*-patch-upstream.md` issues. When the patch has no changes left, delete the patch file and its `patchedDependencies` entry. Bun refuses to install when the patch no longer applies to a new Auspice version, so an upgrade without this step fails visibly.

## Editing the patch

The patch has two parts. The first part is the shared set of Auspice fixes, identical in the four apps that embed Auspice (Nextclade, treetime.rs, treeknit-rs, auspice-vscode), so that each Auspice upgrade rebases the shared fixes once. Its reference text is Nextclade's patch, `packages/nextclade-web/patches/auspice@3.1.0.patch` in [nextstrain/nextclade](https://github.com/nextstrain/nextclade). The second part holds treeknit's own changes: the tree props `axisTitle`, `showTreeButtons`, and `showNodeClickedPanel`.

The patch is a Git diff against the published Auspice package. `bun patch --commit` writes Bun's `.bun-tag-*` marker files into the patch, so write it with Git:

1. Run `npm pack auspice@<version>` in a new directory under `tmp/` and extract the archive into a directory. Run `git init`, `git add -A` and `git commit` there: this commit is the base
2. Apply the shared part with `git apply <shared patch>` and commit it. Then apply or edit treeknit's own changes and commit them
3. Write the patch with `git diff --full-index <base> HEAD > patches/auspice@<version>.patch`, and check that `git diff --full-index <base> <shared commit>` equals the shared reference text
4. Delete the installed Auspice copies in `node_modules/.bun/` and the patched copies `auspice@<version>@@@*` in the Bun cache `.cache/bun/`, because Bun can install a patched copy from an earlier version of the patch file. Run `./dev/docker/run bun install`, check the installed files for each change, and restart the dev server
