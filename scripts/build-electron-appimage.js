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

// 3. Prepare bin/luxmc sidecar and C# Luxmc.Launcher
console.log("3. Preparing native binaries (Rust/C++ Daemon and C# Launch Engine)...");
const binDir = path.join(root, "dist-electron/bin");
if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir, { recursive: true });
}

// 3a. Compile C# launcher
console.log("   Compiling C# (.NET 8) Luxmc.Launcher...");
const dotnetCmd = fs.existsSync(path.join(process.env.HOME || "", ".dotnet/dotnet"))
  ? path.join(process.env.HOME || "", ".dotnet/dotnet")
  : "dotnet";

try {
  execSync(
    `${dotnetCmd} publish src-csharp/Luxmc.Launcher/Luxmc.Launcher.csproj -c Release -r linux-x64 --self-contained -p:PublishSingleFile=true`,
    { cwd: root, stdio: "inherit" }
  );
  const csharpBinary = path.join(root, "src-csharp/Luxmc.Launcher/bin/Release/net8.0/linux-x64/publish/Luxmc.Launcher");
  if (fs.existsSync(csharpBinary)) {
    fs.copyFileSync(csharpBinary, path.join(binDir, "Luxmc.Launcher"));
    fs.chmodSync(path.join(binDir, "Luxmc.Launcher"), 0o755);
    console.log("   ✓ Bundled C# Luxmc.Launcher to dist-electron/bin/Luxmc.Launcher");
  }
} catch (e) {
  console.warn("   ⚠ Failed to build C# launcher:", e.message);
}

// 3b. Copy Rust/C++ daemon
const releaseBinary = path.join(root, "src-tauri/target/release/luxmc");
const debugBinary = path.join(root, "src-tauri/target/debug/luxmc");

if (fs.existsSync(releaseBinary)) {
  fs.copyFileSync(releaseBinary, path.join(binDir, "luxmc"));
  fs.chmodSync(path.join(binDir, "luxmc"), 0o755);
  console.log("   ✓ Copied release luxmc binary to dist-electron/bin/luxmc");
} else if (fs.existsSync(debugBinary)) {
  fs.copyFileSync(debugBinary, path.join(binDir, "luxmc"));
  fs.chmodSync(path.join(binDir, "luxmc"), 0o755);
  console.log("   ✓ Copied debug luxmc binary to dist-electron/bin/luxmc");
} else {
  console.warn("   ⚠ No pre-compiled luxmc binary found; bundling standalone UI");
}

// 4. Run electron-builder
console.log("4. Packaging with electron-builder...");
execSync("npx electron-builder --linux AppImage", { cwd: root, stdio: "inherit" });

console.log("✓ Done! Packaged AppImage is ready in release-electron/");
