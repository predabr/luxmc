import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen as tauriListen, type UnlistenFn } from "@tauri-apps/api/event";

declare global {
	interface Window {
		electronAPI?: {
			invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown>;
			on: (channel: string, callback: (payload: unknown) => void) => () => void;
			window?: {
				minimize: () => Promise<void>;
				maximize: () => Promise<void>;
				close: () => Promise<void>;
				isMaximized: () => Promise<boolean>;
			};
		};
	}
}

function isElectron(): boolean {
	return typeof window !== "undefined" && typeof window.electronAPI !== "undefined";
}

export const api = {
	async invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
		if (isElectron()) {
			return window.electronAPI!.invoke(cmd, args) as Promise<T>;
		}
		return tauriInvoke<T>(cmd, args);
	}
};

export async function listen<T = unknown>(
	event: string,
	handler: (event: { payload: T }) => void
): Promise<UnlistenFn> {
	if (isElectron()) {
		const unsubscribe = window.electronAPI!.on(event, (payload) => {
			handler({ payload: payload as T });
		});
		return () => {
			unsubscribe();
		};
	}
	return tauriListen<T>(event, handler);
}

export type { UnlistenFn };

