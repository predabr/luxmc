import { api } from "./client";
import type { AppSettings } from "$lib/stores/settings.svelte";

export async function settingsGet(): Promise<Partial<AppSettings> | null> {
    return api.invoke<Partial<AppSettings> | null>("settings_get");
}

export async function settingsSet(value: Partial<AppSettings>): Promise<void> {
    return api.invoke<void>("settings_set", { value });
}
