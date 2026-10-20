import { spawn } from 'node:child_process';
import { setTimeout } from 'node:timers/promises';

const base = process.env.LUXMC_SITE_URL || 'http://127.0.0.1:8790/';
let server;
async function ready() {
    try { return (await fetch(base, { signal: AbortSignal.timeout(2000) })).ok; }
    catch { return false; }
}

try {
    if (!await ready()) {
        const url = new URL(base);
        if (url.hostname !== '127.0.0.1') throw new Error('The configured website preview is unavailable.');
        server = spawn(process.execPath, ['scripts/preview-website.mjs'], { env: { ...process.env, LUXMC_SITE_PORT: url.port || '8790' }, stdio: 'inherit', windowsHide: true });
        for (let attempt = 0; attempt < 150 && !await ready(); attempt++) {
            if (server.exitCode !== null) throw new Error('Website preview exited before it was ready.');
            await setTimeout(200);
        }
        if (!await ready()) throw new Error('Website preview did not start.');
    }
    for (const test of ['tests/website-motion.browser.cjs', 'tests/website-interactions.browser.cjs', 'tests/website-languages.browser.cjs', 'tests/site-release-identity.browser.cjs', 'tests/site-session.browser.cjs']) {
        console.log(`Running ${test}`);
        const child = spawn(process.execPath, [test], { env: { ...process.env, LUXMC_SITE_URL: base }, stdio: 'inherit', windowsHide: true });
        const code = await new Promise((resolve, reject) => { child.once('error', reject); child.once('exit', resolve); });
        if (code !== 0) throw new Error(`${test} failed (${code}).`);
    }
} catch (error) {
    console.error(error);
    process.exitCode = 1;
} finally { server?.kill(); }
