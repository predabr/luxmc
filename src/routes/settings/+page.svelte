<script lang="ts">
import { APP_VERSION } from "$lib/version";
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { getSystemSpecs, appDataDirectory, appOpenDataDirectory } from "$lib/api/system";
	import { recommendMemory } from "$lib/utils/platform";
	import { runtimePlatform } from "$lib/stores/platform.svelte";
	import { onMount } from "svelte";
	import {
		Home,
		Users,
		Globe,
		Palette,
		Coffee,
		Terminal,
		ShieldCheck,
		FolderOpen,
		Check,
		ExternalLink,
		RefreshCw,
		Trash2,
		AlertCircle,
		Download,
		Sparkles,
		HardDrive,
		Layers,
		FileText,
		Database,
        SlidersHorizontal
	} from "lucide-svelte";
	import { storageFullReport, storageClearLogs, storageClearCache, storageDeleteInstance } from "$lib/api/system";
	import { settingsSetConcurrentDownloads } from "$lib/api/settings";
	import type { StorageFullReport, InstanceStorageInfo } from "$lib/api/types";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { settings, type AppSettings } from "$lib/stores/settings.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { updaterStore } from "$lib/stores/updater.svelte";
	import { themeStore, THEMES, ACCENTS } from "$lib/stores/theme.svelte";
	import { setLocale, schedulePersist } from "$lib/stores/persistence.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import MicrosoftLogo from "$lib/components/ui/MicrosoftLogo.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { open } from "@tauri-apps/plugin-dialog";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import { useTranslation, setActiveLocale } from "$lib/i18n/useTranslation.svelte";
	import RenderingSection from "$lib/components/settings/RenderingSection.svelte";
	import { getIconSrc } from "$lib/utils/icons";
	import SettingsExtras from "$lib/components/settings/SettingsExtras.svelte";
    import { profilesUpdate } from "$lib/api/instances";
	import ThemeSection from "$lib/components/settings/ThemeSection.svelte";

	const { t } = useTranslation();

	type SettingsTab =
		| "general"
		| "accounts"
		| "language"
		| "appearance"
		| "java"
		| "commands"
		| "privacy"
		| "runtime";

	let activeTab = $state<SettingsTab>("general");

	let releaseChannel = $state<AppSettings["releaseChannel"]>(settings.value.releaseChannel === "beta" ? "beta" : "stable");
	let concurrentDownloads = $state(settings.value.concurrentDownloads ?? 24);
    function resolveDefaultResolution(value: AppSettings) {
        if (value.startFullscreen) return "fullscreen";
        const dimensions = `${value.defaultResWidth ?? 1920}x${value.defaultResHeight ?? 1080}`;
        return dimensions === "854x480" ? "default" : ["1920x1080", "1280x720"].includes(dimensions) ? dimensions : "custom";
    }
    let gameResolution = $state(resolveDefaultResolution(settings.value));
    let customResWidth = $state(settings.value.defaultResWidth ?? 1920);
    let customResHeight = $state(settings.value.defaultResHeight ?? 1080);
	let discordIntegration = $state(settings.value.discordRpc !== false);
	let launcherAction = $state<"keep_open" | "hide_reopen" | "close">(
		settings.value.launcherActionOnLaunch ?? "hide_reopen"
	);
	let showCloseWarning = $state(settings.value.closeWarningOnGameRunning !== false);
	let performanceMode = $state(settings.value.performanceMode ?? false);

	const currentUsername = $derived(account.value?.username || uiText("ui.33a5fcdce7adb8a8"));
	const isMicrosoft = $derived(
		Boolean(
			account.value?.minecraftToken &&
			!account.value?.id.startsWith("offline_") &&
			!account.value?.id.startsWith("offline-")
		)
	);

	let currentLang = $derived<"pt-BR" | "en" | "es">(settings.value.language === "pt-BR" ? "pt-BR" : settings.value.language === "es" ? "es" : "en");

	let selectedTheme = $state(settings.value.theme || "default-dark");
	let selectedAccent = $state(settings.value.accentTheme || themeStore.accent || "blue");
	let density = $state(settings.value.density || "comfortable");

	let maxRamGb = $state((settings.value.maxRamMb || 4096) / 1024);
	let javaPathInput = $state(settings.value.javaPath || "");
	let jvmArgsInput = $state(settings.value.jvmArgs || "");
	let waylandNative = $state(settings.value.waylandNative ?? false);

	let recommendingRam = $state(false);
    const ramPresets = [2, 4, 6, 8, 10, 12, 16];
    async function applyRecommendedRam() {
        recommendingRam = true;
        try {
            const specs = await getSystemSpecs();
            const memory = recommendMemory(specs.totalRamMb);
            maxRamGb = memory.maxRamMb / 1024;
            settings.patch(memory);
            schedulePersist();
            toast(uiText("settings.refinement.ramApplied", { amount: maxRamGb }), "success");
        } catch (error) { toast(String(error), "error"); }
        finally { recommendingRam = false; }
    }

	function setRamPreset(gb: number) {
		maxRamGb = gb;
		saveJava();
	}

    function setJvmPreset(preset: "automatic" | "g1gc") {
        jvmArgsInput = preset === "g1gc" ? "-XX:+UseG1GC -XX:MaxGCPauseMillis=200" : "";
        saveJava();
    }

	let preLaunchCmd = $state("");
    let savingCommands = $state(false);
    $effect(() => { preLaunchCmd = profiles.active?.preLaunchHook || ""; postExitCmd = profiles.active?.postExitHook || ""; gameWrapper = profiles.active?.useGamemode ? "gamemoderun" : ""; });
    async function saveCommands() {
        const profile = profiles.active;
        if (!profile || savingCommands || appState.isGameRunning) return;
        savingCommands = true;
        try { await profilesUpdate({ id: profile.id, preLaunchHook: preLaunchCmd.trim() || null, postExitHook: postExitCmd.trim() || null, useGamemode: runtimePlatform.isLinux && gameWrapper === "gamemoderun" }); profiles.update(profile.id, { preLaunchHook: preLaunchCmd.trim() || null, postExitHook: postExitCmd.trim() || null, useGamemode: runtimePlatform.isLinux && gameWrapper === "gamemoderun" }); toast(uiText("ui.c0050c10a8091f36") + profile.name, "success"); } catch(error) { toast(String(error), "error"); } finally { savingCommands = false; }
    }
	let postExitCmd = $state("");
	let gameWrapper = $state(settings.value.gamemode ? "gamemoderun" : "");

	let streamerMode = $state(settings.value.streamerMode ?? false);
	let hideDiscordDetails = $state(settings.value.hideDiscordDetails ?? false);
	let anonymousTelemetry = $state(settings.value.anonymousTelemetry ?? true);

	let runtimePath = $state("");

	let syncedSettings = {
		theme: settings.value.theme,
		accentTheme: settings.value.accentTheme,
		density: settings.value.density,
		animations: settings.value.animations,
		startFullscreen: settings.value.startFullscreen,
        defaultResWidth: settings.value.defaultResWidth,
        defaultResHeight: settings.value.defaultResHeight,
		discordRpc: settings.value.discordRpc,
		launcherActionOnLaunch: settings.value.launcherActionOnLaunch,
		releaseChannel: settings.value.releaseChannel,
		concurrentDownloads: settings.value.concurrentDownloads,
		closeWarningOnGameRunning: settings.value.closeWarningOnGameRunning,
		performanceMode: settings.value.performanceMode,
		maxRamMb: settings.value.maxRamMb,
		javaPath: settings.value.javaPath,
		jvmArgs: settings.value.jvmArgs,
		waylandNative: settings.value.waylandNative,
		gamemode: settings.value.gamemode,
		streamerMode: settings.value.streamerMode,
		hideDiscordDetails: settings.value.hideDiscordDetails,
		anonymousTelemetry: settings.value.anonymousTelemetry
	};

	$effect(() => {
		const current = settings.value;
		if (current.theme !== syncedSettings.theme) {
			syncedSettings.theme = current.theme;
			selectedTheme = current.theme;
		}
		if (current.accentTheme !== syncedSettings.accentTheme) {
			syncedSettings.accentTheme = current.accentTheme;
			selectedAccent = current.accentTheme;
		}
		if (current.density !== syncedSettings.density) {
			syncedSettings.density = current.density;
			density = current.density;
		}
		if (current.animations !== syncedSettings.animations) {
			syncedSettings.animations = current.animations;
		}
		if (current.releaseChannel !== syncedSettings.releaseChannel) {
			syncedSettings.releaseChannel = current.releaseChannel;
			releaseChannel = current.releaseChannel === "beta" ? "beta" : "stable";
		}
		if (current.concurrentDownloads !== syncedSettings.concurrentDownloads) {
			syncedSettings.concurrentDownloads = current.concurrentDownloads;
			concurrentDownloads = current.concurrentDownloads ?? 24;
		}
		if (current.closeWarningOnGameRunning !== syncedSettings.closeWarningOnGameRunning) {
			syncedSettings.closeWarningOnGameRunning = current.closeWarningOnGameRunning;
			showCloseWarning = current.closeWarningOnGameRunning !== false;
		}
		if (current.startFullscreen !== syncedSettings.startFullscreen || current.defaultResWidth !== syncedSettings.defaultResWidth || current.defaultResHeight !== syncedSettings.defaultResHeight) {
            syncedSettings.startFullscreen = current.startFullscreen;
            syncedSettings.defaultResWidth = current.defaultResWidth;
            syncedSettings.defaultResHeight = current.defaultResHeight;
            if (gameResolution !== "custom" || current.startFullscreen) gameResolution = resolveDefaultResolution(current);
            customResWidth = current.defaultResWidth ?? 1920;
            customResHeight = current.defaultResHeight ?? 1080;
        }
		if (current.discordRpc !== syncedSettings.discordRpc) {
			syncedSettings.discordRpc = current.discordRpc;
			discordIntegration = current.discordRpc !== false;
		}
		if (current.launcherActionOnLaunch !== syncedSettings.launcherActionOnLaunch) {
			syncedSettings.launcherActionOnLaunch = current.launcherActionOnLaunch;
			launcherAction = current.launcherActionOnLaunch ?? "hide_reopen";
		}
		if (current.performanceMode !== syncedSettings.performanceMode) {
			syncedSettings.performanceMode = current.performanceMode;
			performanceMode = current.performanceMode ?? false;
		}
		if (current.maxRamMb !== syncedSettings.maxRamMb) {
			syncedSettings.maxRamMb = current.maxRamMb;
			maxRamGb = (current.maxRamMb || 4096) / 1024;
		}
		if (current.javaPath !== syncedSettings.javaPath) {
			syncedSettings.javaPath = current.javaPath;
			javaPathInput = current.javaPath || "";
		}
		if (current.jvmArgs !== syncedSettings.jvmArgs) {
			syncedSettings.jvmArgs = current.jvmArgs;
			jvmArgsInput = current.jvmArgs || "";
		}
		if (current.waylandNative !== syncedSettings.waylandNative) {
			syncedSettings.waylandNative = current.waylandNative;
			waylandNative = current.waylandNative ?? false;
		}
		if (current.gamemode !== syncedSettings.gamemode) {
			syncedSettings.gamemode = current.gamemode;
			gameWrapper = current.gamemode ? "gamemoderun" : "";
		}
		if (current.streamerMode !== syncedSettings.streamerMode) {
			syncedSettings.streamerMode = current.streamerMode;
			streamerMode = current.streamerMode ?? false;
		}
		if (current.hideDiscordDetails !== syncedSettings.hideDiscordDetails) {
			syncedSettings.hideDiscordDetails = current.hideDiscordDetails;
			hideDiscordDetails = current.hideDiscordDetails ?? false;
		}
		if (current.anonymousTelemetry !== syncedSettings.anonymousTelemetry) {
			syncedSettings.anonymousTelemetry = current.anonymousTelemetry;
			anonymousTelemetry = current.anonymousTelemetry ?? true;
		}
	});

	onMount(async () => {
		try {
			const dataDir = await appDataDirectory();
			if (dataDir) runtimePath = dataDir;
		} catch {}
	});

    const settingsTabs = $derived([
        { id: "general", label: t("settings.tabs.general"), icon: Home },
        { id: "accounts", label: t("settings.tabs.accounts"), icon: Users },
        { id: "language", label: t("settings.tabs.language"), icon: Globe },
        { id: "appearance", label: t("settings.tabs.appearance"), icon: Palette },
        { id: "java", label: t("settings.tabs.java"), icon: Coffee },
        { id: "commands", label: t("settings.tabs.commands"), icon: Terminal },
        { id: "privacy", label: t("settings.tabs.privacy"), icon: ShieldCheck },
        { id: "runtime", label: t("settings.tabs.runtime"), icon: FolderOpen }
    ]);
    function handleSettingsTabKeys(event: KeyboardEvent) {
        if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
        const tabs = Array.from((event.currentTarget as HTMLElement).querySelectorAll<HTMLButtonElement>("[role='tab']"));
        const index = tabs.indexOf(event.target as HTMLButtonElement);
        if (index < 0) return;
        event.preventDefault();
        const next = event.key === "Home" ? 0 : event.key === "End" ? tabs.length - 1 : (index + (event.key === "ArrowRight" ? 1 : -1) + tabs.length) % tabs.length;
        tabs[next].focus();
        tabs[next].click();
    }
    function updateDefaultResolution() {
        customResWidth = Math.max(640, Math.min(7680, Math.round(Number(customResWidth) || 1920)));
        customResHeight = Math.max(360, Math.min(4320, Math.round(Number(customResHeight) || 1080)));
        settings.patch({ defaultResWidth: customResWidth, defaultResHeight: customResHeight, startFullscreen: false });
        gameResolution = "custom";
        schedulePersist();
    }
	function saveGeneral() {
        const dimensions = /^([0-9]+)x([0-9]+)$/.exec(gameResolution);
        if (dimensions) { customResWidth = Number(dimensions[1]); customResHeight = Number(dimensions[2]); }
        if (gameResolution === "default") { customResWidth = 854; customResHeight = 480; }
		settings.patch({
			discordRpc: discordIntegration,
			launcherActionOnLaunch: launcherAction,
			startFullscreen: gameResolution === "fullscreen",
            defaultResWidth: customResWidth,
            defaultResHeight: customResHeight,
			performanceMode: performanceMode,
			releaseChannel,
			concurrentDownloads,
			closeWarningOnGameRunning: showCloseWarning,
		});
		schedulePersist();
		settingsSetConcurrentDownloads(concurrentDownloads).catch((error) => {
			console.error(uiText("ui.04c442acd84772fd"), error);
		});
	}

	function savePrivacy() {
		settings.patch({
			streamerMode,
			hideDiscordDetails,
			anonymousTelemetry,
		});
		schedulePersist();
	}

	function handleLangChange(lang: "pt-BR" | "en" | "es") {
		setActiveLocale(lang);
		setLocale(lang);
		settings.patch({ language: lang });
		schedulePersist();
		toast(t("settings.langChangedSuccess"), "info");
	}

	function handleThemeChange(tName: AppSettings["theme"]) {
		selectedTheme = tName;
		themeStore.setTheme(tName);
		settings.patch({ theme: tName });
		schedulePersist();
	}

	function handleAccentChange(acc: AppSettings["accentTheme"]) {
		selectedAccent = acc;
		themeStore.setAccent(acc);
		settings.patch({ accentTheme: acc });
		schedulePersist();
	}

	function handleDensityChange(d: AppSettings["density"]) {
		density = d;
		settings.patch({ density: d });
		schedulePersist();
	}

	function saveJava() {
		settings.patch({
			maxRamMb: maxRamGb * 1024,
			javaPath: javaPathInput ? javaPathInput.trim() : undefined,
			jvmArgs: jvmArgsInput ? jvmArgsInput.trim() : undefined,
			waylandNative: waylandNative
		});
		schedulePersist();
		toast(uiText("ui.847c046302f1f7c7"), "success");
	}

	async function browseJavaPath() {
		try {
			const sel = await open({
				multiple: false,
				title: uiText("ui.7161f9f39f03b19c")
			});
			if (sel && typeof sel === "string") {
				javaPathInput = sel;
				saveJava();
			}
		} catch (e) {
			toast(uiText("ui.74073fe81c568fb3") + String(e), "error");
		}
	}

	async function handleClearCache() {
		try {
			const keys = ["luxmc_cache_mods", "luxmc_cache_search", "luxmc_temp_skins"];
			for (const k of keys) localStorage.removeItem(k);
		} catch {}
		toast(t("settings.cacheCleared") || uiText("ui.b1f36d2fc6a6621d"), "success");
	}

	let storageReport = $state<StorageFullReport | null>(null);
	let storageLoading = $state(false);
    let storageError = $state("");
    const storageLabels: Record<string, string> = $derived({ libraries: uiText("settings.catLibraries"), assets: uiText("ui.beb65d096de336b3"), versions: uiText("settings.catVersions"), instances: uiText("instances.title"), mods: "Mods", logs: "Logs", cache: uiText("ui.ca803301db3bf0aa"), downloads: "Downloads", java: "Java do launcher", launcher_data: uiText("ui.2b5c717f30be9c31"), configuration: uiText("ui.76b0fb6ad18939ac"), launcher_cache: "Cache do launcher", application: uiText("ui.4fe101016fd71f92"), external_instances: uiText("ui.b59623f79c0f2cf9") });
	let deletingInstanceId = $state<string | null>(null);
	let confirmingDeleteInstance = $state<InstanceStorageInfo | null>(null);

	function formatBytes(bytes: number): string {
		if (!bytes || bytes <= 0) return "0 B";
		const k = 1024;
		const sizes = ["B", "KB", "MB", "GB", "TB"];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
	}

	async function loadStorageReport() {
		storageLoading = true;
        storageError = "";
		try {
			storageReport = await storageFullReport();
		} catch (e) {
            storageError = String(e);
			toast(uiText("ui.ed80732875192710") + String(e), "error");
		} finally {
			storageLoading = false;
		}
	}

	$effect(() => {
		if (activeTab === "runtime") {
			void loadStorageReport();
		}
	});

	async function handleClearLogs() {
		try {
			const freed = await storageClearLogs();
			toast(uiText("ui.936e9d5fd6166fd7", {arg0: (formatBytes(freed))}), "success");
			void loadStorageReport();
		} catch (e) {
			toast(uiText("ui.74755bfde4f77642") + String(e), "error");
		}
	}

	async function handleClearCacheAction() {
		try {
			const freed = await storageClearCache();
			await handleClearCache();
			toast(uiText("ui.1e23968a10a33a9c", {arg0: (formatBytes(freed))}), "success");
			void loadStorageReport();
		} catch (e) {
			toast(uiText("ui.7e5495197cd64b80") + String(e), "error");
		}
	}

	async function handleDeleteInstance(inst: InstanceStorageInfo) {
		deletingInstanceId = inst.id;
		try {
			await storageDeleteInstance(inst.id);
			profiles.remove(inst.id);
			toast(uiText("ui.bea8996b09fea4ce", {arg0: (inst.name)}), "success");
			confirmingDeleteInstance = null;
			void loadStorageReport();
		} catch (e) {
			toast(uiText("ui.66317b8a4bb467bc") + String(e), "error");
		} finally {
			deletingInstanceId = null;
		}
	}
</script>

<div class="settings-page flex flex-col gap-6 pb-16 max-w-6xl mx-auto w-full">
	
    <header class="flex items-center gap-4">
        <span class="flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl border border-brand-400/20 bg-brand-500/10 text-brand-400"><SlidersHorizontal class="h-6 w-6" /></span>
        <div><p class="page-eyebrow">Luxmc</p><h1 class="mt-1 text-2xl font-semibold tracking-tight text-fg">{uiText("settings.refinement.settingsTitle")}</h1></div>
    </header>
    <div class="surface-glass grid grid-cols-2 gap-2 p-2 sm:grid-cols-4 xl:grid-cols-8" role="tablist" aria-label={uiText("settings.refinement.settingsTitle")} tabindex="-1" onkeydown={handleSettingsTabKeys}>
        {#each settingsTabs as tab}
            <button type="button" role="tab" id={`settings-tab-${tab.id}`} aria-controls="settings-panel" aria-selected={activeTab === tab.id} tabindex={activeTab === tab.id ? 0 : -1} onclick={() => activeTab = tab.id as SettingsTab} class={launcherButton({ variant: activeTab === tab.id ? "primary" : "ghost", class: "h-auto min-h-16 w-full flex-col gap-2 px-3 py-3" })}><tab.icon class="h-5 w-5" /><span>{tab.label}</span></button>
        {/each}
    </div>
    <div id="settings-panel" role="tabpanel" aria-labelledby={`settings-tab-${activeTab}`} class="space-y-6">

	{#if activeTab === "general"}
		<div class="space-y-6">
			<div class="flex items-center justify-between">
				<h2 class="text-2xl font-bold text-fg tracking-tight">{t("settings.general")}</h2>
				<span class="text-xs font-mono font-semibold px-2.5 py-1 rounded-lg bg-fg/5 text-fg/60 border border-fg/10">
					{t("settings.version", { version: APP_VERSION })}
				</span>
			</div>

			<div class="bg-gradient-to-r from-brand-500/10 via-bg-elevated to-blue-500/10 border border-brand-500/20 rounded-3xl p-6 shadow-sm flex flex-col md:flex-row md:items-center justify-between gap-4">
				<div class="space-y-1.5">
					<div class="flex items-center gap-2">
						<span class="text-xs font-bold uppercase tracking-wider text-brand-400 flex items-center gap-1.5">
							<Sparkles class="w-3.5 h-3.5" /> {t("settings.autoUpdaterTitle")}
						</span>
						{#if updaterStore.updateAvailable}
							<span class="text-[10px] font-bold px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 animate-pulse">
								{t("settings.newVersionAvailable")}
							</span>
						{:else if updaterStore.verificationStatus === "current"}
							<span class="text-[10px] font-semibold px-2 py-0.5 rounded-full bg-fg/5 text-fg/60 border border-fg/10">
								{t("settings.systemUpToDate")}
							</span>
						{:else}
							<span class="text-[10px] font-semibold px-2 py-0.5 rounded-full bg-fg/5 text-fg/60 border border-fg/10">
								{updaterStore.verificationStatus === "failed" ? t("settings.updateCheckUnavailable") : updaterStore.isChecking ? t("settings.checking") : t("settings.installedVersionBadge")}
							</span>
						{/if}
					</div>
					<h3 class="text-base font-bold text-fg">
						{#if updaterStore.updateAvailable}
							{t("settings.versionReady", { version: updaterStore.newVersion || APP_VERSION })}
						{:else if updaterStore.verificationStatus === "current"}
							{t("settings.versionLatest", { version: updaterStore.currentVersion || APP_VERSION })}
						{:else}
							{t("settings.versionInstalled", { version: updaterStore.currentVersion || APP_VERSION })}
						{/if}
					</h3>
					<p class="text-xs text-fg/50">
						{#if updaterStore.lastChecked}
							{t("settings.lastCheck", { time: new Date(updaterStore.lastChecked).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) })}
						{:else}
							{t("settings.autoCheckDesc")}
						{/if}
					</p>
				</div>

				<div class="flex items-center gap-3 shrink-0">
					{#if updaterStore.updateAvailable}
						<button
							type="button"
							onclick={() => updaterStore.showModal = true}
							disabled={updaterStore.isDownloading}
							class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-2" })}
						>
							{#if updaterStore.isDownloading}
								<RefreshCw class="w-4 h-4 animate-spin" /> {t("settings.downloadingProgress", { percent: updaterStore.downloadProgress })}
							{:else}
								<Download class="w-4 h-4" /> {t("settings.updateNow")}
							{/if}
						</button>
					{:else}
						<button
							type="button"
							onclick={() => updaterStore.check(true)}
							disabled={updaterStore.isChecking}
							class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2" })}
						>
							<RefreshCw class="w-4 h-4 {updaterStore.isChecking ? 'animate-spin text-brand-400' : 'text-fg/60'}" />
							{updaterStore.isChecking ? t("settings.checking") : t("settings.checkUpdates")}
						</button>
					{/if}
				</div>
			</div>

			<div class="bg-bg-elevated border border-fg/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.releaseChannel")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.releaseChannelDesc")}</p>
					</div>
					<select
						bind:value={releaseChannel}
                        aria-label={t("settings.releaseChannel")}
						onchange={saveGeneral}
						class="bg-bg-elevated border border-fg/10 rounded-xl px-4 py-2 text-xs font-semibold text-fg outline-none focus:border-blue-500 cursor-pointer min-w-[140px]"
					>
						<option value="stable">{uiText("ui.90ee305714d71033")}</option>
						<option value="beta">{uiText("ui.703390318bd55aef")}</option>
					</select>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.concurrentDownloads")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.concurrentDownloadsDesc")}</p>
					</div>
					<select
						bind:value={concurrentDownloads}
                        aria-label={t("settings.concurrentDownloads")}
						onchange={saveGeneral}
						class="bg-bg-elevated border border-fg/10 rounded-xl px-4 py-2 text-xs font-semibold text-fg outline-none focus:border-blue-500 cursor-pointer min-w-[100px]"
					>
						<option value={2}>2</option>
						<option value={4}>4</option>
						<option value={8}>8</option>
						<option value={16}>16</option>
						<option value={24}>24</option>
					</select>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.gameResolution")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.gameResolutionDesc")}</p>
					</div>
					<select
						bind:value={gameResolution}
                        aria-label={t("settings.gameResolution")}
						onchange={saveGeneral}
						class="bg-bg-elevated border border-fg/10 rounded-xl px-4 py-2 text-xs font-semibold text-fg outline-none focus:border-blue-500 cursor-pointer min-w-[140px]"
					>
						<option value="default">{t("settings.resolutionDefault")}</option>
						<option value="1920x1080">{uiText("ui.b885a510fc820652")}</option>
						<option value="1280x720">{uiText("ui.1c3b80a3b75d7074")}</option>
						<option value="fullscreen">{t("settings.resolutionFullscreen")}</option>
                        <option value="custom">{uiText("settings.refinement.customResolution")}</option>
					</select>
				</div>

                {#if gameResolution === "custom"}
                    <div class="grid grid-cols-1 gap-4 py-5 sm:grid-cols-2">
                        <label class="space-y-2 text-sm text-fg"><span>{uiText("settings.refinement.resolutionWidth")}</span><input type="number" min="640" max="7680" step="1" bind:value={customResWidth} onchange={updateDefaultResolution} class="w-full rounded-xl border border-border bg-bg-elevated p-3 text-fg" /></label>
                        <label class="space-y-2 text-sm text-fg"><span>{uiText("settings.refinement.resolutionHeight")}</span><input type="number" min="360" max="4320" step="1" bind:value={customResHeight} onchange={updateDefaultResolution} class="w-full rounded-xl border border-border bg-bg-elevated p-3 text-fg" /></label>
                    </div>
                {/if}
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<div class="flex items-center gap-2">
							<h3 class="text-sm font-bold text-fg">{t("settings.discordRpcTitle")}</h3>
							<span class="text-[10px] font-bold px-1.5 py-0.5 rounded bg-blue-500/15 text-blue-400">{uiText("app.version")}</span>
						</div>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.discordRpcDesc")}</p>
					</div>
					<button
						type="button"
						role="switch"
						aria-checked={discordIntegration}
						aria-label={uiText("ui.d2c28d6a2e6a0a47")}
						onclick={() => { discordIntegration = !discordIntegration; saveGeneral(); }}
						class="w-12 h-6 rounded-full transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-200 relative cursor-pointer border {discordIntegration ? 'bg-brand-500 border-brand-400 shadow-md shadow-brand-500/30' : 'bg-fg/15 border-fg/15 hover:bg-fg/25 backdrop-blur-md'}"
					>
						<span class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white transition-transform duration-200 shadow-md {discordIntegration ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.launcherActionTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.launcherActionDesc")}</p>
					</div>
					<select
						bind:value={launcherAction}
                        aria-label={t("settings.launcherActionTitle")}
						onchange={saveGeneral}
						class="bg-bg-elevated border border-fg/10 rounded-xl px-4 py-2 text-xs font-semibold text-fg outline-none focus:border-brand-500 cursor-pointer min-w-[150px]"
					>
						<option value="keep_open">{t("settings.launcherActionNone")}</option>
						<option value="hide_reopen">{t("settings.launcherActionHide")}</option>
						<option value="close">{t("settings.launcherActionClose")}</option>
					</select>
				</div>

				<div class="flex items-center justify-between gap-6 py-5">
                    <div class="max-w-xl space-y-1"><h3 class="text-sm font-bold text-fg">{uiText("settings.closeToTrayTitle")}</h3><p class="text-xs leading-relaxed text-fg/60">{uiText("settings.closeToTrayDesc")}</p></div>
                    <button type="button" role="switch" aria-checked={settings.value.closeToTray !== false} aria-label={uiText("settings.closeToTrayTitle")} onclick={() => settings.patch({ closeToTray: settings.value.closeToTray === false })} class="relative h-6 w-12 shrink-0 rounded-full border transition-colors {settings.value.closeToTray !== false ? 'border-brand-400 bg-brand-500' : 'border-fg/15 bg-fg/15'}"><span class="absolute left-0.5 top-0.5 h-5 w-5 rounded-full bg-fg transition-transform {settings.value.closeToTray !== false ? 'translate-x-6' : ''}"></span></button>
                </div>

                <div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.closeWarningTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.closeWarningDesc")}</p>
					</div>
					<button
						type="button"
						role="switch"
						aria-checked={showCloseWarning}
						aria-label={uiText("ui.879656dc191a6a26")}
						onclick={() => { showCloseWarning = !showCloseWarning; saveGeneral(); }}
						class="w-12 h-6 rounded-full transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-200 relative cursor-pointer border {showCloseWarning ? 'bg-brand-500 border-brand-400 shadow-md shadow-brand-500/30' : 'bg-fg/15 border-fg/15 hover:bg-fg/25 backdrop-blur-md'}"
					>
						<span class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white transition-transform duration-200 shadow-md {showCloseWarning ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.perfModeTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.perfModeDesc")}</p>
					</div>
					<button
						type="button"
						role="switch"
						aria-checked={performanceMode}
						aria-label={uiText("ui.64b61bb4b7584561")}
						onclick={() => { performanceMode = !performanceMode; appState.performanceMode = performanceMode; saveGeneral(); }}
						class="w-12 h-6 rounded-full transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-200 relative cursor-pointer border {performanceMode ? 'bg-emerald-500 border-emerald-400 shadow-md shadow-emerald-500/30' : 'bg-fg/15 border-fg/15 hover:bg-fg/25 backdrop-blur-md'}"
					>
						<span class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white transition-transform duration-200 shadow-md {performanceMode ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

			</div>
            <RenderingSection />
		</div>

	{:else if activeTab === "accounts"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-fg tracking-tight">{t("settings.accountsTitle")}</h2>

			<div class="bg-bg-elevated border border-fg/5 rounded-3xl p-6 shadow-sm space-y-6">
				<div class="flex items-center justify-between p-4 rounded-2xl bg-bg-elevated border border-fg/5">
					<div class="flex items-center gap-4">
						<div class="w-12 h-12 rounded-2xl bg-bg-overlay/40 border border-fg/10 overflow-hidden flex items-center justify-center">
							<img loading="lazy" decoding="async"
								src={activeSkinStore.current.avatarUrl || account.value?.avatarUrl || (account.value?.uuid ? "https://mc-heads.net/avatar/" + account.value.uuid + "/100" : "/logo.png")}
								alt={uiText("ui.ca8e826d9c2ec401")}
								class="w-full h-full object-cover"
								onerror={(e) => {
									const img = e.currentTarget as HTMLImageElement;
									const fallback = account.value?.username ? `https://mc-heads.net/avatar/${account.value.username}/100` : "/logo.png";
									if (img.src !== fallback) img.src = fallback;
									else img.src = "/logo.png";
								}}
							/>
						</div>
						<div>
							<div class="flex items-center gap-2">
								<h3 class="text-sm font-bold text-fg">{currentUsername}</h3>
								{#if isMicrosoft}
									<span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-emerald-500/10 border border-emerald-500/25 text-[10px] font-bold text-emerald-400">
										<MicrosoftLogo size={11} />
										Microsoft
									</span>
								{/if}
							</div>
							<p class="text-xs text-fg/50 flex items-center gap-1.5 mt-0.5">
								{#if isMicrosoft}
									<MicrosoftLogo size={12} />
									<span>{t("settings.msAccountOnline")}</span>
								{:else}
									<span>{t("settings.offlineAccount")}</span>
								{/if}
							</p>
						</div>
					</div>

					<a
						href="/"
						class={launcherButton({ variant: "secondary" })}
					>
						{t("settings.switchAccount")}
					</a>
				</div>

				<div class="p-4 rounded-2xl bg-blue-500/10 border border-blue-500/20 text-xs text-blue-300 leading-relaxed space-y-1.5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
					<div class="space-y-1">
						<p class="font-bold">{t("settings.webPortalManagement")}</p>
						<p class="text-fg/60">{t("settings.webPortalManagementDesc")}</p>
					</div>
					<button
						type="button"
						onclick={() => openUrl("https://luxmc-r92.pages.dev")}
						class={launcherButton({ variant: "primary", size: "sm", class: "shrink-0 flex items-center gap-2" })}
					>
						<span>{t("settings.openWebPortal")}</span>
						<ExternalLink class="w-3.5 h-3.5" />
					</button>
				</div>
			</div>
		</div>

	{:else if activeTab === "language"}
		<div class="space-y-6">
			<div>
				<h2 class="text-2xl font-bold text-fg tracking-tight">{t("settings.languageTitle")}</h2>
				<p class="text-xs text-fg/50 mt-1">{t("settings.languageDesc")}</p>
			</div>

			<div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
				<button
					type="button"
					onclick={() => handleLangChange("pt-BR")}
					class="p-5 rounded-2xl border flex items-center justify-between gap-4 transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer active:scale-[0.98] text-left {currentLang === 'pt-BR' ? 'border-brand-500 bg-bg-subtle ring-2 ring-brand-500/30 shadow-lg' : 'border-fg/5 bg-bg-elevated hover:border-fg/20'}"
				>
					<div class="flex items-center gap-3.5">
						<span class="text-2xl">🇧🇷</span>
						<div>
							<div class="text-sm font-bold text-fg">Português (Brasil)</div>
							<div class="text-[11px] text-fg/40">{uiText("ui.40e3de31827f82c2")}</div>
						</div>
					</div>
					{#if currentLang === "pt-BR"}
						<div class="w-6 h-6 rounded-full bg-brand-500 text-brand-foreground flex items-center justify-center shrink-0">
							<Check class="w-4 h-4 stroke-[3]" />
						</div>
					{/if}
				</button>

				<button
					type="button"
					onclick={() => handleLangChange("en")}
					class="p-5 rounded-2xl border flex items-center justify-between gap-4 transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer active:scale-[0.98] text-left {currentLang === 'en' ? 'border-brand-500 bg-bg-subtle ring-2 ring-brand-500/30 shadow-lg' : 'border-fg/5 bg-bg-elevated hover:border-fg/20'}"
				>
					<div class="flex items-center gap-3.5">
						<span class="text-2xl">🇺🇸</span>
						<div>
							<div class="text-sm font-bold text-fg">{uiText("settings.languageEnglish")}</div>
							<div class="text-[11px] text-fg/40">{uiText("ui.e0c91282ad522a2b")}</div>
						</div>
					</div>
					{#if currentLang === "en"}
						<div class="w-6 h-6 rounded-full bg-brand-500 text-brand-foreground flex items-center justify-center shrink-0">
							<Check class="w-4 h-4 stroke-[3]" />
						</div>
					{/if}
				</button>

				<button
					type="button"
					onclick={() => handleLangChange("es")}
					class="p-5 rounded-2xl border flex items-center justify-between gap-4 transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer active:scale-[0.98] text-left {currentLang === 'es' ? 'border-brand-500 bg-bg-subtle ring-2 ring-brand-500/30 shadow-lg' : 'border-fg/5 bg-bg-elevated hover:border-fg/20'}"
				>
					<div class="flex items-center gap-3.5">
						<span class="text-2xl">🇪🇸</span>
						<div>
							<div class="text-sm font-bold text-fg">{uiText("ui.94b382b61b9dde0f")}</div>
							<div class="text-[11px] text-fg/40">{uiText("ui.54349b39698bfa7f")}</div>
						</div>
					</div>
					{#if currentLang === "es"}
						<div class="w-6 h-6 rounded-full bg-brand-500 text-brand-foreground flex items-center justify-center shrink-0">
							<Check class="w-4 h-4 stroke-[3]" />
						</div>
					{/if}
				</button>
			</div>
		</div>

	{:else if activeTab === "appearance"}
		<div class="space-y-6">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2.5">
					<h2 class="text-2xl font-bold text-fg tracking-tight">{t("settings.tabs.appearance")}</h2>
				</div>
				<span class="text-xs text-fg/50">{t("settings.accentColorHint")}</span>
			</div>

			<div class="bg-bg-elevated border border-fg/5 rounded-3xl p-6 shadow-sm">
				<ThemeSection onSave={() => schedulePersist()} />
			</div>
		</div>

	{:else if activeTab === "java"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-fg tracking-tight">{t("settings.tabs.java")}</h2>

			<div class="bg-bg-elevated border border-fg/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.javaPath")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.javaPathHint")}</p>
					</div>
					<div class="flex items-center gap-2">
						<input
							type="text"
							placeholder={t("settings.javaPathAuto")}
							bind:value={javaPathInput}
							onchange={saveJava}
							class="bg-bg-elevated border border-fg/10 rounded-xl px-3 py-2 text-xs text-fg font-mono placeholder:text-fg/30 outline-none w-56"
						/>
						<button
							type="button"
							onclick={browseJavaPath}
							class={launcherButton({ variant: "secondary", size: "sm", class: "" })}
						>
							{t("settings.browse")}
						</button>
					</div>
				</div>

				<div class="py-5 space-y-3">
					<div class="flex items-center justify-between gap-6">
						<div class="space-y-1 max-w-xl">
							<h3 class="text-sm font-bold text-fg">{t("settings.ramAllocation")}</h3>
							<p class="text-xs text-fg/50 leading-relaxed">{t("settings.ramAllocationDesc", { current: maxRamGb })}</p>
						</div>
						<div class="flex items-center gap-3">
							<input
								type="range"
								min={1}
								max={16}
								step={0.5}
								bind:value={maxRamGb}
								onchange={saveJava}
								class="w-36 accent-blue-500 cursor-pointer"
							/>
							<span class="text-xs font-mono font-bold text-fg bg-bg-elevated px-3 py-1.5 rounded-xl border border-fg/10 min-w-[70px] text-center">
								{maxRamGb} GB
							</span>
						</div>
					</div>
					<div class="flex flex-wrap items-center gap-2 pt-1">
						<span class="text-[11px] font-semibold text-fg/40 mr-1">{t("settings.ramPresets")}:</span>
						{#each ramPresets as preset}
							<button
								type="button"
								onclick={() => setRamPreset(preset)}
								class={launcherButton({ variant: maxRamGb === preset ? "primary" : "secondary", size: "sm" })}
							>
								{preset}G
							</button>
						{/each}
						<button
							type="button"
							onclick={applyRecommendedRam}
                            disabled={recommendingRam}
							class={launcherButton({ variant: "secondary", size: "sm" })}
						>
							{uiText(recommendingRam ? "settings.checking" : "settings.refinement.recommendRam")}
						</button>
					</div>
				</div>

				<div class="py-5 space-y-3">
					<div class="flex items-center justify-between gap-6">
						<div class="space-y-1 max-w-xl">
							<h3 class="text-sm font-bold text-fg">{t("settings.jvmFlags")}</h3>
							<p class="text-xs text-fg/50 leading-relaxed">{t("settings.jvmFlagsDesc")}</p>
						</div>
						<input
							type="text"
							bind:value={jvmArgsInput}
							aria-label={t("settings.jvmFlags")}
							onchange={saveJava}
							class="bg-bg-elevated border border-fg/10 rounded-xl px-3 py-2 text-xs text-fg font-mono placeholder:text-fg/30 outline-none w-80 focus:border-blue-500"
						/>
					</div>
					<div class="flex flex-wrap items-center gap-2 pt-1">
                        <span class="text-[11px] font-semibold text-fg-muted mr-1">{uiText("ui.1d096828d28c5cc3")}</span>
                        <button type="button" onclick={() => setJvmPreset('automatic')} class={launcherButton({ variant: jvmArgsInput ? "secondary" : "primary", size: "sm" })}>{uiText("settings.refinement.automaticJvm")}</button>
                        <button type="button" onclick={() => setJvmPreset('g1gc')} class={launcherButton({ variant: jvmArgsInput === '-XX:+UseG1GC -XX:MaxGCPauseMillis=200' ? "primary" : "secondary", size: "sm" })}>G1GC</button>
                        <p class="w-full pt-2 text-xs leading-relaxed text-fg-muted">{uiText("settings.refinement.jvmPresetsDesc")}</p>

					</div>
				</div>

				{#if runtimePlatform.isLinux}
<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.waylandTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.waylandDesc")}</p>
					</div>
					<button
						type="button"
						role="switch"
						aria-checked={waylandNative}
						aria-label={uiText("ui.e3793b5f9778ce4c")}
						onclick={() => { waylandNative = !waylandNative; saveJava(); }}
						class="w-12 h-6 rounded-full transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-200 relative cursor-pointer border {waylandNative ? 'bg-brand-500 border-brand-400 shadow-md shadow-brand-500/30' : 'bg-fg/15 border-fg/15 hover:bg-fg/25 backdrop-blur-md'}"
					>
						<span class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white transition-transform duration-200 shadow-md {waylandNative ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

				{/if}


			</div>
		</div>

	{:else if activeTab === "commands"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-fg tracking-tight">{t("settings.tabs.commands")}</h2>
            <p class="text-sm text-fg-muted">{uiText("ui.cb504267d059e7fa")} {profiles.active?.name || uiText("ui.64a557b3767a7f20")}{uiText("ui.a913800b3da80d80")}</p>
            <button class={launcherButton({ variant: "primary" })} disabled={!profiles.active || savingCommands || appState.isGameRunning} onclick={saveCommands}>{uiText("ui.69c0d3db22b1c35b")}</button>

			<div class="bg-bg-elevated border border-fg/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.preLaunchTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.preLaunchDesc")}</p>
					</div>
					<input
						type="text"
						placeholder={runtimePlatform.isWindows ? "ex: echo Iniciando Minecraft" : "ex: notify-send 'Iniciando Minecraft'"}
						bind:value={preLaunchCmd}
						class="bg-bg-elevated border border-fg/10 rounded-xl px-3 py-2 text-xs text-fg font-mono placeholder:text-fg/30 outline-none w-72"
					/>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.gameWrapperTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.gameWrapperDesc")}</p>
					</div>
					<select bind:value={gameWrapper} disabled={!runtimePlatform.isLinux} class="rounded-xl border border-border bg-bg-elevated p-3 text-sm"><option value="">{uiText("ui.12780577b0b8c5c7")}</option>{#if runtimePlatform.isLinux}<option value="gamemoderun">{uiText("ui.d0fba50ac6075a55")}</option>{/if}</select>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.postExitTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.postExitDesc")}</p>
					</div>
					<input
						type="text"
						placeholder={runtimePlatform.isWindows ? "ex: echo Minecraft encerrado" : "ex: sync"}
						bind:value={postExitCmd}
						class="bg-bg-elevated border border-fg/10 rounded-xl px-3 py-2 text-xs text-fg font-mono placeholder:text-fg/30 outline-none w-72"
					/>
				</div>

			</div>
		</div>

	{:else if activeTab === "privacy"}
		<div class="space-y-6">
			<div>
				<h2 class="text-2xl font-bold text-fg tracking-tight">{t("settings.tabs.privacy")}</h2>
				<p class="text-xs text-fg/50 mt-1">{t("settings.privacyDesc")}</p>
			</div>

			<div class="bg-bg-elevated border border-fg/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<div class="flex items-center gap-2">
							<ShieldCheck class="w-4 h-4 text-brand-400" />
							<h3 class="text-sm font-bold text-fg">{t("settings.streamerModeTitle")}</h3>
						</div>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.streamerModeDesc")}</p>
					</div>
					<button
						type="button"
						role="switch"
						aria-checked={streamerMode}
						aria-label={uiText("ui.a0dfc9fa1730ed71")}
						onclick={() => { streamerMode = !streamerMode; savePrivacy(); }}
						class="w-12 h-6 rounded-full transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-200 relative cursor-pointer border {streamerMode ? 'bg-brand-500 border-brand-400 shadow-md shadow-brand-500/30' : 'bg-fg/15 border-fg/15 hover:bg-fg/25 backdrop-blur-md'}"
					>
						<span class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white transition-transform duration-200 shadow-md {streamerMode ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.discordPrivacyTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.discordPrivacyDesc")}</p>
					</div>
					<button
						type="button"
						role="switch"
						aria-checked={hideDiscordDetails}
						aria-label={uiText("ui.d86d8509da3b38ca")}
						onclick={() => { hideDiscordDetails = !hideDiscordDetails; savePrivacy(); }}
						class="w-12 h-6 rounded-full transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-200 relative cursor-pointer border {hideDiscordDetails ? 'bg-brand-500 border-brand-400 shadow-md shadow-brand-500/30' : 'bg-fg/15 border-fg/15 hover:bg-fg/25 backdrop-blur-md'}"
					>
						<span class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white transition-transform duration-200 shadow-md {hideDiscordDetails ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.telemetryTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.telemetryDesc")}</p>
					</div>
					<button
						type="button"
						role="switch"
						aria-checked={anonymousTelemetry}
						aria-label={uiText("ui.6e81faba288eafe9")}
						onclick={() => { anonymousTelemetry = !anonymousTelemetry; savePrivacy(); }}
						class="w-12 h-6 rounded-full transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-200 relative cursor-pointer border {anonymousTelemetry ? 'bg-brand-500 border-brand-400 shadow-md shadow-brand-500/30' : 'bg-fg/15 border-fg/15 hover:bg-fg/25 backdrop-blur-md'}"
					>
						<span class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white transition-transform duration-200 shadow-md {anonymousTelemetry ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.clearCache")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.clearCacheDesc")}</p>
					</div>
					<button
						type="button"
						onclick={handleClearCache}
						class={launcherButton({ variant: "danger", size: "sm", class: "flex items-center gap-2" })}
					>
						<Trash2 class="w-3.5 h-3.5" />
						<span>{t("settings.clearCache")}</span>
					</button>
				</div>

			</div>
		</div>

	{:else if activeTab === "runtime"}
		<div class="space-y-6">
            {#if storageError}<p role="alert" class="rounded-xl border border-danger/30 bg-danger/10 p-4 text-sm text-danger">{uiText("ui.5211323645b21734")} {storageError}</p>{/if}
            {#if storageReport?.warnings?.length}<p role="status" class="rounded-xl border border-warning/30 bg-warning/10 p-4 text-sm text-warning">{uiText("ui.3a8133c8b1e3bd1f")} {storageReport.warnings.length} {uiText("ui.c86d9d021e1cfe8c")}</p>{/if}
			<div class="flex items-center justify-between">
				<div>
					<h2 class="text-2xl font-bold text-fg tracking-tight">{t("settings.tabs.runtime")}</h2>
					<p class="text-xs text-fg/50 mt-1">{t("settings.storageDesc")}</p>
				</div>
				<button
					type="button"
					onclick={loadStorageReport}
					disabled={storageLoading}
					class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2 disabled:opacity-50" })}
				>
					<RefreshCw class="w-3.5 h-3.5 {storageLoading ? 'animate-spin' : ''}" />
					<span>{t("settings.refresh")}</span>
				</button>
			</div>

			<div class="bg-bg-elevated border border-fg/5 rounded-3xl p-6 shadow-sm flex flex-col md:flex-row md:items-center justify-between gap-6">
				<div class="space-y-1">
					<span class="text-xs font-bold uppercase tracking-wider text-fg/40">{t("settings.totalDiskUsage")}</span>
					<div class="flex items-baseline gap-3">
						<span class="text-3xl font-black text-fg font-mono tracking-tight">
							{storageReport ? formatBytes(storageReport.totalBytes) : storageLoading ? "Calculando…" : uiText("ui.60cf6b61c55f4a3f")}
						</span>
						<span class="text-xs text-fg/50">{t("settings.usedByLuxmc")}</span>
					</div>
				</div>

				<div class="flex flex-wrap items-center gap-3">
					<button
						type="button"
						onclick={handleClearLogs}
						class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2" })}
					>
						<Trash2 class="w-3.5 h-3.5 text-fg/60" />
						<span>{t("settings.clearLogs", { size: formatBytes(storageReport?.logsBytes || 0) })}</span>
					</button>

					<button
						type="button"
						onclick={handleClearCacheAction}
						class={launcherButton({ variant: "danger", size: "sm", class: "flex items-center gap-2" })}
					>
						<Trash2 class="w-3.5 h-3.5 text-red-400" />
						<span>{t("settings.clearCacheSize", { size: formatBytes(storageReport?.cacheBytes || 0) })}</span>
					</button>
				</div>
			</div>

			<div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-3">
				{#each (storageReport?.categories || []) as cat}
					<div class="bg-bg-elevated border border-fg/5 rounded-2xl p-4 shadow-sm flex flex-col justify-between gap-2">
						<div class="flex items-center justify-between">
							<span class="text-xs font-semibold text-fg/60 capitalize">
								{storageLabels[cat.category] || cat.category}
							</span>
							{#if cat.category === "instances"}
								<Layers class="w-3.5 h-3.5 text-brand-400" />
							{:else if cat.category === "logs"}
								<FileText class="w-3.5 h-3.5 text-fg/40" />
							{:else if cat.category === "cache"}
								<Trash2 class="w-3.5 h-3.5 text-red-400/60" />
							{:else}
								<HardDrive class="w-3.5 h-3.5 text-fg/40" />
							{/if}
						</div>
						<div class="text-lg font-bold text-fg font-mono">
							{formatBytes(cat.bytes)}
						</div>
					</div>
				{/each}
			</div>

			<div class="bg-bg-elevated border border-fg/5 rounded-3xl p-6 shadow-sm space-y-4">
				<div class="flex items-center justify-between">
					<div class="space-y-0.5">
						<h3 class="text-sm font-bold text-fg">{uiText("ui.4329f6fb2641e622")}</h3>
						<p class="text-xs text-fg/50">{uiText("ui.e15ab977c040680c")}</p>
					</div>
					<span class="text-xs font-mono font-bold text-fg/50">
						{storageReport?.instances.length || 0} {uiText("ui.0f3119365532fb7c")}
					</span>
				</div>

				{#if !storageReport?.instances || storageReport.instances.length === 0}
					<div class="py-8 text-center text-xs text-fg/40">
						{uiText("ui.0adaa187d69a7cca")}
					</div>
				{:else}
					<div class="divide-y divide-white/5">
							{#each storageReport.instances as inst (inst.id)}
								{@const iconSrc = getIconSrc(inst.icon)}
							<div class="flex items-center justify-between py-3.5 gap-4">
								<div class="flex items-center gap-3.5 min-w-0">
									<div class="w-10 h-10 rounded-xl overflow-hidden bg-fg/5 border border-fg/10 flex items-center justify-center shrink-0">
										{#if iconSrc !== "/grass_block.png"}
											<img loading="lazy" decoding="async" src={iconSrc} alt={inst.name} class="w-full h-full object-cover" onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/grass_block.png'; }} />
										{:else}
											<img loading="lazy" decoding="async" src="/grass_block.png" alt={inst.name} class="w-6 h-6 object-contain [image-rendering:pixelated]" />
										{/if}
									</div>
									<div class="min-w-0">
										<div class="text-sm font-bold text-fg truncate">{inst.name}</div>
										<div class="flex items-center gap-2 text-[11px] text-fg/50 mt-0.5">
											<span class="font-mono">{inst.mcVersion}</span>
											<span>•</span>
											<span class="uppercase font-semibold text-brand-400">{inst.loader}</span>
										</div>
									</div>
								</div>

								<div class="flex items-center gap-4 shrink-0">
									<span class="font-mono text-xs font-bold text-fg bg-fg/5 px-2.5 py-1.5 rounded-lg border border-fg/10">
										{formatBytes(inst.bytes)}
									</span>
									<button
										type="button"
										title={uiText("ui.1f67d13e9ea1c02c")}
										onclick={() => confirmingDeleteInstance = inst}
										class={launcherButton({ variant: "danger", size: "icon", class: "" })}
									>
										<Trash2 class="w-4 h-4" />
									</button>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>

			<div class="bg-bg-elevated border border-fg/5 rounded-3xl px-6 py-6 shadow-sm space-y-4">
				<div class="space-y-1 max-w-xl">
					<h3 class="text-sm font-bold text-fg">{t("settings.runtimeTitle")}</h3>
					<p class="text-xs text-fg/50 leading-relaxed">{uiText("ui.2b7fcd195a237a22")}</p>
				</div>

				<div class="flex items-center gap-3">
					<input
						type="text"
						readonly
						value={runtimePath}
						class="bg-bg-elevated border border-fg/10 rounded-xl px-4 py-2.5 text-xs text-fg font-mono outline-none flex-1"
					/>
					<button
						type="button"
						disabled={!runtimePath}
                        onclick={() => appOpenDataDirectory().catch(error => toast(String(error), "error"))}
						class={launcherButton({ variant: "secondary", size: "sm", class: "shrink-0" })}
					>
						{uiText("screenshots.openFolder")}
					</button>
				</div>
			</div>
		</div>
	{/if}

	{#if confirmingDeleteInstance}
		<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-md">
			<div class="bg-bg-elevated border border-fg/10 rounded-3xl max-w-md w-full p-6 shadow-2xl space-y-5 animate-in fade-in zoom-in-95 duration-200">
				<div class="flex items-center gap-3 text-red-400">
					<div class="w-10 h-10 rounded-2xl bg-red-500/15 border border-red-500/30 flex items-center justify-center shrink-0">
						<AlertCircle class="w-5 h-5" />
					</div>
					<div>
						<h3 class="text-base font-bold text-fg">{uiText("ui.91d84bee8890761b")}</h3>
						<p class="text-xs text-fg/50">{uiText("ui.c7842b815f11bb3e")}</p>
					</div>
				</div>

				<div class="bg-fg/[0.03] border border-fg/5 rounded-2xl p-4 space-y-2 text-xs">
					<div class="flex justify-between text-fg">
						<span class="text-fg/50">{uiText("ui.cb504267d059e7fa")}</span>
						<span class="font-bold">{confirmingDeleteInstance.name}</span>
					</div>
					<div class="flex justify-between text-fg">
						<span class="text-fg/50">{uiText("ui.c4a9358e97dbf5b0")}</span>
						<span class="font-mono">{confirmingDeleteInstance.mcVersion} ({confirmingDeleteInstance.loader})</span>
					</div>
					<div class="flex justify-between text-fg">
						<span class="text-fg/50">{uiText("ui.1f681dc01135f8fe")}</span>
						<span class="font-bold font-mono text-emerald-400">{formatBytes(confirmingDeleteInstance.bytes)}</span>
					</div>
				</div>

				<div class="flex items-center justify-end gap-3 pt-2">
					<button
						type="button"
						onclick={() => confirmingDeleteInstance = null}
						class={launcherButton({ variant: "secondary", size: "sm", class: "" })}
					>
						{uiText("common.cancel")}
					</button>
					<button
						type="button"
						disabled={deletingInstanceId !== null}
						onclick={() => handleDeleteInstance(confirmingDeleteInstance!)}
						class={launcherButton({ variant: "danger", size: "sm", class: "disabled:opacity-50 flex items-center gap-2" })}
					>
						{#if deletingInstanceId}
							<RefreshCw class="w-3.5 h-3.5 animate-spin" />
							<span>{uiText("ui.abba6d98c1f6fc8f")}</span>
						{:else}
							<Trash2 class="w-3.5 h-3.5" />
							<span>{uiText("ui.6930e70bb14980df")}</span>
						{/if}
					</button>
				</div>
			</div>
		</div>
	{/if}

    <SettingsExtras section={activeTab} />
    </div>
</div>
