const {chromium}=require('playwright-core');
const {setupLauncherDemo}=require('../scripts/launcher-video-fixture.cjs');
const assert=require('node:assert/strict');
const fs=require('node:fs');
(async()=>{
    const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
    fs.mkdirSync('docs/validation/v3.6/ui',{recursive:true});
    const results=[];
    try{
        for(const theme of ['dark','light']){
            const page=await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'});
            await setupLauncherDemo(page);
            await page.addInitScript(theme=>{window.launcherDemo.state.settings.theme=theme==='light'?'default-light':'default-dark';window.launcherDemo.state.settings.language='pt-BR';window.launcherDemo.state.settings.languageMode='manual';},theme);
            const errors=[];page.on('pageerror',error=>errors.push(String(error)));
            for(const route of ['/','/mods','/instances','/friends','/settings','/skins','/news','/hosting','/workshop','/screenshots','/logs-history','/logs','/organizer','/boost','/teamwork-preview']){
                const expectedRoute=({'/organizer':'/','/boost':'/instances'})[route]||route;
                await page.goto(`http://127.0.0.1:1420${route}`,{waitUntil:'domcontentloaded'});
                await page.locator(`[data-page-route="${expectedRoute}"]`).waitFor();
                await page.waitForTimeout(500);
                const overflow=await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth+2);
                assert.equal(overflow,false,`${route} ${theme} root overflow`);
                assert.deepEqual(errors,[],`${route} ${theme}`);
                assert.equal(await page.locator('[data-page-route]').evaluate(node=>node.getAnimations().length),0,'reduced motion must suppress page animations');
                await page.screenshot({path:`docs/validation/v3.6/ui/${theme}-${route.replaceAll('/','')||'home'}.png`});
                results.push({theme,route,actualRoute:new URL(page.url()).pathname,noPageErrors:true,noRootOverflow:true,reducedMotion:true});
            }
            await page.close();
        }
        const page=await browser.newPage({viewport:{width:1440,height:1000}});
        await setupLauncherDemo(page);
        await page.addInitScript(()=>{
            window.motionCalls=[];
            const animate=Element.prototype.animate;
            Element.prototype.animate=function(frames,options){
                const root=this.closest('[data-page-route]');
                const animation=animate.call(this,frames,options);
                if(root){
                    const call={route:root.dataset.pageRoute,panel:this.getAttribute('role')==='tabpanel',duration:typeof options==='object'?options.duration:options,finished:false};
                    window.motionCalls.push(call);
                    animation.finished.then(()=>call.finished=true,()=>{});
                }
                return animation;
            };
        });
        for(const route of ['/','/mods','/instances','/friends','/settings','/skins','/news','/hosting','/workshop','/screenshots','/logs-history','/logs','/teamwork-preview']){
            await page.goto(`http://127.0.0.1:1420${route}`);
            await page.locator(`[data-page-route="${route}"]`).waitFor();
            await page.waitForFunction(route=>window.motionCalls.some(call=>call.route===route&&call.duration===220),route);
            await page.waitForFunction(route=>window.motionCalls.some(call=>call.route===route&&call.duration===220&&call.finished),route);
            results.push({route,entranceAnimation:true,animationFinished:true});
        }
        await page.goto('http://127.0.0.1:1420/settings');
        await page.getByRole('tab',{name:'Java',exact:true}).click();
        await page.waitForFunction(()=>window.motionCalls.some(call=>call.panel&&call.duration===220));
        await page.close();
        fs.writeFileSync('docs/validation/v3.6/ui-results.json',JSON.stringify(results,null,2));
        console.log(JSON.stringify({screens:results.filter(result=>result.theme).length,animatedRoutes:results.filter(result=>result.entranceAnimation).length,errors:0,responsiveRoot:true,reducedMotion:true}));
    }finally{await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
