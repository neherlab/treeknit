import { cn } from "cn";
import {
  Input,
  TextArea,
  TextField as AriaTextField,
  type TextFieldProps as AriaTextFieldProps,
} from "react-aria-components";

import { FieldDescription, FieldErrorMessage, FieldLabel, type FieldProps } from "./Field";
import { fieldValidity } from "./fieldValidity";
import { fieldStyle, inputStyle } from "./styles";

const DEFAULT_ROWS = 4;

const NO_TEXT_CORRECTION = { spellCheck: false, autoCorrect: "off", autoCapitalize: "off" } as const;

export function TextField({
  label,
  labelHidden,
  info,
  description,
  errorMessage,
  isInvalid,
  multiline = false,
  mono = false,
  rows = DEFAULT_ROWS,
  placeholder,
  className,
  ...props
}: TextFieldProps) {
  const correction = mono ? NO_TEXT_CORRECTION : undefined;

  return (
    <AriaTextField
      {...props}
      {...fieldValidity(isInvalid, errorMessage)}
      className={cn(fieldStyle, className)}
    >
      <FieldLabel label={label} labelHidden={labelHidden} info={info} />
      {multiline ? (
        <TextArea
          {...correction}
          rows={rows}
          {...(placeholder === undefined ? undefined : { placeholder })}
          className={inputStyle({ multiline: true, mono })}
        />
      ) : (
        <Input
          {...correction}
          {...(placeholder === undefined ? undefined : { placeholder })}
          className={inputStyle({ mono })}
        />
      )}
      <FieldDescription>{description}</FieldDescription>
      <FieldErrorMessage>{errorMessage}</FieldErrorMessage>
    </AriaTextField>
  );
}

export type TextFieldProps = Omit<AriaTextFieldProps, "className" | "children"> &
  FieldProps & {
    multiline?: boolean;
    mono?: boolean;
    rows?: number;
    placeholder?: string;
    className?: string;
  };
