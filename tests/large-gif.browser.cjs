const {chromium}=require('playwright-core');
const {parseGIF,decompressFrames}=require('gifuct-js');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {setupLauncherDemo}=require('../scripts/launcher-video-fixture.cjs');

(async()=>{
    const input=process.env.LUXMC_TEST_GIF;
    assert.ok(input,'Set LUXMC_TEST_GIF to a real animated GIF');
    const original=fs.readFileSync(input);
    const decode=bytes=>decompressFrames(parseGIF(bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength)),false);
    const originalFrames=decode(original);
    const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
    try {
        const page=await browser.newPage({viewport:{width:1440,height:1000}});
        page.setDefaultTimeout(60000);
        const errors=[];
        page.on('pageerror',error=>errors.push(String(error)));
        await setupLauncherDemo(page);
        await page.addInitScript(()=>{
            window.launcherDemo.state.settings.language='pt-BR';
            window.launcherDemo.state.settings.languageMode='manual';
            localStorage.setItem('luxmc_library_folders_v2',JSON.stringify(['Adventure']));
            window.publicFixture={id:'00000000-0000-4000-8000-000000000001',username:'LuxPlayer',avatarUrl:'/grass_head.png',displayName:'',status:'',description:'',banner:'',portrait:'',packs:[],collections:[]};
            const original=window.electronAPI.invoke;
            const invoke=async(command,args)=>{
                if(command==='social_request'&&args.request.action==='profile_get') return {profile:window.publicFixture};
                if(command==='social_request'&&args.request.action==='profile_save') {window.publicFixture={...window.publicFixture,...args.request};return {ok:true};}
                return original(command,args);
            };
            window.electronAPI.invoke=invoke;window.__TAURI_INTERNALS__.invoke=invoke;
        });
        const validate=data=>{
            const bytes=Buffer.from(data.split(',')[1],'base64');
            const frames=decode(bytes);
            const parsed=parseGIF(bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.byteLength));
            const loop=parsed.frames.find(frame=>frame.application?.id==='NETSCAPE2.0');
            assert.ok(loop);
            assert.equal(loop.application.blocks[1]|(loop.application.blocks[2]<<8),0);
            assert.ok(bytes.length<=700000);
            assert.equal(frames.length,originalFrames.length);
            assert.equal(frames.reduce((n,frame)=>n+frame.delay,0),originalFrames.reduce((n,frame)=>n+frame.delay,0));
            assert.notDeepEqual(frames[0].pixels,frames[frames.length-1].pixels);
            return bytes;
        };
        await page.goto('http://127.0.0.1:1420/');
        const chooser=page.waitForEvent('filechooser');
        await page.getByRole('button',{name:'Escolher imagem da pasta Adventure',exact:true}).click();
        await (await chooser).setFiles(input);
        console.log('Folder upload started');
        await page.locator('[data-folder-image]').waitFor();
        const folderData=await page.locator('[data-folder-image]').getAttribute('src');
        const optimized=validate(folderData);
        if(process.env.LUXMC_TEST_GIF_OUTPUT) fs.writeFileSync(process.env.LUXMC_TEST_GIF_OUTPUT,optimized);
        await page.reload();
        assert.equal(await page.locator('[data-folder-image]').getAttribute('src'),folderData);
        await page.goto('http://127.0.0.1:1420/friends');
        await page.getByRole('button',{name:'Editar meu perfil',exact:true}).first().click();
        const dialog=page.getByRole('dialog');
        for(const field of [0,1]){
            await dialog.locator('input[type=file]').nth(field).setInputFiles(input);
            await page.waitForFunction(()=>!document.querySelector('[role=dialog] input[type=file]').disabled);
            assert.equal(await dialog.locator('[role=alert]').count(),0,await dialog.innerText());
            assert.equal(await dialog.locator('img[src^="data:image/gif;"]').count(),field+1);
        }
        await dialog.getByRole('button',{name:'Salvar',exact:true}).click();
        await page.waitForFunction(()=>window.publicFixture.banner.startsWith('data:image/gif;')&&window.publicFixture.portrait.startsWith('data:image/gif;'));
        const saved=await page.evaluate(()=>window.publicFixture);
        validate(saved.banner);validate(saved.portrait);
        const {validatePublicProfile}=await import('../website/lib/public-profile.js');
        assert.ok(validatePublicProfile(saved));
        assert.deepEqual(errors,[]);
        console.log(JSON.stringify({inputBytes:original.length,outputBytes:optimized.length,frames:originalFrames.length,durationMs:originalFrames.reduce((n,f)=>n+f.delay,0),folderPersists:true,profileBannerAndPortrait:true,serverAccepts:true}));
    } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exit(1);});
