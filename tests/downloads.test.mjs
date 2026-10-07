import test from 'node:test';
import assert from 'node:assert/strict';
import { onRequest } from '../website/functions/download/[platform].js';

const asset = { name: 'Lux.MC.Launcher.exe', browser_download_url: 'https://github.com/predabr/luxmc/releases/download/v3.0.0/Lux.MC.Launcher.exe' };

test('changed installer never appends a partial new file to an old If-Range download', async()=>{
    const original=globalThis.fetch;
    globalThis.fetch=async(url,options)=>{
        if(String(url).startsWith('https://api.github.com/'))return Response.json({tag_name:'v3.0.2',assets:[]});
        return options.headers?.Range ? new Response('MZ',{status:206,headers:{'ETag':'"new"','Content-Length':'2','Content-Range':'bytes 0-1/10'}}) : new Response('MZ01234567',{headers:{'ETag':'"new"','Content-Length':'10'}});
    };
    try {
        const response=await onRequest({request:new Request('https://luxmc.test/download/windows',{headers:{Range:'bytes=0-1','If-Range':'"old"'}}),params:{platform:'windows'},env:{WINDOWS_LOCAL_VERSION:'3.0.2'}});
        assert.equal(response.status,200);
        assert.equal(await response.text(),'MZ01234567');
    }finally{globalThis.fetch=original;}
});

test('hosted installer serves validated byte ranges when storage ignores Range', async () => {
    const original=globalThis.fetch;
    globalThis.fetch=async url=>String(url).startsWith('https://api.github.com/') ? Response.json({tag_name:'v3.0.2',assets:[]}) : new Response('MZ01234567',{headers:{'Content-Length':'10','ETag':'"current"'}});
    try {
        for(const [range,status,body,contentRange] of [['bytes=0-1',206,'MZ','bytes 0-1/10'],['bytes=4-',206,'234567','bytes 4-9/10'],['bytes=-3',206,'567','bytes 7-9/10'],['bytes=2-99',206,'01234567','bytes 2-9/10'],['bytes=20-',416,'','bytes */10'],['bytes=0-1,4-5',416,'','bytes */10']]) {
            const response=await onRequest({request:new Request('https://luxmc.test/download/windows',{headers:{Range:range}}),params:{platform:'windows'},env:{WINDOWS_LOCAL_VERSION:'3.0.2'}});
            assert.equal(response.status,status,range);
            assert.equal(response.headers.get('Content-Range'),contentRange);
            assert.equal(await response.text(),body);
        }
        const head=await onRequest({request:new Request('https://luxmc.test/download/windows',{method:'HEAD',headers:{Range:'bytes=0-1'}}),params:{platform:'windows'},env:{WINDOWS_LOCAL_VERSION:'3.0.2'}});
        assert.equal(head.status,206);
        assert.equal(head.headers.get('Content-Length'),'2');
        assert.equal(await head.text(),'');
        for(const [validator,status,body] of [['"current"',206,'MZ'],['"previous"',200,'MZ01234567'],['W/"current"',200,'MZ01234567']]) {
            const response=await onRequest({request:new Request('https://luxmc.test/download/windows',{headers:{Range:'bytes=0-1','If-Range':validator}}),params:{platform:'windows'},env:{WINDOWS_LOCAL_VERSION:'3.0.2'}});
            assert.equal(response.status,status);
            assert.equal(await response.text(),body);
        }
    } finally { globalThis.fetch=original; }
});

for (const method of ['GET', 'HEAD']) {
    test(`Windows ${method} retains the hosted installer when GitHub metadata is unavailable`, async () => {
        const original = globalThis.fetch;
        globalThis.fetch = async (url, options) => {
            if (String(url).startsWith('https://api.github.com/')) throw Error('GitHub unavailable');
            assert.equal(String(url), 'https://luxmc.test/releases/Lux%20MC%20Launcher.exe');
            assert.equal(options.method, method);
            assert.equal(options.headers.Range, 'bytes=0-1');
            return new Response(method==='HEAD' ? null : 'MZ', {status:206,headers:{'Content-Range':'bytes 0-1/20329717'}});
        };
        try {
            const response=await onRequest({request:new Request('https://luxmc.test/download/windows',{method,headers:{Range:'bytes=0-1'}}),params:{platform:'windows'},env:{WINDOWS_LOCAL_VERSION:'3.0.2'}});
            assert.equal(response.status,206);
            assert.equal(response.headers.get('Cache-Control'),'no-store');
            assert.equal(response.headers.get('Content-Disposition'),'attachment; filename="Lux MC Launcher.exe"');
            assert.equal(await response.text(),method==='HEAD' ? '' : 'MZ');
        } finally { globalThis.fetch=original; }
    });
}

test('same-version Windows hotfix downloads directly from the site without publishing GitHub assets', async () => {
    const original = globalThis.fetch;
    globalThis.fetch = async (url, options) => {
        if (String(url).startsWith('https://api.github.com/')) return Response.json({ tag_name: 'v3.0.2', assets: [asset] });
        assert.equal(String(url), 'https://luxmc.test/releases/Lux%20MC%20Launcher.exe');
        assert.equal(options.method, 'GET');
        return new Response('MZ', { headers: { 'Content-Length': '2' } });
    };
    try {
        const response = await onRequest({ request: new Request('https://luxmc.test/download/windows'), params: { platform: 'windows' }, env: { WINDOWS_LOCAL_VERSION: '3.0.2' } });
        assert.equal(response.status, 200);
        assert.equal(await response.text(), 'MZ');
        assert.equal(response.headers.get('Cache-Control'), 'no-store');
        assert.equal(response.headers.get('Content-Disposition'), 'attachment; filename="Lux MC Launcher.exe"');
    } finally { globalThis.fetch = original; }
});

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
