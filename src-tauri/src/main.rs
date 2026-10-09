// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(all(not(debug_assertions), dev))]
compile_error!("Release requires embedded assets: use pnpm tauri build");

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--virtual-lan-broker") {
        if let Some(id) = std::env::args().nth(2) { let _ = luxmc_lib::network::virtual_lan::run_broker(&id); }
        return;
    }
    luxmc_lib::core::panic_log::install();

    #[cfg(target_os = "linux")]
    {
        extern "C" {
            fn g_set_prgname(prgname: *const std::ffi::c_char);
            fn g_set_application_name(application_name: *const std::ffi::c_char);
        }
        unsafe {
            g_set_prgname(b"luxmc\0".as_ptr() as *const _);
            g_set_application_name(b"Luxmc\0".as_ptr() as *const _);
        }

        if std::env::var("G_PRGNAME").is_err() {
            std::env::set_var("G_PRGNAME", "luxmc");
        }
        std::env::set_var("GDK_BACKEND", "wayland,x11");

        if let Ok(appdir) = std::env::var("APPDIR") {
            let usr_dir = std::path::Path::new(&appdir).join("usr");
            if usr_dir.is_dir() {
                let _ = std::env::set_current_dir(&usr_dir);
                let injected = usr_dir.join("lib/x86_64-linux-gnu/webkit2gtk-4.1/injected-bundle");
                if injected.is_dir() {
                    std::env::set_var("WEBKIT_INJECTED_BUNDLE_PATH", &injected);
                }
                let exec_path = usr_dir.join("lib/x86_64-linux-gnu/webkit2gtk-4.1");
                if exec_path.is_dir() {
                    std::env::set_var("WEBKIT_EXEC_PATH", &exec_path);
                }
            }

            let root = std::path::Path::new(&appdir);
            let gst_dirs: Vec<_> = [
                "usr/lib/gstreamer-1.0",
                "usr/lib64/gstreamer-1.0",
                "usr/lib/x86_64-linux-gnu/gstreamer-1.0",
            ]
            .iter()
            .map(|path| root.join(path))
            .filter(|path| path.is_dir())
            .collect();
            if let Ok(gst_path) = std::env::join_paths(&gst_dirs) {
                for key in [
                    "GST_PLUGIN_SYSTEM_PATH_1_0",
                    "GST_PLUGIN_PATH_1_0",
                    "GST_PLUGIN_SYSTEM_PATH",
                    "GST_PLUGIN_PATH",
                ] {
                    std::env::set_var(key, &gst_path);
                }
            }
            std::env::set_var("GST_REGISTRY_REUSE_PLUGIN_SCANNER", "no");
            let scanner = [
                "usr/lib/gstreamer1.0/gstreamer-1.0/gst-plugin-scanner",
                "usr/lib/x86_64-linux-gnu/gstreamer1.0/gstreamer-1.0/gst-plugin-scanner",
                "usr/libexec/gstreamer-1.0/gst-plugin-scanner",
                "usr/lib/gstreamer-1.0/gst-plugin-scanner",
                "usr/lib64/gstreamer-1.0/gst-plugin-scanner",
            ]
            .iter()
            .map(|path| root.join(path))
            .find(|path| path.is_file())
            .unwrap_or_else(|| root.join("usr/libexec/gstreamer-1.0/gst-plugin-scanner"));
            std::env::set_var("GST_PLUGIN_SCANNER_1_0", &scanner);
            std::env::set_var("GST_PLUGIN_SCANNER", &scanner);
            if gst_dirs.is_empty() || !scanner.is_file() {
                eprintln!(
                    "Luxmc: AppImage incompleto: plugins ou scanner GStreamer ausentes em {}",
                    root.display()
                );
            }
        } else if let Ok(exe) = std::env::current_exe() {
            if let Some(bin) = exe.parent() {
                if let Some(usr) = bin.parent() {
                    let network_proc =
                        usr.join("lib/x86_64-linux-gnu/webkit2gtk-4.1/WebKitNetworkProcess");
                    if network_proc.is_file() {
                        let _ = std::env::set_current_dir(usr);
                        let injected =
                            usr.join("lib/x86_64-linux-gnu/webkit2gtk-4.1/injected-bundle");
                        if injected.is_dir() {
                            std::env::set_var("WEBKIT_INJECTED_BUNDLE_PATH", injected);
                        }
                    }
                }
            }
        }

        if std::env::var("LUXMC_SOFTWARE_RENDER")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false)
        {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
            std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
        }
        if std::env::var("__NV_DISABLE_EXPLICIT_SYNC").is_err() {
            std::env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
        }

        if std::env::var("MALLOC_ARENA_MAX").is_err() {
            std::env::set_var("MALLOC_ARENA_MAX", "2");
        }
        if std::env::var("MALLOC_TRIM_THRESHOLD_").is_err() {
            std::env::set_var("MALLOC_TRIM_THRESHOLD_", "131072");
        }
    }

    let is_daemon = std::env::args().any(|a| a == "--daemon");
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_stack_size(16 * 1024 * 1024)
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    if is_daemon {
        runtime.block_on(luxmc_lib::daemon::run_daemon());
    } else {
        tauri::async_runtime::set(runtime.handle().clone());
        runtime.block_on(luxmc_lib::run());
    }
}
