#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int luxmc_cpp_trim_memory();
int luxmc_cpp_get_cpu_features(char* out_buffer, size_t max_len);
int luxmc_cpp_fast_scan_log(const char* log_data, size_t data_len, char* out_error, size_t max_err_len);
int luxmc_cpp_get_memory_stats(uint64_t* out_rss, uint64_t* out_peak);

#ifdef __cplusplus
}
#endif
