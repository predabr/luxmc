import { createHash, createPublicKey, verify } from 'node:crypto';
import { readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { githubRequest } from './github-authenticated.mjs';
import assert from 'node:assert/strict';

const tag = process.argv[2];
if (!/^v\d+\.\d+\.\d+(?:-revision\.\d+)?$/.test(tag ?? '')) throw new Error('Informe a tag oficial.');
const release = await githubRequest(`repos/predabr/luxmc/releases/tags/${tag}`);
assert.equal(release.draft, false, 'A publicação precisa estar concluída.');
const assets = new Map(release.assets.map(asset => [asset.name, asset]));
const get = async name => {
    const asset = assets.get(name); assert.ok(asset, `Arquivo ausente: ${name}`);
    const response = await fetch(asset.browser_download_url, { signal: AbortSignal.timeout(120000) });
    assert.equal(response.status, 200, name);
    const bytes = Buffer.from(await response.arrayBuffer()); assert.equal(bytes.length, asset.size, name);
    return bytes;
};
const key = createPublicKey(readFileSync('packaging/security/release-public-key.pem'));
const metadata = new Map();
for (const name of ['latest.json', 'SHA256SUMS']) {
    const bytes = await get(name); const signature = await get(`${name}.sig`);
    assert.equal(verify(null, bytes, key, Buffer.from(signature.toString().trim(), 'base64')), true, `${name}: assinatura oficial`);
    metadata.set(name, bytes);
}
const manifest = JSON.parse(metadata.get('latest.json').toString());
const platform = manifest.platforms['windows-x86_64'];
const asset = [...assets.values()].find(item => item.browser_download_url === platform.url);
assert.ok(asset);
const bytes = await get(asset.name);
const hash = createHash('sha256').update(bytes).digest('hex');
assert.equal(hash, platform.sha256); assert.equal(bytes.length, platform.size);
assert.ok(metadata.get('SHA256SUMS').toString().split('\n').includes(`${hash}  ${asset.name}`));
assert.equal(bytes.subarray(0, 2).toString(), 'MZ');
mkdirSync('release-windows', { recursive: true }); mkdirSync('website/releases', { recursive: true });
for (const file of ['release-windows/Lux MC Launcher.exe', `release-windows/Luxmc_${manifest.displayVersion || manifest.version}_x64-setup.exe`, `release-windows/Luxmc_${manifest.displayVersion || manifest.version}_revision.${manifest.revision || 0}_x64-setup.exe`, 'website/releases/Lux MC Launcher.exe']) writeFileSync(file, bytes);
const result = { tag, version: manifest.displayVersion || manifest.version, revision: manifest.revision || 0, officialSignaturesVerified: true, bytes: bytes.length, sha256: hash };
writeFileSync('docs/validation/v3.6/hotfix/official-installer.json', JSON.stringify(result, null, 2));
console.log(JSON.stringify(result));
