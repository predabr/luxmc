import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";
import { appPerformUpdate, appUpdateEnvironment, type UpdateEnvironment } from "$lib/api/updater";
import { settings } from "$lib/stores/settings.svelte";
import { toast } from "$lib/stores/toasts.svelte";
import { resolveUpdateAssetUrl, type UpdateAsset } from "$lib/utils/updaterAssets";
import { isNewerVersion } from "$lib/utils/updateVersion";

interface UpdateProgressPayload {
	percent: number;
	transferred: number;
	total: number;
	status: string;
}

let showModal = $state(false);
let currentVersion = $state("");
let latestVersion = $state("");
let releaseUrl = $state("");
let releaseNotes = $state("");
let downloadUrl = $state("");
let isChecking = $state(false);
let isUpdating = $state(false);
let progressPercent = $state(0);
let statusText = $state("");
let transferredBytes = $state(0);
let totalBytes = $state(0);
let updateError = $state("");
let terminalCommand = $state("");
let environment = $state<UpdateEnvironment["mode"]>("manual");
let verificationStatus = $state<"unchecked" | "checking" | "current" | "available" | "failed">("unchecked");
let unlistenFn: (() => void) | null = null;
let notifiedVersion = "";

let lastChecked = $state<string | null>(null);

export const updaterStore = {
	get showModal() { return showModal; },
	set showModal(v: boolean) { showModal = v; },
	get currentVersion() { return currentVersion; },
	get latestVersion() { return latestVersion; },
	get releaseUrl() { return releaseUrl; },
	get releaseNotes() { return releaseNotes; },
	get downloadUrl() { return downloadUrl; },
	get isChecking() { return isChecking; },
	get isUpdating() { return isUpdating; },
	get isDownloading() { return isUpdating; },
	get progressPercent() { return progressPercent; },
	get downloadProgress() { return progressPercent; },
	get statusText() { return statusText; },
	get transferredBytes() { return transferredBytes; },
	get totalBytes() { return totalBytes; },
	get updateError() { return updateError; },
	get terminalCommand() { return terminalCommand; },
	get environment() { return environment; },
	get verificationStatus() { return verificationStatus; },
	get lastChecked() { return lastChecked; },
	get newVersion() { return latestVersion; },
	get updateAvailable() {
		return Boolean(latestVersion && currentVersion && isNewerVersion(currentVersion, latestVersion));
	},

	async downloadAndInstall() {
		return this.startUpdate();
	},

	async check(interactive = false) {
		if (isChecking || isUpdating) return;
		isChecking = true;
		verificationStatus = "checking";
		try {
			currentVersion = await getVersion();
			try { environment = (await appUpdateEnvironment()).mode; } catch { environment = "manual"; }
			downloadUrl = "";
			lastChecked = new Date().toISOString();

			const channel = settings.value.releaseChannel === "beta" ? "beta" : "stable";
			let foundUpdate = false;

			if (channel === "stable") {
				try {
					const latestRes = await fetch("https://github.com/predabr/luxmc/releases/latest/download/latest.json", { signal: AbortSignal.timeout(10000) });
					if (latestRes.ok) {
						const manifest = await latestRes.json();
						if (manifest && manifest.version) {
							latestVersion = manifest.version.replace(/^v/i, "");
							releaseNotes = manifest.notes || uiText("ui.985b1a01257311f1");
							releaseUrl = "https://github.com/predabr/luxmc/releases/latest";

							const ua = typeof navigator !== "undefined" ? (navigator.userAgent + " " + (navigator.platform || "")).toLowerCase() : "";
							const platformKey = ua.includes("win") ? "windows-x86_64" : ua.includes("mac") ? (ua.includes("arm") ? "darwin-aarch64" : "darwin-x86_64") : "linux-x86_64";

							if (manifest.platforms && manifest.platforms[platformKey] && manifest.platforms[platformKey].url && (environment === "appimage" || environment === "windows" || environment === "macos")) {
								const candidateUrl: string = manifest.platforms[platformKey].url;
								if (environment === "windows") {
									const name = decodeURIComponent(new URL(candidateUrl).pathname.split("/").at(-1) || "");
									downloadUrl = resolveUpdateAssetUrl([{ name, browser_download_url: candidateUrl, size: 0 }], environment);
								} else downloadUrl = candidateUrl;
							}
							foundUpdate = true;
						}
					}
				} catch {
					foundUpdate = false;
				}
			}

			if (!foundUpdate || !downloadUrl) {
				const endpoint = channel === "stable"
					? "https://api.github.com/repos/predabr/luxmc/releases/latest"
					: "https://api.github.com/repos/predabr/luxmc/releases?per_page=20";
				const res = await fetch(endpoint, { signal: AbortSignal.timeout(10000) });
				if (res.ok) {
					const data = await res.json();
					const release = Array.isArray(data)
						? data.find((entry: { draft?: boolean }) => !entry.draft)
						: data;
					if (release && release.tag_name !== undefined) {
						const tag: string = release.tag_name || "";
						latestVersion = tag.replace(/^v/i, "");
						releaseUrl = release.html_url || "https://github.com/predabr/luxmc/releases";
						releaseNotes = release.body || uiText("ui.4343565e98d68e00");
						downloadUrl = resolveUpdateAssetUrl((release.assets || []) as UpdateAsset[], environment);
						foundUpdate = true;
					}
				}
			}

			if (!foundUpdate) throw new Error(uiText("ui.d3184c6678fe9d2d"));
			const newer = isNewerVersion(currentVersion, latestVersion);
			verificationStatus = newer ? "available" : "current";
			if (newer && (interactive || notifiedVersion !== latestVersion)) {
				notifiedVersion = latestVersion;
				showModal = true;
				toast(uiText("ui.2bf47a3eca3c1022", {arg0: (latestVersion)}), "info");
			} else if (interactive) {
				toast(uiText("ui.4151f71b912ec932", {arg0: (currentVersion)}), "success");
			}
		} catch (e) {
			verificationStatus = "failed";
			console.error(uiText("ui.e633daa42e7b39f3"), e);
			if (interactive) {
				toast(uiText("ui.3f1636229e6f9733"), "error");
			}
		} finally {
			isChecking = false;
		}
	},

	async startUpdate() {
		if (isUpdating) return;
		updateError = "";
		terminalCommand = "";
		if (!downloadUrl) {
			updateError = uiText("ui.989acde78ad89368");
			statusText = uiText("ui.4563864a6d1f5a1d");
			showModal = true;
			return;
		}

		isUpdating = true;
		progressPercent = 0;
		transferredBytes = 0;
		totalBytes = 0;
		statusText = uiText("updater.startingDownload");

		try {
			if (unlistenFn) {
				unlistenFn();
				unlistenFn = null;
			}

			unlistenFn = await listen<UpdateProgressPayload>("update-progress", (event) => {
				progressPercent = Math.max(0, Math.min(100, event.payload.percent));
				statusText = uiText(event.payload.percent >= 100 ? "updater.installing" : event.payload.status.toLowerCase().includes("conectando") ? "updater.connecting" : "updater.downloading");
				transferredBytes = event.payload.transferred;
				totalBytes = event.payload.total;
			});

			const outcome = await appPerformUpdate(downloadUrl);
			terminalCommand = outcome.terminalCommand || "";
			if (outcome.action === "system-installer") statusText = uiText("ui.d9b071937870c80a");
			if (outcome.action === "terminal") statusText = uiText("ui.09caa57f801ff845");
		} catch (err: unknown) {
			console.error(uiText("ui.2f6edfbee0516310"), err);
			updateError = String(err);
			statusText = uiText("ui.e9dc4ad6f1bce299");
			toast(uiText("ui.c201c6ea1e026d7a"), "warning");
		} finally {
			isUpdating = false;
			if (unlistenFn) {
				unlistenFn();
				unlistenFn = null;
			}
		}
	}
};
