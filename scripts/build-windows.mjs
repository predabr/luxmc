import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

if (process.platform !== "win32") throw new Error("Este comando gera o instalador no Windows.");
const rustDirectory = join(homedir(), ".cargo", "bin");
if (!process.env.CARGO_TARGET_DIR && process.env.LOCALAPPDATA) process.env.CARGO_TARGET_DIR = join(process.env.LOCALAPPDATA, "Luxmc-build", "target");
if (existsSync(join(rustDirectory, "cargo.exe"))) process.env.PATH = `${rustDirectory};${process.env.PATH ?? ""}`;
const cargo = spawnSync("cargo", ["--version"], { encoding: "utf8", windowsHide: true });
if (cargo.status !== 0) throw new Error("Instale Rust com a toolchain MSVC e os Visual Studio Build Tools com C++ antes de compilar.");
const packageManager = process.env.npm_execpath;
if (!packageManager) throw new Error("Execute pnpm build:windows.");
if (!existsSync("packaging/windows/sidebar.bmp") || !existsSync("packaging/windows/header.bmp")) throw new Error("Gere a identidade do instalador com python scripts/build-installer-art.py antes de compilar.");
const result = spawnSync(process.execPath, [packageManager, "tauri", "build", "--bundles", "nsis"], { stdio: "inherit", windowsHide: true });
if (result.error) throw result.error;
if (result.status === 0) {
    const output = join(process.cwd(), "release-windows");
    const config = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));
    const architecture = process.arch === "arm64" ? "arm64" : process.arch === "ia32" ? "x86" : "x64";
    const setup = join(process.env.CARGO_TARGET_DIR, "release", "bundle", "nsis", `${config.productName}_${config.version}_${architecture}-setup.exe`);
    if (!existsSync(setup)) throw new Error("O instalador NSIS não foi encontrado após a compilação.");
    mkdirSync(output, { recursive: true });
    const destination = join(output, "Lux MC Launcher.exe");
    copyFileSync(setup, destination);
    console.log(`Instalador: ${destination}`);
}
process.exit(result.status ?? 1);
