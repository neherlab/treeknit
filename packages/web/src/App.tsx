import { Suspense } from "react";
import { ErrorBoundary, type FallbackProps } from "react-error-boundary";

import { AnalysisPage } from "./analysis/AnalysisPage";

const LOADING = <p className="text-ink-muted">Loading TreeKnit...</p>;

export function App() {
  return (
    <div className="mx-auto flex min-h-screen max-w-7xl flex-col gap-10 px-6 py-8">
      <header className="border-rule flex flex-col gap-2 border-b pb-6">
        <h1 className="font-mono text-3xl font-semibold tracking-tight">TreeKnit</h1>
        <p className="text-ink-muted max-w-3xl">
          Infer reassortment from segment trees: maximally compatible clades (MCCs) for every pair of trees and, for two
          trees, an ancestral reassortment graph (ARG). The analysis runs in this browser; the trees never leave it.
        </p>
      </header>
      <main>
        <ErrorBoundary FallbackComponent={Failure}>
          <Suspense fallback={LOADING}>
            <AnalysisPage />
          </Suspense>
        </ErrorBoundary>
      </main>
    </div>
  );
}

function Failure({ error }: FallbackProps) {
  return (
    <div role="alert" className="flex flex-col gap-2">
      <p className="text-danger font-semibold">TreeKnit could not start.</p>
      <p className="font-mono text-sm">{error instanceof Error ? error.message : String(error)}</p>
    </div>
  );
}
