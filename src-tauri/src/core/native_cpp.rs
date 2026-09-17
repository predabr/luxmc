use std::ffi::CStr;
use std::os::raw::c_char;
use serde::{Deserialize, Serialize};

extern "C" {
    fn luxmc_cpp_trim_memory() -> i32;
    fn luxmc_cpp_get_cpu_features(out_buffer: *mut c_char, max_len: usize) -> i32;
    fn luxmc_cpp_fast_scan_log(
        log_data: *const c_char,
        data_len: usize,
        out_error: *mut c_char,
        max_err_len: usize,
    ) -> i32;
    fn luxmc_cpp_get_memory_stats(out_rss: *mut u64, out_peak: *mut u64) -> i32;
}

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
    unsafe { luxmc_cpp_trim_memory() == 1 }
}

pub fn get_cpu_profile() -> NativeCpuProfile {
    let mut buf = vec![0u8; 512];
    let res = unsafe {
        luxmc_cpp_get_cpu_features(buf.as_mut_ptr() as *mut c_char, buf.len())
    };

    if res == 0 {
        if let Ok(c_str) = unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char) }.to_str() {
            if let Ok(profile) = serde_json::from_str::<NativeCpuProfile>(c_str) {
                return profile;
            }
        }
    }

    NativeCpuProfile {
        cores: 1,
        avx2: false,
        avx512: false,
        sse42: false,
        aes: false,
        engine: "Fallback".into(),
    }
}

pub fn scan_log_native(log: &str) -> Option<String> {
    if log.is_empty() {
        return None;
    }

    let mut out_buf = vec![0u8; 1024];
    let res = unsafe {
        luxmc_cpp_fast_scan_log(
            log.as_ptr() as *const c_char,
            log.len(),
            out_buf.as_mut_ptr() as *mut c_char,
            out_buf.len(),
        )
    };

    if res == 1 {
        let c_str = unsafe { CStr::from_ptr(out_buf.as_ptr() as *const c_char) };
        c_str.to_str().ok().map(|s| s.to_string())
    } else {
        None
    }
}

pub fn get_memory_stats_native() -> NativeMemoryStats {
    let mut rss = 0u64;
    let mut peak = 0u64;
    let _ = unsafe { luxmc_cpp_get_memory_stats(&mut rss, &mut peak) };
    NativeMemoryStats {
        rss_bytes: rss,
        peak_bytes: peak,
        trimmed: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_cpp_cpu_profile() {
        let profile = get_cpu_profile();
        assert!(profile.cores >= 1);
        assert_eq!(profile.engine, "Luxmc C++17 Native");
    }

    #[test]
    fn test_native_cpp_trim() {
        let _ = trim_memory_native();
    }

    #[test]
    fn test_native_cpp_log_scanner() {
        let test_log = "Error in thread 'main' java.lang.OutOfMemoryError: Java heap space";
        let diag = scan_log_native(test_log);
        assert!(diag.is_some());
        assert!(diag.unwrap().contains("OutOfMemoryError"));

        let normal_log = "[Client thread/INFO]: Setting user: Steve";
        assert!(scan_log_native(normal_log).is_none());
    }
}
