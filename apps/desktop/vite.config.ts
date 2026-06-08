import react from "@vitejs/plugin-react";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

const appRoot = fileURLToPath(new URL(".", import.meta.url));

export default defineConfig({
  base: "./",
  plugins: [react()],
  clearScreen: false,
  root: appRoot,
  server: {
    port: 1420,
    strictPort: false,
  },
  test: {
    environment: "jsdom",
  },
});
