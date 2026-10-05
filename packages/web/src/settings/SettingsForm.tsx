import type { ModeInfo, ToggleSetting } from "@neherlab/treeknit-wasm";
import { useCallback, useMemo, useState } from "react";
import { getErrorMessage } from "react-error-boundary";
import { useFormContext } from "react-hook-form";
import DicesIcon from "~icons/lucide/dices";

import { useSettingsSchema } from "../analysis/queries";
import { Disclosure } from "../ui/Disclosure";
import { IconButton } from "../ui/IconButton";
import { InlineNotice } from "../ui/InlineNotice";
import { Radio, RadioGroup } from "../ui/RadioGroup";
import { Switch } from "../ui/Switch";
import { useWorkspace } from "../workspace/context";
import { fieldMessage, settingField } from "../workspace/fieldErrors";
import { useFieldErrors } from "../workspace/useFieldErrors";
import type { SettingsDraft, ToggleSettingKey } from "./draft";
import { DraftNumberField } from "./DraftNumberField";
import { drawSeed } from "./seed";
import { SettingInfo } from "./SettingInfo";
import { useSettingsActions } from "./useSettingsActions";

const SEQ_LENGTHS_FAILED = "The sequence lengths could not be turned on. Try again.";

export function SettingsForm() {
  const settings = useWorkspace((state) => state.settings);
  const treeCount = useWorkspace((state) => state.trees.length);
  const { data: schema } = useSettingsSchema(treeCount, settings);
  const errors = useFieldErrors();
  const actions = useSettingsActions();
  const { setValue } = useFormContext<SettingsDraft>();
  const seedMax = schema?.settings.seed.max ?? null;

  const commitGamma = useCallback((value: number) => actions.setNumber("gamma", value), [actions]);
  const commitSeed = useCallback((value: number) => actions.setNumber("seed", value), [actions]);
  const commitRounds = useCallback((value: number) => actions.setNumber("rounds", value), [actions]);
  const commitSteps = useCallback((value: number) => actions.setNumber("nMcmcIt", value), [actions]);

  const newSeed = useCallback(() => {
    if (seedMax === null) {
      return;
    }

    const seed = drawSeed(seedMax);

    setValue("seed", seed);
    actions.setNumber("seed", seed);
  }, [actions, seedMax, setValue]);

  const selectMode = useCallback(
    (value: string) => {
      const mode = schema?.modes.find((candidate) => candidate.mode === value);

      if (mode !== undefined) {
        actions.setResolve(mode.mode);
      }
    },
    [actions, schema],
  );

  const [seqLengthsFailure, setSeqLengthsFailure] = useState<string | null>(null);

  const switchSeqLengths = useCallback(
    (enabled: boolean) => {
      setSeqLengthsFailure(null);
      actions.setSeqLengthsEnabled(enabled).catch((cause: unknown) => {
        setSeqLengthsFailure(getErrorMessage(cause) ?? String(cause));
      });
    },
    [actions],
  );

  const dismissSeqLengthsFailure = useCallback(() => {
    setSeqLengthsFailure(null);
  }, []);

  const modesInfo = useMemo(() => (schema === undefined ? null : <ModesInfo modes={schema.modes} />), [schema]);

  const seqLengthsInfo = useMemo(
    () => (schema === undefined ? null : <SettingInfo setting={schema.settings.seqLengths} />),
    [schema],
  );

  if (schema === undefined) {
    return null;
  }

  const fields = schema.settings;

  return (
    <section aria-labelledby="rail-settings" className="flex flex-col gap-4">
      <h2 id="rail-settings" className="text-ink text-sm font-semibold">
        Settings
      </h2>
      <DraftNumberField
        name="gamma"
        label="γ"
        setting={fields.gamma}
        error={fieldMessage(errors, settingField("gamma"))}
        onCommit={commitGamma}
      />
      <RadioGroup label="Resolution" info={modesInfo} value={settings.resolve ?? null} onChange={selectMode}>
        {schema.modes.map(({ mode, name }) => (
          <Radio key={mode} value={mode}>
            {name}
          </Radio>
        ))}
      </RadioGroup>
      <div className="flex items-start gap-1">
        <DraftNumberField
          name="seed"
          label="Seed"
          setting={fields.seed}
          error={fieldMessage(errors, settingField("seed"))}
          onCommit={commitSeed}
          className="flex-1"
        />
        <IconButton
          label="New seed"
          icon={DicesIcon}
          variant="secondary"
          isDisabled={!fields.seed.applies || seedMax === null}
          onPress={newSeed}
          className="mt-6.5"
        />
      </div>
      <Disclosure title="Advanced">
        <SettingSwitch name="preResolve" label="Pre-resolve" setting={fields.preResolve} />
        <DraftNumberField
          name="rounds"
          label="Rounds"
          setting={fields.rounds}
          error={fieldMessage(errors, settingField("rounds"))}
          onCommit={commitRounds}
        />
        <SettingSwitch name="finalRound" label="Final round" setting={fields.finalRound} />
        <SettingSwitch name="likelihood" label="Likelihood tie-break" setting={fields.likelihood} />
        <Switch
          label="Sequence lengths"
          info={seqLengthsInfo}
          isSelected={settings.seqLengths !== null && settings.seqLengths !== undefined}
          isDisabled={!fields.seqLengths.applies}
          onChange={switchSeqLengths}
        />
        {seqLengthsFailure === null ? null : (
          <InlineNotice tone="danger" title={SEQ_LENGTHS_FAILED} onDismiss={dismissSeqLengthsFailure}>
            {seqLengthsFailure}
          </InlineNotice>
        )}
        <DraftNumberField
          name="nMcmcIt"
          label="MCMC steps per leaf"
          setting={fields.nMcmcIt}
          error={fieldMessage(errors, settingField("nMcmcIt"))}
          onCommit={commitSteps}
        />
        <SettingSwitch name="naive" label="Naive mode" setting={fields.naive} />
      </Disclosure>
    </section>
  );
}

function SettingSwitch({ name, label, setting }: SettingSwitchProps) {
  const selected = useWorkspace((state) => state.settings[name] ?? setting.default);
  const actions = useSettingsActions();
  const change = useCallback((value: boolean) => actions.setToggle(name, value), [actions, name]);
  const info = useMemo(() => <SettingInfo setting={setting} />, [setting]);

  return <Switch label={label} info={info} isSelected={selected} isDisabled={!setting.applies} onChange={change} />;
}

interface SettingSwitchProps {
  name: ToggleSettingKey;
  label: string;
  setting: ToggleSetting;
}

function ModesInfo({ modes }: ModesInfoProps) {
  return (
    <dl className="flex flex-col gap-1.5">
      {modes.map(({ mode, name, effect }) => (
        <div key={mode}>
          <dt className="font-semibold">{name}</dt>
          <dd>{effect}</dd>
        </div>
      ))}
    </dl>
  );
}

interface ModesInfoProps {
  modes: readonly ModeInfo[];
}
