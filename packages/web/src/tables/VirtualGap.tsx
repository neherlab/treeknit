import { useMemo } from "react";

export function VirtualGap({ heightPx }: { heightPx: number }) {
  const style = useMemo(() => ({ height: heightPx }), [heightPx]);

  if (heightPx <= 0) {
    return null;
  }

  // oxlint-disable-next-line react/forbid-dom-props -- the height of the rows outside the rendered window comes from the virtualizer at run time, which a class cannot express
  return <tr aria-hidden style={style} />;
}
