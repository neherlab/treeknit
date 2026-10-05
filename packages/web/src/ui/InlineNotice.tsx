import { cva } from "class-variance-authority";
import { cn } from "cn";
import type { ReactNode } from "react";
import DangerIcon from "~icons/lucide/circle-alert";
import InfoIcon from "~icons/lucide/info";
import WarningIcon from "~icons/lucide/triangle-alert";

import type { IconComponent } from "./icon";

export type NoticeTone = "info" | "warning" | "danger";

const TONE_ICONS: Record<NoticeTone, IconComponent> = {
  info: InfoIcon,
  warning: WarningIcon,
  danger: DangerIcon,
};

const noticeStyle = cva("rounded-control text-ink flex items-start gap-2 border px-3 py-2 text-sm", {
  variants: {
    tone: {
      info: "border-rule bg-pane",
      warning: "border-ink-muted bg-pane",
      danger: "border-danger/60 bg-danger/8",
    },
  },
});

const noticeIconStyle = cva("mt-px shrink-0", {
  variants: {
    tone: {
      info: "text-ink-muted",
      warning: "text-ink",
      danger: "text-danger",
    },
  },
});

export function InlineNotice({ tone, title, action, className, children }: InlineNoticeProps) {
  const Icon = TONE_ICONS[tone];

  return (
    <div role={tone === "danger" ? "alert" : undefined} className={cn(noticeStyle({ tone }), className)}>
      <Icon aria-hidden className={noticeIconStyle({ tone })} />
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        {title === undefined ? null : <p className="font-semibold">{title}</p>}
        {children === undefined ? null : <div className="wrap-break-word">{children}</div>}
      </div>
      {action === undefined ? null : <div className="-my-1 -mr-1.5 flex shrink-0 items-center gap-1">{action}</div>}
    </div>
  );
}

export interface InlineNoticeProps {
  tone: NoticeTone;
  title?: ReactNode;
  action?: ReactNode;
  className?: string;
  children?: ReactNode;
}

export function NoticeRegion({ className, children }: NoticeRegionProps) {
  return (
    // oxlint-disable-next-line jsx-a11y/prefer-tag-over-role -- <output> allows only phrasing content, and a notice holds paragraphs
    <div role="status" className={cn("flex flex-col gap-2", className)}>
      {children}
    </div>
  );
}

export interface NoticeRegionProps {
  className?: string;
  children?: ReactNode;
}
