import { cva } from "class-variance-authority";

export const focusRing =
  "outline-hidden data-focus-visible:outline-2 data-focus-visible:outline-offset-2 data-focus-visible:outline-focus";

export const groupFocusRing =
  "group-data-focus-visible:outline-2 group-data-focus-visible:outline-offset-2 group-data-focus-visible:outline-focus";

export const buttonStyle = cva(
  [
    "rounded-control inline-flex shrink-0 cursor-default items-center justify-center gap-1.5 border whitespace-nowrap select-none",
    "transition-colors duration-150 data-disabled:cursor-not-allowed motion-reduce:transition-none [&_svg]:shrink-0",
    focusRing,
  ],
  {
    variants: {
      variant: {
        primary: [
          "border-ink bg-ink text-ground font-semibold",
          "data-hovered:border-ink/85 data-hovered:bg-ink/85 data-pressed:border-ink/75 data-pressed:bg-ink/75",
          "data-disabled:border-rule data-disabled:bg-rule data-disabled:text-ink-muted",
        ],
        secondary: [
          "border-rule bg-ground text-ink",
          "data-hovered:border-ink-muted data-pressed:border-ink-muted data-pressed:bg-pane",
          "data-disabled:text-ink-muted data-disabled:bg-transparent",
        ],
        quiet: [
          "text-ink border-transparent bg-transparent",
          "data-hovered:bg-ink/8 data-pressed:bg-ink/14 data-disabled:text-ink-muted",
        ],
      },
      size: {
        md: "h-8 px-3 text-sm",
        sm: "h-7 px-2 text-sm",
        xs: "h-5 px-1 text-xs",
      },
      iconOnly: {
        true: "px-0",
        false: "",
      },
    },
    compoundVariants: [
      { iconOnly: true, size: "md", className: "w-8 text-base" },
      { iconOnly: true, size: "sm", className: "w-7" },
      { iconOnly: true, size: "xs", className: "w-5" },
    ],
    defaultVariants: { variant: "secondary", size: "md", iconOnly: false },
  },
);

export const fieldStyle = "flex min-w-0 flex-col gap-1.5";

export const labelStyle = "w-fit text-sm text-ink";

export const descriptionStyle = "text-xs text-ink-muted";

export const errorStyle = "flex items-start gap-1 text-xs text-danger [&_svg]:mt-px [&_svg]:shrink-0";

export const inputStyle = cva(
  [
    "rounded-control border-rule bg-ground text-ink placeholder:text-ink-muted w-full min-w-0 border text-sm",
    "transition-colors duration-150 motion-reduce:transition-none",
    "data-hovered:border-ink-muted data-focus-within:border-focus data-focused:border-focus",
    "data-invalid:border-danger data-invalid:data-focused:border-danger data-invalid:data-focus-within:border-danger",
    "data-disabled:border-rule data-disabled:bg-pane data-disabled:text-ink-muted data-disabled:cursor-not-allowed",
    focusRing,
  ],
  {
    variants: {
      multiline: {
        true: "min-h-20 resize-y px-2.5 py-1.5",
        false: "h-8 px-2.5",
      },
      mono: {
        true: "font-mono",
        false: "font-sans",
      },
    },
    defaultVariants: { multiline: false, mono: false },
  },
);

export const popoverStyle =
  "min-w-(--trigger-width) rounded-control border border-rule bg-ground text-ink shadow-lg outline-hidden";

export const listBoxStyle = "max-h-80 overflow-auto p-1 outline-hidden";

export const optionStyle = [
  "flex cursor-default items-center gap-2 rounded-control px-2 py-1.5 text-sm text-ink outline-hidden select-none",
  "data-focused:bg-pane data-pressed:bg-pane data-selected:font-semibold data-disabled:text-ink-muted",
].join(" ");
