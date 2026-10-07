import type { AnalysisRequest } from "@neherlab/treeknit-wasm";
import { useMemo } from "react";
import RunIcon from "~icons/lucide/rotate-ccw";

import type { FailureKind } from "../analysis/client";
import { useVersion } from "../analysis/queries";
import { Button } from "../ui/Button";
import { ExternalLink } from "../ui/ExternalLink";
import { InlineNotice } from "../ui/InlineNotice";
import { bugReportUrl, failureNotice } from "./failure";

export function RunFailureNotice({ kind, message, request, runBlocked, onRunAgain }: RunFailureNoticeProps) {
  const notice = failureNotice(kind, message);
  const { data: version } = useVersion();
  const settings = request.settings;

  const reportUrl = useMemo(
    () =>
      version === undefined
        ? undefined
        : bugReportUrl(version.newIssue, {
            kind,
            version: version.version,
            userAgent: navigator.userAgent,
            settings: settings ?? {},
          }),
    [kind, settings, version],
  );

  if (notice.tone === "info") {
    return <InlineNotice tone="info" title={notice.title} />;
  }

  return (
    <InlineNotice tone="danger" title={notice.title}>
      <div className="flex flex-col gap-2">
        {notice.details.map((detail) => (
          <p key={detail}>{detail}</p>
        ))}
        {notice.canRunAgain || notice.canReport ? (
          <div className="flex flex-wrap items-center gap-3">
            {notice.canRunAgain ? (
              <Button size="sm" icon={RunIcon} isDisabled={runBlocked} onPress={onRunAgain}>
                Run again
              </Button>
            ) : null}
            {notice.canReport && reportUrl !== undefined ? (
              <ExternalLink href={reportUrl} className="text-sm">
                Report the problem
              </ExternalLink>
            ) : null}
          </div>
        ) : null}
      </div>
    </InlineNotice>
  );
}

export interface RunFailureNoticeProps {
  kind: FailureKind;
  message: string;
  request: AnalysisRequest;
  runBlocked: boolean;
  onRunAgain: () => void;
}
