import { readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const source = readFileSync('C:/Users/preda/AppData/Local/Temp/codex-clipboard-74bf9b2a-3590-4634-9a26-9a4774147350.png');
assert.equal(source.subarray(1, 4).toString(), 'PNG');
const width = source.readUInt32BE(16);
const height = source.readUInt32BE(20);
assert.equal(width, 1919);
assert.equal(height, 1030);
const windowHeight = 1022;
const output = 'website/assets/captures-3.6/personalizer-window.svg';
writeFileSync(output, `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${windowHeight}" viewBox="0 0 ${width} ${windowHeight}" overflow="hidden"><defs><clipPath id="window"><rect width="${width}" height="${windowHeight}"/></clipPath></defs><image width="${width}" height="${height}" clip-path="url(#window)" href="data:image/png;base64,${source.toString('base64')}"/></svg>`);
writeFileSync('docs/validation/v3.6/layout-polish/personalizer-capture.json', JSON.stringify({ sourceWidth: width, sourceHeight: height, visibleHeight: windowHeight, excludedBottomPixels: height - windowHeight, originalPixelsPreserved: true, sourceSha256: createHash('sha256').update(source).digest('hex'), output }, null, 2));
console.log(`Personalizer viewport prepared: ${width} × ${windowHeight}, original screenshot pixels preserved.`);
