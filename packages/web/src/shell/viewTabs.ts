import { isViewAvailable, WORKSPACE_VIEWS, type WorkspaceAvailability, type WorkspaceView } from "../workspace/search";

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

const VIEW_FITS_TREE_COUNT: Partial<Record<WorkspaceView, (treeCount: number) => boolean>> = {
  arg: (treeCount) => treeCount === 2,
  constellation: (treeCount) => treeCount >= 3,
};

export function viewTabs(availability: WorkspaceAvailability): ViewTab[] {
  const treeCount = availability.hasResult ? availability.resultTreeCount : availability.treeCount;

  return WORKSPACE_VIEWS.filter((view) => VIEW_FITS_TREE_COUNT[view]?.(treeCount) ?? true).map((view) => ({
    view,
    label: VIEW_LABELS[view],
    isDisabled: !isViewAvailable(view, availability),
  }));
}
