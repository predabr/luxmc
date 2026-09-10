import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
// @ts-expect-error type error without @types/node package
import process from "node:process";
const host = process.env.TAURI_DEV_HOST;

/** @type {() => import('vite').Plugin} */
const svelteCssGuard = () => ({
  name: "svelte-virtual-css-guard",
  enforce: "post",
  load(id) {
    if (id.includes(".svelte") && id.includes("type=style")) {
      return { code: "", map: null };
    }
    return null;
  },
});

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [sveltekit(), svelteCssGuard()],

  optimizeDeps: {
    include: [],
    exclude: ["lucide-svelte", "svelte-sonner"],
  },

  clearScreen: false,
  build: {
    target: ["es2021", "chrome100", "safari14"],
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || "127.0.0.1",
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
          overlay: false,
        }
      : {
          overlay: false,
        },
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
