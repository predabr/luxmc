const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const skin = `data:image/png;base64,${fs.readFileSync('static/alex.png').toString('base64')}`;
    try {
        for (const kind of ['offline', 'luxmc', 'microsoft']) {
            const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
            const errors = [];
            page.on('pageerror', error => errors.push(String(error)));
            await page.addInitScript(({ texture, kind }) => {
                window.calls = [];
                window.holdAppearance = true;
                window.failSync = kind === 'microsoft';
                localStorage.setItem('luxmc.locale', 'pt-BR');
                if (!sessionStorage.getItem('skinApplyFixtureReady')) {
                    localStorage.setItem('luxmc_saved_skins', JSON.stringify([{ id: 'applied-alex', name: 'Alex aplicada', url: texture, model: 'alex' }]));
                    localStorage.setItem('luxmc_selected_skin_id', 'applied-alex');
                    sessionStorage.setItem('skinApplyFixtureReady', '1');
                }
                window.fixtureAccount = JSON.parse(sessionStorage.getItem('skinApplyAccount') || 'null') || {
                    id: kind === 'offline' ? 'offline_fixture' : kind === 'luxmc' ? 'luxmc:fixture' : 'microsoft_fixture',
                    uuid: '11111111111111111111111111111111', username: 'Player', accessToken: kind === 'microsoft' ? 'x'.repeat(150) : '',
                    skinUrl: texture, skinVariant: 'slim', capeUrl: null
                };
                const invoke = async (command, args) => {
                    window.calls.push({ command, args });
                    if (command === 'app_init') return { account: window.fixtureAccount, profiles: [], activeProfileId: null, devMode: true };
                    if (command === 'auth_save_appearance') {
                        if (window.holdAppearance) await new Promise(resolve => { window.releaseAppearance = resolve; });
                        Object.assign(window.fixtureAccount, { skinUrl: args.skinUrl, skinVariant: args.variant, capeUrl: args.capeUrl });
                        sessionStorage.setItem('skinApplyAccount', JSON.stringify(window.fixtureAccount));
                        return null;
                    }
                    if (command === 'auth_change_skin') {
                        if (window.failSync) { window.failSync = false; throw Error('Falha Microsoft simulada'); }
                        return null;
                    }
                    if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
                    if (command === 'plugin:store|get') return args.key === 'current_account' ? [window.fixtureAccount, true] : [{ language: 'pt-BR', languageMode: 'manual', animations: false, liveWallpaper: false, soundscapesEnabled: false }, true];
                    if (command === 'java_scan') return { runtimes: [] };
                    if (command === 'get_system_specs') return { totalRamMb: 16384, osDistro: 'Windows 11', arch: 'x86_64' };
                    if (command === 'social_request') return { me: { id: 'fixture', username: 'Player' }, friends: [] };
                    if (['deep_links_take', 'profiles_list', 'instances_list', 'screenshots_list', 'changelog_get'].includes(command)) return [];
                    return null;
                };
                window.electronAPI = { invoke, on: () => () => {} };
                window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
                window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: () => '' };
            }, { texture: skin, kind });
            await page.route('https://**/*', route => route.abort());
            await page.goto(`${process.env.LUXMC_BASE_URL || 'http://127.0.0.1:1420'}/skins`);
            const actionBar = page.locator('.sticky').filter({ has: page.getByRole('status') }).first();
            const apply = actionBar.getByRole('button');
            await page.getByText('Sua aparência está aplicada.', { exact: true }).waitFor();
            assert.equal(await apply.isVisible(), true, kind);
            assert.equal(await apply.isEnabled(), true, kind);
            assert.ok((await apply.boundingBox()).height >= 48);
            assert.equal(await page.evaluate(() => window.calls.filter(call => call.command === 'auth_save_appearance').length), 0);
            await apply.click();
            await page.waitForFunction(() => typeof window.releaseAppearance === 'function');
            assert.equal(await apply.isDisabled(), true, kind);
            await apply.evaluate(button => { button.click(); button.click(); });
            assert.equal(await page.evaluate(() => window.calls.filter(call => call.command === 'auth_save_appearance').length), 1);
            await page.evaluate(() => { window.holdAppearance = false; window.releaseAppearance(); });
            if (kind === 'microsoft') {
                await page.getByText(/Aparência salva no launcher. Falha na sincronização Microsoft/).first().waitFor();
                assert.equal(await apply.isEnabled(), true);
                await apply.click();
                await page.getByText(/Aparência salva e skin sincronizada com Minecraft/).first().waitFor();
                assert.equal(await page.evaluate(() => window.calls.filter(call => call.command === 'auth_change_skin').length), 2);
            }
            await page.waitForFunction(() => !document.querySelector('.sticky button')?.disabled);
            await page.locator('.launcher-sidebar a[href="/instances"]').click();
            await page.waitForURL('**/instances');
            await page.locator('.launcher-sidebar a[href="/skins"]').click();
            await page.waitForURL('**/skins');
            await page.getByText('Sua aparência está aplicada.', { exact: true }).waitFor();
            assert.equal(await apply.isVisible(), true, `${kind} after navigation`);
            assert.equal(await apply.isEnabled(), true, `${kind} after navigation`);
            const before = await page.evaluate(() => window.calls.filter(call => call.command === 'auth_save_appearance').length);
            await apply.click();
            await page.waitForFunction(count => window.calls.filter(call => call.command === 'auth_save_appearance').length === count + 1, before);
            await page.waitForFunction(() => !document.querySelector('.sticky button')?.disabled);
            await page.reload();
            await page.getByText('Sua aparência está aplicada.', { exact: true }).waitFor();
            assert.equal(await apply.isVisible(), true, `${kind} after reload`);
            assert.equal(await apply.isEnabled(), true, `${kind} after reload`);
            assert.deepEqual(errors, []);
            if (kind === 'luxmc') {
                fs.mkdirSync('docs/validation', { recursive: true });
                await page.screenshot({ path: 'docs/validation/skin-apply-persistent.png' });
            }
            await page.close();
        }
        console.log('Apply remains visible and enabled for an unchanged applied skin across offline, Luxmc and Microsoft accounts, navigation and reload; saves again, blocks concurrent saves and retries Microsoft sync.');
    } finally {
        await browser.close();
    }
})().catch(error => { console.error(error); process.exit(1); });
