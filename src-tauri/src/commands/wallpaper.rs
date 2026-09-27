use crate::error::{AppError, AppResult};
use sha2::Digest;
use std::{sync::LazyLock, time::Duration};

static PREPARING: LazyLock<tokio::sync::Semaphore> = LazyLock::new(|| tokio::sync::Semaphore::new(1));

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
    let result = tokio::time::timeout(Duration::from_secs(60), tokio::process::Command::new("ffmpeg")
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
            let result = tokio::time::timeout(Duration::from_secs(20), tokio::process::Command::new("ffmpeg")
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
    async fn prepares_a_static_poster_from_video() {
        if tokio::process::Command::new("ffmpeg").arg("-version").output().await.is_err() { return; }
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/fixtures/wallpaper.mp4");
        let poster = super::wallpaper_prepare_poster(path.to_owned()).await.unwrap();
        assert!(poster.starts_with("data:image/png;base64,"));
    }
}
