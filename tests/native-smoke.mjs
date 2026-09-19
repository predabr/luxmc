import { spawn, execFileSync } from "node:child_process";
import { createInterface } from "node:readline";
import { mkdtempSync, readFileSync, writeFileSync, existsSync, readdirSync, statSync, renameSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

const root = mkdtempSync(join(tmpdir(), "luxmc-native-smoke-"));
const daemon = spawn(resolve("src-tauri/target/debug/luxmc"), ["--daemon"], { env: { ...process.env, XDG_DATA_HOME: join(root, "data"), XDG_CONFIG_HOME: join(root, "config"), XDG_CACHE_HOME: join(root, "cache") }, stdio: ["pipe", "pipe", "pipe"] });
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
    const timer = setTimeout(() => { waiting.delete(id); reject(new Error(`Timeout: ${command}`)); }, 600000);
    waiting.set(id, { resolve, reject, timer });
    daemon.stdin.write(JSON.stringify({ id, command, args }) + "\n");
});
const sha = (buffer, algorithm = "sha512") => createHash(algorithm).update(buffer).digest("hex");
const results = [];
const runningGames = new Set();
const passed = (name, detail) => { results.push({ name, detail }); console.log(`PASS ${name}: ${detail}`); };
try {
    assert.deepEqual(await call("profiles_list"), []);
    const one = await call("profiles_create", { input: { name: "../../outside", mcVersion: "1.20.1", loader: "vanilla" } });
    const two = await call("profiles_create", { input: { name: "../../outside", mcVersion: "1.20.1", loader: "vanilla" } });
    assert.ok(one.gameDir.startsWith(join(root, "data")));
    assert.equal(one.gameDir.split(/[\\/]/).at(-1), one.id);
    assert.notEqual(one.gameDir, two.gameDir);
    passed("isolated profile paths", "path traversal and duplicate names cannot share gameDir");

    const versionsResponse = await fetch('https://api.modrinth.com/v2/project/fabulously-optimized/version?game_versions=%5B%221.20.1%22%5D');
    assert.equal(versionsResponse.status, 200);
    const version = (await versionsResponse.json())[0];
    const packFile = version.files.find(file => file.primary) || version.files[0];
    const pack = await call("mods_download_to_temp", { url: packFile.url, fileName: packFile.filename });
    assert.equal(sha(readFileSync(pack)), packFile.hashes.sha512);
    const imported = await call("instance_import_mrpack", { filePath: pack, profileName: "Smoke Fabulously Optimized", ramMb: 4096 });
    const profile = imported.profile || imported;
    assert.ok(profile.id && profile.gameDir);
    const index = JSON.parse(readFileSync(join(profile.gameDir, "modrinth.index.json")));
    let verified = 0;
    for (const file of index.files.filter(file => file.env?.client !== "unsupported")) {
        const bytes = readFileSync(join(profile.gameDir, file.path.replaceAll("\\", "/")));
        assert.ok(bytes.length > 0);
        if (file.hashes.sha512) assert.equal(sha(bytes), file.hashes.sha512);
        else if (file.hashes.sha1) assert.equal(sha(bytes, "sha1"), file.hashes.sha1);
        verified++;
    }
    passed("real Modrinth import", `${version.name}; ${verified} files verified by hash`);
    const mods = index.files.filter(file => file.path.endsWith(".jar") && file.env?.client !== "unsupported");
    const damaged = join(profile.gameDir, mods[0].path);
    const disabled = join(profile.gameDir, mods[1].path);
    writeFileSync(damaged, "");
    renameSync(disabled, disabled + ".disabled");
    await call("instance_repair_modpack", { profileId: profile.id });
    assert.ok(statSync(damaged).size > 0);
    assert.equal(sha(readFileSync(damaged), mods[0].hashes.sha512 ? "sha512" : "sha1"), mods[0].hashes.sha512 || mods[0].hashes.sha1);
    assert.ok(existsSync(disabled + ".disabled"));
    assert.ok(!existsSync(disabled));
    passed("auto-heal", "zero-byte JAR restored with matching hash; disabled JAR stays disabled");

    const cfVersions = await call("mods_versions", { projectId: "396246", mcVersion: "1.20.1", source: "curseforge" });
    const cfVersion = cfVersions.find(value => value.files.length);
    assert.ok(cfVersion, "CurseForge published modpack must have files");
    const cfFile = cfVersion.files[0];
    const cfPack = await call("mods_download_to_temp", { url: cfFile.url, fileName: cfFile.filename });
    const cfResult = await call("instance_import_modpack", { filePath: cfPack, profileName: "Smoke CurseForge", mcVersion: "1.20.1", loader: "fabric", ramMb: 4096 });
    const cfProfile = cfResult.profile || cfResult;
    assert.ok(cfProfile.gameDir);
    const manifest = JSON.parse(readFileSync(join(cfProfile.gameDir, "manifest.json")));
    const cfJars = readdirSync(join(cfProfile.gameDir, "mods")).filter(name => name.endsWith(".jar"));
    assert.ok(cfJars.length >= manifest.files.length);
    assert.ok(cfJars.every(name => statSync(join(cfProfile.gameDir, "mods", name)).size > 0));
    passed("real CurseForge import", `${cfVersion.name}; ${manifest.files.length} manifest entries, ${cfJars.length} JARs preserved`);

    const nested = join(root, "nested.mrpack");
    execFileSync("python3", ["-c", 'import zipfile,sys\nsrc,dst=sys.argv[1:]\nwith zipfile.ZipFile(src) as a,zipfile.ZipFile(dst,"w") as b:\n for f in a.infolist(): b.writestr("nested\\\\"+f.filename.replace("/","\\\\"),a.read(f))\n b.writestr("nested\\\\overrides\\\\config\\\\smoke.txt","base")\n b.writestr("nested\\\\client-overrides\\\\config\\\\smoke.txt","client")', pack, nested]);
    const nestedResult = await call("instance_import_mrpack", { filePath: nested, profileName: "Nested separators smoke" });
    assert.equal(readFileSync(join((nestedResult.profile || nestedResult).gameDir, "config/smoke.txt"), "utf8"), "client");
    passed("nested mrpack roots", "Windows separators normalized; client-overrides wins recursively");

    const beforeCancel = (await call("profiles_list")).length;
    const pending = call("instance_import_mrpack", { filePath: pack, profileName: "Cancelled smoke" }).then(() => ({ ok: true }), error => ({ error: error.message }));
    await new Promise(resolve => setTimeout(resolve, 30));
    await assert.rejects(call("instance_import_mrpack", { filePath: pack, profileName: "Concurrent smoke" }), /importação em andamento/);
    await call("instance_cancel_import");
    const cancelled = await pending;
    assert.match(cancelled.error, /cancelad/i);
    assert.equal((await call("profiles_list")).length, beforeCancel);
    passed("cancel and concurrent import", "second import rejected; cancellation leaves no incomplete profile");

    if (process.argv.includes("--launch")) {
        const account = await call("auth_offline_login", { username: "LuxmcSmoke" });
        renameSync(disabled + ".disabled", disabled);
        writeFileSync(damaged, "");
        for (const target of [profile, cfProfile]) {
        const launched = await call("launch_game", { request: { versionId: target.mcVersion, profileId: target.id, accountId: account.id, enableVulkan: false } });
        assert.ok(launched.pid > 0);
        runningGames.add(launched.pid);
        assert.ok(statSync(damaged).size > 0);
        passed("native launch", `${target.name}; JVM pid ${launched.pid}; prelaunch repaired zero-byte JAR`);
        await new Promise(resolve => setTimeout(resolve, 45000));
        const log = join(target.gameDir, "logs/latest.log");
        const text = existsSync(log) ? readFileSync(log, "utf8") : "";
        writeFileSync(join(root, `minecraft-${target.id}.log`), text);
        assert.match(text, /Backend library: LWJGL|OpenGL Vendor|Reloading ResourceManager|Sound engine started/);
        try { process.kill(launched.pid, "SIGTERM"); } catch {}
        runningGames.delete(launched.pid);
        passed("Minecraft startup", `${target.name}; renderer/resources initialized`);
        }
    }
    writeFileSync(join(root, "results.json"), JSON.stringify({ root, results }, null, 2));
    console.log(`Evidence: ${root}/results.json`);
} catch (error) {
    console.error(error);
    writeFileSync(join(root, "failure.log"), String(error) + "\n" + stderr);
    console.error(`Evidence: ${root}`);
    process.exitCode = 1;
} finally {
    for (const pid of runningGames) { try { process.kill(pid, "SIGTERM"); } catch {} }
    daemon.stdin.end(); daemon.kill();
}
