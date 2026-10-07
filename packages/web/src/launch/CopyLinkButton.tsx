import type { AnalysisRequest } from "@neherlab/treeknit-wasm";
import { useMutation } from "@tanstack/react-query";
import { getRouteApi } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";
import { getErrorMessage } from "react-error-boundary";
import SaveIcon from "~icons/lucide/download";
import LinkIcon from "~icons/lucide/link";

import { useAnalysisClient } from "../analysis/context";
import { formatCount } from "../format/count";
import { useSaveSessionFile } from "../results/useSaveSessionFile";
import { Button } from "../ui/Button";
import { InlineNotice, NoticeRegion } from "../ui/InlineNotice";
import { useWorkspaceStore } from "../workspace/context";
import { shownLink } from "./addressBar";
import { type CopyCheck, copyCheck, inlineSessionLink } from "./copyLink";

const workspaceRoute = getRouteApi("/");

export function CopyLinkButton() {
  const client = useAnalysisClient();
  const store = useWorkspaceStore();
  const written = workspaceRoute.useSearch();
  const { mutate: save, error: saveError, reset: resetSave } = useSaveSessionFile();

  const {
    mutate: copy,
    data: outcome,
    error: copyError,
    reset: resetCopy,
  } = useMutation({
    mutationFn: async (): Promise<CopyOutcome> => {
      const link = shownLink(store.getState());

      const [pairs, limits] = await client.stateless(async (api) =>
        Promise.all([api.launchPairs(link), api.linkLimits()]),
      );

      const pageUrl = `${window.location.origin}${window.location.pathname}`;

      const text =
        pairs === undefined
          ? inlineSessionLink(pageUrl, written, await client.stateless(async (api) => api.inlineSession(link.request)))
          : window.location.href;

      const check = copyCheck(text, limits);

      if (check.kind === "copy") {
        await navigator.clipboard.writeText(text);
      }

      return { check, request: link.request, longLinkChars: limits.longLinkChars };
    },
  });

  const start = useCallback(() => {
    resetSave();
    copy();
  }, [copy, resetSave]);

  const dismiss = useCallback(() => {
    resetCopy();
    resetSave();
  }, [resetCopy, resetSave]);

  return (
    <div className="flex flex-col gap-1.5">
      <Button
        size="sm"
        icon={LinkIcon}
        tooltip="Copy a link that opens these trees, settings, and view"
        onPress={start}
      >
        Copy link
      </Button>
      <NoticeRegion>
        <CopyNotice outcome={outcome} copyError={copyError} saveError={saveError} onSave={save} onDismiss={dismiss} />
      </NoticeRegion>
    </div>
  );
}

function CopyNotice({ outcome, copyError, saveError, onSave, onDismiss }: CopyNoticeProps) {
  const request = outcome?.request;

  const saveRequest = useCallback(() => {
    if (request !== undefined) {
      onSave(request);
    }
  }, [onSave, request]);

  const saveButton = useMemo(
    () => (
      <Button size="sm" icon={SaveIcon} onPress={saveRequest}>
        Save session file
      </Button>
    ),
    [saveRequest],
  );

  if (copyError !== null) {
    return (
      <InlineNotice tone="danger" title="The link could not be copied" onDismiss={onDismiss}>
        {getErrorMessage(copyError) ?? String(copyError)}
      </InlineNotice>
    );
  }

  if (saveError !== null) {
    return (
      <InlineNotice tone="danger" title="The session file could not be saved" onDismiss={onDismiss}>
        {getErrorMessage(saveError) ?? String(saveError)}
      </InlineNotice>
    );
  }

  if (outcome === undefined) {
    return null;
  }

  const { check, longLinkChars } = outcome;

  if (check.kind === "tooLong") {
    return (
      <InlineNotice tone="warning" action={saveButton} onDismiss={onDismiss}>
        The link would have {formatCount(check.length)} characters, too many for a link. Share the session file instead.
      </InlineNotice>
    );
  }

  return (
    <InlineNotice tone={check.long ? "warning" : "info"} onDismiss={onDismiss}>
      Link copied, {formatCount(check.length)} characters.
      {check.long
        ? ` Chat apps, link shorteners, and QR codes may break links longer than ${formatCount(longLinkChars)} characters.`
        : null}
    </InlineNotice>
  );
}

interface CopyNoticeProps {
  outcome: CopyOutcome | undefined;
  copyError: Error | null;
  saveError: Error | null;
  onSave: (request: AnalysisRequest) => void;
  onDismiss: () => void;
}

interface CopyOutcome {
  check: CopyCheck;
  request: AnalysisRequest;
  longLinkChars: number;
}
