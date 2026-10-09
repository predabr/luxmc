import { createPrivateKey, createPublicKey, sign, verify } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const [directory] = process.argv.slice(2);
if (!directory || !process.env.LUXMC_RELEASE_SIGNING_KEY) throw new Error('Informe o diretório da release e LUXMC_RELEASE_SIGNING_KEY.');
const key = createPrivateKey(readFileSync(process.env.LUXMC_RELEASE_SIGNING_KEY));
if (key.asymmetricKeyType !== 'ed25519') throw new Error('A chave da release precisa ser Ed25519.');
const official = createPublicKey(readFileSync(new URL('../packaging/security/release-public-key.pem', import.meta.url)));
if (!createPublicKey(key).export({ type: 'spki', format: 'der' }).equals(official.export({ type: 'spki', format: 'der' }))) throw new Error('Chave de assinatura diferente da chave oficial.');
for (const name of ['SHA256SUMS', 'latest.json']) {
    const bytes = readFileSync(resolve(directory, name));
    const signature = sign(null, bytes, key);
    if (!verify(null, bytes, official, signature)) throw new Error('Falha na conferência da assinatura.');
    writeFileSync(resolve(directory, `${name}.sig`), signature.toString('base64') + '\n');
}
console.log('Manifestos assinados e verificados com a chave oficial.');
