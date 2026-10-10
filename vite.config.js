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
export default defineConfig(({ isSsrBuild, mode }) => ({
  plugins: [sveltekit(), svelteCssGuard(), { name: "luxmc-runtime-chunks", configResolved(config) { serverBuild = Boolean(config.build.ssr); } }],

  optimizeDeps: {
    include: [
      "@tauri-apps/api/app",
      "@tauri-apps/api/core",
      "@tauri-apps/api/event",
      "@tauri-apps/api/window",
      "@tauri-apps/plugin-dialog",
      "@tauri-apps/plugin-opener",
      "@tauri-apps/plugin-store",
      "@panzoom/panzoom",
      "cmdk-svelte",
      "i18next",
      "zod",
      "bits-ui",
      "@tanstack/svelte-virtual",
      "canvas-confetti",
      "skinview3d",
      "three",
      "clsx",
      "tailwind-merge",
      "tailwind-variants",
      "runed",
      "@floating-ui/dom",
      "lucide-svelte",
      "howler",
      "colord",
      "marked",
      "gifuct-js",
      "gifenc",
      "dompurify",
      ...(mode === "browser-test" ? [
        "@codemirror/state",
        "@threlte/core",
        "@threlte/extras",
        "@vibrant/core",
        "@vibrant/generator-default",
        "@vibrant/image-browser",
        "@vibrant/quantizer-mmcq",
        "codemirror",
        "diff",
        "idb-keyval",
        "motion",
        "qrcode",
        "wavesurfer.js"
      ] : [])
    ],
    noDiscovery: mode === "browser-test",
    exclude: ["svelte-sonner"],
    holdUntilCrawlEnd: true,
  },

  ssr: { noExternal: ["@panzoom/panzoom", "wavesurfer.js", "codemirror", "@codemirror/state"] },
  clearScreen: false,
  build: {
    target: ["es2021", "chrome100", "safari14"],
    chunkSizeWarningLimit: 550,
    rollupOptions: isSsrBuild ? {} : {
      output: {
        manualChunks(id) {
          if (serverBuild) return;
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
      ignored: ["**/src-tauri/**", "**/release-windows/**", "**/release-electron/**", "**/build/**"],
    },
  },
}));
