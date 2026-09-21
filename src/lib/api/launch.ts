import { z } from "zod";
import { api } from "./client";

export async function launchGame(request: {
	versionId: string;
	accountId: string;
	profileId: string;
	enableVulkan?: boolean;
	skinUrl?: string | null;
	skinVariant?: string | null;
	capeUrl?: string | null;
	serverIp?: string | null;
	serverPort?: number | null;
}): Promise<{ pid: number }> {
	return api.invoke("launch_game", { request });
}

export async function stopGame(pid?: number): Promise<boolean> {
	return api.invoke("stop_game", { pid });
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
	if (!resp.ok) throw new Error(`Falha ao carregar versões: HTTP ${resp.status}`);
	const data = z.object({ latest: z.object({ release: z.string(), snapshot: z.string() }), versions: z.array(z.object({ id: z.string(), type: z.string(), releaseTime: z.string() })) }).parse(await resp.json());
	return {
		versions: data.versions.map((v) => ({
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
