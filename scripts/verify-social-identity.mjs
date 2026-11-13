import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { writeFileSync } from 'node:fs';
import assert from 'node:assert/strict';
const binary = process.env.LUXMC_NATIVE_BINARY;
assert.ok(binary);
const daemon = spawn(binary, ['--daemon'], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
const pending = new Map(); let sequence = 0;
createInterface({ input: daemon.stdout }).on('line', line => { let response; try { response = JSON.parse(line); } catch { return; } const request = pending.get(response.id); if (!request) return; pending.delete(response.id); clearTimeout(request.timer); response.error ? request.reject(new Error(response.error)) : request.resolve(response.result); });
const call = (command, args) => new Promise((resolve, reject) => { const id = ++sequence; const timer = setTimeout(() => { pending.delete(id); reject(new Error('Native command timeout')); }, 60000); pending.set(id, { resolve, reject, timer }); daemon.stdin.write(JSON.stringify({ id, command, args }) + '\n'); });
try {
    const initialized = await call('app_init', {});
    const account = initialized.account;
    assert.ok(account, 'Nenhuma conta local selecionada');
    const matchesRequestedOwner = account.username.toLowerCase() === 'spect3rbw';
    const accountId = account.id.startsWith('luxmc:') ? account.id : account.uuid || account.id;
    const result = await call('social_request', { accountId, request: { action: 'profile_get' } });
    const profile = result.profile;
    const report = { authenticatedProfileRetrieved: true, matchesRequestedOwner, id: profile.id, role: profile.role, hasCustomName: !!profile.displayName, hasPortrait: !!profile.portrait, portraitType: profile.portrait?.match(/^data:([^;]+)/)?.[1] ?? null, hasBanner: !!profile.banner, instanceCount: initialized.profiles?.length ?? null, temporaryTests: initialized.profiles?.filter(item => item.name === 'Teste temporário · desempenho 3.6').map(item => ({id:item.id,createdAt:item.createdAt,gameDir:item.gameDir})) };
    writeFileSync('docs/validation/v3.6/upgrades/social-native.json', JSON.stringify(report, null, 2));
    console.log(JSON.stringify({ ...report, id: undefined }));
} finally { daemon.stdin.end(); }
