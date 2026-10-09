import { api } from "./client";

export interface VirtualLanStatus {
	preparation?: { stage: "checking" | "downloading" | "verifying" | "installing" | "testing" | "authorizing"; downloadedBytes: number; totalBytes: number | null } | null;
	installed: boolean;
	supported: boolean;
	active: boolean;
	invitation: string | null;
	address: string | null;
	peers: { name: string; address: string }[];
	error: string | null;
}

export const virtualLanStatus = () => api.invoke<VirtualLanStatus>("virtual_lan_status");
export const virtualLanInstall = () => api.invoke<void>("virtual_lan_install");
export const virtualLanConnect = (name: string, invite: string | null = null) =>
	api.invoke<void>("virtual_lan_connect", { name, invite });
export const virtualLanStop = () => api.invoke<void>("virtual_lan_stop");

export interface VirtualLanWorlds {
	worlds: { id: string; owner: string; name: string; localAddress: string; version: string; latencyMs: number | null; available?: boolean }[];
	localWorld: { motd: string; port: number } | null;
	error: string | null;
}
export const virtualLanWorlds = () => api.invoke<VirtualLanWorlds>("virtual_lan_worlds");
export const virtualLanPrepareWorld = (id: string) => api.invoke<string>("virtual_lan_prepare_world", { id });
export const virtualLanWorldPort = (port: number) => api.invoke<void>("virtual_lan_world_port", { port });
