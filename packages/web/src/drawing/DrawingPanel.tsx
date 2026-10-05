import type { ReactNode } from "react";
import { useKeyboard } from "react-aria";

import { InlineNotice } from "../ui/InlineNotice";
import { ProgressBar } from "../ui/ProgressBar";

export function DrawingPanel({ toolbar, loading, error, errorTitle, notice, onEscape, children }: DrawingPanelProps) {
  const { keyboardProps } = useKeyboard({
    onKeyDown: (event) => {
      if (event.key === "Escape") {
        onEscape();
      } else {
        event.continuePropagation();
      }
    },
  });

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="border-rule flex shrink-0 flex-wrap items-center gap-x-3 gap-y-1.5 border-b px-3 py-1.5">
        {toolbar}
      </div>
      {notice === undefined ? null : <div className="px-3 pt-3">{notice}</div>}
      <div {...keyboardProps} className="relative min-h-0 flex-1">
        <DrawingBody loading={loading} error={error} errorTitle={errorTitle}>
          {children}
        </DrawingBody>
      </div>
    </div>
  );
}

export interface DrawingPanelProps {
  toolbar: ReactNode;
  loading?: string | undefined;
  error?: string | undefined;
  errorTitle: string;
  notice?: ReactNode;
  onEscape: () => void;
  children: ReactNode;
}

function DrawingBody({
  loading,
  error,
  errorTitle,
  children,
}: Pick<DrawingPanelProps, "loading" | "error" | "errorTitle" | "children">) {
  if (error !== undefined) {
    return (
      <div className="p-3">
        <InlineNotice tone="danger" title={errorTitle}>
          <p>{error}</p>
          <p>The tables and files are still available.</p>
        </InlineNotice>
      </div>
    );
  }

  if (loading !== undefined) {
    return (
      <div className="flex h-full items-center justify-center p-6">
        <ProgressBar label={loading} isIndeterminate className="w-64" />
      </div>
    );
  }

  return <div className="absolute inset-0">{children}</div>;
}
