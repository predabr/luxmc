import assert from 'node:assert/strict';
import { createHash, createPublicKey, verify } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';

const [metadataFile, outputFile] = process.argv.slice(2);
assert.ok(metadataFile && outputFile, 'Pass release metadata and a report path.');
const version = JSON.parse(readFileSync(new URL('../package.json', import.meta.url), 'utf8')).version;
const release = JSON.parse(readFileSync(metadataFile, 'utf8'));
const compatibilityVersion = version.replace(/\d+$/, value => String(Number(value) + 1));
assert.ok(release.tag_name === `v${version}` || new RegExp(`^v${compatibilityVersion.replaceAll('.', '\\.')}-revision\\.\\d+$`).test(release.tag_name));
assert.equal(release.draft, false);
assert.equal(release.prerelease, false);
const names = ['Lux.MC.Launcher.exe', `Luxmc_${version}_x64-setup.exe`, `Luxmc_${version}_amd64.AppImage`, `Luxmc_${version}_amd64.deb`, `Luxmc-${version}-1.x86_64.rpm`, `luxmc-${version}-1-x86_64.pkg.tar.zst`, `Luxmc_${version}_universal.dmg`, 'latest.json', 'SHA256SUMS', 'latest.json.sig', 'SHA256SUMS.sig'];
const assets = new Map(release.assets.map(asset => [asset.name, asset]));
const canonical = assets.has('Lux MC Launcher.exe') ? 'Lux MC Launcher.exe' : names[0];
names[0] = canonical;
const downloadChecks = [];
for (const name of names) {
    const asset = assets.get(name);
    assert.ok(asset?.size > 0, name);
    const response = await fetch(asset.browser_download_url, { method: 'HEAD', signal: AbortSignal.timeout(30000) });
    assert.equal(response.status, 200, name);
    downloadChecks.push({ name, size: asset.size, status: response.status, digest: asset.digest });
}
const key = createPublicKey(readFileSync(new URL('../packaging/security/release-public-key.pem', import.meta.url)));
const signedFiles = new Map();
for (const name of ['latest.json', 'SHA256SUMS']) {
    const response = await fetch(assets.get(name).browser_download_url, { signal: AbortSignal.timeout(30000) });
    const signatureResponse = await fetch(assets.get(`${name}.sig`).browser_download_url, { signal: AbortSignal.timeout(30000) });
    assert.equal(response.status, 200);
    assert.equal(signatureResponse.status, 200);
    const bytes = Buffer.from(await response.arrayBuffer());
    const signature = Buffer.from((await signatureResponse.text()).trim(), 'base64');
    assert.equal(verify(null, bytes, key, signature), true, `${name} official signature`);
    signedFiles.set(name, bytes);
}
const manifest = JSON.parse(signedFiles.get('latest.json').toString('utf8'));
assert.equal(manifest.displayVersion || manifest.version, version);
if (release.tag_name.includes('-revision.')) assert.equal(manifest.revision, Number(release.tag_name.split('-revision.')[1]));
for (const [platform, filename] of Object.entries({ 'windows-x86_64': canonical, 'linux-x86_64': names[2], 'darwin-x86_64': names[6], 'darwin-aarch64': names[6] })) {
    const asset = assets.get(filename);
    assert.equal(manifest.platforms[platform].url, asset.browser_download_url);
    assert.equal(manifest.platforms[platform].size, asset.size);
    assert.match(manifest.platforms[platform].sha256, /^[a-f0-9]{64}$/);
    if (asset.digest) assert.equal(`sha256:${manifest.platforms[platform].sha256}`, asset.digest);
}
const origin = 'https://luxmc-r92.pages.dev';
const site = await fetch(`${origin}/?validation=${Date.now()}`, { signal: AbortSignal.timeout(30000) });
assert.equal(site.status, 200);
assert.ok((await site.text()).includes(version), 'Website version');
const releaseResponse = await fetch(`${origin}/api/latest-release?validation=${Date.now()}`, { signal: AbortSignal.timeout(30000) });
assert.equal(releaseResponse.status, 200);
assert.equal((await releaseResponse.json()).tag_name, release.tag_name);
const siteChecks = [];
for (const [platform, filename] of Object.entries({ windows: canonical, linux: names[2], deb: names[3], rpm: names[4], arch: names[5], macos: names[6] })) {
    const response = await fetch(`${origin}/download/${platform}?validation=${Date.now()}`, { method: 'HEAD', redirect: 'manual', signal: AbortSignal.timeout(30000) });
    assert.equal(response.status, platform === 'windows' ? 200 : 302, platform);
    if (platform === 'windows') {
        assert.equal(response.headers.get('Content-Disposition'), 'attachment; filename="Lux MC Launcher.exe"');
        const advertisedLength = response.headers.get('Content-Length');
        if (advertisedLength !== null) assert.equal(Number(advertisedLength), assets.get(filename).size);
    } else assert.equal(response.headers.get('location'), assets.get(filename).browser_download_url, platform);
    siteChecks.push({ platform, filename, status: response.status });
}
const installer = await fetch(`${origin}/download/windows?validation=${Date.now()}`, { signal: AbortSignal.timeout(60000) });
assert.equal(installer.status, 200);
const bytes = Buffer.from(await installer.arrayBuffer());
assert.equal(bytes.length, assets.get(canonical).size);
const hash = createHash('sha256').update(bytes).digest('hex');
assert.equal(hash, manifest.platforms['windows-x86_64'].sha256);
assert.ok(signedFiles.get('SHA256SUMS').toString('utf8').split('\n').includes(`${hash}  ${canonical}`));
const report = { version, releaseUrl: release.html_url, publishedAt: release.published_at, downloadChecks, siteChecks, manifestVerified: true, officialSignaturesVerified: true, websiteInstallerSha256: hash };
writeFileSync(outputFile, JSON.stringify(report, null, 2));
console.log(JSON.stringify(report));
