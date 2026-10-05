export function fieldValidity(isInvalid: boolean | undefined, errorMessage: string | undefined): FieldValidity {
  if (errorMessage !== undefined) {
    return { isInvalid: isInvalid ?? true };
  }

  return isInvalid === undefined ? {} : { isInvalid };
}

interface FieldValidity {
  isInvalid?: boolean;
}
