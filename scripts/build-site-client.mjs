import {build} from 'esbuild';
import {writeSiteOutputs} from './write-site-outputs.mjs';

const result = await build({entryPoints:{app:'site-client/app.ts',account:'site-client/account.ts',i18n:'site-client/i18n.ts','visuals/motion':'site-client/visuals/motion.ts'},outdir:'website',bundle:false,target:'es2022',legalComments:'none',charset:'utf8',write:false,tsconfigRaw:{compilerOptions:{alwaysStrict:false}}});
await writeSiteOutputs(result.outputFiles);
