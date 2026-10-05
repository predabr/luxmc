const { chromium } = require('playwright-core');
const fs = require('node:fs');
(async () => {
    const browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
    try {
        const page = browser.contexts()[0].pages().find(page => page.url().includes('tauri.localhost'));
        if (!page) throw new Error('Janela Tauri não encontrada');
        await page.waitForFunction(() => Boolean(window.__TAURI_INTERNALS__), { timeout: 20000 });
        const result = await page.evaluate(async () => {
            const invoke = window.__TAURI_INTERNALS__.invoke;
            const specs = await invoke('get_system_specs');
            const report = await invoke('storage_full_report');
            const room = await invoke('tunnel_status');
            return {
                url: location.origin,
                launcherVersion: specs.launcherVersion,
                os: specs.osDistro,
                totalBytes: report.totalBytes,
                categories: report.categories.map(item => ({ category: item.category, bytes: item.bytes })),
                warnings: report.warnings?.length || 0,
                session: room?.mode || 'idle',
                applicationCounted: report.categories.some(item => item.category === 'application' && item.bytes > 0),
            };
        });
        if (!result.applicationCounted || result.totalBytes <= 0) throw new Error('Contagem nativa de armazenamento incorreta');
        await page.screenshot({ path: `docs/visual/2026-10-03-hosting/windows-${Date.now()}.png` });
        console.log(JSON.stringify(result));
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });

