import {
  isViewAvailable,
  viewFitsTreeCount,
  WORKSPACE_VIEWS,
  type WorkspaceAvailability,
  type WorkspaceView,
} from "../workspace/search";

export interface ViewTab {
  view: WorkspaceView;
  label: string;
  isDisabled: boolean;
}

const VIEW_LABELS: Record<WorkspaceView, string> = {
  overview: "Overview",
  tanglegram: "Tanglegram",
  arg: "ARG",
  mccs: "MCCs",
  constellation: "Constellation",
  files: "Files",
  diagnostics: "Diagnostics",
};

export function viewTabs(availability: WorkspaceAvailability): ViewTab[] {
  const treeCount = availability.hasResult ? availability.resultTreeCount : availability.treeCount;

  return WORKSPACE_VIEWS.filter((view) => viewFitsTreeCount(view, treeCount)).map((view) => ({
    view,
    label: VIEW_LABELS[view],
    isDisabled: !isViewAvailable(view, availability),
  }));
}
