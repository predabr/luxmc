import { api } from "./client";

export interface SavedSkinRecord {
	id: string;
	name: string;
	skinUrl: string;
	avatarUrl: string;
	modelType: string;
	isCustom: boolean;
	createdAt: string;
}

export interface SaveSkinPayload {
	id?: string;
	name: string;
	skinUrl: string;
	avatarUrl?: string;
	modelType?: string;
	isCustom?: boolean;
}

export async function skinsList(): Promise<SavedSkinRecord[]> {
	return api.invoke("skins_list");
}

export async function skinsSave(request: SaveSkinPayload): Promise<SavedSkinRecord> {
	return api.invoke("skins_save", { request });
}

export async function skinsDelete(id: string): Promise<void> {
	return api.invoke("skins_delete", { id });
}

export async function skinsImportFile(
	path: string,
	name?: string,
	modelType?: string
): Promise<SavedSkinRecord> {
	return api.invoke("skins_import_file", { path, name, modelType });
}

export async function gamingStatsGet(): Promise<string | null> {
	return api.invoke("gaming_stats_get");
}

export async function gamingStatsSave(data: string): Promise<void> {
	return api.invoke("gaming_stats_save", { data });
}

export interface SavedCapeRecord {
	id: string;
	name: string;
	capeUrl: string;
	createdAt: string;
}

export interface SaveCapePayload {
	id?: string;
	name: string;
	capeUrl: string;
}

export async function capesList(): Promise<SavedCapeRecord[]> {
	return api.invoke("capes_list");
}

export async function capesSave(request: SaveCapePayload): Promise<SavedCapeRecord> {
	return api.invoke("capes_save", { request });
}

export async function capesDelete(id: string): Promise<void> {
	return api.invoke("capes_delete", { id });
}


const uuidRequests = new Map<string, Promise<string | null>>();
export async function minecraftUuid(username: string): Promise<string | null> {
    const key = username.toLowerCase();
    if (!/^[a-zA-Z0-9_]{1,16}$/.test(key)) return null;
    const existing = uuidRequests.get(key);
    if (existing) return existing;
    const request = api.invoke<string | null>("minecraft_uuid", { username }).catch(error => { uuidRequests.delete(key); throw error; });
    if (uuidRequests.size >= 256) uuidRequests.clear();
    uuidRequests.set(key, request);
    return request;
}
