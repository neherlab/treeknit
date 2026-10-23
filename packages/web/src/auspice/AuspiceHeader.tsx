import type { AuspiceFiles } from "@neherlab/treeknit-wasm";
import { useMutation } from "@tanstack/react-query";
import { errorNotification } from "auspice/src/actions/notifications";
import { updateVisibleTipsAndBranchThicknesses } from "auspice/src/actions/tree";
import { publications } from "auspice/src/components/download/downloadModal";
import { SVG } from "auspice/src/components/download/helperFunctions";
import { datasetSummary } from "auspice/src/components/info/datasetSummary";
import FiltersSummary from "auspice/src/components/info/filtersSummary";
import { getParentBeyondPolytomy } from "auspice/src/components/tree/phyloTree/helpers";
import { type Ref, useCallback, useMemo } from "react";
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
import { TreeTabButton } from "./TreeTabButton";
import { type ZoomAction, zoomButtonStates, zoomRoots, type ZoomTree } from "./zoom";

const JSON_TOOLTIP =
  "Auspice JSON v2 datasets of the shown trees. auspice.us opens two of them together as a tanglegram";

const SVG_TOOLTIP = "Figure of the shown trees and their tangle lines (SVG)";

const SVG_FAILED = "The SVG figure could not be made.";

const DOWNLOAD_TOOLTIP = "The figure of the shown trees (SVG), or an Auspice JSON v2 dataset of a shown tree";

const SVG_ITEM = "svg";

export function useDatasetSummary(): string {
  const { t } = useTranslation();
  const tree = useSelector((state: AuspiceState) => state.tree);
  const mainTreeNumTips = useSelector((state: AuspiceState) => state.metadata.mainTreeNumTips);
  const branchLengthsToDisplay = useSelector((state: AuspiceState) => state.controls.branchLengthsToDisplay);

  return datasetSummary({ nodes: tree.nodes, visibility: tree.visibility, mainTreeNumTips, branchLengthsToDisplay, t });
}

export function HeaderSummary({ summary }: { summary: string }) {
  return (
    <>
      <span className="min-w-0 shrink truncate text-[14px] leading-5 font-medium text-[#888]">{summary}</span>
      <div className="auspice-filters-summary auspice-badges">
        <FiltersSummary />
      </div>
      <div className="flex-1" />
    </>
  );
}

export function HeaderDownloads({ files, treeLabels, summary, compact, groupRef }: HeaderDownloadsProps) {
  const svg = useSvgDownload(files?.svgPrefix, summary);

  return (
    <div ref={groupRef} className="flex shrink-0 gap-1 self-start">
      {compact ? (
        <DownloadMenu svg={svg} files={files?.json} treeLabels={treeLabels} />
      ) : (
        <>
          <TreeTabButton
            label="SVG"
            icon={DownloadIcon}
            tooltip={SVG_TOOLTIP}
            isDisabled={svg.isDisabled}
            onPress={svg.download}
          />
          <JsonButton files={files?.json} treeLabels={treeLabels} />
        </>
      )}
    </div>
  );
}

export interface HeaderDownloadsProps {
  files: AuspiceFiles | undefined;
  treeLabels: readonly string[];
  summary: string;
  compact: boolean;
  groupRef: Ref<HTMLDivElement>;
}

export function HeaderZoom({ groupRef }: { groupRef: Ref<HTMLDivElement> }) {
  return (
    <div ref={groupRef} className="ml-auto flex shrink-0 gap-1 self-start">
      <ZoomButtons />
    </div>
  );
}

interface SvgDownload {
  download: () => void;
  isDisabled: boolean;
}

function useSvgDownload(prefix: string | undefined, summary: string): SvgDownload {
  const { t } = useTranslation();
  const dispatch = useDispatch<AuspiceStore["dispatch"]>();
  const metadata = useSelector((state: AuspiceState) => state.metadata);
  const tree = useSelector((state: AuspiceState) => state.tree);
  const { panelsToDisplay, panelLayout } = useSelector((state: AuspiceState) => state.controls);
  const title = metadata.title ?? "";

  const { mutate, isPending } = useMutation({
    mutationFn: async (filePrefix: string) => {
      const caption = figureCaption({ title, summary, publications: [publications.nextstrain] });

      await SVG(
        dispatch,
        t,
        metadata,
        tree.nodes,
        tree.visibility,
        filePrefix,
        panelsToDisplay,
        panelLayout,
        [],
        caption,
      );
    },
    onError: (error) => {
      dispatch(errorNotification({ message: SVG_FAILED, details: error.message }));
    },
  });

  const download = useCallback(() => {
    if (prefix !== undefined) {
      mutate(prefix);
    }
  }, [prefix, mutate]);

  return useMemo(() => ({ download, isDisabled: prefix === undefined || isPending }), [download, prefix, isPending]);
}

function JsonButton({ files, treeLabels }: { files: AuspiceFiles["json"] | undefined; treeLabels: readonly string[] }) {
  const items = useJsonItems(files, treeLabels);
  const download = useItemDownload(items);

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

function DownloadMenu({ svg, files, treeLabels }: DownloadMenuProps) {
  const json = useJsonItems(files, treeLabels);
  const downloadJson = useItemDownload(json);
  const { download: downloadSvg, isDisabled: svgDisabled } = svg;

  const items = useMemo(
    () => [
      { id: SVG_ITEM, label: "SVG figure", isDisabled: svgDisabled },
      ...json.map(({ id, label }) => ({ id, label: `JSON: ${label}`, isDisabled: false })),
    ],
    [json, svgDisabled],
  );

  const disabledKeys = useMemo(() => items.filter((item) => item.isDisabled).map(({ id }) => id), [items]);

  const download = useCallback(
    (key: Key) => {
      if (key === SVG_ITEM) {
        downloadSvg();
      } else {
        downloadJson(key);
      }
    },
    [downloadSvg, downloadJson],
  );

  const trigger = useMemo(() => <TreeTabButton label="Download" icon={DownloadIcon} tooltip={DOWNLOAD_TOOLTIP} />, []);

  return (
    <Menu
      aria-label="Download the figure or an Auspice dataset"
      trigger={trigger}
      items={items}
      disabledKeys={disabledKeys}
      onAction={download}
      placement="bottom end"
    >
      {(item) => <MenuItem id={item.id}>{item.label}</MenuItem>}
    </Menu>
  );
}

interface DownloadMenuProps {
  svg: SvgDownload;
  files: AuspiceFiles["json"] | undefined;
  treeLabels: readonly string[];
}

interface JsonItem {
  id: string;
  label: string;
  file: AuspiceFiles["json"][number];
}

function useJsonItems(files: AuspiceFiles["json"] | undefined, treeLabels: readonly string[]): JsonItem[] {
  return useMemo(
    () => (files ?? []).map((file, index) => ({ id: file.path, label: treeLabels[index] ?? file.path, file })),
    [files, treeLabels],
  );
}

function useItemDownload(items: readonly JsonItem[]): (key: Key) => void {
  return useCallback(
    (key: Key) => {
      const item = items.find(({ id }) => id === key);

      if (item !== undefined) {
        downloadFile({ name: item.file.path, mediaType: item.file.mediaType, content: item.file.text });
      }
    },
    [items],
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
