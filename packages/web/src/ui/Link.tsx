import { createLink } from "@tanstack/react-router";
import { cn } from "cn";
import { composeRenderProps, Link as AriaLink, type LinkProps as AriaLinkProps } from "react-aria-components";

import { focusRing } from "./styles";

function StyledLink({ className, ...props }: AriaLinkProps) {
  return (
    <AriaLink
      {...props}
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

export const Link = createLink(StyledLink);
