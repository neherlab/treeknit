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
  return (
    <header className="border-rule bg-ground flex h-12 shrink-0 items-center gap-6 border-b px-4">
      <span className="text-ink text-lg font-semibold">TreeKnit</span>
      <nav aria-label="Main" className="flex flex-1 items-center gap-1">
        <Link to="/" className={navLinkStyle} activeOptions={EXACT_PATH} activeProps={ACTIVE_NAV_LINK_PROPS}>
          Workspace
        </Link>
        <Link to="/help" className={navLinkStyle} activeProps={ACTIVE_NAV_LINK_PROPS}>
          Help
        </Link>
      </nav>
      <ThemeToggle />
    </header>
  );
}
