import type { Settings } from "@neherlab/treeknit-wasm";
import { useCallback, useMemo } from "react";
import { Form, type FormProps } from "react-aria-components";
import { FormProvider, useForm, useWatch } from "react-hook-form";
import RunIcon from "~icons/lucide/play";

import { Results } from "../results/Results";
import { Button } from "../ui/Button";
import { useAnalysis, useDefaultSettings } from "./context";
import { type AnalysisForm, analysisRequest } from "./request";
import { SettingsFields } from "./SettingsFields";
import { TreeInputs } from "./TreeInputs";

export function AnalysisPage() {
  const defaults = useDefaultSettings();
  const defaultValues = useMemo(() => formDefaults(defaults), [defaults]);
  const form = useForm<AnalysisForm>({ defaultValues });
  const analysis = useAnalysis();
  const { mutate } = analysis;

  const { handleSubmit } = form;

  const run = useCallback<SubmitHandler>(
    (event) => {
      void handleSubmit((values) => {
        mutate(analysisRequest(values));
      })(event);
    },
    [handleSubmit, mutate],
  );

  return (
    <div className="grid gap-10 lg:grid-cols-[minmax(0,28rem)_minmax(0,1fr)]">
      <FormProvider {...form}>
        <Form onSubmit={run} className="flex flex-col gap-8">
          <TreeInputs />
          <SettingsFields />
          <RunButton isPending={analysis.isPending} />
        </Form>
      </FormProvider>
      <section aria-labelledby="results-heading" aria-busy={analysis.isPending} className="flex flex-col gap-4">
        <h2 id="results-heading" className="text-xl font-semibold">
          Results
        </h2>
        {analysis.isError ? (
          <p role="alert" className="text-danger">
            {analysis.error.message}
          </p>
        ) : null}
        {analysis.isSuccess ? (
          <Results analysis={analysis.data} />
        ) : (
          <p className="text-ink-muted">{analysis.isPending ? "Running the analysis..." : "No results yet."}</p>
        )}
      </section>
    </div>
  );
}

type SubmitHandler = NonNullable<FormProps["onSubmit"]>;

function RunButton({ isPending }: { isPending: boolean }) {
  const trees = useWatch<AnalysisForm, "trees">({ name: "trees" });

  return (
    <Button type="submit" variant="primary" isPending={isPending} isDisabled={trees.length < 2} className="self-start">
      <RunIcon aria-hidden />
      {isPending ? "Running" : "Run"}
    </Button>
  );
}

function formDefaults(settings: Settings): AnalysisForm {
  return { trees: [], settings };
}
