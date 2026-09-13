import { api } from "./client";
import type { JavaScanResult, JavaInstallStatus, DepCheckResult, ModpackUpdateInfo } from "./types";

export async function javaScan(): Promise<JavaScanResult> {
	return api.invoke<JavaScanResult>("java_scan");
}

export async function javaInstall(major: number): Promise<JavaInstallStatus> {
	return api.invoke<JavaInstallStatus>("java_install", { major });
}

export async function javaUninstall(major: number): Promise<void> {
	return api.invoke("java_uninstall", { major });
}

export async function modsCheckMissingDeps(
	profileId: string,
	mcVersion: string,
	loader: string
): Promise<DepCheckResult> {
	return api.invoke<DepCheckResult>("mods_check_missing_deps", { profileId, mcVersion, loader });
}

export async function modsInstallMissingDeps(
	profileId: string,
	mcVersion: string,
	loader: string,
	projectIds: string[]
): Promise<number> {
	return api.invoke<number>("mods_install_missing_deps", { profileId, mcVersion, loader, projectIds });
}

export async function modpackCheckUpdate(profileId: string): Promise<ModpackUpdateInfo> {
	return api.invoke<ModpackUpdateInfo>("modpack_check_update", { profileId });
}

export async function modpackUpdateAtomic(
	profileId: string,
	versionId: string,
	source: string,
	projectId: string
): Promise<void> {
	return api.invoke("modpack_update_atomic", { profileId, versionId, source, projectId });
}
