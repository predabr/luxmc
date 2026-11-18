import { invoke } from "@tauri-apps/api/core";

export type PluginPermission = "network" | "storage" | "profiles";

export type PluginManifest = {
	id: string;
	name: string;
	version: string;
	author: string;
	description: string;
	homepage?: string | null;
	main: string;
	permissions: PluginPermission[];
};

export type PluginFinding = {
	severity: "critical" | "warning" | "info" | string;
	code: string;
	message: string;
	file: string;
};

export type PluginScan = {
	clean: boolean;
	scannedAt: string;
	findings: PluginFinding[];
	files: number;
	bytes: number;
};

export type PluginRecord = {
	manifest: PluginManifest;
	enabled: boolean;
	installedAt: string;
	sha256: string;
	sizeBytes: number;
	scan: PluginScan;
	cover?: string | null;
	community?: boolean;
};

export type RegistryPlugin = {
	id: string;
	name: string;
	version: string;
	author: string;
	description: string;
	homepage?: string | null;
	downloadUrl?: string | null;
	sha256?: string | null;
	iconUrl?: string | null;
	tags: string[];
};

export type PluginRegistry = {
	plugins: RegistryPlugin[];
	source: string;
	fetchedAt: string;
};

export type PublishedPlugin = {
	zipPath: string;
	entryJson: string;
	sha256: string;
	sizeBytes: number;
	coverDataUrl?: string | null;
};

export type PluginScript = {
	manifest: PluginManifest;
	script: string;
};

export const pluginsApi = {
	list: () => invoke<PluginRecord[]>("plugins_list"),
	registry: (url?: string | null) => invoke<PluginRegistry>("plugins_registry", { url: url ?? null }),
	install: (id: string, url: string, sha256?: string | null) =>
		invoke<PluginRecord>("plugins_install", { id, url, sha256: sha256 ?? null }),
	installLocal: (path: string) => invoke<PluginRecord>("plugins_install_local", { path }),
	uninstall: (id: string) => invoke<void>("plugins_uninstall", { id }),
	setEnabled: (id: string, enabled: boolean) => invoke<PluginRecord>("plugins_set_enabled", { id, enabled }),
	rescan: (id?: string | null) => invoke<PluginRecord[]>("plugins_rescan", { id: id ?? null }),
	script: (id: string) => invoke<PluginScript>("plugins_script", { id }),
	create: (payload: { id: string; name: string; author: string; description: string; permissions: PluginPermission[]; source: string }) =>
		invoke<PluginRecord>("plugins_create", payload),
	updateSource: (id: string, source: string) => invoke<PluginRecord>("plugins_update_source", { id, source }),
	source: (id: string) => invoke<string>("plugins_source", { id }),
	openFolder: () => invoke<string>("plugins_open_folder"),
	setCover: (id: string, path: string, premium: boolean) => invoke<PluginRecord>("plugins_set_cover", { id, path, premium }),
	clearCover: (id: string) => invoke<PluginRecord>("plugins_clear_cover", { id }),
	cover: (id: string) => invoke<string | null>("plugins_cover", { id }),
	publish: (id: string, publicBase?: string | null) => invoke<PublishedPlugin>("plugins_publish", { id, publicBase: publicBase ?? null }),
};
