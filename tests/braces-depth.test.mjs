import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const chokidar = require.resolve('chokidar', { paths: [require.resolve('tailwindcss')] });
const braces = require(require.resolve('braces', { paths: [chokidar] }));

test('build glob parsing and expansion retain normal behavior', () => {
    assert.deepEqual(braces.expand('src/{lib,routes}/**/*.{svelte,ts}'), [
        'src/lib/**/*.svelte', 'src/lib/**/*.ts', 'src/routes/**/*.svelte', 'src/routes/**/*.ts',
    ]);
    assert.equal(braces.compile('file.{png,jpg}'), 'file.(png|jpg)');
    assert.equal(braces.stringify(braces.parse('file.{png,jpg}')), 'file.{png,jpg}');
});

test('deep untrusted string patterns fail before recursive stack exhaustion', () => {
    const input = '{'.repeat(4000) + 'x,y' + '}'.repeat(4000);
    for (const operation of ['parse', 'compile', 'expand']) {
        assert.throws(() => braces[operation](input), error => error instanceof SyntaxError && /safe depth limit/.test(error.message));
    }
});

test('direct AST entry points also bound recursive depth', () => {
    let ast = { type: 'text', value: 'x' };
    for (let index = 0; index < 1000; index++) ast = { type: 'root', nodes: [ast] };
    for (const operation of ['compile', 'expand', 'stringify']) {
        assert.throws(() => braces[operation](ast), error => error instanceof SyntaxError && /safe depth limit/.test(error.message));
    }
});
