import { settings } from "$lib/stores/settings.svelte";
import { goto } from "$app/navigation";

export async function handlePostLaunchActions(): Promise<void> {
	if (settings.value.showLogsOnLaunch === "always") {
		goto("/logs").catch(() => {});
	}

	if (settings.value.launcherActionOnLaunch === "hide_reopen") {
		try {
			const { getCurrentWindow } = await import("@tauri-apps/api/window");
			await getCurrentWindow().hide();
		} catch {}
	} else if (settings.value.launcherActionOnLaunch === "close") {
		try {
			const { getCurrentWindow } = await import("@tauri-apps/api/window");
			await getCurrentWindow().close();
		} catch {}
	}
}
