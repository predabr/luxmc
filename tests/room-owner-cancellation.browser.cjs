const {chromium} = require('playwright-core');
const {setupLauncherDemo} = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const base = process.env.LUXMC_BROWSER_BASE_URL || 'http://127.0.0.1:1420';

(async () => {
    const browser = await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
    const output = 'docs/validation/v3.6/room-owner-cancellation';
    fs.mkdirSync(output,{recursive:true});
    try {
        const page = await browser.newPage({viewport:{width:1600,height:1050}});
        await page.route(/https:\/\/(mc-heads\.net|minotar\.net|crafatar\.com)\//,route => route.abort());
        await setupLauncherDemo(page);
        await page.addInitScript(() => {
            const state = window.launcherDemo.state;
            Object.assign(state.settings,{customBackground:'custom',customWallpaperUrl:'/vanilla_banner.png',animations:true,respectReducedMotion:false,performanceMode:false});
            window.roomCalls = [];
            window.fixtureRoom = null;
            window.fixtureCancel = false;
            window.fixtureTags = JSON.parse(sessionStorage.getItem('fixture-tags') || '[]');
            window.fixtureDeleted=false;
            window.fixtureOwnerTags=[];
            window.fixtureOwner=sessionStorage.getItem('fixture-owner') !== 'false';
            const invoke = window.electronAPI.invoke;
            window.electronAPI.invoke = async (command,args={}) => {
                window.roomCalls.push(command);
                if (command === 'tunnel_status') return window.fixtureRoom;
                if (command === 'host_world') {
                    window.fixtureRoom = {mode:'host',invitation:'LUXMC1-1234567890123456',roomCode:'LUXMC1-1234567890123456',localAddress:null,expiresAt:9999999999,pingMs:null,transport:'QUIC',members:[{id:'host',username:'LuxPlayer',uuid:'host',avatarUrl:null,joinedAt:1,isHost:true},{id:'friend',username:'Alex',uuid:'friend',avatarUrl:null,joinedAt:2,isHost:false}],maxPlayers:10,roomLocked:false,worldReady:true,worlds:[{ownerId:'friend',ownerUsername:'Alex',motd:'Nosso mundo',localAddress:'127.0.0.1:25565',compatibility:{mcVersion:'1.21.1',loader:'fabric',modFingerprint:'a'.repeat(64),modCount:8}}]};
                    return window.fixtureRoom;
                }
                if (command === 'stop_session') {window.fixtureRoom=null;return;}
                if (command === 'tunnel_set_locked') {window.fixtureRoom.roomLocked=args.locked;return;}
                if (command === 'social_request' && args.request.action === 'profile_get') return {profile:{id:args.request.targetId || state.account.uuid,username:args.request.targetId ? 'Alex' : 'LuxPlayer',role:args.request.targetId || !window.fixtureOwner ? 'member' : 'owner',displayName:args.request.targetId ? 'Alex' : 'Lux Creator',status:'Criando mundos',description:'',banner:'',portrait:'',packs:[],collections:[],tags:args.request.targetId ? window.fixtureTags : window.fixtureOwnerTags}};
                if (command === 'social_request' && args.request.action === 'owner_users') return {stats:{total:123,online:17,playing:9},users:window.fixtureDeleted ? [] : [{id:'00000000-0000-4000-8000-000000000010',username:'Alex',displayName:'Alex',avatarUrl:null,status:'online',lastSeen:null,tags:window.fixtureTags}],total:window.fixtureDeleted ? 0 : 1,nextOffset:null};
                if (command === 'social_request' && args.request.action === 'owner_tags') {window.fixtureTags=args.request.tags;sessionStorage.setItem('fixture-tags',JSON.stringify(window.fixtureTags));return {ok:true,tags:window.fixtureTags};}
                if (command === 'social_request' && args.request.action === 'owner_delete') {window.fixtureDeleted=true;window.fixtureDeleteRequest=args.request;return {ok:true};}
                if (command === 'instance_begin_import') return 'fixture-operation';
                if (command === 'instance_end_import') return;
                if (command === 'mods_versions') return [{id:'pack-version',versionNumber:'1',loaders:['fabric'],gameVersions:['1.21.1'],files:[{url:'https://cdn.modrinth.com/demo.mrpack',filename:'demo.mrpack',size:10,sha1:''}]}];
                if (command === 'mods_download_to_temp') return 'C:/Luxmc/Fixture/demo.mrpack';
                if (command === 'verify_pack_download') return;
                if (command === 'instance_import_mrpack') {window.fixtureFinalPhase=true;await new Promise(resolve => window.finishFixtureImport=resolve);return window.fixtureExistingProfile ? {...state.profiles[0],importedNew:false} : {...state.profiles[0],id:'cancelled-import',name:'Cancelled pack',importedNew:true};}
                if (command === 'instance_cancel_import') {window.fixtureCancel=true;window.finishFixtureImport?.();return;}
                if (command === 'profiles_delete' && args.id === 'cancelled-import') {window.fixtureRollback=true;return;}
                if (command === 'profiles_delete' && window.fixtureExistingProfile) {window.fixtureExistingDeleted=true;return;}
                if (command === 'social_request' && args.request.action === 'sync') {const result=await invoke(command,args);result.me.role=window.fixtureOwner ? 'owner' : 'member';result.me.tags=window.fixtureOwnerTags;result.friends=result.friends.map(friend=>({...friend,role:'member',tags:friend.username.startsWith('Alex')?window.fixtureTags:[]}));return result;}
                return invoke(command,args);
            };
        });
        const errors = [];
        page.on('pageerror',error => errors.push(error.message));
        await page.goto(`${base}/hosting`);
        await page.getByRole('button',{name:'Criar minha sala'}).click();
        await page.getByText('Sua sala está conectada').waitFor();
        await page.getByText('Nosso mundo',{exact:false}).waitFor();
        assert.equal(await page.getByText('LAN disponível',{exact:true}).count(),1);
        assert.equal(await page.getByText('Conexão direta',{exact:false}).count(),0);
        assert.ok((await page.getByText('Criador da sala',{exact:true}).count()) === 1);
        await page.getByRole('button',{name:'Fechar novas entradas'}).click();
        await page.getByRole('button',{name:'Permitir entradas'}).waitFor();
        await page.waitForFunction(() => [...document.querySelectorAll('img[alt="Avatar de Alex"]')].some(image => image.complete && image.naturalWidth > 0));
        await page.waitForTimeout(600);
        await page.screenshot({path:`${output}/room.png`});
        await page.getByRole('button',{name:'Amigos',exact:true}).click();
        await page.waitForURL('**/friends');
        await page.waitForTimeout(1000);
        assert.ok(page.url().endsWith('/friends'));
        await page.goto(`${base}/settings`);
        await page.locator('#settings-panel').waitFor();
        await page.evaluate(() => document.documentElement.classList.add('has-custom-wallpaper'));
        const settingsBackground = await page.locator('#settings-panel').evaluate(element => getComputedStyle(element).backgroundColor);
        assert.equal(settingsBackground,'rgba(0, 0, 0, 0)');
        await page.goto(`${base}/instances/demo-fo`);
        await page.locator('[role="tabpanel"]').first().waitFor();
        await page.evaluate(() => document.documentElement.classList.add('has-custom-wallpaper'));
        const instanceBackground = await page.locator('[role="tabpanel"]').first().evaluate(element => getComputedStyle(element).backgroundColor);
        assert.equal(instanceBackground,'rgba(0, 0, 0, 0)');
        await page.goto(`${base}/owner`);
        await page.getByText('Perfis cadastrados',{exact:true}).waitFor();
        await page.getByRole('button',{name:'+ Testador',exact:true}).click();
        await page.getByRole('button',{name:'Remover: Testador',exact:true}).waitFor();
        await page.screenshot({path:`${output}/owner.png`});
        await page.goto(`${base}/friends`);
        await page.locator('.friend-row').filter({hasText:'Alex'}).getByText('Testador',{exact:true}).waitFor();
        await page.goto(`${base}/owner`);
        await page.getByRole('button',{name:'Remover: Testador',exact:true}).waitFor();
        await page.getByRole('button',{name:'Excluir perfil de Alex',exact:true}).click();
        const confirmation=page.getByRole('dialog',{name:'Excluir perfil da comunidade'});
        assert.equal(await confirmation.getByRole('button',{name:'Excluir perfil',exact:true}).isEnabled(),false);
        await confirmation.getByRole('textbox').fill('Wrong');
        assert.equal(await confirmation.getByRole('button',{name:'Excluir perfil',exact:true}).isEnabled(),false);
        await confirmation.getByRole('textbox').fill('Alex');
        await confirmation.getByRole('button',{name:'Excluir perfil',exact:true}).click();
        await confirmation.waitFor({state:'hidden'});
        await page.getByText('Nenhum perfil encontrado.',{exact:true}).waitFor();
        assert.equal((await page.evaluate(()=>window.fixtureDeleteRequest)).confirmation,'Alex');
        const cancellation = await page.evaluate(async () => {
            const {modpackInstallation} = await import('/src/lib/stores/modpackInstallation.svelte.ts');
            await modpackInstallation.start({sourceId:'fixture',source:'modrinth',title:'Cancellation fixture',slug:'fixture',description:'',downloads:1,author:null,versions:['1.21.1'],categories:['fabric'],iconUrl:null,bannerUrl:null},'Cancellation fixture',4096,'1.21.1');
            const deadline = Date.now()+10000;
            while (!window.fixtureFinalPhase && Date.now()<deadline) await new Promise(resolve => setTimeout(resolve,25));
            if (!window.fixtureFinalPhase) throw new Error(`Final phase not reached: ${modpackInstallation.state.error}`);
            await modpackInstallation.cancel();
            while (modpackInstallation.state.busy && Date.now()<deadline) await new Promise(resolve => setTimeout(resolve,25));
            const {profiles} = await import('/src/lib/stores/profiles.svelte.ts');
            return {busy:modpackInstallation.state.busy,rollback:window.fixtureRollback,visible:profiles.list.some(profile => profile.id === 'cancelled-import'),calls:window.roomCalls};
        });
        assert.equal(cancellation.busy,false);
        assert.equal(cancellation.rollback,true);
        assert.equal(cancellation.visible,false);
        assert.ok(cancellation.calls.includes('instance_end_import'));
        const existingCancellation = await page.evaluate(async () => {
            window.fixtureFinalPhase=false;window.fixtureExistingProfile=true;
            const {modpackInstallation} = await import('/src/lib/stores/modpackInstallation.svelte.ts');
            await modpackInstallation.start({sourceId:'existing',source:'modrinth',title:'Existing pack',slug:'existing',description:'',downloads:1,author:null,versions:['1.21.1'],categories:['fabric'],iconUrl:null,bannerUrl:null},'Existing pack',4096,'1.21.1');
            const deadline=Date.now()+10000;
            while(!window.fixtureFinalPhase && Date.now()<deadline) await new Promise(resolve=>setTimeout(resolve,25));
            if(!window.fixtureFinalPhase)throw new Error('Existing pack final phase not reached');
            await modpackInstallation.cancel();
            while(modpackInstallation.state.busy && Date.now()<deadline) await new Promise(resolve=>setTimeout(resolve,25));
            return {busy:modpackInstallation.state.busy,deleted:!!window.fixtureExistingDeleted};
        });
        assert.deepEqual(existingCancellation,{busy:false,deleted:false});
        await page.evaluate(async()=>{window.fixtureOwner=false;sessionStorage.setItem('fixture-owner','false');const {friendsState}=await import('/src/lib/stores/friends.svelte.ts');await friendsState.refreshProfile();});
        await page.goto(`${base}/owner`);
        await page.getByText('Este painel está disponível apenas para a conta autenticada do dono.',{exact:true}).waitFor();
        assert.equal(await page.getByRole('button',{name:'Excluir perfil de Alex',exact:true}).count(),0);
        assert.equal(await page.getByRole('button',{name:'+ Testador',exact:true}).count(),0);
        assert.deepEqual(errors,[]);
        fs.writeFileSync(`${output}/browser.json`,JSON.stringify({passed:true,roomProfiles:true,avatarFallback:true,backToFriends:true,lanFirst:true,settingsBackground,instanceBackground,ownerTags:true,ownerDeletion:true,publicTags:true,nonOwnerControlsHidden:true,cancellation,existingCancellation},null,2));
        console.log('Room, LAN guidance, owner tags, rounded-panel backgrounds and late cancellation passed.');
    } finally {await browser.close();}
})().catch(error => {console.error(error);process.exitCode=1;});
