import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, readdir, rm, stat, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve, sep } from 'node:path';
import { writeSiteOutputs } from '../scripts/write-site-outputs.mjs';

async function fixture(run) {
    const root = await mkdtemp(join(tmpdir(), 'luxmc-site-output-'));
    try { await run(root); }
    finally {
        assert.ok(resolve(root).startsWith(resolve(tmpdir()) + sep) && root.includes('luxmc-site-output-'));
        await rm(root, { recursive: true, force: true });
    }
}

test('unchanged generated files retain their modification time and identity', () => fixture(async root => {
    const path = join(root, 'app.js');
    const contents = Buffer.from('export const value = 1;');
    await writeFile(path, contents);
    const before = await stat(path, { bigint: true });
    await writeSiteOutputs([{ path, contents }], root);
    const after = await stat(path, { bigint: true });
    assert.equal(after.mtimeNs, before.mtimeNs);
    assert.equal(after.ino, before.ino);
}));

test('changed generated files are replaced without leaving temporary files', () => fixture(async root => {
    const path = join(root, 'app.js');
    await writeFile(path, 'old');
    await writeSiteOutputs([{ path, contents: Buffer.from('new') }], root);
    assert.equal(await readFile(path, 'utf8'), 'new');
    assert.deepEqual(await readdir(root), ['app.js']);
}));

test('generated nested directories are created and escaping output paths are rejected', () => fixture(async root => {
    const path = join(root, 'visuals', 'motion.js');
    await writeSiteOutputs([{ path, contents: Buffer.from('motion') }], root);
    assert.equal(await readFile(path, 'utf8'), 'motion');
    await assert.rejects(writeSiteOutputs([{ path: join(root, '..', 'outside.js'), contents: Buffer.from('outside') }], root), /outside the website directory/);
}));
