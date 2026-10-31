import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  // Pre-bundle the lazily imported 3D engine so dev mode serves it as a single
  // optimized chunk instead of a waterfall of module requests.
  optimizeDeps: {
    include: ["skinview3d", "three"],
  },
  build: {
    target: "es2022",
  },
});
