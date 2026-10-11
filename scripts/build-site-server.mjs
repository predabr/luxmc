import { build } from 'esbuild';
import { readdir } from 'node:fs/promises';
import { join } from 'node:path';
import { writeSiteOutputs } from './write-site-outputs.mjs';

const entryPoints=[];
async function collect(directory) {
    for (const entry of await readdir(directory,{withFileTypes:true})) {
        const path=join(directory,entry.name);
        if (entry.isDirectory()) await collect(path);
        else if (entry.name.endsWith('.ts') && !entry.name.endsWith('.d.ts')) entryPoints.push(path);
    }
}
await collect('site-server/functions');
await collect('site-server/lib');
const result = await build({entryPoints,outbase:'site-server',outdir:'website',bundle:false,format:'esm',target:'es2022',legalComments:'none',write:false});
await writeSiteOutputs(result.outputFiles);
