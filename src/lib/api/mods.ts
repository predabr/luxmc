import { api } from "./client";
import type {
	ModSearchResultItem,
	ModVersion,
	ModProjectDetails,
} from "./types";

export async function modsSearch(
	query: string,
	mcVersion: string,
	limit?: number,
	offset?: number,
	contentType?: string,
	sortBy?: string,
	loader?: string,
	category?: string,
	source?: string
): Promise<Array<ModSearchResultItem>> {
	return api.invoke("mods_search", {
		query,
		mcVersion,
		limit,
		offset,
		contentType,
		sortBy,
		loader,
		category,
		source
	});
}

export async function modsSearchTyped(
	query: string,
	mcVersion: string,
	contentType: string,
	limit?: number,
	offset?: number,
	sortBy?: string,
	loader?: string,
	category?: string,
	source?: string
): Promise<Array<ModSearchResultItem>> {
	return api.invoke("mods_search_typed", {
		query,
		mcVersion,
		contentType,
		limit,
		offset,
		sortBy,
		loader,
		category,
		source
	});
}

export async function modsVersions(projectId: string, mcVersion: string, source?: string): Promise<ModVersion[]> {
	return api.invoke("mods_versions", { projectId, mcVersion, source });
}

export async function modsProjectDetails(projectId: string, source?: string): Promise<ModProjectDetails> {
	return api.invoke("mods_project_details", { projectId, source });
}

export async function modsList(profileId: string): Promise<Array<{
	profileId: string;
	projectId: string;
	versionId: string;
	fileName: string;
	sha1: string;
	source: string;
	installedAt: string;
}>> {
	return api.invoke("mods_list", { profileId });
}

export async function modsInstall(request: {
	profileId: string;
	projectId: string;
	versionId: string;
	source: string;
	contentType?: string;
}): Promise<void> {
	return api.invoke("mods_install", { request });
}

export async function modsRemove(profileId: string, projectId: string): Promise<void> {
	return api.invoke("mods_remove", { profileId, projectId });
}

export async function modsCheckUpdates(profileId: string): Promise<Array<{
	projectId: string;
	projectTitle: string;
	currentVersionId: string;
	currentVersionNumber: string;
	latestVersionId: string;
	latestVersionNumber: string;
}>> {
	return api.invoke("mods_check_updates", { profileId });
}

export async function modsUpdate(projectId: string, versionId: string, profileId: string): Promise<void> {
	return api.invoke("mods_update", { projectId, versionId, profileId });
}

export async function modsDownloadToTemp(url: string, fileName: string): Promise<string> {
	return api.invoke("mods_download_to_temp", { url, fileName });
}

export async function curseforgeStatus(): Promise<boolean> {
	return api.invoke("curseforge_status");
}

export async function curseforgeGetKey(): Promise<string | null> {
	return api.invoke("curseforge_get_key");
}

export async function curseforgeSetKey(key: string): Promise<void> {
	return api.invoke("curseforge_set_key", { key });
}

export async function curseforgeRemoveKey(): Promise<void> {
	return api.invoke("curseforge_remove_key");
}

export async function curseforgeValidateKey(): Promise<boolean> {
	return api.invoke("curseforge_validate_key");
}

export async function modsResolveNames(profileId: string): Promise<number> {
	return api.invoke("mods_resolve_names", { profileId });
}
