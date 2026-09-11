import { api } from "./client";
import type {
	HealthCheckResult,
	FileTreeEntry,
	WorldDetail,
} from "./types";

export async function profilesList(): Promise<Array<{
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
}>> {
	return api.invoke("profiles_list");
}

export async function profilesGet(id: string): Promise<{
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
}> {
	return api.invoke("profiles_get", { id });
}

export async function profilesCreate(input: {
	name: string;
	icon?: string;
	mcVersion: string;
	loader: string;
	loaderVersion?: string;
	javaPath?: string;
	jvmArgs?: string;
	resolutionW?: number;
	resolutionH?: number;
	fullscreen?: boolean;
	gameDir?: string;
	ramMb?: number;
	autoOptimize?: boolean;
	useVulkan?: boolean;
}): Promise<{
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
	ramMb?: number | null;
	autoOptimize: boolean;
	useVulkan: boolean;
	createdAt: string;
	updatedAt: string;
}> {
	return api.invoke("profiles_create", { input });
}

export async function profilesUpdate(input: {
	id: string;
	name?: string;
	icon?: string;
	mcVersion?: string;
	loader?: string;
	loaderVersion?: string | null;
	javaPath?: string | null;
	jvmArgs?: string | null;
	resolutionW?: number | null;
	resolutionH?: number | null;
	fullscreen?: boolean;
	gameDir?: string;
	ramMb?: number;
	autoOptimize?: boolean;
	useVulkan?: boolean;
	lastPlayed?: string;
	launchCount?: number;
}): Promise<{
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
	ramMb?: number | null;
	autoOptimize: boolean;
	useVulkan: boolean;
	createdAt: string;
	updatedAt: string;
}> {
	return api.invoke("profiles_update", { input });
}

export async function profilesDelete(id: string): Promise<void> {
	return api.invoke("profiles_delete", { id });
}

export async function instancesList(): Promise<Array<{
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
}>> {
	return api.invoke("instances_list");
}

export async function instancesDuplicate(id: string): Promise<{
	id: string;
	name: string;
	icon: string;
	mcVersion: string;
	loader: string;
	loaderVersion: string | null;
	gameDir: string;
	createdAt: string;
	updatedAt: string;
}> {
	return api.invoke("instances_duplicate", { id });
}

export async function instancesOpenFolder(id: string): Promise<void> {
	return api.invoke("instances_open_folder", { id });
}

export async function instancesScreenshots(id: string): Promise<Array<{
	name: string;
	path: string;
	modified: string;
	dataUrl?: string | null;
}>> {
	return api.invoke("instances_screenshots", { id });
}

export async function screenshotDelete(path: string): Promise<void> {
	return api.invoke("screenshot_delete", { path });
}

export async function screenshotsOpenFolder(profileId: string): Promise<void> {
	return api.invoke("screenshots_open_folder", { profileId });
}

export async function instanceImportModpack(
	filePath: string,
	profileName: string,
	mcVersion: string,
	loader: string,
	icon?: string
): Promise<{
	id: string;
	name: string;
	mcVersion: string;
	loader: string;
	gameDir: string;
	icon?: string;
}> {
	return api.invoke("instance_import_modpack", { filePath, profileName, mcVersion, loader, icon });
}

export async function instanceImportMrpack(
	filePath: string,
	profileName: string,
	icon?: string
): Promise<{
	id: string;
	name: string;
	mcVersion: string;
	loader: string;
	gameDir: string;
	icon?: string;
}> {
	return api.invoke("instance_import_mrpack", { filePath, profileName, icon });
}

export async function instanceHealthCheck(profileId: string): Promise<HealthCheckResult> {
	return api.invoke("instance_health_check", { profileId });
}

export async function instanceFileTree(profileId: string, subPath?: string): Promise<FileTreeEntry[]> {
	return api.invoke("instance_file_tree", { profileId, subPath });
}

export async function instanceExport(profileId: string, destPath: string): Promise<string> {
	return api.invoke("instance_export", { profileId, destPath });
}

export async function instanceSetNotes(profileId: string, notes: string | null): Promise<void> {
	return api.invoke("instance_set_notes", { profileId, notes });
}

export async function instanceSetFavorite(profileId: string, favorite: boolean): Promise<void> {
	return api.invoke("instance_set_favorite", { profileId, favorite });
}

export async function instanceModToggle(profileId: string, fileName: string, enabled: boolean): Promise<string> {
	return api.invoke<string>("instance_mod_toggle", { profileId, fileName, enabled });
}

export async function instanceModDelete(profileId: string, fileName: string): Promise<void> {
	return api.invoke("instance_mod_delete", { profileId, fileName });
}

export async function instanceModAdd(profileId: string, sourcePath: string): Promise<string> {
	return api.invoke<string>("instance_mod_add", { profileId, sourcePath });
}

export async function instanceModsOpenFolder(profileId: string): Promise<void> {
	return api.invoke("instance_mods_open_folder", { profileId });
}

export async function instancePackAdd(profileId: string, packType: string, sourcePath: string): Promise<string> {
	return api.invoke<string>("instance_pack_add", { profileId, packType, sourcePath });
}

export async function instancePackDelete(profileId: string, packType: string, fileName: string): Promise<void> {
	return api.invoke("instance_pack_delete", { profileId, packType, fileName });
}

export async function instancePackOpenFolder(profileId: string, packType: string): Promise<void> {
	return api.invoke("instance_pack_open_folder", { profileId, packType });
}

export async function instanceExportZip(profileId: string, outputPath: string): Promise<string> {
	return api.invoke<string>("instance_export_zip", { profileId, outputPath });
}

export async function instanceBackupSaves(profileId: string, outputPath: string): Promise<string> {
	return api.invoke<string>("instance_backup_saves", { profileId, outputPath });
}

export async function instanceRestoreSaves(profileId: string, zipPath: string): Promise<string> {
	return api.invoke<string>("instance_restore_saves", { profileId, zipPath });
}

export async function instanceRepair(profileId: string): Promise<void> {
	return api.invoke("instance_repair", { profileId });
}

export async function instanceDiskUsage(profileId: string): Promise<number> {
	return api.invoke<number>("instance_disk_usage", { profileId });
}

export async function versionRepair(versionId: string): Promise<void> {
	return api.invoke("version_repair", { versionId });
}

export async function instanceWorldsList(profileId: string): Promise<WorldDetail[]> {
	return api.invoke<WorldDetail[]>("instance_worlds_list", { profileId });
}

export async function instanceWorldDelete(profileId: string, folderName: string): Promise<void> {
	return api.invoke<void>("instance_world_delete", { profileId, folderName });
}
