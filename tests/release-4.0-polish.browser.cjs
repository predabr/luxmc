const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const base = process.env.LUXMC_BROWSER_BASE_URL || 'http://127.0.0.1:1420';
const output = process.env.LUXMC_ARTIFACT_DIR || 'docs/validation/release-4.0-polish';

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    fs.mkdirSync(output, { recursive: true });
    try {
        for (const width of [1440, 960]) {
            const page = await browser.newPage({ viewport: { width, height: 900 } });
            const errors = [];
            page.on('pageerror', error => errors.push(String(error)));
            await setupLauncherDemo(page);
            await page.addInitScript(() => {
                const invoke = window.electronAPI.invoke;
                window.polishCalls = [];
                window.electronAPI.invoke = async (command, args) => {
                    window.polishCalls.push({ command, args });
                    if (command === 'modpack_check_update' && args.profileId === 'demo-fo') return {hasUpdate:true,currentVersion:'1',latestVersion:'50',source:'modrinth',projectId:'demo-fo',versionId:'version-0'};
                    if (command === 'mods_versions') return Array.from({ length: 50 }, (_, index) => ({ id: `version-${index}`, name: `Release ${50 - index} with a long descriptive name`, versionNumber: String(50 - index), files: [] }));
                    return invoke(command, args);
                };
            });
            await page.goto(`${base}/instances/demo-fo`);
            const versions = page.locator('details').filter({ has: page.locator('summary', { hasText: 'Escolher outra versão' }) });
            await versions.locator('summary').click();
            const trigger = page.getByRole('button', { name: 'Versão do modpack', exact: true });
            await trigger.click();
            const popup = page.getByRole('listbox', { name: 'Versão do modpack', exact: true });
            await popup.waitFor();
            const bounds = await popup.boundingBox();
            const main = await page.locator('main').boundingBox();
            assert.ok(bounds.x >= main.x && bounds.x + bounds.width <= width && bounds.y >= 0 && bounds.y + bounds.height <= 900, JSON.stringify({ bounds, main, width }));
            assert.equal(await popup.evaluate(element => element.matches(':popover-open')), true);
            await popup.getByRole('option').first().press('End');
            await page.keyboard.press('Enter');
            await popup.waitFor({ state: 'hidden' });
            assert.match(await trigger.innerText(), /Release 1 with a long descriptive name/);
            await page.screenshot({ path: `${output}/versions-${width}.png` });
            await page.getByRole('link', { name: 'Personalização', exact: true }).click();
            const steve = page.locator('.studio-skin-card').filter({ hasText: 'Steve' });
            await steve.click();
            const prompt = page.getByRole('dialog', { name: 'Aplicar sua nova skin?', exact: true });
            await prompt.waitFor();
            const box = await prompt.locator(':scope > div').boundingBox();
            assert.ok(Math.abs(box.x + box.width / 2 - width / 2) < 3);
            assert.equal(await page.evaluate(() => window.polishCalls.filter(call => call.command === 'auth_save_appearance').length), 0);
            await page.screenshot({ path: `${output}/skin-confirm-${width}.png` });
            await prompt.getByRole('button', { name: 'Aplicar este visual', exact: true }).click();
            await page.waitForFunction(() => window.polishCalls.some(call => call.command === 'auth_save_appearance'));
            await prompt.waitFor({ state: 'hidden' });
            const calls = await page.evaluate(() => window.polishCalls.filter(call => call.command === 'auth_save_appearance'));
            assert.equal(calls.length, 1);
            assert.equal(calls[0].args.variant, 'classic');
            await page.goto(`${base}/instances`);
            const card = page.locator('.instance-library-card').filter({hasText:'Fabulously Optimized'});
            await card.getByRole('button',{name:'Atualização disponível',exact:true}).waitFor();
            const cardBounds = await card.boundingBox();
            const controls = await card.locator('.instance-card-controls button').evaluateAll(buttons => buttons.map(button => {const rect=button.getBoundingClientRect();return {left:rect.left,right:rect.right,bottom:rect.bottom};}));
            assert.ok(controls.every(rect => rect.left >= cardBounds.x && rect.right <= cardBounds.x + cardBounds.width && rect.bottom <= cardBounds.y + cardBounds.height),JSON.stringify({cardBounds,controls}));
            await card.screenshot({path:`${output}/instance-update-card-${width}.png`});
            assert.deepEqual(errors, []);
            await page.close();
        }
        console.log('PASS 4.0: long version list stays inside the content and viewport, keyboard selection works, centered skin confirmation applies only on explicit action at 1440/960.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
