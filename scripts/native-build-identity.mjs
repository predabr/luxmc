import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import { setTimeout } from 'node:timers/promises';
import { chromium } from 'playwright-core';
import assert from 'node:assert/strict';

export async function nativeBuildIdentity(binary, expectedRevision) {
    const reservation = createServer();
    await new Promise((resolve, reject) => {
        reservation.once('error', reject);
        reservation.listen(0, '127.0.0.1', resolve);
    });
    const port = reservation.address().port;
    await new Promise(resolve => reservation.close(resolve));
    const app = spawn(binary, [], {
        windowsHide: true,
        stdio: ['ignore', 'ignore', 'pipe'],
        env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}` }
    });
    let failure;
    let stderr = '';
    app.stderr.on('data', chunk => { stderr = (stderr + chunk).slice(-2000); });
    app.on('error', error => { failure = error; });
    let browser;
    try {
        const deadline = Date.now() + 60000;
        let ready = false;
        while (Date.now() < deadline) {
            if (failure) throw failure;
            if (app.exitCode !== null) throw new Error(`Native UI exited before identity verification: ${app.exitCode}`);
            try {
                const response = await fetch(`http://127.0.0.1:${port}/json/version`, { signal: AbortSignal.timeout(1000) });
                if (response.ok) { ready = true; break; }
            } catch {}
            await setTimeout(200);
        }
        assert.ok(ready, `Native WebView2 did not expose its debugging endpoint: ${stderr}`);
        browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`, { timeout: 10000 });
        let page;
        while (Date.now() < deadline && !page) {
            page = browser.contexts().flatMap(context => context.pages()).find(page => !page.isClosed());
            if (!page) await setTimeout(100);
        }
        assert.ok(page, 'Native Tauri page must be available');
        await page.waitForFunction(() => typeof window.__TAURI_INTERNALS__?.invoke === 'function');
        const environment = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('app_update_environment'));
        assert.equal(environment.mode, 'windows');
        assert.equal(environment.revision, expectedRevision);
        return environment;
    } finally {
        await browser?.close().catch(() => {});
        if (!failure && app.exitCode === null && !app.killed) app.kill();
    }
}
