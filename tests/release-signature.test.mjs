import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { generateKeyPairSync, verify, createPublicKey } from 'node:crypto';
import { spawnSync } from 'node:child_process';

test('release signing refuses a key that is not the embedded official key', () => {
    const directory=mkdtempSync(join(tmpdir(),'luxmc-signature-'));
    try {
        const {privateKey}=generateKeyPairSync('ed25519');
        const privatePath=join(directory,'test-key.pem');
        writeFileSync(privatePath,privateKey.export({type:'pkcs8',format:'pem'}));
        writeFileSync(join(directory,'SHA256SUMS'),'test\n');writeFileSync(join(directory,'latest.json'),'{}\n');
        const result=spawnSync(process.execPath,['scripts/sign-release.mjs',directory],{encoding:'utf8',env:{...process.env,LUXMC_RELEASE_SIGNING_KEY:privatePath}});
        assert.notEqual(result.status,0);assert.match(result.stderr,/diferente da chave oficial/);
    } finally {rmSync(directory,{recursive:true,force:true});}
});

test('published public PEM and embedded raw verification key identify the same signer', () => {
    const key=createPublicKey(readFileSync('packaging/security/release-public-key.pem'));
    assert.equal(key.asymmetricKeyType,'ed25519');
    assert.deepEqual(Buffer.from(key.export({format:'jwk'}).x,'base64url'),readFileSync('packaging/security/release-public-key.bin'));
});
