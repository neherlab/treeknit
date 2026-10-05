import { type ReactNode, useEffect, useMemo, useRef } from "react";
import { FormProvider, useForm, useFormContext, useWatch } from "react-hook-form";

import { useWorkspace } from "../workspace/context";
import { draftFromSettings, hasDraft, nextDraft, type SettingsDraft } from "./draft";

export function SettingsFormProvider({ children }: SettingsFormProviderProps) {
  const settings = useWorkspace((state) => state.settings);
  const trees = useWorkspace((state) => state.trees);
  const resetRevision = useWorkspace((state) => state.resetRevision);
  const treeIds = useMemo(() => trees.map(({ id }) => id), [trees]);
  const form = useForm<SettingsDraft>({ defaultValues: draftFromSettings(settings, treeIds) });
  const syncedRevision = useRef(resetRevision);

  useEffect(() => {
    const replaced = syncedRevision.current !== resetRevision;

    syncedRevision.current = resetRevision;
    form.reset(nextDraft(form.getValues(), settings, treeIds, replaced));
  }, [form, settings, treeIds, resetRevision]);

  return <FormProvider {...form}>{children}</FormProvider>;
}

export interface SettingsFormProviderProps {
  children: ReactNode;
}

export function useSettingsDraftPending(): boolean {
  const { control } = useFormContext<SettingsDraft>();

  return useWatch({ control, compute: hasDraft });
}
