import type { ReactNode } from "react";

import { InlineNotice, NoticeRegion, type NoticeTone } from "../ui/InlineNotice";
import { QueryState, type QueryStateProps } from "../ui/QueryState";
import { useEscapeKey } from "../ui/useEscapeKey";

const OTHER_VIEWS_AVAILABLE = "The tables and files are still available.";

export function DrawingPanel<T>({
  toolbar,
  notice,
  failure,
  onEscape,
  query,
  loading,
  errorTitle,
  children,
}: DrawingPanelProps<T>) {
  const keyboardProps = useEscapeKey(onEscape);

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="border-rule flex shrink-0 flex-wrap items-center gap-x-3 gap-y-1.5 border-b px-3 py-1.5">
        {toolbar}
      </div>
      {failure === undefined ? null : (
        <div className="px-3 pt-3">
          <InlineNotice tone="danger" title={failure.title} onDismiss={failure.onDismiss}>
            {failure.message}
          </InlineNotice>
        </div>
      )}
      <NoticeRegion>
        {notice === undefined ? null : (
          <div className="px-3 pt-3">
            <InlineNotice tone={notice.tone}>{notice.text}</InlineNotice>
          </div>
        )}
      </NoticeRegion>
      <div {...keyboardProps} className="relative min-h-0 flex-1">
        <QueryState query={query} loading={loading} errorTitle={errorTitle} errorNote={OTHER_VIEWS_AVAILABLE}>
          {(data) => <div className="absolute inset-0">{children(data)}</div>}
        </QueryState>
      </div>
    </div>
  );
}

export interface DrawingPanelProps<T> extends Pick<QueryStateProps<T>, "query" | "loading" | "errorTitle"> {
  toolbar: ReactNode;
  notice?: DrawingNotice | undefined;
  failure?: DrawingFailure | undefined;
  onEscape: (() => void) | null;
  children: (data: T) => ReactNode;
}

export interface DrawingNotice {
  tone: NoticeTone;
  text: string;
}

export interface DrawingFailure {
  title: string;
  message: string;
  onDismiss: () => void;
}
