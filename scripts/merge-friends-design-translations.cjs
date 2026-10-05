const fs = require('node:fs');
const bundle = JSON.parse(fs.readFileSync('docs/validation/friends-redesign-translations.json', 'utf8'));
for (const [locale, translations] of Object.entries(bundle)) {
  const path = `src/lib/i18n/${locale}.json`;
  const current = JSON.parse(fs.readFileSync(path, 'utf8'));
  Object.assign(current, translations);
  fs.writeFileSync(path, JSON.stringify(current, null, 2) + '\n');
}
console.log('Friends translations merged for pt-BR, en and es.');
