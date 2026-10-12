import { readFileSync } from "node:fs";

const version = JSON.parse(readFileSync("package.json", "utf8")).version;
const tauri = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));
const cargo = readFileSync("src-tauri/Cargo.toml", "utf8").match(/^version = "([^"]+)"/m)?.[1];
const lock = readFileSync("src-tauri/Cargo.lock", "utf8").match(/name = "luxmc"\r?\nversion = "([^"]+)"/)?.[1];
if (version !== tauri.version || version !== cargo || version !== lock) throw new Error("Launcher version metadata does not match.");
const revision = Number(readFileSync("src/lib/utils/updateVersion.ts", "utf8").match(/RELEASE_REVISION = (\d+)/)?.[1] || 0);
const compatibilityVersion = version.replace(/\d+$/, value => String(Number(value) + 1));
if (process.argv[2] && ![`v${version}`, `v${compatibilityVersion}-revision.${revision}`].includes(process.argv[2])) throw new Error("Release tag does not match the launcher version.");
if (tauri.productName !== "Luxmc" || tauri.identifier !== "io.github.luxmc.Luxmc") throw new Error("The launcher identity must remain unchanged.");
if (!readFileSync('packaging/windows/hooks.nsh', 'utf8').includes(`!define MUI_WELCOMEPAGE_TITLE "Luxmc ${version}"`)) throw new Error('Windows installer welcome version mismatch.');
for (const locale of ["pt-BR", "en", "es"]) {
  const data = JSON.parse(readFileSync(`src/lib/i18n/${locale}.json`, "utf8"));
  if (data.app.version !== `v${version}`) throw new Error(`Version translation mismatch: ${locale}`);
}
const arch = readFileSync("packaging/arch/PKGBUILD", "utf8");
if (arch.match(/^pkgver=(.+)$/m)?.[1].trim() !== version || !arch.includes(`_tagver="${version}"`)) throw new Error("Arch package version mismatch.");
if (!readFileSync("website/wrangler.toml", "utf8").includes(`WINDOWS_LOCAL_VERSION = "${version}"`)) throw new Error("Website download version mismatch.");
if (!readFileSync("website/index.html", "utf8").includes(`class="live-version-tag">v${version}<`)) throw new Error("Website display version mismatch.");
console.log(`Luxmc ${version}: consistent version metadata and unchanged product identity.`);
