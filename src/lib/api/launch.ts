import { api } from "./client";

export async function launchGame(request: {
	versionId: string;
	accountId: string;
	profileId: string;
	enableVulkan?: boolean;
}): Promise<{ pid: number }> {
	return api.invoke("launch_game", { request });
}

export async function loadersVersions(loader: string, mcVersion: string): Promise<{
	versions: Array<{
		id: string;
		stable: boolean;
	}>;
}> {
	return api.invoke("loaders_versions", { loader, mcVersion });
}

export async function versionsList(): Promise<{
	versions: Array<{
		id: string;
		versionType: string;
		url: string;
		releaseTime: string;
	}>;
	latestRelease: string;
	latestSnapshot: string;
}> {
	return api.invoke("versions_list");
}

export async function fetchVersionsDirect(): Promise<{
	versions: Array<{ id: string; versionType: string; releaseTime: string }>;
	latestRelease: string;
	latestSnapshot: string;
}> {
	const resp = await fetch("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json");
	const data = await resp.json();
	return {
		versions: data.versions.map((v: any) => ({
			id: v.id,
			versionType: v.type,
			releaseTime: v.releaseTime,
		})),
		latestRelease: data.latest.release,
		latestSnapshot: data.latest.snapshot,
	};
}

export async function versionsDetail(id: string): Promise<{
	id: string;
	versionType: string;
	mainClass: string;
	assetIndex: { id: string; url: string; size: number };
}> {
	return api.invoke("versions_detail", { id });
}

export async function versionsDownload(id: string): Promise<void> {
	return api.invoke("versions_download", { id });
}

export async function versionsCheckInstalled(id: string): Promise<boolean> {
	return api.invoke("versions_check_installed", { id });
}
