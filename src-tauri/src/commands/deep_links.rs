use parking_lot::Mutex;
use tauri::{Emitter, Manager};

#[derive(Default)]
pub struct PendingLinks(Mutex<Vec<String>>);

pub fn enqueue(app: &tauri::AppHandle, urls: Vec<String>) {
    let state = app.state::<PendingLinks>();
    let mut pending = state.0.lock();
    for value in urls {
        if value.len() <= 8192 && value.starts_with("luxmc://") && pending.len() < 32 && !pending.contains(&value) {
            pending.push(value);
        }
    }
    drop(pending);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
    let _ = app.emit("deep-link-pending", ());
}

#[tauri::command]
pub fn deep_links_take(state: tauri::State<'_, PendingLinks>) -> Vec<String> {
    std::mem::take(&mut *state.0.lock())
}

#[tauri::command]
pub fn open_portal(section: String) -> Result<(), String> {
    let path = match section.as_str() {
        "home" => "",
        "catalog" => "#mods",
        "news" => "#changelog",
        "skins" => "skins.html",
        _ => return Err("Página do portal inválida".into()),
    };
    open::that(format!("https://luxmc-r92.pages.dev/{path}")).map_err(|error| error.to_string())
}
