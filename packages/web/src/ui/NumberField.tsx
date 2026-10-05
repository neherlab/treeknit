import { cn } from "cn";
import {
  FieldError,
  Input,
  Label,
  NumberField as AriaNumberField,
  type NumberFieldProps as AriaNumberFieldProps,
  Text,
} from "react-aria-components";

import { descriptionClassName, errorClassName, fieldClassName, inputClassName, labelClassName } from "./fieldStyles";

export function NumberField({ label, description, inputClassName: inputWidth, ...props }: NumberFieldProps) {
  return (
    <AriaNumberField {...props} className={fieldClassName}>
      <Label className={labelClassName}>{label}</Label>
      <Input className={cn(inputClassName, inputWidth ?? "w-28")} />
      {description === undefined ? null : (
        <Text slot="description" className={descriptionClassName}>
          {description}
        </Text>
      )}
      <FieldError className={errorClassName} />
    </AriaNumberField>
  );
}

export interface NumberFieldProps extends Omit<AriaNumberFieldProps, "className" | "children"> {
  label: string;
  description?: string;
  inputClassName?: string;
}
