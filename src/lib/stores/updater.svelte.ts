import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";
import { appPerformUpdate, appUpdateEnvironment, type UpdateEnvironment } from "$lib/api/updater";
import { settings } from "$lib/stores/settings.svelte";
import { toast } from "$lib/stores/toasts.svelte";
import { resolveUpdateAssetUrl, type UpdateAsset } from "$lib/utils/updaterAssets";
import { isReleaseUpdate, RELEASE_REVISION, releaseIdentity } from "$lib/utils/updateVersion";

interface UpdateProgressPayload {
	percent: number;
	transferred: number;
	total: number;
	status: string;
}

let showModal = $state(false);
let currentVersion = $state("");
let currentRevision = $state(RELEASE_REVISION);
let latestVersion = $state("");
let latestRevision = $state(0);
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


type ReleaseEntry = {
	tag_name?: string;
	name?: string;
	body?: string;
	html_url?: string;
	draft?: boolean;
	prerelease?: boolean;
	published_at?: string;
	assets?: UpdateAsset[];
};

function versionWeight(tag: string) {
	const identity = releaseIdentity(tag);
	const parts = String(identity.version).split(/[^0-9]+/).filter(Boolean).map((value) => Number(value));
	const revision = Number(identity.revision) || 0;
	return { major: parts[0] || 0, minor: parts[1] || 0, patch: parts[2] || 0, build: parts[3] || 0, revision };
}

function isNewer(candidate: string, current: string) {
	const first = versionWeight(candidate);
	const second = versionWeight(current);
	const order: (keyof typeof first)[] = ["major", "minor", "patch", "build", "revision"];
	for (const key of order) {
		if (first[key] !== second[key]) return first[key] > second[key];
	}
	return false;
}

async function fetchReleases(): Promise<ReleaseEntry[]> {
	const endpoints = [
		"https://api.github.com/repos/predabr/luxmc/releases/latest",
		"https://api.github.com/repos/predabr/luxmc/releases?per_page=100"
	];
	for (const endpoint of endpoints) {
		try {
			const res = await fetch(endpoint, { cache: "no-store", signal: AbortSignal.timeout(12000) });
			if (!res.ok) continue;
			const data = await res.json();
			const list: ReleaseEntry[] = Array.isArray(data) ? data : [data];
			if (list.length) return list;
		} catch { /* tenta o proximo endpoint */ }
	}
	return [];
}

async function newestRelease(channel: "stable" | "beta"): Promise<ReleaseEntry | null> {
	const list = (await fetchReleases()).filter((entry) => entry && !entry.draft && (channel === "beta" || !entry.prerelease) && entry.tag_name);
	if (!list.length) return null;
	return list.reduce((best, entry) => (isNewer(entry.tag_name!, best.tag_name!) ? entry : best), list[0]);
}

export const updaterStore = {
	get showModal() { return showModal; },
	set showModal(v: boolean) { showModal = v; },
	get currentVersion() { return currentVersion; },
    get currentRevision() { return currentRevision; },
    get latestRevision() { return latestRevision; },
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
		return Boolean(latestVersion && currentVersion && isReleaseUpdate(currentVersion, currentRevision, latestVersion, latestRevision));
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
            try {
                const installed = await appUpdateEnvironment();
                environment = installed.mode;
                currentRevision = Number.isSafeInteger(installed.revision) && installed.revision! >= 0 ? installed.revision! : RELEASE_REVISION;
            } catch { environment = "manual"; currentRevision = RELEASE_REVISION; }
			downloadUrl = "";
			latestRevision = 0;
			lastChecked = new Date().toISOString();

			const channel = settings.value.releaseChannel === "beta" ? "beta" : "stable";
			const release = await newestRelease(channel);
			if (!release) throw new Error(uiText("ui.d3184c6678fe9d2d"));
			const identity = releaseIdentity(release.tag_name || "");
			latestVersion = identity.version;
			latestRevision = identity.revision;
			releaseUrl = release.html_url || "https://github.com/predabr/luxmc/releases";
			releaseNotes = (release.body || "").trim();
			downloadUrl = resolveUpdateAssetUrl((release.assets || []) as UpdateAsset[], environment);

			const newer = isReleaseUpdate(currentVersion, currentRevision, latestVersion, latestRevision);
			verificationStatus = newer ? "available" : "current";
			if (newer && (interactive || notifiedVersion !== `${latestVersion}:${latestRevision}`)) {
				notifiedVersion = `${latestVersion}:${latestRevision}`;
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
				statusText = uiText(event.payload.percent >= 100 ? "updater.openingInstaller" : event.payload.status.toLowerCase().includes("conectando") ? "updater.connecting" : "updater.downloading");
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
