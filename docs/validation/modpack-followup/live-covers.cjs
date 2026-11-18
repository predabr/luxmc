const fs = require('node:fs');
const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../../../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');

(async () => {
    const key = /DEFAULT_CURSEFORGE_KEY: &str = "([^"]+)"/.exec(fs.readFileSync('src-tauri/src/core/mods/curseforge.rs', 'utf8'))?.[1];
    const projects = [];
    for (const id of ['285109', '392141']) {
        const options = { headers: { 'x-api-key': key }, signal: AbortSignal.timeout(20000) };
        const response = await fetch(`https://api.curseforge.com/v1/mods/${id}`, options).then(response => response.json());
        const item = response.data;
        const description = await fetch(`https://api.curseforge.com/v1/mods/${id}/description`, options).then(response => response.json());
        projects.push({ source: 'curseforge', sourceId: id, title: item.name, slug: item.slug, iconUrl: item.logo.url, bannerUrl: item.screenshots[0]?.url || null, categories: ['forge'], versions: ['1.20.1'], description: item.summary, downloads: item.downloadCount, author: item.authors[0]?.name || '', details: { id, source: 'curseforge', gallery: item.screenshots.map(image => ({ url: image.url })), body: description.data, iconUrl: item.logo.url } });
    }
    for (const id of ['l9m9tuPN', 'rQiXwLhB', 'nJ2Rn3SV']) {
        const item = await fetch(`https://api.modrinth.com/v2/project/${id}`, { signal: AbortSignal.timeout(20000) }).then(response => response.json());
        projects.push({ source: 'modrinth', sourceId: id, title: item.title, slug: item.slug, iconUrl: item.icon_url, bannerUrl: item.gallery[0]?.url || null, categories: item.loaders, versions: item.game_versions, description: item.description, downloads: item.downloads, author: '', details: { id, source: 'modrinth', gallery: item.gallery.map(image => ({ url: image.raw_url || image.url })), body: item.body, iconUrl: item.raw_icon_url || item.icon_url } });
    }
    fs.writeFileSync('docs/validation/modpack-followup/provider-artwork.json', JSON.stringify(projects, null, 2));
    const browser = await chromium.launch({ headless: true, executablePath: 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe' });
    try {
        const page = await browser.newPage({ viewport: { width: 1700, height: 1200 } });
        page.on('requestfailed', request => { if (request.resourceType() === 'image') console.log(JSON.stringify({ failedImage: request.url(), error: request.failure()?.errorText })); });
        await setupLauncherDemo(page);
        await page.route('https://**/*', route => route.request().resourceType() === 'image' ? route.continue() : route.fallback());
        await page.addInitScript(projects => {
            const invoke = window.electronAPI.invoke;
            window.electronAPI.invoke = async (command, args = {}) => command === 'mods_search' ? projects : command === 'mods_project_details' ? projects.find(project => project.sourceId === args.projectId && project.source === args.source).details : invoke(command, args);
        }, projects);
        await page.goto('http://127.0.0.1:1420/mods');
        await page.waitForFunction(() => [...document.querySelectorAll('.catalog-cover-image')].length === 5 && [...document.querySelectorAll('.catalog-cover-image')].every(image => image.complete && image.naturalWidth && image.style.opacity === '1')).catch(async error => { await page.screenshot({path:'docs/validation/modpack-followup/live-covers-debug.png'}); console.log(JSON.stringify(await page.locator('.catalog-card').evaluateAll(cards => cards.map(card => ({title:card.querySelector('h3')?.textContent, image:card.querySelector('.catalog-cover-image')?.src, loaded:card.querySelector('.catalog-cover-image')?.complete}))))); throw error; });
        await page.waitForFunction(() => [...document.querySelectorAll('.catalog-card')].find(card => card.textContent.includes('RLCraft'))?.querySelector('.catalog-cover-image')?.src.includes('BH_RL_Header'));
        await page.waitForFunction(() => [...document.querySelectorAll('.catalog-card')].find(card => card.textContent.includes('Zombie Invade'))?.querySelector('.catalog-cover-image')?.src.includes('bfcaaa3364e2ac0b2817ea44451e79025f69859e'));
        await page.screenshot({ path: 'docs/validation/modpack-followup/covers-providers.png', animations: 'disabled' });
        const covers = await page.locator('.catalog-card').evaluateAll(cards => cards.map(card => ({ title: card.querySelector('h3')?.textContent.trim(), url: card.querySelector('.catalog-cover-image')?.src, width: card.querySelector('.catalog-cover-image')?.naturalWidth, fit: getComputedStyle(card.querySelector('.catalog-cover-image')).objectFit })));
        assert.ok(covers.every(cover => cover.fit === 'cover' && cover.width > 96));
        fs.writeFileSync('docs/validation/modpack-followup/live-cover-results.json', JSON.stringify(covers, null, 2));
        console.log(JSON.stringify(covers));
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
