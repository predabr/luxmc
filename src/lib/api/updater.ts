import { api } from "./client";

export interface UpdateEnvironment {
	mode: "appimage" | "pacman" | "debian" | "system" | "windows" | "macos" | "manual";
}

export interface UpdateOutcome {
	action: "system-installer" | "terminal" | "downloaded";
	terminalCommand: string | null;
}

export async function appUpdateEnvironment(): Promise<UpdateEnvironment> {
	return api.invoke<UpdateEnvironment>("app_update_environment");
}

export async function appPerformUpdate(downloadUrl: string): Promise<UpdateOutcome> {
	return api.invoke<UpdateOutcome>("app_perform_update", { downloadUrl });
}
