import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
assert.ok(process.env.LUXMC_NATIVE_BINARY);
const daemon = spawn(process.env.LUXMC_NATIVE_BINARY, ['--daemon'], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
const waiting = new Map(); let sequence = 0;
createInterface({ input: daemon.stdout }).on('line', line => { let response; try { response = JSON.parse(line); } catch { return; } const pending = waiting.get(response.id); if (!pending) return; waiting.delete(response.id); clearTimeout(pending.timer); response.error ? pending.reject(new Error(response.error)) : pending.resolve(response.result); });
const call = (command, args = {}) => new Promise((resolvePromise, reject) => { const id = ++sequence; const timer = setTimeout(() => reject(new Error(`Native timeout: ${command}`)), 600000); waiting.set(id, { resolve: resolvePromise, reject, timer }); daemon.stdin.write(JSON.stringify({ id, command, args }) + '\n'); });
const sleep = ms => new Promise(resolvePromise => setTimeout(resolvePromise, ms));
const alive = pid => { try { process.kill(pid, 0); return true; } catch { return false; } };
let fixture;
let pid;
let initial;
const report = { version: '1.8.9', completed: false, buildVariant: process.env.LUXMC_BUILD_VARIANT || 'release' };
const output = process.env.LUXMC_VALIDATION_REPORT || 'docs/validation/v3.6/upgrades/native-legacy.json';
try {
    const initialized = await call('app_init');
    assert.ok(initialized.account);
    initial = await call('profiles_list');
    fixture = await call('profiles_create', { input: { name: 'Teste temporário · Java 8 e Minecraft 1.8.9', mcVersion: '1.8.9', loader: 'vanilla', ramMb: 2048 } });
    assert.ok(!initial.some(profile => profile.id === fixture.id));
    const root = resolve(process.env.APPDATA, 'github/Luxmc/data/instances', fixture.id);
    assert.equal(resolve(fixture.gameDir).toLowerCase(), resolve(root, '.minecraft').toLowerCase());
    report.fixtureId = fixture.id;
    const start = performance.now();
    const launched = await call('launch_game', { request: { versionId: '1.8.9', profileId: fixture.id, accountId: initialized.account.id, enableVulkan: false } });
    pid = launched.pid;
    assert.ok(pid > 0);
    report.preparationSeconds = Number(((performance.now() - start) / 1000).toFixed(2));
    writeFileSync(output, JSON.stringify(report, null, 2));
    let ready = false;
    for (let attempt = 0; attempt < 180; attempt++) {
        assert.ok(alive(pid), 'O Minecraft 1.8.9 encerrou durante o teste');
        const logFile = join(fixture.gameDir, 'logs/latest.log');
        const log = existsSync(logFile) ? readFileSync(logFile, 'utf8') : '';
        if (/Sound engine started/i.test(log)) { ready = true; break; }
        await sleep(1000);
    }
    assert.ok(ready, 'O Minecraft 1.8.9 não inicializou o áudio e os recursos');
    await sleep(8000);
    assert.ok(alive(pid));
    report.resourcesSeconds = Number(((performance.now() - start) / 1000).toFixed(2));
    report.completed = true;
    console.log(`Minecraft 1.8.9 real iniciou com Java 8; preparação ${report.preparationSeconds}s.`);
} catch (error) { report.error = error.message; console.error(error.message); process.exitCode = 1; }
finally {
    try {
        if (pid) { await call('stop_game', { pid }); for (let attempt = 0; attempt < 50 && alive(pid); attempt++) await sleep(200); assert.equal(alive(pid), false); }
        if (fixture) {
            assert.ok(!initial.some(profile => profile.id === fixture.id));
            const root = resolve(process.env.APPDATA, 'github/Luxmc/data/instances', fixture.id);
            assert.equal(resolve(fixture.gameDir).toLowerCase(), resolve(root, '.minecraft').toLowerCase());
            await call('profiles_delete', { id: fixture.id });
            assert.equal(existsSync(root), false);
            const current = await call('profiles_list');
            assert.deepEqual(current.map(profile => profile.id).sort(), initial.map(profile => profile.id).sort());
            report.originalInstancesPreserved = true;
        }
    } catch (error) { report.cleanupError = error.message; process.exitCode = 1; }
    writeFileSync(output, JSON.stringify(report, null, 2));
    daemon.stdin.end(); daemon.kill();
}
