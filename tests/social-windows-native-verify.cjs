const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
(async () => {
    const browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
    try {
        const page = browser.contexts()[0].pages().find(item => item.url().includes('tauri.localhost'));
        assert.ok(page, 'Janela Tauri aberta');
        await page.goto(new URL(page.url()).origin);
        await page.waitForFunction(() => Boolean(window.__TAURI_INTERNALS__));
        await page.locator('button[title="Adicionar amigo"]').waitFor({ timeout: 30000 });
        const sidebarWidth = await page.locator('aside').last().evaluate(element => Math.round(element.getBoundingClientRect().width));
        await page.locator('button[title="Adicionar amigo"]').click();
        await page.getByRole('heading', { name: 'Adicionar Novo Amigo' }).waitFor();
        assert.equal(new URL(page.url()).searchParams.get('tab'), 'add');
        const discordAccepted = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('discord_set_activity', {
            details: 'No Luxmc', state: 'Pronto para jogar', largeText: 'Luxmc 2.0.2', inGame: false,
            buttons: [{ label: 'Site oficial', url: 'https://luxmc-r92.pages.dev' }]
        }));
        await page.locator('aside button[title="Início"]').first().click();
        await page.waitForURL(url => url.pathname === '/');
        assert.equal(await page.getByText('Relatório Pós-Partida', { exact: true }).count(), 0);
        console.log(JSON.stringify({ native: true, sidebarWidth, addFriendNavigation: true, automaticReportAbsent: true, discordAccepted }));
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
