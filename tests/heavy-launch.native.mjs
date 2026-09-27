import { spawn, execFileSync } from "node:child_process";
import { createInterface } from "node:readline";
import { readFileSync, writeFileSync, existsSync, readdirSync, statSync, readlinkSync } from "node:fs";
import { join, resolve } from "node:path";
import assert from "node:assert/strict";

const root = resolve(process.argv[2]);
const daemon = spawn(resolve(process.env.LUXMC_NATIVE_BINARY || "src-tauri/target/debug/luxmc"), ["--daemon"], { env: { ...process.env, XDG_DATA_HOME: join(root, "data"), XDG_CONFIG_HOME: join(root, "config"), XDG_CACHE_HOME: join(root, "cache") }, stdio: ["pipe", "pipe", "pipe"] });
const waiting = new Map();
let sequence = 0;
let stderr = "";
daemon.stderr.on("data", chunk => { stderr = (stderr + chunk).slice(-12000); });
createInterface({ input: daemon.stdout }).on("line", line => {
    let message;
    try { message = JSON.parse(line); } catch { return; }
    const pending = waiting.get(message.id);
    if (!pending) return;
    waiting.delete(message.id);
    clearTimeout(pending.timer);
    if (message.error) pending.reject(new Error(message.error)); else pending.resolve(message.result);
});
daemon.on("exit", code => { for (const pending of waiting.values()) { clearTimeout(pending.timer); pending.reject(new Error(`Daemon exited: ${code}; ${stderr}`)); } waiting.clear(); });
const call = (command, args = {}) => new Promise((resolve, reject) => {
    const id = ++sequence;
    const timer = setTimeout(() => { waiting.delete(id); reject(new Error(`Timeout: ${command}`)); }, 1200000);
    waiting.set(id, { resolve, reject, timer });
    daemon.stdin.write(JSON.stringify({ id, command, args }) + "\n");
});

let pid;
let confirmed = false;
let forcedCleanup = false;
let result;
const launchedAt = Date.now();
try {
    const list=await call('profiles_list');
    const target=list.find(p=>p.name===(process.argv[3]||'Validation All the Mods 10'));
    assert.ok(target,'Requested test profile must exist');
    const account=await call('auth_offline_login',{username:'LuxmcHeavyTest'});
    console.log('Preparing JVM and loader',target.mcVersion,target.loader,target.loaderVersion);
    const started=await call('launch_game',{request:{versionId:target.mcVersion,profileId:target.id,accountId:account.id,enableVulkan:false}});
    pid=started.pid;console.log('JVM started',pid);
    let ready=false;
    console.log('Awaiting visual menu confirmation:',join(root,`menu-${target.id}.json`));
    for(let attempt=0;attempt<240;attempt++) {
        await new Promise(r=>setTimeout(r,5000));
        const log=join(target.gameDir,'logs/latest.log');
        try{process.kill(pid,0)}catch{throw new Error('Minecraft exited before startup; inspect '+log)}
        const confirmation=join(root,`menu-${target.id}.json`);
        if(existsSync(confirmation) && statSync(confirmation).mtimeMs >= launchedAt && attempt>=12) {
            const evidence=JSON.parse(readFileSync(confirmation,'utf8'));
            assert.ok(evidence.menuConfirmed && existsSync(evidence.screenshot),'Menu confirmation requires a saved screenshot');
            ready=true;confirmed=true;break;
        }
    }
    assert.ok(ready,'No visual confirmation of the main menu within twenty minutes');
    console.log('PASS',target.name,': main menu visually confirmed; no premature shutdown');
    result = {profileId:target.id,pid,ready};
} catch(error) {console.error(String(error));writeFileSync(join(root,'launch-failure.log'),String(error)+'\n'+stderr);process.exitCode=1;}
finally {
    if(pid && confirmed && process.platform==='linux') {
        try {
            execFileSync('hyprctl', ['dispatch', `hl.dsp.focus({window = "pid:${pid}"})`], {stdio:'pipe'});
            const active = JSON.parse(execFileSync('hyprctl', ['activewindow','-j'], {encoding:'utf8'}));
            assert.equal(active.pid, pid, 'Only close the test game window');
            execFileSync('hyprctl', ['dispatch', 'hl.dsp.window.close()']);
        } catch {
            try { execFileSync('hyprctl',['dispatch','closewindow',`pid:${pid}`], {stdio:'pipe'}); } catch {}
        }
        for(let attempt=0;attempt<30;attempt++) {
            try { process.kill(pid,0); } catch { pid=undefined;break; }
            await new Promise(r=>setTimeout(r,1000));
        }
    }
    if(pid){try{process.kill(pid,'SIGTERM');forcedCleanup=true;}catch{}}
    if(forcedCleanup && confirmed) { console.error('FAIL: game did not close gracefully after menu confirmation');process.exitCode=1; }
    if(process.platform==='linux') for(const entry of readdirSync('/proc').filter(x=>/^\d+$/.test(x))) {
        try {
            if(readFileSync(`/proc/${entry}/comm`,'utf8').trim()==='java' && readlinkSync(`/proc/${entry}/cwd`).startsWith(root+'/')) process.kill(Number(entry),'SIGTERM');
        } catch {}
    }
    if(result) writeFileSync(join(root,`launch-${result.profileId}.json`),JSON.stringify({...result,gracefulClose:!forcedCleanup},null,2));
    daemon.kill();
}
