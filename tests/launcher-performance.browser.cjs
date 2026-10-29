const {chromium}=require('playwright-core');
const fs=require('node:fs');
const {setupLauncherDemo}=require('../scripts/launcher-video-fixture.cjs');

(async()=>{
    const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
    const results=[];
    try {
        for(const route of ['/','/skins','/friends','/instances/demo-vp']) {
            const page=await browser.newPage({viewport:{width:1440,height:900}});
            const errors=[];page.on('pageerror',error=>{errors.push(error.message);console.error(error.stack);});
            await setupLauncherDemo(page);
            await page.addInitScript(()=>{
                window.performanceProbe={contexts:0,longTasks:0};
                const canvases=new WeakSet();
                const original=HTMLCanvasElement.prototype.getContext;
                HTMLCanvasElement.prototype.getContext=function(type,...args){const result=original.call(this,type,...args);if(result && /webgl/.test(type) && !canvases.has(this)){canvases.add(this);window.performanceProbe.contexts++;}return result;};
                new PerformanceObserver(list=>{window.performanceProbe.longTasks+=list.getEntries().length;}).observe({type:'longtask',buffered:true});
            });
            const cdp=await page.context().newCDPSession(page);await cdp.send('Performance.enable');
            const started=Date.now();
            await page.goto(`${process.env.LUXMC_BROWSER_BASE_URL||'http://127.0.0.1:4210'}${route}`);
            await page.locator('.launcher-sidebar').waitFor();
            await page.waitForFunction(()=>document.documentElement.classList.contains('settings-ready'));
            const readyMs=Date.now()-started;
            await page.waitForTimeout(2500);
            const initial=await cdp.send('Performance.getMetrics');
            await page.waitForTimeout(2000);
            const end=await cdp.send('Performance.getMetrics');
            const value=(metrics,name)=>metrics.metrics.find(metric=>metric.name===name)?.value;
            results.push({route,readyMs,taskSecondsDuringIdle:value(end,'TaskDuration')-value(initial,'TaskDuration'),jsHeapBytes:value(end,'JSHeapUsedSize'),...await page.evaluate(()=>window.performanceProbe),errors});
            await page.close();
        }
    } finally {await browser.close();}
    const record={fixtureOnly:true,localTimingNotAGameFPSMeasurement:true,results};
    if(process.argv[2])fs.writeFileSync(process.argv[2],JSON.stringify(record,null,2));
    console.log(JSON.stringify(record));
    if(results.some(result=>result.errors.length))process.exitCode=1;
})().catch(error=>{console.error(error);process.exitCode=1;});
