import {readFile} from 'node:fs/promises';
import {build} from 'esbuild';
import {writeSiteOutputs} from './write-site-outputs.mjs';

const result = await build({entryPoints:{app:'site-client/app.ts',account:'site-client/account.ts',i18n:'site-client/i18n.ts','visuals/motion':'site-client/visuals/motion.ts'},outdir:'website',bundle:false,target:'es2022',legalComments:'none',charset:'utf8',write:false,tsconfigRaw:{compilerOptions:{alwaysStrict:false}}});
await writeSiteOutputs(result.outputFiles);
const commerce = await build({entryPoints:{commerce:'site-client/commerce.ts'},outdir:'website',bundle:true,format:'esm',target:'es2022',legalComments:'none',charset:'utf8',minify:true,write:false});
await writeSiteOutputs(commerce.outputFiles);

await writeSiteOutputs([{path:'website/profile-signature.css',contents:new Uint8Array(await readFile('src/lib/styles/profileSignature.css'))}]);
