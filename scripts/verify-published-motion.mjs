import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const origin = 'https://luxmc-r92.pages.dev';
const hashes = [];
for (const file of ['revision.css', 'cinematic.js']) {
    const local = await readFile(`website/${file}`);
    const response = await fetch(`${origin}/${file}?v=20261009-polish`, { signal: AbortSignal.timeout(20000) });
    assert.equal(response.status, 200);
    const remote = Buffer.from(await response.arrayBuffer());
    const hash = bytes => createHash('sha256').update(bytes).digest('hex');
    assert.equal(hash(remote), hash(local), file);
    hashes.push({ file, sha256: hash(remote), bytes: remote.length });
}
await writeFile('docs/validation/v3.6/layout-polish/published-site.json', JSON.stringify({ passed: true, origin, deployment: '90c2e775', assetsMatchLocalBuild: true, githubMutations: false, hashes }, null, 2));
console.log('Published animation assets match the local build exactly.');
