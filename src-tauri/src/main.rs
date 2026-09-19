// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;

        // Fix for Linux Wayland EGL_BAD_PARAMETER crash in AppImages:
        // Bundled libwayland-client (from Ubuntu 22.04 build environment) causes EGL initialization
        // to abort with "Could not create default EGL display: EGL_BAD_PARAMETER" on modern Linux.
        // Preloading the host system's native libwayland-client resolves the display conflict.
        if std::env::var("LUXMC_WAYLAND_PRELOADED").is_err() {
            let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok()
                || std::env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland");

            if is_wayland {
                let candidates = [
                    "/usr/lib/libwayland-client.so.0",
                    "/usr/lib/libwayland-client.so",
                    "/usr/lib64/libwayland-client.so.0",
                    "/usr/lib64/libwayland-client.so",
                    "/usr/lib/x86_64-linux-gnu/libwayland-client.so.0",
                    "/usr/lib/x86_64-linux-gnu/libwayland-client.so",
                ];

                if let Some(host_wayland) = candidates.iter().find(|p| std::path::Path::new(p).exists()) {
                    let current_preload = std::env::var("LD_PRELOAD").unwrap_or_default();
                    if !current_preload.contains("libwayland-client") {
                        if let Ok(exe) = std::env::current_exe() {
                            let mut cmd = std::process::Command::new(exe);
                            cmd.args(std::env::args().skip(1));

                            let new_preload = if current_preload.trim().is_empty() {
                                host_wayland.to_string()
                            } else {
                                format!("{}:{}", host_wayland, current_preload)
                            };
                            cmd.env("LD_PRELOAD", new_preload);
                            cmd.env("LUXMC_WAYLAND_PRELOADED", "1");
                            cmd.env("MALLOC_ARENA_MAX", "2");
                            cmd.env("MALLOC_TRIM_THRESHOLD_", "131072");
                            let _ = cmd.exec();
                        }
                    }
                }
            }
        }

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

        // WebKitGTK DMA-BUF renderer causes runaway memory allocation loops (+1 GB/s)
        // on Mesa drivers with bundled WebKitGTK, and causes SIGSEGV crashes in libgbm.so/dri_gbm.so
        // on modern Linux (Mesa 24+ / Wayland / AMD / Intel / NVIDIA).
        // Disabling DMA-BUF renderer keeps memory stable at ~150-250 MB and prevents blank/grey screens.
        if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
        if std::env::var("__NV_DISABLE_EXPLICIT_SYNC").is_err() {
            std::env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
        }

        if let Ok(appdir) = std::env::var("APPDIR") {
            let gst_dir = std::path::Path::new(&appdir).join("usr/lib/x86_64-linux-gnu/gstreamer-1.0");
            let gst_dir_alt = std::path::Path::new(&appdir).join("usr/lib/gstreamer-1.0");
            if gst_dir.is_dir() {
                std::env::set_var("GST_PLUGIN_SYSTEM_PATH_1_0", &gst_dir);
                std::env::set_var("GST_PLUGIN_PATH_1_0", &gst_dir);
            } else if gst_dir_alt.is_dir() {
                std::env::set_var("GST_PLUGIN_SYSTEM_PATH_1_0", &gst_dir_alt);
                std::env::set_var("GST_PLUGIN_PATH_1_0", &gst_dir_alt);
            } else {
                std::env::remove_var("GST_PLUGIN_SYSTEM_PATH_1_0");
                std::env::remove_var("GST_PLUGIN_PATH_1_0");
                std::env::remove_var("GST_PLUGIN_SCANNER_1_0");
            }
        }

        // Memory management: prevent glibc multi-arena memory fragmentation
        if std::env::var("MALLOC_ARENA_MAX").is_err() {
            std::env::set_var("MALLOC_ARENA_MAX", "2");
        }
        if std::env::var("MALLOC_TRIM_THRESHOLD_").is_err() {
            std::env::set_var("MALLOC_TRIM_THRESHOLD_", "131072");
        }
    }

    let is_daemon = std::env::args().any(|a| a == "--daemon");
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    if is_daemon {
        runtime.block_on(luxmc_lib::daemon::run_daemon());
    } else {
        runtime.block_on(luxmc_lib::run());
    }
}
