import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
import { getSystemSpecs } from "$lib/api/system";
import { detectPlatform, type LauncherPlatform } from "$lib/utils/platform";

let platform = $state<LauncherPlatform>(typeof navigator === "undefined" ? "unknown" : detectPlatform(navigator.userAgent));

export const runtimePlatform = {
    get current() { return platform; },
    get isLinux() { return platform === "linux"; },
    get isWindows() { return platform === "windows"; },
    get label() { return { windows: "Windows", linux: "Linux", macos: "macOS", unknown: uiText("ui.f150afd3c5994d18") }[platform]; },
    async refresh() {
        try {
            const specs = await getSystemSpecs();
            const detected = detectPlatform(specs.osDistro);
            if (detected !== "unknown") platform = detected;
        } catch {}
    }
};
