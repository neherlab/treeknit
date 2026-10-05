import { cn } from "cn";
import { composeRenderProps, Link as AriaLink, type LinkProps as AriaLinkProps } from "react-aria-components";

import { focusRing } from "./styles";

export function ExternalLink({ className, ...props }: ExternalLinkProps) {
  return (
    <AriaLink
      {...props}
      target="_blank"
      rel="noreferrer"
      className={composeRenderProps(className, (custom) =>
        cn(
          "rounded-control text-ink decoration-rule cursor-pointer underline underline-offset-3",
          "data-hovered:decoration-ink transition-colors duration-150 motion-reduce:transition-none",
          focusRing,
          custom,
        ),
      )}
    />
  );
}

export type ExternalLinkProps = Omit<AriaLinkProps, "target" | "rel"> & { href: string };
