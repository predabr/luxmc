use base64::Engine;
use serde::Serialize;
use tauri::{Emitter, Manager};

use crate::error::{AppError, AppResult};

const PICKER_LABEL: &str = "namemc-picker";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NameMcSkin {
    skin_id: String,
    skin_url: String,
}

fn is_namemc_host(host: &str) -> bool {
    host == "namemc.com" || host.ends_with(".namemc.com")
}

fn skin_id_from_url(source: &str) -> AppResult<String> {
    let url = url::Url::parse(source.trim())
        .map_err(|_| AppError::InvalidInput("Cole o link de uma skin do NameMC".into()))?;
    if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() || url.port().is_some() {
        return Err(AppError::InvalidInput("Use um link HTTPS do NameMC".into()));
    }
    let host = url.host_str().unwrap_or_default();
    let path = url.path().trim_end_matches('/');
    let id = if host == "s.namemc.com" {
        path.strip_prefix("/i/").and_then(|id| id.strip_suffix(".png"))
    } else if is_namemc_host(host) {
        path.strip_prefix("/skin/")
    } else {
        None
    };
    match id {
        Some(id) if id.len() == 16 && id.bytes().all(|byte| byte.is_ascii_hexdigit()) => Ok(id.to_ascii_lowercase()),
        _ => Err(AppError::InvalidInput("Abra uma skin no NameMC e copie o link da página".into())),
    }
}

fn capture_skin_selection(app: &tauri::AppHandle, target: &url::Url) -> bool {
    if let Ok(id) = skin_id_from_url(target.as_str()) {
        if let Some(main) = app.get_webview_window("main") {
            let _ = main.emit("namemc-skin-selected", format!("https://namemc.com/skin/{id}"));
            let _ = main.set_focus();
        }
        return false;
    }
    target.scheme() == "https" && target.host_str().is_some_and(is_namemc_host)
}

#[tauri::command]
pub async fn namemc_open_picker(window: tauri::WebviewWindow) -> AppResult<()> {
    require_main_window(&window)?;
    let app = window.app_handle().clone();
    if let Some(window) = app.get_webview_window(PICKER_LABEL) {
        window.show().map_err(|error| AppError::Internal(error.to_string()))?;
        window.set_focus().map_err(|error| AppError::Internal(error.to_string()))?;
        return Ok(());
    }
    let target = url::Url::parse("https://namemc.com/minecraft-skins")
        .map_err(|error| AppError::Internal(error.to_string()))?;
    let navigation_app = app.clone();
    let popup_app = app.clone();
    let builder = tauri::WebviewWindowBuilder::new(&app, PICKER_LABEL, tauri::WebviewUrl::External(target))
        .title("NameMC · Lux MC")
        .inner_size(1100.0, 780.0)
        .min_inner_size(760.0, 540.0)
        .center()
        .on_navigation(move |target| capture_skin_selection(&navigation_app, target))
        .on_new_window(move |target, _| {
            capture_skin_selection(&popup_app, &target);
            tauri::webview::NewWindowResponse::Deny
        });
    let builder = if let Some(main) = app.get_webview_window("main") {
        builder.parent(&main).map_err(|error| AppError::Internal(error.to_string()))?
    } else {
        builder
    };
    builder.build().map_err(|error| AppError::Internal(error.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn namemc_close_picker(window: tauri::WebviewWindow) -> AppResult<()> {
    require_main_window(&window)?;
    let app = window.app_handle();
    if let Some(window) = app.get_webview_window(PICKER_LABEL) {
        window.close().map_err(|error| AppError::Internal(error.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn namemc_import_skin(window: tauri::WebviewWindow, url: String) -> AppResult<NameMcSkin> {
    require_main_window(&window)?;
    namemc_import_skin_core(url).await
}

fn require_main_window(window: &tauri::WebviewWindow) -> AppResult<()> {
    if window.label() != "main" {
        return Err(AppError::InvalidInput("Esta ação só pode ser iniciada pelo launcher".into()));
    }
    Ok(())
}

pub async fn namemc_import_skin_core(url: String) -> AppResult<NameMcSkin> {
    let skin_id = skin_id_from_url(&url)?;
    let http = reqwest::Client::builder()
        .user_agent(concat!("Luxmc/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(12))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let mut response = http.get(format!("https://s.namemc.com/i/{skin_id}.png"))
        .send().await?.error_for_status()?;
    if response.content_length().unwrap_or_default() > 3 * 1024 * 1024 {
        return Err(AppError::InvalidInput("A skin excede 3 MB".into()));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() + chunk.len() > 3 * 1024 * 1024 {
            return Err(AppError::InvalidInput("A skin excede 3 MB".into()));
        }
        bytes.extend_from_slice(&chunk);
    }
    validate_skin_png(&bytes)?;
    Ok(NameMcSkin {
        skin_id,
        skin_url: format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)),
    })
}

fn validate_skin_png(bytes: &[u8]) -> AppResult<()> {
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err(AppError::InvalidInput("O NameMC não retornou uma skin PNG válida".into()));
    }
    let decoder = image::codecs::png::PngDecoder::new(std::io::Cursor::new(bytes))
        .map_err(|_| AppError::InvalidInput("A skin PNG está incompleta".into()))?;
    use image::ImageDecoder;
    let (width, height) = decoder.dimensions();
    if width < 64 || width > 2048 || width % 64 != 0 || (height != width && height != width / 2) {
        return Err(AppError::InvalidInput("A imagem não usa o formato de uma skin Minecraft".into()));
    }
    image::DynamicImage::from_decoder(decoder)
        .map_err(|_| AppError::InvalidInput("A skin PNG está incompleta".into()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_namemc_skin_links() {
        for source in ["https://namemc.com/skin/63455d7069b397c2", "https://pt.namemc.com/skin/63455D7069B397C2/", "https://s.namemc.com/i/63455d7069b397c2.png"] {
            assert_eq!(skin_id_from_url(source).unwrap(), "63455d7069b397c2");
        }
        for source in ["https://namemc.com.evil.test/skin/63455d7069b397c2", "https://evilnamemc.com/skin/63455d7069b397c2", "http://namemc.com/skin/63455d7069b397c2", "https://user@namemc.com/skin/63455d7069b397c2", "https://namemc.com:8443/skin/63455d7069b397c2", "https://namemc.com/profile/Steve", "https://s.namemc.com/i/../../skin.png", "https://namemc.com/skin/../../foo"] {
            assert!(skin_id_from_url(source).is_err(), "{source}");
        }
    }

    #[test]
    fn rejects_non_skin_pngs_and_accepts_real_texture() {
        assert!(validate_skin_png(include_bytes!("../../../static/steve.png")).is_ok());
        assert!(validate_skin_png(b"<!doctype html>").is_err());
        let mut tiny = Vec::new();
        image::RgbaImage::new(16, 16).write_to(&mut std::io::Cursor::new(&mut tiny), image::ImageFormat::Png).unwrap();
        assert!(validate_skin_png(&tiny).is_err());
    }

    #[tokio::test]
    #[ignore = "requires the public NameMC skin endpoint"]
    async fn imports_public_skin_with_identified_http_client() {
        let result = namemc_import_skin_core("https://namemc.com/skin/63455d7069b397c2".into()).await.unwrap();
        assert_eq!(result.skin_id, "63455d7069b397c2");
        let png = result.skin_url.strip_prefix("data:image/png;base64,").unwrap();
        let bytes = base64::engine::general_purpose::STANDARD.decode(png).unwrap();
        assert!(validate_skin_png(&bytes).is_ok());
    }
}
