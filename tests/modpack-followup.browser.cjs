const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const base = process.env.LUXMC_BROWSER_BASE_URL || 'http://127.0.0.1:1420';
const output = process.env.LUXMC_ARTIFACT_DIR || 'docs/validation/modpack-followup';

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    fs.mkdirSync(output, { recursive: true });
    try {
        const page = await browser.newPage({ viewport: { width: 1600, height: 1000 } });
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await setupLauncherDemo(page);
        await page.addInitScript(() => {
            const invoke = window.electronAPI.invoke;
            window.followupCalls = [];
            const names = ['RLCraft', 'Zombie Invade', 'Battle Armory', 'SkyFactory 5', 'FPS modpack'];
            const versions = Array.from({ length: 55 }, (_, index) => ({ id: String(1000 - index), name: `Release ${55 - index}`, versionNumber: String(55 - index), files: [] }));
            window.electronAPI.invoke = async (command, args = {}) => {
                window.followupCalls.push({ command, args });
                if (command === 'mods_search') return names.map((title, index) => ({ title, slug: String(index), source: 'curseforge', sourceId: String(index + 1), description: 'Arte original do projeto.', downloads: 250000, iconUrl: '/modpack_fo_icon.png', bannerUrl: '/modpack_fo_icon.png', author: 'Author', categories: ['forge'], versions: ['1.20.1'] }));
                if (command === 'mods_project_details') return { id: args.projectId, source: 'curseforge', title: names[Number(args.projectId) - 1], slug: args.projectId, description: 'Project', body: '# Project', bodyType: 'markdown', downloads: 1, categories: ['forge'], loaders: ['forge'], gameVersions: ['1.20.1'], gallery: [] };
                if (command === 'mods_versions') return args.projectId === 'project' ? [{id:'v2',name:'2',versionNumber:'2',files:[]},{id:'v1',name:'1',versionNumber:'1',files:[]}] : versions.slice(0, 50);
                if (command === 'mods_version_history') return { versions: versions.slice(args.offset, args.offset + 50), nextOffset: args.offset === 0 ? 50 : null };
                if (command === 'mods_changelog') return `<h4>Changes ${args.versionId}</h4><ul><li>Previous version preserved.</li></ul><img src=x onerror=alert(1)>`;
                if (command === 'modpack_version_diff') return {available:true,added:['new.jar'],removed:[],updated:[],configuration:[],risks:[],dependencies:[],worlds:1,targetMinecraft:'1.20.1',targetLoader:'fabric'};
                if (command === 'modpack_check_update') return { hasUpdate: !window.updateDone, currentVersion: window.updateDone ? '2' : '1', latestVersion: '2', changelog: '', source: 'modrinth', projectId: 'project', versionId: 'v2' };
                if (command === 'modpack_update_atomic') return new Promise(resolve => { window.finishUpdate = () => { window.updateDone = true; resolve(); }; });
                return invoke(command, args);
            };
        });
        await page.goto(`${base}/mods`);
        await page.waitForFunction(() => [...document.querySelectorAll('.catalog-cover-image')].length === 5 && [...document.querySelectorAll('.catalog-cover-image')].every(image => {
            const style = getComputedStyle(image);
            return image.complete && image.naturalWidth && image.style.opacity === '1' && style.objectFit === 'cover' && style.paddingTop === '0px' && image.clientWidth === image.parentElement.clientWidth && image.clientHeight === image.parentElement.clientHeight;
        }));
        const coverStyles = await page.locator('.catalog-cover-image').evaluateAll(images => images.map(image => ({ fit: getComputedStyle(image).objectFit, padding: getComputedStyle(image).paddingTop, filled: image.clientWidth === image.parentElement.clientWidth && image.clientHeight === image.parentElement.clientHeight })));
        assert.ok(coverStyles.every(style => style.fit === 'cover' && style.padding === '0px' && style.filled), JSON.stringify(coverStyles));
        await page.screenshot({ path: `${output}/covers.png`, animations: 'disabled' });
        await page.locator('.catalog-card').filter({ has: page.getByText('RLCraft', { exact: true }) }).click();
        await page.getByRole('button', { name: 'Changelog', exact: true }).click();
        await page.getByRole('heading', { name: 'Changes 1000', exact: true }).waitFor();
        await page.getByRole('heading', { name: 'Changes 995', exact: true }).waitFor();
        assert.equal(await page.getByLabel('Alterações desta versão').inputValue(), '');
        for (let index = 0; index < 9; index++) await page.getByRole('button', { name: 'Carregar mais versões', exact: true }).click();
        await page.getByRole('heading', { name: 'Changes 946', exact: true }).waitFor();
        assert.equal(await page.locator('[data-changelog-version]').count(), 55);
        assert.equal(await page.locator('[data-changelog-version] [onerror]').count(), 0);
        assert.equal(await page.evaluate(() => window.followupCalls.filter(call => call.command === 'mods_version_history' && call.args.offset === 50).length), 1);
        await page.screenshot({ path: `${output}/history.png`, animations: 'disabled' });
        await page.locator('a[href="/instances/demo-fo"]').first().click();
        assert.equal(await page.evaluate(() => window.followupCalls.filter(call => call.command === 'modpack_version_diff').length),0);
        await page.getByRole('button', { name: 'Revisar atualização', exact: true }).click();
        await page.getByRole('button', { name: 'Fazer backup e aplicar', exact: true }).click();
        await page.waitForFunction(() => typeof window.finishUpdate === 'function');
        await page.evaluate(() => window.launcherDemo.emit('modpack-progress', { profileId: 'demo-fo', percent: 62, status: 'Baixando arquivos da atualização…' }));
        await page.locator('[data-modpack-update-status]').getByText('Baixando arquivos da atualização…', { exact: true }).waitFor();
        await page.getByRole('link', { name: 'Amigos', exact: true }).click();
        await page.locator('[data-modpack-update-status]').getByText('Baixando arquivos da atualização…', { exact: true }).waitFor();
        await page.locator('[data-modpack-update-status]').getByRole('button', { name: /Abrir instância/ }).click();
        await page.waitForURL('**/instances/demo-fo');
        assert.equal(await page.locator('[data-modpack-update-status] [role=progressbar]').getAttribute('aria-valuenow'), '62');
        await page.evaluate(() => window.finishUpdate());
        await page.waitForFunction(() => document.querySelector('[data-modpack-update-status] [role=progressbar]')?.getAttribute('aria-valuenow') === '100');
        await page.waitForFunction(() => ![...document.querySelectorAll('main button')].some(button => button.textContent.includes('Revisar atualização')));
        await page.locator('[data-modpack-update-status]').getByText('Modpack atualizado', { exact: true }).waitFor();
        assert.equal(await page.evaluate(() => window.followupCalls.filter(call => call.command === 'modpack_update_atomic').length), 1);
        await page.getByRole('link', { name: 'Personalização', exact: true }).click();
        await page.locator('.skin-preview-stage').waitFor();
        assert.equal(await page.locator('.skin-preview-stage').evaluate(stage => getComputedStyle(stage).backgroundImage.includes('radial-gradient')), true);
        assert.equal(await page.locator('.skin-preview-stage').evaluate(stage => getComputedStyle(stage, '::after').backgroundColor), 'rgba(0, 0, 0, 0.12)');
        await page.screenshot({ path: `${output}/personalizer.png`, animations: 'disabled' });
        assert.deepEqual(errors, []);
        console.log('Full cover layout, 55-version changelog, sanitized history, update progress across navigation, notice removal and neutral personalizer verified.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
