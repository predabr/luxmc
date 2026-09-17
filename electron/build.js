import * as esbuild from "esbuild";
import * as path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

async function buildElectron() {
  await esbuild.build({
    entryPoints: [
      path.join(__dirname, "main.ts"),
      path.join(__dirname, "preload.ts"),
    ],
    bundle: true,
    platform: "node",
    target: "node18",
    outdir: path.join(__dirname, "../dist-electron"),
    outExtension: { ".js": ".cjs" },
    external: ["electron"],
    sourcemap: true,
    format: "cjs",
  });
  console.log("✓ Electron TypeScript compiled to dist-electron/");
}

buildElectron().catch((err) => {
  console.error("Electron build failed:", err);
  process.exit(1);
});
