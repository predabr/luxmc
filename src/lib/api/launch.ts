import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
import { z } from "zod";
import { api } from "./client";
import { appState } from "$lib/stores/app.svelte";

export function sanitizeLaunchDiagnostic(value: unknown): string {
    return String(value).replace(/(?:authorization[\s:="']+(?:bearer\s+)?|bearer\s+|(?:access[_-]?token|refresh[_-]?token|session[_-]?id|password)[\s:="']+)[^\s"',;]+|eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+/gi, "[redacted]").slice(0, 4096);
}

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
}): Promise<{ pid: number; sessionId?: string }> {
    const revision = appState.gameSessionRevision;
    appState.launchAttemptStartedAt = Date.now();
    appState.lastLaunchError = "";
    appState.wasManuallyTerminated = false;
    const sessionId = crypto.randomUUID();
    appState.activeLaunchSessionId = sessionId;
    const result = await api.invoke<{ pid: number; sessionId?: string }>("launch_game", { request: { ...request, sessionId } }).catch(error => {
        appState.lastLaunchError = sanitizeLaunchDiagnostic(error);
        throw new Error(appState.lastLaunchError);
    });
    if (revision !== appState.gameSessionRevision || result.pid <= 0) {
        appState.lastLaunchError = "O Minecraft encerrou durante o início. Consulte os logs da instância.";
        throw new Error("O Minecraft encerrou durante o início. Consulte os logs da instância.");
    }
    appState.activeLaunchSessionId = result.sessionId || sessionId;
    return result;
}

export async function gameLaunchStatus(pid: number, sessionId?: string): Promise<{ running: boolean; windowReady: boolean }> {
    return api.invoke("game_launch_status", { pid, ...(sessionId ? { sessionId } : {}) });
}

export interface LaunchSession {
    sessionId: string;
    profileId: string;
    versionId: string;
    phase: "preparing" | "startingJava" | "waitingForWindow" | "opened" | "failed" | "cancelled" | "exited";
    stage: string;
    pid: number;
    startedAt: string;
    updatedAt: string;
    elapsedSeconds: number;
    error: string | null;
    exitCode: number | null;
}

export function launchSessionStatus(): Promise<LaunchSession | null> {
    return api.invoke("launch_session_status");
}

export function cancelLaunch(sessionId: string): Promise<boolean> {
    return api.invoke("cancel_launch", { sessionId });
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
	if (!resp.ok) throw new Error(uiText("ui.fcb9e551a3386696", {arg0: (resp.status)}));
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
