import { mkdtempSync, rmSync, readFileSync, writeFileSync, readdirSync, cpSync, mkdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
const temporary = mkdtempSync(join(tmpdir(), 'luxmc-agent-'));
const javaHome = process.env.LUXMC_JAVA_HOME || process.env.JAVA_HOME;
const executable = name => javaHome ? join(javaHome, 'bin', process.platform === 'win32' ? `${name}.exe` : name) : name;
const dependency = resolve('src-tauri/client-agent/dependencies/asm-9.10.1.jar');
if (createHash('sha256').update(readFileSync(dependency)).digest('hex') !== 'ed825d10ab1399c8c0cb669e688cf0c8c82629b4c8399b58352b68e92ca10fcb') throw new Error('ASM checksum mismatch');
function run(name, args, cwd) {
    const result = spawnSync(executable(name), args, { stdio: 'inherit', cwd });
    if (result.error) throw result.error;
    if (result.status !== 0) throw new Error(`${name} failed: ${result.status}`);
}
function relocateClass(file) {
    const bytes = readFileSync(file);
    const parts = [bytes.subarray(0, 10)];
    let offset = 10;
    for (let index = 1; index < bytes.readUInt16BE(8); index++) {
        const start = offset;
        const tag = bytes[offset++];
        if (tag === 1) {
            const length = bytes.readUInt16BE(offset); offset += 2;
            const text = bytes.subarray(offset, offset + length).toString('latin1').replaceAll('org/objectweb/asm', 'io/github/luxmc/shaded/asm');
            const value = Buffer.from(text, 'latin1');
            const header = Buffer.alloc(3); header[0] = 1; header.writeUInt16BE(value.length, 1);
            parts.push(header, value); offset += length;
        } else {
            const sizes = {3:4,4:4,5:8,6:8,7:2,8:2,9:4,10:4,11:4,12:4,15:3,16:2,17:4,18:4,19:2,20:2};
            if (!sizes[tag]) throw new Error(`Unknown class constant ${tag}`);
            offset += sizes[tag]; parts.push(bytes.subarray(start, offset));
            if (tag === 5 || tag === 6) index++;
        }
    }
    parts.push(bytes.subarray(offset)); writeFileSync(file, Buffer.concat(parts));
}
function walk(directory) {
    for (const entry of readdirSync(directory, {withFileTypes:true})) {
        const file = join(directory, entry.name);
        if (entry.isDirectory()) walk(file);
        else if (file.endsWith('.class')) relocateClass(file);
    }
}
try {
    const sources = resolve('src-tauri/client-agent/io/github/luxmc/client');
    run('javac', ['--release', '8', '-cp', dependency, '-d', temporary, ...readdirSync(sources).filter(name=>name.endsWith('.java')).map(name=>join(sources,name))]);
    run('jar', ['xf', dependency], temporary);
    rmSync(join(temporary, 'module-info.class'), {force:true});
    walk(temporary);
    mkdirSync(join(temporary, 'io/github/luxmc/shaded'), {recursive:true});
    cpSync(join(temporary, 'org/objectweb/asm'), join(temporary, 'io/github/luxmc/shaded/asm'), {recursive:true});
    rmSync(join(temporary, 'org'), {recursive:true,force:true});
    cpSync(resolve('src-tauri/client-agent/dependencies/ASM-LICENSE.txt'), join(temporary, 'META-INF/ASM-LICENSE.txt'));
    const support = join(temporary, 'support');
    const client = join(temporary, 'io/github/luxmc/client');
    mkdirSync(join(support, 'io/github/luxmc/client'), {recursive:true});
    for (const name of readdirSync(client).filter(name=>(name.startsWith('AppearanceAgent') || name.startsWith('P2PAgent')) && name.endsWith('.class'))) {
        cpSync(join(client,name), join(support, 'io/github/luxmc/client',name));
        rmSync(join(client,name));
    }
    cpSync(join(temporary, 'io/github/luxmc/shaded'), join(support, 'io/github/luxmc/shaded'), {recursive:true});
    rmSync(join(temporary, 'io/github/luxmc/shaded'), {recursive:true});
    mkdirSync(join(support, 'META-INF'), {recursive:true});
    cpSync(resolve('src-tauri/client-agent/dependencies/ASM-LICENSE.txt'), join(support, 'META-INF/ASM-LICENSE.txt'));
    run('jar', ['--create', '--file', join(temporary, 'luxmc-appearance-support.jar'), '-C', support, '.']);
    rmSync(support, {recursive:true});
    run('jar', ['--create', '--file', resolve('src-tauri/assets/luxmc-client-agent.jar'), '--manifest', resolve('src-tauri/client-agent/MANIFEST.MF'), '-C', temporary, '.']);
} finally { rmSync(temporary, { recursive: true, force: true }); }
