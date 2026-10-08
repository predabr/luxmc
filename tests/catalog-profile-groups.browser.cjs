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
                localStorage.setItem('luxmc_library_groups',JSON.stringify(['Adventure']));
                window.originalGameDirs=Object.fromEntries(state.profiles.map(profile=>[profile.id,profile.gameDir]));
                window.publicFixture={id:'00000000-0000-4000-8000-000000000001',username:'LuxPlayer',avatarUrl:'/grass_head.png',displayName:'',status:'',description:'',banner:'',portrait:'',packs:[]};
                const original=window.electronAPI.invoke;
                const invoke=async(command,args)=>{
                    if(command==='social_request'&&args.request.action==='profile_get') return {profile:window.publicFixture};
                    if(command==='social_request'&&args.request.action==='profile_save') {window.publicFixture={...window.publicFixture,...args.request};return {ok:true};}
                    return original(command,args);
                };
                window.electronAPI.invoke=invoke;window.__TAURI_INTERNALS__.invoke=invoke;
            },theme);
            await page.goto('http://127.0.0.1:1420/',{waitUntil:'domcontentloaded'});
            console.log(`${theme}: selecting group`);
            const originalDirs=await page.evaluate(()=>window.originalGameDirs);
            const selector=page.getByLabel('Mover para grupo: Vanilla Perfected',{exact:true});
            await selector.click();
            await page.getByRole('option',{name:'Adventure',exact:true}).click();
            await page.waitForFunction(()=>window.launcherDemo.state.profiles.find(profile=>profile.name==='Vanilla Perfected').instanceGroup==='Adventure');
            const target=page.getByRole('button',{name:/^Adventure/});
            const cards=page.locator('.home-instance-card');
            const second=cards.filter({has:page.getByText('Fabulously Optimized',{exact:true})});
            await second.waitFor();
            await second.dragTo(target);
            console.log(`${theme}: checking drag`);
            await page.waitForFunction(()=>window.launcherDemo.state.profiles.find(profile=>profile.name==='Fabulously Optimized').instanceGroup==='Adventure');
            assert.deepEqual(await page.evaluate(()=>Object.fromEntries(window.launcherDemo.state.profiles.map(profile=>[profile.id,profile.gameDir]))),originalDirs);
            await page.reload();
            await page.getByLabel('Mover para grupo: Vanilla Perfected',{exact:true}).waitFor();
            assert.match(await page.getByLabel('Mover para grupo: Vanilla Perfected',{exact:true}).textContent(),/Adventure/);
            assert.deepEqual(await page.evaluate(()=>Object.fromEntries(window.launcherDemo.state.profiles.map(profile=>[profile.id,profile.gameDir]))),originalDirs);
            await page.screenshot({path:`docs/validation/catalog-profile-groups/groups-${theme}.png`});
            await page.goto('http://127.0.0.1:1420/friends');
            console.log(`${theme}: editing profile`);
            await page.getByRole('button',{name:'Editar meu perfil',exact:true}).first().click();
            const dialog=page.getByRole('dialog');
            await dialog.getByLabel('Nome de exibição',{exact:true}).fill('Explorer');
            await dialog.getByLabel('Status do perfil',{exact:true}).fill('Construindo mundos');
            const gif=Buffer.from('R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7','base64');
            await dialog.locator('input[type=file]').last().setInputFiles({name:'profile.gif',mimeType:'image/gif',buffer:gif});
            await dialog.locator('img[src^="data:image/gif;"]').waitFor();
            await dialog.getByRole('button',{name:'Salvar',exact:true}).click();
            await page.waitForFunction(()=>window.publicFixture.displayName==='Explorer'&&window.publicFixture.portrait.startsWith('data:image/gif;'));
            assert.equal(await page.evaluate(()=>window.publicFixture.portrait),'data:image/gif;base64,'+gif.toString('base64'));
            await dialog.getByText('Construindo mundos',{exact:true}).waitFor();
            await page.screenshot({path:`docs/validation/catalog-profile-groups/profile-${theme}.png`});
            assert.deepEqual(errors,[]);
            console.log(JSON.stringify({theme,groupsPersist:true,gameDirsPreserved:true,gifPreserved:true,profileIdentity:true}));
            await page.close();
        }
    } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exit(1)});
