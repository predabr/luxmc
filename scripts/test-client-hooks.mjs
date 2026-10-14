import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { delimiter, join, resolve } from 'node:path';
const root = mkdtempSync(join(tmpdir(), 'luxmc-hooks-probe-'));
const home = process.env.LUXMC_JAVA_HOME || process.env.JAVA_HOME;
const executable = name => home ? join(home, 'bin', name + (process.platform === 'win32' ? '.exe' : '')) : name;
const agent = resolve('src-tauri/assets/luxmc-client-agent.jar');
const asm = resolve('src-tauri/client-agent/dependencies/asm-9.10.1.jar');
function run(name, args) {
    const result = spawnSync(executable(name), args, { stdio: 'inherit', windowsHide: true, timeout: 30000 });
    if (result.error) throw result.error;
    if (result.status !== 0) throw new Error(`${name} failed: ${result.status}`);
}
try {
    run('javac', ['--release', '8', '-encoding', 'UTF-8', '-cp', asm, '-d', root, 'tests/java/AgentHooksProbe.java']);
    run('java', ['-Djava.awt.headless=true', '-Dluxmc.appearance.uuid=01234567-89ab-cdef-0123-456789abcdef', `-javaagent:${agent}=appearance-only`, '-cp', [root, asm].join(delimiter), 'AgentHooksProbe']);
    run('javac', ['--release', '8', '-encoding', 'UTF-8', '-cp', asm, '-d', root, 'tests/java/ResourceArchiveProbe.java']);
    run('java', ['-Djava.awt.headless=true', '-Dluxmc.resourceIndex=true', `-javaagent:${agent}=appearance-only`, '-cp', [root, asm].join(delimiter), 'ResourceArchiveProbe']);
} finally { rmSync(root, { recursive: true, force: true }); }
