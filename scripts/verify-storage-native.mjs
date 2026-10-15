import {spawn} from 'node:child_process';
import {createInterface} from 'node:readline';
import {mkdir,writeFile,readFile,link,stat} from 'node:fs/promises';
import {join,resolve} from 'node:path';
import assert from 'node:assert/strict';

if(process.env.GITHUB_ACTIONS!=='true'||process.platform!=='win32') throw Error('This destructive fixture validation runs only on an ephemeral Windows CI runner.');
const binary=resolve(process.env.LUXMC_NATIVE_BINARY||'');
assert.ok(binary.toLowerCase().endsWith('luxmc.exe'));
const daemon=spawn(binary,['--daemon'],{windowsHide:true,stdio:['pipe','pipe','pipe']});
const requests=new Map();
let sequence=0;
let stderr='';
daemon.stderr.on('data',chunk=>stderr=(stderr+chunk).slice(-2000));
createInterface({input:daemon.stdout}).on('line',line=>{
    let message;try{message=JSON.parse(line);}catch{return;}
    const pending=requests.get(message.id);if(!pending)return;
    clearTimeout(pending.timer);requests.delete(message.id);
    message.error?pending.reject(Error(message.error)):pending.resolve(message.result);
});
daemon.on('exit',code=>{for(const pending of requests.values()){clearTimeout(pending.timer);pending.reject(Error(`Native process exited ${code}: ${stderr}`));}requests.clear();});
function call(command,args={}) {
    return new Promise((resolve,reject)=>{
        const id=++sequence;
        const timer=setTimeout(()=>{requests.delete(id);reject(Error(`Timeout: ${command}`));},60000);
        requests.set(id,{resolve,reject,timer});
        daemon.stdin.write(JSON.stringify({id,command,args})+'\n');
    });
}
let profile;
try {
    const environment=await call('app_update_environment');
    const [run,attempt]=process.env.LUXMC_BUILD_ID.split('.').map(Number);
    assert.equal(environment.revision,run*1000+attempt);
    assert.equal(environment.mode,'windows');
    const base=await call('app_data_directory');
    profile=await call('profiles_create',{input:{name:'CI storage fixture',mcVersion:'1.21.1',loader:'vanilla'}});
    assert.match(profile.id,/^[a-f0-9-]{36}$/);
    const initial=await call('storage_full_report',{refresh:true});
    const before=initial.instances.find(item=>item.id===profile.id).bytes;
    const world=join(profile.gameDir,'saves/fixture/level.dat');
    const options=join(profile.gameDir,'options.txt');
    const shared=join(base,'libraries',`storage-fixture-${profile.id}.jar`);
    const cache=join(base,'cache',`fixture-${profile.id}`);
    await mkdir(join(profile.gameDir,'mods'),{recursive:true});
    await mkdir(join(profile.gameDir,'saves/fixture'),{recursive:true});
    await mkdir(join(profile.gameDir,'logs'),{recursive:true});
    await mkdir(join(base,'libraries'),{recursive:true});
    await mkdir(cache,{recursive:true});
    await writeFile(world,Buffer.alloc(521));
    const oldOptions=await readFile(options).catch(()=>Buffer.alloc(0));
    await writeFile(shared,Buffer.alloc(512));
    await writeFile(join(profile.gameDir,'logs/fixture.log'),Buffer.alloc(14));
    await writeFile(join(cache,'fixture.bin'),Buffer.alloc(4096));
    const beforeLink=await call('storage_full_report',{refresh:true});
    await link(shared,join(profile.gameDir,'mods/shared.jar'));
    const measuredAt=performance.now();
    const report=await call('storage_full_report',{refresh:true});
    assert.equal(report.instances.find(item=>item.id===profile.id).bytes-before,1047);
    assert.equal(report.categories.find(item=>item.category==='instances')?.bytes||0,beforeLink.categories.find(item=>item.category==='instances')?.bytes||0);
    assert.equal(report.categories.reduce((sum,item)=>sum+item.bytes,0),report.totalBytes);
    assert.ok(report.logsBytes>=14);
    assert.ok(report.cacheBytes>=4096);
    assert.ok(await call('storage_clear_logs')>=14);
    assert.ok(await call('storage_clear_cache')>=4096);
    assert.equal((await stat(world)).size,521);
    assert.equal((await stat(join(profile.gameDir,'mods/shared.jar'))).size,512);
    assert.deepEqual(await readFile(options).catch(()=>Buffer.alloc(0)),oldOptions);
    const after=await call('storage_full_report',{refresh:true});
    assert.equal(after.instances.find(item=>item.id===profile.id).bytes,before+1033);
    console.log(JSON.stringify({nativeStorage:true,buildRevision:environment.revision,sharedFilesCountedOnce:true,cleanupPreservesWorldsModsAndOptions:true,measurementAndCleanupMs:Math.round(performance.now()-measuredAt)}));
} finally {
    if(profile) await call('profiles_delete',{id:profile.id}).catch(()=>{});
    daemon.stdin.end();
    if(daemon.exitCode===null)daemon.kill();
}
