const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');

(async () => {
    const browser = await chromium.launch({headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE});
    const report = [];
    const output=process.env.LUXMC_ARTIFACT_DIR || 'docs/validation/performance-oct11';
    fs.mkdirSync(output,{recursive:true});
    try {
        for (const theme of ['dark', 'light']) {
            const page = await browser.newPage({viewport: {width: 1440, height: 960}});
            const errors = [];
            page.on('pageerror', error => errors.push(String(error)));
            await setupLauncherDemo(page);
            await page.addInitScript(theme => {
                window.launcherDemo.state.settings.theme = `default-${theme}`;
                localStorage.setItem('luxmc_theme', theme);
                window.launcherDemo.state.settings.motionStyle = 'expressive';
                window.launcherDemo.state.settings.liveWallpaper = false;
                const original = window.electronAPI.invoke;
                let pendingImport;
                window.installationFixture = {cancelled: false, completed: false};
                const invoke = async (command, args = {}) => {
                    if (command === 'installation_queue_get') return [];
                    if (command === 'installation_queue_save') return null;
                    if (command === 'mods_versions') return [{id:'fixture-version', name:'Fixture', versionNumber:'1', loaders:['fabric'], files:[{url:'https://cdn.modrinth.com/fixture.mrpack', filename:'fixture.mrpack', size:1, sha1:''}]}];
                    if (command === 'mods_download_to_temp') return 'C:\\Fixture\\fixture.mrpack';
                    if (command === 'pack_verify_download') return null;
                    if (command === 'instance_import_mrpack') { window.installationFixture.importing = true; return new Promise((resolve, reject) => pendingImport = {resolve, reject}); }
                    if (command === 'instance_cancel_import') { window.installationFixture.cancelled = true; pendingImport?.reject(new Error('Importação cancelada')); return null; }
                    if (command === 'instance_link_pack') { window.installationFixture.completed = true; return null; }
                    return original(command, args);
                };
                window.electronAPI.invoke = invoke;
                window.__TAURI_INTERNALS__.invoke = invoke;
                window.completeInstallation = () => pendingImport.resolve({...window.launcherDemo.state.profiles[1], id:'fixture-installed', importedNew:true});
            }, theme);
            await page.goto('http://127.0.0.1:1420/');
            await page.locator('.launcher-sidebar').waitFor();
            await page.waitForFunction(() => document.documentElement.classList.contains('settings-ready'));
            assert.equal(await page.evaluate(() => document.documentElement.classList.contains('light')), theme === 'light');
            const routes = ['/', '/instances', '/instances/demo-vp', '/friends', '/hosting', '/settings', '/skins', '/mods', '/workshop', '/news', '/screenshots', '/boost', '/logs', '/logs-history', '/organizer'];
            for (const route of routes) {
                console.log(`Checking ${theme} route ${route}`);
                const link = page.locator(`.launcher-sidebar a[href="${route}"]`).first();
                if (await link.count()) await link.click();
                else await page.goto(`http://127.0.0.1:1420${route}`);
                const expectedPath = route === '/organizer' ? '/' : route === '/boost' ? '/instances' : route;
                await page.waitForURL(url => url.pathname === expectedPath);
                await page.waitForTimeout(700);
                assert.ok(await page.locator('main').evaluate(node => node.scrollWidth <= node.clientWidth + 1), `${theme} overflow ${route}`);
                assert.equal(new URL(page.url()).pathname, expectedPath);
                assert.equal(await page.evaluate(() => document.documentElement.classList.contains('light')), theme === 'light');
            }
            await page.goto('http://127.0.0.1:1420/mods');
            await page.locator('.catalog-card').first().waitFor();
            await page.screenshot({path:`${output}/catalog-${theme}.png`});
            await page.setViewportSize({width:960, height:760});
            assert.ok(await page.locator('main').evaluate(node => node.scrollWidth <= node.clientWidth + 1));
            await page.setViewportSize({width:1440, height:960});
            const motion = await page.evaluate(async () => {
                const {pageMotion} = await import('/src/lib/actions/pageMotion.ts');
                const {catalogReveal} = await import('/src/lib/actions/catalogReveal.ts');
                const root = document.createElement('div');
                root.style.cssText = 'position:fixed;top:0;left:0;width:500px;height:800px;z-index:-1';
                for (let index=0; index<800; index++) {
                    const card = document.createElement('section');
                    card.className = 'surface-glass';
                    card.style.cssText = 'position:absolute;top:10px;width:100px;height:40px';
                    card.innerHTML = '<header><h2>Fixture</h2></header>';
                    root.append(card);
                }
                document.body.append(root);
                let reads = 0;
                const bounds = Element.prototype.getBoundingClientRect;
                Element.prototype.getBoundingClientRect = function() { if (root.contains(this)) reads++; return bounds.call(this); };
                const motion = pageMotion(root, 'fixture');
                await new Promise(resolve => setTimeout(resolve, 80));
                const animations = root.getAnimations({subtree:true}).filter(animation => animation.id === 'luxmc-page-entry').length;
                motion.destroy();
                Element.prototype.getBoundingClientRect = bounds;
                root.remove();
                const NativeObserver = window.IntersectionObserver;
                let observers = 0;
                window.IntersectionObserver = class extends NativeObserver { constructor(...args) { super(...args); observers++; } };
                const cards = Array.from({length:60}, (_, index) => {
                    const node = document.createElement('div');
                    node.style.cssText = 'position:fixed;top:10px;left:10px;width:20px;height:20px';
                    document.body.append(node);
                    return {node, action:catalogReveal(node, `observer-fixture-${index}`)};
                });
                await new Promise(resolve => setTimeout(resolve, 80));
                for (const {node, action} of cards) { action.destroy(); node.remove(); }
                window.IntersectionObserver = NativeObserver;
                return {reads, animations, observers};
            });
            assert.ok(motion.reads <= 12, `Unbounded layout work: ${motion.reads}`);
            assert.ok(motion.animations > 0 && motion.animations <= 11);
            assert.equal(motion.observers, 1);
            await page.evaluate(() => window.launcherDemo.emit('download-progress', {phase:'libraries', total:10, completed:10, currentFile:'fixture.jar', bytesDownloaded:0, totalBytes:0, speed:null}));
            assert.equal(await page.getByText('Download Concluído!', {exact:true}).count(), 0);
            await page.evaluate(() => window.launcherDemo.emit('download-progress', {phase:'done', total:1, completed:1, currentFile:'', bytesDownloaded:0, totalBytes:0, speed:null}));
            await page.getByText('Download Concluído!', {exact:true}).waitFor();
            await page.evaluate(async () => {
                const {modpackInstallation} = await import('/src/lib/stores/modpackInstallation.svelte.ts');
                await modpackInstallation.start({sourceId:'fixture', source:'modrinth', title:'Verificação de instalação', slug:'fixture', description:'Fixture', downloads:1, versions:['1.20.1'], categories:['fabric'], iconUrl:'/modpack_fo_icon.png', bannerUrl:null, author:'Fixture'}, 'Verificação de instalação', 4096, '1.20.1');
            });
            await page.locator('[data-installation-status]').waitFor();
            await page.waitForFunction(() => window.installationFixture.importing);
            await page.evaluate(() => window.launcherDemo.emit('modpack-progress', {phase:'validation', current:20, total:60, percent:96, status:'Conferindo arquivos: 20 de 60…'}));
            await page.locator('[data-installation-status] [aria-current="step"]').getByText('Conferência', {exact:true}).waitFor();
            await page.screenshot({path:`${output}/installation-${theme}.png`});
            await page.locator('[data-installation-status]').getByRole('button', {name:'Cancelar', exact:true}).click();
            await page.waitForFunction(() => window.installationFixture.cancelled);
            await page.waitForFunction(async () => { const {modpackInstallation} = await import('/src/lib/stores/modpackInstallation.svelte.ts'); return !modpackInstallation.state.busy; });
            assert.equal(await page.evaluate(() => window.installationFixture.completed), false);
            assert.equal(await page.evaluate(() => window.launcherDemo.state.profiles.some(profile => profile.id==='fixture-installed')), false);
            await page.emulateMedia({reducedMotion:'reduce'});
            const reduced = await page.evaluate(async () => {
                const {pageMotion} = await import('/src/lib/actions/pageMotion.ts');
                const node = document.createElement('div');
                document.body.append(node);
                const action = pageMotion(node,'reduced');
                await new Promise(resolve => setTimeout(resolve, 80));
                const count = node.getAnimations().length;
                action.destroy(); node.remove(); return count;
            });
            assert.equal(reduced, 0);
            assert.deepEqual(errors, []);
            report.push({theme, routes, compactCatalog:true, motion, cancellation:true, reducedMotion:true, pageErrors:errors});
            await page.close();
        }
        fs.writeFileSync(`${output}/browser.json`, JSON.stringify({fixtureOnly:true, passed:true, scenarios:report}, null, 2));
        console.log(JSON.stringify(report));
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
