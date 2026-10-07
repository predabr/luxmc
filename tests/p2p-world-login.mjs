import assert from 'node:assert/strict';
import {cpSync, mkdtempSync, mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join, resolve, delimiter} from 'node:path';
import {spawnSync} from 'node:child_process';

const data=process.env.LUXMC_GAME_DATA;
const world=process.env.LUXMC_TEST_WORLD;
assert.ok(data && world, 'Set LUXMC_GAME_DATA and LUXMC_TEST_WORLD');
const directory=mkdtempSync(join(tmpdir(),'luxmc-world-login-'));
mkdirSync(join(directory,'saves'),{recursive:true});
cpSync(world,join(directory,'saves','P2P Regression'),{recursive:true});
writeFileSync(join(directory,'options.txt'),'renderDistance:4\nsimulationDistance:4\nmaxFps:30\nfullscreen:false\n');
const source=`
import java.io.*; import java.net.*; import java.nio.file.*; import java.util.*; import java.lang.instrument.*; import java.util.concurrent.*;
public class GameProbe {
 static void vi(DataOutputStream out,int v)throws Exception {do{int b=v&127;v>>>=7;out.writeByte(v==0?b:b|128);}while(v!=0);}
 static int vi(DataInputStream in)throws Exception {int v=0;for(int i=0;i<5;i++){int b=in.readUnsignedByte();v|=(b&127)<<(7*i);if((b&128)==0)return v;}throw new IOException("Bad varint");}
 static void str(DataOutputStream out,String s)throws Exception {byte[] b=s.getBytes("UTF-8");vi(out,b.length);out.write(b);}
 static void packet(OutputStream out,byte[] b)throws Exception {DataOutputStream d=new DataOutputStream(out);vi(d,b.length);d.write(b);d.flush();}
 public static void premain(String args,Instrumentation instrumentation){
  Thread t=new Thread(()->{Object mc=null;try {
   Class<?> minecraft=Class.forName("net.minecraft.client.Minecraft"); Object server=null;
   for(int i=0;i<1200;i++){mc=minecraft.getMethod("getInstance").invoke(null);if(mc!=null)server=minecraft.getMethod("getSingleplayerServer").invoke(mc);if(server!=null && minecraft.getMethod("getConnection").invoke(mc)!=null && (Boolean)server.getClass().getMethod("isReady").invoke(server))break;Thread.sleep(100);}
   if(server==null)throw new AssertionError("No integrated server");
   if(!(Boolean)server.getClass().getMethod("usesAuthentication").invoke(server))throw new AssertionError("Authentication must be enabled for regression");
   Class<?> scope=Class.forName("net.minecraft.server.MinecraftServer$MultiplayerScope"); Object lan=scope.getField("LAN").get(null);
   int port;try(ServerSocket s=new ServerSocket(0)){port=s.getLocalPort();}
   final Object host=server; final int p=port; CompletableFuture<Boolean> published=new CompletableFuture<>();
   minecraft.getMethod("execute",Runnable.class).invoke(mc,(Runnable)()->{try{published.complete((Boolean)host.getClass().getMethod("publishServer",scope,int.class).invoke(host,lan,p));}catch(Throwable e){published.completeExceptionally(e);}});
   if(!published.get(20,TimeUnit.SECONDS))throw new AssertionError("Cannot open LAN");
   final Path file=Paths.get(System.getProperty("luxmc.p2p.session"));
   Thread heartbeat=new Thread(()->{try{while(true){Files.write(file,("expires="+(System.currentTimeMillis()+4000)+"\\nport="+p+"\\n").getBytes("UTF-8"));Thread.sleep(200);}}catch(Exception ignored){}});heartbeat.setDaemon(true);heartbeat.start(); Thread.sleep(600);
   int protocol=(Integer)Class.forName("net.minecraft.SharedConstants").getField("RELEASE_NETWORK_PROTOCOL_VERSION").get(null);
   try(Socket socket=new Socket("127.0.0.1",port)){socket.setSoTimeout(20000); ByteArrayOutputStream bytes=new ByteArrayOutputStream();DataOutputStream out=new DataOutputStream(bytes);
    vi(out,0);vi(out,protocol);str(out,"localhost");out.writeShort(port);vi(out,2);packet(socket.getOutputStream(),bytes.toByteArray());
    bytes.reset();vi(out,0);str(out,"LuxmcProbe");UUID uuid=UUID.nameUUIDFromBytes("OfflinePlayer:LuxmcProbe".getBytes("UTF-8"));out.writeLong(uuid.getMostSignificantBits());out.writeLong(uuid.getLeastSignificantBits());packet(socket.getOutputStream(),bytes.toByteArray());
    DataInputStream in=new DataInputStream(socket.getInputStream());boolean compression=false,success=false;
    for(int i=0;i<5;i++){int length=vi(in);byte[] payload=new byte[length];in.readFully(payload);DataInputStream response=new DataInputStream(new ByteArrayInputStream(payload));if(compression && vi(response)!=0)throw new AssertionError("Unexpected compressed login packet");int id=vi(response);if(id==1)throw new AssertionError("Official encryption request still sent to Luxmc private peer");if(id==0)throw new AssertionError("Login disconnected: "+new String(payload,"UTF-8"));if(id==3)compression=true;if(id==2){success=true;break;}}
    if(!success)throw new AssertionError("No login success");
    System.out.println("LUXMC_WORLD_LOGIN_PASS: actual Minecraft LAN login succeeded for private offline profile while server authentication was enabled");
   }
  }catch(Throwable e){e.printStackTrace();System.out.println("LUXMC_WORLD_LOGIN_FAIL");}finally{try{if(mc!=null){final Object target=mc;mc.getClass().getMethod("execute",Runnable.class).invoke(mc,(Runnable)()->{try{target.getClass().getMethod("stop").invoke(target);}catch(Exception e){e.printStackTrace();}});}}catch(Exception e){e.printStackTrace();}}},"Luxmc-World-Regression");t.setDaemon(true);t.start();
 }
}`;
writeFileSync(join(directory,'GameProbe.java'),source);
writeFileSync(join(directory,'MANIFEST.MF'),'Manifest-Version: 1.0\nPremain-Class: GameProbe\n\n');
const java=join(data,'java','25','bin');
let result=spawnSync(join(java,'javac.exe'),['-d',directory,join(directory,'GameProbe.java')],{encoding:'utf8',windowsHide:true});
assert.equal(result.status,0,result.stderr);
result=spawnSync(join(java,'jar.exe'),['cfm',join(directory,'probe.jar'),join(directory,'MANIFEST.MF'),'-C',directory,'GameProbe.class'],{encoding:'utf8',windowsHide:true});
assert.equal(result.status,0,result.stderr);
const manifest=JSON.parse(readFileSync(join(data,'versions','26.3','26.3.json')));
const libraries=manifest.libraries.filter(lib=>!lib.rules || lib.rules.some(rule=>rule.action==='allow' && (!rule.os || rule.os.name==='windows'))).map(lib=>{const[group,name,version,classifier]=lib.name.split(':');return join(data,'libraries',lib.downloads.artifact.path||
 `${group.replaceAll('.','/')}/${name}/${version}/${name}-${version}${classifier?'-'+classifier:''}.jar`);});
const classpath=[join(data,'versions','26.3','26.3.jar'),...libraries].join(delimiter);
result=spawnSync(join(java,'java.exe'),['-Xms512M','-Xmx2G','--enable-native-access=ALL-UNNAMED','--add-exports','java.base/jdk.internal.misc=ALL-UNNAMED',`-javaagent:${resolve('src-tauri/assets/luxmc-client-agent.jar')}`,`-javaagent:${join(directory,'probe.jar')}`,`-Dluxmc.p2p.session=${join(directory,'room.properties')}`,'-cp',classpath,'net.minecraft.client.main.Main','--username','LuxmcHost','--version','26.3','--gameDir',directory,'--assetsDir',join(data,'assets'),'--assetIndex',manifest.assetIndex.id,'--uuid','00000000000000000000000000000001','--accessToken','0','--versionType','release','--width','854','--height','480','--quickPlaySingleplayer','P2P Regression'],{encoding:'utf8',windowsHide:true,timeout:180000,maxBuffer:8*1024*1024});
writeFileSync(join(directory,'test-output.log'),result.stdout+result.stderr);
console.log(`Evidence: ${directory}`);
assert.ok(result.stdout.includes('LUXMC_WORLD_LOGIN_PASS'),result.stdout.slice(-7000)+result.stderr.slice(-1500));
assert.equal(result.status,0,result.stdout.slice(-3500)+String(result.error||''));
console.log('PASS: actual LAN login and clean Minecraft shutdown');
