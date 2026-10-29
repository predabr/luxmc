use crate::error::{AppError, AppResult};
use serde::{Deserialize,Serialize};
use sha1::{Digest,Sha1};
use std::{collections::{BTreeMap,HashSet},path::PathBuf,sync::{Arc,LazyLock},time::Duration};
use tokio::{io::{AsyncReadExt,AsyncWriteExt},net::{TcpListener,TcpStream},sync::{Mutex,Semaphore}};

const LIMIT:u64 = 128 * 1024 * 1024;
static FILES:LazyLock<Mutex<BTreeMap<String,Asset>>> = LazyLock::new(||Mutex::new(BTreeMap::new()));
static PORTS:LazyLock<Mutex<BTreeMap<u16,tokio::task::JoinHandle<()>>>> = LazyLock::new(||Mutex::new(BTreeMap::new()));
static REQUESTS:LazyLock<Arc<Semaphore>> = LazyLock::new(||Arc::new(Semaphore::new(8)));
static LIFECYCLE:Mutex<()> = Mutex::const_new(());

#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct PackDescriptor { pub sha1:String, pub bytes:u64, pub port:u16 }

impl PackDescriptor {
    pub fn valid(&self)->bool { self.sha1.len()==40 && self.sha1.bytes().all(|byte|byte.is_ascii_hexdigit()) && self.bytes>0 && self.bytes<=LIMIT && (20000..40000).contains(&self.port) }
    pub fn url(&self)->String { format!("http://127.0.0.1:{}/luxmc/packs/{}.zip",self.port,self.sha1) }
}

fn pack_port(hash:&str)->u16 { 20000 + u16::from_str_radix(hash.get(..4).unwrap_or(""),16).unwrap_or(0)%20000 }
pub fn is_managed_url(value:&str,hash:&str)->bool {hash.len()==40 && hash.bytes().all(|byte|byte.is_ascii_hexdigit()) && url::Url::parse(value).is_ok_and(|url|url.scheme()=="http" && url.host_str()==Some("127.0.0.1") && url.port().is_some_and(|port|(20000..40000).contains(&port)) && url.path()==format!("/luxmc/packs/{hash}.zip") && url.query().is_none() && url.fragment().is_none() && url.username().is_empty() && url.password().is_none())}
fn error(value:impl ToString)->AppError {AppError::InvalidState(value.to_string())}
#[derive(Clone)]
enum Source { Local{server:String,path:PathBuf}, Remote{connection:iroh::endpoint::Connection,secret:[u8;32]} }
#[derive(Clone)]
struct Asset { description:PackDescriptor, source:Source }

async fn listen(port:u16)->AppResult<()> {
    let mut ports=PORTS.lock().await;
    if ports.contains_key(&port) { return Ok(()); }
    let listener=TcpListener::bind(("127.0.0.1",port)).await.map_err(|_|error(format!("A porta {port} para transferir texturas está ocupada. Feche o aplicativo que a utiliza e tente novamente.")))?;
    let task=tokio::spawn(async move {
        loop {
            let Ok((tcp,_))=listener.accept().await else {break};
            let Ok(permit)=REQUESTS.clone().try_acquire_owned() else {continue};
            tokio::spawn(async move {let _permit=permit;let _=tokio::time::timeout(Duration::from_secs(600),http(tcp,port)).await;});
        }
    });
    ports.insert(port,task);
    Ok(())
}

pub async fn host(server:&str,path:PathBuf)->AppResult<PackDescriptor> {
    host_asset(server,path,true).await
}
#[cfg(test)]
pub(crate) async fn host_transfer_fixture(server:&str,path:PathBuf)->AppResult<PackDescriptor> {host_asset(server,path,false).await}

async fn host_asset(server:&str,path:PathBuf,serve_http:bool)->AppResult<PackDescriptor> {
    let input=path.clone();
    let (hash,bytes)=tokio::task::spawn_blocking(move ||->AppResult<_> {
        let metadata=std::fs::symlink_metadata(&input)?;
        if crate::core::mods::pack_download::metadata_is_link(&metadata) || !metadata.is_file() || metadata.len()==0 || metadata.len()>LIMIT {return Err(error("O pacote de texturas deve ter até 128 MiB e não pode ser um link."));}
        let mut file=std::fs::File::open(input)?;let mut hash=Sha1::new();let mut buffer=[0u8;65536];
        loop {let count=std::io::Read::read(&mut file,&mut buffer)?;if count==0 {break};hash.update(&buffer[..count]);}
        Ok((format!("{:x}",hash.finalize()),metadata.len()))
    }).await.map_err(error)??;
    let mut description=PackDescriptor{port:pack_port(&hash),sha1:hash.clone(),bytes};
    let _guard=LIFECYCLE.lock().await;
    if serve_http {
        let mut ready=false;
        for offset in 0..32 {description.port=20000+(pack_port(&hash)-20000+offset)%20000;if listen(description.port).await.is_ok(){ready=true;break;}}
        if !ready{return Err(error("Não foi possível reservar uma porta local para as texturas."));}
    }
    let mut files=FILES.lock().await;files.retain(|_,asset|!matches!(&asset.source,Source::Local{server:owner,..} if owner==server));
    files.insert(hash,Asset{description:description.clone(),source:Source::Local{server:server.into(),path}});drop(files);cleanup_ports().await;
    Ok(description)
}

pub async fn hosted(server:&str)->Option<PackDescriptor> {FILES.lock().await.values().find_map(|asset|matches!(&asset.source,Source::Local{server:owner,..} if owner==server).then(||asset.description.clone()))}
pub async fn remote(connection:iroh::endpoint::Connection,secret:[u8;32],description:PackDescriptor)->AppResult<()> {
    let _guard=LIFECYCLE.lock().await;
    if !description.valid() {return Err(error("Pacote de texturas anunciado inválido."));}
    listen(description.port).await?;
    let mut files=FILES.lock().await;
    if files.get(&description.sha1).is_some_and(|asset|matches!(asset.source,Source::Local{..})) {return Ok(());}
    if files.len()>=32 && !files.contains_key(&description.sha1) {return Err(error("Pacotes de texturas ativos demais."));}
    files.insert(description.sha1.clone(),Asset{description,source:Source::Remote{connection,secret}});Ok(())
}
async fn cleanup_ports(){let used=FILES.lock().await.values().map(|asset|asset.description.port).collect::<HashSet<_>>();PORTS.lock().await.retain(|port,task|{if used.contains(port){true}else{task.abort();false}});}
pub async fn clear_remote() {let _guard=LIFECYCLE.lock().await;FILES.lock().await.retain(|_,asset|matches!(asset.source,Source::Local{..}));cleanup_ports().await;}
pub async fn clear_host(server:&str) {let _guard=LIFECYCLE.lock().await;FILES.lock().await.retain(|_,asset|!matches!(&asset.source,Source::Local{server:owner,..} if owner==server));cleanup_ports().await;}

async fn http(mut tcp:TcpStream,port:u16)->AppResult<()> {
    let mut request=Vec::new();let mut byte=[0u8;1];
    loop {
        if request.len()>=4096 {return Err(error("Pedido de pacote muito grande."));}
        tokio::time::timeout(Duration::from_secs(5),tcp.read_exact(&mut byte)).await.map_err(error)??;
        request.push(byte[0]);if request.ends_with(b"\r\n\r\n") {break;}
    }
    let line=std::str::from_utf8(&request).map_err(error)?.lines().next().unwrap_or("");
    let mut parts=line.split_whitespace();let method=parts.next().unwrap_or("");let path=parts.next().unwrap_or("");
    let hash=path.strip_prefix("/luxmc/packs/").and_then(|path|path.strip_suffix(".zip")).unwrap_or("");
    let asset=FILES.lock().await.get(hash).filter(|asset|asset.description.port==port).cloned();
    if !["GET","HEAD"].contains(&method) || asset.is_none() {tcp.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").await?;return Ok(());}
    let asset=asset.unwrap();
    let mut remote=match &asset.source {
        Source::Remote{connection,secret} if method=="GET"=>{
            let (mut send,mut recv)=connection.open_bi().await.map_err(error)?;
            let mut header=[10u8;33];header[1..].copy_from_slice(secret);send.write_all(&header).await.map_err(error)?;send.write_all(hash.as_bytes()).await.map_err(error)?;let _=send.finish();
            let mut response=[0u8;9];recv.read_exact(&mut response).await.map_err(error)?;
            if response[0]!=1 || u64::from_be_bytes(response[1..].try_into().unwrap())!=asset.description.bytes {return Err(error("O anfitrião não disponibilizou este pacote de texturas."));}
            Some(recv)
        },_=>None
    };
    let header=format!("HTTP/1.1 200 OK\r\nContent-Type: application/zip\r\nContent-Length: {}\r\nCache-Control: private, max-age=86400, immutable\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\n\r\n",asset.description.bytes);
    tcp.write_all(header.as_bytes()).await?;
    if method=="HEAD" {return Ok(());}
    if let Some(recv)=&mut remote {
        let mut remaining=asset.description.bytes;let mut bytes=vec![0u8;65536];
        while remaining>0 {let count=remaining.min(bytes.len() as u64) as usize;recv.read_exact(&mut bytes[..count]).await.map_err(error)?;tcp.write_all(&bytes[..count]).await?;remaining-=count as u64;}
    } else if let Source::Local{path,..}=asset.source {let mut file=tokio::fs::File::open(path).await?;tokio::io::copy(&mut (&mut file).take(asset.description.bytes),&mut tcp).await?;}
    Ok(())
}

pub async fn transfer(mut send:iroh::endpoint::SendStream,mut recv:iroh::endpoint::RecvStream)->AppResult<()> {
    let _permit=REQUESTS.clone().try_acquire_owned().map_err(error)?;
    let mut bytes=[0u8;40];recv.read_exact(&mut bytes).await.map_err(error)?;
    let hash=std::str::from_utf8(&bytes).map_err(error)?;
    let asset=FILES.lock().await.get(hash).filter(|asset|matches!(asset.source,Source::Local{..})).cloned();
    let Some(Asset{description,source:Source::Local{path,..}})=asset else {send.write_all(&[0]).await.map_err(error)?;return Ok(());};
    let mut file=tokio::fs::File::open(path).await?;
    send.write_all(&[1]).await.map_err(error)?;send.write_all(&description.bytes.to_be_bytes()).await.map_err(error)?;
    let mut remaining=description.bytes;let mut buffer=[0u8;65536];
    while remaining>0 {let count=file.read(&mut buffer[..remaining.min(65536) as usize]).await?;if count==0 {return Err(error("Pacote alterado durante a transferência."));}send.write_all(&buffer[..count]).await.map_err(error)?;remaining-=count as u64;}
    let _=send.finish();Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn local_pack_http_is_hash_addressed_bounded_and_never_serves_arbitrary_paths() {
        let temp=tempfile::tempdir().unwrap();let path=temp.path().join("pack.zip");let payload=b"resource-pack-http-fixture";tokio::fs::write(&path,payload).await.unwrap();
        let pack=host("fixture-http",path).await.unwrap();assert!(pack.valid());
        let http=reqwest::Client::new();let response=http.get(pack.url()).send().await.unwrap();assert_eq!(response.status(),200);assert_eq!(response.bytes().await.unwrap().as_ref(),payload);
        assert_eq!(http.get(format!("http://127.0.0.1:{}/outside.zip",pack.port)).send().await.unwrap().status(),404);
        assert_eq!(http.post(pack.url()).send().await.unwrap().status(),404);
        let mut bad=pack.clone();bad.port=80;assert!(!bad.valid());bad.sha1="../outside".into();assert!(!bad.valid());
        clear_host("fixture-http").await;assert!(!FILES.lock().await.contains_key(&pack.sha1));assert!(!PORTS.lock().await.contains_key(&pack.port));
    }
}
