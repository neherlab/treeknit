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
import { DownloadButtons } from "auspice/src/components/download/downloadButtons";
import { publications } from "auspice/src/components/download/downloadModal";
import FiltersSummary from "auspice/src/components/info/filtersSummary";
import Tree from "auspice/src/components/tree";
import { calcUsableWidth } from "auspice/src/util/computeResponsive";
import { useCallback, useState } from "react";
import { I18nextProvider } from "react-i18next";
import { Provider } from "react-redux";
import { ThemeProvider } from "styled-components";
import DownloadIcon from "~icons/lucide/download";

import { useMeasuredSize } from "../canvas/useMeasuredSize";
import type { MeasuredSize } from "../canvas/viewState";
import { doiUrl, inlineAuthors, TREEKNIT_PUBLICATION } from "../help/citation";
import { Button } from "../ui/Button";
import { AUSPICE_I18N } from "./i18n";
import { type AuspiceInput, useAuspiceStore } from "./useAuspiceStore";

const SIDEBAR_THEME = {
  background: "var(--color-pane)",
  color: "var(--color-ink)",
  "font-family": "var(--font-sans)",
  sidebarBoxShadow: "rgba(0, 0, 0, 0.15)",
  selectedColor: "var(--color-focus)",
  unselectedColor: "var(--color-ink-muted)",
  alternateBackground: "var(--color-ground)",
};

const TREEKNIT_AUSPICE_PUBLICATION = {
  author: inlineAuthors(TREEKNIT_PUBLICATION),
  title: TREEKNIT_PUBLICATION.title,
  journal: TREEKNIT_PUBLICATION.journal,
  year: TREEKNIT_PUBLICATION.year,
  href: doiUrl(TREEKNIT_PUBLICATION),
};

const RELEVANT_PUBLICATIONS = [TREEKNIT_AUSPICE_PUBLICATION, publications.nextstrain];

const MIN_TREE_WIDTH = 320;

const MIN_TREE_HEIGHT = 480;

export default function AuspiceView(input: AuspiceInput) {
  const store = useAuspiceStore(input);
  const [downloadsOpen, setDownloadsOpen] = useState(false);

  const toggleDownloads = useCallback(() => {
    setDownloadsOpen((open) => !open);
  }, []);

  return (
    <I18nextProvider i18n={AUSPICE_I18N}>
      <ThemeProvider theme={SIDEBAR_THEME}>
        <Provider store={store}>
          <div className="light-scope bg-ground text-ink @container absolute inset-0 overflow-auto">
            <div className="grid min-h-full grid-cols-1 @[900px]:grid-cols-[260px_minmax(0,1fr)]">
              <aside
                aria-label="Auspice controls"
                className="border-rule bg-pane border-b **:box-content @[900px]:border-r @[900px]:border-b-0"
              >
                <ControlsContainer>
                  <ControlHeader title="Color By" tooltip={ColorByInfo} />
                  <ColorBy />
                  <ControlHeader title="Filter Data" tooltip={FilterInfo} />
                  <FilterData measurementsOn={false} />
                  <ControlHeader title="Tree" tooltip={TreeInfo} />
                  <ChooseLayout />
                  <ChooseMetric />
                  <ToggleFocus />
                  <ChooseBranchLabelling />
                  <ChooseTipLabel />
                  <ToggleTangle />
                </ControlsContainer>
              </aside>
              <div className="flex min-w-0 flex-col">
                <div className="flex items-start gap-2 px-3 pt-2">
                  <div className="min-w-0 flex-1 **:box-content">
                    <FiltersSummary />
                  </div>
                  <Button
                    variant="quiet"
                    size="sm"
                    icon={DownloadIcon}
                    aria-expanded={downloadsOpen}
                    onPress={toggleDownloads}
                  >
                    {downloadsOpen ? "Hide downloads" : "Download figure and data"}
                  </Button>
                </div>
                {downloadsOpen ? (
                  <div className="bg-ink text-ground rounded-control mx-3 mt-2 px-4 py-3 **:box-content">
                    <DownloadButtons relevantPublications={RELEVANT_PUBLICATIONS} />
                  </div>
                ) : null}
                <SizedTree />
              </div>
            </div>
          </div>
        </Provider>
      </ThemeProvider>
    </I18nextProvider>
  );
}

function SizedTree() {
  const [size, setSize] = useState<MeasuredSize | null>(null);
  const { areaRef, canvasRef } = useMeasuredSize(setSize);
  const width = size === null ? 0 : Math.floor(calcUsableWidth(size.canvas.width, 1));
  const height = Math.max(MIN_TREE_HEIGHT, size?.canvas.height ?? 0);

  return (
    <div ref={areaRef} className="relative min-h-120 min-w-0 flex-1">
      <div ref={canvasRef} className="absolute inset-0 **:box-content">
        {width >= MIN_TREE_WIDTH ? <Tree width={width} height={height} /> : null}
      </div>
    </div>
  );
}
