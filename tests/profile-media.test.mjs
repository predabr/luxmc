import test from 'node:test';
import assert from 'node:assert/strict';
import {profileImage,validatePublicProfile} from '../website/lib/public-profile.js';

test('GIF profile media accepts the documented budget and rejects overflow',()=>{
    const image=size=>'data:image/gif;base64,'+Buffer.concat([Buffer.from('GIF89a'),Buffer.alloc(size-6)]).toString('base64');
    assert.equal(profileImage(image(250000)),true);
    assert.equal(profileImage(image(700000)),true);
    assert.equal(profileImage(image(700001)),false);
    assert.equal(profileImage('data:image/svg+xml;base64,'+Buffer.from('<svg/>').toString('base64')),false);
});

test('profile identity is bounded and old clients retain defaults',()=>{
    const legacy={description:'About me',banner:'',portrait:'',packs:['Homestead']};
    assert.equal(validatePublicProfile(legacy).displayName,'');
    assert.equal(validatePublicProfile({...legacy,displayName:' Explorer ',status:' Building worlds '}).displayName,'Explorer');
    assert.equal(validatePublicProfile({...legacy,displayName:'x'.repeat(33)}),null);
    assert.equal(validatePublicProfile({...legacy,status:'x'.repeat(81)}),null);
    assert.equal(validatePublicProfile(null),null);
});

test('collections retain exact provider versions and reject arbitrary download links',()=>{
    const base={description:'',banner:'',portrait:'',packs:[]};
    const collection={id:'group-one',title:'Adventure',description:'Friends',entries:[{source:'modrinth',projectId:'project123',versionId:'version123',name:'Pack'}]};
    assert.equal(validatePublicProfile({...base,collections:[collection]}).collections[0].entries[0].versionId,'version123');
    assert.equal(validatePublicProfile({...base,collections:[{...collection,entries:[{...collection.entries[0],versionId:'https://unsafe.example/file'}]}]}),null);
    assert.equal(validatePublicProfile({...base,collections:[collection,collection]}),null);
});
