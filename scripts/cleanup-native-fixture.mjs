import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { existsSync, readdirSync, writeFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
import assert from 'node:assert/strict';
const id = process.argv[2];
assert.match(id || '', /^[a-f0-9-]{36}$/);
const daemon = spawn(process.env.LUXMC_NATIVE_BINARY, ['--daemon'], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
const pending = new Map(); let sequence = 0;
createInterface({ input: daemon.stdout }).on('line', line => { let message; try { message = JSON.parse(line); } catch { return; } const request = pending.get(message.id); if (!request) return; pending.delete(message.id); clearTimeout(request.timer); message.error ? request.reject(new Error(message.error)) : request.resolve(message.result); });
const call = (command, args = {}) => new Promise((resolvePromise, reject) => { const requestId = ++sequence; const timer = setTimeout(() => reject(new Error('Native timeout')), 60000); pending.set(requestId, { resolve: resolvePromise, reject, timer }); daemon.stdin.write(JSON.stringify({ id: requestId, command, args }) + '\n'); });
try {
    const initial = await call('profiles_list');
    const fixture = initial.find(profile => profile.id === id);
    assert.ok(fixture);
    assert.equal(fixture.name, 'Teste temporário · desempenho 3.6');
    assert.ok(new Date(fixture.createdAt).getTime() >= Date.parse('2026-10-09T23:40:00Z'));
    const root = resolve(process.env.APPDATA, 'github/Luxmc/data/instances', id);
    assert.equal(resolve(fixture.gameDir).toLowerCase(), resolve(root, '.minecraft').toLowerCase());
    const saves = join(fixture.gameDir, 'saves');
    if (existsSync(saves)) assert.equal(readdirSync(saves).length, 0, 'A instância contém mundos; revise manualmente');
    await call('profiles_delete', { id });
    assert.equal(existsSync(root), false);
    const current = await call('profiles_list');
    assert.deepEqual(current.map(profile => profile.id).sort(), initial.filter(profile => profile.id !== id).map(profile => profile.id).sort());
    const report = { fixtureId: id, realFilesRemoved: true, originalInstancesPreserved: true, remainingCount: current.length };
    writeFileSync('docs/validation/v3.6/upgrades/baseline-cleanup.json', JSON.stringify(report, null, 2));
    console.log(JSON.stringify(report));
} finally { daemon.stdin.end(); daemon.kill(); }
