import { api } from "./client";
import type {
	JvmValidationResult,
	JavaRuntimeInfo,
	CrashSummary,
	LaunchLogSummary,
	LaunchLogLine,
} from "./types";

export async function shareLogMclogs(content: string): Promise<string> {
	return api.invoke<string>("share_log_mclogs", { content });
}

export async function launchLogsList(limit = 100): Promise<LaunchLogSummary[]> {
	return api.invoke<LaunchLogSummary[]>("launch_logs_list", { limit });
}

export async function launchLogsGet(id: number): Promise<LaunchLogLine[]> {
	return api.invoke<LaunchLogLine[]>("launch_logs_get", { id });
}

export async function launchLogsSearch(query: string, minLevel: string, limit = 200): Promise<LaunchLogLine[]> {
	return api.invoke<LaunchLogLine[]>("launch_logs_search", { query, minLevel, limit });
}

export async function launchLogsClear(): Promise<void> {
	return api.invoke("launch_logs_clear");
}

export async function launchLogOpen(profileId: string | null, versionId: string): Promise<number> {
	return api.invoke<number>("launch_log_open", { profileId, versionId });
}

export async function launchLogAppend(logId: number, stream: string, level: string, message: string): Promise<void> {
	return api.invoke("launch_log_append", { logId, stream, level, message });
}

export async function launchLogClose(logId: number, exitCode: number | null, summary: string | null, errorClassification: string | null): Promise<void> {
	return api.invoke("launch_log_close", { logId, exitCode, summary, errorClassification });
}

export async function jvmArgsValidate(input: string): Promise<JvmValidationResult> {
	return api.invoke<JvmValidationResult>("jvm_args_validate", { input });
}

export async function javaRuntimeStatus(major: number): Promise<JavaRuntimeInfo> {
	return api.invoke<JavaRuntimeInfo>("java_runtime_status", { major });
}

export async function crashSummary(lines: string[]): Promise<CrashSummary> {
	return api.invoke<CrashSummary>("crash_summary", { lines });
}
