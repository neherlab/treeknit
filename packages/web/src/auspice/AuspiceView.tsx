import type { AuspiceFiles } from "@neherlab/treeknit-wasm";
import { AnnotatedTitle } from "auspice/src/components/controls/annotatedTitle";
import ChooseBranchLabelling from "auspice/src/components/controls/choose-branch-labelling";
import ChooseLayout from "auspice/src/components/controls/choose-layout";
import ChooseMetric from "auspice/src/components/controls/choose-metric";
import ChooseTipLabel from "auspice/src/components/controls/choose-tip-label";
import ColorBy, { ColorByInfo } from "auspice/src/components/controls/color-by";
import { ControlHeader } from "auspice/src/components/controls/controlHeader";
import FilterData, { FilterInfo } from "auspice/src/components/controls/filter";
import { TreeInfo } from "auspice/src/components/controls/miscInfoText";
import { ControlsContainer } from "auspice/src/components/controls/styles";
import { ToggleFocus } from "auspice/src/components/controls/toggle-focus";
import ToggleTangle from "auspice/src/components/controls/toggle-tangle";
import Tree from "auspice/src/components/tree";
import { useState } from "react";
import { I18nextProvider } from "react-i18next";
import { Provider } from "react-redux";
import { ThemeProvider } from "styled-components";

import { useMeasuredSize } from "../canvas/useMeasuredSize";
import type { MeasuredSize } from "../canvas/viewState";
import { usePaneOpen } from "../shell/paneStore";
import { CollapsiblePane } from "../ui/CollapsiblePane";
import { AuspiceHeader } from "./AuspiceHeader";
import { AUSPICE_I18N } from "./i18n";
import { SIDEBAR_WIDTH_PX, sidebarOverlays, treeSize } from "./layout";
import { type AuspiceInput, useAuspiceStore } from "./useAuspiceStore";

const SIDEBAR = <AuspiceSidebar />;

const SIDEBAR_THEME = {
  background: "#F2F2F2",
  color: "#000",
  "font-family": "var(--font-sans)",
  sidebarBoxShadow: "rgba(0, 0, 0, 0.2)",
  selectedColor: "#5097BA",
  unselectedColor: "#333",
  alternateBackground: "#888",
};

export default function AuspiceView({ files, treeLabels, axisTitle, ...input }: AuspiceViewProps) {
  const store = useAuspiceStore(input);

  return (
    <I18nextProvider i18n={AUSPICE_I18N}>
      <ThemeProvider theme={SIDEBAR_THEME}>
        <Provider store={store}>
          <AuspiceLayout files={files} treeLabels={treeLabels} axisTitle={axisTitle} />
        </Provider>
      </ThemeProvider>
    </I18nextProvider>
  );
}

export interface AuspiceViewProps extends AuspiceInput {
  files: AuspiceFiles | undefined;
  treeLabels: readonly string[];
  axisTitle: string | undefined;
}

function AuspiceLayout({ files, treeLabels, axisTitle }: Omit<AuspiceViewProps, keyof AuspiceInput>) {
  const [size, setSize] = useState<MeasuredSize | null>(null);
  const { areaRef, canvasRef } = useMeasuredSize(setSize, { beforePaint: true });
  const isOverlay = size !== null && sidebarOverlays(size.areaWidth);
  const [isOpen, setOpen] = usePaneOpen("auspice", isOverlay);

  return (
    <div ref={areaRef} className="light-scope absolute inset-0 isolate flex overflow-hidden bg-[#fff] text-[#000]">
      <CollapsiblePane
        side="left"
        width={SIDEBAR_WIDTH_PX}
        title="Auspice controls"
        name="Auspice controls"
        isOpen={isOpen}
        onOpenChange={setOpen}
        isOverlay={isOverlay}
        look="auspice"
        pane={SIDEBAR}
      >
        <div className="flex min-h-0 flex-1 pl-4">
          <div ref={canvasRef} className="relative min-h-0 min-w-0 flex-1 overflow-hidden [contain:size]">
            <SizedTree size={size} files={files} treeLabels={treeLabels} axisTitle={axisTitle} />
          </div>
        </div>
      </CollapsiblePane>
    </div>
  );
}

function SizedTree({
  size,
  files,
  treeLabels,
  axisTitle,
}: { size: MeasuredSize | null } & Omit<AuspiceViewProps, keyof AuspiceInput>) {
  const fitted = size === null ? null : treeSize(size.canvas);

  if (fitted === null) {
    return null;
  }

  return (
    <div className="relative">
      <div className="auspice-card">
        <Tree
          width={fitted.width}
          height={fitted.height}
          axisTitle={axisTitle}
          showTreeButtons={false}
          showNodeClickedPanel={false}
        />
      </div>
      <AuspiceHeader files={files} treeLabels={treeLabels} />
    </div>
  );
}

function AuspiceSidebar() {
  return (
    <ControlsContainer className="auspice-controls auspice-badges">
      <ControlHeader title="Color By" tooltip={ColorByInfo} />
      <ColorBy />
      <ControlHeader title="Filter Data" tooltip={FilterInfo} />
      <FilterData measurementsOn={false} />
      <div className="h-2.5 shrink-0" />
      <div className="flex flex-col">
        <div className="mt-2 border-t-[0.5px] border-[#495057] pt-4 pb-2">
          <AnnotatedTitle title="Tree" tooltip={TreeInfo} />
        </div>
        <ChooseLayout />
        <ToggleFocus />
        <div className="h-2.5 shrink-0" />
        <ChooseMetric />
        <ChooseBranchLabelling />
        <div className="h-[25px] shrink-0" />
        <ChooseTipLabel />
        <div className="h-2.5 shrink-0" />
        <ToggleTangle />
      </div>
    </ControlsContainer>
  );
}
