import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "cn";
import { Button as AriaButton, type ButtonProps as AriaButtonProps, composeRenderProps } from "react-aria-components";

const buttonStyle = cva(
  "data-focus-visible:ring-accent data-focus-visible:ring-offset-paper inline-flex cursor-default items-center justify-center gap-2 rounded-md px-3 py-1.5 text-sm font-medium transition-colors outline-none data-disabled:opacity-50 data-focus-visible:ring-2 data-focus-visible:ring-offset-2",
  {
    variants: {
      variant: {
        primary: "bg-accent text-accent-ink data-hovered:bg-accent-strong data-pressed:bg-accent-strong",
        secondary: "border-rule bg-panel text-ink data-hovered:border-ink-muted data-pressed:border-ink border",
        quiet: "text-ink-muted data-hovered:text-ink data-pressed:text-ink px-1.5",
      },
    },
    defaultVariants: { variant: "secondary" },
  },
);

export function Button({ variant, className, ...props }: ButtonProps) {
  return (
    <AriaButton
      {...props}
      className={composeRenderProps(className, (custom) => cn(buttonStyle({ variant }), custom))}
    />
  );
}

export interface ButtonProps extends AriaButtonProps, VariantProps<typeof buttonStyle> {}
