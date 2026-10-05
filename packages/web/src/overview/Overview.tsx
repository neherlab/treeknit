import { cn } from "cn";

import { EmptyCenter } from "../shell/EmptyCenter";
import { InlineNotice, NoticeRegion } from "../ui/InlineNotice";
import { useWorkspace } from "../workspace/context";
import { selectStale } from "../workspace/store";
import { InputOverview } from "./InputOverview";
import { ResultOverview } from "./ResultOverview";

export const RESULTS_STALE = "Trees or settings changed. Run again to update the results.";

export const RUN_TO_SEE_RESULTS = "Run TreeKnit to see the results.";

export function Overview() {
  const treeCount = useWorkspace((state) => state.trees.length);
  const result = useWorkspace((state) => state.result);
  const restored = useWorkspace((state) => state.restored);
  const stale = useWorkspace(selectStale);

  if (treeCount === 0 && result === null) {
    return <EmptyCenter />;
  }

  return (
    <div className="flex max-w-5xl flex-col gap-10 px-8 py-6">
      <NoticeRegion>
        {result === null && restored ? <InlineNotice tone="info" title={RUN_TO_SEE_RESULTS} /> : null}
        {stale ? <InlineNotice tone="warning" title={RESULTS_STALE} /> : null}
      </NoticeRegion>
      {result === null ? null : (
        <div className={cn("transition-opacity duration-150 motion-reduce:transition-none", stale && "opacity-60")}>
          <ResultOverview result={result} />
        </div>
      )}
      {treeCount === 0 ? null : <InputOverview />}
    </div>
  );
}
