import { api } from "./client";

export interface KeybindEntry {
	id: string;
	label: string;
	category: string;
	rawKey: string;
	displayKey: string;
	isConflict: boolean;
	conflictingWith: string[];
}

export interface KeybindListResult {
	keybinds: KeybindEntry[];
	totalConflicts: number;
}

export async function keybindsList(profileId: string): Promise<KeybindListResult> {
	return api.invoke<KeybindListResult>("keybinds_list", { profileId });
}

export async function keybindsUpdate(
	profileId: string,
	updates: Record<string, string>
): Promise<boolean> {
	return api.invoke<boolean>("keybinds_update", { profileId, updates });
}
