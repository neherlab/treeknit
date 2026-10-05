import { type ComponentType, useCallback, useMemo } from "react";
import type { Key } from "react-aria-components";

import { ArgPanel } from "../arg/ArgPanel";
import { AuspicePanel } from "../auspice/AuspicePanel";
import { Overview } from "../overview/Overview";
import { DiagnosticsTabLabel } from "../results/DiagnosticsTabLabel";
import { DiagnosticsView } from "../results/DiagnosticsView";
import { FilesView } from "../results/FilesView";
import { ConstellationPanel } from "../tables/ConstellationPanel";
import { MccTablePanel } from "../tables/MccTablePanel";
import { TanglegramPanel } from "../tanglegram/TanglegramPanel";
import { PanelBoundary } from "../ui/PanelBoundary";
import { Tab, TabList, TabPanel } from "../ui/Tabs";
import { useWorkspaceAvailability } from "../workspace/availability";
import { WORKSPACE_VIEWS, type WorkspaceView } from "../workspace/search";
import { useWorkspaceSearch } from "../workspace/useWorkspaceSearch";
import { type ViewTab, viewTabs } from "./viewTabs";

const VIEW_PANELS: Record<WorkspaceView, ComponentType> = {
  overview: Overview,
  tanglegram: TanglegramPanel,
  auspice: AuspicePanel,
  arg: ArgPanel,
  mccs: MccTablePanel,
  constellation: ConstellationPanel,
  files: FilesView,
  diagnostics: DiagnosticsView,
};

export function useViewTabs(): ViewTabsState {
  const { search, update } = useWorkspaceSearch();
  const availability = useWorkspaceAvailability();
  const tabs = useMemo(() => viewTabs(availability), [availability]);
  const disabledKeys = useMemo(() => tabs.filter((tab) => tab.isDisabled).map((tab) => tab.view), [tabs]);

  const selectView = useCallback(
    (key: Key) => {
      const view = WORKSPACE_VIEWS.find((candidate) => candidate === key);

      if (view !== undefined) {
        update((written) => ({ ...written, view }));
      }
    },
    [update],
  );

  return { tabs, disabledKeys, selectedKey: search.view, selectView };
}

export interface ViewTabsState {
  tabs: ViewTab[];
  disabledKeys: WorkspaceView[];
  selectedKey: WorkspaceView;
  selectView: (key: Key) => void;
}

export function ViewTabList({ tabs }: { tabs: readonly ViewTab[] }) {
  return (
    <TabList aria-label="Views" items={tabs} className="h-full min-w-0 flex-1 border-b-0">
      {(tab) => (
        <Tab id={tab.view} tooltip={tab.tooltip}>
          {tab.view === "diagnostics" ? <DiagnosticsTabLabel label={tab.label} /> : tab.label}
        </Tab>
      )}
    </TabList>
  );
}

export function ViewPanels({ tabs }: { tabs: readonly ViewTab[] }) {
  return tabs.map(({ view }) => {
    const Panel = VIEW_PANELS[view];

    return (
      <TabPanel key={view} id={view} className="relative min-h-0 overflow-y-auto">
        <PanelBoundary title="This view could not be shown">
          <Panel />
        </PanelBoundary>
      </TabPanel>
    );
  });
}
