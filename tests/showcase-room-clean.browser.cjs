const {chromium} = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const {setupLauncherDemo} = require('../scripts/launcher-video-fixture.cjs');

(async () => {
    const browser = await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
    const output = 'docs/validation/showcase-room-2026-10-25';
    fs.mkdirSync(output,{recursive:true});
    try {
        const clean = await browser.newPage();
        await clean.route('https://namemc.com/**',route => route.fulfill({contentType:'text/html',body:'<html><head></head><body><main id="skins">Skins</main><div class="ad-container">Ad</div><form id="login">Login</form><div id="challenge">Verification</div></body></html>'}));
        await clean.addInitScript({content:fs.readFileSync('src-tauri/src/commands/namemc-clean.js','utf8')});
        await clean.goto('https://namemc.com/minecraft-skins');
        assert.equal(await clean.locator('.ad-container').isVisible(),false);
        assert.equal(await clean.locator('#skins').isVisible(),true);
        assert.equal(await clean.locator('#login').isVisible(),true);
        assert.equal(await clean.locator('#challenge').isVisible(),true);
        await clean.evaluate(() => {const node=document.createElement('ins');node.className='adsbygoogle';node.textContent='Inserted ad';document.body.appendChild(node);});
        assert.equal(await clean.locator('ins.adsbygoogle').isVisible(),false);
        await clean.close();
        const page = await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'});
        const errors=[];page.on('pageerror',error=>errors.push(error.message));
        await setupLauncherDemo(page);
        await page.route('https://media.forgecdn.net/**',route=>route.fulfill({contentType:'image/png',body:fs.readFileSync('static/grass_block.png')}));
        await page.addInitScript(() => {
            const invoke=window.electronAPI.invoke;
            window.showcaseSave=null;
            window.roomReads=0;
            window.fixtureRoom=null;
            window.electronAPI.invoke=async(command,args={})=>{
                if(command==='social_request'&&args.request.action==='profile_get') return {profile:{id:window.launcherDemo.state.account.uuid,username:'LuxPlayer',displayName:'Explorer',status:'',description:'',banner:'',portrait:'',packs:['pixelmon','The Pixelmon Modpack'],packArtwork:[{name:'pixelmon',title:'The Pixelmon Modpack',icon:'https://media.forgecdn.net/avatars/pixelmon.png'}],collections:[],profileStyle:{accent:'brand',frame:'none',effect:'none',layout:'classic',showcaseLayout:'spotlight'},membership:{plan:'pro',expiresAt:1893456000}}};
                if(command==='mods_search'&&args.query==='Jade') return [{slug:'jade',title:'Jade',iconUrl:'https://media.forgecdn.net/avatars/jade.png',bannerUrl:null,description:'',downloads:10,author:'Fixture',categories:[],versions:[],source:'modrinth',sourceId:'jade'}];
                if(command==='social_request'&&args.request.action==='profile_save'){window.showcaseSave=args.request;return {ok:true};}
                if(command==='tunnel_status') {
                    if(++window.roomReads===2) return new Promise(resolve=>window.releaseOldRoom=()=>resolve(null));
                    return window.fixtureRoom;
                }
                if(command==='host_world') return window.fixtureRoom={mode:'host',invitation:'LUXMC-fixture',roomCode:'LUXMC-fixture',expiresAt:1893456000,pingMs:18,transport:'QUIC',localAddress:null,members:[],maxPlayers:20,roomLocked:false,worlds:[]};
                return invoke(command,args);
            };
        });
        await page.goto('http://127.0.0.1:1420/friends');
        await page.getByRole('button',{name:'Editar meu perfil',exact:true}).click();
        const dialog=page.getByRole('dialog');
        await dialog.getByRole('tab',{name:'Vitrine',exact:true}).click();
        await page.waitForFunction(()=>document.querySelector('.profile-live-preview .showcase-pack img')?.naturalWidth>0);
        assert.equal(await dialog.locator('.profile-live-preview .showcase-pack').count(),1);
        await dialog.getByLabel('Buscar favorito no catálogo').selectOption('mod');
        await dialog.locator('form input[maxlength="80"]').fill('Jade');
        await dialog.getByRole('region',{name:'Resultados para favoritos'}).getByRole('button',{name:/Jade/}).click();
        await dialog.getByRole('button',{name:'Salvar perfil',exact:true}).click();
        await page.waitForFunction(()=>window.showcaseSave!==null);
        assert.deepEqual(await page.evaluate(()=>window.showcaseSave.packs),['The Pixelmon Modpack','Jade']);
        await dialog.locator('.showcase-spotlight').waitFor();
        assert.equal(await dialog.locator('.showcase-pack').count(),2);
        assert.equal(await page.evaluate(()=>window.showcaseSave.packArtwork.find(item=>item.name==='Jade').kind),'mod');
        assert.equal(await dialog.locator('.showcase-pack span').first().innerText(),'The Pixelmon Modpack');
        await dialog.screenshot({path:`${output}/showcase-pro.png`});
        await page.goto('http://127.0.0.1:1420/hosting');
        await page.getByRole('button',{name:'Criar minha sala',exact:true}).waitFor();
        await page.waitForFunction(()=>typeof window.releaseOldRoom==='function');
        await page.getByRole('button',{name:'Criar minha sala',exact:true}).click();
        await page.getByText('Sua sala está conectada',{exact:true}).waitFor();
        await page.evaluate(()=>window.releaseOldRoom());
        await page.waitForTimeout(100);
        assert.equal(await page.getByText('Sua sala está conectada',{exact:true}).isVisible(),true);
        assert.deepEqual(errors,[]);
        await page.close();
        console.log('PASS NameMC cosmetic filter, dynamic ads, showcase canonical title and cover, public Pro layout, stale room poll protection');
    } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
