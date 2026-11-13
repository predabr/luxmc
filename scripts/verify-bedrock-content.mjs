import { spawn, execFileSync } from 'node:child_process';
import { createInterface } from 'node:readline';
import { mkdtempSync, existsSync, writeFileSync, unlinkSync, rmdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import assert from 'node:assert/strict';
const binary = process.env.LUXMC_NATIVE_BINARY;
assert.ok(binary && process.env.LUXMC_TEST_PYTHON);
const daemon = spawn(binary, ['--daemon'], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
const waiting = new Map(); let sequence = 0;
createInterface({ input: daemon.stdout }).on('line', line => { let response; try { response = JSON.parse(line); } catch { return; } const pending = waiting.get(response.id); if (!pending) return; waiting.delete(response.id); clearTimeout(pending.timer); response.error ? pending.reject(new Error(response.error)) : pending.resolve(response.result); });
const call = (command, args = {}) => new Promise((resolvePromise, reject) => { const id = ++sequence; const timer = setTimeout(() => reject(new Error(`Native timeout: ${command}`)), 600000); waiting.set(id, { resolve: resolvePromise, reject, timer }); daemon.stdin.write(JSON.stringify({ id, command, args }) + '\n'); });
const temporary = mkdtempSync(join(tmpdir(), 'luxmc-owned-content-test-'));
const archive = join(temporary, 'resource.mcpack');
const title = 'Teste descartável Luxmc 3.6';
let fixture;
let instance;
let initial;
try {
    let state = await call('bedrock_state');
    const versions = await call('bedrock_versions');
    assert.ok(versions.some(version => version.channel === 'release'));
    instance = state.instances.find(item => item.profileId === 'managed');
    let installationPerformed = false;
    if (!instance && process.argv.includes('--install-current')) {
        const version = versions.find(version => version.channel === 'release');
        assert.ok(version);
        console.log(`Testando instalação integrada do Bedrock ${version.version}.`);
        await call('bedrock_install', { versionId: version.id, name: `Bedrock ${version.version}` });
        state = await call('bedrock_state');
        instance = state.instances.find(item => item.profileId === 'managed' && item.installationId === version.id);
        installationPerformed = true;
    }
    assert.ok(instance, 'Não há uma versão Bedrock gerenciada na biblioteca local');
    initial = await call('bedrock_instance_content', { id: instance.id });
    assert.equal(resolve(initial.directory).toLowerCase(), resolve(process.env.APPDATA, 'github/Luxmc/data/bedrock/instances', instance.id).toLowerCase());
    execFileSync(process.env.LUXMC_TEST_PYTHON, ['-c', 'import json,sys,uuid,zipfile\nmanifest={"format_version":2,"header":{"name":sys.argv[2],"description":"Disposable integration fixture","uuid":str(uuid.uuid4()),"version":[1,0,0],"min_engine_version":[1,20,0]},"modules":[{"type":"resources","uuid":str(uuid.uuid4()),"version":[1,0,0]}]}\nwith zipfile.ZipFile(sys.argv[1],"w") as archive: archive.writestr("manifest.json",json.dumps(manifest))', archive, title], { windowsHide: true });
    await call('bedrock_import_content', { id: instance.id, kind: 'resources', path: archive });
    const imported = await call('bedrock_instance_content', { id: instance.id });
    fixture = imported.entries.find(entry => entry.title === title && !initial.entries.some(existing => existing.name === entry.name));
    assert.ok(fixture);
    assert.equal(fixture.kind, 'resources');
    const fixturePath = resolve(initial.directory, 'shared/resource_packs', fixture.name.split('/')[1]);
    assert.ok(fixturePath.startsWith(resolve(initial.directory) + '\\'));
    assert.ok(existsSync(join(fixturePath, 'manifest.json')));
    await call('bedrock_delete_content', { id: instance.id, kind: fixture.kind, name: fixture.name });
    assert.equal(existsSync(fixturePath), false);
    fixture = undefined;
    const after = await call('bedrock_instance_content', { id: instance.id });
    assert.deepEqual(after.entries, initial.entries);
    const report = { nativeCommands: true, ownedInstanceDirectory: true, installationPerformed, catalogVersions: versions.length, realImport: true, realDeletion: true, existingContentPreserved: true, worldAccountCount: after.worldAccounts.length, isolated: after.isolated };
    writeFileSync(process.env.LUXMC_VALIDATION_REPORT || 'docs/validation/v3.6/upgrades/native-bedrock-content.json', JSON.stringify(report, null, 2));
    console.log(JSON.stringify(report));
} finally {
    if (fixture && instance) await call('bedrock_delete_content', { id: instance.id, kind: fixture.kind, name: fixture.name });
    if (existsSync(archive)) unlinkSync(archive);
    rmdirSync(temporary);
    daemon.stdin.end(); daemon.kill();
}
