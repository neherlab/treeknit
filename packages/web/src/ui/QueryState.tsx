import type { ReactNode } from "react";
import { getErrorMessage } from "react-error-boundary";

import { InlineNotice } from "./InlineNotice";
import { ProgressBar } from "./ProgressBar";

export function QueryState<T>({ query, loading, errorTitle, errorNote, children }: QueryStateProps<T>) {
  if (query.isError) {
    return (
      <div className="p-3">
        <InlineNotice tone="danger" title={errorTitle}>
          <p>{getErrorMessage(query.error) ?? String(query.error)}</p>
          {errorNote === undefined ? null : <p>{errorNote}</p>}
        </InlineNotice>
      </div>
    );
  }

  if (query.data === undefined) {
    return (
      <div className="flex h-full items-center justify-center p-6">
        <ProgressBar label={loading} isIndeterminate className="w-64" />
      </div>
    );
  }

  return children(query.data);
}

export interface QueryStateProps<T> {
  query: { isError: boolean; error: unknown; data: T | undefined };
  loading: string;
  errorTitle: string;
  errorNote?: string;
  children: (data: T) => ReactNode;
}
