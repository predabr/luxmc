import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { basename, resolve } from "node:path";

const [metadataFile, directory = "."] = process.argv.slice(2);
if (!metadataFile) throw new Error("Pass the GitHub release metadata JSON file.");
const release = JSON.parse(readFileSync(metadataFile, "utf8"));
if (!/^v\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?$/.test(release.tag_name)) throw new Error("Invalid release tag.");
const revisionMatch = /-revision\.(\d+)$/.exec(release.tag_name);
const revision = revisionMatch ? Number(revisionMatch[1]) : 0;
if (!Number.isSafeInteger(revision) || revision < 0) throw new Error("Invalid release revision.");
const transportVersion = release.tag_name.replace(/^v/, "").replace(/-revision\.\d+$/, "");
if (revision && Number(transportVersion.split('.').at(-1)) < 1) throw new Error("Invalid compatibility patch.");
const version = revision ? transportVersion.replace(/\d+$/, value => String(Number(value) - 1)) : transportVersion;
const files = {
  "windows-x86_64": release.assets.some(asset => asset.name === "Lux MC Launcher.exe") ? "Lux MC Launcher.exe" : "Lux.MC.Launcher.exe",
  "linux-x86_64": `Luxmc_${version}_amd64.AppImage`,
  "darwin-x86_64": `Luxmc_${version}_universal.dmg`,
  "darwin-aarch64": `Luxmc_${version}_universal.dmg`,
};
const platforms = {};
const sums = new Map();
for (const asset of release.assets) {
  if (!/\.(?:exe|AppImage|deb|rpm|dmg|pkg\.tar\.zst)$/.test(asset.name)) continue;
  if (basename(asset.name) !== asset.name) throw new Error("Invalid release asset name.");
  const bytes = readFileSync(resolve(directory, asset.name));
  if (!bytes.length || bytes.length !== asset.size) throw new Error(`Release asset size mismatch: ${asset.name}`);
  const url = new URL(asset.browser_download_url);
  const prefix = `/predabr/luxmc/releases/download/${release.tag_name}/`;
  const draftPrefix = release.draft === true && /^\/predabr\/luxmc\/releases\/download\/untagged-[a-f0-9]+\//.exec(url.pathname)?.[0];
  const allowedPrefix = url.pathname.startsWith(prefix) ? prefix : draftPrefix;
  if (url.origin !== "https://github.com" || url.username || url.password || !allowedPrefix || decodeURIComponent(url.pathname.slice(allowedPrefix.length)) !== asset.name) throw new Error("Unofficial release URL.");
  const publicUrl = `https://github.com${prefix}${encodeURIComponent(asset.name)}`;
  const sha256 = createHash("sha256").update(bytes).digest("hex");
  sums.set(asset.name, sha256);
  for (const [platform, filename] of Object.entries(files)) {
    if (filename === asset.name) platforms[platform] = { url: publicUrl, sha256, size: bytes.length };
  }
}
if (Object.keys(platforms).length !== Object.keys(files).length) throw new Error("Missing supported platform installer.");
writeFileSync(resolve(directory, "SHA256SUMS"), [...sums].sort(([a], [b]) => a.localeCompare(b)).map(([name, hash]) => `${hash}  ${name}`).join("\n") + "\n");
const legacyVersion = transportVersion;
writeFileSync(resolve(directory, "latest.json"), JSON.stringify({ version: legacyVersion, ...(revision ? { displayVersion: version, revision } : {}), notes: release.body, pub_date: release.published_at || new Date().toISOString(), platforms }, null, 2) + "\n");
console.log(`Verified ${sums.size} installers; update manifest created for ${version}.`);
