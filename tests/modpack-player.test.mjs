import assert from 'node:assert/strict';
import {test} from 'node:test';
import {onRequestGet} from '../website/functions/api/player.js';
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
