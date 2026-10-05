import { cn } from "cn";
import type { ReactNode } from "react";
import GitHubIcon from "~icons/lucide/github";
import TerminalIcon from "~icons/lucide/square-terminal";

import { useVersion } from "../analysis/queries";
import { CiteButton } from "../help/CiteButton";
import { ExternalIconLink } from "../ui/ExternalLink";
import { Link } from "../ui/Link";
import { usePaneOpen } from "./paneStore";
import { ThemeToggle } from "./ThemeToggle";
import { useShellLayout } from "./useShellLayout";

const navLinkStyle = [
  "text-ink-muted relative flex h-12 items-center px-2 text-sm no-underline -outline-offset-4",
  "data-hovered:text-ink",
  "after:absolute after:inset-x-2 after:bottom-0 after:h-0.5 after:bg-transparent",
].join(" ");

const ACTIVE_NAV_LINK_PROPS = { className: "text-ink after:bg-ink" };

const EXACT_PATH = { exact: true, includeSearch: false };

export function Header({ center, className }: { center?: ReactNode; className?: string }) {
  const { data: version } = useVersion();
  const layout = useShellLayout();
  const [railOpen] = usePaneOpen("rail", false);
  const [inspectorOpen] = usePaneOpen("inspector", false);
  const alignRail = layout.rail === "pane" && railOpen;
  const alignInspector = layout.inspector === "pane" && inspectorOpen;

  return (
    <header className={cn("border-rule bg-ground flex h-12 shrink-0 items-stretch border-b", className)}>
      <div
        className={cn("flex shrink-0 items-center gap-3 px-4 sm:gap-6", alignRail && "border-rule w-[336px] border-r")}
      >
        <span className="text-ink text-lg font-semibold">TreeKnit</span>
        <nav aria-label="Main" className="flex items-center gap-1">
          <Link to="/" className={navLinkStyle} activeOptions={EXACT_PATH} activeProps={ACTIVE_NAV_LINK_PROPS}>
            Workspace
          </Link>
          <Link to="/help" className={navLinkStyle} activeProps={ACTIVE_NAV_LINK_PROPS}>
            Help
          </Link>
        </nav>
      </div>
      <div className="flex min-w-0 flex-1 items-stretch">{center}</div>
      <div
        className={cn(
          "flex shrink-0 items-center justify-end gap-1 px-2",
          alignInspector && "border-rule w-[320px] border-l",
        )}
      >
        <CiteButton variant="quiet" size="sm" placement="bottom end" iconOnly={layout.inspector === "sheet"} />
        {version === undefined ? null : (
          <>
            <ExternalIconLink
              href={version.repository}
              label="Source code"
              tooltip={`Source code of TreeKnit ${version.version}`}
              icon={GitHubIcon}
              tooltipPlacement="bottom"
              className="max-sm:hidden"
            />
            <ExternalIconLink
              href={version.releases}
              label="Command-line releases"
              icon={TerminalIcon}
              tooltipPlacement="bottom"
              className="max-sm:hidden"
            />
          </>
        )}
        <ThemeToggle />
      </div>
    </header>
  );
}
