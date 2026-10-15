import {createHash} from 'node:crypto';
import {mkdir,writeFile} from 'node:fs/promises';
import {resolve,join} from 'node:path';

if (!process.argv[2]) throw new Error('Pass a separate validation directory.');
const root=resolve(process.argv[2]);
await mkdir(join(root,'mods'),{recursive:true});
const files=[
    {name:'geckolib-forge-1.19-3.1.40.jar',url:'https://edge.forgecdn.net/files/4407/241/geckolib-forge-1.19-3.1.40.jar',size:2938383,sha1:'92a447e19afe1a23a33219f5428b2198a58094f4'},
    {name:'Schizo Cave Dweller.jar',url:'https://edge.forgecdn.net/files/5743/667/Schizo%20Cave%20Dweller.jar',size:1012102,sha1:'53feac6e39bf26309a0b94c8b42d9356f8e913f8'}
];
for (const file of files) {
    let verified;
    for (let attempt=0;attempt<3;attempt++) {
        try {
            const response=await fetch(file.url,{signal:AbortSignal.timeout(45000)});
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            const chunks=[];
            let size=0;
            for await (const chunk of response.body) {
                size+=chunk.length;
                if (size>file.size) throw new Error('Provider file exceeds its verified size.');
                chunks.push(chunk);
            }
            const bytes=Buffer.concat(chunks);
            if (bytes.length!==file.size||createHash('sha1').update(bytes).digest('hex')!==file.sha1) throw new Error('Provider file integrity mismatch.');
            verified=bytes;
            break;
        } catch (error) {if (attempt===2) throw error;}
    }
    await writeFile(join(root,'mods',file.name),verified,{flag:'wx'});
    console.log(JSON.stringify({name:file.name,bytes:verified.length,sha256:createHash('sha256').update(verified).digest('hex')}));
}
