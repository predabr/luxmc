import { get, set, del, clear } from "idb-keyval";

export async function idbCacheSet<T>(key: string, value: T): Promise<void> {
	try {
		await set(key, value);
	} catch {}
}

export async function idbCacheGet<T>(key: string): Promise<T | null> {
	try {
		const val = await get<T>(key);
		return val !== undefined ? val : null;
	} catch {
		return null;
	}
}

export async function idbCacheDelete(key: string): Promise<void> {
	try {
		await del(key);
	} catch {}
}

export async function idbCacheClear(): Promise<void> {
	try {
		await clear();
	} catch {}
}
