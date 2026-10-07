import { type ReactNode, useMemo } from "react";
import { getErrorMessage } from "react-error-boundary";

import { ErrorNotice } from "./ErrorNotice";
import { Loading } from "./Loading";

export function QueryState<T>({ query, loading, errorTitle, errorNote, children }: QueryStateProps<T>) {
  const errorDetails = useMemo(
    () => [getErrorMessage(query.error) ?? String(query.error), ...(errorNote === undefined ? [] : [errorNote])],
    [query.error, errorNote],
  );

  if (query.isError) {
    return (
      <div className="p-3">
        <ErrorNotice title={errorTitle} details={errorDetails} />
      </div>
    );
  }

  if (query.data === undefined) {
    return <Loading label={loading} />;
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
