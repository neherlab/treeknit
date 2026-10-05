import { cn } from "cn";
import {
  Input,
  TextArea,
  TextField as AriaTextField,
  type TextFieldProps as AriaTextFieldProps,
} from "react-aria-components";

import { FieldDescription, FieldErrorMessage, FieldLabel, type FieldProps } from "./Field";
import { fieldStyle, inputStyle } from "./styles";

const DEFAULT_ROWS = 4;

const NO_TEXT_CORRECTION = { spellCheck: false, autoCorrect: "off", autoCapitalize: "off" } as const;

export function TextField({
  label,
  labelHidden,
  info,
  description,
  errorMessage,
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
      isInvalid={props.isInvalid ?? errorMessage !== undefined}
      className={cn(fieldStyle, className)}
    >
      <FieldLabel label={label} labelHidden={labelHidden} info={info} />
      {multiline ? (
        <TextArea
          {...correction}
          rows={rows}
          placeholder={placeholder ?? ""}
          className={inputStyle({ multiline: true, mono })}
        />
      ) : (
        <Input {...correction} placeholder={placeholder ?? ""} className={inputStyle({ mono })} />
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
