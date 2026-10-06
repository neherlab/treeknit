import { useCallback } from "react";
import RetryIcon from "~icons/lucide/refresh-cw";

import { Button } from "../ui/Button";
import { InlineNotice, NoticeRegion } from "../ui/InlineNotice";
import { useLaunch } from "./context";

export function LaunchNotices() {
  const { state, control } = useLaunch();
  const { status, notes } = state;

  const retry = useCallback(() => {
    control.retry();
  }, [control]);

  const dismissStatus = useCallback(() => {
    control.dismissStatus();
  }, [control]);

  const dismissNotes = useCallback(() => {
    control.dismissNotes();
  }, [control]);

  return (
    <>
      <NoticeRegion>
        {status.kind === "loading" ? <InlineNotice tone="info">{status.message}</InlineNotice> : null}
      </NoticeRegion>
      {status.kind === "failed" ? (
        <InlineNotice
          tone="danger"
          title="The link could not be opened"
          action={
            <Button size="sm" icon={RetryIcon} onPress={retry}>
              Retry
            </Button>
          }
          onDismiss={dismissStatus}
        >
          <ul className="flex flex-col gap-1">
            {status.problems.map((problem) => (
              <li key={problem}>{problem}</li>
            ))}
          </ul>
        </InlineNotice>
      ) : null}
      {notes.length === 0 ? null : (
        <InlineNotice tone="warning" title="TreeKnit did not use every part of the link" onDismiss={dismissNotes}>
          <ul className="flex flex-col gap-1">
            {notes.map((note) => (
              <li key={note}>{note}</li>
            ))}
          </ul>
        </InlineNotice>
      )}
    </>
  );
}
