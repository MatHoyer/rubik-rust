import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  // Pre-bundling would break the package's `new URL("...wasm", import.meta.url)`.
  optimizeDeps: { exclude: ["rubik-wasm"] },
  worker: { format: "es" },
  // rubik-wasm is a link to ../wasm/pkg, outside this folder.
  server: { fs: { allow: [".", "../wasm/pkg"] } },
  // Relative asset paths: dist/ works from any folder or sub-path.
  base: "./",
  // three.js alone is ~700 kB; one chunk is fine for a single-page app.
  build: { chunkSizeWarningLimit: 1500 },
});
