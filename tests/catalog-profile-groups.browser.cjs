const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {setupLauncherDemo}=require('../scripts/launcher-video-fixture.cjs');

(async()=>{
    const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE,args:['--disable-gpu']});
    try {
        fs.mkdirSync('docs/validation/catalog-profile-groups',{recursive:true});
        for(const theme of ['dark','light']) {
            const page=await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'});
            page.setDefaultTimeout(30000);
            console.log(`${theme}: opening library`);
            const errors=[];
            page.on('pageerror',error=>errors.push(String(error)));
            await setupLauncherDemo(page);
            await page.addInitScript(theme=>{
                const state=window.launcherDemo.state;
                state.settings.theme=theme==='light'?'default-light':'default-dark';
                state.settings.language='pt-BR';state.settings.languageMode='manual';
                if (!localStorage.getItem('luxmc_library_groups')) {
                    localStorage.setItem('luxmc_library_groups',JSON.stringify(['Vanilla','Modded']));
                    for (const profile of state.profiles) profile.instanceGroup = profile.loader === 'vanilla' ? 'Vanilla' : 'Modded';
                }
                window.originalGameDirs=Object.fromEntries(state.profiles.map(profile=>[profile.id,profile.gameDir]));
                window.publicFixture={id:'00000000-0000-4000-8000-000000000001',username:'LuxPlayer',role:'owner',avatarUrl:'/grass_head.png',displayName:'',status:'',description:'',banner:'',portrait:'',packs:[]};
                const original=window.electronAPI.invoke;
                const invoke=async(command,args)=>{
                    if(command==='social_request'&&args.request.action==='profile_get') return {profile:window.publicFixture};
                    if(command==='social_request'&&args.request.action==='profile_save') {window.publicFixture={...window.publicFixture,...args.request};return {ok:true};}
                    return original(command,args);
                };
                window.electronAPI.invoke=invoke;window.__TAURI_INTERNALS__.invoke=invoke;
            },theme);
            await page.goto('http://127.0.0.1:1420/',{waitUntil:'domcontentloaded'});
            console.log(`${theme}: creating a user folder`);
            assert.equal(await page.getByRole('button',{name:/^(Vanilla|Modded) \d/}).count(),0);
            await page.getByRole('button',{name:'+ Nova pasta',exact:true}).click();
            await page.getByPlaceholder('Nome da nova pasta...').fill('Adventure');
            await page.getByPlaceholder('Nome da nova pasta...').press('Enter');
            await page.locator('[data-library-folder="Adventure"]').waitFor();
            await page.evaluate(async()=>{
                const url=performance.getEntriesByType('resource').find(entry=>entry.name.includes('/stores/profiles.svelte.ts')).name;
                const {profiles}=await import(url);
                const original=profiles.list[0];
                profiles.add({...original});
                if(profiles.list.filter(profile=>profile.id===original.id).length!==1) throw new Error('Resumed installation duplicated an instance');
            });
            assert.deepEqual(await page.evaluate(()=>JSON.parse(localStorage.getItem('luxmc_library_folders_v2'))),['Adventure']);
            const originalDirs=await page.evaluate(()=>window.originalGameDirs);
            assert.equal(await page.getByLabel('Mover para grupo: Vanilla Perfected',{exact:true}).count(),0);
            const target=page.locator('[data-library-folder="Adventure"]');
            const cards=page.locator('.home-instance-card');
            assert.equal(await cards.count(),3);
            await cards.filter({has:page.getByText('Vanilla Perfected',{exact:true})}).dragTo(target);
            await page.waitForFunction(()=>window.launcherDemo.state.profiles.find(profile=>profile.name==='Vanilla Perfected').instanceGroup==='Adventure');
            await cards.filter({has:page.getByText('Fabulously Optimized',{exact:true})}).dragTo(target);
            await page.waitForFunction(()=>window.launcherDemo.state.profiles.find(profile=>profile.name==='Fabulously Optimized').instanceGroup==='Adventure');
            assert.equal(await cards.count(),1);
            assert.equal(await page.locator('.home-instance-grid [data-library-folder]').count(),1);
            assert.deepEqual(await page.evaluate(()=>Object.fromEntries(window.launcherDemo.state.profiles.map(profile=>[profile.id,profile.gameDir]))),originalDirs);
            await page.reload();
            await page.locator('[data-library-folder="Adventure"]').click();
            assert.equal(await cards.count(),2);
            assert.equal(await page.locator('[data-library-folder]').count(),0);
            await cards.filter({has:page.getByText('Vanilla Perfected',{exact:true})}).dragTo(page.locator('[data-library-back]'));
            await page.waitForFunction(()=>window.launcherDemo.state.profiles.find(profile=>profile.name==='Vanilla Perfected').instanceGroup==='');
            await page.locator('[data-library-back]').click();
            assert.equal(await cards.count(),2);
            assert.deepEqual(await page.evaluate(()=>Object.fromEntries(window.launcherDemo.state.profiles.map(profile=>[profile.id,profile.gameDir]))),originalDirs);
            await page.screenshot({path:`docs/validation/catalog-profile-groups/groups-${theme}.png`});
            await page.goto('http://127.0.0.1:1420/instances/demo-vp');
            await page.getByRole('heading',{name:'Vanilla Perfected',exact:true}).waitFor();
            assert.equal(await page.getByText('Memória alocada',{exact:true}).count(),0);
            assert.equal(await page.getByText('Arquivos de mods',{exact:true}).count(),0);
            await page.screenshot({path:`docs/validation/catalog-profile-groups/instance-clean-${theme}.png`});
            await page.goto('http://127.0.0.1:1420/friends');
            console.log(`${theme}: editing profile`);
            await page.getByRole('button',{name:'Editar meu perfil',exact:true}).first().click();
            const dialog=page.getByRole('dialog');
            await dialog.getByLabel('Nome de exibição',{exact:true}).fill('Explorer');
            await dialog.getByLabel('Status do perfil',{exact:true}).fill('Construindo mundos');
            const gif=Buffer.from('R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7','base64');
            await dialog.locator('input[type=file]').last().setInputFiles({name:'profile.gif',mimeType:'image/gif',buffer:gif});
            await dialog.locator('img[src^="data:image/gif;"]').waitFor();
            await dialog.getByRole('button',{name:'Ver prévia',exact:true}).click();
            await dialog.getByRole('heading',{name:'Explorer',exact:true}).waitFor();
            assert.equal(await dialog.getByLabel('Nome de exibição',{exact:true}).count(),0);
            await dialog.getByRole('button',{name:'Continuar editando',exact:true}).click();
            await dialog.getByRole('button',{name:'Salvar',exact:true}).click();
            await page.waitForFunction(()=>window.publicFixture.displayName==='Explorer'&&window.publicFixture.portrait.startsWith('data:image/gif;'));
            assert.equal(await page.evaluate(()=>window.publicFixture.portrait),'data:image/gif;base64,'+gif.toString('base64'));
            await dialog.getByText('Construindo mundos',{exact:true}).waitFor();
            const sideAvatar=page.locator('aside img[src^="data:image/gif;"]').first();
            await sideAvatar.waitFor();
            assert.equal(await sideAvatar.getAttribute('src'),'data:image/gif;base64,'+gif.toString('base64'));
            await page.locator('aside').first().getByText('Explorer',{exact:true}).first().waitFor({state:'attached'});
            await dialog.getByText('Dono',{exact:true}).waitFor();
            await dialog.getByRole('button',{name:'Editar meu perfil',exact:true}).click();
            await dialog.getByLabel('Nome de exibição',{exact:true}).fill('Descartar este rascunho');
            await dialog.getByRole('button',{name:'Descartar alterações',exact:true}).click();
            await dialog.getByRole('heading',{name:'Explorer',exact:true}).waitFor();
            await page.screenshot({path:`docs/validation/catalog-profile-groups/profile-${theme}.png`});
            assert.deepEqual(errors,[]);
            console.log(JSON.stringify({theme,groupsPersist:true,gameDirsPreserved:true,gifPreserved:true,profileIdentity:true}));
            await page.close();
        }
    } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exit(1)});
