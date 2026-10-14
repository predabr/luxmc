import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

function flatten(value, prefix = '') {
    return Object.fromEntries(Object.entries(value).flatMap(([key, item]) => {
        const path = prefix ? `${prefix}.${key}` : key;
        return typeof item === 'object' && item !== null ? Object.entries(flatten(item, path)) : [[path, item]];
    }));
}
const dictionaries = Object.fromEntries(['en','pt-BR','es'].map(locale => [locale,flatten(JSON.parse(readFileSync(new URL(`../src/lib/i18n/${locale}.json`,import.meta.url),'utf8')))]));
test('all launcher locales cover the same keys and preserve every interpolation variable', () => {
    const keys = Object.keys(dictionaries.en).sort();
    const variables = text => [...new Set([...text.matchAll(/{{\s*([\w.]+)\s*}}/g)].map(match => match[1]))].sort();
    for (const [locale, dictionary] of Object.entries(dictionaries)) {
        assert.deepEqual(Object.keys(dictionary).sort(),keys,locale);
        for (const key of keys) {
            assert.equal(typeof dictionary[key],'string',`${locale}: ${key}`);
            assert.ok(dictionary[key].trim(),`${locale}: ${key}`);
            assert.deepEqual(variables(dictionary[key]),variables(dictionaries.en[key]),`${locale}: ${key}`);
        }
    }
});
