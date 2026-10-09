#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use luxmc_repair::{cache_for, Recovery, MAIN};
mod cleanup;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("--uninstall-data") {
        let count=cleanup::uninstall_data()?;
        println!("{{\"dataDirectoriesRemoved\":{count}}}");
        return Ok(());
    }
    let executable = std::env::current_exe()?;
    let directory = executable.parent().ok_or("Pasta do verificador indisponível")?.to_path_buf();
    let expected = option_env!("LUXMC_MAIN_SHA256").unwrap_or("");
    let recovery = Recovery::new(directory.clone(), cache_for(&directory)?, expected.to_owned(), env!("CARGO_PKG_VERSION").to_owned())?;
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let mode = args.first().and_then(|value| value.to_str());
    let report = match mode {
        Some("--initialize") => recovery.initialize()?,
        Some("--verify-only") => recovery.verify(false)?,
        _ => recovery.verify(true)?,
    };
    println!("{}", serde_json::to_string(&report)?);
    if matches!(mode, Some("--initialize" | "--verify-only" | "--repair-only")) { return Ok(()); }
    if !report.healthy { return Err("Não há uma cópia válida para iniciar o Luxmc. Reinstale pelo site oficial.".into()); }
    std::process::Command::new(directory.join(MAIN)).args(args).current_dir(directory).spawn()?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        #[cfg(windows)]
        if !std::env::args().skip(1).any(|arg| arg.starts_with("--")) {
            #[link(name = "user32")]
            extern "system" { fn MessageBoxW(window: *mut std::ffi::c_void, text: *const u16, title: *const u16, kind: u32) -> i32; }
            let message: Vec<u16> = format!("Não foi possível recuperar o Luxmc: {error}\n\nReinstale em https://luxmc-r92.pages.dev . Seus dados do jogo não foram alterados.").encode_utf16().chain(std::iter::once(0)).collect();
            let title: Vec<u16> = "Recuperação do Luxmc".encode_utf16().chain(std::iter::once(0)).collect();
            unsafe { MessageBoxW(std::ptr::null_mut(), message.as_ptr(), title.as_ptr(), 0x10); }
        }
        std::process::exit(1);
    }
}
