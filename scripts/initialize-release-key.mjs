import { generateKeyPairSync, createPublicKey } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { homedir } from 'node:os';

const directory = join(homedir(), '.codex', 'luxmc-release');
const privatePath = join(directory, 'signing-key.pem');
mkdirSync(directory, { recursive: true });
mkdirSync('packaging/security', { recursive: true });
if (!existsSync(privatePath)) {
    const key = generateKeyPairSync('ed25519');
    writeFileSync(privatePath, key.privateKey.export({ type: 'pkcs8', format: 'pem' }), { mode: 0o600, flag: 'wx' });
}
const publicKey = createPublicKey(readFileSync(privatePath));
writeFileSync('packaging/security/release-public-key.pem', publicKey.export({ type: 'spki', format: 'pem' }));
writeFileSync('packaging/security/release-public-key.bin', Buffer.from(publicKey.export({ format: 'jwk' }).x, 'base64url'));
console.log('Chave privada preservada fora do projeto; somente a chave pública foi exportada.');
