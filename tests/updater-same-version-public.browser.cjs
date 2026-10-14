const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const fs = require('node:fs');
const assert = require('node:assert/strict');

(async () => {
    const manifestResponse = await fetch('https://github.com/predabr/luxmc/releases/latest/download/latest.json', { cache: 'no-store' });
    assert.equal(manifestResponse.status, 200);
    const manifest = await manifestResponse.json();
    assert.equal(manifest.displayVersion, '3.6.0');
    assert.equal(manifest.revision, 2);
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
        await setupLauncherDemo(page);
        await page.route('**/latest.json', route => route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(manifest) }));
        await page.addInitScript(() => {
            window.publicUpdateCalls = [];
            const original = window.electronAPI.invoke;
            const invoke = async (command, args) => {
                window.publicUpdateCalls.push({ command, args });
                if (command === 'plugin:app|version') return '3.6.0';
                if (command === 'app_update_environment') return { mode: 'windows' };
                if (command === 'app_perform_update') throw Error('This check must not install an update.');
                return original(command, args);
            };
            window.electronAPI.invoke = invoke;
            window.__TAURI_INTERNALS__.invoke = invoke;
        });
        await page.goto('http://127.0.0.1:1421/settings');
        await page.getByRole('button', { name: 'Verificar Atualizações', exact: true }).click();
        const dialog = page.getByRole('dialog', { name: 'Uma nova versão está pronta' });
        await dialog.waitFor({ timeout: 45000 });
        assert.match(await dialog.innerText(), /3\.6\.0/);
        const calls = await page.evaluate(() => window.publicUpdateCalls);
        assert.equal(calls.some(call => call.command === 'app_perform_update'), false);
        await dialog.screenshot({ path: 'docs/validation/v3.6/publication-revision2/same-version-update.png' });
        const result = { passed: true, client: 'Previous locally compiled 3.6.0 frontend, revision 1', responseSource: 'Actual public release manifest fetched at test start, routed into the isolated browser fixture', remoteDisplayVersion: manifest.displayVersion, remoteRevision: manifest.revision, updateOffered: true, installedUpdate: false, downloadUrl: manifest.platforms['windows-x86_64'].url };
        fs.writeFileSync('docs/validation/v3.6/publication-revision2/same-version-update.json', JSON.stringify(result, null, 2));
        console.log(JSON.stringify(result));
    } finally {
        await browser.close();
    }
})().catch(error => { console.error(error); process.exitCode = 1; });
