import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';

const binary = process.env.LUXMC_NATIVE_BINARY;
assert.ok(binary, 'Defina o executável instalado.');
const output = 'docs/validation/v3.6/input-motion';
const manifest = join(output, 'native-fixture.json');
const daemon = spawn(binary, ['--daemon'], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
const waiting = new Map();
let sequence = 0;
createInterface({ input: daemon.stdout }).on('line', line => {
    let response;
    try { response = JSON.parse(line); } catch { return; }
    const pending = waiting.get(response.id);
    if (!pending) return;
    clearTimeout(pending.timer);
    waiting.delete(response.id);
    response.error ? pending.reject(new Error(response.error)) : pending.resolve(response.result);
});
daemon.on('exit', code => {
    for (const pending of waiting.values()) { clearTimeout(pending.timer); pending.reject(new Error(`Daemon encerrou: ${code}`)); }
    waiting.clear();
});
const call = (command, args = {}) => new Promise((resolve, reject) => {
    const id = ++sequence;
    const timer = setTimeout(() => { waiting.delete(id); reject(new Error(`Tempo esgotado: ${command}`)); }, 30000);
    waiting.set(id, { resolve, reject, timer });
    daemon.stdin.write(JSON.stringify({ id, command, args }) + '\n');
});
try {
    if (process.argv[2] === 'prepare') {
        if (existsSync(manifest)) {
            const prior = JSON.parse(readFileSync(manifest, 'utf8'));
            assert.equal((await call('profiles_list')).some(profile => profile.id === prior.profile.id), false, 'Finalize o teste anterior antes de criar outro.');
        }
        const initial = await call('profiles_list');
        const profile = await call('profiles_create', { input: { name: 'Teste temporário · cliques e animações', mcVersion: '1.21.1', loader: 'vanilla' } });
        const root = resolve(join(process.env.APPDATA, 'github/Luxmc/data/instances'));
        assert.equal(resolve(profile.gameDir).toLowerCase(), resolve(root, profile.id, '.minecraft').toLowerCase());
        mkdirSync(join(profile.gameDir, 'saves/disposable-test'), { recursive: true });
        writeFileSync(join(profile.gameDir, 'saves/disposable-test/fixture.txt'), 'Disposable UI regression fixture');
        mkdirSync(output, { recursive: true });
        writeFileSync(manifest, JSON.stringify({ profile, initialIds: initial.map(item => item.id).sort() }, null, 2));
        console.log(JSON.stringify({ name: profile.name, id: profile.id, disposable: true }));
    } else if (process.argv[2] === 'verify') {
        const fixture = JSON.parse(readFileSync(manifest, 'utf8'));
        const current = await call('profiles_list');
        assert.equal(current.some(profile => profile.id === fixture.profile.id), false);
        assert.equal(existsSync(resolve(fixture.profile.gameDir, '..')), false);
        assert.deepEqual(current.map(profile => profile.id).sort(), fixture.initialIds);
        writeFileSync(join(output, 'native-deletion.json'), JSON.stringify({ realFilesRemoved: true, recordRemoved: true, originalProfilesPreserved: true, fixtureId: fixture.profile.id }, null, 2));
        console.log('Exclusão real confirmada; instâncias originais preservadas.');
    } else throw new Error('Use prepare ou verify.');
} finally { daemon.stdin.end(); }
