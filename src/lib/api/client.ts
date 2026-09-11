import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export const api = {
	invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
		return invoke<T>(cmd, args);
	}
};

export { listen, type UnlistenFn };
