const { chromium } = require('playwright-core');
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const {parseGIF} = require('gifuct-js');
(async () => {
    const browser = await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
    try {
        const page = await browser.newPage();
        await page.goto('http://127.0.0.1:1420/settings');
        const results = [];
        for (const name of ['flight-flightreacts.gif', 'scuba-scuba-cat.gif']) {
            const bytes = fs.readFileSync(path.join(process.env.USERPROFILE, 'Downloads', name));
            const output = await page.evaluate(async ({base64,name}) => {
                const {prepareAnimatedImage} = await import('/src/lib/utils/animatedImage.ts');
                const input = Uint8Array.from(atob(base64), char => char.charCodeAt(0));
                const started = performance.now();
                const output = await prepareAnimatedImage(new File([input], name, {type:'image/gif'}));
                const outputBytes = Uint8Array.from(atob(output.split(',')[1]), char => char.charCodeAt(0));
                const type = output.startsWith('data:image/webp;') ? 'image/webp' : 'image/gif';
                const decoder = new ImageDecoder({ data: outputBytes, type });
                await decoder.tracks.ready;
                const count = decoder.tracks.selectedTrack.frameCount;
                const first = await decoder.decode({frameIndex:0}), last = await decoder.decode({frameIndex:count-1});
                const decoded = {frames:count,width:first.image.displayWidth,height:first.image.displayHeight,duration:(last.image.timestamp + last.image.duration)/1000,type};
                first.image.close(); last.image.close(); decoder.close();
                return {output,seconds:(performance.now()-started)/1000,decoded};
            }, {base64:bytes.toString('base64'),name});
            const compressed = Buffer.from(output.output.split(',')[1], 'base64');
            const original = parseGIF(Uint8Array.from(bytes).buffer);
            const frames = gif => gif.frames.filter(frame => frame.image);
            const duration = gif => frames(gif).reduce((sum,frame) => sum + (frame.gce?.delay || 10) * 10,0);
            const webp = output.decoded.type === 'image/webp';
            const loopOffset = compressed.indexOf(Buffer.from('ANIM'));
            const result = {name,inputBytes:bytes.length,outputBytes:compressed.length,inputWidth:original.lsd.width,inputHeight:original.lsd.height,...output.decoded,originalFrames:frames(original).length,originalDuration:duration(original),seconds:output.seconds,animated:output.decoded.frames>1,loopsForever:webp ? loopOffset >= 0 && compressed.readUInt16LE(loopOffset + 12) === 0 : compressed.includes(Buffer.from('NETSCAPE2.0\x03\x01\x00\x00\x00','binary'))};
            assert.equal(result.frames, result.originalFrames);
            assert.equal(result.duration, result.originalDuration);
            assert.ok(result.outputBytes <= 700000 && result.animated);
            assert.ok(result.loopsForever);
            results.push(result);
            console.log(JSON.stringify(result));
        }
        fs.writeFileSync('docs/validation/v3.6/upgrades/gifs.json',JSON.stringify(results,null,2));
    } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
