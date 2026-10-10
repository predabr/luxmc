const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    fs.mkdirSync('docs/validation/v3.6/upgrades', { recursive: true });
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
        await setupLauncherDemo(page);
        await page.addInitScript(() => {
            const state = window.launcherDemo.state;
            state.profiles[0].instanceGroup = 'Pasta de teste';
            localStorage.setItem('luxmc_library_folders_v2', JSON.stringify(['Pasta de teste']));
            window.upgradeCalls = [];
            const invoke = window.electronAPI.invoke;
            const wrapped = async (command, args = {}) => {
                window.upgradeCalls.push({ command, args });
                if (command === 'bedrock_state') return { supported: true, provider: null, warning: null, instances: [{ id: 'bedrock-fixture', name: 'Bedrock de teste', profileId: 'managed', installationId: 'package' }], installations: [{ id: 'package', profileId: 'managed', profileName: 'Luxmc', name: 'Bedrock', version: '1.20.81', directory: 'C:/fixture/package' }] };
                if (command === 'bedrock_instance_content') return { directory: 'C:/fixture/bedrock', isolated: true, worldAccounts: [], worldAccount: null, entries: [{ name: 'shared/example', title: 'Recurso de teste', kind: 'resources', icon: '/grass_block.png' }] };
                if (command === 'bedrock_delete_content') return;
                if (command === 'bedrock_versions') return ['release', 'preview', 'beta'].map(channel => ({ id: channel, version: '1.20.81', channel, packageType: 'UWP', packageVersion: '1.20.8101.0' }));
                if (command === 'instance_file_tree' && args.subPath === 'mods') return Array.from({ length: 128 }, (_, i) => ({ name: `mod-${i}.jar`, path: `C:/fixture/mod-${i}.jar`, size: 100, isDir: false, icon: null, iconKey: `mod-${i}`, iconResolved: false }));
                if (command === 'instance_content_icons') return args.fileNames.map(name => ({ name, iconKey: name.replace('.jar', ''), icon: '/grass_block.png', resolved: true }));
                if (command === 'social_request' && args.request.action === 'profile_get') return { profile: { id: args.request.targetId || 'self', username: 'Original', displayName: args.request.targetId ? 'Dono personalizado' : 'Meu perfil', role: 'owner', description: '', status: '', banner: '', portrait: 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7', packs: [], collections: [] } };
                return invoke(command, args);
            };
            window.electronAPI.invoke = wrapped;
            window.__TAURI_INTERNALS__.invoke = wrapped;
        });
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await page.goto('http://127.0.0.1:1420/');
        await page.getByRole('button', { name: 'Excluir pasta Pasta de teste', exact: true }).click();
        const dialog = page.getByRole('dialog');
        await dialog.getByRole('button', { name: 'Excluir pasta', exact: true }).click();
        await page.waitForFunction(() => !JSON.parse(localStorage.getItem('luxmc_library_folders_v2')).includes('Pasta de teste'));
        assert.equal(await page.locator('[role="dialog"]').count(), 0);
        assert.equal(await page.evaluate(() => window.upgradeCalls.some(call => call.command === 'profiles_update' && call.args.input.instanceGroup === '')), true);
        assert.equal(await page.evaluate(() => window.launcherDemo.state.profiles.length), 3);
        const recent = page.locator('a[href^="/instances/"][class*="min-w-"]').first();
        await recent.click();
        await page.waitForURL(/\/instances\/[^/]+$/);
        await page.waitForFunction(() => window.upgradeCalls.filter(call => call.command === 'instance_content_icons').reduce((count, call) => count + call.args.fileNames.length, 0) >= 128);
        await page.locator('[role="tabpanel"] img[src="/grass_block.png"]').first().waitFor();
        await page.goto('http://127.0.0.1:1420/friends');
        await page.getByRole('button', { name: 'Dono personalizado', exact: true }).first().waitFor();
        assert.ok(await page.locator('.friend-row img[src^="data:image/gif"]').count() > 0);
        assert.equal(await page.locator('.friend-row img[src^="data:image/gif"]').first().evaluate(image => getComputedStyle(image).imageRendering), 'auto');
        await page.screenshot({ path: 'docs/validation/v3.6/upgrades/friends-browser.png' });
        await page.goto('http://127.0.0.1:1420/bedrock/bedrock-fixture');
        await page.getByRole('heading', { name: 'Bedrock de teste', exact: true }).waitFor();
        await page.getByRole('button', { name: /Pacotes de recursos/ }).click();
        await page.getByRole('heading', { name: 'Recurso de teste' }).waitFor();
        await page.getByRole('button', { name: 'Excluir Recurso de teste' }).click();
        await page.getByRole('dialog').getByRole('button', { name: 'Cancelar' }).click();
        await page.getByRole('button', { name: /Mundos/ }).click();
        await page.screenshot({ path: 'docs/validation/v3.6/upgrades/bedrock-browser.png' });
        await page.goto('http://127.0.0.1:1420/instances');
        await page.getByRole('button', { name: /Nova Instância|Nova instância/, exact: false }).last().click();
        await page.getByRole('button', { name: /Bedrock Edition/ }).click();
        await page.locator('.bedrock-channel').first().waitFor();
        const contrasts = [];
        for (const theme of ['default-light', 'default-dark']) {
            for (const accentTheme of ['gold', 'cyan', 'emerald', 'rose', 'violet', 'orange', 'blue']) {
                await page.evaluate(async ({ theme, accentTheme }) => { const { themeStore } = await import('/src/lib/stores/theme.svelte.ts'); themeStore.setTheme(theme); themeStore.setAccent(accentTheme); }, { theme, accentTheme });
                await page.waitForTimeout(400);
                for (const label of ['Estáveis', 'Preview', 'Betas']) {
                    await page.getByRole('button', { name: label, exact: true }).click();
                    const ratio = await page.locator('.bedrock-channel[aria-pressed="true"]').evaluate(button => {
                        const luminance = rgb => { const colors = rgb.match(/[\d.]+/g).slice(0, 3).map(Number).map(value => { const n = value / 255; return n <= .04045 ? n / 12.92 : ((n + .055) / 1.055) ** 2.4; }); return colors[0] * .2126 + colors[1] * .7152 + colors[2] * .0722; };
                        const fg = luminance(getComputedStyle(button.querySelector('span')).color), bg = luminance(getComputedStyle(button).backgroundColor); return (Math.max(fg, bg) + .05) / (Math.min(fg, bg) + .05);
                    });
                    assert.ok(ratio >= 4.5, `${theme}/${accentTheme}/${label}: ${ratio}`);
                    contrasts.push({ theme, accentTheme, label, ratio });
                }
            }
        }
        assert.deepEqual(errors, []);
        fs.writeFileSync('docs/validation/v3.6/upgrades/browser.json', JSON.stringify({ passed: true, iconRequestsBeforeScroll: 128, folderDeletion: true, recentNavigation: true, friendPublicIdentity: true, bedrockContent: true, contrasts }, null, 2));
        console.log('Upgrade browser checks passed: folders, navigation, 128 eager icons, public profile, Bedrock content and 42 theme contrasts.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
