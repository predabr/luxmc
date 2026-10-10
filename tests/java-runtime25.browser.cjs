const {chromium}=require('playwright-core');
const {setupLauncherDemo}=require('../scripts/launcher-video-fixture.cjs');
const assert=require('node:assert/strict');
(async()=>{
    const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
    try {
        const page=await browser.newPage({viewport:{width:1440,height:1000}});
        await setupLauncherDemo(page);
        await page.addInitScript(()=>{
            const original=window.electronAPI.invoke;
            window.electronAPI.invoke=async(command,args={})=>command==='java_scan' ? {runtimes:[8,16,17,21,25].map(major=>({major,installed:true,path:`C:/fixture/java/${major}/bin/java.exe`,versionString:`Java ${major}`,isSystem:false}))} : original(command,args);
        });
        await page.goto('http://127.0.0.1:1420/instances/demo-fo');
        await page.getByRole('tab',{name:'Configurações',exact:true}).click();
        const java=page.locator('#instance-java');
        await java.locator('option').filter({hasText:'Java 25'}).waitFor({state:'attached'});
        await java.selectOption('C:/fixture/java/25/bin/java.exe');
        assert.equal(await java.inputValue(),'C:/fixture/java/25/bin/java.exe');
        await java.selectOption('C:/fixture/java/16/bin/java.exe');
        assert.equal(await java.inputValue(),'C:/fixture/java/16/bin/java.exe');
        console.log('Java 16 and 25 appear in instance settings and can be selected.');
    } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
