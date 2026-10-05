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
  return (
    <AriaTextField
      {...props}
      isInvalid={props.isInvalid ?? errorMessage !== undefined}
      className={cn(fieldStyle, className)}
    >
      <FieldLabel label={label} labelHidden={labelHidden} info={info} />
      {multiline ? (
        <TextArea rows={rows} placeholder={placeholder ?? ""} className={inputStyle({ multiline: true, mono })} />
      ) : (
        <Input placeholder={placeholder ?? ""} className={inputStyle({ mono })} />
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
