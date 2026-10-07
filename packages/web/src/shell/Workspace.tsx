import { useCallback, useMemo, useRef, useState } from "react";
import PanelLeftIcon from "~icons/lucide/panel-left";
import PanelRightIcon from "~icons/lucide/panel-right";

import { isBehindModal } from "../run/runControl";
import { useRunShortcut } from "../run/useRunShortcut";
import { SidePane } from "../ui/CollapsiblePane";
import { Dialog } from "../ui/Dialog";
import { IconButton } from "../ui/IconButton";
import { PanelBoundary } from "../ui/PanelBoundary";
import { Tabs } from "../ui/Tabs";
import { ActionToastRegion } from "../ui/Toast";
import { useDocumentKeyDown } from "../ui/useDocumentKeyDown";
import { useUndoToast } from "../workspace/useUndoToast";
import { useViewTabs, ViewPanels, ViewTabList } from "./CenterViews";
import { Header } from "./Header";
import { Inspector } from "./Inspector";
import {
  INSPECTOR_TITLE,
  INSPECTOR_WIDTH_CLASS,
  INSPECTOR_WIDTH_PX,
  keepSheetOpen,
  type PanePlacement,
  RAIL_TITLE,
  RAIL_WIDTH_CLASS,
  RAIL_WIDTH_PX,
} from "./layout";
import { isTextEntry, PANE_SHORTCUTS, paneShortcut } from "./panes";
import { togglePane, usePaneOpen } from "./paneStore";
import { Rail } from "./Rail";
import { RunBar } from "./RunBar";
import { useShellLayout } from "./useShellLayout";

const INSPECTOR = (
  <PanelBoundary title="The inspector could not be shown">
    <Inspector />
  </PanelBoundary>
);

export function Workspace() {
  const layout = useShellLayout();
  const [railSheetOpen, setRailSheetOpen] = useSheetOpen(layout.rail);
  const [inspectorSheetOpen, setInspectorSheetOpen] = useSheetOpen(layout.inspector);
  const [railOpen, setRailOpen] = usePaneOpen("rail", false);
  const [inspectorOpen, setInspectorOpen] = usePaneOpen("inspector", false);
  const railPane = layout.rail === "pane";
  const inspectorPane = layout.inspector === "pane";
  const { tabs, disabledKeys, selectedKey, selectView } = useViewTabs();
  const undoQueue = useUndoToast();
  const root = useRef<HTMLDivElement>(null);

  useRunShortcut(root);
  usePaneShortcuts(root, railPane, inspectorPane);

  const openRail = useCallback(() => {
    setRailSheetOpen(true);
  }, [setRailSheetOpen]);

  const openInspector = useCallback(() => {
    setInspectorSheetOpen(true);
  }, [setInspectorSheetOpen]);

  const inspectorButton = inspectorPane ? null : (
    <IconButton label={INSPECTOR_TITLE} icon={PanelRightIcon} onPress={openInspector} />
  );

  const tabList = <ViewTabList tabs={tabs} />;

  const railPaneContent = useMemo(() => <RailPane showRunBar={railOpen} />, [railOpen]);

  const main = (
    <main className="col-start-2 row-start-2 flex min-h-0 min-w-0 flex-col">
      {railPane ? null : (
        <div className="border-rule flex h-10 shrink-0 items-stretch gap-1 border-b px-1">
          <div className="flex items-center">
            <IconButton label={RAIL_TITLE} icon={PanelLeftIcon} onPress={openRail} />
          </div>
          {tabList}
          <div className="flex items-center">{inspectorButton}</div>
        </div>
      )}
      <ViewPanels tabs={tabs} />
      {railPane && railOpen ? null : (
        <div className="border-rule bg-pane shrink-0 border-t">
          <RunBar />
        </div>
      )}
    </main>
  );

  const headerCenter = railPane ? (
    <>
      {tabList}
      {inspectorButton === null ? null : <div className="flex items-center px-1">{inspectorButton}</div>}
    </>
  ) : undefined;

  return (
    <div
      ref={root}
      className="@container isolate grid min-h-0 flex-1 grid-cols-[auto_minmax(0,1fr)_auto] grid-rows-[auto_minmax(0,1fr)]"
    >
      <Tabs selectedKey={selectedKey} disabledKeys={disabledKeys} onSelectionChange={selectView} className="contents">
        <Header center={headerCenter} className="col-span-3" />
        {main}
      </Tabs>
      {railPane ? (
        <SidePane
          className="col-start-1 row-start-2"
          side="left"
          width={RAIL_WIDTH_PX}
          title={RAIL_TITLE}
          name="trees and settings"
          shortcut={PANE_SHORTCUTS.rail}
          isOpen={railOpen}
          onOpenChange={setRailOpen}
          isOverlay={false}
          look="app"
          pane={railPaneContent}
        />
      ) : null}
      {inspectorPane ? (
        <SidePane
          className="col-start-3 row-start-2"
          side="right"
          width={INSPECTOR_WIDTH_PX}
          title={INSPECTOR_TITLE}
          name="details"
          shortcut={PANE_SHORTCUTS.inspector}
          isOpen={inspectorOpen}
          onOpenChange={setInspectorOpen}
          isOverlay={false}
          look="app"
          pane={INSPECTOR}
        />
      ) : null}
      <Dialog
        title={RAIL_TITLE}
        placement="left"
        className={RAIL_WIDTH_CLASS}
        isOpen={railSheetOpen}
        onOpenChange={setRailSheetOpen}
      >
        <Rail />
      </Dialog>
      <Dialog
        title={INSPECTOR_TITLE}
        placement="right"
        className={INSPECTOR_WIDTH_CLASS}
        isOpen={inspectorSheetOpen}
        onOpenChange={setInspectorSheetOpen}
      >
        {INSPECTOR}
      </Dialog>
      <ActionToastRegion queue={undoQueue} />
    </div>
  );
}

function RailPane({ showRunBar }: { showRunBar: boolean }) {
  return (
    <div className="flex h-full flex-col">
      <div className="min-h-0 flex-1 overflow-y-auto">
        <Rail />
      </div>
      {showRunBar ? (
        <div className="border-rule border-t">
          <RunBar />
        </div>
      ) : null}
    </div>
  );
}

function usePaneShortcuts(root: { current: HTMLElement | null }, railPane: boolean, inspectorPane: boolean): void {
  useDocumentKeyDown((event: KeyboardEvent) => {
    const pane = paneShortcut(event);
    const element = root.current;

    if (pane === null || element === null || isBehindModal(element)) {
      return;
    }

    const focused = document.activeElement;

    if (focused instanceof HTMLElement && isTextEntry(textEntryOf(focused))) {
      return;
    }

    if ((pane === "rail" && railPane) || (pane === "inspector" && inspectorPane)) {
      event.preventDefault();
      togglePane(pane);
    }
  });
}

function textEntryOf(element: HTMLElement) {
  return {
    tagName: element.tagName,
    isContentEditable: element.isContentEditable,
    type: element instanceof HTMLInputElement ? element.type : undefined,
  };
}

function useSheetOpen(placement: PanePlacement): [boolean, (open: boolean) => void] {
  const [open, setOpen] = useState(false);
  const kept = keepSheetOpen(open, placement);

  if (kept !== open) {
    setOpen(kept);
  }

  return [kept, setOpen];
}
