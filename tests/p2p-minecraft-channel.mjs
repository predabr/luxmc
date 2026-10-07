import assert from 'node:assert/strict';
import {mkdtempSync, readFileSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join, resolve, delimiter} from 'node:path';
import {spawnSync} from 'node:child_process';

const data = process.env.LUXMC_GAME_DATA;
assert.ok(data, 'Set LUXMC_GAME_DATA to the installed Minecraft cache');
const version = '26.3';
const manifest = JSON.parse(readFileSync(join(data, 'versions', version, `${version}.json`)));
const directory = mkdtempSync(join(tmpdir(), 'luxmc-real-channel-'));
const java = join(data, 'java', '25', 'bin');
const source = `
import java.lang.reflect.*; import java.net.*; import java.nio.channels.*; import java.nio.file.*;
public class RealChannel {
 static Object allocate(Class<?> type) throws Exception { Class<?> u=Class.forName("sun.misc.Unsafe"); Field f=u.getDeclaredField("theUnsafe"); f.setAccessible(true); return u.getMethod("allocateInstance",Class.class).invoke(f.get(null),type); }
 static void set(Object value,String name,Object child) throws Exception { Field f=value.getClass().getDeclaredField(name); f.setAccessible(true); f.set(value,child); }
 public static void main(String[] args) throws Exception {
  Class.forName("net.minecraft.SharedConstants").getMethod("tryDetectVersion").invoke(null);
  Class.forName("net.minecraft.server.Bootstrap").getMethod("bootStrap").invoke(null);
  try(ServerSocketChannel listener=ServerSocketChannel.open()) {
   listener.bind(new InetSocketAddress("127.0.0.1",0)); int port=((InetSocketAddress)listener.getLocalAddress()).getPort();
   try(SocketChannel client=SocketChannel.open(new InetSocketAddress("127.0.0.1",port)); SocketChannel accepted=listener.accept()) {
    Object channel=Class.forName("io.netty.channel.socket.nio.NioSocketChannel").getConstructor(SocketChannel.class).newInstance(accepted);
    Object connection=allocate(Class.forName("net.minecraft.network.Connection")); set(connection,"channel",channel);
    Object login=allocate(Class.forName("net.minecraft.server.network.ServerLoginPacketListenerImpl"));
    set(login,"connection",connection); set(login,"server",allocate(Class.forName("net.minecraft.client.server.IntegratedServer")));
    Path file=Paths.get(args[0]); System.setProperty("luxmc.p2p.session",file.toString());
    Files.write(file,("expires="+(System.currentTimeMillis()+4000)+"\\nport="+port+"\\n").getBytes("UTF-8"));
    Method authenticate=Class.forName("io.github.luxmc.client.P2PAgent").getMethod("authenticate",boolean.class,Object.class);
    if ((Boolean)authenticate.invoke(null,true,login)) throw new AssertionError("Actual Minecraft/Netty private connection was not recognized");
    Files.write(file,("expires="+(System.currentTimeMillis()+4000)+"\\nport="+(port==65535?port-1:port+1)+"\\n").getBytes("UTF-8")); Thread.sleep(600);
    if (!(Boolean)authenticate.invoke(null,true,login)) throw new AssertionError("Unrelated port lost authentication");
    Files.write(file,"expires=0\\nport=0\\n".getBytes("UTF-8")); Thread.sleep(600);
    if (!(Boolean)authenticate.invoke(null,true,login)) throw new AssertionError("Expired room lost authentication");
    System.out.println("PASS: actual Minecraft 26.3 integrated server/login/Connection and actual Netty TCP channel; private port only, expired session rejected");
   }
  }
 }
}`;
writeFileSync(join(directory,'RealChannel.java'),source);
let result=spawnSync(join(java,'javac.exe'),['-d',directory,join(directory,'RealChannel.java')],{encoding:'utf8',windowsHide:true});
assert.equal(result.status,0,result.stderr);
const libraries=manifest.libraries.filter(lib=>!lib.rules || lib.rules.some(rule=>rule.action==='allow' && (!rule.os || rule.os.name==='windows'))).map(lib=> {
 const [group,name,version,classifier]=lib.name.split(':');
 return join(data,'libraries',lib.downloads.artifact.path || `${group.replaceAll('.','/')}/${name}/${version}/${name}-${version}${classifier?'-'+classifier:''}.jar`);
});
const classpath=[directory,join(data,'versions',version,`${version}.jar`),...libraries].join(delimiter);
result=spawnSync(join(java,'java.exe'),[`-javaagent:${resolve('src-tauri/assets/luxmc-client-agent.jar')}=appearance-only`,'-cp',classpath,'RealChannel',join(directory,'room.properties')],{encoding:'utf8',windowsHide:true,timeout:60000});
assert.equal(result.status,0,result.stdout+result.stderr);
console.log(result.stdout.trim());
