import PlayIcon from "~icons/lucide/play";

import { Button } from "../ui/Button";

export function RunBar() {
  return (
    <div className="flex flex-col gap-2 px-4 py-3">
      <Button variant="primary" icon={PlayIcon} isDisabled className="w-full">
        Run TreeKnit
      </Button>
    </div>
  );
}
