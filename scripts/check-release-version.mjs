import { readFileSync } from "node:fs";

const version = JSON.parse(readFileSync("package.json", "utf8")).version;
const tauri = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));
const cargo = readFileSync("src-tauri/Cargo.toml", "utf8").match(/^version = "([^"]+)"/m)?.[1];
const lock = readFileSync("src-tauri/Cargo.lock", "utf8").match(/name = "luxmc"\r?\nversion = "([^"]+)"/)?.[1];
if (version !== tauri.version || version !== cargo || version !== lock) throw new Error("Launcher version metadata does not match.");
if (process.argv[2] && process.argv[2] !== `v${version}`) throw new Error("Release tag does not match the launcher version.");
if (tauri.productName !== "Luxmc" || tauri.identifier !== "io.github.luxmc.Luxmc") throw new Error("The launcher identity must remain unchanged.");
for (const locale of ["pt-BR", "en", "es"]) {
  const data = JSON.parse(readFileSync(`src/lib/i18n/${locale}.json`, "utf8"));
  if (data.app.version !== `v${version}`) throw new Error(`Version translation mismatch: ${locale}`);
}
console.log(`Luxmc ${version}: consistent version metadata and unchanged product identity.`);
