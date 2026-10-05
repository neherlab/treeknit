import { useCallback } from "react";

import { Button } from "../ui/Button";
import { InlineNotice } from "../ui/InlineNotice";
import { Switch } from "../ui/Switch";
import { usePersistenceSwitch } from "../workspace/context";
import type { PersistenceProblem, PersistenceProblemKind } from "./persistence";

export const KEEP_WORKSPACE = "Keep my workspace in this browser";

interface ProblemText {
  title: string;
  hint: string;
  canRetry: boolean;
}

const PROBLEM_TEXTS: Record<PersistenceProblemKind, ProblemText> = {
  restore: {
    title: "The workspace stored in this browser could not be restored.",
    hint: "It stays stored until you turn the switch on, which replaces it with the current workspace.",
    canRetry: false,
  },
  enable: {
    title: "This browser could not save the workspace.",
    hint: "Allow this page to store data and turn the switch on again.",
    canRetry: false,
  },
  save: {
    title: "The latest changes are not saved in this browser.",
    hint: "The next change tries again.",
    canRetry: true,
  },
  disable: {
    title: "The workspace could not be removed from this browser.",
    hint: "It stays stored in this browser. Try again.",
    canRetry: true,
  },
};

export function PersistenceSwitch() {
  const { enabled, problem, setEnabled, saveNow } = usePersistenceSwitch();

  const change = useCallback(
    (next: boolean) => {
      void setEnabled(next);
    },
    [setEnabled],
  );

  const retry = useCallback(() => {
    void (problem?.kind === "disable" ? setEnabled(false) : saveNow());
  }, [problem, setEnabled, saveNow]);

  return (
    <div className="flex flex-col gap-2">
      <Switch
        label={KEEP_WORKSPACE}
        info={
          <>
            <p>
              Your trees and settings are saved in this browser until you turn this off or clear the workspace. Other
              people using this browser can see them.
            </p>
            <p>With several tabs open, the last change is kept.</p>
          </>
        }
        isSelected={enabled}
        onChange={change}
      />
      {problem === null ? null : <ProblemNotice problem={problem} onRetry={retry} />}
    </div>
  );
}

function ProblemNotice({ problem, onRetry }: { problem: PersistenceProblem; onRetry: () => void }) {
  const text = PROBLEM_TEXTS[problem.kind];

  return (
    <InlineNotice tone="danger" title={text.title}>
      <div className="flex flex-col gap-2">
        <p>{problem.message}</p>
        <p>{text.hint}</p>
        {text.canRetry ? (
          <div>
            <Button size="sm" onPress={onRetry}>
              Try again
            </Button>
          </div>
        ) : null}
      </div>
    </InlineNotice>
  );
}
