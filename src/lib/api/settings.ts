import { api } from "./client";

export async function settingsGet(): Promise<unknown> {
	return api.invoke("settings_get");
}

export async function settingsSet(value: unknown): Promise<void> {
	return api.invoke("settings_set", { value });
}
