import { ADD_NOTIFICATION } from "auspice/src/actions/types";
import type { AnyAction } from "redux";
import * as z from "zod";

import type { NoticeTone } from "../ui/InlineNotice";
import type { AuspiceMiddleware } from "./store";

const NOTICE_TONES = { warning: "warning", error: "danger" } as const satisfies Readonly<Record<string, NoticeTone>>;

const NOTIFICATION_ACTION = z.object({
  type: z.literal(ADD_NOTIFICATION),
  notificationType: z.enum(["warning", "error"]),
  message: z.string(),
  details: z.string().optional(),
});

export interface AuspiceNotice {
  tone: NoticeTone;
  title: string;
  message: string;
}

export function auspiceNotice(action: AnyAction): AuspiceNotice | undefined {
  const notification = NOTIFICATION_ACTION.safeParse(action);

  if (!notification.success) {
    return undefined;
  }

  const { notificationType, message, details } = notification.data;

  return { tone: NOTICE_TONES[notificationType], title: message, message: details ?? "" };
}

export function noticeMiddleware(onNotice: (notice: AuspiceNotice) => void): AuspiceMiddleware {
  return () => (next) => (action: AnyAction) => {
    const result: unknown = next(action);
    const notice = auspiceNotice(action);

    if (notice !== undefined) {
      onNotice(notice);
    }

    return result;
  };
}
