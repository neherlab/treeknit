import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import Icons from "unplugin-icons/vite";
import { defineConfig } from "vite";

import { contentSecurityPolicyMeta } from "./build/content-security-policy.ts";

const port = Number(process.env["TREEKNIT_WEB_PORT"] ?? "6180");

const ICON_SIZE = "1.25em";

export default defineConfig(({ mode }) => ({
  plugins: [
    contentSecurityPolicyMeta(),
    Icons({
      compiler: "jsx",
      jsx: "react",
      iconCustomizer(_collection, _icon, props) {
        props["width"] = ICON_SIZE;
        props["height"] = ICON_SIZE;
      },
    }),
    tailwindcss(),
    react(),
  ],
  base: "./",
  clearScreen: false,
  build: { target: "es2024", minify: mode === "production", sourcemap: mode !== "production" },
  worker: { format: "es" },
  server: { port, strictPort: true },
  preview: { port, strictPort: true },
}));
