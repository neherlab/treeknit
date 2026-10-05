import { EmptyState } from "../ui/EmptyState";
import { Link } from "../ui/Link";

const OPEN_WORKSPACE = <Link to="/">Open the workspace</Link>;

export function NotFound() {
  return (
    <main className="flex-1 overflow-y-auto px-6 py-8">
      <EmptyState title="This page does not exist." action={OPEN_WORKSPACE} />
    </main>
  );
}
