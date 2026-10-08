use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeCpuProfile {
    pub cores: u32,
    pub avx2: bool,
    pub avx512: bool,
    pub sse42: bool,
    pub aes: bool,
    pub engine: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeMemoryStats {
    pub rss_bytes: u64,
    pub peak_bytes: u64,
    pub trimmed: bool,
}

pub fn trim_memory_native() -> bool {
    #[cfg(windows)]
    {
        #[link(name = "kernel32")]
        extern "system" {
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
            fn SetProcessWorkingSetSize(process: *mut std::ffi::c_void, minimum: usize, maximum: usize) -> i32;
        }
        unsafe { SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX) != 0 }
    }
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        extern "C" { fn malloc_trim(pad: usize) -> i32; }
        unsafe { malloc_trim(256 * 1024) != 0 }
    }
    #[cfg(not(any(windows, all(target_os = "linux", target_env = "gnu"))))]
    { false }
}

pub fn get_cpu_profile() -> NativeCpuProfile {
    let mut profile = NativeCpuProfile {
        cores: std::thread::available_parallelism().map(|value| value.get() as u32).unwrap_or(1),
        avx2: false, avx512: false, sse42: false, aes: false,
        engine: "Luxmc Rust Native".into(),
    };
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        profile.avx2 = std::is_x86_feature_detected!("avx2");
        profile.avx512 = std::is_x86_feature_detected!("avx512f");
        profile.sse42 = std::is_x86_feature_detected!("sse4.2");
        profile.aes = std::is_x86_feature_detected!("aes");
    }
    profile
}

pub fn scan_log_native(log: &str) -> Option<String> {
    const SIGNATURES: &[(&str, &str)] = &[
        ("OutOfMemoryError", "Crash por falta de memória RAM alocada (OutOfMemoryError)"),
        ("WGL: The driver does not appear to support OpenGL", "Falha de inicialização gráfica no Windows (WGL/OpenGL incompatível)"),
        ("GLFW error 65548", "Erro de inicialização GLFW (ícone de janela não suportado no Wayland)"),
        ("GLFW error 65542", "Driver gráfico não suporta a versão necessária do OpenGL"),
        ("org.spongepowered.asm.mixin.transformer.throwables", "Conflito crítico de Mixin entre mods instalados"),
        ("java.lang.NoSuchMethodError", "Incompatibilidade de versões entre mods ou biblioteca ausente"),
        ("java.lang.ClassNotFoundException", "Dependência obrigatória de mod não encontrada"),
        ("optifine", "Possível incompatibilidade com OptiFine em versões modernas"),
        ("Pixel format not accelerated", "Aceleração por hardware OpenGL indisponível"),
        ("UnsupportedClassVersionError", "Versão do Java incompatível com esta versão do Minecraft"),
        ("Connection refused", "Servidor de destino offline ou porta incorreta"),
    ];
    SIGNATURES.iter().find(|(pattern, _)| log.contains(pattern)).map(|(_, diagnosis)| diagnosis.to_string())
}

pub fn get_memory_stats_native() -> NativeMemoryStats {
    #[cfg(windows)]
    {
        #[repr(C)]
        #[derive(Default)]
        struct Counters {
            cb: u32, page_faults: u32, peak_working_set: usize, working_set: usize,
            peak_paged_pool: usize, paged_pool: usize, peak_nonpaged_pool: usize,
            nonpaged_pool: usize, pagefile: usize, peak_pagefile: usize,
        }
        #[link(name = "kernel32")]
        extern "system" { fn GetCurrentProcess() -> *mut std::ffi::c_void; }
        #[link(name = "psapi")]
        extern "system" { fn GetProcessMemoryInfo(process: *mut std::ffi::c_void, counters: *mut Counters, size: u32) -> i32; }
        let mut counters = Counters::default();
        let size = std::mem::size_of::<Counters>() as u32;
        counters.cb = size;
        if unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, size) } != 0 {
            return NativeMemoryStats {rss_bytes:counters.working_set as u64,peak_bytes:counters.peak_working_set as u64,trimmed:false};
        }
    }
    let pid = sysinfo::Pid::from_u32(std::process::id());
    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
    let rss = system.process(pid).map(|process| process.memory()).unwrap_or(0);
    static PEAK: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let previous = PEAK.fetch_max(rss, std::sync::atomic::Ordering::Relaxed);
    let peak = rss.max(previous);
    #[cfg(target_os = "linux")]
    let peak = std::fs::read_to_string("/proc/self/status").ok().and_then(|status| status.lines().find_map(|line| line.strip_prefix("VmHWM:").and_then(|value| value.split_whitespace().next()).and_then(|value| value.parse::<u64>().ok()))).map(|kib| peak.max(kib.saturating_mul(1024))).unwrap_or(peak);
    NativeMemoryStats { rss_bytes: rss, peak_bytes: peak, trimmed: false }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_cpu_profile() {
        let profile = get_cpu_profile();
        assert!(profile.cores >= 1);
        assert_eq!(profile.engine, "Luxmc Rust Native");
    }
    #[test]
    fn native_log_signatures() {
        assert!(scan_log_native("java.lang.OutOfMemoryError: heap").unwrap().contains("OutOfMemoryError"));
        assert!(scan_log_native("[Client thread/INFO]: Setting user: Steve").is_none());
        assert!(scan_log_native("erro 🦀 Connection refused").unwrap().contains("offline"));
    }
    #[test]
    fn memory_snapshot() {
        let stats = get_memory_stats_native();
        assert!(stats.peak_bytes >= stats.rss_bytes);
    }
}
