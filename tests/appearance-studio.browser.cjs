const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');

(async () => {
    const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
    const output='docs/validation/appearance-shield';fs.mkdirSync(output,{recursive:true});
    try {
        for(const [locale,theme,width] of [['pt-BR','dark',1440],['en','light',1440],['es','dark',720]]) {
            const strings=require(`../src/lib/i18n/${locale}.json`);
            const page=await browser.newPage({viewport:{width,height:1050},reducedMotion:'no-preference'});
            const errors=[];page.on('pageerror',error=>errors.push(error.message));
            await setupLauncherDemo(page);
            await page.addInitScript(({locale,theme})=>{
                const state=window.launcherDemo.state;state.settings.language=locale;state.settings.languageMode='manual';state.settings.theme=`default-${theme}`;localStorage.setItem('luxmc.locale',locale);localStorage.setItem('luxmc_theme',theme);
                const invoke=window.electronAPI.invoke;
                window.electronAPI.invoke=async(command,args={})=>{
                    if(command==='social_request'&&args.request.action==='profile_get')return {profile:{id:args.request.targetId||state.account.uuid,username:'LuxPlayer',displayName:'Explorer',status:'',description:'',banner:'',portrait:'/grass_head.png',packs:[],collections:[],profileStyle:{accent:'rose',frame:'double',effect:'pulse',layout:'classic'},membership:{plan:'pro',expiresAt:1893456000}}};
                    return invoke(command,args);
                };
            },{locale,theme});
            await page.goto('http://127.0.0.1:1420/friends');
            await page.getByRole('button',{name:strings.publicProfile.edit,exact:true}).click();
            const dialog=page.getByRole('dialog');await dialog.getByRole('tab',{name:strings.profileStudio.appearance,exact:true}).click();
            const studio=dialog.locator('.signature-studio');await studio.waitFor();
            const label=(key,option)=>`${strings.planProfile[key]}: ${strings.planProfile[`${key}_${option}`]}`;
            const original=studio.getByRole('button',{name:label('frame','none'),exact:true});
            const double=studio.getByRole('button',{name:label('frame','double'),exact:true});
            assert.equal(await original.locator('[data-profile-avatar]').evaluate(el=>getComputedStyle(el,'::after').borderTopWidth),'0px');
            assert.equal(await double.locator('[data-profile-avatar]').evaluate(el=>getComputedStyle(el,'::after').borderTopWidth),'4px');
            assert.equal(await double.locator('[data-profile-avatar]').evaluate(el=>getComputedStyle(el,'::after').borderRadius===getComputedStyle(el).borderRadius),true);
            assert.equal(await studio.getByRole('button',{name:label('effect','none'),exact:true}).locator('[data-profile-avatar]').evaluate(el=>getComputedStyle(el,'::before').display),'none');
            assert.equal(await studio.locator('.signature-mini').evaluateAll(nodes=>nodes.flatMap(node=>node.getAnimations({subtree:true})).filter(animation=>animation.constructor.name==='CSSAnimation'&&animation.playState==='running').length),0);
            await original.click();assert.equal(await original.getAttribute('aria-pressed'),'true');
            await double.click();assert.equal(await double.getAttribute('aria-pressed'),'true');
            await dialog.screenshot({path:`${output}/studio-${locale}-${theme}.png`});
            await dialog.locator(':scope > div.overflow-y-auto').evaluate(el=>el.scrollTop+=650);
            await dialog.screenshot({path:`${output}/effects-${locale}-${theme}.png`});
            assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
            assert.equal((await studio.innerText()).includes('planProfile.'),false);
            await page.evaluate(()=>document.documentElement.classList.add('resource-busy'));
            assert.equal(await studio.locator('.signature-preview [data-profile-avatar]').evaluate(el=>getComputedStyle(el,'::before').animationPlayState),'paused');
            assert.deepEqual(errors,[]);await page.close();
        }
        console.log('PASS prévias isoladas, escolhas distintas, três idiomas, dois temas, telas compactas e animações econômicas');
    } finally {await browser.close();}
})().catch(error=>{console.error(error.message);process.exitCode=1;});
