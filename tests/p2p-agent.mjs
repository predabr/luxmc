import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve, sep } from "node:path";
import { spawnSync } from "node:child_process";

const directory = mkdtempSync(join(tmpdir(), "luxmc-p2p-agent-test-"));
const javaHome = process.env.LUXMC_JAVA_HOME || process.env.JAVA_HOME;
if (!javaHome) throw new Error("Set LUXMC_JAVA_HOME to a JDK before running this test.");
const sources = {
    "com/mojang/authlib/GameProfile.java": `package com.mojang.authlib; public class GameProfile {}`,
    "net/minecraft/server/MinecraftServer.java": `package net.minecraft.server; public class MinecraftServer { public boolean online() { return true; } public java.security.KeyPair key() { return new java.security.KeyPair(null, null); } }`,
    "Probe.java": `
import java.net.*; import java.nio.file.*; import com.mojang.authlib.GameProfile; import net.minecraft.server.MinecraftServer;
public class Probe {
 public static class Branch { public Branch next; public Branch( int remaining) { if (remaining > 0) next = new Branch(remaining - 1); } }
 public static class IntegratedServer extends MinecraftServer { public Branch[] worlds = new Branch[40]; public IntegratedServer() { for (int i=0;i<worlds.length;i++) worlds[i]=new Branch(3); } public Branch a=new Branch(3), b=new Branch(3), c=new Branch(3), d=new Branch(3), e=new Branch(3), f=new Branch(3), g=new Branch(3), h=new Branch(3), i=new Branch(3), j=new Branch(3), k=new Branch(3), l=new Branch(3), m=new Branch(3), n=new Branch(3), o=new Branch(3), p=new Branch(3); }
 public static class Channel {
  public InetSocketAddress remote = new InetSocketAddress("127.0.0.1", 50000);
  public InetSocketAddress local = new InetSocketAddress("127.0.0.1", 47123);
  public SocketAddress remoteAddress() { return remote; }
  public SocketAddress localAddress() { return local; }
 }
 public static class Login {
  public MinecraftServer server = new IntegratedServer(); public Channel channel = new Channel();
  public boolean hello() { GameProfile profile = new GameProfile(); boolean online = server.online(); if (online) server.key().getPublic(); return online; }
 }
 static void expect(boolean actual, boolean wanted) { if (actual != wanted) throw new AssertionError("Authentication mismatch: " + actual + " expected " + wanted); }
 public static void main(String[] args) throws Exception {
  Login login = new Login(); expect(login.hello(), true);
  Path file = Paths.get(args[0]); System.setProperty("luxmc.p2p.session", file.toString());
  Files.write(file, ("expires=" + (System.currentTimeMillis()+4000) + "\\nport=47123\\n").getBytes("UTF-8")); Thread.sleep(600);
  expect(login.hello(), false);
  login.channel.remote = new InetSocketAddress("203.0.113.10", 50000); expect(login.hello(), true);
  login.channel.remote = new InetSocketAddress("127.0.0.1", 50000); login.channel.local = new InetSocketAddress("127.0.0.1", 47124); expect(login.hello(), true);
  login.channel.local = new InetSocketAddress("127.0.0.1", 47123);
  Files.write(file, "expires=0\\nport=47123\\n".getBytes("UTF-8")); Thread.sleep(600); expect(login.hello(), true);
  if (Boolean.getBoolean("probe.official.skin")) {
   Class<?> appearance = Class.forName("io.github.luxmc.client.AppearanceAgent");
   java.lang.reflect.Method lookup = appearance.getDeclaredMethod("officialAppearance", String.class); lookup.setAccessible(true);
   if (lookup.invoke(null, "Notch") == null) throw new AssertionError("Official skin fallback failed");
   System.out.println("PASS: official public skin resolved for offline LAN profile");
  }
  String legacy = System.getProperty("probe.legacy.classpath", "");
  if (!legacy.isEmpty()) {
   String[] paths = legacy.split(java.io.File.pathSeparator); URL[] urls = new URL[paths.length];
   for (int i=0;i<paths.length;i++) urls[i] = Paths.get(paths[i]).toUri().toURL();
   try (URLClassLoader loader = new URLClassLoader(urls, null)) { Class.forName(System.getProperty("probe.minecraft.login.class", "lo"), false, loader); }
  }
  System.out.println("PASS: private loopback LAN only; external peers, other ports and expired rooms retain official authentication");
 }
}`
};
try {
    for (const [name, source] of Object.entries(sources)) {
        const path = join(directory, name); mkdirSync(resolve(path, ".."), { recursive: true }); writeFileSync(path, source);
    }
    const executable = name => join(javaHome, "bin", process.platform === "win32" ? `${name}.exe` : name);
    const compile = spawnSync(executable("javac"), ["--release", "8", "-d", directory, ...Object.keys(sources).map(name => join(directory, name))], { encoding: "utf8", windowsHide: true });
    assert.equal(compile.status, 0, compile.stderr);
    const legacy = process.env.LUXMC_LEGACY_CLASSPATH;
    const runtime = process.env.LUXMC_TEST_RUNTIME_HOME ? join(process.env.LUXMC_TEST_RUNTIME_HOME, "bin", process.platform === "win32" ? "java.exe" : "java") : executable("java");
    const loginClass = process.env.LUXMC_TEST_LOGIN_CLASS || "lo";
    const result = spawnSync(runtime, [`-javaagent:${resolve("src-tauri/assets/luxmc-client-agent.jar")}=appearance-only`, ...(legacy ? [`-Dprobe.legacy.classpath=${legacy}`, `-Dprobe.minecraft.login.class=${loginClass}`] : []), ...(process.env.LUXMC_TEST_OFFICIAL_SKIN === "1" ? ["-Dprobe.official.skin=true"] : []), "-cp", directory, "Probe", join(directory, "room.properties")], { encoding: "utf8", windowsHide: true, timeout: 30000 });
    assert.equal(result.status, 0, result.stdout + result.stderr);
    assert.match(result.stdout, /Private LAN login support: Probe\$Login/);
    if (legacy) assert.ok(result.stdout.includes(`Private LAN login support: ${loginClass.replaceAll(".", "/")}`), result.stdout);
    console.log(result.stdout.trim());
    const shutdown = spawnSync(runtime, [`-javaagent:${resolve("src-tauri/assets/luxmc-client-agent.jar")}`, "-cp", directory, "Probe", join(directory, "shutdown.properties")], { encoding: "utf8", windowsHide: true, timeout: 12000 });
    assert.equal(shutdown.status, 0, shutdown.stdout + shutdown.stderr + String(shutdown.error || ""));
    console.log("PASS: complete client agent releases its HUD and exits without a shutdown watchdog");
} finally {
    if (!resolve(directory).startsWith(resolve(tmpdir()) + sep) || !directory.includes("luxmc-p2p-agent-test-")) throw new Error("Unexpected test directory");
    rmSync(directory, { recursive: true, force: true });
}
