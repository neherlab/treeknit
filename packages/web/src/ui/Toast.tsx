import { cn } from "cn";
import { type CSSProperties, useMemo, useState } from "react";
import { mergeProps, useFocusWithin, useHover } from "react-aria";
import {
  Button,
  type QueuedToast,
  UNSTABLE_Toast as AriaToast,
  UNSTABLE_ToastContent as AriaToastContent,
  type UNSTABLE_ToastQueue as ToastQueue,
  UNSTABLE_ToastRegion as AriaToastRegion,
} from "react-aria-components";
import CloseIcon from "~icons/lucide/x";

import { useCountdown } from "./countdown";
import { IconButton } from "./IconButton";
import { buttonStyle } from "./styles";

const RING_RADIUS = 7;

const RING_CIRCUMFERENCE = 2 * Math.PI * RING_RADIUS;

export function ActionToastRegion({ queue }: { queue: ToastQueue<ActionToastContent> }) {
  return (
    <AriaToastRegion
      queue={queue}
      className="fixed bottom-4 left-1/2 z-50 flex -translate-x-1/2 flex-col items-center gap-2 outline-hidden"
    >
      {({ toast }) => <ActionToast toast={toast} />}
    </AriaToastRegion>
  );
}

export interface ActionToastContent {
  message: string;
  actionLabel: string;
  onAction: () => void;
  timeoutMs: number;
}

function ActionToast({ toast }: { toast: QueuedToast<ActionToastContent> }) {
  const { message, actionLabel, onAction, timeoutMs } = toast.content;
  const [hovered, setHovered] = useState(false);
  const [focused, setFocused] = useState(false);
  const { hoverProps } = useHover({ onHoverChange: setHovered });
  const { focusWithinProps } = useFocusWithin({ onFocusWithinChange: setFocused });
  const paused = hovered || focused;
  const seconds = useCountdown(timeoutMs, paused);
  const countdownStyle = useMemo((): CountdownStyle => ({ "--countdown-ms": `${String(timeoutMs)}ms` }), [timeoutMs]);

  return (
    <AriaToast
      toast={toast}
      className={cn(
        "rounded-control border-rule bg-pane text-ink border text-sm shadow-lg",
        "data-focus-visible:outline-focus outline-hidden data-focus-visible:outline-2 data-focus-visible:outline-offset-2",
      )}
    >
      <div {...mergeProps(hoverProps, focusWithinProps)} className="flex items-center gap-3 py-1.5 pr-1.5 pl-3">
        <AriaToastContent className="min-w-0">
          <span>{message}</span>
        </AriaToastContent>
        <Button
          aria-label={`${actionLabel}: ${message}`}
          onPress={onAction}
          className={buttonStyle({ variant: "quiet", size: "sm" })}
        >
          <span aria-hidden className="relative inline-flex size-5 items-center justify-center">
            <svg viewBox="0 0 20 20" className="absolute inset-0 size-5 -rotate-90">
              <circle cx={10} cy={10} r={RING_RADIUS} className="stroke-rule fill-none stroke-2" />
              <circle
                cx={10}
                cy={10}
                r={RING_RADIUS}
                strokeDasharray={RING_CIRCUMFERENCE}
                // oxlint-disable-next-line react/forbid-dom-props -- the ring runs for the timeout of each toast, which a class cannot express; the style only sets the custom property that the animation reads
                style={countdownStyle}
                className={cn(
                  "stroke-ink fill-none stroke-2 motion-safe:animate-[countdown_var(--countdown-ms)_linear_forwards] motion-reduce:hidden",
                  paused && "[animation-play-state:paused]",
                )}
              />
            </svg>
            <span className="text-[10px] leading-none tabular-nums">{seconds}</span>
          </span>
          <span aria-hidden>{actionLabel}</span>
        </Button>
        <IconButton slot="close" label="Dismiss" icon={CloseIcon} size="sm" />
      </div>
    </AriaToast>
  );
}

type CountdownStyle = CSSProperties & Record<"--countdown-ms", string>;
