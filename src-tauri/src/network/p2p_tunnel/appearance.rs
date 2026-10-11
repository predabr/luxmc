use super::*;
use std::io::Cursor;

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Appearance {
    username: String,
    uuid: String,
    skin: String,
    cape: Option<String>,
    model: String,
}

pub(crate) fn path() -> Option<std::path::PathBuf> {
    private_lan_session_path().map(|path| path.with_file_name("room-appearances.properties"))
}

fn texture(bytes: &[u8], cape: bool) -> Option<String> {
    if bytes.len() > 131072 || !bytes.starts_with(b"\x89PNG\r\n\x1a\n") { return None; }
    let reader = image::ImageReader::with_format(Cursor::new(bytes), image::ImageFormat::Png);
    let (width, height) = reader.into_dimensions().ok()?;
    if if cape { width < 2 || width > 512 || height < 2 || height > 512 } else { width != 64 || ![32, 64].contains(&height) } { return None; }
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).ok()?;
    let image = if cape {
        let rgba = image.to_rgba8();
        let normalized = if width == height * 2 { rgba }
            else if (width, height) == (22, 17) || (width, height) == (44, 34) {
                let scale = width / 22;
                let mut canvas = image::RgbaImage::new(64 * scale, 32 * scale);
                image::imageops::overlay(&mut canvas, &rgba, 0, 0);
                canvas
            } else { crate::core::launcher::format_custom_cape_texture(&rgba, width, height).0 };
        image::DynamicImage::ImageRgba8(normalized).resize_exact(64, 32, image::imageops::FilterType::Nearest)
    } else { image::DynamicImage::ImageRgba8(crate::core::launcher::normalize_skin_image(image.to_rgba8())) };
    let mut output = Cursor::new(Vec::new());
    image.write_to(&mut output, image::ImageFormat::Png).ok()?;
    Some(base64::engine::general_purpose::STANDARD.encode(output.into_inner()))
}

impl Appearance {
    fn validate(mut self, member: &TunnelMember) -> Option<Self> {
        if !matches!(self.model.as_str(), "classic" | "default" | "slim") { return None; }
        let decode = |value: &str, cape| {
            if value.len() > 180000 { return None; }
            texture(&base64::engine::general_purpose::STANDARD.decode(value).ok()?, cape)
        };
        self.skin = decode(&self.skin, false)?;
        self.cape = self.cape.as_deref().and_then(|cape| decode(cape, true));
        self.username = member.username.clone();
        self.uuid = member.uuid.clone();
        if !self.username.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') || !(3..=16).contains(&self.username.len()) || uuid::Uuid::parse_str(&self.uuid).is_err() { return None; }
        Some(self)
    }
}

pub(super) async fn local(identity: &RoomIdentity) -> Option<Appearance> {
    if cfg!(test) { return None; }
    let db = crate::db::shared_db().await.ok()?;
    let account = sqlx::query_as::<_, crate::db::models::AccountRow>("SELECT * FROM accounts WHERE replace(uuid, '-', '') = ? COLLATE NOCASE LIMIT 1").bind(identity.uuid.replace('-', "")).fetch_optional(db.pool()).await.ok()??;
    let http = crate::core::mods::pack_download::client().ok()?;
    let skin = tokio::time::timeout(Duration::from_secs(3), crate::commands::skins::load_asset_bytes(&http, account.skin_url.as_deref()?)).await.ok()??;
    let cape = if let Some(source) = account.cape_url {
        tokio::time::timeout(Duration::from_secs(2), crate::commands::skins::load_asset_bytes(&http, &source)).await.ok().flatten().and_then(|bytes| texture(&bytes, true))
    } else { None };
    Appearance { username: identity.username.clone(), uuid: identity.uuid.clone(), skin: texture(&skin, false)?, cape, model: account.skin_variant.unwrap_or_else(|| "classic".into()) }.validate(&TunnelMember::new(String::new(), identity.clone(), false))
}

pub(super) async fn control(id: &str, mut send: iroh::endpoint::SendStream, mut recv: iroh::endpoint::RecvStream, registry: Arc<Mutex<RoomRegistry>>) {
    let mut length = [0; 4];
    if !matches!(tokio::time::timeout(Duration::from_secs(3), recv.read_exact(&mut length)).await, Ok(Ok(_))) { return; }
    let length = u32::from_be_bytes(length) as usize;
    if length > 400000 { return; }
    let mut bytes = vec![0; length];
    if !matches!(tokio::time::timeout(Duration::from_secs(3), recv.read_exact(&mut bytes)).await, Ok(Ok(_))) { return; }
    let Ok(value) = serde_json::from_slice::<Option<Appearance>>(&bytes) else { return; };
    let mut room = registry.lock().await;
    let Some(member) = room.members.get(id).cloned() else { return; };
    if let Some(value) = value {
        let unchanged = room.appearances.get(id).is_some_and(|saved| saved.skin == value.skin && saved.cape == value.cape && saved.model == value.model);
        if !unchanged {
            let Some(value) = value.validate(&member) else { return; };
            room.appearances.insert(id.into(), value);
        }
    } else { room.appearances.remove(id); }
    let Ok(bytes) = serde_json::to_vec(&room.appearances) else { return; };
    let entries = properties(&room.appearances);
    drop(room);
    if let Some(path) = path() { let _ = crate::core::mods::pack_download::atomic_write(&path, format!("expires={}\n{entries}", (now() + 8) * 1000).as_bytes()).await; }
    if bytes.len() > 512000 { return; }
    if send.write_all(&(bytes.len() as u32).to_be_bytes()).await.is_ok() { let _ = send.write_all(&bytes).await; let _ = send.finish(); }
}

fn properties(values: &BTreeMap<String, Appearance>) -> String {
    let mut entries = String::new();
    for appearance in values.values() {
        entries.push_str(&format!("{}.uuid={}\n{}.skin={}\n{}.model={}\n", appearance.username, appearance.uuid, appearance.username, appearance.skin, appearance.username, appearance.model));
        if let Some(cape) = &appearance.cape { entries.push_str(&format!("{}.cape={cape}\n", appearance.username)); }
    }
    entries
}

async fn exchange(connection: &iroh::endpoint::Connection, secret: [u8; 32], own: &Option<Appearance>) -> AppResult<BTreeMap<String, Appearance>> {
    let (mut send, mut recv) = connection.open_bi().await.map_err(failure)?;
    let mut header = [9; 33]; header[1..].copy_from_slice(&secret);
    let bytes = serde_json::to_vec(own)?;
    send.write_all(&header).await.map_err(failure)?;
    send.write_all(&(bytes.len() as u32).to_be_bytes()).await.map_err(failure)?;
    send.write_all(&bytes).await.map_err(failure)?;
    let _ = send.finish();
    let mut length = [0; 4]; recv.read_exact(&mut length).await.map_err(failure)?;
    let length = u32::from_be_bytes(length) as usize;
    if length > 512000 { return Err(failure("Texturas da sala acima do limite")); }
    let mut bytes = vec![0; length]; recv.read_exact(&mut bytes).await.map_err(failure)?;
    let values: BTreeMap<String, Appearance> = serde_json::from_slice(&bytes)?;
    if values.len() > ROOM_CAPACITY { return Err(failure("Texturas da sala inválidas")); }
    Ok(values)
}

pub(super) fn watch(registry: Option<Arc<Mutex<RoomRegistry>>>, connection: Option<iroh::endpoint::Connection>, secret: [u8; 32], own: Option<Appearance>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        let mut previous = Vec::new();
        let mut entries = String::new();
        loop {
            interval.tick().await;
            let values = if let Some(registry) = &registry { registry.lock().await.appearances.clone() }
                else if let Some(connection) = &connection {
                    if connection.close_reason().is_some() { break; }
                    match tokio::time::timeout(Duration::from_secs(3), exchange(connection, secret, &own)).await { Ok(Ok(values)) => values, _ => continue }
                } else { break; };
            let Some(path) = path() else { continue; };
            let Ok(payload) = serde_json::to_vec(&values) else { continue; };
            if payload != previous {
                entries.clear();
                for appearance in values.values() {
                let member = TunnelMember::new(String::new(), RoomIdentity { username: appearance.username.clone(), uuid: appearance.uuid.clone(), avatar_url: None, display_name: None }, false);
                let Some(appearance) = appearance.clone().validate(&member) else { continue; };
                entries.push_str(&format!("{}.uuid={}\n{}.skin={}\n{}.model={}\n", appearance.username, appearance.uuid, appearance.username, appearance.skin, appearance.username, appearance.model));
                if let Some(cape) = appearance.cape { entries.push_str(&format!("{}.cape={cape}\n", appearance.username)); }
                }
                previous = payload;
            }
            let text = format!("expires={}\n{entries}", (now() + 8) * 1000);
            let _ = crate::core::mods::pack_download::atomic_write(&path, text.as_bytes()).await;
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn room_textures_are_bounded_and_bound_to_the_admitted_identity() {
        let identity = RoomIdentity { username: "LuxmcPeer".into(), uuid: uuid::Uuid::new_v4().to_string(), ..Default::default() };
        let member = TunnelMember::new("peer".into(), identity, false);
        let bytes = include_bytes!("../../../../static/steve.png");
        let value = Appearance { username: "Impostor".into(), uuid: "wrong".into(), skin: texture(bytes, false).unwrap(), cape: None, model: "slim".into() };
        let valid = value.clone().validate(&member).unwrap();
        assert_eq!(valid.username, member.username);
        assert_eq!(valid.uuid, member.uuid);
        let mut invalid = value; invalid.skin = "not an image".into();
        assert!(invalid.validate(&member).is_none());
        assert!(texture(b"not a PNG", false).is_none());
    }
    #[tokio::test]
    async fn exchanges_textures_only_over_an_admitted_encrypted_room_connection() {
        let host_identity = RoomIdentity { username: "LuxmcHost".into(), uuid: uuid::Uuid::new_v4().to_string(), ..Default::default() };
        let guest_identity = RoomIdentity { username: "LuxmcPeer".into(), uuid: uuid::Uuid::new_v4().to_string(), ..Default::default() };
        let host = start_host_as(0, endpoint().await.unwrap(), host_identity.clone()).await.unwrap();
        let client = start_client_as(parse_invitation(host.status.invitation.as_ref().unwrap()).unwrap(), endpoint().await.unwrap(), guest_identity.clone()).await.unwrap();
        let skin = texture(include_bytes!("../../../../static/steve.png"), false).unwrap();
        let make = |identity: &RoomIdentity| Appearance { username: identity.username.clone(), uuid: identity.uuid.clone(), skin: skin.clone(), cape: None, model: "slim".into() };
        host.room.as_ref().unwrap().lock().await.appearances.insert(host.endpoint.id().to_string(), make(&host_identity));
        let values = exchange(client.connection.as_ref().unwrap(), client.secret, &Some(make(&guest_identity))).await.unwrap();
        assert_eq!(values.len(), 2);
        assert!(values.values().any(|value| value.username == "LuxmcHost" && value.skin == skin));
        assert!(values.values().any(|value| value.username == "LuxmcPeer" && value.uuid == guest_identity.uuid));
        client.stop().await;
        host.stop().await;
    }
}
