import assert from 'node:assert/strict';
import {mkdtempSync, writeFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join, resolve} from 'node:path';
import {spawnSync} from 'node:child_process';

const javaHome = process.env.LUXMC_JAVA_HOME || process.env.JAVA_HOME;
if (!javaHome) throw new Error('Set LUXMC_JAVA_HOME to a JDK.');
const directory = mkdtempSync(join(tmpdir(), 'luxmc-client-readiness-'));
const executable = name => join(javaHome, 'bin', process.platform === 'win32' ? `${name}.exe` : name);
try {
    const source = join(directory, 'ReadinessProbe.java');
    const ready = join(directory, 'window.ready');
    writeFileSync(source, `
import java.nio.file.*;
public class ReadinessProbe {
    private static boolean hasThread(String name) {
        for (Thread thread : Thread.getAllStackTraces().keySet()) if (thread.getName().startsWith(name)) return true;
        return false;
    }
    public static void main(String[] args) throws Exception {
        Thread.sleep(600);
        if (hasThread("Luxmc-Client-Hook") || hasThread("AWT-EventQueue")) throw new AssertionError("Optional UI started before readiness");
        if (!hasThread("Luxmc-Client-Readiness")) throw new AssertionError("Readiness monitor missing");
        Files.write(Paths.get(args[0]), new byte[] {1});
        long deadline = System.nanoTime() + 5000000000L;
        while (!hasThread("Luxmc-Client-Hook") && System.nanoTime() < deadline) Thread.sleep(20);
        if (!hasThread("Luxmc-Client-Hook")) throw new AssertionError("Optional HUD never started after readiness");
        if (Files.exists(Paths.get(args[0]))) throw new AssertionError("Readiness marker leaked");
        System.out.println("Readiness defers optional UI and preserves activation");
        System.exit(0);
    }
}`);
    const compiled = spawnSync(executable('javac'), ['--release', '8', '-d', directory, source], {encoding: 'utf8', windowsHide: true});
    assert.equal(compiled.status, 0, compiled.stderr);
    const probe = spawnSync(executable('java'), ['-Djava.awt.headless=true', `-Dluxmc.client.windowReady=${ready}`, `-javaagent:${resolve('src-tauri/assets/luxmc-client-agent.jar')}`, '-cp', directory, 'ReadinessProbe', ready], {encoding: 'utf8', windowsHide: true, timeout: 15000});
    assert.equal(probe.status, 0, probe.stderr + probe.stdout);
    assert.match(probe.stdout, /Readiness defers optional UI and preserves activation/);
    process.stdout.write(probe.stdout);
} finally { rmSync(directory, {recursive: true, force: true}); }
