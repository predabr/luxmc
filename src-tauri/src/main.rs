// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    {
        // Fix for linuxdeploy relative WebKit subprocess path bug:
        // linuxdeploy patches libwebkit2gtk with relative paths (././/lib/x86_64-linux-gnu/webkit2gtk-4.1).
        // If the process working directory is not $APPDIR/usr, WebKitNetworkProcess and WebKitWebProcess fail to spawn:
        // "Unable to spawn a new child process: Falha ao criar processo filho ... (Arquivo ou diretório inexistente)".
        // Setting current_dir to usr guarantees WebKit finds its subprocesses inside the AppImage.
        if let Ok(appdir) = std::env::var("APPDIR") {
            let usr_dir = std::path::Path::new(&appdir).join("usr");
            if usr_dir.is_dir() {
                let _ = std::env::set_current_dir(&usr_dir);
                let injected = usr_dir.join("lib/x86_64-linux-gnu/webkit2gtk-4.1/injected-bundle");
                if injected.is_dir() {
                    std::env::set_var("WEBKIT_INJECTED_BUNDLE_PATH", injected);
                }
            }
        } else if let Ok(exe) = std::env::current_exe() {
            if let Some(bin) = exe.parent() {
                if let Some(usr) = bin.parent() {
                    let network_proc = usr.join("lib/x86_64-linux-gnu/webkit2gtk-4.1/WebKitNetworkProcess");
                    if network_proc.is_file() {
                        let _ = std::env::set_current_dir(usr);
                        let injected = usr.join("lib/x86_64-linux-gnu/webkit2gtk-4.1/injected-bundle");
                        if injected.is_dir() {
                            std::env::set_var("WEBKIT_INJECTED_BUNDLE_PATH", injected);
                        }
                    }
                }
            }
        }

        // WebKitGTK Linux stability settings
        if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
        if std::env::var("WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS").is_err() {
            std::env::set_var("WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS", "1");
        }
        if std::env::var("__NV_DISABLE_EXPLICIT_SYNC").is_err() {
            std::env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
        }
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");
    runtime.block_on(luxmc_lib::run());
}
