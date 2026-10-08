use super::*;
use crate::commands::studio::RoomRecipe;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Preparation {
    pub phase:String,
    pub percent:u8,
    pub owner_id:String,
    pub fingerprint:String,
}

impl Preparation {
    pub fn valid(&self) -> bool {
        ["idle","planning","downloading","validating","ready","failed"].contains(&self.phase.as_str()) && self.percent<=100 && self.owner_id.len()<=128 && (self.fingerprint.is_empty() || self.fingerprint.len()==64 && self.fingerprint.chars().all(|ch| ch.is_ascii_hexdigit()))
    }
}

static LOCAL: LazyLock<Mutex<Option<Preparation>>> = LazyLock::new(|| Mutex::new(None));
pub(super) async fn reset(){*LOCAL.lock().await=None;}
pub(super) fn normalize(value:&mut Option<Preparation>,worlds:&[RoomWorld]) {
    let Some(preparation)=value.as_mut() else{return;};
    let Some(world)=worlds.iter().find(|world|world.owner_id==preparation.owner_id) else{*value=None;return;};
    if preparation.phase=="ready" && world.compatibility.as_ref().is_none_or(|compatibility|compatibility.mod_fingerprint!=preparation.fingerprint) {preparation.phase="failed".into();preparation.percent=0;}
}

async fn write_value(mut send: iroh::endpoint::SendStream, value: serde_json::Value) {
    if let Ok(bytes) = serde_json::to_vec(&value) {
        if bytes.len()<=1024*1024 && send.write_all(&(bytes.len() as u32).to_be_bytes()).await.is_ok() { let _=send.write_all(&bytes).await; let _=send.finish(); }
    }
}

pub(super) async fn local_recipe(send:iroh::endpoint::SendStream) {
    let value = match crate::commands::studio::active_recipe().await {
        Ok(recipe) => serde_json::json!({"ok":true,"recipe":recipe}),
        Err(error) => serde_json::json!({"ok":false,"error":error.to_string()}),
    };
    write_value(send,value).await;
}

async fn remote_value(connection:&iroh::endpoint::Connection,secret:[u8;32],owner:&str) -> AppResult<serde_json::Value> {
    let (mut send,mut recv)=connection.open_bi().await.map_err(failure)?;
    let mut header=[7u8;33];header[1..].copy_from_slice(&secret);
    send.write_all(&header).await.map_err(failure)?;
    send.write_all(&[owner.len() as u8]).await.map_err(failure)?;
    send.write_all(owner.as_bytes()).await.map_err(failure)?; let _=send.finish();
    let mut length=[0;4];recv.read_exact(&mut length).await.map_err(failure)?;
    let length=u32::from_be_bytes(length) as usize;
    if length>1024*1024 {return Err(failure("Receita da sala acima do limite"));}
    let mut bytes=vec![0;length];recv.read_exact(&mut bytes).await.map_err(failure)?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub(super) async fn control(op:u8,id:&str,send:iroh::endpoint::SendStream,mut recv:iroh::endpoint::RecvStream,registry:Arc<Mutex<RoomRegistry>>) {
    if op==8 {
        let mut length=[0;2];if recv.read_exact(&mut length).await.is_err(){return;}
        let length=u16::from_be_bytes(length) as usize;if length>512{return;}
        let mut bytes=vec![0;length];if recv.read_exact(&mut bytes).await.is_err(){return;}
        let Ok(value)=serde_json::from_slice::<Preparation>(&bytes) else{return;};if !value.valid(){return;}
        if let Some(member)=registry.lock().await.members.get_mut(id){member.preparation=Some(value);}
    } else {
        let mut length=[0];if recv.read_exact(&mut length).await.is_err() || length[0]>128{return;}
        let mut bytes=vec![0;length[0] as usize];if recv.read_exact(&mut bytes).await.is_err(){return;}
        let Ok(owner)=String::from_utf8(bytes) else{return;};
        let (host,target,allowed)={let room=registry.lock().await;(room.host.as_ref().is_some_and(|host| host.id==owner),room.connections.get(&owner).cloned().zip(room.member_secrets.get(&owner).copied()),room.worlds.contains_key(&owner))};
        if !allowed {write_value(send,serde_json::json!({"ok":false,"error":"O mundo LAN não está mais aberto"})).await;return;}
        if host {local_recipe(send).await;}
        else if let Some((connection,secret))=target {
            let value=match tokio::time::timeout(Duration::from_secs(30),remote_value(&connection,secret,"")).await {Ok(Ok(value))=>value,_=>serde_json::json!({"ok":false,"error":"O dono do mundo não respondeu à receita"})};
            write_value(send,value).await;
        }
    }
}

pub(super) async fn publish(connection:&iroh::endpoint::Connection,secret:[u8;32]) {
    let Some(value)=LOCAL.lock().await.clone() else{return;};
    let Ok(bytes)=serde_json::to_vec(&value) else{return;};
    let Ok((mut send,_))=connection.open_bi().await else{return;};
    let mut header=[8u8;33];header[1..].copy_from_slice(&secret);
    if send.write_all(&header).await.is_ok() && send.write_all(&(bytes.len() as u16).to_be_bytes()).await.is_ok(){let _=send.write_all(&bytes).await;let _=send.finish();}
}

#[tauri::command]
pub async fn tunnel_set_preparation(value:Preparation) -> AppResult<()> {
    if !value.valid(){return Err(failure("Estado de preparação inválido"));}
    let state=SESSION.lock().await;
    let session=state.as_ref().ok_or_else(|| failure("Entre na sala antes de preparar a instância"))?;
    let present=if let Some(room)=&session.room {room.lock().await.worlds.contains_key(&value.owner_id)} else if let Some(room)=&session.client_room {room.lock().await.worlds.iter().any(|world|world.owner_id==value.owner_id)} else {false};
    if !present{return Err(failure("O mundo da preparação já não está nesta sala"));}
    if let Some(room)=&session.room {if let Some(host)=room.lock().await.host.as_mut(){host.preparation=Some(value.clone());}}
    *LOCAL.lock().await=Some(value);Ok(())
}

#[tauri::command]
pub async fn tunnel_room_recipe(owner_id:String) -> AppResult<RoomRecipe> {
    if owner_id.is_empty() || owner_id.len()>128{return Err(failure("Participante inválido"));}
    let (connection,secret,owner)={
        let state=SESSION.lock().await;let session=state.as_ref().ok_or_else(|| failure("Entre na sala antes de preparar a instância"))?;
        if let Some(room)=&session.room {
            let room=room.lock().await;
            if !room.worlds.contains_key(&owner_id){return Err(failure("O mundo LAN não está aberto"));}
            if room.host.as_ref().is_some_and(|host| host.id==owner_id){drop(room);drop(state);return crate::commands::studio::active_recipe().await;}
            let connection=room.connections.get(&owner_id).cloned().ok_or_else(|| failure("Participante desconectado"))?;
            let secret=*room.member_secrets.get(&owner_id).ok_or_else(|| failure("Participante desconectado"))?;
            (connection,secret,String::new())
        } else {(session.connection.clone().ok_or_else(|| failure("Sala desconectada"))?,session.secret,owner_id)}
    };
    let value=tokio::time::timeout(Duration::from_secs(35),remote_value(&connection,secret,&owner)).await.map_err(|_| failure("O dono do mundo não respondeu à receita"))??;
    if value["ok"]!=true{return Err(failure(value["error"].as_str().unwrap_or("Receita indisponível")));}
    Ok(serde_json::from_value(value["recipe"].clone())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preparation_has_bounded_authenticated_metadata() {
        let mut value=Preparation {phase:"ready".into(),percent:100,owner_id:"owner".into(),fingerprint:"a".repeat(64)};
        assert!(value.valid());value.percent=101;assert!(!value.valid());value.percent=100;value.phase="script".into();assert!(!value.valid());
    }
    #[test]
    fn ready_status_is_removed_when_a_world_closes() {
        let mut state=Some(Preparation{phase:"ready".into(),percent:100,owner_id:"owner".into(),fingerprint:"a".repeat(64)});
        normalize(&mut state,&[]);assert!(state.is_none());
    }
    #[tokio::test]
    async fn lost_connection_finishes_client_and_allows_rejoining() {
        let host=start_host_as(0,endpoint().await.unwrap(),RoomIdentity::default()).await.unwrap();
        let invitation=host.status.invitation.clone().unwrap();
        let client=start_client_as(parse_invitation(&invitation).unwrap(),endpoint().await.unwrap(),RoomIdentity::default()).await.unwrap();
        let id=client.endpoint.id().to_string();
        {
            let room=host.room.as_ref().unwrap().lock().await;
            room.connections.get(&id).unwrap().close(0u8.into(),b"network interruption fixture");
        }
        tokio::time::timeout(Duration::from_secs(5),async{while !client.task.is_finished(){tokio::time::sleep(Duration::from_millis(25)).await;}}).await.unwrap();
        client.stop().await;
        let rejoined=start_client_as(parse_invitation(&invitation).unwrap(),endpoint().await.unwrap(),RoomIdentity::default()).await.unwrap();
        let snapshot=read_room_snapshot(rejoined.connection.as_ref().unwrap(),rejoined.secret).await.unwrap();
        assert_eq!(snapshot.members.len(),2);
        rejoined.stop().await;host.stop().await;
    }
}
