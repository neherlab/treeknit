import { describe, expect, test } from "vitest";

import { fieldValidity } from "../fieldValidity";

describe("fieldValidity", () => {
  test("leaves validation to React Aria when there is no message and no explicit state", () => {
    expect(fieldValidity(undefined, undefined)).toStrictEqual({});
  });

  test("marks the field invalid when a message exists", () => {
    expect(fieldValidity(undefined, "Enter a number")).toStrictEqual({ isInvalid: true });
  });

  test("keeps an explicit state with a message", () => {
    expect(fieldValidity(false, "Enter a number")).toStrictEqual({ isInvalid: false });
  });

  test("keeps an explicit state without a message", () => {
    expect(fieldValidity(true, undefined)).toStrictEqual({ isInvalid: true });
    expect(fieldValidity(false, undefined)).toStrictEqual({ isInvalid: false });
  });
});
