use crate::error::{AppError, AppResult};
use sha2::Digest;
use std::{sync::LazyLock, time::Duration};

static PREPARING: LazyLock<tokio::sync::Semaphore> = LazyLock::new(|| tokio::sync::Semaphore::new(1));

#[tauri::command]
pub async fn wallpaper_import(path: String) -> AppResult<String> {
    let project = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("Pasta de wallpapers indisponível".into()))?;
    import_into(std::path::Path::new(&path), &project.data_local_dir().join("wallpapers")).await
}

async fn import_into(source: &std::path::Path, directory: &std::path::Path) -> AppResult<String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let source = tokio::fs::canonicalize(source).await?;
    let metadata = tokio::fs::metadata(&source).await?;
    let extension = source.extension().and_then(|value| value.to_str()).unwrap_or_default().to_ascii_lowercase();
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > 512 * 1024 * 1024 ||
        !matches!(extension.as_str(), "mp4" | "webm" | "gif" | "png" | "jpg" | "jpeg" | "webp") {
        return Err(AppError::InvalidInput("Selecione uma imagem ou vídeo de até 512 MB".into()));
    }
    tokio::fs::create_dir_all(directory).await?;
    let managed = tokio::fs::canonicalize(directory).await?;
    if let Ok(relative) = source.strip_prefix(&managed) { return Ok(directory.join(relative).to_string_lossy().into_owned()); }
    let temporary = managed.join(format!("{}.import", uuid::Uuid::new_v4()));
    let result: AppResult<String> = async {
        let mut input = tokio::fs::File::open(&source).await?;
        let mut output = tokio::fs::OpenOptions::new().write(true).create_new(true).open(&temporary).await?;
        let mut digest = sha2::Sha256::new();
        let mut buffer = vec![0u8; 65536];
        let mut copied = 0u64;
        loop {
            let count = input.read(&mut buffer).await?;
            if count == 0 { break; }
            copied += count as u64;
            if copied > 512 * 1024 * 1024 { return Err(AppError::InvalidInput("Wallpaper excede 512 MB".into())); }
            digest.update(&buffer[..count]);
            output.write_all(&buffer[..count]).await?;
        }
        if copied == 0 { return Err(AppError::InvalidInput("Wallpaper vazio".into())); }
        output.sync_all().await?;
        drop(output);
        let destination = directory.join(format!("{:x}.{extension}", digest.finalize()));
        if tokio::fs::metadata(&destination).await.is_ok_and(|metadata| metadata.len() == copied) {
            tokio::fs::remove_file(&temporary).await?;
        } else {
            tokio::fs::rename(&temporary, &destination).await?;
        }
        Ok(destination.to_string_lossy().into_owned())
    }.await;
    if result.is_err() { let _ = tokio::fs::remove_file(&temporary).await; }
    result
}

#[tauri::command]
pub async fn wallpaper_prepare_video(path: String, width: u32, fps: u32) -> AppResult<String> {
    if ![960, 1280, 1920].contains(&width) || ![15, 24, 30].contains(&fps) {
        return Err(AppError::InvalidInput("Qualidade de wallpaper inválida".into()));
    }
    let source = tokio::fs::canonicalize(path).await?;
    let meta = tokio::fs::metadata(&source).await?;
    if !meta.is_file() || meta.len() > 512 * 1024 * 1024 {
        return Err(AppError::InvalidInput("Vídeo excede o limite de otimização".into()));
    }
    let key = format!("{:?}:{}:{:?}:{width}:{fps}:v1", source, meta.len(), meta.modified()?);
    let key = format!("{:x}", sha2::Sha256::digest(key.as_bytes()));
    let project = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("Cache indisponível".into()))?;
    let directory = project.cache_dir().join("wallpapers");
    tokio::fs::create_dir_all(&directory).await?;
    let target = directory.join(format!("{key}.mp4"));
    let ext = source.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    if (ext == "mp4" || ext == "webm") && meta.len() <= 100 * 1024 * 1024 {
        return Ok(source.to_string_lossy().into_owned());
    }
    let _permit = PREPARING.acquire().await.map_err(|e| AppError::InvalidState(e.to_string()))?;
    if tokio::fs::metadata(&target).await.is_ok_and(|m| m.len() > 0) { return Ok(target.to_string_lossy().into_owned()); }
    let temporary = directory.join(format!("{key}-{}.mp4", uuid::Uuid::new_v4()));
    let result = tokio::time::timeout(Duration::from_secs(60), crate::core::process::tokio_command("ffmpeg")
        .args(["-nostdin", "-v", "error", "-y", "-threads", "1", "-protocol_whitelist", "file,pipe", "-i"])
        .arg(&source).args(["-map", "0:v:0", "-an", "-sn", "-dn", "-vf"])
        .arg(format!("scale='min(iw,{width})':-2,fps={fps}"))
        .args(["-c:v", "libx264", "-threads", "1", "-preset", "ultrafast", "-crf", "28", "-pix_fmt", "yuv420p", "-movflags", "+faststart"])
        .arg(&temporary).kill_on_drop(true).output()).await;
    if !matches!(result, Ok(Ok(ref output)) if output.status.success()) {
        let _ = tokio::fs::remove_file(&temporary).await;
        return Ok(source.to_string_lossy().into_owned());
    }
    tokio::fs::rename(&temporary, &target).await?;
    Ok(target.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn wallpaper_prepare_poster(path: String) -> AppResult<String> {
    use base64::Engine;

    let source = tokio::fs::canonicalize(path).await?;
    let metadata = tokio::fs::metadata(&source).await?;
    let extension = source.extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase();
    if !metadata.is_file() || metadata.len() > 512 * 1024 * 1024 || !matches!(extension.as_str(), "mp4" | "webm") {
        return Err(AppError::InvalidInput("Selecione um vídeo MP4 ou WebM válido".into()));
    }
    let project = directories::ProjectDirs::from("io", "github", "Luxmc")
        .ok_or_else(|| AppError::InvalidState("Cache indisponível".into()))?;
    let directory = project.cache_dir().join("wallpapers");
    tokio::fs::create_dir_all(&directory).await?;
    let key = format!("{:x}", sha2::Sha256::digest(format!("{:?}:{}:{:?}:poster-v1", source, metadata.len(), metadata.modified()?).as_bytes()));
    let target = directory.join(format!("{key}.png"));
    if !tokio::fs::metadata(&target).await.is_ok_and(|meta| meta.len() > 0) {
        let _permit = PREPARING.acquire().await.map_err(|error| AppError::InvalidState(error.to_string()))?;
        if !tokio::fs::metadata(&target).await.is_ok_and(|meta| meta.len() > 0) {
            let temporary = directory.join(format!("{key}-{}.png", uuid::Uuid::new_v4()));
            let result = tokio::time::timeout(Duration::from_secs(20), crate::core::process::tokio_command("ffmpeg")
                .args(["-nostdin", "-v", "error", "-y", "-ss", "0.05", "-i"])
                .arg(&source)
                .args(["-map", "0:v:0", "-frames:v", "1", "-vf", "scale='min(iw,640)':-2", "-c:v", "png"])
                .arg(&temporary)
                .kill_on_drop(true)
                .output()).await;
            if !matches!(result, Ok(Ok(ref output)) if output.status.success()) {
                let _ = tokio::fs::remove_file(&temporary).await;
                return Err(AppError::InvalidState("Não foi possível gerar a prévia do wallpaper".into()));
            }
            tokio::fs::rename(&temporary, &target).await?;
        }
    }
    let png = tokio::fs::read(&target).await?;
    if png.len() > 3 * 1024 * 1024 || !png.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err(AppError::InvalidState("Prévia do wallpaper inválida".into()));
    }
    Ok(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png)))
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn imported_wallpaper_survives_deleted_source_and_deduplicates() {
        let directory = std::env::temp_dir().join(format!("luxmc-wallpaper-test-{}", uuid::Uuid::new_v4()));
        let source = directory.join("source.png");
        let managed = directory.join("managed");
        tokio::fs::create_dir_all(&directory).await.unwrap();
        let original = include_bytes!("../../../static/alex.png");
        tokio::fs::write(&source, original).await.unwrap();
        let imported = super::import_into(&source, &managed).await.unwrap();
        assert_eq!(imported, super::import_into(&source, &managed).await.unwrap());
        tokio::fs::remove_file(&source).await.unwrap();
        assert_eq!(tokio::fs::read(&imported).await.unwrap(), original);
        assert_eq!(imported, super::import_into(std::path::Path::new(&imported), &managed).await.unwrap());
        tokio::fs::remove_file(&imported).await.unwrap();
        tokio::fs::remove_dir(&managed).await.unwrap();
        tokio::fs::remove_dir(&directory).await.unwrap();
    }
    #[tokio::test]
    async fn prepares_a_static_poster_from_video() {
        if crate::core::process::tokio_command("ffmpeg").arg("-version").output().await.is_err() { return; }
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/fixtures/wallpaper.mp4");
        let poster = super::wallpaper_prepare_poster(path.to_owned()).await.unwrap();
        assert!(poster.starts_with("data:image/png;base64,"));
    }
}
