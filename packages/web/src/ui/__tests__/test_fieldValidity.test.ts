import { describe, expect, test } from "vitest";

import { fieldValidity } from "../fieldValidity";

describe("fieldValidity", () => {
  test("uses ARIA validation, so the browser shows no native messages, and sets no state without a message or an explicit state", () => {
    expect(fieldValidity(undefined, undefined)).toStrictEqual({ validationBehavior: "aria" });
  });

  test("marks the field invalid when a message exists", () => {
    expect(fieldValidity(undefined, "Enter a number")).toStrictEqual({ validationBehavior: "aria", isInvalid: true });
  });

  test("keeps an explicit state with a message", () => {
    expect(fieldValidity(false, "Enter a number")).toStrictEqual({ validationBehavior: "aria", isInvalid: false });
  });

  test("keeps an explicit state without a message", () => {
    expect(fieldValidity(true, undefined)).toStrictEqual({ validationBehavior: "aria", isInvalid: true });
    expect(fieldValidity(false, undefined)).toStrictEqual({ validationBehavior: "aria", isInvalid: false });
  });
});
