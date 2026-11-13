import { createHash } from 'node:crypto';
import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
const response=await fetch('https://api.github.com/repos/cli/cli/releases/latest',{signal:AbortSignal.timeout(30000)});
if(!response.ok)throw new Error('GitHub CLI metadata unavailable');
const metadata=await response.json();
const asset=metadata.assets.find(value=>/^gh_[\d.]+_windows_amd64\.zip$/.test(value.name));
if(!asset||!asset.browser_download_url.startsWith('https://github.com/cli/cli/releases/download/'))throw new Error('Official CLI archive unavailable');
const downloaded=await fetch(asset.browser_download_url,{signal:AbortSignal.timeout(120000)});
if(!downloaded.ok)throw new Error('GitHub CLI download failed');
const bytes=Buffer.from(await downloaded.arrayBuffer());
if(bytes.length!==asset.size)throw new Error('CLI archive size mismatch');
let expected=asset.digest?.replace(/^sha256:/,'');
if(!expected){const checksums=metadata.assets.find(value=>/_checksums\.txt$/.test(value.name));const text=await(await fetch(checksums.browser_download_url)).text();expected=text.split('\n').find(line=>line.endsWith(`  ${asset.name}`))?.split(/\s/)[0];}
if(!expected||createHash('sha256').update(bytes).digest('hex')!==expected)throw new Error('CLI checksum mismatch');
writeFileSync(join(process.env.TEMP,'luxmc-github-cli.zip'),bytes);
console.log(JSON.stringify({officialCliVersion:metadata.tag_name,checksumVerified:true}));
