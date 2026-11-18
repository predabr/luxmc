const {chromium} = require('playwright-core');
const {setupLauncherDemo} = require('../../../scripts/launcher-video-fixture.cjs');
(async () => {
    const browser = await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
    try {
        for (const theme of ['dark','light']) {
            const page = await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'});
            await setupLauncherDemo(page);
            await page.addInitScript(theme => {window.launcherDemo.state.settings.theme=theme==='light'?'default-light':'default-dark';window.launcherDemo.state.settings.language='pt-BR';window.launcherDemo.state.settings.languageMode='manual';}, theme);
            await page.goto('http://127.0.0.1:1420/instances');
            await page.getByText('Vanilla Perfected',{exact:true}).first().waitFor();
            await page.screenshot({path:`docs/validation/revision6/instances-${theme}.png`});
            await page.close();
        }
    } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
