import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';

const binary = process.env.LUXMC_NATIVE_BINARY;
const archive = process.env.LUXMC_VALIDATION_ARCHIVE;
const output = process.env.LUXMC_VALIDATION_REPORT;
assert.ok(binary && archive && output);
const daemon = spawn(binary, ['--daemon'], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'], env: { ...process.env, RUST_LOG: 'warn,luxmc_lib::commands::instances=info,luxmc_lib::core::mods::validation=info' } });
const waiting = new Map();
let sequence = 0;
let fixture;
let initial;
const report = { sourceArchive: resolve(archive), phases: [] };
daemon.stderr.on('data', chunk => {
    for (const line of chunk.toString().split('\n')) if (/Modpack registration phase completed|Slow archive validation/.test(line)) {
        report.phases.push(line.replace(/\u001b\[[0-9;]*m/g, ''));
        writeFileSync(output, JSON.stringify(report, null, 2));
        console.log(line);
    }
});
createInterface({ input: daemon.stdout }).on('line', line => {
    let response;
    try { response = JSON.parse(line); } catch { return; }
    const pending = waiting.get(response.id);
    if (!pending) return;
    waiting.delete(response.id);
    clearTimeout(pending.timer);
    response.error ? pending.reject(new Error(response.error)) : pending.resolve(response.result);
});
daemon.on('exit', code => {
    for (const pending of waiting.values()) { clearTimeout(pending.timer); pending.reject(new Error(`Daemon encerrou: ${code}`)); }
    waiting.clear();
});
const call = (command, args = {}) => new Promise((resolvePromise, reject) => {
    const id = ++sequence;
    const timer = setTimeout(() => { waiting.delete(id); reject(new Error(`Tempo esgotado: ${command}`)); }, 900000);
    waiting.set(id, { resolve: resolvePromise, reject, timer });
    daemon.stdin.write(JSON.stringify({ id, command, args }) + '\n');
});
try {
    await call('app_init');
    initial = await call('profiles_list');
    const name = `Validação temporária de importação ${Date.now()}`;
    assert.ok(!initial.some(profile => profile.name === name));
    const started = performance.now();
    const imported = await call('instance_import_modpack', { filePath: archive, profileName: name, mcVersion: '1.21.1', loader: 'neoforge', ramMb: 6144 });
    fixture = imported.profile || imported;
    assert.ok(fixture.id && !initial.some(profile => profile.id === fixture.id));
    report.installSeconds = +((performance.now() - started) / 1000).toFixed(2);
    const history = await call('performance_history');
    report.installationMetrics = history.find(entry => entry.profileId === fixture.id && entry.metrics?.kind === 'installation')?.metrics;
    writeFileSync(output, JSON.stringify(report, null, 2));
    const root = resolve(process.env.APPDATA, 'github/Luxmc/data/instances', fixture.id, '.minecraft');
    assert.equal(resolve(fixture.gameDir).toLowerCase(), root.toLowerCase());
    const manifest = JSON.parse(readFileSync(join(root, 'manifest.json'), 'utf8'));
    const cc = manifest.files.filter(file => [282001, 1676502].includes(file.projectID));
    assert.equal(cc.length, 1, 'A importação manteve as duas versões de computercraft');
    assert.equal(cc[0].fileID, 8778582);
    assert.equal(existsSync(join(root, 'mods/cc-tweaked-1.21.1-forge-1.113.1.jar')), false);
    assert.equal(existsSync(join(root, 'mods/cc-tweaked-1.21.1-forge-1.120.2.jar')), true);
    report.duplicateResolved = true;
    report.manifestEntries = manifest.files.length;
    const readiness = await call('doctor_instance_readiness', { profileId: fixture.id });
    assert.equal(readiness.conflicts.hasConflicts, false);
    console.log(`ATM10 importado em ${report.installSeconds}s; duplicata resolvida durante a instalação.`);
} catch (error) { report.error = error.message; process.exitCode = 1; }
finally {
    if (fixture) {
        try {
            assert.ok(initial && !initial.some(profile => profile.id === fixture.id));
            const ownedRoot = resolve(process.env.APPDATA, 'github/Luxmc/data/instances', fixture.id);
            assert.equal(resolve(fixture.gameDir).toLowerCase(), resolve(ownedRoot, '.minecraft').toLowerCase());
            await call('profiles_delete', { id: fixture.id });
            assert.equal(existsSync(ownedRoot), false);
            const final = await call('profiles_list');
            assert.deepEqual(final.map(profile => profile.id).sort(), initial.map(profile => profile.id).sort());
            report.originalInstancesPreserved = true;
        } catch (error) { report.cleanupError = error.message; process.exitCode = 1; }
    }
    writeFileSync(output, JSON.stringify(report, null, 2));
    daemon.stdin.end();
    daemon.kill();
}
