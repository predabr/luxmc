import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";
import { appPerformUpdate, appUpdateEnvironment, type UpdateEnvironment } from "$lib/api/updater";
import { toast } from "$lib/stores/toasts.svelte";
import { resolveUpdateAssetUrl, type UpdateAsset } from "$lib/utils/updaterAssets";

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
			try { environment = (await appUpdateEnvironment()).mode; } catch { environment = "manual"; }
			downloadUrl = "";
			lastChecked = new Date().toISOString();

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
						
							if (manifest.platforms && manifest.platforms[platformKey] && manifest.platforms[platformKey].url && (environment === "appimage" || environment === "windows" || environment === "macos")) {
								downloadUrl = manifest.platforms[platformKey].url;
						}
						foundUpdate = true;
					}
				}
			} catch {
				foundUpdate = false;
			}

			if (!foundUpdate || !downloadUrl) {
				const res = await fetch("https://api.github.com/repos/predabr/luxmc/releases/latest");
				if (res.ok) {
					const data = await res.json();
					const tag: string = data.tag_name || "";
					latestVersion = tag.replace(/^v/i, "");
					releaseUrl = data.html_url || "https://github.com/predabr/luxmc/releases/latest";
					releaseNotes = data.body || "Atualização de melhorias e performance!";
				downloadUrl = resolveUpdateAssetUrl((data.assets || []) as UpdateAsset[], environment);
					foundUpdate = true;
				}
			}

			const newer = isNewerVersion(currentVersion, latestVersion);
			if (newer) {
				showModal = true;
				toast(`Nova versão v${latestVersion} do Luxmc disponível!`, "info");
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
		updateError = "";
		terminalCommand = "";
		if (!downloadUrl) {
			updateError = "Não encontramos um pacote automático compatível com esta instalação. Use o botão abaixo para baixar o pacote oficial manualmente.";
			statusText = "Atualização manual necessária";
			showModal = true;
			return;
		}

		isUpdating = true;
		progressPercent = 0;
		transferredBytes = 0;
		totalBytes = 0;
		statusText = "Iniciando download...";

		try {
			if (unlistenFn) {
				unlistenFn();
				unlistenFn = null;
			}

			unlistenFn = await listen<UpdateProgressPayload>("update-progress", (event) => {
				progressPercent = event.payload.percent;
				statusText = event.payload.status;
				transferredBytes = event.payload.transferred;
				totalBytes = event.payload.total;
			});

			const outcome = await appPerformUpdate(downloadUrl);
			terminalCommand = outcome.terminalCommand || "";
			if (outcome.action === "system-installer") statusText = "O assistente de privilégios do sistema foi aberto.";
			if (outcome.action === "terminal") statusText = "Copie o comando para concluir a atualização no Terminal.";
		} catch (err: unknown) {
			console.error("Falha ao instalar atualização automaticamente:", err);
			updateError = String(err);
			toast("Não foi possível atualizar automaticamente. Baixe a versão compatível no GitHub.", "warning");
		} finally {
			isUpdating = false;
			if (unlistenFn) {
				unlistenFn();
				unlistenFn = null;
			}
		}
	}
};
