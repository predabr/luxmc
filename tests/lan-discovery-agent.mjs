import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve, delimiter, sep } from 'node:path';
import { spawnSync } from 'node:child_process';

const data = process.env.LUXMC_GAME_DATA;
assert.ok(data, 'Set LUXMC_GAME_DATA to the installed Minecraft cache');
const directory = mkdtempSync(join(tmpdir(), 'luxmc-lan-discovery-test-'));
const source = `
import java.io.*; import java.net.*; import java.nio.file.*; import java.lang.instrument.*; import java.lang.management.*; import java.lang.reflect.*; import java.util.*; import java.util.jar.*; import org.objectweb.asm.*;
public class LanDiscoveryProbe {
 public static class Binder implements Closeable {
  ServerSocket socket;
  public void bind(InetAddress address, int port) throws IOException { socket = new ServerSocket(port, 50, address); }
  public void close() throws IOException { if(socket != null) socket.close(); }
 }
 public static class ForgeIntegrated extends net.minecraft.server.MinecraftServer {
  Binder connection = new Binder();
  public boolean publish(int port) throws IOException { connection.bind(InetAddress.getLoopbackAddress(), port); System.out.println("Started serving on " + port); return true; }
 }
 public static class VanillaIntegrated extends net.minecraft.server.MinecraftServer {
  Binder connection = new Binder();
  public boolean publish(int port) throws IOException { connection.bind(InetAddress.getLoopbackAddress(), port); System.out.println("Published LAN server on port " + port); return true; }
 }
 public static class Unrelated {
  Binder connection = new Binder();
  public void bind(int port) throws IOException { connection.bind(InetAddress.getLoopbackAddress(), port); System.out.println("Started serving on " + port); }
 }
 static int freePort() throws IOException { try(ServerSocket socket = new ServerSocket(0)) { return socket.getLocalPort(); } }
 static void report(Path path, int port) throws Exception {
  Properties values = new Properties(); try(InputStream stream = Files.newInputStream(path)) { values.load(stream); }
  if(!values.getProperty("port").equals(Integer.toString(port))) throw new AssertionError("Wrong bound port");
  if(!values.getProperty("pid").equals(ManagementFactory.getRuntimeMXBean().getName().split("@",2)[0])) throw new AssertionError("Wrong process identity");
 }
 static void inspect(String archive, String className) throws Exception {
  final ClassFileTransformer[] captured = new ClassFileTransformer[1];
  Instrumentation collector = (Instrumentation) java.lang.reflect.Proxy.newProxyInstance(LanDiscoveryProbe.class.getClassLoader(), new Class<?>[]{Instrumentation.class}, (proxy, method, args) -> { if(method.getName().equals("addTransformer")) captured[0] = (ClassFileTransformer) args[0]; return null; });
  Class.forName("io.github.luxmc.client.LanAgent").getMethod("install",Instrumentation.class).invoke(null,collector);
  byte[] bytes; try(JarFile jar = new JarFile(archive); InputStream stream = jar.getInputStream(jar.getJarEntry(className + ".class")); ByteArrayOutputStream output = new ByteArrayOutputStream()) { byte[] chunk=new byte[8192]; int n; while((n=stream.read(chunk))>=0) output.write(chunk,0,n); bytes=output.toByteArray(); }
  byte[] transformed = captured[0].transform(LanDiscoveryProbe.class.getClassLoader(),className,null,null,bytes);
  if(transformed == null) throw new AssertionError("Actual integrated server was not recognized: " + archive);
  final int[] hooks = new int[2];
  new ClassReader(transformed).accept(new ClassVisitor(Opcodes.ASM9) {
   public MethodVisitor visitMethod(int access,String name,String desc,String signature,String[] exceptions) { return new MethodVisitor(Opcodes.ASM9) {
    public void visitMethodInsn(int opcode,String owner,String name,String desc,boolean itf) { if(owner.equals("io/github/luxmc/client/LanAgent")) { if(name.equals("prepare")) hooks[0]++; if(name.equals("opened")) hooks[1]++; } }
   }; }
  },ClassReader.SKIP_DEBUG);
  if(hooks[0] != 1 || hooks[1] != 1) throw new AssertionError("Unexpected actual server hook count: " + Arrays.toString(hooks));
  System.out.println("ACTUAL_BYTECODE_HOOK_OK: " + archive);
 }
 public static void main(String[] args) throws Exception {
  Path path=Paths.get(args[0]); System.setProperty("luxmc.lan.published",path.toString());
  Path session = path.resolveSibling("room.properties");
  System.setProperty("luxmc.p2p.session",session.toString());
  Files.createDirectories(path.getParent());
  Files.write(session,("expires="+(System.currentTimeMillis()+5000)+"\\nport=25565\\nmaxPlayers=20\\n").getBytes("UTF-8"));
  net.minecraft.server.players.PlayerList players = new net.minecraft.server.players.PlayerList(8);
  if(players.limit()!=20) throw new AssertionError("Pro room did not expand the integrated player limit");
  Files.write(session,("expires="+(System.currentTimeMillis()+5000)+"\\nport=25565\\nmaxPlayers=10\\n").getBytes("UTF-8"));
  if(players.limit()!=10) throw new AssertionError("Standard room capacity incorrect");
  Files.write(session,"expires=0\\nport=25565\\nmaxPlayers=20\\n".getBytes("UTF-8"));
  if(players.limit()!=8) throw new AssertionError("Expired room changed normal Minecraft capacity");
  System.clearProperty("luxmc.p2p.session");
  ForgeIntegrated forge = new ForgeIntegrated(); int port=freePort();
  if(!forge.publish(port)) throw new AssertionError("Forge publish failed"); report(path,port);
  try(Socket client=new Socket(InetAddress.getLoopbackAddress(),port); Socket accepted=forge.connection.socket.accept()) { client.getOutputStream().write(42); if(accepted.getInputStream().read()!=42) throw new AssertionError("Bound world is not reachable"); }
  forge.connection.close();
  byte[] before=Files.readAllBytes(path);
  try(ServerSocket occupied=new ServerSocket(0)) { try { new ForgeIntegrated().publish(occupied.getLocalPort()); throw new AssertionError("Expected bind failure"); } catch(IOException expected) {} }
  if(!Arrays.equals(before,Files.readAllBytes(path))) throw new AssertionError("A failed bind changed the active world");
  Unrelated other=new Unrelated(); other.bind(freePort()); other.connection.close();
  if(!Arrays.equals(before,Files.readAllBytes(path))) throw new AssertionError("An unrelated listener changed the active world");
  VanillaIntegrated vanilla=new VanillaIntegrated(); port=freePort(); vanilla.publish(port); report(path,port); vanilla.connection.close();
  for(int i=1;i<args.length;i+=2) inspect(args[i],args[i+1]);
  System.out.println("PASS: Forge and vanilla LAN reports use the bound TCP port and game PID; failed and unrelated binds are ignored");
 }
}`;

try {
    writeFileSync(join(directory, 'LanDiscoveryProbe.java'), source);
    const serverSource = join(directory, 'net/minecraft/server/MinecraftServer.java');
    mkdirSync(resolve(serverSource, '..'), { recursive: true });
    writeFileSync(serverSource, 'package net.minecraft.server; public class MinecraftServer {}');
    const playerSource = join(directory, 'net/minecraft/server/players/PlayerList.java');
    mkdirSync(resolve(playerSource, '..'), {recursive:true});
    writeFileSync(playerSource, 'package net.minecraft.server.players; public class PlayerList { private final int maxPlayers; public PlayerList(int maxPlayers) { this.maxPlayers=maxPlayers; } public int limit() { return maxPlayers; } }');
    const asm = resolve('src-tauri/client-agent/dependencies/asm-9.10.1.jar');
    const java17 = join(data, 'java/17/bin');
    let result = spawnSync(join(java17, 'javac.exe'), ['--release', '8', '-cp', asm, '-d', directory, serverSource, playerSource, join(directory, 'LanDiscoveryProbe.java')], { encoding: 'utf8', windowsHide: true });
    assert.equal(result.status, 0, result.stdout + result.stderr);
    const archives = [
        [join(data, 'versions/1.8.9/1.8.9.jar'), 'bpo'],
        [join(data, 'versions/1.20.1/1.20.1.jar'), 'fyp'],
        [join(data, 'libraries/net/minecraftforge/forge/1.20.1-47.4.10/forge-1.20.1-47.4.10-client.jar'), 'net/minecraft/client/server/IntegratedServer'],
        [join(data, 'versions/1.21.1/1.21.1.jar'), 'guo'],
        [join(data, 'versions/26.3/26.3.jar'), 'net/minecraft/client/server/IntegratedServer'],
    ];
    const availableArchives = archives.filter(([archive]) => existsSync(archive));
    assert.ok(availableArchives.length > 0, 'No installed Minecraft bytecode fixtures found');
    for (const runtime of [17, 25]) {
        const output = join(directory, `runtime-${runtime}`, 'lan.properties');
        mkdirSync(resolve(output, '..'), { recursive: true });
        result = spawnSync(join(data, `java/${runtime}/bin/java.exe`), ['-Djava.awt.headless=true', `-javaagent:${resolve('src-tauri/assets/luxmc-client-agent.jar')}=appearance-only`, '-cp', [directory, asm].join(delimiter), 'LanDiscoveryProbe', output, ...availableArchives.flat()], { encoding: 'utf8', windowsHide: true, timeout: 30000 });
        assert.equal(result.status, 0, result.stdout + result.stderr);
        assert.match(result.stdout, /PASS: Forge and vanilla LAN reports/);
        console.log(`Java ${runtime}: ${result.stdout.trim()}`);
    }
} finally {
    assert.ok(resolve(directory).startsWith(resolve(tmpdir()) + sep) && directory.includes('luxmc-lan-discovery-test-'));
    rmSync(directory, { recursive: true, force: true });
}
