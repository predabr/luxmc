const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const base = process.env.LUXMC_SITE_URL || 'http://127.0.0.1:8790/';
const output = process.env.LUXMC_SITE_MOTION_OUTPUT || 'docs/validation/site-motion-2026-10-20';

function contrast(foreground, background) {
    const luminance = color => {
        const values = color.match(/[\d.]+/g).slice(0, 3).map(Number).map(value => {
            const channel = value / 255;
            return channel <= .04045 ? channel / 12.92 : ((channel + .055) / 1.055) ** 2.4;
        });
        return values[0] * .2126 + values[1] * .7152 + values[2] * .0722;
    };
    const first = luminance(foreground), second = luminance(background);
    return (Math.max(first, second) + .05) / (Math.min(first, second) + .05);
}

async function fixtures(page) {
    await page.route('**/api/latest-release*', route => route.fulfill({ json: { tag_name: 'v4.0.0', assets: [] } }));
    await page.route('https://api.modrinth.com/**', route => route.fulfill({ json: { hits: [], total_hits: 0 } }));
    await page.route('**/api/session', route => route.fulfill({ json: { user: null, csrfToken: 'fixture' } }));
    await page.route('**/api/commerce/status', route => route.fulfill({ json: { identity: null, membership: { plan: 'free' }, checkout: { enabled: false }, orders: [] } }));
}

(async () => {
    fs.mkdirSync(output, { recursive: true });
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const evidence = { selectedCards: [], pointerSpotlight: false, keyboardTour: false, pause: false, persistedPause: false, reducedMotion: false, mobile: false, hiddenPause: false, plans: false };
    try {
        const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await fixtures(page);
        if (process.env.LUXMC_SITE_BEFORE_CSS) {
            const before = await browser.newPage({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
            await fixtures(before);
            await before.route('**/revision.css*', route => route.fulfill({ contentType: 'text/css', body: fs.readFileSync(process.env.LUXMC_SITE_BEFORE_CSS) }));
            await before.goto(base);
            await before.evaluate(() => document.fonts.ready);
            await before.locator('.product-steps').scrollIntoViewIfNeeded();
            await before.locator('.product-steps').screenshot({ path: path.join(output, 'tour-before.png') });
            const number = await before.locator('[data-product-screen][aria-pressed="true"] > span').evaluate(element => ({ text: element.textContent, color: getComputedStyle(element).color, background: getComputedStyle(element).backgroundColor }));
            assert.ok(contrast(number.color, number.background) < 1.1);
            evidence.before = number;
            await before.close();
        }
        await page.goto(base);
        await page.locator('.hero-capture.motion-surface.motion-entered').waitFor();
        await page.evaluate(() => document.fonts.ready);
        const halo = page.locator('.atmosphere-halo');
        await page.waitForFunction(() => getComputedStyle(document.querySelector('.atmosphere-halo')).animationName === 'atmosphere-drift');
        await page.screenshot({ path: path.join(output, 'home-desktop.png'), animations: 'disabled' });
        const capture = page.locator('.hero-capture');
        await capture.hover({ position: { x: 50, y: 60 } });
        await page.waitForFunction(() => document.querySelector('.hero-capture').style.getPropertyValue('--surface-x') !== '');
        assert.equal(await capture.evaluate(element => element.classList.contains('motion-pointed')), true);
        evidence.pointerSpotlight = true;
        for (const [width, height, name] of [[1440, 1000, 'desktop'], [390, 844, 'mobile']]) {
            await page.setViewportSize({ width, height });
            if (name === 'mobile') {
                await page.evaluate(() => scrollTo(0, 0));
                await page.screenshot({ path: path.join(output, 'home-mobile.png'), animations: 'disabled' });
                assert.equal(await capture.evaluate(element => getComputedStyle(element, '::after').content), 'none');
                assert.equal(await capture.evaluate(element => element.style.getPropertyValue('--surface-x')), '');
            }
            for (let index = 0; index < 4; index++) {
                const button = page.locator('[data-product-screen]').nth(index);
                await button.click();
                await page.waitForFunction(index => document.querySelectorAll('[data-product-screen]')[index].getAttribute('aria-pressed') === 'true', index);
                await page.waitForTimeout(300);
                const selected = await button.locator(':scope > span').evaluate(element => ({ text: element.textContent, color: getComputedStyle(element).color, background: getComputedStyle(element).backgroundColor, width: element.getBoundingClientRect().width }));
                assert.equal(selected.text, String(index + 1).padStart(2, '0'));
                assert.ok(contrast(selected.color, selected.background) >= 4.5, `${name} card ${index + 1} needs readable contrast`);
                assert.ok(selected.width >= 32);
                assert.equal(await page.locator('[data-product-screen][aria-pressed="true"]').count(), 1);
                await page.locator('.product-steps').screenshot({ path: path.join(output, `tour-${name}-${index + 1}.png`), animations: 'disabled' });
                evidence.selectedCards.push({ viewport: name, selected: selected.text, contrast: contrast(selected.color, selected.background) });
            }
            assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1));
        }
        evidence.mobile = true;
        await page.setViewportSize({ width: 1440, height: 1000 });
        await page.locator('[data-product-screen]').first().focus();
        await page.keyboard.press('End');
        await page.waitForFunction(() => document.querySelectorAll('[data-product-screen]')[3].getAttribute('aria-pressed') === 'true');
        assert.equal(await page.locator('[data-product-screen]').nth(3).evaluate(element => element === document.activeElement), true);
        await page.keyboard.press('ArrowRight');
        await page.waitForFunction(() => document.querySelectorAll('[data-product-screen]')[0].getAttribute('aria-pressed') === 'true');
        evidence.keyboardTour = true;
        await page.locator('#motionToggle').click();
        assert.equal(await page.locator('body').evaluate(element => element.classList.contains('motion-paused')), true);
        assert.equal(await page.locator('.motion-pointed').count(), 0);
        assert.equal(await page.locator('.motion-item').evaluateAll(elements => elements.every(element => getComputedStyle(element).opacity === '1')), true);
        evidence.pause = true;
        await page.reload();
        await page.waitForFunction(() => document.body.classList.contains('motion-paused'));
        evidence.persistedPause = true;
        await page.locator('#motionToggle').click();
        await page.emulateMedia({ reducedMotion: 'reduce' });
        await page.waitForFunction(() => document.body.classList.contains('motion-paused'));
        assert.equal(await page.locator('#motionToggle').isDisabled(), true);
        assert.equal(await halo.evaluate(element => getComputedStyle(element).animationName), 'none');
        await page.evaluate(() => scrollTo(0, 0));
        await page.screenshot({ path: path.join(output, 'home-reduced-motion.png'), animations: 'disabled' });
        evidence.reducedMotion = true;
        await page.emulateMedia({ reducedMotion: 'no-preference' });
        await page.evaluate(() => scrollTo(0, 0));
        await page.waitForFunction(() => document.querySelector('.hero-section').classList.contains('motion-visible'));
        await page.evaluate(() => {
            Object.defineProperty(document, 'hidden', { configurable: true, value: true });
            document.dispatchEvent(new Event('visibilitychange'));
        });
        assert.equal(await page.locator('body').evaluate(element => element.classList.contains('motion-paused')), true);
        await page.evaluate(() => {
            delete document.hidden;
            document.dispatchEvent(new Event('visibilitychange'));
        });
        await page.waitForFunction(() => !document.body.classList.contains('motion-paused'));
        evidence.hiddenPause = true;
        await page.goto(new URL('/planos', base).href);
        await page.locator('.plan-card.motion-surface').first().waitFor();
        await page.emulateMedia({ reducedMotion: 'reduce' });
        assert.equal(await page.locator('body').evaluate(element => element.classList.contains('motion-paused')), true);
        await page.locator('.plans-page .faq details').first().locator('summary').click();
        assert.equal(await page.locator('.plans-page .faq details').first().evaluate(element => element.open), true);
        await page.locator('.plan-grid').screenshot({ path: path.join(output, 'plans-desktop.png'), animations: 'disabled' });
        evidence.plans = true;
        assert.deepEqual(errors, []);
        fs.writeFileSync(path.join(output, 'browser.json'), JSON.stringify(evidence, null, 2));
        console.log('Website interaction regression passed: all tour numbers desktop/mobile, pointer spotlight, keyboard tour, pause persistence, reduced motion, lifecycle and plans.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
