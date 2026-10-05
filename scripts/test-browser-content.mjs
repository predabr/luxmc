import { spawn } from 'node:child_process';
import { setTimeout } from 'node:timers/promises';

let server;
async function ready() {
    try { return (await fetch('http://127.0.0.1:1420')).ok; }
    catch { return false; }
}

try {
    if (!await ready()) {
        server = spawn(process.execPath, ['node_modules/vite/bin/vite.js', '--host', '127.0.0.1'], { stdio: 'inherit' });
        for (let attempt = 0; attempt < 300 && !await ready(); attempt++) {
            if (server.exitCode !== null) throw new Error('O servidor de teste encerrou antes de iniciar.');
            await setTimeout(200);
        }
        if (!await ready()) throw new Error('O servidor de teste não iniciou em 60 segundos.');
    }
    for (const test of ['tests/mods-catalog.browser.cjs', 'tests/instance-content.browser.cjs', 'tests/create-instance.browser.cjs', 'tests/instance-lab.browser.cjs', 'tests/launcher-links.browser.cjs', 'tests/skin-import.browser.cjs', 'tests/windows-launcher.browser.cjs', 'tests/hosting-storage.browser.cjs']) {
        const child = spawn(process.execPath, [test], { stdio: 'inherit', env: process.env });
        const code = await new Promise((resolve, reject) => {
            child.once('error', reject);
            child.once('exit', resolve);
        });
        if (code !== 0) throw new Error(`${test} falhou (${code}).`);
    }
} catch (error) {
    console.error(error);
    process.exitCode = 1;
} finally {
    server?.kill();
}
