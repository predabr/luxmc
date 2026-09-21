#include "luxmc_core.hpp"

#include <string>
#include <string_view>
#include <vector>
#include <cstring>
#include <thread>
#include <cstdio>

#if defined(__linux__)
#include <malloc.h>
#include <sys/resource.h>
#include <unistd.h>
#elif defined(_WIN32)
#include <windows.h>
#include <psapi.h>
#elif defined(__APPLE__)
#include <mach/mach.h>
#include <sys/resource.h>
#endif

#if defined(__x86_64__) || defined(_M_X64) || defined(__i386__) || defined(_M_IX86)
#if defined(_MSC_VER)
#include <intrin.h>
#else
#include <cpuid.h>
#endif
#define LUXMC_ARCH_X86 1
#else
#define LUXMC_ARCH_X86 0
#endif

extern "C" {

int luxmc_cpp_trim_memory() {
#if defined(__linux__) && defined(__GLIBC__)
    int res = malloc_trim(256 * 1024);
    return res ? 1 : 0;
#elif defined(_WIN32)
    BOOL ok = SetProcessWorkingSetSize(GetCurrentProcess(), (SIZE_T)-1, (SIZE_T)-1);
    return ok ? 1 : 0;
#else
    return 0;
#endif
}

int luxmc_cpp_get_cpu_features(char* out_buffer, size_t max_len) {
    if (!out_buffer || max_len == 0) return -1;

    unsigned int cores = std::thread::hardware_concurrency();
    bool has_avx2 = false;
    bool has_avx512 = false;
    bool has_sse42 = false;
    bool has_aes = false;

#if LUXMC_ARCH_X86
#if defined(_MSC_VER)
    int cpu_info[4];
    __cpuid(cpu_info, 1);
    has_sse42 = (cpu_info[2] & (1 << 20)) != 0;
    has_aes = (cpu_info[2] & (1 << 25)) != 0;

    int extended_info[4];
    __cpuidex(extended_info, 7, 0);
    has_avx2 = (extended_info[1] & (1 << 5)) != 0;
    has_avx512 = (extended_info[1] & (1 << 16)) != 0;
#elif defined(__GNUC__) || defined(__clang__)
    unsigned int eax, ebx, ecx, edx;
    if (__get_cpuid(1, &eax, &ebx, &ecx, &edx)) {
        has_sse42 = (ecx & (1 << 20)) != 0;
        has_aes = (ecx & (1 << 25)) != 0;
    }
    if (__get_cpuid_count(7, 0, &eax, &ebx, &ecx, &edx)) {
        has_avx2 = (ebx & (1 << 5)) != 0;
        has_avx512 = (ebx & (1 << 16)) != 0;
    }
#endif
#endif

    int written = snprintf(
        out_buffer, max_len,
        "{\"cores\":%u,\"avx2\":%s,\"avx512\":%s,\"sse42\":%s,\"aes\":%s,\"engine\":\"Luxmc C++17 Native\"}",
        cores,
        has_avx2 ? "true" : "false",
        has_avx512 ? "true" : "false",
        has_sse42 ? "true" : "false",
        has_aes ? "true" : "false"
    );

    return (written > 0 && (size_t)written < max_len) ? 0 : -1;
}

struct LogSignature {
    std::string_view pattern;
    std::string_view diagnosis;
};

static const LogSignature SIGNATURES[] = {
    {"OutOfMemoryError", "Crash por falta de memória RAM alocada (OutOfMemoryError)"},
    {"WGL: The driver does not appear to support OpenGL", "Falha de inicialização gráfica no Windows (WGL/OpenGL incompatível)"},
    {"GLFW error 65548", "Erro de inicialização GLFW (ícone de janela não suportado no Wayland)"},
    {"GLFW error 65542", "Driver gráfico não suporta a versão necessária do OpenGL"},
    {"org.spongepowered.asm.mixin.transformer.throwables", "Conflito crítico de Mixin entre mods instalados"},
    {"java.lang.NoSuchMethodError", "Incompatibilidade de versões entre mods ou biblioteca ausente"},
    {"java.lang.ClassNotFoundException", "Dependência obrigatória de mod não encontrada"},
    {"optifine", "Possível incompatibilidade com OptiFine em versões modernas"},
    {"Pixel format not accelerated", "Aceleração por hardware OpenGL indisponível"},
    {"UnsupportedClassVersionError", "Versão do Java incompatível com esta versão do Minecraft"},
    {"Connection refused", "Servidor de destino offline ou porta incorreta"}
};

int luxmc_cpp_fast_scan_log(const char* log_data, size_t data_len, char* out_error, size_t max_err_len) {
    if (!log_data || data_len == 0 || !out_error || max_err_len == 0) return 0;

    std::string_view sv(log_data, data_len);

    for (const auto& sig : SIGNATURES) {
        if (sv.find(sig.pattern) != std::string_view::npos) {
            size_t to_copy = sig.diagnosis.length();
            if (to_copy >= max_err_len) to_copy = max_err_len - 1;
            std::memcpy(out_error, sig.diagnosis.data(), to_copy);
            out_error[to_copy] = '\0';
            return 1;
        }
    }

    return 0;
}

int luxmc_cpp_get_memory_stats(uint64_t* out_rss, uint64_t* out_peak) {
    if (!out_rss || !out_peak) return -1;
    *out_rss = 0;
    *out_peak = 0;

#if defined(__linux__)
    struct rusage usage;
    if (getrusage(RUSAGE_SELF, &usage) == 0) {
        *out_peak = (uint64_t)usage.ru_maxrss * 1024ULL;
    }
    FILE* f = fopen("/proc/self/statm", "r");
    if (f) {
        long pages = 0;
        if (fscanf(f, "%*s %ld", &pages) == 1) {
            long page_size = sysconf(_SC_PAGESIZE);
            *out_rss = (uint64_t)pages * (uint64_t)page_size;
        }
        fclose(f);
    }
    return 0;
#elif defined(_WIN32)
    PROCESS_MEMORY_COUNTERS pmc;
    if (GetProcessMemoryInfo(GetCurrentProcess(), &pmc, sizeof(pmc))) {
        *out_rss = (uint64_t)pmc.WorkingSetSize;
        *out_peak = (uint64_t)pmc.PeakWorkingSetSize;
        return 0;
    }
    return -1;
#else
    return -1;
#endif
}

}
