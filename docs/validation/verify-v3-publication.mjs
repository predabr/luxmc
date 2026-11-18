import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';

const release = JSON.parse(readFileSync(process.argv[2], 'utf8'));
assert.equal(release.tag_name, 'v3.0.0');
assert.equal(release.draft, false);
assert.equal(release.prerelease, false);
const names = ['Lux.MC.Launcher.exe', 'Luxmc_3.0.0_x64-setup.exe', 'Luxmc_3.0.0_amd64.AppImage', 'Luxmc_3.0.0_amd64.deb', 'Luxmc-3.0.0-1.x86_64.rpm', 'luxmc-3.0.0-1-x86_64.pkg.tar.zst', 'Luxmc_3.0.0_universal.dmg', 'latest.json', 'SHA256SUMS'];
const assets = new Map(release.assets.map(asset => [asset.name, asset]));
const downloadChecks = [];
for (const name of names) {
    const asset = assets.get(name);
    assert.ok(asset?.size > 0, name);
    const response = await fetch(asset.browser_download_url, { method: 'HEAD', signal: AbortSignal.timeout(30000) });
    assert.equal(response.status, 200, name);
    downloadChecks.push({ name, size: asset.size, status: response.status, digest: asset.digest });
}
const manifestResponse = await fetch(assets.get('latest.json').browser_download_url, { signal: AbortSignal.timeout(30000) });
assert.equal(manifestResponse.status, 200);
const manifest = await manifestResponse.json();
assert.equal(manifest.version, '3.0.0');
for (const [platform, filename] of Object.entries({ 'windows-x86_64': names[0], 'linux-x86_64': names[2], 'darwin-x86_64': names[6], 'darwin-aarch64': names[6] })) {
    const asset = assets.get(filename);
    assert.equal(manifest.platforms[platform].url, asset.browser_download_url);
    assert.equal(manifest.platforms[platform].size, asset.size);
    assert.match(manifest.platforms[platform].sha256, /^[a-f0-9]{64}$/);
    if (asset.digest) assert.equal(`sha256:${manifest.platforms[platform].sha256}`, asset.digest);
}
const origin = 'https://luxmc-r92.pages.dev';
const releaseResponse = await fetch(`${origin}/api/latest-release?validation=${Date.now()}`, { signal: AbortSignal.timeout(30000) });
assert.equal(releaseResponse.status, 200);
assert.equal((await releaseResponse.json()).tag_name, 'v3.0.0');
const siteChecks = [];
for (const [platform, filename] of Object.entries({ windows: names[0], linux: names[2], deb: names[3], rpm: names[4], arch: names[5], macos: names[6] })) {
    const response = await fetch(`${origin}/download/${platform}?validation=${Date.now()}`, { method: 'HEAD', redirect: 'manual', signal: AbortSignal.timeout(30000) });
    assert.equal(response.status, platform === 'windows' ? 200 : 302, platform);
    if (platform === 'windows') {
        assert.equal(response.headers.get('Content-Disposition'), 'attachment; filename="Lux MC Launcher.exe"');
        assert.equal(Number(response.headers.get('Content-Length')), assets.get(filename).size);
    } else assert.equal(response.headers.get('location'), assets.get(filename).browser_download_url, platform);
    siteChecks.push({ platform, filename, status: response.status });
}
const report = { version: '3.0.0', releaseUrl: release.html_url, publishedAt: release.published_at, downloadChecks, siteChecks, manifestVerified: true };
writeFileSync('docs/validation/v3-publication-verified.json', JSON.stringify(report, null, 2));
console.log(JSON.stringify(report));
