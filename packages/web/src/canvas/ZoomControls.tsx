import { useCallback } from "react";
import FitIcon from "~icons/lucide/scan";
import ZoomInIcon from "~icons/lucide/zoom-in";
import ZoomOutIcon from "~icons/lucide/zoom-out";

import { IconButton } from "../ui/IconButton";
import { type TreeViewHandle, useZoomRoom } from "./useTreeView";

export interface ZoomControlsProps {
  view: TreeViewHandle;
}

export function ZoomControls({ view }: ZoomControlsProps) {
  const { actions } = view;
  const { canZoomIn, canZoomOut } = useZoomRoom(view);

  const zoomIn = useCallback(() => {
    actions.zoomIn();
  }, [actions]);

  const zoomOut = useCallback(() => {
    actions.zoomOut();
  }, [actions]);

  const fit = useCallback(() => {
    actions.fit();
  }, [actions]);

  return (
    <div className="flex items-center gap-0.5">
      <IconButton label="Zoom in" icon={ZoomInIcon} size="sm" isDisabled={!canZoomIn} onPress={zoomIn} />
      <IconButton label="Zoom out" icon={ZoomOutIcon} size="sm" isDisabled={!canZoomOut} onPress={zoomOut} />
      <IconButton label="Fit to view" icon={FitIcon} size="sm" isDisabled={!canZoomOut} onPress={fit} />
    </div>
  );
}
