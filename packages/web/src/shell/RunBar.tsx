import { useCallback, useState } from "react";
import ErrorIcon from "~icons/lucide/circle-alert";
import PlayIcon from "~icons/lucide/play";
import StopIcon from "~icons/lucide/square";

import { formatElapsed } from "../format/elapsed";
import { useDelayedIndicator } from "../indicator/useDelayedIndicator";
import { progressLabel, progressPercent } from "../run/runControl";
import { RunFailureNotice } from "../run/RunFailureNotice";
import { useNow } from "../run/useNow";
import { useRunReadiness } from "../run/useRunReadiness";
import { Button } from "../ui/Button";
import { NoticeRegion } from "../ui/InlineNotice";
import { ProgressBar } from "../ui/ProgressBar";
import { errorStyle } from "../ui/styles";
import { useWorkspace } from "../workspace/context";
import type { RunState } from "../workspace/store";
import { useRunAnalysis } from "../workspace/useRunAnalysis";

type RunningState = Extract<RunState, { status: "running" }>;

export function RunBar() {
  const run = useWorkspace((state) => state.run);
  const { run: start, cancel } = useRunAnalysis();
  const { blockedReason, generalErrors } = useRunReadiness();
  const running = run.status === "running";
  const showProgress = useDelayedIndicator(running);
  const [shownRun, setShownRun] = useState<RunningState | null>(null);

  if (run.status === "running" && run !== shownRun) {
    setShownRun(run);
  }

  const runAgain = useCallback(() => {
    start();
  }, [start]);

  return (
    <div className="flex flex-col gap-3 px-4 py-3">
      <NoticeRegion>
        {run.status === "failed" && run.kind === "cancelled" ? (
          <RunFailureNotice kind={run.kind} message={run.message} request={run.request} onRunAgain={runAgain} />
        ) : null}
      </NoticeRegion>
      {run.status === "failed" && run.kind !== "cancelled" ? (
        <RunFailureNotice kind={run.kind} message={run.message} request={run.request} onRunAgain={runAgain} />
      ) : null}
      {running || generalErrors.length === 0 ? null : (
        <ul className="flex flex-col gap-1">
          {generalErrors.map(({ message }) => (
            <li key={message} className={errorStyle}>
              <ErrorIcon aria-hidden />
              <span>{message}</span>
            </li>
          ))}
        </ul>
      )}
      {showProgress && shownRun !== null ? (
        <RunProgress run={shownRun} active={running} onCancel={cancel} />
      ) : (
        <div className="flex flex-col gap-1.5">
          <Button
            variant="primary"
            icon={PlayIcon}
            isDisabled={blockedReason !== null}
            isPending={running}
            onPress={start}
            className="w-full"
          >
            Run TreeKnit
          </Button>
          {blockedReason === null || running ? null : <p className="text-ink-muted text-xs">{blockedReason}</p>}
        </div>
      )}
    </div>
  );
}

function RunProgress({ run, active, onCancel }: RunProgressProps) {
  const now = useNow(active);
  const percent = progressPercent(run.progress);

  return (
    <div className="flex flex-col gap-2">
      <ProgressBar
        label={progressLabel(run.progress)}
        {...(percent === null ? { isIndeterminate: true } : { value: percent })}
      />
      <div className="flex items-center justify-between gap-3">
        <span className="text-ink-muted text-xs tabular-nums">
          <span className="sr-only">Elapsed time </span>
          {formatElapsed(now - run.startedAt)}
        </span>
        <Button size="sm" icon={StopIcon} isDisabled={!active} onPress={onCancel}>
          Cancel
        </Button>
      </div>
    </div>
  );
}

interface RunProgressProps {
  run: RunningState;
  active: boolean;
  onCancel: () => void;
}
