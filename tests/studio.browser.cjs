const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {setupLauncherDemo}=require('../scripts/launcher-video-fixture.cjs');
(async()=>{
    const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE,args:['--disable-gpu']});
    try {
        fs.mkdirSync('docs/validation/v3.2',{recursive:true});
        for(const theme of ['dark','light']) {
            const page=await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'});
            page.setDefaultTimeout(30000);
            const errors=[];page.on('pageerror',error=>errors.push(String(error)));
            await setupLauncherDemo(page,{room:true});
            await page.addInitScript(theme=>{
                const state=window.launcherDemo.state;
                state.settings.theme=theme==='light'?'default-light':'default-dark';
                state.settings.language='pt-BR';state.settings.languageMode='manual';
                if(location.pathname==='/friends')state.room=null;
                if(location.pathname==='/hosting'){
                    state.room ||= {mode:'host',members:[{id:'demo-host',username:'LuxPlayer',isHost:true},{id:'demo-guest',username:'AlexBuilder',isHost:false}],maxPlayers:10};
                    state.room.worldReady=true;
                    state.room.worlds=[{ownerId:'demo-guest',ownerUsername:'AlexBuilder',motd:'Adventure LAN',localAddress:'127.0.0.1:54321',compatibility:{mcVersion:'1.21.1',loader:'vanilla',modFingerprint:'b'.repeat(64),modCount:0}}];
                }
                const profile={id:state.account.id.slice(6),username:'LuxPlayer',avatarUrl:'/grass_head.png',displayName:'Explorer',status:'Construindo mundos',description:'Uma turma, várias aventuras.',banner:'',portrait:'',packs:[],collections:[]};
                const original=window.electronAPI.invoke;
                const invoke=async(command,args)=>{
                    if(command==='performance_history')return [];
                    if(command==='pc_recommendations')return {totalRamMb:16384,availableRamMb:8192,budgetMb:6656,items:[{profileId:'demo-vp',name:'Vanilla Perfected',icon:'/modpack_vanilla_perfected_icon.webp',samples:3,peakRamMb:2048,preparationSeconds:2.4,fitsNow:true,suggestedRamMb:2560}]};
                    if(command==='launcher_resource_sample')return {ramMb:140,cpuPercent:0.2,processes:4,sampleMilliseconds:250};
                    if(command==='dependency_graph')return [{id:'api',name:'Adventure API',file:'api.jar',version:'1',requires:[],provides:['shared'],requiredBy:['client'],missing:[],metadataKnown:true},{id:'client',name:'Adventure Client',file:'client.jar',version:'1',requires:['shared'],provides:[],requiredBy:[],missing:[],metadataKnown:true}];
                    if(command==='instance_pack_reference')return {source:'modrinth',projectId:'known-project',versionId:'pinned-42',name:'Vanilla Perfected'};
                    if(command==='social_request'&&args.request.action==='profile_get')return {profile};
                    if(command==='social_request'&&args.request.action==='profile_save'){Object.assign(profile,args.request);window.savedStudioProfile=structuredClone(profile);return {ok:true};}
                    if(command==='tunnel_room_recipe')return {name:'Adventure LAN',mcVersion:'1.21.1',loader:'vanilla',loaderVersion:null,fingerprint:'b'.repeat(64),mods:[],pack:null,unavailable:[]};
                    if(command==='room_prepare_instance'){
                        await new Promise(resolve=>setTimeout(resolve,1600));
                        const created={...state.profiles[0],id:'prepared-room',name:'Sala · Adventure LAN',gameDir:'C:\\Luxmc\\Demonstracao\\prepared-room',instanceGroup:'Salas'};
                        state.profiles.push(created);window.launcherDemo.emit('room-preparation-progress',{percent:100,name:created.name,profileId:created.id});
                        return created;
                    }
                    return original(command,args);
                };
                window.electronAPI.invoke=invoke;window.__TAURI_INTERNALS__.invoke=invoke;
            },theme);
            await page.goto('http://127.0.0.1:1420/workshop',{waitUntil:'domcontentloaded'});
            await page.getByText('Cabe na memória disponível, com margem',{exact:true}).waitFor().catch(async e=>{console.log(errors, (await page.locator('body').innerText()).slice(-6000));throw e});
            await page.getByRole('button',{name:'Medir consumo do launcher',exact:true}).click();
            await page.getByText('0.2% CPU · 140 MB · 4 processos',{exact:true}).waitFor();
            await page.getByRole('button',{name:'Ler dependências',exact:true}).click();
            const map=page.locator('section').filter({has:page.getByRole('heading',{name:'Mapa de dependências',exact:true})});
            await map.getByRole('button',{name:'Adventure API',exact:true}).waitFor();
            await map.getByRole('button',{name:'Adventure API',exact:true}).click();
            assert.match(await map.textContent(),/Afetados se este arquivo for removido.*Adventure Client/s);
            await map.screenshot({path:`docs/validation/v3.2/dependencies-${theme}.png`});
            await page.goto('http://127.0.0.1:1420/friends',{waitUntil:'domcontentloaded'});
            await page.getByRole('button',{name:'Editar meu perfil',exact:true}).first().click().catch(async e=>{console.log(errors,(await page.locator('body').innerText()).slice(-6000));throw e});
            const dialog=page.getByRole('dialog');
            const collections=dialog.locator('section').filter({has:page.getByRole('heading',{name:'Coleções de modpacks',exact:true})});
            await collections.getByLabel('Nome da coleção',{exact:true}).fill('Co-op do fim de semana');
            await collections.getByRole('button',{name:'Adicionar',exact:true}).click();
            await collections.getByRole('button',{name:'Escolher instância vinculada',exact:true}).click();
            await collections.getByRole('option',{name:'Vanilla Perfected',exact:true}).click();
            await collections.getByRole('button',{name:'Adicionar pack vinculado',exact:true}).click();
            await collections.getByText(/known-project · pinned-42/).waitFor();
            await dialog.getByRole('button',{name:'Salvar',exact:true}).click();
            await page.waitForFunction(()=>window.savedStudioProfile.collections[0].entries[0].versionId==='pinned-42');
            await dialog.screenshot({path:`docs/validation/v3.2/collections-${theme}.png`});
            await dialog.getByRole('button',{name:'Fechar',exact:true}).click();
            await page.goto('http://127.0.0.1:1420/hosting',{waitUntil:'domcontentloaded'});
            await page.getByRole('button',{name:'Ver receita da sala',exact:true}).click();
            await page.getByRole('button',{name:'Preparar minha instância',exact:true}).click();
            await page.locator('a[href="/settings"]').first().click();
            await page.getByRole('link',{name:'Abrir instância preparada',exact:true}).waitFor();
            const paths=await page.evaluate(()=>window.launcherDemo.state.profiles.map(profile=>profile.gameDir));
            assert.equal(new Set(paths).size,paths.length);
            assert.ok(await page.evaluate(()=>window.launcherDemo.state.profiles.some(profile=>profile.id==='prepared-room')));
            assert.deepEqual(errors,[]);
            console.log(JSON.stringify({theme,measuredRecommendations:true,dependencyImpact:true,pinnedCollection:true,roomPreparationPersists:true,separateDirectories:true}));
            await page.close();
        }
    }finally{await browser.close();}
})().catch(error=>{console.error(error);process.exit(1);});
