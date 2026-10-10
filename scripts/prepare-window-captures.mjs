import { readFileSync, writeFileSync } from 'node:fs';
import assert from 'node:assert/strict';

for (const [name, visibleHeight] of [['home', 1022], ['mods', 1022], ['instances', 1007], ['friends', 1022]]) {
    const source = readFileSync(`website/assets/captures-3.6/${name}.png`);
    assert.equal(source.subarray(1, 4).toString(), 'PNG');
    const width = source.readUInt32BE(16);
    const height = source.readUInt32BE(20);
    assert.ok(visibleHeight > 0 && visibleHeight < height);
    const output = `website/assets/captures-3.6/${name}-window.svg`;
    writeFileSync(output, `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${visibleHeight}" viewBox="0 0 ${width} ${visibleHeight}" overflow="hidden"><defs><clipPath id="window"><rect width="${width}" height="${visibleHeight}"/></clipPath></defs><image width="${width}" height="${height}" clip-path="url(#window)" href="data:image/png;base64,${source.toString('base64')}"/></svg>`);
    console.log(`${output}: ${width} × ${visibleHeight}`);
}
