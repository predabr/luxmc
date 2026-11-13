import { readFileSync, writeFileSync, copyFileSync, mkdirSync } from 'node:fs';
const replace = (file, from, to) => writeFileSync(file, readFileSync(file, 'utf8').replaceAll(from, to));
for (const file of ['package.json','src-tauri/tauri.conf.json','src-tauri/Cargo.toml','site-server/lib/releases.ts','site-client/app.ts','.github/workflows/release.yml']) replace(file,'3.5.0','3.6.0');
for (const [file, name] of [['src-tauri/Cargo.lock','luxmc']]) replace(file,`name = "${name}"\nversion = "3.5.0"`,`name = "${name}"\nversion = "3.6.0"`);
for (const language of ['pt-BR','en','es']) {
    const file = `src/lib/i18n/${language}.json`, content = JSON.parse(readFileSync(file,'utf8'));
    content.app.version = 'v3.6.0';
    writeFileSync(file,JSON.stringify(content,null,2)+'\n');
}
replace('packaging/windows/hooks.nsh', 'Luxmc 3.5.0 · revisão 6', 'Luxmc 3.6.0');
replace('packaging/windows/hooks.nsh', '"LuxmcRevision" 6', '"LuxmcRevision" 0');
replace('src/lib/utils/updateVersion.ts','RELEASE_REVISION = 6','RELEASE_REVISION = 0');
replace('website/wrangler.toml','WINDOWS_LOCAL_VERSION = "3.5.1-revision.2"','WINDOWS_LOCAL_VERSION = "3.6.0"');
replace('website/index.html','v3.5.0','v3.6.0');
replace('website/index.html','LUXMC / 3.2','LUXMC / 3.6');
mkdirSync('website/assets/captures-3.6',{recursive:true});
const images = {
    home:'5938db7b-70d9-47b5-81fc-641d30ec4b7f',
    mods:'1c1aac10-e452-4886-aa1d-42976f9890e0',
    personalizer:'9a64c5e4-b3da-4c62-a36b-f55a02ef2330',
    friends:'82945981-474f-46d0-a5e7-4310af967533',
    instances:'e5ab7f6f-409e-4ebf-81c5-fd420b0025b7'
};
for (const [name, id] of Object.entries(images)) {
    copyFileSync(`${process.env.LOCALAPPDATA}/Temp/codex-clipboard-${id}.png`,`website/assets/captures-3.6/${name}.png`);
    replace('website/index.html',`assets/launcher-${name}.webp`,`assets/captures-3.6/${name}.png`);
}
console.log('Versão 3.6 e capturas fornecidas pelo autor preparadas.');
