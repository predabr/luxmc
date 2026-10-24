const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const base = process.env.LUXMC_SITE_BASE_URL || 'http://127.0.0.1:8790';
    const output = process.env.LUXMC_ARTIFACT_DIR;
    if (output) fs.mkdirSync(output, { recursive: true });
    try {
        for (const width of [1440, 390]) {
            const page = await browser.newPage({locale:'pt-BR', viewport: { width, height: 1000 } });
            const errors = [];
            page.on('pageerror', error => errors.push(error.message));
            await page.goto(`${base}/conta`);
            const button = page.getByRole('button', { name: 'Entrar com Microsoft', exact: true });
            await button.click();
            await page.getByRole('alert').filter({ hasText: 'O login Microsoft do site ainda precisa ser habilitado' }).waitFor();
            assert.equal(await button.isEnabled(), true);
            assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), true);
            if (output) await page.screenshot({ path: `${output}/microsoft-${width}.png`, fullPage: true });
            await page.route('**/api/microsoft/start', route => route.fulfill({ json: { url: 'https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize?state=browser-test' } }));
            await page.route('https://login.microsoftonline.com/**', route => route.fulfill({ contentType: 'text/html', body: '<h1>Microsoft authorization destination verified</h1>' }));
            await button.click();
            await page.waitForURL('https://login.microsoftonline.com/**');
            await page.getByRole('heading', { name: 'Microsoft authorization destination verified' }).waitFor();
            assert.deepEqual(errors, []);
            await page.close();
        }
        const page = await browser.newPage({locale:'pt-BR'});
        await page.route('**/api/session', route => route.fulfill({ json: { user: { id: 'test-social', username: 'MicrosoftPlayer', displayName: '', avatarUrl: '/assets/avatar-steve.png', membership: { plan: 'free' }, canManageAccount: true }, csrfToken: 'test-csrf' } }));
        await page.route('**/api/account/me', route => route.fulfill({ json: { account: { id: 'test-ms-account', username: 'MicrosoftPlayer', socialId: 'test-social', provider: 'microsoft', preferences: {}, revision: 0 } } }));
        await page.route('**/api/social/**', route => route.fulfill({ json: { friends: [], requests: [], me: { username: 'MicrosoftPlayer' } } }));
        await page.goto(`${base}/conta`);
        await page.locator('#accountNickname').filter({ hasText: 'MicrosoftPlayer' }).waitFor();
        assert.equal(await page.locator('#passwordForm').isVisible(), false);
        assert.equal(await page.locator('#microsoftSecurity').getAttribute('hidden'), null);
        assert.equal(await page.locator('#authPanel').isVisible(), false);
        console.log('PASS Microsoft website button at desktop/mobile, honest unconfigured state, authorization navigation, and Microsoft-managed password UI.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
