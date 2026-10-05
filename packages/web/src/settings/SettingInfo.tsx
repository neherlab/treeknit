import type { ToggleSetting } from "@neherlab/treeknit-wasm";

export function SettingInfo({ setting }: SettingInfoProps) {
  return (
    <>
      <p>{setting.help}</p>
      {setting.reason === null ? null : <p className="text-ink-muted">{setting.reason}</p>}
    </>
  );
}

export interface SettingInfoProps {
  setting: Pick<ToggleSetting, "help" | "reason">;
}
