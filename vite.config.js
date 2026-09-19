import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import process from "node:process";
const host = process.env.TAURI_DEV_HOST;
let serverBuild = false;

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
export default defineConfig(({ isSsrBuild }) => ({
  plugins: [sveltekit(), svelteCssGuard(), { name: "luxmc-runtime-chunks", configResolved(config) { serverBuild = Boolean(config.build.ssr); } }],

  optimizeDeps: {
    include: [],
    exclude: ["lucide-svelte", "svelte-sonner"],
  },

  ssr: { noExternal: ["@panzoom/panzoom", "wavesurfer.js", "codemirror", "@codemirror/state"] },
  clearScreen: false,
  build: {
    target: ["es2021", "chrome100", "safari14"],
    rollupOptions: isSsrBuild ? {} : {
      output: {
        manualChunks(id) {
          if (serverBuild) return;
          if (id.includes("/node_modules/three/")) return "three-runtime";
          if (/\/node_modules\/(?:@codemirror|codemirror|@lezer)\//.test(id)) return "editor-runtime";
        }
      }
    },
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
