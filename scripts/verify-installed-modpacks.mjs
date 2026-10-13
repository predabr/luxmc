import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { existsSync, readFileSync, writeFileSync, mkdirSync, copyFileSync, statSync } from 'node:fs';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';

const binary = process.env.LUXMC_NATIVE_BINARY;
const output = process.env.LUXMC_VALIDATION_REPORT;
const backup = process.env.LUXMC_VALIDATION_BACKUP;
assert.ok(binary && output && backup, 'Configure o executável, relatório e diretório de backup');
const names = JSON.parse(process.env.LUXMC_VALIDATION_PACKS || '["RLCraft","All the Mods 10 - ATM10","Vanilla 26.3"]');
const daemon = spawn(binary, ['--daemon'], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
const waiting = new Map();
let sequence = 0;
let ownedPid;
const report = { binary: resolve(binary), runs: [] };
const redact = value => String(value).replace(/(--(?:accessToken|clientToken|password)\s+)[^\s]+/gi, '$1[redacted]').replace(/eyJ[A-Za-z0-9_.-]+/g, '[redacted]');
daemon.stderr.on('data', chunk => {
    for (const line of chunk.toString().split('\n')) if (/Modpack registration phase completed|Slow archive validation/.test(line)) console.log(redact(line));
});
createInterface({ input: daemon.stdout }).on('line', line => {
    let response;
    try { response = JSON.parse(line); } catch { return; }
    const pending = waiting.get(response.id);
    if (!pending) return;
    waiting.delete(response.id);
    clearTimeout(pending.timer);
    response.error ? pending.reject(new Error(redact(response.error))) : pending.resolve(response.result);
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
const sleep = ms => new Promise(resolvePromise => setTimeout(resolvePromise, ms));
const alive = pid => { try { process.kill(pid, 0); return true; } catch { return false; } };
const persist = () => writeFileSync(output, JSON.stringify(report, null, 2));
const stop = async () => {
    if (!ownedPid || !alive(ownedPid)) { ownedPid = undefined; return; }
    await call('stop_game', { pid: ownedPid });
    for (let n = 0; n < 100 && alive(ownedPid); n++) await sleep(200);
    assert.equal(alive(ownedPid), false, 'O jogo iniciado pelo teste ainda está aberto');
    ownedPid = undefined;
};
try {
    const initialized = await call('app_init');
    assert.ok(initialized.account?.id, 'Selecione uma conta no launcher');
    const initial = await call('profiles_list');
    for (const name of names) {
        const matches = initial.filter(profile => profile.name === name);
        assert.equal(matches.length, 1, `Instância não identificada com segurança: ${name}`);
        const profile = matches[0];
        const expected = resolve(process.env.APPDATA, 'github/Luxmc/data/instances', profile.id, '.minecraft');
        assert.equal(resolve(profile.gameDir).toLowerCase(), expected.toLowerCase());
        const saved = join(backup, profile.id);
        mkdirSync(saved, { recursive: true });
        for (const file of ['manifest.json', 'modrinth.index.json', 'options.txt']) if (existsSync(join(expected, file)) && !existsSync(join(saved, file))) copyFileSync(join(expected, file), join(saved, file));
        const entry = { name, minecraft: profile.mcVersion, loader: profile.loader, runs: [] };
        report.runs.push(entry);
        persist();
        try {
            if (process.argv.includes('--repair')) {
                const start = performance.now();
                await call('instance_repair', { profileId: profile.id });
                entry.repairedMods = await call('instance_repair_modpack', { profileId: profile.id });
                entry.repairSeconds = +((performance.now() - start) / 1000).toFixed(2);
                console.log(`${name}: reparo completo em ${entry.repairSeconds}s, ${entry.repairedMods} arquivos recuperados/substituídos.`);
            }
            const start = performance.now();
            const readiness = await call('doctor_instance_readiness', { profileId: profile.id });
            entry.readinessSeconds = +((performance.now() - start) / 1000).toFixed(2);
            entry.blockers = readiness.blockers;
            assert.ok(readiness.ready, readiness.blockers.join('\n'));
            for (let run = 1; run <= (process.argv.includes('--single-run') ? 1 : 2); run++) {
                const started = performance.now();
                const logStartedAt = Date.now();
                const launched = await call('launch_game', { request: { versionId: profile.mcVersion, profileId: profile.id, accountId: initialized.account.id, enableVulkan: false } });
                assert.ok(launched.pid > 0);
                ownedPid = launched.pid;
                const result = { run, windowSeconds: +((performance.now() - started) / 1000).toFixed(2), windowConfirmed: true };
                entry.runs.push(result);
                persist();
                const logFile = join(expected, 'logs/latest.log');
                let ready = false;
                const requiresModInitialization = profile.loader === 'neoforge' || (profile.loader === 'forge' && profile.mcVersion === '1.12.2');
                for (let attempt = 0; attempt < 600; attempt++) {
                    assert.ok(alive(ownedPid), 'O Minecraft encerrou durante o carregamento');
                    const log = existsSync(logFile) && statSync(logFile).mtimeMs >= logStartedAt ? readFileSync(logFile, 'utf8') : '';
                    assert.ok(!/\b(?:ModLoadingException|ModResolutionException|ReportedException)\b|Mod loading error has occurred|Crash report saved to/.test(log), 'O jogo registrou uma falha fatal de carregamento');
                    const resources = /Sound engine started|Created:.*minecraft:textures\/atlas\/blocks|OpenAL initialized/i.test(log);
                    const initialized = /Forge Mod Loader has successfully loaded \d+ mods|Game took [\d.]+ seconds to start|ScreenCustomizationLayer registered: title_screen/i.test(log);
                    if (resources && (!requiresModInitialization || initialized)) { result.modInitializationConfirmed = initialized; ready = true; break; }
                    await sleep(1000);
                }
                assert.ok(ready, 'O jogo não confirmou a inicialização dos recursos em dez minutos');
                result.resourcesSeconds = +((performance.now() - started) / 1000).toFixed(2);
                for (let n = 0; n < 60; n++) { assert.ok(alive(ownedPid), 'O jogo encerrou após iniciar os recursos'); await sleep(1000); }
                result.remainedOpenSeconds = 60;
                console.log(`${name}, abertura ${run}: janela ${result.windowSeconds}s, recursos ${result.resourcesSeconds}s, estável por 60s.`);
                await stop();
                persist();
            }
        } catch (error) {
            entry.error = redact(error.message);
            console.error(`${name}: ${entry.error}`);
            process.exitCode = 1;
            await stop();
            persist();
        }
    }
    const final = await call('profiles_list');
    assert.deepEqual(final.map(item => item.id).sort(), initial.map(item => item.id).sort());
    report.originalInstancesPreserved = true;
} catch (error) { report.error = redact(error.message); process.exitCode = 1; }
finally {
    try { await stop(); } catch (error) { report.cleanupError = redact(error.message); process.exitCode = 1; }
    persist();
    daemon.stdin.end();
    if (!report.cleanupError) daemon.kill();
}
