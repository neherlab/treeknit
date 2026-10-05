import { cn } from "cn";
import type { ReactNode } from "react";
import { FieldError, Label, Text } from "react-aria-components";
import ErrorIcon from "~icons/lucide/circle-alert";

import { InfoButton } from "./InfoButton";
import { descriptionStyle, errorStyle, labelStyle } from "./styles";

export function FieldLabel({ label, labelHidden = false, info }: FieldLabelProps) {
  const text = <Label className={cn(labelStyle, labelHidden && "sr-only")}>{label}</Label>;

  if (info === undefined) {
    return text;
  }

  return (
    <div className="flex items-center gap-1">
      {text}
      <InfoButton topic={label}>{info}</InfoButton>
    </div>
  );
}

export function FieldDescription({ children }: { children: ReactNode }) {
  if (children === undefined || children === null) {
    return null;
  }

  return (
    <Text slot="description" className={descriptionStyle}>
      {children}
    </Text>
  );
}

export function FieldErrorMessage({ children }: { children: string | undefined }) {
  return (
    <FieldError className={errorStyle}>
      {({ validationErrors }) => (
        <>
          <ErrorIcon aria-hidden />
          <span>{children ?? validationErrors.join(" ")}</span>
        </>
      )}
    </FieldError>
  );
}

export interface FieldLabelProps {
  label: string;
  labelHidden?: boolean | undefined;
  info?: ReactNode;
}

export interface FieldProps extends FieldLabelProps {
  description?: ReactNode;
  errorMessage?: string;
}
