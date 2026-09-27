import { spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { mkdtempSync, writeFileSync, readdirSync, mkdirSync } from "node:fs";
import { join, resolve } from "node:path";
import assert from "node:assert/strict";

const base = resolve(process.env.LUXMC_VALIDATION_HOME || join(process.env.HOME, ".cache/luxmc-validation"));
mkdirSync(base, {recursive:true});
const root = process.env.LUXMC_HEAVY_ROOT ? resolve(process.env.LUXMC_HEAVY_ROOT) : mkdtempSync(join(base, "heavy-packs-"));
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

console.log(`Evidence directory: ${root}`);
const results=[];
try {
    const versions=await call('mods_versions',{projectId:'925200',mcVersion:'1.21.1',source:'curseforge'});
    const version=versions.find(v=>v.files?.length && /10-8\.2$/.test(v.name));
    if (!version) throw new Error('No published All the Mods 10 version returned');
    const file=version.files[0];
    console.log('Downloading CurseForge',version.name);
    const archive=await call('mods_download_to_temp',{url:file.url,fileName:file.filename});
    console.log('Importing CurseForge',version.name);
    const value=await call('instance_import_modpack',{filePath:archive,profileName:'Validation All the Mods 10',mcVersion:'1.21.1',loader:'neoforge'});
    const profile=value.profile||value;
    const jars=readdirSync(join(profile.gameDir,'mods')).filter(x=>x.endsWith('.jar'));
    assert.ok(jars.length>200,`Expected heavy modpack, found ${jars.length}`);
    await call('instance_repair_modpack',{profileId:profile.id});
    results.push({source:'curseforge',name:version.name,profileId:profile.id,jars:jars.length,gameDir:profile.gameDir});
    console.log('PASS CurseForge',JSON.stringify(results.at(-1)));
} catch(error) { results.push({source:'curseforge',error:String(error)});console.error('FAIL CurseForge',String(error)); }
try {
    const versions=await fetch('https://api.modrinth.com/v2/project/prominence-2-fabric/version').then(r=>r.json());
    const version=versions.find(v=>v.version_number==='4.1.0'||v.name.endsWith('v4.1.0'));assert.ok(version,'Pinned Prominence 4.1.0 must exist');const file=version.files.find(f=>f.primary)||version.files[0];
    console.log('Downloading Modrinth',version.name);
    const archive=await call('mods_download_to_temp',{url:file.url,fileName:file.filename});
    console.log('Importing Modrinth',version.name);
    const value=await call('instance_import_mrpack',{filePath:archive,profileName:'Validation Prominence II'});
    const profile=value.profile||value;
    const jars=readdirSync(join(profile.gameDir,'mods')).filter(x=>x.endsWith('.jar'));
    assert.ok(jars.length>200,`Expected heavy modpack, found ${jars.length}`);
    await call('instance_repair_modpack',{profileId:profile.id});
    results.push({source:'modrinth',name:version.name,profileId:profile.id,jars:jars.length,gameDir:profile.gameDir});
    console.log('PASS Modrinth',JSON.stringify(results.at(-1)));
} catch(error) {results.push({source:'modrinth',error:String(error)});console.error('FAIL Modrinth',String(error));}
writeFileSync(join(root,'results.json'),JSON.stringify(results,null,2));
daemon.kill();
process.exitCode=results.some(r=>r.error)?1:0;
