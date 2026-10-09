import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { readFileSync, writeFileSync, mkdirSync, existsSync, cpSync, readdirSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const binary = process.env.LUXMC_NATIVE_BINARY;
if (!binary) throw new Error('Defina o executável que será verificado.');
const daemon = spawn(binary, ['--daemon'], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
const waiting = new Map();
let sequence = 0;
let stderr = '';
daemon.stderr.on('data', chunk => stderr = (stderr + chunk).slice(-2000));
createInterface({ input: daemon.stdout }).on('line', line => {
    let message; try { message = JSON.parse(line); } catch { return; }
    const pending = waiting.get(message.id); if (!pending) return;
    clearTimeout(pending.timer); waiting.delete(message.id);
    message.error ? pending.reject(new Error(message.error)) : pending.resolve(message.result);
});
daemon.on('exit', code => { for (const pending of waiting.values()) { clearTimeout(pending.timer); pending.reject(new Error(`Daemon encerrou: ${code}; ${stderr}`)); } waiting.clear(); });
const call = (command, args = {}) => new Promise((resolve, reject) => {
    const id = ++sequence;
    const timer = setTimeout(() => { waiting.delete(id); reject(new Error(`Tempo esgotado: ${command}`)); }, 600000);
    waiting.set(id, { resolve, reject, timer });
    daemon.stdin.write(JSON.stringify({ id, command, args }) + '\n');
});
const created = new Set();
const report = { version: '3.6.0', revision: 1, checks: [] };
try {
    const initial = await call('profiles_list');
    const one = await call('profiles_create', { input: { name: 'Verificação de exclusão 3.6 R1', mcVersion: '1.21.1', loader: 'vanilla' } }); created.add(one.id);
    const two = await call('profiles_create', { input: { name: 'Verificação de preservação 3.6 R1', mcVersion: '1.21.1', loader: 'vanilla' } }); created.add(two.id);
    const root = resolve(join(process.env.APPDATA, 'github/Luxmc/data/instances'));
    for (const profile of [one, two]) assert.equal(resolve(profile.gameDir).toLowerCase(), resolve(root, profile.id, '.minecraft').toLowerCase());
    const marker = join(two.gameDir, 'options.txt'); writeFileSync(marker, 'Fixture must remain');
    mkdirSync(join(one.gameDir, 'saves/fixture'), { recursive: true }); writeFileSync(join(one.gameDir, 'saves/fixture/level.dat'), 'Disposable fixture');
    const lockedFile = join(one.gameDir, 'locked.txt'); writeFileSync(lockedFile, 'Disposable locked file');
    const shell = join(process.env.SystemRoot, 'System32/WindowsPowerShell/v1.0/powershell.exe');
    const lock = spawn(shell, ['-NoProfile', '-NonInteractive', '-Command', "$file=[IO.File]::Open($env:LUXMC_TEST_LOCK,[IO.FileMode]::Open,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None); [Console]::WriteLine('locked'); [Console]::In.ReadLine()|Out-Null; $file.Dispose()"], { windowsHide: true, env: { ...process.env, LUXMC_TEST_LOCK: lockedFile }, stdio: ['pipe', 'pipe', 'pipe'] });
    await new Promise((resolve, reject) => { lock.stdout.once('data', resolve); lock.once('error', reject); lock.once('exit', code => code && reject(new Error('Falha no bloqueio de teste'))); });
    try {
        await assert.rejects(call('profiles_delete', { id: one.id }), /apagar os arquivos|access|acesso/i);
        assert.ok((await call('profiles_list')).some(profile => profile.id === one.id));
        report.checks.push({ name: 'locked-file-failure', databaseRecordPreserved: true });
    } finally { lock.stdin.end('\n'); await new Promise(resolve => lock.once('exit', resolve)); }
    await call('profiles_delete', { id: one.id }); created.delete(one.id);
    assert.equal(existsSync(resolve(one.gameDir, '..')), false);
    assert.equal((await call('profiles_list')).some(profile => profile.id === one.id), false);
    assert.equal(readFileSync(marker, 'utf8'), 'Fixture must remain');
    report.checks.push({ name: 'real-java-deletion', filesRemoved: true, databaseRecordRemoved: true, otherInstancePreserved: true });
    await call('profiles_delete', { id: two.id }); created.delete(two.id);
    assert.deepEqual((await call('profiles_list')).map(profile => profile.id).sort(), initial.map(profile => profile.id).sort());
    const preferences = join(process.env.APPDATA, 'io.github.luxmc.Luxmc/settings.json');
    if (!process.argv.includes('--fixtures-only') && existsSync(preferences)) {
        const cached = JSON.parse(readFileSync(preferences, 'utf8')).app;
        const appearance = Object.fromEntries(['theme', 'accentTheme', 'customBackground', 'customWallpaperUrl', 'customWallpaperType', 'wallpaperLibrary', 'performanceMode'].filter(key => cached[key] !== undefined).map(key => [key, cached[key]]));
        await call('settings_set', { value: { ...appearance, animations: true, respectReducedMotion: false, settingsSavedAt: Date.now() } });
        const restored = await call('settings_get');
        for (const [key, value] of Object.entries(appearance)) assert.deepEqual(restored[key], value);
        report.checks.push({ name: 'appearance', selectedThemePreserved: true, selectedWallpaperPreserved: true, explicitAnimationsEnabled: true });
    }
    if (process.argv.includes('--bedrock-removal')) {
        const worldRoot = join(process.env.LOCALAPPDATA, 'Packages/Microsoft.MinecraftUWP_8wekyb3d8bbwe/LocalState/games/com.mojang/minecraftWorlds');
        const worldBackup = join(process.env.LOCALAPPDATA, 'Luxmc-build', 'validation-world-preservation', `${Date.now()}`);
        const worldHashes = new Map();
        function recordWorlds(path, relative = '') {
            for (const entry of readdirSync(path, { withFileTypes: true })) {
                if (entry.isSymbolicLink()) throw new Error('Os mundos de teste não podem conter vínculos.');
                const name = join(relative, entry.name);
                if (entry.isDirectory()) recordWorlds(join(path, entry.name), name);
                else if (entry.isFile()) worldHashes.set(name, createHash('sha256').update(readFileSync(join(path, entry.name))).digest('hex'));
            }
        }
        if (existsSync(worldRoot)) { recordWorlds(worldRoot); mkdirSync(worldBackup, { recursive: true }); cpSync(worldRoot, worldBackup, { recursive: true }); }
        const stateFile = join(process.env.APPDATA, 'github/Luxmc/data/bedrock-provider.json');
        const state = JSON.parse(readFileSync(stateFile, 'utf8'));
        const old = state.managed.find(item => item.installation.version === '1.20.81.01');
        const latest = state.managed.find(item => item.installation.version === '1.26.52.3');
        assert.ok(old && latest && existsSync(old.package) && existsSync(latest.package));
        const name = 'Verificação de remoção Bedrock 3.6 R1';
        assert.equal(state.instances.some(item => item.installationId === old.installation.id), false, 'A instalação de teste não pode substituir uma instância do usuário.');
        try {
            await call('bedrock_install', { versionId: old.installation.id, name });
            const instance = (await call('bedrock_state')).instances.find(item => item.name === name);
            assert.ok(instance);
            await call('bedrock_remove', { id: instance.id });
            assert.equal(existsSync(old.installation.directory), false);
            assert.equal((await call('bedrock_state')).instances.some(item => item.id === instance.id), false);
            report.checks.push({ name: 'real-bedrock-removal', version: old.installation.version, windowsPackageRemoved: true, cachedFilesRemoved: true, libraryRecordRemoved: true });
        } finally {
            try {
                await call('bedrock_install', { versionId: latest.installation.id, name: `Bedrock ${latest.installation.version}` });
            } finally {
                if (worldHashes.size) {
                    mkdirSync(worldRoot, { recursive: true }); cpSync(worldBackup, worldRoot, { recursive: true });
                    for (const [path, hash] of worldHashes) assert.equal(createHash('sha256').update(readFileSync(join(worldRoot, path))).digest('hex'), hash);
                }
            }
            report.checks.push({ name: 'bedrock-world-preservation', filesVerified: worldHashes.size, originalFilesPreserved: true });
            report.checks.push({ name: 'bedrock-restoration', version: latest.installation.version, installedAgain: true });
        }
    }
    report.binarySha256 = createHash('sha256').update(readFileSync(binary)).digest('hex');
    mkdirSync('docs/validation/v3.6/hotfix', { recursive: true });
    writeFileSync('docs/validation/v3.6/hotfix/native-results.json', JSON.stringify(report, null, 2));
    console.log(JSON.stringify(report));
} finally {
    for (const id of created) { try { await call('profiles_delete', { id }); } catch {} }
    daemon.stdin.end();
}
