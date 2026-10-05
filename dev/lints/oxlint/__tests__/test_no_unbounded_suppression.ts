import { noUnboundedSuppressionRule } from "../rules/no-unbounded-suppression.ts";
import { ruleTester } from "./rule-tester.ts";

const tester = ruleTester("ts");

const error = { messageId: "unbounded" };

tester.run("treeknit/no-unbounded-suppression", noUnboundedSuppressionRule, {
  valid: [
    "// oxlint-disable treeknit/foo -- reason\nconst x = 1\n// oxlint-enable treeknit/foo",
    "// oxlint-disable-next-line treeknit/foo -- reason\nconst x = 1",
    "// oxlint-disable-line treeknit/foo -- reason\nconst x = 1",
    "// a normal comment\nconst x = 1",
    "const x = 1",
  ],
  invalid: [
    { code: "// oxlint-disable treeknit/foo -- reason\nconst x = 1", errors: [error] },
    {
      code: "// oxlint-enable treeknit/foo\n// oxlint-disable treeknit/foo -- reason\nconst x = 1",
      errors: [error],
    },
  ],
});
