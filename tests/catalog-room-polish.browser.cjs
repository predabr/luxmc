const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const base = process.env.LUXMC_BROWSER_BASE_URL || 'http://127.0.0.1:1420';

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await setupLauncherDemo(page);
        await page.addInitScript(() => {
            const invoke = window.electronAPI.invoke;
            window.polishCalls = [];
            window.polishAnimations = [];
            const animate = Element.prototype.animate;
            Element.prototype.animate = function (...args) {
                const animation = animate.apply(this, args);
                if (this.matches('.catalog-card')) window.polishAnimations.push(animation);
                return animation;
            };
            window.fixtureRoom = null;
            window.electronAPI.invoke = async (command, args = {}) => {
                window.polishCalls.push({ command, args });
                if (command === 'tunnel_status') return window.fixtureRoom;
                if (command === 'host_world') {
                    window.fixtureRoom = { mode: 'host', invitation: 'LUXMC1-1234567890123456', members: [{ id: 'host', username: 'LuxPlayer', uuid: 'host', isHost: true, joinedAt: 1, avatarUrl: null }], maxPlayers: 10, roomLocked: false, worlds: [], worldReady: false, pingMs: null };
                    return window.fixtureRoom;
                }
                if (command === 'mods_search') return [{ sourceId: 'artwork-fixture', source: 'modrinth', slug: 'artwork-fixture', title: 'Pack sem galeria', description: 'Um projeto que publica somente seu ícone.', downloads: 1000, iconUrl: '/modpack_vanilla_perfected_icon.webp', bannerUrl: '/missing-banner-fixture.png', author: 'Fixture', categories: ['fabric'], versions: ['1.21.1'] }];
                return invoke(command, args);
            };
        });
        await page.goto(`${base}/mods`);
        await page.getByRole('heading', { name: 'Um novo mundo começa aqui.' }).waitFor();
        const card = page.locator('.catalog-card').filter({ hasText: 'Pack sem galeria' });
        await card.waitFor();
        await page.waitForFunction(() => {
            const image = document.querySelector('.catalog-card img.relative');
            return image?.complete && image.naturalWidth > 0 && image.style.opacity === '0.4' && getComputedStyle(image).objectFit === 'cover';
        });
        assert.equal(await page.evaluate(() => window.polishAnimations.some(animation => animation.id === 'luxmc-catalog-reveal')), true);
        fs.mkdirSync('docs/validation/local-polish', { recursive: true });
        await page.screenshot({ path: 'docs/validation/local-polish/catalog.png', animations: 'disabled' });
        await page.getByRole('button', { name: 'Abrir minhas instâncias' }).click();
        await page.waitForURL('**/instances');
        await page.locator('a[href="/friends"]').first().click();
        await page.getByRole('button', { name: 'Jogar juntos', exact: true }).first().click();
        await page.getByRole('button', { name: 'Criar minha sala' }).click();
        await page.getByRole('heading', { name: 'Sua sala está conectada' }).waitFor();
        await page.locator('a[href="/instances"]').first().click();
        await page.waitForURL('**/instances');
        await page.locator('a[href="/friends"]').first().click();
        await page.getByRole('heading', { name: 'Sua sala está conectada' }).waitFor();
        assert.equal(await page.evaluate(() => window.polishCalls.filter(call => call.command === 'host_world').length), 1);
        await page.evaluate(async () => {
            const { appState } = await import('/src/lib/stores/app.svelte.ts');
            appState.isGameRunning = true;
            appState.activeGameDetails = { profileId: 'demo-fo', name: 'Nossa partida', version: '1.21.1', loader: 'fabric' };
            window.launcherDemo.emit('game-state-change', { status: 'singleplayer', detail: 'Mundo privado', playerCount: 3 });
        });
        await page.waitForFunction(() => window.polishCalls.some(call => call.command === 'discord_set_activity' && call.args.state?.includes('3 jogadores')));
        assert.deepEqual(errors, []);
        console.log('Catalog artwork fallback, library action, room return and Discord game player count verified.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exit(1); });
