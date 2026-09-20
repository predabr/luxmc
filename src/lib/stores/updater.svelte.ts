import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { appPerformUpdate } from "$lib/api/updater";
import { toast } from "$lib/stores/toasts.svelte";

interface GitHubAsset {
	name: string;
	browser_download_url: string;
	size: number;
}

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
let unlistenFn: (() => void) | null = null;

function isNewerVersion(current: string, latest: string): boolean {
	const cleanParts = (v: string) =>
		v.replace(/^v/i, "").split("-")[0].split(".").map((x) => parseInt(x, 10) || 0);
	const currParts = cleanParts(current);
	const latestParts = cleanParts(latest);

	for (let i = 0; i < Math.max(currParts.length, latestParts.length); i++) {
		const c = currParts[i] || 0;
		const l = latestParts[i] || 0;
		if (l > c) return true;
		if (l < c) return false;
	}
	return false;
}

let lastChecked = $state<string | null>(null);

function resolveAssetUrl(assets: GitHubAsset[]): string {
	if (!assets || assets.length === 0) return "";
	const ua = typeof navigator !== "undefined" ? (navigator.userAgent + " " + (navigator.platform || "")).toLowerCase() : "";
	const isWin = ua.includes("win");
	const isMac = ua.includes("mac");

	if (isWin) {
		const exe = assets.find((a) => a.name.toLowerCase().endsWith(".exe"));
		if (exe) return exe.browser_download_url;
		const msi = assets.find((a) => a.name.toLowerCase().endsWith(".msi"));
		if (msi) return msi.browser_download_url;
	} else if (isMac) {
		const dmg = assets.find((a) => a.name.toLowerCase().endsWith(".dmg"));
		if (dmg) return dmg.browser_download_url;
	} else {
		const appImage = assets.find((a) => a.name.toLowerCase().endsWith(".appimage"));
		if (appImage) return appImage.browser_download_url;
		const deb = assets.find((a) => a.name.toLowerCase().endsWith(".deb"));
		if (deb) return deb.browser_download_url;
	}

	const fallback = assets.find((a) =>
		a.name.toLowerCase().endsWith(".appimage") ||
		a.name.toLowerCase().endsWith(".exe") ||
		a.name.toLowerCase().endsWith(".deb")
	);
	return fallback ? fallback.browser_download_url : "";
}

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
	get lastChecked() { return lastChecked; },
	get newVersion() { return latestVersion; },
	get updateAvailable() {
		return Boolean(latestVersion && currentVersion && isNewerVersion(currentVersion, latestVersion));
	},

	async downloadAndInstall() {
		return this.startUpdate();
	},

	async check(interactive = false) {
		if (isChecking) return;
		isChecking = true;
		try {
			currentVersion = await getVersion();
			lastChecked = new Date().toLocaleTimeString();

			let foundUpdate = false;

			try {
				const latestRes = await fetch("https://github.com/predabr/luxmc/releases/latest/download/latest.json");
				if (latestRes.ok) {
					const manifest = await latestRes.json();
					if (manifest && manifest.version) {
						latestVersion = manifest.version.replace(/^v/i, "");
						releaseNotes = manifest.notes || "Atualização oficial de alta performance e correções.";
						releaseUrl = "https://github.com/predabr/luxmc/releases/latest";
						
						const ua = typeof navigator !== "undefined" ? (navigator.userAgent + " " + (navigator.platform || "")).toLowerCase() : "";
						const platformKey = ua.includes("win") ? "windows-x86_64" : ua.includes("mac") ? (ua.includes("arm") ? "darwin-aarch64" : "darwin-x86_64") : "linux-x86_64";
						
						if (manifest.platforms && manifest.platforms[platformKey] && manifest.platforms[platformKey].url) {
							downloadUrl = manifest.platforms[platformKey].url;
						}
						foundUpdate = true;
					}
				}
			} catch {
				foundUpdate = false;
			}

			if (!foundUpdate) {
				const res = await fetch("https://api.github.com/repos/predabr/luxmc/releases/latest");
				if (res.ok) {
					const data = await res.json();
					const tag: string = data.tag_name || "";
					latestVersion = tag.replace(/^v/i, "");
					releaseUrl = data.html_url || "https://github.com/predabr/luxmc/releases/latest";
					releaseNotes = data.body || "Atualização de melhorias e performance!";
					downloadUrl = resolveAssetUrl(data.assets || []);
					foundUpdate = true;
				}
			}

			const newer = isNewerVersion(currentVersion, latestVersion);
			if (newer) {
				if (interactive) {
					showModal = true;
				}
			} else if (interactive) {
				toast(`Você já está na versão mais recente do Luxmc (v${currentVersion})!`, "success");
			}
		} catch (e) {
			console.error("Falha ao checar atualizações:", e);
			if (interactive) {
				toast("Erro de conexão ao verificar atualizações.", "error");
			}
		} finally {
			isChecking = false;
		}
	},

	async startUpdate() {
		if (isUpdating) return;
		if (!downloadUrl) {
			// Fallback if no direct asset found
			await openUrl(releaseUrl || "https://github.com/predabr/luxmc/releases/latest");
			showModal = false;
			return;
		}

		isUpdating = true;
		progressPercent = 0;
		statusText = "Iniciando download...";

		try {
			if (unlistenFn) {
				unlistenFn();
				unlistenFn = null;
			}

			unlistenFn = await listen<UpdateProgressPayload>("update-progress", (event) => {
				progressPercent = event.payload.percent;
				statusText = event.payload.status;
			});

			await appPerformUpdate(downloadUrl);
		} catch (err: unknown) {
			console.error("Falha ao instalar atualização automaticamente:", err);
			toast("Não foi possível atualizar automaticamente. Abrindo página de download...", "warning");
			await openUrl(releaseUrl || "https://github.com/predabr/luxmc/releases/latest");
			showModal = false;
		} finally {
			isUpdating = false;
			if (unlistenFn) {
				unlistenFn();
				unlistenFn = null;
			}
		}
	}
};
