import test from 'node:test';
import assert from 'node:assert/strict';
import { onRequest } from '../website/functions/download/[platform].js';

const asset = { name: 'Lux.MC.Launcher.exe', browser_download_url: 'https://github.com/predabr/luxmc/releases/download/v3.0.0/Lux.MC.Launcher.exe' };

for (const method of ['GET', 'HEAD']) {
    test(`Windows ${method} downloads preserve the filename and published installer`, async () => {
        const original = globalThis.fetch;
        const calls = [];
        globalThis.fetch = async (url, options) => {
            calls.push({ url, options });
            if (url.startsWith('https://api.github.com/')) return Response.json({ tag_name: 'v3.0.0', assets: [asset] });
            assert.equal(url, asset.browser_download_url);
            assert.equal(options.method, method);
            assert.equal(options.headers.Range, 'bytes=0-1');
            return new Response(method === 'HEAD' ? null : 'MZ', { status: 206, headers: { 'Content-Length': '2', 'Content-Range': 'bytes 0-1/20000000', 'Accept-Ranges': 'bytes' } });
        };
        try {
            const response = await onRequest({ request: new Request('https://luxmc.test/download/windows', { method, headers: { Range: 'bytes=0-1' } }), params: { platform: 'windows' } });
            assert.equal(response.status, 206);
            assert.equal(response.headers.get('Content-Disposition'), 'attachment; filename="Lux MC Launcher.exe"');
            assert.equal(response.headers.get('Content-Range'), 'bytes 0-1/20000000');
            assert.equal(await response.text(), method === 'HEAD' ? '' : 'MZ');
            assert.equal(calls.length, 2);
        } finally { globalThis.fetch = original; }
    });
}

test('failed upstream downloads do not return an installer payload', async () => {
    const original = globalThis.fetch;
    globalThis.fetch = async url => url.startsWith('https://api.github.com/') ? Response.json({ tag_name: 'v3.0.0', assets: [asset] }) : new Response('not an installer', { status: 404 });
    try {
        const response = await onRequest({ request: new Request('https://luxmc.test/download/windows'), params: { platform: 'windows' } });
        assert.equal(response.status, 502);
        assert.equal(response.headers.get('Content-Disposition'), null);
        assert.equal(response.headers.get('Cache-Control'), 'no-store');
    } finally { globalThis.fetch = original; }
});
