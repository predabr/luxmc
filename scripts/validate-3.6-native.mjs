import { spawn, execFileSync } from 'node:child_process';
import { createInterface } from 'node:readline';
import { mkdirSync, readFileSync, writeFileSync, existsSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { createHash, randomInt } from 'node:crypto';
import assert from 'node:assert/strict';

const output = resolve('docs/validation/v3.6');
mkdirSync(output, { recursive: true });
const daemon = spawn(process.env.LUXMC_NATIVE_BINARY || `${process.env.LOCALAPPDATA}/Luxmc/luxmc.exe`, ['--daemon'], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
const pending = new Map();
let sequence = 0;
let gamePid;
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
        const timer = setTimeout(() => { pending.delete(id); reject(new Error(`Tempo esgotado: ${command}`)); }, 600000);
        pending.set(id, { resolve, reject, timer });
        daemon.stdin.write(JSON.stringify({ id, command, args }) + '\n');
    });
}
function check(name, details) { receipt.checks.push({ name, ...details }); console.log(JSON.stringify({ name, ...details })); }
try {
    const info = await rpc('app_info');
    check('build', { version: info.version });
    const accounts = await rpc('auth_accounts');
    const account = accounts.find(value => value.refreshToken && !value.id.startsWith('luxmc:'));
    assert.ok(account, 'Conta Microsoft autenticada indisponível para o teste real.');
    check('account', { authenticatedAccountSelected: true, credentialsRecorded: false });
    let profile;
    if (process.env.LUXMC_REUSE_VALIDATION === '1') {
        profile = (await rpc('profiles_list')).filter(value => value.name.startsWith('Validação 3.6')).at(-1);
        assert.ok(profile);
    } else {
    const candidates = ['simply-optimized', 'adrenaline', 'fabulously-optimized'];
    const project = candidates[randomInt(candidates.length)];
    const response = await fetch(`https://api.modrinth.com/v2/project/${project}/version?game_versions=%5B%221.21.1%22%5D`, { signal: AbortSignal.timeout(30000) });
    assert.equal(response.status, 200);
    const version = (await response.json()).find(value => value.files.some(file => file.filename.endsWith('.mrpack')));
    assert.ok(version, 'Pacote publicado não encontrado.');
    const file = version.files.find(file => file.primary) || version.files[0];
    const started = performance.now();
    const archive = await rpc('mods_download_to_temp', { url: file.url, fileName: file.filename });
    assert.equal(createHash('sha512').update(readFileSync(archive)).digest('hex'), file.hashes.sha512);
    profile = await rpc('instance_import_mrpack', { filePath: archive, profileName: `Validação 3.6 · ${project}`, ramMb: 3072 });
    const index = JSON.parse(readFileSync(join(profile.gameDir, 'modrinth.index.json'), 'utf8'));
    let verified = 0;
    for (const entry of index.files.filter(value => value.env?.client !== 'unsupported')) {
        const bytes = readFileSync(join(profile.gameDir, entry.path));
        const algorithm = entry.hashes.sha512 ? 'sha512' : 'sha1';
        assert.equal(createHash(algorithm).update(bytes).digest('hex'), entry.hashes[algorithm]);
        verified++;
    }
    check('real-modpack-install', { project, version: version.version_number, minecraft: profile.mcVersion, verifiedFiles: verified, seconds: Math.round((performance.now() - started) / 100) / 10 });
    }
    await rpc('profiles_update', { input: { id: profile.id, resolutionW: 854, resolutionH: 480, fullscreen: false } });
    const launchStarted = performance.now();
    const game = await rpc('launch_game', { request: { versionId: profile.mcVersion, accountId: account.id, profileId: profile.id } });
    gamePid = game.pid;
    assert.ok(gamePid > 0);
    check('game-process', { started: true, preparationSeconds: Math.round((performance.now() - launchStarted) / 100) / 10 });
    let initialized = false;
    for (let attempt = 0; attempt < 24; attempt++) {
        await new Promise(resolve => setTimeout(resolve, 5000));
        const logPath = join(profile.gameDir, 'logs/latest.log');
        const log = existsSync(logPath) ? readFileSync(logPath, 'utf8') : '';
        initialized = /Sound engine started|Created: .*minecraft:textures\/atlas\/blocks/.test(log);
        if (initialized) break;
        if (/Exception in thread "main"|Failed to create window/.test(log)) throw new Error('O jogo falhou durante a inicialização.');
    }
    const processInfo = JSON.parse(execFileSync('powershell.exe', ['-NoProfile', '-Command', `Get-Process -Id ${Number(gamePid)} -ErrorAction SilentlyContinue | Select-Object Responding,MainWindowTitle | ConvertTo-Json -Compress`], { encoding: 'utf8', windowsHide: true }) || 'null');
    assert.ok(initialized && processInfo?.Responding);
    check('minecraft-renderer', { initialized, responding: processInfo.Responding, windowPresent: Boolean(processInfo.MainWindowTitle), worldsModified: false });
    for (let attempt = 0; attempt < 18; attempt++) {
        await new Promise(resolve => setTimeout(resolve, 5000));
        try { process.kill(gamePid, 0); } catch { throw new Error('O Minecraft encerrou durante a observação de estabilidade.'); }
    }
    check('game-stability', { observedAfterRendererSeconds: 90, processAlive: true });
    const closed = execFileSync('powershell.exe', ['-NoProfile', '-Command', `$game = Get-Process -Id ${Number(gamePid)} -ErrorAction Stop; if (!$game.MainWindowTitle.Contains('Minecraft')) { throw 'Janela Minecraft não localizada' }; $game.CloseMainWindow() | Out-Null; if (!$game.WaitForExit(30000)) { throw 'Minecraft não encerrou normalmente' }; Write-Output 'closed'`], { encoding: 'utf8', windowsHide: true });
    assert.match(closed, /closed/);
    gamePid = undefined;
    const finalLog = readFileSync(join(profile.gameDir, 'logs/latest.log'), 'utf8');
    assert.match(finalLog, /Stopping!/);
    check('normal-shutdown', { stoppedCleanly: true, forcedTermination: false });
} catch (error) {
    receipt.failure = String(error);
    console.error(receipt.failure);
    process.exitCode = 1;
} finally {
    if (gamePid) { try { process.kill(gamePid); } catch {} }
    writeFileSync(join(output, process.env.LUXMC_VALIDATION_NAME || 'native-results.json'), JSON.stringify(receipt, null, 2) + '\n');
    daemon.stdin.end();
    daemon.kill();
}
