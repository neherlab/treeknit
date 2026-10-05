import type { AuspiceFiles } from "@neherlab/treeknit-wasm";
import { updateVisibleTipsAndBranchThicknesses } from "auspice/src/actions/tree";
import { publications } from "auspice/src/components/download/downloadModal";
import { SVG } from "auspice/src/components/download/helperFunctions";
import { datasetSummary } from "auspice/src/components/info/datasetSummary";
import FiltersSummary from "auspice/src/components/info/filtersSummary";
import { getParentBeyondPolytomy } from "auspice/src/components/tree/phyloTree/helpers";
import { useCallback, useMemo } from "react";
import type { Key } from "react-aria-components";
import { useTranslation } from "react-i18next";
import { useDispatch, useSelector } from "react-redux";
import DownloadIcon from "~icons/lucide/download";
import ZoomOutIcon from "~icons/lucide/zoom-out";

import { downloadFile } from "../download";
import { Menu, MenuItem } from "../ui/Menu";
import { figureCaption } from "./figure";
import type { AuspiceNode, AuspiceState, ObservedMutations } from "./state";
import type { AuspiceStore } from "./store";
import { TreeTabButton, TreeTabButtonGroup } from "./TreeTabButton";
import { type ZoomAction, zoomButtonStates, zoomRoots, type ZoomTree } from "./zoom";

const JSON_TOOLTIP =
  "Auspice JSON v2 datasets of the shown trees. auspice.us opens two of them together as a tanglegram";

const SVG_TOOLTIP = "Figure of the shown trees and their tangle lines (SVG)";

export function AuspiceHeader({ files, treeLabels }: AuspiceHeaderProps) {
  const { t } = useTranslation();
  const tree = useSelector((state: AuspiceState) => state.tree);
  const metadata = useSelector((state: AuspiceState) => state.metadata);
  const branchLengthsToDisplay = useSelector((state: AuspiceState) => state.controls.branchLengthsToDisplay);

  const summary = datasetSummary({
    nodes: tree.nodes,
    visibility: tree.visibility,
    mainTreeNumTips: metadata.mainTreeNumTips,
    branchLengthsToDisplay,
    t,
  });

  return (
    <div className="absolute inset-x-0 top-0 z-1 flex h-[26px] items-center gap-4">
      <span className="shrink-0 text-[14px] leading-5 font-medium whitespace-nowrap text-[#888]">{summary}</span>
      <div className="auspice-filters-summary auspice-badges">
        <FiltersSummary />
      </div>
      <div className="flex-1" />
      <TreeTabButtonGroup>
        <SvgButton prefix={files?.svgPrefix} summary={summary} />
        <JsonButton files={files?.json} treeLabels={treeLabels} />
        <ZoomButtons />
      </TreeTabButtonGroup>
    </div>
  );
}

export interface AuspiceHeaderProps {
  files: AuspiceFiles | undefined;
  treeLabels: readonly string[];
}

function SvgButton({ prefix, summary }: { prefix: string | undefined; summary: string }) {
  const { t } = useTranslation();
  const dispatch = useDispatch<AuspiceStore["dispatch"]>();
  const metadata = useSelector((state: AuspiceState) => state.metadata);
  const tree = useSelector((state: AuspiceState) => state.tree);
  const { panelsToDisplay, panelLayout } = useSelector((state: AuspiceState) => state.controls);
  const title = metadata.title ?? "";

  const download = useCallback(() => {
    if (prefix !== undefined) {
      const caption = figureCaption({ title, summary, publications: [publications.nextstrain] });

      SVG(dispatch, t, metadata, tree.nodes, tree.visibility, prefix, panelsToDisplay, panelLayout, [], caption);
    }
  }, [prefix, title, summary, dispatch, t, metadata, tree, panelsToDisplay, panelLayout]);

  return (
    <TreeTabButton
      label="SVG"
      icon={DownloadIcon}
      tooltip={SVG_TOOLTIP}
      isDisabled={prefix === undefined}
      onPress={download}
    />
  );
}

function JsonButton({ files, treeLabels }: { files: AuspiceFiles["json"] | undefined; treeLabels: readonly string[] }) {
  const items = useMemo(
    () => (files ?? []).map((file, index) => ({ id: file.path, label: treeLabels[index] ?? file.path, file })),
    [files, treeLabels],
  );

  const download = useCallback(
    (key: Key) => {
      const item = items.find(({ id }) => id === key);

      if (item !== undefined) {
        downloadFile({ name: item.file.path, mediaType: item.file.mediaType, content: item.file.text });
      }
    },
    [items],
  );

  const trigger = useMemo(
    () => <TreeTabButton label="JSON" icon={DownloadIcon} tooltip={JSON_TOOLTIP} isDisabled={files === undefined} />,
    [files],
  );

  return (
    <Menu
      aria-label="Download an Auspice dataset"
      trigger={trigger}
      items={items}
      onAction={download}
      placement="bottom end"
    >
      {(item) => <MenuItem id={item.id}>{item.label}</MenuItem>}
    </Menu>
  );
}

function ZoomButtons() {
  const dispatch = useDispatch<AuspiceStore["dispatch"]>();
  const tree = useSelector((state: AuspiceState) => state.tree);
  const treeToo = useSelector((state: AuspiceState) => state.treeToo);
  const controls = useSelector((state: AuspiceState) => state.controls);
  const trees = useMemo(() => shownZoomTrees({ tree, treeToo, controls }), [tree, treeToo, controls]);
  const enabled = zoomButtonStates(trees.filter((zoomTree) => zoomTree !== undefined));
  const focusMode = controls.focus === "selected";

  const zoom = useCallback(
    (action: ZoomAction) => {
      const [main, second] = trees;

      if (main !== undefined) {
        dispatch(updateVisibleTipsAndBranchThicknesses({ root: zoomRoots(action, [main, second]) }));
      }
    },
    [dispatch, trees],
  );

  const zoomOut = useCallback(() => {
    zoom("out");
  }, [zoom]);

  const zoomToSelected = useCallback(() => {
    zoom("selected");
  }, [zoom]);

  const zoomToRoot = useCallback(() => {
    zoom("root");
  }, [zoom]);

  return (
    <>
      <TreeTabButton label="Zoom out" icon={ZoomOutIcon} iconOnly isDisabled={!enabled.out} onPress={zoomOut} />
      {focusMode ? null : (
        <TreeTabButton
          label="Zoom to selected"
          tooltip="Zoom each tree to the smallest clade that holds the selected leaves"
          isDisabled={!enabled.selected}
          onPress={zoomToSelected}
        />
      )}
      <TreeTabButton
        label={focusMode ? "Show all selected" : "Zoom to root"}
        isDisabled={!enabled.root}
        onPress={zoomToRoot}
      />
    </>
  );
}

function shownZoomTrees({
  tree,
  treeToo,
  controls,
}: Pick<AuspiceState, "tree" | "treeToo" | "controls">): [ZoomTree | undefined, ZoomTree | undefined] {
  const main = zoomTree(tree.nodes, tree.idxOfInViewRootNode, tree.idxOfFilteredRoot, controls.distanceMeasure, tree);

  const second =
    controls.showTreeToo === false || controls.showTreeToo === undefined
      ? undefined
      : zoomTree(
          treeToo.nodes,
          treeToo.idxOfInViewRootNode,
          treeToo.idxOfFilteredRoot,
          controls.distanceMeasure,
          treeToo,
        );

  return [main, second];
}

function zoomTree(
  nodes: readonly AuspiceNode[] | null | undefined,
  inViewRoot: number | undefined,
  filteredRoot: number | undefined,
  distanceMeasure: string,
  { observedMutations }: { observedMutations?: ObservedMutations | undefined },
): ZoomTree | undefined {
  if (nodes === null || nodes === undefined || inViewRoot === undefined) {
    return undefined;
  }

  return {
    inViewRoot,
    filteredRoot,
    parentOfInViewRoot: () => {
      const root = nodes[inViewRoot];

      return root === undefined ? 0 : getParentBeyondPolytomy(root, distanceMeasure, observedMutations).arrayIdx;
    },
  };
}
