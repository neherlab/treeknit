import { cn } from "cn";
import type { ComponentType, ReactNode, SVGProps } from "react";

export function EmptyState({ icon: Icon, title, action, className, children }: EmptyStateProps) {
  return (
    <div className={cn("flex max-w-[60ch] flex-col items-start gap-2 text-sm", className)}>
      {Icon === undefined ? null : <Icon aria-hidden className="text-ink-muted mb-1 size-6" />}
      <p className="text-ink">{title}</p>
      {children === undefined ? null : <div className="text-ink-muted">{children}</div>}
      {action === undefined ? null : <div className="mt-2 flex flex-wrap items-center gap-2">{action}</div>}
    </div>
  );
}

export interface EmptyStateProps {
  icon?: ComponentType<SVGProps<SVGSVGElement>>;
  title: ReactNode;
  action?: ReactNode;
  className?: string;
  children?: ReactNode;
}
