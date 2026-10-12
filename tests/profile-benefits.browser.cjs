const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const { spawn } = require('node:child_process');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const output = 'docs/validation/plan-benefits';
const defaultStyle = {accent:'brand',frame:'none',effect:'none',layout:'classic'};
const proStyle = {accent:'lilac',frame:'orbit',effect:'shimmer',layout:'centered'};

(async () => {
    fs.mkdirSync(output,{recursive:true});
    const server = spawn(process.execPath,['scripts/preview-website.mjs'],{env:{...process.env,LUXMC_SITE_PORT:'8794'},stdio:'ignore'});
    const browser = await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE || 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe'});
    const errors = [], results = [];
    try {
        for (let i=0;i<100;i++) {try {if((await fetch('http://127.0.0.1:8794/planos')).ok)break;} catch {} await new Promise(resolve=>setTimeout(resolve,100));}
        for (const theme of process.env.LUXMC_SITE_ONLY ? [] : ['dark','light']) for (const plan of ['free','plus','pro']) {
            const page = await browser.newPage({viewport:{width:1440,height:1050},reducedMotion:'reduce'});
            page.on('pageerror',error=>errors.push(error.message));
            await setupLauncherDemo(page);
            await page.addInitScript(({theme,plan,defaultStyle})=>{
                window.launcherDemo.state.contacts=[];window.launcherDemo.state.settings.theme=`default-${theme}`; localStorage.setItem('luxmc_theme',theme);
                const membership={plan,expiresAt:plan==='free'?null:1893456000,revision:1,collections:plan==='pro'?24:plan==='plus'?12:6,presets:plan==='pro'?20:plan==='plus'?6:0};
                const profile={id:window.launcherDemo.state.account.uuid,username:'LuxPlayer',avatarUrl:'/grass_head.png',displayName:'Explorer',role:'member',tags:['Testador'],membership,status:'Construindo com meus amigos',description:'Cada mundo tem uma história.',banner:'',portrait:'',packs:Array.from({length:plan==='pro'?17:plan==='plus'?9:0},(_,index)=>`Pack ${index}`),collections:[],profileStyle:defaultStyle};
                window.benefitSaves=[]; window.benefitProfile=profile;
                const invoke=window.electronAPI.invoke;
                window.electronAPI.invoke=async(command,args={})=>{
                    if(command==='membership_request' && args.request.action==='status')return {identity:{id:'00000000-0000-4000-8000-000000000001',username:'LuxPlayer',owner:false},membership,settings:{preferences:{},presets:[],revision:0}};
                    if(command==='social_request' && args.request.action==='profile_get')return {profile:{...profile,packs:profile.packs.slice(0,args.request.favouritesLimit===24?24:8),id:args.request.targetId || profile.id}};
                    if(command==='social_request' && args.request.action==='profile_save'){window.benefitSaves.push(structuredClone(args.request));Object.assign(profile,args.request);return {ok:true};}
                    const result=await invoke(command,args);
                    if(command==='social_request' && ['register','sync'].includes(args.request.action))Object.assign(result.me,{membership,profileStyle:profile.profileStyle});
                    return result;
                };
            },{theme,plan,defaultStyle});
            await page.goto('http://127.0.0.1:1420/friends');
            await page.getByRole('button',{name:'Editar meu perfil',exact:true}).click();
            const dialog=page.getByRole('dialog');await dialog.getByRole('tab',{name:'Vitrine',exact:true}).click();await dialog.getByText(`${plan==='pro'?17:plan==='plus'?9:0}/${plan==='pro'?24:plan==='plus'?16:8}`,{exact:true}).waitFor();await dialog.getByRole('tab',{name:'Aparência',exact:true}).click();
            assert.equal(await page.evaluate(()=>document.documentElement.classList.contains('light')),theme==='light');
            assert.equal(await dialog.getByRole('button',{name:'Paleta: Menta',exact:true}).isDisabled(),plan==='free');
            assert.equal(await dialog.getByRole('button',{name:'Moldura do avatar: Órbita',exact:true}).isDisabled(),plan!=='pro');
            if(plan!=='free') {await dialog.getByRole('button',{name:'Paleta: Menta',exact:true}).click();await dialog.getByRole('button',{name:'Moldura do avatar: Contorno duplo',exact:true}).click();}
            if(plan==='pro') {await dialog.getByRole('button',{name:'Paleta: Lilás',exact:true}).click();await dialog.getByRole('button',{name:'Moldura do avatar: Órbita',exact:true}).click();await dialog.getByRole('button',{name:'Efeito: Brilho da capa',exact:true}).click();await dialog.getByRole('button',{name:'Composição: Centralizada',exact:true}).click();}
            await dialog.screenshot({path:`${output}/editor-${plan}-${theme}.png`});
            if(plan!=='free') {
                await dialog.getByRole('button',{name:'Prévia completa',exact:true}).click();
                const signature=dialog.locator('..');
                assert.equal(await signature.getAttribute('data-profile-accent'),plan==='pro'?'lilac':'mint');
                assert.equal(await signature.getAttribute('data-profile-frame'),plan==='pro'?'orbit':'double');
                if(plan==='pro') assert.equal(await dialog.locator('[data-profile-heading]').evaluate(element=>getComputedStyle(element).flexDirection),'column');
                assert.equal(await dialog.locator('[data-profile-avatar]').first().evaluate(element=>getComputedStyle(element,'::after').animationName),'none');
                await dialog.screenshot({path:`${output}/profile-${plan}-${theme}.png`});
                await dialog.getByRole('button',{name:'Salvar perfil',exact:true}).click();
                await page.waitForFunction(()=>window.benefitSaves.length===1);
                const saved=await page.evaluate(()=>window.benefitSaves[0]); assert.deepEqual(saved.profileStyle,plan==='pro'?proStyle:{accent:'mint',frame:'double',effect:'none',layout:'classic'});
            }
            if(plan==='pro') {
                await page.emulateMedia({reducedMotion:'no-preference'});await page.waitForFunction(()=>getComputedStyle(document.querySelector('[role=dialog] [data-profile-avatar]'),'::after').animationName!=='none');await page.emulateMedia({reducedMotion:'reduce'});
                await page.setViewportSize({width:720,height:1000});
                assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
                await dialog.screenshot({path:`${output}/profile-compact-${theme}.png`});
            }
            assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
            results.push({area:'launcher',plan,theme,overflow:false}); await page.close();
        }
        for (const plan of ['free','plus','pro']) {
            const page=await browser.newPage({viewport:{width:plan==='pro'?430:1440,height:1000},reducedMotion:'reduce'});page.on('pageerror',error=>errors.push(error.message));
            let style={...defaultStyle};let saved=null;
            await page.route('**/api/commerce/**',async route=>{
                const action=route.request().url().split('/').pop();let result={ok:true};
                if(action==='status')result={identity:{id:'22222222-2222-4222-8222-222222222222',username:'Explorer',owner:false},membership:{plan,expiresAt:1893456000,collections:plan==='pro'?24:plan==='plus'?12:6,presets:plan==='pro'?20:plan==='plus'?6:0},profile:{profileStyle:style,displayName:'Explorer',status:'Construindo mundos'},checkout:{enabled:false,bank:'Nubank',pixKey:''},settings:{preferences:{},presets:[],revision:0},orders:[],csrfToken:'d'.repeat(64)};
                if(action==='profile_style'){saved=route.request().postDataJSON();style=saved.profileStyle;}
                await route.fulfill({status:200,contentType:'application/json',body:JSON.stringify(result)});
            });
            await page.goto('http://127.0.0.1:8794/planos');await page.getByRole('heading',{name:'Um perfil com a sua assinatura'}).waitFor();
            const editor=page.locator('#profile-customization');fs.writeFileSync(`${output}/site-${plan}-controls.json`,JSON.stringify(await editor.locator('select').evaluateAll(items=>items.map(select=>({name:select.getAttribute('aria-label'),options:[...select.options].map(option=>({value:option.value,disabled:option.disabled}))}))),null,2));assert.equal(await editor.getByLabel('Moldura do avatar',{exact:true}).locator('option[value="orbit"]').evaluate(option=>option.disabled),plan!=='pro');
            if(plan!=='free') {await editor.getByLabel('Paleta',{exact:true}).selectOption('mint');await editor.getByLabel('Moldura do avatar',{exact:true}).selectOption('double');}
            if(plan==='pro') {await editor.getByLabel('Paleta',{exact:true}).selectOption('lilac');await editor.getByLabel('Moldura do avatar',{exact:true}).selectOption('orbit');await editor.getByLabel('Efeito',{exact:true}).selectOption('shimmer');await editor.getByLabel('Composição',{exact:true}).selectOption('centered');}
            const refreshed=page.waitForResponse('**/api/commerce/status');await editor.getByRole('button',{name:'Salvar assinatura visual'}).click();await refreshed;assert.deepEqual(saved.profileStyle,style);
            await editor.screenshot({path:`${output}/site-${plan}.png`,style:'.skip-link {visibility:hidden !important}'});assert.equal(await page.getByRole('button',{name:'Escolher Plus'}).isDisabled(),true);assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
            assert.equal(await editor.locator('[data-profile-avatar]').evaluate(element=>getComputedStyle(element,'::after').animationName),'none');
            results.push({area:'site',plan,overflow:false});await page.close();
        }
        assert.deepEqual(errors,[]);fs.writeFileSync(`${output}/browser.json`,JSON.stringify({results,errors},null,2));console.log('Browser: Free, Plus e Pro, temas claro/escuro, prévia, gravação, site móvel e movimento reduzido passaram.');
    } finally {await browser.close();server.kill();}
})().catch(error=>{console.error(error);process.exitCode=1;});
