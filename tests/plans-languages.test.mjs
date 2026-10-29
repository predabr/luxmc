import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { JSDOM } from 'jsdom';

test('plans switch between three languages without changing prices, identities or product actions', () => {
    const dom = new JSDOM(readFileSync(new URL('../website/planos.html',import.meta.url),'utf8'),{url:'https://luxmc.test/planos',runScripts:'outside-only'});
    try {
        const {window}=dom;
        window.eval(readFileSync(new URL('../website/i18n.js',import.meta.url),'utf8'));
        window.document.dispatchEvent(new window.Event('DOMContentLoaded'));
        const before=[...window.document.querySelectorAll('[data-plan]')].map(node=>node.dataset.plan);
        for (const [locale,title] of [['en','Your world.'],['es','Tu mundo.'],['pt','Seu mundo.']]) {
            window.LuxI18n.setLanguage(locale);
            assert.ok(window.document.querySelector('h1').textContent.includes(title));
            assert.equal(window.document.querySelector('[data-language-select]').value,locale);
            assert.deepEqual([...window.document.querySelectorAll('[data-plan]')].map(node=>node.dataset.plan),before);
            assert.ok(window.document.querySelector('[data-tier=plus] .price').textContent.includes('20,00'));
            for (const key of ['plans.gif', 'plans.showcase', 'plans.proLan', 'plans.wallpapers', 'plans.lanNote']) {
                const node=window.document.querySelector(`[data-i18n="${key}"]`);
                assert.equal(node.textContent,window.LuxI18n.t(key));
                assert.ok(node.textContent.length>20);
            }
            assert.equal(window.document.querySelectorAll('[data-tier=plus] [data-i18n="plans.showcase"]').length,0);
            assert.ok(window.document.querySelector('[data-tier=pro] [data-i18n="plans.proLan"]').textContent.includes('10'));
            for (const node of window.document.querySelectorAll('[data-i18n]')) assert.notEqual(window.LuxI18n.t(node.dataset.i18n),node.dataset.i18n);
        }
    } finally {dom.window.close();}
});
