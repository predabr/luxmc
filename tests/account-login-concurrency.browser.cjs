const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1400, height: 1000 }, reducedMotion: 'reduce' });
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await page.addInitScript(() => {
            window.calls = [];
            window.modalCloses = 0;
            localStorage.setItem('luxmc.locale', 'pt-BR');
            const initial = { id: 'offline_fixture', uuid: 'offline_fixture', username: 'Player', accessToken: '', skinUrl: '/steve.png' };
            window.loginResult = { id: 'ms_fixture', uuid: '11111111111111111111111111111111', username: 'MicrosoftPlayer', accessToken: 'fixture'.repeat(25), refreshToken: '', expiresAt: Math.floor(Date.now() / 1000) + 60, skinUrl: '/alex.png', skinVariant: 'slim', capeUrl: '/fixture-cape.png' };
            const invoke = async (command, args) => {
                window.calls.push({ command, args });
                if (command === 'app_init') return { account: initial, profiles: [], activeProfileId: null, devMode: true };
                if (command === 'auth_login') return new Promise(resolve => { window.resolveMicrosoft = () => resolve(window.loginResult); });
                if (command === 'lux_account_login') return new Promise(resolve => { window.resolveLux = () => resolve({ id: 'luxmc:fixture', uuid: '22222222222222222222222222222222', username: 'LuxPlayer', accessToken: '', expiresAt: 0, skinUrl: '/alex.png', skinVariant: 'slim' }); });
                if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
                if (command === 'plugin:store|get') return [{ language: 'pt-BR', languageMode: 'manual', animations: false, liveWallpaper: false }, true];
                if (['profiles_list', 'deep_links_take', 'screenshots_list', 'changelog_get'].includes(command)) return [];
                if (command === 'java_scan') return { runtimes: [] };
                if (command === 'get_system_specs') return { totalRamMb: 8192, osDistro: 'Windows 11', arch: 'x86_64' };
                return null;
            };
            window.electronAPI = { invoke, on: () => () => {} };
            window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: () => '' };
        });
        await page.route('https://**/*', route => route.abort());
        await page.goto((process.env.LUXMC_BASE_URL || 'http://127.0.0.1:1420') + '/');
        await page.locator('.launcher-sidebar').waitFor();
        const mountModal = async () => page.evaluate(async () => {
            if (window.testModal) {
                const { unmount } = await import('/node_modules/.vite/deps/svelte.js');
                await unmount(window.testModal);
                window.testModalRoot.remove();
            }
            const [{ mount }, { default: AccountLoginModal }] = await Promise.all([
                import('/node_modules/.vite/deps/svelte.js'),
                import('/src/lib/components/profile/AccountLoginModal.svelte')
            ]);
            window.testModalRoot = document.createElement('div');
            document.body.append(window.testModalRoot);
            window.testModal = mount(AccountLoginModal, {
                target: window.testModalRoot,
                props: { isOpen: true, onClose: () => { window.modalCloses++; } }
            });
        });
        await mountModal();
        const dialog = page.getByRole('dialog');
        await dialog.waitFor();
        await dialog.locator('button.launcher-tab').nth(2).click();
        await dialog.locator('button.launcher-button--microsoft').click();
        await page.waitForFunction(() => typeof window.resolveMicrosoft === 'function');
        assert.equal(await dialog.locator('.launcher-tab:disabled').count(), 3);
        assert.equal(await dialog.getByRole('button', { name: 'Fechar', exact: true }).isDisabled(), true);
        await page.evaluate(() => window.resolveMicrosoft());
        await dialog.waitFor({ state: 'hidden' });
        assert.equal(await page.evaluate(() => window.modalCloses), 1);
        assert.equal(await page.evaluate(() => window.calls.filter(call => call.command === 'auth_login').length), 1);
        const microsoftState = await page.evaluate(async () => {
            const { account } = await import('/src/lib/stores/account.svelte.ts');
            return { account: { skinUrl: account.value.skinUrl, skinVariant: account.value.skinVariant, capeUrl: account.value.capeUrl, expiresAt: account.value.expiresAt }, appearance: JSON.parse(localStorage.getItem('luxmc_active_skin_data')), expectedExpiresAt: window.loginResult.expiresAt * 1000 };
        });
        assert.deepEqual(microsoftState.account, { skinUrl: '/alex.png', skinVariant: 'slim', capeUrl: '/fixture-cape.png', expiresAt: microsoftState.expectedExpiresAt });
        assert.equal(microsoftState.appearance.id, '11111111111111111111111111111111');
        assert.equal(microsoftState.appearance.skinUrl, '/alex.png');
        assert.equal(microsoftState.appearance.type, 'alex');
        assert.equal(microsoftState.appearance.hasCape, true);
        assert.equal(microsoftState.appearance.capeType, 'custom');
        assert.equal(microsoftState.appearance.customCapeUrl, '/fixture-cape.png');
        assert.match(microsoftState.appearance.avatarUrl, /MicrosoftPlayer/);
        await mountModal();
        await dialog.waitFor();
        await dialog.getByLabel('Nickname cadastrado no site', { exact: true }).fill('LuxPlayer');
        await dialog.locator('form button[type=submit]').click();
        await dialog.getByLabel('Senha da conta Luxmc', { exact: true }).fill('test-only-password');
        await dialog.locator('form button[type=submit]').click();
        await page.waitForFunction(() => typeof window.resolveLux === 'function');
        assert.equal(await dialog.locator('.launcher-tab:disabled').count(), 3);
        assert.equal(await dialog.getByRole('button', { name: 'Fechar', exact: true }).isDisabled(), true);
        await dialog.locator('form').evaluate(form => {
            form.dispatchEvent(new SubmitEvent('submit', { bubbles: true, cancelable: true }));
            form.dispatchEvent(new SubmitEvent('submit', { bubbles: true, cancelable: true }));
        });
        assert.equal(await page.evaluate(() => window.calls.filter(call => call.command === 'lux_account_login').length), 1);
        await page.keyboard.press('Escape');
        assert.equal(await dialog.isVisible(), true);
        await page.evaluate(() => window.resolveLux());
        await dialog.waitFor({ state: 'hidden' });
        assert.equal(await page.evaluate(() => window.modalCloses), 2);
        const luxState = await page.evaluate(async () => {
            const { account } = await import('/src/lib/stores/account.svelte.ts');
            return { account: { skinUrl: account.value.skinUrl, skinVariant: account.value.skinVariant, capeUrl: account.value.capeUrl, expiresAt: account.value.expiresAt }, appearance: JSON.parse(localStorage.getItem('luxmc_active_skin_data')) };
        });
        assert.deepEqual(luxState.account, { skinUrl: '/alex.png', skinVariant: 'slim', capeUrl: null, expiresAt: 0 });
        assert.equal(luxState.appearance.id, '22222222222222222222222222222222');
        assert.equal(luxState.appearance.skinUrl, '/alex.png');
        assert.equal(luxState.appearance.type, 'alex');
        assert.equal(luxState.appearance.hasCape, false);
        assert.equal(luxState.appearance.capeType, 'none');
        assert.equal(luxState.appearance.customCapeUrl, '');
        assert.match(luxState.appearance.avatarUrl, /LuxPlayer/);
        assert.deepEqual(errors, []);
        console.log('Microsoft login closes and preserves skin/model/cape metadata, normalizes expiry seconds; Lux login blocks concurrent providers/close and duplicate submissions, resets previous account cape, preserves its own slim skin and closes.');
    } finally {
        await browser.close();
    }
})().catch(error => { console.error(error); process.exit(1); });
