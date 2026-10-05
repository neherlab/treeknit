import type { ComponentType } from "react";

declare const Tree: ComponentType<{
  width: number;
  height: number;
  axisTitle?: string | undefined;
  showTreeButtons?: boolean;
  showNodeClickedPanel?: boolean;
}>;

export default Tree;
