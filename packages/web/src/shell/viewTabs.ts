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
  tooltip: string;
  isDisabled: boolean;
}

export const UNAVAILABLE_VIEW_TOOLTIP = "Available after a run";

const VIEW_LABELS: Record<WorkspaceView, string> = {
  overview: "Overview",
  tanglegram: "Tanglegram",
  auspice: "Auspice",
  arg: "ARG",
  mccs: "MCCs",
  constellation: "Constellation",
  files: "Files",
  diagnostics: "Diagnostics",
};

const VIEW_DESCRIPTIONS: Record<WorkspaceView, string> = {
  overview: "The trees, their leaves, and the result of the run",
  tanglegram: "Both trees side by side, linked by MCC",
  auspice: "The trees of a pair in Auspice, with its colorings, filters, and layouts",
  arg: "The ancestral reassortment graph of the two trees",
  mccs: "The MCCs of a pair, sorted and filtered in a table",
  constellation: "The MCC of every leaf in every pair",
  files: "Download the output files",
  diagnostics: "The warnings and the log of the run",
};

export function viewTabs(availability: WorkspaceAvailability): ViewTab[] {
  const treeCount = availability.hasResult ? availability.resultTreeCount : availability.treeCount;

  return WORKSPACE_VIEWS.filter((view) => viewFitsTreeCount(view, treeCount)).map((view) => {
    const isDisabled = !isViewAvailable(view, availability);

    return {
      view,
      label: VIEW_LABELS[view],
      tooltip: isDisabled ? UNAVAILABLE_VIEW_TOOLTIP : VIEW_DESCRIPTIONS[view],
      isDisabled,
    };
  });
}
