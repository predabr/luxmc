import {build} from 'esbuild';

await build({entryPoints:{app:'site-client/app.ts',account:'site-client/account.ts',i18n:'site-client/i18n.ts','visuals/motion':'site-client/visuals/motion.ts','visuals/studio':'site-client/visuals/studio.ts','visuals/skinEngine':'site-client/visuals/skinEngine.ts','visuals/capes':'site-client/visuals/capes.ts'},outdir:'website',bundle:false,target:'es2022',legalComments:'none',charset:'utf8',tsconfigRaw:{compilerOptions:{alwaysStrict:false}}});
