const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {setupLauncherDemo}=require('../scripts/launcher-video-fixture.cjs');

(async()=>{
    const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
    try {
        fs.mkdirSync('docs/validation/folder-images',{recursive:true});
        for (const theme of ['dark','light']) {
            const page=await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'});
            const errors=[];
            page.on('pageerror',error=>errors.push(String(error)));
            await setupLauncherDemo(page);
            await page.addInitScript(theme=>{
                window.launcherDemo.state.settings.theme=theme==='light'?'default-light':'default-dark';
                window.launcherDemo.state.settings.language='pt-BR';
                window.launcherDemo.state.settings.languageMode='manual';
                localStorage.setItem('luxmc_library_folders_v2',JSON.stringify(['Adventure']));
            },theme);
            await page.goto('http://127.0.0.1:1420/');
            const folder=page.locator('[data-library-folder="Adventure"]');
            await folder.waitFor();
            const image=folder.locator('[data-folder-image]');
            const choose=page.getByRole('button',{name:'Escolher imagem da pasta Adventure',exact:true});
            const upload=async file=>{
                const pending=page.waitForEvent('filechooser');
                await choose.click();
                await (await pending).setFiles(file);
            };
            const png=await page.evaluate(()=>{
                const canvas=document.createElement('canvas');canvas.width=800;canvas.height=200;
                const context=canvas.getContext('2d');context.fillStyle='tomato';context.fillRect(0,0,800,200);
                return canvas.toDataURL('image/png').split(',')[1];
            });
            await upload({name:'folder.png',mimeType:'image/png',buffer:Buffer.from(png,'base64')});
            await page.waitForFunction(()=>document.querySelector('[data-folder-image]')?.naturalWidth===384);
            assert.equal(await image.evaluate(node=>node.naturalHeight),96);
            assert.equal(await image.evaluate(node=>getComputedStyle(node).objectFit),'contain');
            assert.equal(await page.locator('[data-library-back]').count(),0);
            const saved=await image.getAttribute('src');
            await page.reload();await folder.waitFor();assert.equal(await image.getAttribute('src'),saved);
            await upload({name:'bad.png',mimeType:'image/png',buffer:Buffer.from('invalid image')});
            await page.getByText('Escolha uma imagem PNG, JPG, WebP ou GIF válida.',{exact:true}).waitFor();
            assert.equal(await image.getAttribute('src'),saved);
            const gif=Buffer.from('R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7','base64');
            await upload({name:'folder.gif',mimeType:'image/gif',buffer:gif});
            const gifData='data:image/gif;base64,'+gif.toString('base64');
            await page.waitForFunction(expected=>document.querySelector('[data-folder-image]')?.src===expected,gifData);
            await page.reload();await folder.waitFor();assert.equal(await image.getAttribute('src'),gifData);
            await page.evaluate(()=>{
                const original=Storage.prototype.setItem;
                window.restoreImageStorage=()=>Storage.prototype.setItem=original;
                Storage.prototype.setItem=function(key,value){if(key==='luxmc_library_folder_images_v1')throw new DOMException('Full','QuotaExceededError');return original.call(this,key,value);};
            });
            await upload({name:'folder.png',mimeType:'image/png',buffer:Buffer.from(png,'base64')});
            await page.getByText('Não foi possível salvar a imagem. Use um arquivo menor para liberar espaço.',{exact:true}).waitFor();
            assert.equal(await image.getAttribute('src'),gifData);
            await page.evaluate(()=>window.restoreImageStorage());
            await page.screenshot({path:`docs/validation/folder-images/${theme}.png`});
            await page.getByRole('button',{name:'Remover imagem da pasta Adventure',exact:true}).click();
            await page.waitForFunction(()=>!document.querySelector('[data-folder-image]'));
            await page.reload();await folder.waitFor();assert.equal(await image.count(),0);
            await folder.click();assert.equal(await page.locator('[data-library-back]').count(),1);
            assert.deepEqual(errors,[]);
            console.log(JSON.stringify({theme,thumbnail:true,persisted:true,gifPreserved:true,invalidRejected:true,quotaPreservesPrevious:true,removed:true}));
            await page.close();
        }
    } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exit(1)});
