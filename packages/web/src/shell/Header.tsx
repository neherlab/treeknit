import GitHubIcon from "~icons/lucide/github";
import TerminalIcon from "~icons/lucide/square-terminal";

import { useVersion } from "../analysis/queries";
import { CiteButton } from "../help/CiteButton";
import { ExternalIconLink } from "../ui/ExternalLink";
import { Link } from "../ui/Link";
import { ThemeToggle } from "./ThemeToggle";

const navLinkStyle = [
  "text-ink-muted relative flex h-12 items-center px-2 text-sm no-underline -outline-offset-4",
  "data-hovered:text-ink",
  "after:absolute after:inset-x-2 after:bottom-0 after:h-0.5 after:bg-transparent",
].join(" ");

const ACTIVE_NAV_LINK_PROPS = { className: "text-ink after:bg-ink" };

const EXACT_PATH = { exact: true, includeSearch: false };

export function Header() {
  const { data: version } = useVersion();

  return (
    <header className="border-rule bg-ground flex h-12 shrink-0 items-center gap-3 border-b px-4 sm:gap-6">
      <span className="text-ink text-lg font-semibold">TreeKnit</span>
      <nav aria-label="Main" className="flex flex-1 items-center gap-1">
        <Link to="/" className={navLinkStyle} activeOptions={EXACT_PATH} activeProps={ACTIVE_NAV_LINK_PROPS}>
          Workspace
        </Link>
        <Link to="/help" className={navLinkStyle} activeProps={ACTIVE_NAV_LINK_PROPS}>
          Help
        </Link>
      </nav>
      <div className="flex items-center gap-1">
        <CiteButton variant="quiet" size="sm" placement="bottom end" />
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
