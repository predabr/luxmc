import { api } from "./client";
import type {
	PerformancePackInfo,
	GpuInfo,
} from "./types";

export async function optimizerGetFlags(ramMb: number, autoOptimize: boolean): Promise<string[]> {
	return api.invoke("optimizer_get_flags", { ramMb, autoOptimize });
}

export async function optimizerGetPerfPack(loader: string, mcVersion: string): Promise<PerformancePackInfo> {
	return api.invoke("optimizer_get_perf_pack", { loader, mcVersion });
}

export async function optimizerInstallPerfPack(instanceId: string): Promise<string[]> {
	return api.invoke("optimizer_install_perf_pack", { instanceId });
}

export async function optimizerDetectGpu(): Promise<GpuInfo> {
	return api.invoke("optimizer_detect_gpu");
}
