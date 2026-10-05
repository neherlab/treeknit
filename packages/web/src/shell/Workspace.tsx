import { useCallback, useState } from "react";

import { Dialog } from "../ui/Dialog";
import { CenterViews } from "./CenterViews";
import { Inspector } from "./Inspector";
import { INSPECTOR_TITLE, keepSheetOpen, type PanePlacement, RAIL_TITLE } from "./layout";
import { Rail } from "./Rail";
import { RunBar } from "./RunBar";
import { useShellLayout } from "./useShellLayout";

export function Workspace() {
  const layout = useShellLayout();
  const [railOpen, setRailOpen] = useSheetOpen(layout.rail);
  const [inspectorOpen, setInspectorOpen] = useSheetOpen(layout.inspector);
  const railSheet = layout.rail === "sheet";
  const inspectorSheet = layout.inspector === "sheet";

  const openRail = useCallback(() => {
    setRailOpen(true);
  }, [setRailOpen]);

  const openInspector = useCallback(() => {
    setInspectorOpen(true);
  }, [setInspectorOpen]);

  return (
    <div className="flex min-h-0 flex-1">
      {railSheet ? null : (
        <aside aria-label={RAIL_TITLE} className="border-rule bg-pane flex w-84 shrink-0 flex-col border-r">
          <div className="min-h-0 flex-1 overflow-y-auto">
            <Rail />
          </div>
          <div className="border-rule border-t">
            <RunBar />
          </div>
        </aside>
      )}
      <main className="flex min-w-0 flex-1 flex-col">
        <CenterViews
          onOpenRail={railSheet ? openRail : undefined}
          onOpenInspector={inspectorSheet ? openInspector : undefined}
        />
        {railSheet ? (
          <div className="border-rule bg-pane shrink-0 border-t">
            <RunBar />
          </div>
        ) : null}
      </main>
      {inspectorSheet ? null : (
        <aside aria-label={INSPECTOR_TITLE} className="border-rule bg-pane w-80 shrink-0 overflow-y-auto border-l">
          <Inspector />
        </aside>
      )}
      <Dialog title={RAIL_TITLE} placement="left" isOpen={railOpen} onOpenChange={setRailOpen}>
        <Rail />
      </Dialog>
      <Dialog
        title={INSPECTOR_TITLE}
        placement="right"
        isOpen={inspectorOpen}
        onOpenChange={setInspectorOpen}
      >
        <Inspector />
      </Dialog>
    </div>
  );
}

function useSheetOpen(placement: PanePlacement): [boolean, (open: boolean) => void] {
  const [open, setOpen] = useState(false);
  const kept = keepSheetOpen(open, placement);

  if (kept !== open) {
    setOpen(kept);
  }

  return [kept, setOpen];
}
