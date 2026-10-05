import { TreeList } from "../inputs/TreeList";
import { WorkspaceHeader } from "../inputs/WorkspaceHeader";
import { SettingsForm } from "../settings/SettingsForm";

export function Rail() {
  return (
    <div className="flex flex-col gap-6 px-4 py-4">
      <WorkspaceHeader />
      <TreeList />
      <SettingsForm />
    </div>
  );
}
