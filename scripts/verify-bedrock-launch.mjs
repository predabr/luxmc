import { spawn, execFileSync } from 'node:child_process';
import { createInterface } from 'node:readline';
import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import assert from 'node:assert/strict';

assert.ok(process.env.LUXMC_NATIVE_BINARY);
const daemon = spawn(process.env.LUXMC_NATIVE_BINARY, ['--daemon'], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
const requests = new Map(); let sequence = 0;
createInterface({ input: daemon.stdout }).on('line', line => { let response; try { response = JSON.parse(line); } catch { return; } const pending = requests.get(response.id); if (!pending) return; requests.delete(response.id); clearTimeout(pending.timer); response.error ? pending.reject(new Error(response.error)) : pending.resolve(response.result); });
const call = (command, args = {}) => new Promise((resolve, reject) => { const id = ++sequence; const timer = setTimeout(() => reject(new Error(`Native timeout: ${command}`)), 600000); requests.set(id, { resolve, reject, timer }); daemon.stdin.write(JSON.stringify({ id, command, args }) + '\n'); });
const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
const powershell = join(process.env.SystemRoot, 'System32/WindowsPowerShell/v1.0/powershell.exe');
const inspect = () => JSON.parse(execFileSync(powershell, ['-NoProfile', '-NonInteractive', '-Command', 'ConvertTo-Json -Compress -InputObject @(Get-CimInstance Win32_Process | Where-Object { $_.Name -match "^Minecraft(?:\\.Windows|\\.WindowsBeta)?\\.exe$" } | Select-Object Name,ProcessId,ExecutablePath)'], { encoding: 'utf8', windowsHide: true }).trim());
let launched;
const report = { processLaunched: false, visualGameplayVerified: false };
try {
    assert.equal(inspect().length, 0, 'Já existe uma sessão Bedrock aberta; teste cancelado');
    const state = await call('bedrock_state');
    const instance = state.instances.find(item => item.profileId === 'managed');
    assert.ok(instance);
    report.instanceId = instance.id;
    const result = await call('bedrock_open', { instanceId: instance.id });
    report.notice = result.notice;
    for (let attempt = 0; attempt < 36; attempt++) {
        const processes = inspect();
        if (processes.length) { assert.equal(processes.length, 1); launched = processes[0]; break; }
        await sleep(2500);
    }
    assert.ok(launched, 'O Windows não iniciou o processo Bedrock dentro do prazo');
    assert.ok(launched.ExecutablePath && /(?:WindowsApps|XboxGames)/i.test(launched.ExecutablePath));
    report.processLaunched = true;
    report.pid = launched.ProcessId;
    await sleep(20000);
    assert.ok(inspect().some(item => item.ProcessId === launched.ProcessId), 'O Bedrock encerrou após iniciar');
    report.remainedOpen20Seconds = true;
    const content = await call('bedrock_instance_content', { id: instance.id });
    report.isolatedInstanceDirectory = content.isolated;
    assert.equal(content.isolated, true);
    console.log('Bedrock integrado abriu pelo Luxmc; processo estável e pasta isolada confirmados.');
} catch (error) { report.error = error.message; console.error(error.message); process.exitCode = 1; }
finally {
    if (launched && Number.isSafeInteger(launched.ProcessId)) {
        const current = inspect().find(item => item.ProcessId === launched.ProcessId);
        if (current && current.ExecutablePath === launched.ExecutablePath) execFileSync(powershell, ['-NoProfile', '-NonInteractive', '-Command', `Stop-Process -Id ${launched.ProcessId} -ErrorAction Stop`], { windowsHide: true });
    }
    writeFileSync('docs/validation/v3.6/upgrades/native-bedrock-launch.json', JSON.stringify(report, null, 2));
    daemon.stdin.end(); daemon.kill();
}
