const {chromium} = require('playwright-core');
const assert = require('node:assert/strict');
const {setupLauncherDemo} = require('../scripts/launcher-video-fixture.cjs');

(async () => {
    const browser = await chromium.launch({headless:true, executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
    try {
        for (const plan of ['free','plus','pro']) {
            const page = await browser.newPage({viewport:{width:1280,height:900},reducedMotion:'no-preference'});
            const errors = []; page.on('pageerror', error => errors.push(error.message));
            await setupLauncherDemo(page);
            await page.addInitScript(plan => {
                const state = window.launcherDemo.state;
                state.settings.animations = true; state.settings.respectReducedMotion = false;
                localStorage.setItem('luxmc_library_folders_v2',JSON.stringify(['Adventure']));
                window.openedPlans = 0;
                const original = window.electronAPI.invoke;
                window.electronAPI.invoke = async (command,args={}) => {
                    if (command === 'membership_request' && args.request.action === 'status') return {identity:{id:'00000000-0000-4000-8000-000000000001',username:'LuxPlayer',owner:false},membership:{plan,expiresAt:plan === 'free' ? null : 1893456000,revision:1,collections:6,presets:0},settings:{preferences:{},presets:[],revision:1}};
                    if (command === 'membership_request' && args.request.action === 'portal_ticket') return {path:'/planos',ticket:'a'.repeat(64)};
                    if (command === 'open_url') { assertUrl(args.url); window.openedPlans++; return null; }
                    const result = await original(command,args);
                    return result;
                };
                function assertUrl(url) { if (url !== 'https://luxmc-r92.pages.dev/planos#ticket='+'a'.repeat(64)) throw Error('Unexpected plans destination'); }
            },plan);
            await page.goto('http://127.0.0.1:1420/');
            await page.waitForFunction(async () => (await import('/src/lib/stores/membership.svelte.ts')).membershipState.ready);
            const folder = page.locator('[data-library-folder="Adventure"]'); await folder.waitFor();
            const chooser = page.waitForEvent('filechooser');
            await page.getByRole('button',{name:'Escolher imagem da pasta Adventure',exact:true}).click();
            const gif = Buffer.from('R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7','base64');
            await (await chooser).setFiles({name:'renamed.png',mimeType:'image/png',buffer:gif});
            if (plan === 'free') {
                const prompt = page.getByRole('dialog',{name:'Personalize com Plus ou Pro',exact:true});
                await prompt.getByText('GIFs nas pastas exigem Luxmc Plus ou Pro.',{exact:true}).waitFor();
                assert.equal(await folder.locator('[data-folder-image]').count(),0);
                assert.equal(await page.evaluate(() => localStorage.getItem('luxmc_library_folder_images_v1')),null);
                await prompt.getByRole('button',{name:'Conhecer os planos',exact:true}).click();
                await page.waitForFunction(() => window.openedPlans === 1);
                await prompt.waitFor({state:'detached'});
            } else {
                await page.waitForFunction(() => document.querySelector('[data-folder-image]')?.src.startsWith('data:image/gif;'));
                await page.reload();
                await folder.locator('[data-folder-image]').waitFor();
            }
            const feedback = await page.evaluate(async () => {
                const {appState} = await import('/src/lib/stores/app.svelte.ts');
                const node = document.createElement('button'); node.className = 'launcher-button'; node.textContent = 'Test feedback';
                node.onclick = () => { appState.isLaunching = true; };
                document.querySelector('main').append(node); node.click();
                const hasWave = Boolean(node.querySelector('.launcher-button-wave')); appState.isLaunching = false; node.remove(); return hasWave;
            });
            assert.equal(feedback,true);
            assert.deepEqual(errors,[]); await page.close();
        }
        console.log('PASS Free folder GIF blocked by file signature, Plus/Pro GIF preserved after restart, plan dialog destination, immediate launch button feedback');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
