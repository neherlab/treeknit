import { type ComponentType, useCallback, useMemo } from "react";
import type { Key } from "react-aria-components";
import PanelLeftIcon from "~icons/lucide/panel-left";
import PanelRightIcon from "~icons/lucide/panel-right";

import { ArgPanel } from "../arg/ArgPanel";
import { DiagnosticsTabLabel } from "../results/DiagnosticsTabLabel";
import { DiagnosticsView } from "../results/DiagnosticsView";
import { FilesView } from "../results/FilesView";
import { Overview } from "../overview/Overview";
import { TanglegramPanel } from "../tanglegram/TanglegramPanel";
import { Button } from "../ui/Button";
import { Tab, TabList, TabPanel, Tabs } from "../ui/Tabs";
import { useWorkspaceAvailability } from "../workspace/availability";
import { WORKSPACE_VIEWS, type WorkspaceView } from "../workspace/search";
import { useWorkspaceSearch } from "../workspace/useWorkspaceSearch";
import { INSPECTOR_TITLE, RAIL_TITLE } from "./layout";
import { ViewPlaceholder } from "./ViewPlaceholder";
import { viewTabs } from "./viewTabs";

const VIEW_PANELS: Record<WorkspaceView, ComponentType> = {
  overview: Overview,
  tanglegram: TanglegramPanel,
  arg: ArgPanel,
  mccs: ViewPlaceholder,
  constellation: ViewPlaceholder,
  files: FilesView,
  diagnostics: DiagnosticsView,
};

export function CenterViews({ onOpenRail, onOpenInspector }: CenterViewsProps) {
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

  return (
    <Tabs
      selectedKey={search.view}
      disabledKeys={disabledKeys}
      onSelectionChange={selectView}
      className="min-h-0 flex-1"
    >
      <div className="border-rule flex h-10 shrink-0 items-center gap-2 border-b px-2">
        {onOpenRail === undefined ? null : (
          <Button variant="quiet" size="sm" icon={PanelLeftIcon} onPress={onOpenRail}>
            {RAIL_TITLE}
          </Button>
        )}
        <TabList aria-label="Views" items={tabs} className="min-w-0 flex-1 overflow-x-auto border-b-0">
          {(tab) => (
            <Tab id={tab.view}>
              {tab.view === "diagnostics" ? <DiagnosticsTabLabel label={tab.label} /> : tab.label}
            </Tab>
          )}
        </TabList>
        {onOpenInspector === undefined ? null : (
          <Button variant="quiet" size="sm" icon={PanelRightIcon} onPress={onOpenInspector}>
            {INSPECTOR_TITLE}
          </Button>
        )}
      </div>
      {tabs.map(({ view }) => {
        const Panel = VIEW_PANELS[view];

        return (
          <TabPanel key={view} id={view} className="overflow-y-auto">
            <Panel />
          </TabPanel>
        );
      })}
    </Tabs>
  );
}

export interface CenterViewsProps {
  onOpenRail?: (() => void) | undefined;
  onOpenInspector?: (() => void) | undefined;
}
