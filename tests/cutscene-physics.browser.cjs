const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {setupLauncherDemo}=require('../scripts/launcher-video-fixture.cjs');

(async()=>{
    const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
    fs.mkdirSync('docs/validation/cutscene-physics',{recursive:true});
    try {
        for(const theme of ['dark','light']) {
            const page=await browser.newPage({viewport:{width:1180,height:760},reducedMotion:'no-preference'});
            const errors=[];page.on('pageerror',error=>errors.push(String(error)));
            await setupLauncherDemo(page);
            await page.addInitScript(theme=>{
                window.launcherDemo.state.settings.theme=theme==='light'?'default-light':'default-dark';
                const original=window.electronAPI.invoke;
                const invoke=async(command,args)=>{if(command==='app_init')await new Promise(resolve=>setTimeout(resolve,3500));return original(command,args);};
                window.electronAPI.invoke=invoke;window.__TAURI_INTERNALS__.invoke=invoke;
            },theme);
            await page.goto('http://127.0.0.1:1420/',{waitUntil:'domcontentloaded'});
            const scene=page.locator('[data-cutscene]'),canvas=scene.locator('canvas');
            await page.waitForFunction(()=>Number(document.querySelector('[data-cutscene] canvas')?.dataset.fragments)>20);
            await page.waitForFunction(()=>Number(document.querySelector('[data-cutscene] canvas')?.dataset.elapsed)>550);
            await page.screenshot({path:`docs/validation/cutscene-physics/${theme}-logo.png`});
            await page.waitForFunction(()=>document.querySelector('[data-cutscene]')?.dataset.phase==='shatter');
            console.log(await canvas.evaluate(node=>({...node.dataset})));
            await page.waitForFunction(()=>Number(document.querySelector('[data-cutscene] canvas')?.dataset.displacement)>150);
            await page.screenshot({path:`docs/validation/cutscene-physics/${theme}-shatter.png`});
            await page.waitForFunction(()=>document.querySelector('[data-cutscene]')?.dataset.phase==='reassemble');
            await scene.waitFor({state:'visible'});
            await page.screenshot({path:`docs/validation/cutscene-physics/${theme}-return.png`});
            await page.waitForFunction(()=>document.querySelector('[data-cutscene]')?.dataset.phase==='complete');
            const motion=await canvas.evaluate(node=>({...node.dataset}));
            assert.ok(Number(motion.floorContacts)>0);
            assert.ok(Number(motion.displacement)<1,JSON.stringify(motion));
            assert.ok(Number(motion.rotationError)<.05,JSON.stringify(motion));
            await page.screenshot({path:`docs/validation/cutscene-physics/${theme}-complete.png`});
            await scene.waitFor({state:'detached'});
            assert.deepEqual(errors,[]);
            console.log(JSON.stringify({theme,...motion,startupTimeoutDoesNotHide:true,completed:true}));
            await page.close();
        }
        const page=await browser.newPage({reducedMotion:'reduce'});
        await setupLauncherDemo(page);await page.goto('http://127.0.0.1:1420/',{waitUntil:'domcontentloaded'});
        await page.waitForFunction(()=>document.querySelector('[data-cutscene]')?.dataset.phase==='shatter');
        await page.waitForFunction(()=>document.querySelector('[data-cutscene]')?.dataset.phase==='complete');
        await page.locator('[data-cutscene]').waitFor({state:'detached'});
        await page.emulateMedia({reducedMotion:'no-preference'});await page.reload({waitUntil:'domcontentloaded'});
        await page.locator('[data-cutscene]').waitFor();await page.keyboard.press('Escape');await page.locator('[data-cutscene]').waitFor({state:'detached'});
        console.log(JSON.stringify({windowsReducedMotionStillPlaysFullIntro:true,skip:true}));
        await page.close();
        const shell=await browser.newPage({javaScriptEnabled:false});
        await shell.goto('http://127.0.0.1:1420/');
        const background=await shell.evaluate(()=>getComputedStyle(document.documentElement).backgroundColor);
        assert.ok(background.match(/\d+/g).slice(0,3).every(value=>Number(value)<80),background);
        console.log(JSON.stringify({initialDocumentDark:true,background}));
        await shell.close();
    } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exit(1);});
