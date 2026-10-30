const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        for (const role of ['member', 'owner']) {
            const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
            await setupLauncherDemo(page);
            await page.addInitScript(role => {
                const invoke = window.electronAPI.invoke;
                const id = window.launcherDemo.state.account.id.replace('luxmc:', '');
                window.electronAPI.invoke = async (command, args = {}) => {
                    if (command === 'membership_request' && args.request.action === 'status') return { identity: { id, username: 'LuxPlayer', owner: true }, membership: { plan: 'pro', expiresAt: 1893456000, revision: 1, collections: 24, presets: 20 }, settings: { preferences: {}, presets: [], revision: 1 } };
                    if (command === 'social_request' && args.request.action === 'profile_get') return { profile: { id, username: 'LuxPlayer', role, avatarUrl: '/grass_head.png', displayName: '', status: '', description: '', banner: '', portrait: '', packs: [] } };
                    const result = await invoke(command, args);
                    if (command === 'social_request' && ['register', 'sync'].includes(args.request.action)) result.me.role = role;
                    return result;
                };
            }, role);
            await page.goto(`${process.env.LUXMC_BROWSER_BASE_URL || 'http://127.0.0.1:1420'}/`);
            await page.locator('aside button').filter({ hasText: 'LuxPlayer' }).click();
            await page.getByRole('heading', { name: 'Seu perfil', exact: true }).waitFor();
            const central = page.getByRole('dialog').getByRole('link', { name: 'Diagnósticos e manutenção', exact: true });
            if (role === 'owner') {
                await central.waitFor();
                assert.equal(await central.getAttribute('href'), '/owner-tools');
            }
            else {
                await page.getByText('Perfis sincronizados de aparência · 0/20', { exact: true }).waitFor();
                assert.equal(await central.count(), 0);
            }
            await page.close();
        }
        console.log('PASS owner central is visible only for the current verified owner, including when another Pro account has stale owner metadata.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
