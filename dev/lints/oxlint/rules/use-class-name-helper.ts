import { defineRule } from "@oxlint/plugins";

export const useClassNameHelperRule = defineRule({
  meta: {
    type: "suggestion",
    docs: {
      description:
        "Disallow importing `clsx` or `tailwind-merge` directly; use the `cn` package, which merges Tailwind classes.",
    },
    messages: {
      useCn: "Import `cn` from the `cn` package instead of `{{source}}`.",
    },
  },
  createOnce(context) {
    return {
      ImportDeclaration(node) {
        const source = node.source.value;

        if (source === "clsx" || source === "tailwind-merge") {
          context.report({ node, messageId: "useCn", data: { source } });
        }
      },
    };
  },
});
