import { execFileSync } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import assert from 'node:assert/strict';

export async function nativeBuildIdentity(binary, expectedRevision) {
    assert.ok(Number.isSafeInteger(expectedRevision) && expectedRevision > 0);
    const bytes = await readFile(binary);
    assert.equal(bytes.subarray(0, 2).toString(), 'MZ');
    const peOffset = bytes.readUInt32LE(0x3c);
    assert.equal(bytes.subarray(peOffset, peOffset + 4).toString(), 'PE\0\0');
    assert.equal(bytes.readUInt16LE(peOffset + 4), 0x8664);
    assert.ok(bytes.includes(Buffer.from(String(expectedRevision))), 'Expected compiled build revision must be present in the native executable');
    const version = execFileSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', '(Get-Item -LiteralPath $env:LUXMC_IDENTITY_BINARY).VersionInfo.ProductVersion'], {
        env: { ...process.env, LUXMC_IDENTITY_BINARY: binary },
        encoding: 'utf8',
        windowsHide: true
    }).trim();
    const expectedVersion = JSON.parse(await readFile(new URL('../package.json', import.meta.url), 'utf8')).version;
    assert.equal(version, expectedVersion);
    return { mode: 'windows', revision: expectedRevision, version, compiledIdentityVerified: true };
}
