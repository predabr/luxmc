import { spawn, execFileSync } from 'node:child_process';
import { createInterface } from 'node:readline';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
import assert from 'node:assert/strict';

const output = resolve('docs/validation/v3.6');
mkdirSync(output, { recursive: true });
const daemon = spawn(process.env.LUXMC_NATIVE_BINARY || `${process.env.LOCALAPPDATA}/Luxmc-build/target/debug/luxmc.exe`, ['--daemon'], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
const pending = new Map();
let sequence = 0;
const receipt = { startedAt: new Date().toISOString(), checks: [] };
daemon.stderr.on('data', () => {});
createInterface({ input: daemon.stdout }).on('line', line => {
    try {
        const message = JSON.parse(line), request = pending.get(message.id);
        if (!request) return;
        clearTimeout(request.timer); pending.delete(message.id);
        message.error ? request.reject(new Error(message.error)) : request.resolve(message.result);
    } catch {}
});
daemon.on('exit', code => {
    for (const request of pending.values()) { clearTimeout(request.timer); request.reject(new Error(`Daemon terminou: ${code}`)); }
    pending.clear();
});
function rpc(command, args = {}) {
    return new Promise((resolve, reject) => {
        const id = ++sequence;
        const timer = setTimeout(() => { pending.delete(id); reject(new Error(`Tempo esgotado: ${command}`)); }, 1200000);
        pending.set(id, { resolve, reject, timer });
        daemon.stdin.write(JSON.stringify({ id, command, args }) + '\n');
    });
}
function check(name, details) { receipt.checks.push({ name, ...details }); console.log(JSON.stringify({ name, ...details })); }
try {
    const info = await rpc('app_info');
    check('build', { version: info.version });
    const versions = await rpc('bedrock_versions');
    assert.ok(versions.length > 100);
    check('live-catalog', { versions: versions.length, latestStable: versions.find(value => value.channel === 'release')?.version, packageTypes: [...new Set(versions.map(value => value.packageType))] });
    const version = versions.find(value => value.version === (process.env.LUXMC_BEDROCK_TEST_VERSION || '1.20.81.01') && value.channel === 'release');
    assert.ok(version, 'Versão de teste indisponível no catálogo');
    const started = performance.now();
    check('installation-started', { version: version.version, packageType: version.packageType });
    await rpc('bedrock_install', { versionId: version.id, name: `Validação Bedrock 3.6 · ${version.version}` });
    const state = await rpc('bedrock_state');
    const installation = state.installations.find(value => value.version === version.version && value.profileId === 'managed');
    assert.ok(installation, 'Instalação gerenciada não persistiu');
    check('native-package-installed', { version: installation.version, seconds: Math.round((performance.now() - started) / 100) / 10, managed: true });
    const instance = state.instances.find(value => value.installationId === installation.id && value.profileId === installation.profileId);
    assert.ok(instance);
    await rpc('bedrock_open', { instanceId: instance.id });
    let game;
    for (let attempt=0;attempt<120;attempt++) {
        await new Promise(resolve => setTimeout(resolve, 5000));
        const processes = JSON.parse(execFileSync('powershell.exe', ['-NoProfile', '-Command', '$games=@(Get-Process -Name Minecraft.Windows,Minecraft.WindowsBeta -ErrorAction SilentlyContinue | Select-Object Id,Responding,MainWindowTitle); ConvertTo-Json -InputObject $games -Compress'], { encoding: 'utf8', windowsHide: true }) || '[]');
        game = processes.find(value => value?.Responding);
        if (game) break;
    }
    assert.ok(game, 'O pacote foi instalado, mas o processo Bedrock não permaneceu aberto');
    check('bedrock-process', { responding: game.Responding, windowPresent: Boolean(game.MainWindowTitle), worldsOpened: false });
    check('ready-for-visual-review', { processLeftOpen: true, forcedTermination: false });
} catch (error) {
    receipt.failure = String(error);
    console.error(receipt.failure);
    process.exitCode = 1;
} finally {
    writeFileSync(join(output, process.env.LUXMC_BEDROCK_RECEIPT || 'bedrock-native-results.json'), JSON.stringify(receipt, null, 2) + '\n');
    daemon.stdin.end(); daemon.kill();
}
