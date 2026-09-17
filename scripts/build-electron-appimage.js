import { execSync } from "child_process";
import * as fs from "fs";
import * as path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(__dirname, "..");

console.log("=== Building Luxmc Electron AppImage ===");

// 1. Build frontend
console.log("1. Building SvelteKit frontend...");
execSync("pnpm build", { cwd: root, stdio: "inherit" });

// 2. Compile Electron TypeScript
console.log("2. Compiling Electron TypeScript...");
execSync("node electron/build.js", { cwd: root, stdio: "inherit" });

// 3. Prepare bin/luxmc sidecar
console.log("3. Preparing native sidecar binary...");
const binDir = path.join(root, "dist-electron/bin");
if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir, { recursive: true });
}

const releaseBinary = path.join(root, "src-tauri/target/release/luxmc");
const debugBinary = path.join(root, "src-tauri/target/debug/luxmc");

if (fs.existsSync(releaseBinary)) {
  fs.copyFileSync(releaseBinary, path.join(binDir, "luxmc"));
  fs.chmodSync(path.join(binDir, "luxmc"), 0o755);
  console.log("✓ Copied release luxmc binary to dist-electron/bin/luxmc");
} else if (fs.existsSync(debugBinary)) {
  fs.copyFileSync(debugBinary, path.join(binDir, "luxmc"));
  fs.chmodSync(path.join(binDir, "luxmc"), 0o755);
  console.log("✓ Copied debug luxmc binary to dist-electron/bin/luxmc");
} else {
  console.warn("⚠ No pre-compiled luxmc binary found; bundling standalone UI");
}

// 4. Run electron-builder
console.log("4. Packaging with electron-builder...");
execSync("npx electron-builder --linux AppImage", { cwd: root, stdio: "inherit" });

console.log("✓ Done! Packaged AppImage is ready in release-electron/");
