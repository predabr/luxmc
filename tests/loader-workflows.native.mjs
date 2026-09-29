import { spawn, execFileSync } from 'node:child_process';
import { createInterface } from 'node:readline';
import { mkdtempSync, mkdirSync, writeFileSync, statSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';

const root = mkdtempSync(join(tmpdir(), 'luxmc-loaders-'));
const daemon = spawn(resolve(process.env.LUXMC_NATIVE_BINARY || 'src-tauri/target/release/luxmc'), ['--daemon'], {
    env: { ...process.env, XDG_DATA_HOME: join(root, 'data'), XDG_CONFIG_HOME: join(root, 'config'), XDG_CACHE_HOME: join(root, 'cache') },
    stdio: ['pipe', 'pipe', 'pipe']
});
const pending = new Map();
let sequence = 0;
let stderr = '';
daemon.stderr.on('data', chunk => { stderr = (stderr + chunk).slice(-4000); });
createInterface({ input: daemon.stdout }).on('line', line => {
    let response;
    try { response = JSON.parse(line); } catch { return; }
    const request = pending.get(response.id);
    if (!request) return;
    pending.delete(response.id);
    clearTimeout(request.timer);
    if (response.error) request.reject(new Error(response.error)); else request.resolve(response.result);
});
daemon.on('exit', code => {
    for (const request of pending.values()) request.reject(new Error(`Daemon exited ${code}: ${stderr}`));
    pending.clear();
});
function call(command, args = {}) {
    return new Promise((resolve, reject) => {
        const id = ++sequence;
        const timer = setTimeout(() => { pending.delete(id); reject(new Error(`Timed out: ${command}`)); }, 90000);
        pending.set(id, { resolve, reject, timer });
        daemon.stdin.write(JSON.stringify({ id, command, args }) + '\n');
    });
}

try {
    const combinations = [
        ['forge', '1.20.1'],
        ['quilt', '1.21.4'],
        ['neoforge', '1.21.4']
    ];
    const profiles = new Map();
    for (const [loader, mcVersion] of combinations) {
        const response = await call('loaders_versions', { loader, mcVersion });
        assert.ok(response.versions?.length, `${loader} ${mcVersion} has no versions`);
        const selected = response.versions.find(version => version.stable) || response.versions[0];
        const profile = await call('profiles_create', { input: { name: `${loader} smoke`, mcVersion, loader, loaderVersion: selected.id } });
        profiles.set(loader, profile);
        assert.equal(profile.loaderVersion, selected.id);
        if (process.platform === 'linux') assert.equal(profile.useGamemode, true);
        console.log(`PASS ${loader} ${mcVersion}: ${selected.id}`);
    }

    const fabric = await call('profiles_create', { input: { name: 'Fabric mod import', mcVersion: '1.21.4', loader: 'fabric' } });
    const metadata = join(root, 'fabric.mod.json');
    const jar = join(root, 'sample-fabric.jar');
    writeFileSync(metadata, JSON.stringify({ schemaVersion: 1, id: 'smoke', version: '1.0.0', name: 'Smoke mod' }));
    execFileSync('bsdtar', ['--format', 'zip', '-cf', jar, 'fabric.mod.json'], { cwd: root });
    await call('instance_mod_add', { profileId: fabric.id, sourcePath: jar });
    assert.ok(statSync(join(fabric.gameDir, 'mods', 'sample-fabric.jar')).size > 0);
    const health = await call('instance_health_check', { profileId: fabric.id });
    assert.equal(health.modsOk, true, JSON.stringify(health.issues));
    await call('instance_mod_add', { profileId: fabric.id, sourcePath: jar });
    const otherDir = join(root, 'other');
    mkdirSync(otherDir, { recursive: true });
    writeFileSync(join(otherDir, 'fabric.mod.json'), JSON.stringify({ schemaVersion: 1, id: 'smoke', version: '2.0.0', name: 'Other smoke' }));
    execFileSync('bsdtar', ['--format', 'zip', '-cf', join(otherDir, 'sample-fabric.jar'), 'fabric.mod.json'], { cwd: otherDir });
    await assert.rejects(
        call('instance_mod_add', { profileId: fabric.id, sourcePath: join(otherDir, 'sample-fabric.jar') }),
        /conteúdo diferente/
    );
    console.log('PASS manual JAR import, health sync and conflicting duplicate protection');

    mkdirSync(join(root, 'META-INF'));
    for (const [loader, metadataFile] of [['forge', 'META-INF/mods.toml'], ['quilt', 'quilt.mod.json'], ['neoforge', 'META-INF/neoforge.mods.toml']]) {
        writeFileSync(join(root, metadataFile), loader === 'quilt' ? '{"quilt_loader":{"id":"smoke"}}' : 'modLoader="javafml"');
        const loaderJar = join(root, `sample-${loader}.jar`);
        execFileSync('bsdtar', ['--format', 'zip', '-cf', loaderJar, metadataFile], { cwd: root });
        await call('instance_mod_add', { profileId: profiles.get(loader).id, sourcePath: loaderJar });
        assert.ok(statSync(join(profiles.get(loader).gameDir, 'mods', `sample-${loader}.jar`)).size > 0);
        console.log(`PASS ${loader} JAR import`);
    }

    const vanilla = await call('profiles_create', { input: { name: 'Vanilla JAR guard', mcVersion: '1.21.4', loader: 'vanilla' } });
    await assert.rejects(call('instance_mod_add', { profileId: vanilla.id, sourcePath: jar }), /Vanilla não carregam mods/);
    console.log('PASS Vanilla JAR guard');

    writeFileSync(join(root, 'bare-asset.txt'), 'not a real mod');
    const bareJar = join(root, 'sample-bare.jar');
    execFileSync('bsdtar', ['--format', 'zip', '-cf', bareJar, 'bare-asset.txt'], { cwd: root });
    await call('instance_mod_add', { profileId: profiles.get('forge').id, sourcePath: bareJar });
    assert.ok(statSync(join(profiles.get('forge').gameDir, 'mods', 'sample-bare.jar')).size > 0);
    console.log('PASS jar without loader metadata is accepted');

    const payload = readFileSync(jar).toString('base64');
    await call('instance_mod_add_bytes', { profileId: fabric.id, fileName: 'dropped-fabric.jar', dataBase64: payload });
    assert.ok(statSync(join(fabric.gameDir, 'mods', 'dropped-fabric.jar')).size > 0);
    await assert.rejects(
        call('instance_mod_add_bytes', { profileId: fabric.id, fileName: '../evil.jar', dataBase64: payload }),
        /Invalid file name/
    );
    await assert.rejects(
        call('instance_mod_add_bytes', { profileId: fabric.id, fileName: 'notes.txt', dataBase64: payload }),
        /Apenas ficheiros \.jar/
    );
    console.log('PASS base64 drop import and filename guards');
    console.log(`Evidence directory: ${root}`);
} finally {
    daemon.stdin.end();
    daemon.kill();
}
