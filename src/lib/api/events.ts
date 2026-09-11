import { listen, type UnlistenFn } from "./client";
import type {
	DownloadProgress,
	GameLogEntry,
	GameExitEvent,
} from "./types";

export async function listenDownloadProgress(
	callback: (progress: DownloadProgress) => void,
): Promise<UnlistenFn> {
	return listen<DownloadProgress>("download-progress", (event) => {
		callback(event.payload);
	});
}

export async function listenGameLog(
	callback: (entry: GameLogEntry) => void,
): Promise<UnlistenFn> {
	return listen<GameLogEntry>("game-log", (event) => {
		callback(event.payload);
	});
}

export async function listenGameExit(
	callback: (event: GameExitEvent) => void,
): Promise<UnlistenFn> {
	return listen<GameExitEvent>("game-exit", (event) => {
		callback(event.payload);
	});
}

export async function listenLauncherLog(
	callback: (message: string) => void,
): Promise<UnlistenFn> {
	return listen<string>("launcher-log", (event) => {
		callback(event.payload);
	});
}
