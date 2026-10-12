import { fileURLToPath } from "node:url";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { svelteTesting } from "@testing-library/svelte/vite";
import { defineConfig } from "vitest/config";

export default defineConfig({
  resolve: { conditions: ["browser"], alias: { $lib: fileURLToPath(new URL("./src/lib", import.meta.url)), "$app/navigation": fileURLToPath(new URL("./node_modules/@sveltejs/kit/src/runtime/app/navigation.js", import.meta.url)) } },
  plugins: [svelte({ hot: !process.env.VITEST }), svelteTesting()],
  test: {
    include: ["src/**/*.{test,spec}.{js,ts}"],
    environment: "jsdom",
    maxWorkers: 4,
    globals: true,
    setupFiles: ["./src/tests/setup.ts"]
  }
});
