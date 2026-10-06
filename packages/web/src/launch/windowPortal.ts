import type { MessageSource } from "@neherlab/treeknit-wasm";
import * as z from "zod";

import type { MessagePortal, MessageTarget } from "./message";

const messageTargetSchema = z.custom<MessageTarget>(
  (value) => value instanceof Object && "postMessage" in value,
  "not a window",
);

export function windowPortal(source: MessageSource): MessagePortal {
  const candidate: unknown = source === "opener" ? window.opener : framingWindow();

  return {
    target: messageTargetSchema.safeParse(candidate).data ?? null,
    listen: (listener) => {
      const forward = ({ source: sender, origin, data }: MessageEvent) => {
        listener({ source: sender, origin, data });
      };

      window.addEventListener("message", forward);

      return () => {
        window.removeEventListener("message", forward);
      };
    },
  };
}

function framingWindow(): Window | null {
  return window.parent === window ? null : window.parent;
}
