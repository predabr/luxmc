import assert from 'node:assert/strict';
import {test} from 'node:test';
import {onRequestGet} from '../website/functions/api/player.js';
import {onRequest as middleware} from '../website/functions/_middleware.js';
test('player supplies HTTPS referrer context and rejects invalid video identifiers', async()=>{
  const response=await onRequestGet({request:new Request('https://luxmc-r92.pages.dev/api/player?video=M7lc1UVf-VE')});
  assert.equal(response.status,200);
  assert.equal(response.headers.get('referrer-policy'),'strict-origin-when-cross-origin');
  assert.ok(response.headers.get('content-security-policy').includes('frame-ancestors'));
  const body=await response.text();
  assert.ok(body.includes('https://www.youtube-nocookie.com/embed/M7lc1UVf-VE'));
  assert.ok(body.includes('autoplay=1'));
  for(const video of ['','<script>','../../','abc']){
    const invalid=await onRequestGet({request:new Request('https://luxmc-r92.pages.dev/api/player?video='+encodeURIComponent(video))});
    assert.equal(invalid.status,400);
  }
});
test('the security middleware allows only the public player to be embedded by the launcher', async () => {
  const request = new Request('https://luxmc-r92.pages.dev/api/player?video=M7lc1UVf-VE');
  const response = await middleware({request,next:()=>onRequestGet({request})});
  assert.equal(response.headers.get('x-frame-options'), null);
  const csp = response.headers.get('content-security-policy');
  assert.ok(csp.includes('frame-src https://www.youtube-nocookie.com'));
  assert.ok(csp.includes('http://tauri.localhost'));
  assert.ok(!csp.includes('frame-ancestors *'));
  for (const path of ['/api/commerce/status', '/api/commerce/owner_queue', '/api/player/other', '/download/windows']) {
    const secured = await middleware({request:new Request('https://luxmc-r92.pages.dev'+path),next:()=>new Response('{}',{headers:{'Content-Type':'application/json'}})});
    assert.equal(secured.headers.get('x-frame-options'),'DENY');
    assert.equal(secured.headers.get('content-security-policy'),"default-src 'none'; frame-ancestors 'none'");
  }
  const invalid = new Request('https://luxmc-r92.pages.dev/api/player?video=invalid');
  assert.equal((await middleware({request:invalid,next:()=>onRequestGet({request:invalid})})).headers.get('x-frame-options'),'DENY');
});
