import { api } from "./client";

export interface CapsuleInfo {
    filename: string;
    label: string;
    createdAt: string;
    sizeBytes: number;
}

export interface IsolationState {
    profileId: string;
    originalEnabled: string[];
    suspects: string[];
    trialDisabled: string[];
    round: number;
    phase: "testing" | "found";
}

export interface BenchmarkResult {
    id: string;
    label: string;
    createdAt: string;
    averageFps: number;
    lowOnePercentFps: number;
    fpsDrops: number;
    samples: number;
    source: string;
}

export const instanceCapsuleCreate = (profileId: string, label: string) => api.invoke<CapsuleInfo>("instance_capsule_create", { profileId, label });
export const instanceCapsulesList = (profileId: string) => api.invoke<CapsuleInfo[]>("instance_capsules_list", { profileId });
export const instanceCapsuleRestore = (profileId: string, filename: string) => api.invoke<CapsuleInfo>("instance_capsule_restore", { profileId, filename });
export const instanceIsolationStatus = (profileId: string) => api.invoke<IsolationState | null>("instance_isolation_status", { profileId });
export const instanceIsolationStart = (profileId: string) => api.invoke<IsolationState>("instance_isolation_start", { profileId });
export const instanceIsolationReport = (profileId: string, crashed: boolean) => api.invoke<IsolationState>("instance_isolation_report", { profileId, crashed });
export const instanceIsolationRestore = (profileId: string) => api.invoke<void>("instance_isolation_restore", { profileId });
export const instanceBenchmarksList = (profileId: string) => api.invoke<BenchmarkResult[]>("instance_benchmarks_list", { profileId });
export const instanceBenchmarkImport = (profileId: string, label: string, csvPath: string) => api.invoke<BenchmarkResult>("instance_benchmark_import", { profileId, label, csvPath });
