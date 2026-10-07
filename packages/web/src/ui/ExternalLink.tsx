import { cn } from "cn";
import type { ComponentType, ReactNode, SVGProps } from "react";
import { composeRenderProps, Link as AriaLink, type LinkProps as AriaLinkProps } from "react-aria-components";

import { buttonStyle, linkStyle } from "./styles";
import { TooltipTrigger, type TooltipTriggerProps } from "./TooltipTrigger";

export function ExternalLink({ className, ...props }: ExternalLinkProps) {
  return (
    <AriaLink
      {...props}
      target="_blank"
      rel="noreferrer"
      className={composeRenderProps(className, (custom) => cn(linkStyle, custom))}
    />
  );
}

export type ExternalLinkProps = Omit<AriaLinkProps, "target" | "rel"> & { href: string };

export function ExternalButtonLink({ href, tooltip, icon: Icon, children }: ExternalButtonLinkProps) {
  return (
    <TooltipTrigger tooltip={tooltip}>
      <AriaLink
        href={href}
        target="_blank"
        rel="noreferrer"
        className={buttonStyle({ variant: "secondary", size: "sm" })}
      >
        <Icon aria-hidden />
        {children}
      </AriaLink>
    </TooltipTrigger>
  );
}

export interface ExternalButtonLinkProps {
  href: string;
  tooltip: ReactNode;
  icon: ComponentType<SVGProps<SVGSVGElement>>;
  children: ReactNode;
}

export function ExternalIconLink({
  href,
  label,
  tooltip,
  icon: Icon,
  tooltipPlacement,
  className,
}: ExternalIconLinkProps) {
  return (
    <TooltipTrigger tooltip={tooltip ?? label} repeatsName={tooltip === undefined} placement={tooltipPlacement}>
      <AriaLink
        href={href}
        target="_blank"
        rel="noreferrer"
        aria-label={label}
        className={cn(buttonStyle({ variant: "quiet", size: "md", iconOnly: true }), className)}
      >
        <Icon aria-hidden />
      </AriaLink>
    </TooltipTrigger>
  );
}

export interface ExternalIconLinkProps {
  href: string;
  label: string;
  tooltip?: ReactNode;
  icon: ComponentType<SVGProps<SVGSVGElement>>;
  tooltipPlacement?: TooltipTriggerProps["placement"];
  className?: string;
}
