import test from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";

function fixture(run) {
  const directory = mkdtempSync(resolve(tmpdir(), "luxmc-release-test-"));
  const names = ["Lux MC Launcher.exe", "Luxmc_3.0.0_amd64.AppImage", "Luxmc_3.0.0_universal.dmg"];
  const bytes = Buffer.from("Release test fixture");
  const release = { tag_name: "v3.0.0", body: "Release notes", assets: names.map(name => ({ name, size: bytes.length, browser_download_url: `https://github.com/predabr/luxmc/releases/download/v3.0.0/${encodeURIComponent(name)}` })) };
  for (const name of names) writeFileSync(resolve(directory, name), bytes);
  try { run({ directory, release, bytes }); }
  finally { rmSync(directory, { recursive: true, force: true }); }
}

function generate(directory, release) {
  const metadata = resolve(directory, "release.json");
  writeFileSync(metadata, JSON.stringify(release));
  return spawnSync(process.execPath, ["scripts/build-release-manifest.mjs", metadata, directory], { encoding: "utf8" });
}

test("release manifest preserves the Windows filename and checksums the downloaded installers", () => fixture(({ directory, release, bytes }) => {
  assert.equal(generate(directory, release).status, 0);
  const manifest = JSON.parse(readFileSync(resolve(directory, "latest.json"), "utf8"));
  assert.equal(manifest.version, "3.0.0");
  assert.equal(manifest.platforms["windows-x86_64"].url, release.assets[0].browser_download_url);
  assert.equal(manifest.platforms["windows-x86_64"].sha256, createHash("sha256").update(bytes).digest("hex"));
  assert.equal(manifest.platforms["darwin-aarch64"].url, manifest.platforms["darwin-x86_64"].url);
  assert.match(readFileSync(resolve(directory, "SHA256SUMS"), "utf8"), /  Lux MC Launcher\.exe\n/);
}));

test("draft assets use the final public tag and GitHub-normalized Windows filename", () => fixture(({ directory, release }) => {
  release.draft = true;
  renameSync(resolve(directory, release.assets[0].name), resolve(directory, "Lux.MC.Launcher.exe"));
  release.assets[0].name = "Lux.MC.Launcher.exe";
  for (const asset of release.assets) asset.browser_download_url = `https://github.com/predabr/luxmc/releases/download/untagged-123abc/${encodeURIComponent(asset.name)}`;
  assert.equal(generate(directory, release).status, 0);
  const manifest = JSON.parse(readFileSync(resolve(directory, "latest.json"), "utf8"));
  assert.equal(manifest.platforms["windows-x86_64"].url, "https://github.com/predabr/luxmc/releases/download/v3.0.0/Lux.MC.Launcher.exe");
}));

for (const [name, change] of [
  ["missing platform", release => release.assets.pop()],
  ["truncated installer", release => release.assets[0].size++],
  ["unofficial download host", release => release.assets[0].browser_download_url = "https://example.com/installer.exe"],
  ["path traversal", release => release.assets[0].name = "../installer.exe"],
  ["unpublished URL on public release", release => release.assets[0].browser_download_url = "https://github.com/predabr/luxmc/releases/download/untagged-123abc/Lux%20MC%20Launcher.exe"],
  ["mismatched asset filename", release => release.assets[0].browser_download_url = "https://github.com/predabr/luxmc/releases/download/v3.0.0/another.exe"],
  ["invalid release tag", release => release.tag_name = "../another"],
]) {
  test(`release publication rejects ${name}`, () => fixture(({ directory, release }) => {
    change(release);
    assert.notEqual(generate(directory, release).status, 0);
  }));
}
