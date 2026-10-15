const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const output = process.env.LUXMC_ARTIFACT_DIR;
    if (output) fs.mkdirSync(output, { recursive: true });
    try {
        for (const width of [1440, 960]) {
            const page = await browser.newPage({ viewport: { width, height: 900 }, reducedMotion: 'reduce' });
            await setupLauncherDemo(page, { login: true });
            await page.addInitScript(() => {
                Object.assign(window.launcherDemo.state.settings, { language: 'pt-BR', languageMode: 'manual', blur: false, animations: true, respectReducedMotion: false, performanceMode: false });
            });
            await page.goto('http://127.0.0.1:1420/');
            await page.getByRole('button', { name: 'Microsoft', exact: true }).waitFor();
            assert.equal(await page.locator('aside.login-story').count(), 0);
            assert.equal(await page.locator('section.login-story').count(), 1);
            assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), true);
            if (width === 1440) {
                assert.equal(await page.locator('.login-story').evaluate(node => getComputedStyle(node).backgroundColor), 'rgba(0, 0, 0, 0)');
                assert.equal(await page.locator('.login-story').isVisible(), true);
            }
            await page.getByText('Suas skins e capas ficam no personalizador do launcher após entrar.', { exact: true }).waitFor();
            assert.equal(await page.locator('a[href*="skin-studio"]').count(), 0);
            assert.equal(await page.evaluate(() => document.documentElement.classList.contains('reduce-motion')), false);
            if (output) await page.screenshot({ path: `${output}/login-${width}.png` });
            await page.close();
        }
        const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
        await setupLauncherDemo(page);
        await page.addInitScript(() => {
            const original = window.electronAPI.invoke;
            const invoke = async (command, args = {}) => {
                if (command === 'membership_request' && args.request.action === 'status') return new Promise((resolve, reject) => {
                    window.completeMembership = success => success ? resolve({ identity: { id: window.launcherDemo.state.account.id.replace('luxmc:', ''), username: 'LuxPlayer', owner: false }, membership: { plan: 'pro', expiresAt: 1893456000, revision: 1, collections: 24, presets: 20 }, settings: { preferences: {}, presets: [], revision: 1 } }) : reject(new Error('Service unavailable'));
                });
                return original(command, args);
            };
            window.electronAPI.invoke = invoke;
            window.__TAURI_INTERNALS__.invoke = invoke;
        });
        await page.goto('http://127.0.0.1:1420/');
        await page.locator('aside button').filter({ hasText: 'LuxPlayer' }).click();
        await page.getByRole('heading', { name: 'Consultando seu plano…', exact: true }).waitFor();
        assert.equal(await page.getByRole('heading', { name: 'Luxmc Free', exact: true }).count(), 0);
        await page.evaluate(() => window.completeMembership(false));
        await page.getByRole('heading', { name: 'Plano indisponível', exact: true }).waitFor();
        assert.equal(await page.getByRole('heading', { name: 'Luxmc Free', exact: true }).count(), 0);
        await page.getByRole('button', { name: 'Atualizar meu plano', exact: true }).click();
        await page.evaluate(() => window.completeMembership(true));
        await page.getByRole('heading', { name: 'Luxmc Pro', exact: true }).waitFor();
        if (output) await page.screenshot({ path: `${output}/membership-restored.png` });
        console.log('PASS login layout at 1440/960, restored motion preference, and membership pending/error/retry without a false Free plan.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
