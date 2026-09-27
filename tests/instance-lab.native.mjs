import { spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, existsSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import assert from "node:assert/strict";

const root = mkdtempSync(join(tmpdir(), "luxmc-lab-native-"));
const binary = resolve(process.env.LUXMC_NATIVE_BINARY || "src-tauri/target/release/luxmc");
const daemon = spawn(binary, ["--daemon"], { env: { ...process.env, XDG_DATA_HOME: join(root, "data"), XDG_CONFIG_HOME: join(root, "config"), XDG_CACHE_HOME: join(root, "cache") }, stdio: ["pipe", "pipe", "pipe"] });
const pending = new Map();
let sequence = 0;
let stderr = "";
daemon.stderr.on("data", chunk => { stderr = (stderr + chunk).slice(-4000); });
createInterface({ input: daemon.stdout }).on("line", line => {
    let reply;
    try { reply = JSON.parse(line); } catch { return; }
    const item = pending.get(reply.id);
    if (!item) return;
    pending.delete(reply.id);
    clearTimeout(item.timer);
    if (reply.error) item.reject(new Error(reply.error)); else item.resolve(reply.result);
});
daemon.on("exit", code => {
    for (const item of pending.values()) { clearTimeout(item.timer); item.reject(new Error(`Daemon exited ${code}: ${stderr}`)); }
    pending.clear();
});
const call = (command, args = {}) => new Promise((resolveCall, rejectCall) => {
    const id = ++sequence;
    const timer = setTimeout(() => { pending.delete(id); rejectCall(new Error(`Timeout: ${command}`)); }, 30000);
    pending.set(id, { resolve: resolveCall, reject: rejectCall, timer });
    daemon.stdin.write(JSON.stringify({ id, command, args }) + "\n");
});

try {
    const profile = await call("profiles_create", { input: { name: "Laboratório", mcVersion: "1.20.1", loader: "fabric", ramMb: 4096 } });
    const mods = join(profile.gameDir, "mods");
    mkdirSync(mods, { recursive: true });
    mkdirSync(join(profile.gameDir, "saves", "World"), { recursive: true });
    writeFileSync(join(mods, "alpha.jar"), "alpha-original");
    writeFileSync(join(mods, "beta.jar"), "beta-original");
    writeFileSync(join(profile.gameDir, "saves", "World", "level.dat"), "world-original");
    writeFileSync(join(profile.gameDir, "options.txt"), "renderDistance:12");

    const trial = await call("instance_isolation_start", { profileId: profile.id });
    assert.equal(trial.phase, "testing");
    assert.equal(trial.trialDisabled.length, 1);
    assert.ok(existsSync(join(mods, `${trial.trialDisabled[0]}.disabled`)));
    await assert.rejects(call("instance_mod_delete", { profileId: profile.id, fileName: "beta.jar" }), /diagnóstico/);
    const found = await call("instance_isolation_report", { profileId: profile.id, crashed: false });
    assert.equal(found.phase, "found");
    assert.equal(found.suspects.length, 1);
    assert.ok(existsSync(join(mods, "alpha.jar")));
    assert.ok(existsSync(join(mods, "beta.jar")));
    await call("instance_isolation_restore", { profileId: profile.id });
    assert.equal(await call("instance_isolation_status", { profileId: profile.id }), null);

    const capsule = await call("instance_capsule_create", { profileId: profile.id, label: "Original" });
    await call("profiles_update", { input: { id: profile.id, ramMb: 8192 } });
    writeFileSync(join(mods, "alpha.jar"), "alpha-changed");
    writeFileSync(join(profile.gameDir, "saves", "World", "level.dat"), "world-changed");
    writeFileSync(join(profile.gameDir, "options.txt"), "renderDistance:2");
    const previous = await call("instance_capsule_restore", { profileId: profile.id, filename: capsule.filename });
    assert.equal(readFileSync(join(mods, "alpha.jar"), "utf8"), "alpha-original");
    assert.equal(readFileSync(join(profile.gameDir, "saves", "World", "level.dat"), "utf8"), "world-original");
    assert.equal(readFileSync(join(profile.gameDir, "options.txt"), "utf8"), "renderDistance:12");
    assert.equal((await call("profiles_get", { id: profile.id })).ramMb, profile.ramMb);
    await call("instance_capsule_restore", { profileId: profile.id, filename: previous.filename });
    assert.equal(readFileSync(join(mods, "alpha.jar"), "utf8"), "alpha-changed");

    const csv = join(root, "mangohud.csv");
    writeFileSync(csv, "fps,frametime\n" + "100,10\n".repeat(98) + "20,80\n20,80\n");
    const benchmark = await call("instance_benchmark_import", { profileId: profile.id, label: "Antes", csvPath: csv });
    assert.equal(benchmark.fpsDrops, 2);
    assert.equal((await call("instance_benchmarks_list", { profileId: profile.id })).length, 1);
    console.log(JSON.stringify({ isolation: found.suspects[0], capsule: capsule.filename, reversible: true, fpsDrops: benchmark.fpsDrops }));
} finally {
    daemon.kill();
    rmSync(root, { recursive: true, force: true });
}
