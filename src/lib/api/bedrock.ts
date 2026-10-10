import { api } from "./client";

export interface BedrockInstallation {
	id: string;
	profileId: string;
	profileName: string;
	name: string;
	version: string;
	directory: string;
}

export interface BedrockInstance {
	id: string;
	name: string;
	installationId: string;
	profileId: string;
}

export interface BedrockState {
	provider: { executable: string; dataDirectory: string } | null;
	installations: BedrockInstallation[];
	instances: BedrockInstance[];
	warning: string | null;
	supported: boolean;
}

export const bedrockState = () => api.invoke<BedrockState>("bedrock_state");
export interface BedrockVersion { id: string; version: string; channel: string; packageType: string; packageVersion: string; urls: string[] }
export interface BedrockInstallProgress { phase: string; percent: number; downloaded: number; total: number }
export const bedrockVersions = () => api.invoke<BedrockVersion[]>("bedrock_versions");
export const bedrockInstall = (versionId: string, name: string) => api.invoke<void>("bedrock_install", { versionId, name });
export const bedrockCancelInstall = () => api.invoke<void>("bedrock_cancel_install");
export const bedrockConnect = (executable: string, dataDirectory: string) =>
	api.invoke<void>("bedrock_connect", { executable, dataDirectory });
export const bedrockAdd = (name: string, profileId: string, installationId: string) =>
	api.invoke<void>("bedrock_add", { name, profileId, installationId });
export const bedrockRemove = (id: string) => api.invoke<void>("bedrock_remove", { id });
export const bedrockRename = (id: string, name: string) => api.invoke<void>("bedrock_rename", { id, name });
export const bedrockOpen = (instanceId: string | null = null) =>
	api.invoke<{ providerOpened: boolean; notice: string | null }>("bedrock_open", { instanceId });
export interface BedrockContentEntry { name: string; title: string; kind: 'resources' | 'behaviors' | 'worlds'; icon: string | null }
export interface BedrockInstanceContent { directory: string; entries: BedrockContentEntry[]; isolated: boolean; worldAccounts: string[]; worldAccount: string | null }
export const bedrockInstanceContent = (id: string) => api.invoke<BedrockInstanceContent>('bedrock_instance_content', { id });
export const bedrockImportContent = (id: string, kind: string, path: string) => api.invoke<void>('bedrock_import_content', { id, kind, path });
export const bedrockDeleteContent = (id: string, kind: string, name: string) => api.invoke<void>('bedrock_delete_content', { id, kind, name });
export const bedrockOpenFolder = (id: string) => api.invoke<void>('bedrock_open_folder', { id });
export const bedrockSelectWorldAccount = (id: string, account: string) => api.invoke<void>('bedrock_select_world_account', { id, account });
