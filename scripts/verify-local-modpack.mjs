import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { existsSync, readFileSync, writeFileSync, renameSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const binary = process.env.LUXMC_NATIVE_BINARY;
assert.ok(binary);
const daemon = spawn(binary, ['--daemon'], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
const waiting = new Map();
let sequence = 0;
let fixture;
let initial;
let gamePid;
let gameStopped = true;
const report = { originalInstancesPreserved: false, runs: [], buildVariant: process.env.LUXMC_BUILD_VARIANT || 'release' };
const output = process.env.LUXMC_VALIDATION_REPORT || 'docs/validation/v3.6/upgrades/native-modpack.json';
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
    const timer = setTimeout(() => { waiting.delete(id); reject(new Error(`Tempo esgotado: ${command}`)); }, 600000);
    waiting.set(id, { resolve: resolvePromise, reject, timer });
    daemon.stdin.write(JSON.stringify({ id, command, args }) + '\n');
});
const sleep = ms => new Promise(resolvePromise => setTimeout(resolvePromise, ms));
const hash = (bytes, algorithm) => createHash(algorithm).update(bytes).digest('hex');
const alive = pid => { try { process.kill(pid, 0); return true; } catch { return false; } };
const stopOwnedGame = async () => {
    if (!gamePid || !alive(gamePid)) { gamePid = undefined; gameStopped = true; return; }
    await call('stop_game', { pid: gamePid });
    for (let n = 0; n < 50 && alive(gamePid); n++) await sleep(200);
    assert.equal(alive(gamePid), false, 'O jogo de teste ainda está aberto');
    gamePid = undefined;
    gameStopped = true;
};
try {
    const initialized = await call('app_init');
    assert.ok(initialized.account, 'Selecione sua conta no launcher antes de testar');
    initial = await call('profiles_list');
    const response = await fetch('https://api.modrinth.com/v2/project/fabulously-optimized/version?game_versions=%5B%221.21.1%22%5D', { signal: AbortSignal.timeout(30000) });
    assert.equal(response.status, 200);
    const version = (await response.json()).find(item => item.version_type === 'release');
    assert.ok(version);
    const file = version.files.find(item => item.primary) || version.files[0];
    const start = performance.now();
    const archive = await call('mods_download_to_temp', { url: file.url, fileName: file.filename });
    assert.equal(hash(readFileSync(archive), 'sha512'), file.hashes.sha512);
    const imported = await call('instance_import_mrpack', { filePath: archive, profileName: 'Teste temporário · desempenho 3.6', ramMb: 4096 });
    fixture = imported.profile || imported;
    assert.ok(fixture.id && !initial.some(item => item.id === fixture.id));
    const ownedRoot = resolve(process.env.APPDATA, 'github/Luxmc/data/instances', fixture.id);
    assert.equal(resolve(fixture.gameDir).toLowerCase(), resolve(ownedRoot, '.minecraft').toLowerCase());
    report.fixtureId = fixture.id;
    report.initialIds = initial.map(item => item.id).sort();
    report.version = version.version_number;
    report.minecraft = fixture.mcVersion;
    report.installSeconds = Number(((performance.now() - start) / 1000).toFixed(2));
    const index = JSON.parse(readFileSync(join(fixture.gameDir, 'modrinth.index.json'), 'utf8'));
    let verified = 0;
    for (const entry of index.files.filter(item => item.env?.client !== 'unsupported')) {
        const bytes = readFileSync(join(fixture.gameDir, entry.path.replaceAll('\\', '/')));
        assert.ok(bytes.length);
        const algorithm = entry.hashes.sha512 ? 'sha512' : 'sha1';
        assert.equal(hash(bytes, algorithm), entry.hashes[algorithm]);
        verified++;
    }
    report.filesVerifiedByHash = verified;
    const readiness = await call('doctor_instance_readiness', { profileId: fixture.id });
    report.blockers = readiness.blockers;
    report.warnings = readiness.warnings;
    assert.equal(readiness.blockers.length, 0);
    if (!process.argv.includes('--known-warning-baseline')) assert.ok(!readiness.warnings.some(warning => /parece ser para o Minecraft/i.test(warning)));
    console.log(`Modpack real instalado: ${verified} arquivos verificados; ${report.installSeconds}s.`);
    for (let run = 0; run < (process.argv.includes('--prepare-only') ? 0 : process.argv.includes('--single-run') ? 1 : 2); run++) {
        const logFile = join(fixture.gameDir, 'logs/latest.log');
        if (existsSync(logFile)) renameSync(logFile, join(fixture.gameDir, `logs/test-run-${run}.log`));
        const launchStart = performance.now();
        const launched = await call('launch_game', { request: { versionId: fixture.mcVersion, profileId: fixture.id, accountId: initialized.account.id, enableVulkan: false } });
        assert.ok(launched.pid > 0);
        gamePid = launched.pid;
        gameStopped = false;
        report.lastGamePid = gamePid;
        writeFileSync(output, JSON.stringify(report, null, 2));
        const preparationSeconds = Number(((performance.now() - launchStart) / 1000).toFixed(2));
        let log = '';
        let rendererReady = false;
        for (let attempt = 0; attempt < 150; attempt++) {
            assert.ok(alive(gamePid), 'O Minecraft encerrou durante o teste');
            if (existsSync(logFile)) log = readFileSync(logFile, 'utf8');
            if (/Sound engine started|Created:.*minecraft:textures\/atlas\/blocks/i.test(log)) { rendererReady = true; break; }
            await sleep(1000);
        }
        assert.ok(rendererReady, 'O Minecraft não inicializou os recursos a tempo');
        const resourcesSeconds = Number(((performance.now() - launchStart) / 1000).toFixed(2));
        await sleep(8000);
        assert.ok(alive(gamePid), 'O Minecraft encerrou depois de carregar os recursos');
        report.runs.push({ run: run + 1, preparationSeconds, resourcesSeconds, rendererReady, remainedOpen: true });
        console.log(`Abertura ${run + 1}: preparação ${preparationSeconds}s; recursos ${resourcesSeconds}s.`);
        await stopOwnedGame();
    }
} catch (error) {
    report.error = error.message;
    process.exitCode = 1;
    console.error(error.message);
} finally {
    try { await stopOwnedGame(); } catch (error) { report.cleanupError = error.message; process.exitCode = 1; }
    if (fixture && gameStopped) {
        try {
            assert.ok(initial && !initial.some(item => item.id === fixture.id));
            const root = resolve(process.env.APPDATA, 'github/Luxmc/data/instances', fixture.id);
            assert.equal(resolve(fixture.gameDir).toLowerCase(), resolve(root, '.minecraft').toLowerCase());
            const deletionStart = performance.now();
            for (let attempt = 0;; attempt++) {
                try { await call('profiles_delete', { id: fixture.id }); break; }
                catch (error) { if (attempt >= 5 || !/Feche o Minecraft desta instância/i.test(error.message)) throw error; await sleep(2000); }
            }
            report.deletionSeconds = Number(((performance.now() - deletionStart) / 1000).toFixed(2));
            assert.equal(existsSync(root), false);
            const current = await call('profiles_list');
            assert.deepEqual(current.map(item => item.id).sort(), initial.map(item => item.id).sort());
            report.originalInstancesPreserved = true;
            report.realFilesRemoved = true;
        } catch (error) { report.cleanupError = error.message; process.exitCode = 1; }
    }
    writeFileSync(output, JSON.stringify(report, null, 2));
    if (report.cleanupError && process.argv.includes('--hold-on-cleanup-error')) {
        console.log(`Diagnóstico disponível: daemon ${daemon.pid}; instância ${fixture?.id}.`);
        await sleep(120000);
    }
    daemon.stdin.end();
    daemon.kill();
}
