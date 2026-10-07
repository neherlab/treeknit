import { createLink } from "@tanstack/react-router";
import { cn } from "cn";
import { composeRenderProps, Link as AriaLink, type LinkProps as AriaLinkProps } from "react-aria-components";

import { linkStyle } from "./styles";

function StyledLink({ className, ...props }: AriaLinkProps) {
  return <AriaLink {...props} className={composeRenderProps(className, (custom) => cn(linkStyle, custom))} />;
}

export const Link = createLink(StyledLink);
