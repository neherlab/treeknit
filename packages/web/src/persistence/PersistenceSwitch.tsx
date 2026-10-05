import { useCallback, useState } from "react";

import { InlineNotice } from "../ui/InlineNotice";
import { Switch } from "../ui/Switch";
import { usePersistenceSwitch } from "../workspace/context";

export const KEEP_WORKSPACE = "Keep my workspace in this browser";

export function PersistenceSwitch() {
  const { enabled, setEnabled } = usePersistenceSwitch();
  const [error, setError] = useState<string | null>(null);

  const change = useCallback(
    (next: boolean) => {
      setError(null);
      setEnabled(next).catch((cause: Error) => {
        setError(
          `This browser could not save the workspace: ${cause.message}. Allow this page to store data and turn the switch on again.`,
        );
      });
    },
    [setEnabled],
  );

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
      {error === null ? null : <InlineNotice tone="danger">{error}</InlineNotice>}
    </div>
  );
}
