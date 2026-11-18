const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const { createHash } = require('node:crypto');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, locale: 'pt-BR', reducedMotion: 'reduce' });
        await page.goto('https://luxmc-r92.pages.dev/', { waitUntil: 'domcontentloaded', timeout: 60000 });
        await page.waitForFunction(() => window.LuxLatestVersion === 'v3.0.0');
        const versions = await page.locator('.live-version-tag').allTextContents();
        assert.ok(versions.length > 0 && versions.every(version => version.trim() === 'v3.0.0'));
        for (const selector of ['#heroPrimaryBtn', '#downloadExeLink']) {
            assert.equal(await page.locator(selector).getAttribute('href'), '/download/windows');
        }
        await page.screenshot({ path: 'docs/validation/v3-public-site-final.png' });
        const [download] = await Promise.all([page.waitForEvent('download'), page.locator('#heroPrimaryBtn').click()]);
        assert.equal(download.suggestedFilename(), 'Lux MC Launcher.exe');
        const bytes = fs.readFileSync(await download.path());
        const expected = JSON.parse(fs.readFileSync('docs/validation/v3-official-installer-delivery.json', 'utf8'));
        const sha256 = createHash('sha256').update(bytes).digest('hex');
        assert.equal(sha256, expected.Sha256.toLowerCase());
        const report = { version: '3.0.0', versionBadges: versions.length, filename: download.suggestedFilename(), size: bytes.length, sha256, browserDownloadMatchesGitHub: true };
        fs.writeFileSync('docs/validation/v3-public-browser-verified.json', JSON.stringify(report, null, 2));
        console.log(JSON.stringify(report));
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
