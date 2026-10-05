import { requireSuppressionReasonRule } from "../rules/require-suppression-reason.ts";
import { ruleTester } from "./rule-tester.ts";

const tester = ruleTester("ts");

const error = { messageId: "missingReason" };

tester.run("treeknit/require-suppression-reason", requireSuppressionReasonRule, {
  valid: [
    "// oxlint-disable-next-line treeknit/no-vague-identifiers -- entry name is fixed\nconst utils = 1",
    "// oxlint-disable treeknit/foo -- justified\nconst x = 1\n// oxlint-enable treeknit/foo",
    "// oxlint-enable treeknit/foo\nconst x = 1",
    "// a normal comment\nconst x = 1",
    "const x = 1",
  ],
  invalid: [
    {
      code: "// oxlint-disable-next-line treeknit/no-vague-identifiers\nconst utils = 1",
      errors: [error],
    },
    { code: "// oxlint-disable-line treeknit/foo\nconst x = 1", errors: [error] },
    {
      code: "// oxlint-disable treeknit/foo\nconst x = 1\n// oxlint-enable treeknit/foo",
      errors: [error],
    },
    { code: "// oxlint-disable-next-line treeknit/foo --\nconst x = 1", errors: [error] },
  ],
});
