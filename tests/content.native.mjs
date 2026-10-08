import { spawn, execFileSync } from 'node:child_process';
import { createInterface } from 'node:readline';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { performance } from 'node:perf_hooks';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

if (process.platform !== 'linux') throw new Error('Este teste usa isolamento XDG e deve ser executado no Linux.');
const binary = resolve(process.env.LUXMC_NATIVE_BINARY || 'src-tauri/target/release/luxmc');
assert.ok(existsSync(binary), 'Compile o launcher com pnpm tauri build --no-bundle antes deste teste.');
const root = mkdtempSync(join(tmpdir(), 'luxmc content '));
const daemon = spawn(binary, ['--daemon'], {
    env: { ...process.env, XDG_DATA_HOME: join(root, 'data'), XDG_CONFIG_HOME: join(root, 'config'), XDG_CACHE_HOME: join(root, 'cache') },
    stdio: ['pipe', 'pipe', 'pipe']
});
let sequence = 0;
const pending = new Map();
let stderr = '';
daemon.stderr.on('data', chunk => { stderr = (stderr + chunk).slice(-8000); });
createInterface({ input: daemon.stdout }).on('line', line => {
    let message;
    try { message = JSON.parse(line); } catch { return; }
    const item = pending.get(message.id);
    if (!item) return;
    pending.delete(message.id);
    clearTimeout(item.timer);
    if (message.error) item.reject(new Error(message.error));
    else item.resolve(message.result);
});
daemon.on('exit', code => {
    for (const item of pending.values()) { clearTimeout(item.timer); item.reject(new Error(`Daemon exited ${code}: ${stderr}`)); }
    pending.clear();
});
const call = (command, args = {}) => new Promise((resolveCall, rejectCall) => {
    const id = ++sequence;
    const timer = setTimeout(() => { pending.delete(id); rejectCall(new Error(`${command} timed out`)); }, 60000);
    pending.set(id, { resolve: resolveCall, reject: rejectCall, timer });
    daemon.stdin.write(JSON.stringify({ id, command, args }) + '\n');
});
const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
try {
    assert.deepEqual(await call('profiles_list'), []);
    const profile = await call('profiles_create', { input: { name: 'Conteúdo com espaços', mcVersion: '1.20.1', loader: 'vanilla' } });
    const mods = join(profile.gameDir, 'mods');
    mkdirSync(mods, { recursive: true });
    const fixture = readFileSync('static/steve.png');
    writeFileSync(join(root, 'icon.png'), fixture);
    execFileSync('python3', ['-c', 'import zipfile,sys,pathlib\nroot,mods=map(pathlib.Path,sys.argv[1:])\nwith zipfile.ZipFile(root/"template.jar","w") as archive: archive.writestr("icon.png",(root/"icon.png").read_bytes())\nsource=(root/"template.jar").read_bytes()\nfor i in range(1000): (mods/f"project-{i:04d}.jar").write_bytes(source)', root, mods]);
    const first = join(mods, 'project-0000.jar');
    const original = hash(first);
    const started = performance.now();
    const entries = await call('instance_file_tree', { profileId: profile.id, subPath: 'mods' });
    const coldMs = performance.now() - started;
    assert.equal(entries.length, 1000);
    assert.ok(entries.every(entry => !entry.icon && !entry.iconResolved && entry.iconKey.startsWith('content:v2:')));
    const visible = await call('instance_content_icons', {profileId:profile.id, subPath:'mods',fileNames:entries.slice(0,24).map(entry=>entry.name)});
    assert.ok(visible.every(entry=>entry.resolved && entry.icon.startsWith('data:image/png;base64,') && entry.icon.length<32768));
    assert.equal(visible.length,24);
    assert.equal(hash(first), original);
    const warmStart = performance.now();
    assert.equal((await call('instance_file_tree', { profileId: profile.id, subPath: 'mods' })).length, 1000);
    const warmMs = performance.now() - warmStart;
    await call('instance_mod_toggle', { profileId: profile.id, fileName: 'project-0000.jar', enabled: false });
    assert.equal(hash(first + '.disabled'), original);
    await call('instance_mod_toggle', { profileId: profile.id, fileName: 'project-0000.jar.disabled', enabled: true });
    assert.equal(hash(first), original);
    mkdirSync(join(profile.gameDir, 'shaderpacks'), { recursive: true });
    writeFileSync(join(profile.gameDir, 'shaderpacks', 'shader.zip'), readFileSync(join(root, 'template.jar')));
    assert.equal((await call('instance_file_tree', { profileId: profile.id, subPath: 'shaderpacks' }))[0].name, 'shader.zip');
    console.log(JSON.stringify({ mods: entries.length, coldMs, warmMs, boundedIcons: true, originalJarsPreserved: true, toggle: true, shaders: true, isolatedData: root }));
} catch (error) {
    console.error(error, stderr);
    process.exitCode = 1;
} finally {
    daemon.stdin.end();
    daemon.kill();
    await new Promise(resolveExit => { if (daemon.exitCode !== null) resolveExit(); else daemon.once('exit', resolveExit); });
    rmSync(root, { recursive: true, force: true });
}
