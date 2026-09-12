import { api } from "./client";
import type {
	EnvCheckResult,
	StorageBreakdown,
	ChangelogEntry,
} from "./types";

export async function ping(): Promise<string> {
	return api.invoke<string>("ping");
}

export async function appInit(): Promise<{
	devMode: boolean;
	account: {
		id: string;
		username: string;
		uuid: string;
		refreshToken: string;
		accessToken: string | null;
		expiresAt: string | null;
		createdAt: string;
		updatedAt: string;
		skinUrl?: string | null;
		skinVariant?: string | null;
		capeUrl?: string | null;
	} | null;
	profiles: Array<{
		id: string;
		name: string;
		icon: string;
		mcVersion: string;
		loader: string;
		loaderVersion: string | null;
		javaPath: string | null;
		jvmArgs: string | null;
		resolutionW: number | null;
		resolutionH: number | null;
		fullscreen: boolean;
		gameDir: string;
		createdAt: string;
		updatedAt: string;
		favorite: boolean;
		notes: string | null;
		lastPlayed: string | null;
		launchCount: number;
		modCount: number;
		diskUsage: number;
		ramMb: number | null;
		instanceGroup: string | null;
	}>;
	activeProfileId: string | null;
	stressTest?: boolean;
}> {
	return api.invoke("app_init");
}

export async function envCheck(): Promise<EnvCheckResult> {
	return api.invoke("env_check");
}

export async function getSystemSpecs(): Promise<{
	osDistro: string;
	kernelVersion: string;
	arch: string;
	totalRamMb: number;
	launcherVersion: string;
	gpuVendor: string;
	gpuRenderer: string;
	gpuSupportsZink: boolean;
}> {
	return api.invoke("get_system_specs");
}

export async function changelogGet(): Promise<ChangelogEntry[]> {
	return api.invoke<ChangelogEntry[]>("changelog_get");
}

export async function storageBreakdown(): Promise<StorageBreakdown[]> {
	return api.invoke<StorageBreakdown[]>("storage_breakdown");
}

export async function storageTotal(paths: string[]): Promise<number> {
	return api.invoke<number>("storage_total", { paths });
}

export async function directoryExists(path: string): Promise<boolean> {
	return api.invoke<boolean>("directory_exists", { path });
}

export async function ensureDirectory(path: string): Promise<void> {
	return api.invoke("ensure_directory", { path });
}

export async function readTextFile(path: string): Promise<string> {
	return api.invoke<string>("read_text_file", { path });
}

export async function writeTextFile(path: string, contents: string): Promise<void> {
	return api.invoke("write_text_file", { path, contents });
}

export async function deleteFileOrDir(path: string): Promise<void> {
	return api.invoke("delete_file_or_dir", { path });
}
