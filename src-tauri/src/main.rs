// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;

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
                            cmd.env("G_PRGNAME", "luxmc");
                            cmd.env("LUXMC_WAYLAND_PRELOADED", "1");
                            cmd.env("MALLOC_ARENA_MAX", "2");
                            cmd.env("MALLOC_TRIM_THRESHOLD_", "131072");
                            let _ = cmd.exec();
                        }
                    }
                }
            }
        }

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

            let gst_candidates = [
                std::path::Path::new(&appdir).join("usr/lib/gstreamer-1.0"),
                std::path::Path::new(&appdir).join("usr/lib/x86_64-linux-gnu/gstreamer-1.0"),
            ];
            for gst_dir in gst_candidates {
                if gst_dir.is_dir() {
                    std::env::set_var("GST_PLUGIN_SYSTEM_PATH_1_0", &gst_dir);
                    std::env::set_var("GST_PLUGIN_PATH_1_0", &gst_dir);
                    std::env::set_var("GST_PLUGIN_SYSTEM_PATH", &gst_dir);
                    std::env::set_var("GST_PLUGIN_PATH", &gst_dir);
                    std::env::set_var("GST_REGISTRY_REUSE_PLUGIN_SCANNER", "no");
                    break;
                }
            }

            let scanner_candidates = [
                std::path::Path::new(&appdir).join("usr/lib/gstreamer1.0/gstreamer-1.0/gst-plugin-scanner"),
                std::path::Path::new(&appdir).join("usr/lib/x86_64-linux-gnu/gstreamer1.0/gstreamer-1.0/gst-plugin-scanner"),
            ];
            for scanner in scanner_candidates {
                if scanner.is_file() {
                    std::env::set_var("GST_PLUGIN_SCANNER_1_0", &scanner);
                    std::env::set_var("GST_PLUGIN_SCANNER", &scanner);
                    break;
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

        if std::env::var("LUXMC_SOFTWARE_RENDER").map(|v| v == "1" || v == "true").unwrap_or(false) {
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
