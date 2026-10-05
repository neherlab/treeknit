export function fieldValidity(isInvalid: boolean | undefined, errorMessage: string | undefined): FieldValidity {
  if (errorMessage !== undefined) {
    return { validationBehavior: "aria", isInvalid: isInvalid ?? true };
  }

  return isInvalid === undefined ? { validationBehavior: "aria" } : { validationBehavior: "aria", isInvalid };
}

interface FieldValidity {
  validationBehavior: "aria";
  isInvalid?: boolean;
}
