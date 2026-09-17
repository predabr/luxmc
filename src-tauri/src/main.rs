// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;

        // Priority 1: Prevent AppImage WebKitGTK / Mesa runaway memory leak (+1 GB/s).
        // linuxdeploy bundles 2022 Ubuntu 22.04 WebKitGTK and libraries into AppImages.
        // On modern Linux distros (Arch, Fedora, Ubuntu 24+, etc.), this ancient WebKitGTK
        // enters a runaway memory allocation loop with modern Mesa GPU drivers.
        // If the host system has native WebKitGTK (which all modern Linux desktops have),
        // we strip the bundled AppImage library paths from LD_LIBRARY_PATH and re-exec,
        // using the host's native WebKitGTK, Mesa, and GLib.
        // This keeps memory rock-solid at ~140 MB with 0 leaks.
        if std::env::var("LUXMC_CLEAN_HOST_TRIED").is_err() && std::env::var("APPDIR").is_ok() {
            let host_webkit_candidates = [
                "/usr/lib/libwebkit2gtk-4.1.so.0",
                "/usr/lib64/libwebkit2gtk-4.1.so.0",
                "/usr/lib/x86_64-linux-gnu/libwebkit2gtk-4.1.so.0",
            ];

            if host_webkit_candidates.iter().any(|p| std::path::Path::new(p).exists()) {
                if let Ok(exe) = std::env::current_exe() {
                    let mut cmd = std::process::Command::new(exe);
                    cmd.args(std::env::args().skip(1));

                    let appdir = std::env::var("APPDIR").unwrap_or_default();
                    let current_ld = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
                    let filtered_ld = current_ld
                        .split(':')
                        .filter(|p| !p.trim().is_empty() && (appdir.is_empty() || !p.contains(&appdir)))
                        .collect::<Vec<_>>()
                        .join(":");

                    if filtered_ld.trim().is_empty() {
                        cmd.env_remove("LD_LIBRARY_PATH");
                    } else {
                        cmd.env("LD_LIBRARY_PATH", filtered_ld);
                    }

                    cmd.env_remove("WEBKIT_INJECTED_BUNDLE_PATH");
                    cmd.env_remove("GST_PLUGIN_SYSTEM_PATH_1_0");
                    cmd.env_remove("GST_PLUGIN_PATH_1_0");
                    cmd.env_remove("GST_PLUGIN_SCANNER_1_0");

                    let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok()
                        || std::env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland");
                    if is_wayland {
                        cmd.env_remove("GDK_BACKEND");
                    }

                    cmd.env("LUXMC_CLEAN_HOST_TRIED", "1");
                    cmd.env("LUXMC_WAYLAND_PRELOADED", "1");
                    cmd.env("MALLOC_ARENA_MAX", "2");
                    cmd.env("MALLOC_TRIM_THRESHOLD_", "131072");

                    let _ = cmd.exec();
                }
            }
        }

        // Fix for Linux Wayland EGL_BAD_PARAMETER crash in AppImages (fallback path when host WebKit is absent):
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
        if std::env::var("LUXMC_CLEAN_HOST_TRIED").is_err() {
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
        }

        if std::env::var("LUXMC_SOFTWARE_RENDER").as_deref() == Ok("1")
            || std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").as_deref() == Ok("1")
        {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
        if std::env::var("WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS").is_err() {
            std::env::set_var("WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS", "1");
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

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");
    runtime.block_on(luxmc_lib::run());
}
