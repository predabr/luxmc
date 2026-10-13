import { settings } from "$lib/stores/settings.svelte";
import { appState } from "$lib/stores/app.svelte";
import { gameLaunchStatus } from "$lib/api/launch";
import { goto } from "$app/navigation";

export async function handlePostLaunchActions(pid: number): Promise<void> {
	if (settings.value.showLogsOnLaunch === "always") {
		goto("/logs").catch(() => {});
	}

    const hidesLauncher = () => settings.value.launcherActionOnLaunch === "hide_reopen" || settings.value.launcherActionOnLaunch === "close";
    if (!hidesLauncher()) return;
    const revision = appState.gameSessionRevision;
    const sessionId = appState.activeLaunchSessionId || undefined;
    const deadline = Date.now() + 180_000;
    while (Date.now() < deadline && revision === appState.gameSessionRevision && appState.isGameRunning) {
        try {
            const status = await gameLaunchStatus(pid, sessionId);
            if (!status.running) return;
            if (status.windowReady) {
                const { getCurrentWindow } = await import("@tauri-apps/api/window");
                if (revision !== appState.gameSessionRevision || !appState.isGameRunning) return;
                const confirmed = await gameLaunchStatus(pid, sessionId);
                if (!confirmed.running || !confirmed.windowReady) return;
                if (!hidesLauncher()) return;
                const win = getCurrentWindow();
                await win.hide();
                if (revision !== appState.gameSessionRevision || !appState.isGameRunning) await win.show();
                return;
            }
        } catch { return; }
        await new Promise(resolve => setTimeout(resolve, 1_000));
    }
}
