import { readFileSync, writeFileSync } from 'node:fs';

const version = '3.9.0';
const previous = JSON.parse(readFileSync('package.json', 'utf8')).version;
if (!['3.8.0', version].includes(previous)) throw new Error('Unexpected release version.');
for (const file of ['package.json', 'src-tauri/tauri.conf.json', 'src-tauri/Cargo.toml', 'src-tauri/repair-helper/Cargo.toml', 'site-server/lib/releases.ts', '.github/workflows/release.yml', '.github/workflows/uninstaller-verification.yml', 'website/index.html', 'website/wrangler.toml', 'packaging/arch/PKGBUILD']) {
    writeFileSync(file, readFileSync(file, 'utf8').replaceAll('3.8.0', version));
}
for (const [file, name] of [['src-tauri/Cargo.lock', 'luxmc'], ['src-tauri/repair-helper/Cargo.lock', 'luxmc-repair']]) {
    const value = readFileSync(file, 'utf8');
    writeFileSync(file, value.replace(new RegExp(`(name = "${name}"\\r?\\nversion = ")[^"]+`), `$1${version}`));
}
for (const locale of ['pt-BR', 'en', 'es']) {
    const file = `src/lib/i18n/${locale}.json`;
    const value = JSON.parse(readFileSync(file, 'utf8'));
    value.app.version = `v${version}`;
    writeFileSync(file, JSON.stringify(value, null, 2) + '\n');
}
const hooks = 'packaging/windows/hooks.nsh';
writeFileSync(hooks, readFileSync(hooks, 'utf8').replace(/!define MUI_WELCOMEPAGE_TITLE "[^"]+"/, `!define MUI_WELCOMEPAGE_TITLE "Luxmc ${version}"`));
writeFileSync('packaging/arch/PKGBUILD', readFileSync('packaging/arch/PKGBUILD', 'utf8').replace("license=('custom:Luxmc')", "license=('GPL-3.0-only')"));
console.log(`Prepared Luxmc ${version}.`);
