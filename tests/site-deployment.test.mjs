import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { onRequest as social } from '../website/functions/api/social/[action].js';
import { onRequest as legacySocial } from '../functions/api/social/[action].js';
import { onRequest as account } from '../website/functions/api/account/[action].js';
import { onRequest as legacyAccount } from '../functions/api/account/[action].js';

test('deployment discovers the canonical Pages functions regardless of the caller directory', async () => {
    const directory = await mkdtemp(join(tmpdir(), 'luxmc-deploy-test-'));
    try {
        const tool = join(directory, 'wrangler.mjs');
        await writeFile(tool, "console.log(JSON.stringify({cwd:process.cwd(),args:process.argv.slice(2)}));");
        const result = spawnSync(process.execPath, [fileURLToPath(new URL('../scripts/deploy-website.mjs', import.meta.url))], { cwd: directory, env: { ...process.env, LUXMC_WRANGLER_BINARY: tool }, encoding: 'utf8', windowsHide: true });
        assert.equal(result.status, 0, result.stderr);
        const command = JSON.parse(result.stdout);
        assert.equal(command.cwd, fileURLToPath(new URL('../website/', import.meta.url)).replace(/[\\/]$/, ''));
        assert.deepEqual(command.args, ['pages', 'deploy', '.', '--project-name=luxmc', '--branch=main', '--commit-dirty=true']);
    } finally { await rm(directory, { recursive: true, force: true }); }
});

test('legacy social entrypoint retains all current actions and their authentication boundary', async () => {
    assert.equal(legacySocial, social);
    assert.equal(legacyAccount, account);
    for (const handler of [social, legacySocial]) {
        for (const action of ['register', 'sync', 'search', 'invite', 'accept', 'remove', 'block', 'unblock', 'room_create', 'room_join', 'room_close', 'stream_ticket', 'profile_get', 'profile_save', 'owner_users', 'owner_tags', 'owner_delete']) {
            const request = new Request(`https://luxmc.test/api/social/${action}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: '{}' });
            const response = await handler({ request, env: { SOCIAL_DB: {} }, params: { action }, waitUntil() {} });
            assert.equal(response.status, 401, action);
            assert.equal((await response.json()).error, 'Authentication required', action);
        }
        const invalid = await handler({ request: new Request('https://luxmc.test/api/social/invalid', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: '{}' }), env: { SOCIAL_DB: {} }, params: { action: 'invalid' }, waitUntil() {} });
        assert.equal(invalid.status, 404);
        const foreign = await handler({ request: new Request('https://luxmc.test/api/social/profile_get', { method: 'POST', headers: { 'Content-Type': 'application/json', Origin: 'https://foreign.test' }, body: '{}' }), env: { SOCIAL_DB: {} }, params: { action: 'profile_get' }, waitUntil() {} });
        assert.equal(foreign.status, 403);
    }
});
