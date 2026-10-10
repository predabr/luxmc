const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const output = 'docs/validation/v3.6/layout-polish';
    fs.mkdirSync(output, { recursive: true });
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1668, height: 985 } });
        await setupLauncherDemo(page);
        await page.addInitScript(() => {
            window.polishCalls = [];
            const original = window.electronAPI.invoke;
            const invoke = async (command, args) => {
                window.polishCalls.push({ command, args });
                if (command === 'virtual_lan_status') return { installed: true, supported: true, active: false, invitation: null, address: null, peers: [], error: null };
                if (command === 'versions_list') return { latestRelease: '26.3', latestSnapshot: '26.4-snapshot-very-long-version-name', versions: [
                    { id: '26.3', versionType: 'release', releaseTime: '2026-10-01' },
                    { id: '1.21.1', versionType: 'release', releaseTime: '2024-08-08' },
                    { id: '26.4-snapshot-very-long-version-name', versionType: 'snapshot', releaseTime: '2026-10-02' },
                    { id: 'b1.7.3', versionType: 'old_beta', releaseTime: '2011-07-08' }
                ] };
                return original(command, args);
            };
            window.electronAPI.invoke = invoke;
            window.__TAURI_INTERNALS__.invoke = invoke;
        });
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await page.goto('http://127.0.0.1:1420/instances');
        await page.getByRole('button', { name: /Nova Instância|Nova instância/ }).last().click();
        const dialog = page.getByRole('dialog');
        await dialog.locator('.edition-card').first().waitFor();
        assert.equal(await dialog.locator('.edition-card').count(), 2);
        const entryMotion = await dialog.locator('.edition-card').first().evaluate(card => card.getAnimations().length > 0);
        assert.ok(entryMotion, 'Edition cards should animate on entry');
        await page.waitForTimeout(800);
        await page.screenshot({ path: `${output}/edition-dark.png` });
        await dialog.getByRole('button', { name: /Java Edition/ }).click();
        await dialog.locator('.version-option').first().waitFor();
        const contrasts = [];
        for (const theme of ['default-light', 'default-dark']) {
            for (const accent of ['gold', 'cyan', 'emerald', 'rose', 'violet', 'orange', 'blue']) {
                await page.evaluate(async ({ theme, accent }) => {
                    const { themeStore } = await import('/src/lib/stores/theme.svelte.ts');
                    themeStore.setTheme(theme); themeStore.setAccent(accent);
                }, { theme, accent });
                await page.waitForTimeout(400);
                for (let tab = 0; tab < 4; tab++) {
                    const label = await dialog.locator('.version-filter').nth(tab).innerText();
                    await dialog.locator('.version-filter').nth(tab).click();
                    await page.waitForTimeout(250);
                    const ratio = await dialog.locator('.version-filter[aria-pressed="true"]').evaluate(button => {
                        const luminance = color => {
                            const values = color.match(/[\d.]+/g).slice(0, 3).map(Number).map(value => { const n = value / 255; return n <= .04045 ? n / 12.92 : ((n + .055) / 1.055) ** 2.4; });
                            return values[0] * .2126 + values[1] * .7152 + values[2] * .0722;
                        };
                        const a = luminance(getComputedStyle(button.querySelector('span')).color), b = luminance(getComputedStyle(button).backgroundColor);
                        return (Math.max(a, b) + .05) / (Math.min(a, b) + .05);
                    });
                    assert.ok(ratio >= 4.5, `${theme}/${accent}/${label}: ${ratio}`);
                    contrasts.push({ theme, accent, label, ratio });
                }
            }
        }
        await page.evaluate(async () => {
            const { themeStore } = await import('/src/lib/stores/theme.svelte.ts');
            themeStore.setTheme('default-light'); themeStore.setAccent('gold');
        });
        await dialog.locator('.version-filter').nth(1).click();
        const measurements = [];
        for (const width of [1668, 960, 640, 390]) {
            await page.setViewportSize({ width, height: 985 });
            const layout = await dialog.locator('.version-selector').evaluate(root => {
                const input = root.querySelector('input').getBoundingClientRect();
                const filters = root.querySelector('.version-filters').getBoundingClientRect();
                const longLabel = [...root.querySelectorAll('.version-label')].find(label => label.textContent.includes('very-long'));
                return { scrollWidth: root.scrollWidth, width: root.clientWidth, separateRows: filters.top >= input.bottom, longLabelFits: longLabel.scrollHeight <= longLabel.clientHeight, cardHeight: longLabel.closest('button').getBoundingClientRect().height };
            });
            assert.ok(layout.scrollWidth <= layout.width + 1, `Version overflow at ${width}`);
            assert.ok(layout.separateRows && layout.longLabelFits && layout.cardHeight >= 64);
            measurements.push({ width, ...layout });
        }
        await page.setViewportSize({ width: 960, height: 1000 });
        await page.screenshot({ path: `${output}/java-light-gold.png` });
        await page.keyboard.press('Escape');
        await page.waitForTimeout(300);
        assert.equal(await page.getByRole('dialog').count(), 0);
        await page.goto('http://127.0.0.1:1420/friends');
        await page.locator('.friend-row').first().waitFor();
        const friends = [];
        for (const width of [1668, 1440, 960, 640]) {
            await page.setViewportSize({ width, height: 985 });
            await page.waitForTimeout(700);
            const layout = await page.locator('.friends-page').evaluate(root => {
                const header = root.querySelector('.friends-header').getBoundingClientRect(), workspace = root.querySelector('.friends-workspace').getBoundingClientRect();
                return { width: root.getBoundingClientRect().width, header: header.width, workspace: workspace.width, scrollWidth: root.scrollWidth, clientWidth: root.clientWidth };
            });
            assert.ok(layout.header >= layout.width * .98 && layout.workspace >= layout.width * .98, `Shrinking friends page at ${width}`);
            assert.ok(layout.scrollWidth <= layout.clientWidth + 1, `Friends overflow at ${width}`);
            friends.push({ viewport: width, ...layout });
        }
        await page.setViewportSize({ width: 1668, height: 985 });
        await page.evaluate(async () => {
            const { themeStore } = await import('/src/lib/stores/theme.svelte.ts');
            themeStore.setTheme('default-dark');
            document.documentElement.classList.add('has-custom-wallpaper');
        });
        await page.waitForTimeout(700);
        await page.screenshot({ path: `${output}/friends-wide.png` });
        await page.locator('.friend-row').first().hover();
        await page.waitForTimeout(400);
        assert.equal(await page.locator('.friend-row').first().evaluate(row => getComputedStyle(row).translate), '0px -3px');
        await page.emulateMedia({ reducedMotion: 'reduce' });
        assert.equal(await page.locator('.friend-row').first().evaluate(row => getComputedStyle(row).translate), 'none');
        await page.getByRole('textbox', { name: 'Buscar amigos' }).fill('SteveCraft');
        await page.waitForTimeout(700);
        assert.equal(await page.locator('.friend-row').count(), 1);
        await page.getByRole('textbox', { name: 'Buscar amigos' }).fill('');
        await page.getByRole('button', { name: 'Jogar juntos', exact: true }).click();
        await page.getByRole('button', { name: 'Criar sala', exact: true }).waitFor();
        assert.deepEqual(errors, []);
        fs.writeFileSync(`${output}/launcher-browser.json`, JSON.stringify({ passed: true, contrasts, measurements, friends, editionChoice: true, editionEntrance: entryMotion, reducedMotion: true, friendHover: true, modalClose: true, friendsSearch: true, lanEntry: true }, null, 2));
        console.log('Layout checks passed: 56 active-label contrasts, four Java widths, full-width friends, search and LAN entry.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
