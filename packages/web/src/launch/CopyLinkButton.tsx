import type { AnalysisRequest } from "@neherlab/treeknit-wasm";
import { getRouteApi } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";
import SaveIcon from "~icons/lucide/download";
import LinkIcon from "~icons/lucide/link";

import type { AnalysisClient } from "../analysis/client";
import { useAnalysisClient } from "../analysis/context";
import { downloadFile } from "../download";
import { formatCount } from "../format/count";
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
  const [outcome, setOutcome] = useState<CopyOutcome | null>(null);

  const copy = useCallback(async () => {
    const link = shownLink(store.getState());
    const [pairs, limits] = await Promise.all([client.launchPairs(link), client.linkLimits()]);
    const pageUrl = `${window.location.origin}${window.location.pathname}`;

    const text =
      pairs === undefined
        ? inlineSessionLink(pageUrl, written, await client.inlineSession(link.request))
        : window.location.href;

    const check = copyCheck(text, limits);

    if (check.kind === "copy") {
      await navigator.clipboard.writeText(text);
    }

    setOutcome({ check, request: link.request, longLinkChars: limits.longLinkChars });
  }, [client, store, written]);

  const start = useCallback(() => {
    copy().catch((cause: unknown) => {
      setOutcome({ failure: cause instanceof Error ? cause.message : String(cause) });
    });
  }, [copy]);

  const dismiss = useCallback(() => {
    setOutcome(null);
  }, []);

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
      <NoticeRegion>{outcome === null ? null : <CopyNotice outcome={outcome} onDismiss={dismiss} />}</NoticeRegion>
    </div>
  );
}

function CopyNotice({ outcome, onDismiss }: { outcome: CopyOutcome; onDismiss: () => void }) {
  const client = useAnalysisClient();
  const request = "request" in outcome ? outcome.request : null;

  const save = useCallback(() => {
    if (request !== null) {
      void saveSessionFile(client, request);
    }
  }, [client, request]);

  const saveButton = useMemo(
    () => (
      <Button size="sm" icon={SaveIcon} onPress={save}>
        Save session file
      </Button>
    ),
    [save],
  );

  if ("failure" in outcome) {
    return (
      <InlineNotice tone="danger" title="The link could not be copied" onDismiss={onDismiss}>
        {outcome.failure}
      </InlineNotice>
    );
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

async function saveSessionFile(client: Pick<AnalysisClient, "requestFile">, request: AnalysisRequest): Promise<void> {
  const file = await client.requestFile(request);

  downloadFile({ name: file.path, mediaType: file.mediaType, content: file.text });
}

type CopyOutcome = { check: CopyCheck; request: AnalysisRequest; longLinkChars: number } | { failure: string };
