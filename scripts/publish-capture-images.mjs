import { githubRequest } from './github-authenticated.mjs';
import { readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const names = ['home', 'mods', 'instances', 'friends', 'personalizer'];
const manifest = JSON.parse(readFileSync('docs/validation/v3.6/upgrades/captures.json', 'utf8'));
assert.equal(manifest.windowOnly, true);
const repo = 'repos/predabr/luxmc';
const ref = await githubRequest(`${repo}/git/ref/heads/main`);
const base = await githubRequest(`${repo}/git/commits/${ref.object.sha}`);
const tree = [];
for (const name of names) {
    const path = `website/assets/captures-3.6/${name}.png`;
    const bytes = readFileSync(path);
    const capture = manifest.images.find(image => image.path === path);
    assert.ok(capture);
    assert.equal(createHash('sha256').update(bytes).digest('hex'), capture.sha256);
    assert.equal(bytes.readUInt32BE(16), capture.width);
    assert.equal(bytes.readUInt32BE(20), capture.height);
    const blob = await githubRequest(`${repo}/git/blobs`, { method: 'POST', body: { content: bytes.toString('base64'), encoding: 'base64' } });
    tree.push({ path, mode: '100644', type: 'blob', sha: blob.sha });
}
const createdTree = await githubRequest(`${repo}/git/trees`, { method: 'POST', body: { base_tree: base.tree.sha, tree } });
const commit = await githubRequest(`${repo}/git/commits`, { method: 'POST', body: { message: 'Atualiza capturas do Luxmc sem a barra de tarefas', tree: createdTree.sha, parents: [ref.object.sha] } });
const detail = await githubRequest(`${repo}/commits/${commit.sha}`);
assert.deepEqual(detail.files.map(file => file.filename).sort(), tree.map(file => file.path).sort());
await githubRequest(`${repo}/git/refs/heads/main`, { method: 'PATCH', body: { sha: commit.sha, force: false } });
for (const entry of tree) {
    const blob = await githubRequest(`${repo}/git/blobs/${entry.sha}`);
    assert.equal(Buffer.from(blob.content.replaceAll('\n', ''), 'base64').equals(readFileSync(entry.path)), true);
}
const report = { commit: commit.sha, url: commit.html_url, imageFilesOnly: true, files: tree.map(file => file.path) };
writeFileSync('docs/validation/v3.6/upgrades/image-publication.json', JSON.stringify(report, null, 2));
console.log(JSON.stringify(report));
