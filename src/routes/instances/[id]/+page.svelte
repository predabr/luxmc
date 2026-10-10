<script lang="ts">
    import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { settings } from '$lib/stores/settings.svelte';
    import ModpackVersions from "$lib/components/instances/ModpackVersions.svelte";
	import { runtimePlatform } from "$lib/stores/platform.svelte";
	import { pageMotion } from "$lib/actions/pageMotion";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { backOut, quintOut } from "svelte/easing";
    import InstanceHero from "$lib/components/instances/InstanceHero.svelte";
	import { button } from "$lib/components/ui/button";
	import LoaderBadge from "$lib/components/instances/LoaderBadge.svelte";
	import { javaScan } from "$lib/api/java";
	import type { JavaInstallStatus } from "$lib/api/types";
	import { page } from "$app/state";
    import { goto } from "$app/navigation";
	import { fade, scale } from "svelte/transition";
	import { onMount, untrack } from "svelte";
	import {
		ArrowLeft,
		Download,
		Play,
		Settings as SettingsIcon,
		MoreVertical,
		Package,
		Globe2,
		Image,
		Folder,
		Search,
		RefreshCw,
		Plus,
		Layers,
		Sparkles,
		Box,
		Code,
		Check,
		FolderOpen,
		FileText,
		Trash2,
		ChevronRight,
		ArrowUp,
		File,
		Save,
		X,
		Copy,
		Share2,
		Puzzle,
		ToggleLeft,
		ToggleRight,
		Cpu,
		Zap,
		Gauge,
		Activity,
		ZoomIn,
		ZoomOut,
		RotateCcw,
		ShieldCheck,
		ShieldAlert,
		Compass,
		MapPin,
		Sliders,
		Radio,
		Archive,
		ArrowUpCircle,
		AlertCircle,
		Keyboard,
		HardDrive,
		Skull,
		Disc,
		FolderPlus,
		ArrowLeftRight,
		Snowflake,
		Link,
        Wrench
	} from "lucide-svelte";
	import RightSidebar from "$lib/components/layout/RightSidebar.svelte";
	import VirtualList from "$lib/components/ui/VirtualList.svelte";
	import WorldSnapshotsModal from "$lib/components/ui/WorldSnapshotsModal.svelte";
	import InstanceLab from "$lib/components/instances/InstanceLab.svelte";
	import InstanceConfigEditorModal from "$lib/components/ui/InstanceConfigEditorModal.svelte";
	import P2PHostModal from "$lib/components/ui/P2PHostModal.svelte";
	import KeybindEditorModal from "$lib/components/instances/KeybindEditorModal.svelte";
	import ModConflictModal from "$lib/components/instances/ModConflictModal.svelte";
	import ModpackExportModal from "$lib/components/instances/ModpackExportModal.svelte";
	import WorldBackupModal from "$lib/components/instances/WorldBackupModal.svelte";
	import DeathDetectorModal from "$lib/components/instances/DeathDetectorModal.svelte";
	import JukeboxModal from "$lib/components/instances/JukeboxModal.svelte";
	import { profiles, isSafeBannerUrl, type Profile } from "$lib/stores/profiles.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { getFullCapeDataUrl } from "$lib/utils/capeTextures";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { resolveProfileBanner } from "$lib/utils/curatedBanners";
	import { open, save, confirm as confirmDialog } from "@tauri-apps/plugin-dialog";
	import { convertFileSrc } from "@tauri-apps/api/core";
	import { handlePostLaunchActions } from "$lib/utils/launcherLifecycle";
	import {
		launchGame,
		stopGame,
		versionsCheckInstalled,
		versionsDownload,
		instanceFileTree,
        instanceContentIcons,
		instancesScreenshots,
		instancesOpenFolder,
		screenshotsOpenFolder,
		screenshotDelete,
		authDevLogin,
		instanceWorldsList,
		instanceWorldDelete,
		instanceWorldImport,
		instanceModToggle,
		instanceModDelete,
		instanceModAdd,
		instanceModAddBytes,
		instanceModsOpenFolder,
		instancePackAdd,
		instancePackDelete,
		instancePackOpenFolder,
		profilesUpdate,
        profilesDelete,
		discordSetActivity,
		instanceFileRead,
		instanceFileWrite,
		instanceFileDelete,
		p2pGetHostLink,
		instanceRepair,
		instanceRepairModpack,
		instanceExportZip,
		instanceExportShareCode,
		modsCheckUpdates,
		modsUpdate,
		instanceBackupSaves,
		instanceRestoreSaves,
		jvmArgsValidate,
		getSystemSpecs,
		optimizerGetFlags,
		optimizerDetectGpu,
		modsResolveNames,

		type GpuInfo,
		type JvmValidationResult,
		type FileTreeEntry,
		type WorldDetail,
		type HostLinkInfo,
		instanceShieldScan,
		type ShieldScanResult,
		modpackCheckUpdate,
		modpackVersionDiff,
		modpackUpdateAtomic,
		type ModpackUpdateInfo,
		type ModpackVersionDiff,
		doctorCheckInstanceConflicts,
		doctorInstanceReadiness,
		doctorRepairAll,
		type PreLaunchCheckResult,
		listen,
		upnpOpenPort,
		upnpClosePort,
		type UpnpPortMappingResult,
		type ModUpdateItem
	} from "$lib/api";
	import { achievements } from "$lib/stores/achievements.svelte";
	import { playSound } from "$lib/utils/sound";
	import { getIconSrc } from "$lib/utils/icons";

	const instanceId = $derived(page.params.id ?? "");
	const activeProfile = $derived(profiles.list.find(p => p.id === instanceId) ?? null);

	const settingsIcon = $derived(getIconSrc(activeProfile?.icon));

	const heroBanner = $derived(resolveProfileBanner(activeProfile));

    function loadWhenVisible(node: HTMLElement, initial: { id: string; section: DataSection }) {
        let target = initial;
        const observer = new IntersectionObserver(entries => {
            if (entries.some(entry => entry.isIntersecting)) {
                observer.disconnect();
                if (target.id === instanceId) void loadSection(target.section);
            }
        }, { root: node.closest("main"), rootMargin: "400px" });
        observer.observe(node);
        return { update(next: typeof initial) { target = next; observer.disconnect(); observer.observe(node); }, destroy() { observer.disconnect(); } };
    }

    function handleTabKeys(event: KeyboardEvent) {
        if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
        const tabs = Array.from((event.currentTarget as HTMLElement).querySelectorAll<HTMLButtonElement>('[role="tab"]'));
        const index = tabs.indexOf(event.target as HTMLButtonElement);
        if (index < 0) return;
        event.preventDefault();
        const next = event.key === 'Home' ? 0 : event.key === 'End' ? tabs.length - 1 : (index + (event.key === 'ArrowRight' ? 1 : -1) + tabs.length) % tabs.length;
        tabs[next].focus();
        tabs[next].click();
    }

    function goToSection(section: typeof mainTab) {
        mainTab = section;

    }

	let mainTab = $state<"conteudo" | "mundos" | "galeria" | "ficheiros" | "configuracoes">("conteudo");
	let subTab = $state<"mods" | "resourcepacks" | "shaders" | "datapacks">("mods");
	let modsDragOver = $state(false);
	let isImportingDrop = $state(false);
	let modsDragDepth = 0;
	let searchQuery = $state("");
    let debouncedSearch = $state("");
    $effect(() => {
        const query = searchQuery;
        const timer = setTimeout(() => { debouncedSearch = query; }, 150);
        return () => clearTimeout(timer);
    });

	let modpackUpdate = $state<ModpackUpdateInfo | null>(null);
	let modpackDiff = $state<ModpackVersionDiff | null>(null);
	let isCheckingUpdate = $state(false);
	let isUpdatingModpack = $state(false);
	let updateStatusText = $state("");
	let updateProgressPercent = $state(0);

    let deletingInstance = $state(false);
    async function deleteCurrentInstance() {
        const profile = activeProfile;
        if (!profile || deletingInstance || appState.isGameRunning) return;
        deletingInstance = true;
        try {
        const approved = await confirmDialog(uiText("ui.3e8cdb2fdd295340", {arg0: (profile.name)}), { title: uiText("ui.79ef883cba2c401e"), kind: "warning", okLabel: uiText("screenshots.deleteBtn"), cancelLabel: uiText("common.cancel") });
        if (!approved) return;
        await profilesDelete(profile.id); profiles.remove(profile.id); await goto("/instances"); }
        catch (error) { toast(uiText("ui.28ba246943f0d64d", {arg0: (String(error))}), "error"); }
        finally { deletingInstance = false; }
    }

	let showModConflictModal = $state(false);
	let pendingLaunchConflicts = $state<PreLaunchCheckResult | null>(null);
	let showModpackExportModal = $state(false);
	let showWorldBackupModal = $state(false);
	let showKeybindEditorModal = $state(false);
    let toolsExpanded = $state(false);
	let showDeathDetectorModal = $state(false);
	let showJukeboxModal = $state(false);

	function getModBadge(name: string) {
		const clean = name.replace(/[^a-zA-Z0-9]/g, " ").trim();
		const parts = clean.split(/\s+/).filter(Boolean);
		let initials = "";
		if (parts.length >= 2) {
			initials = (parts[0][0] + parts[1][0]).toUpperCase();
		} else if (parts.length === 1) {
			initials = parts[0].slice(0, 2).toUpperCase();
		} else {
			initials = "MD";
		}
		let hash = 0;
		for (let i = 0; i < name.length; i++) {
			hash = name.charCodeAt(i) + ((hash << 5) - hash);
		}
		const gradients = [
			"from-violet-600 via-indigo-600 to-purple-800 text-violet-100 border-violet-400/30",
			"from-emerald-600 via-teal-600 to-cyan-800 text-emerald-100 border-emerald-400/30",
			"from-amber-600 via-orange-600 to-red-800 text-amber-100 border-amber-400/30",
			"from-rose-600 via-pink-600 to-purple-800 text-rose-100 border-rose-400/30",
			"from-blue-600 via-cyan-600 to-sky-800 text-blue-100 border-blue-400/30",
			"from-fuchsia-600 via-pink-600 to-rose-800 text-fuchsia-100 border-fuchsia-400/30"
		];
		const theme = gradients[Math.abs(hash) % gradients.length];
		return { initials, theme };
	}

	const settingsVisible = $derived(!!activeProfile && mainTab === "configuracoes");
    let savingInstanceSettings = $state(false);
	let labExpanded = $state(false);
	let instanceNameInput = $state("Latest Release");
	let instanceRamMb = $state(4096);
    let instanceBanner = $state("");
	let instanceMinRamMb = $state(1024);
	let javaRuntimes = $state<JavaInstallStatus[]>([]);
	let scanningJava = $state(false);
	async function detectJava() {
		scanningJava = true;
		try { javaRuntimes = (await javaScan()).runtimes.filter(runtime => runtime.installed); }
		catch (error) { toast(String(error), "error"); }
		finally { scanningJava = false; }
	}
	function applyJvmPreset(preset: "automatic" | "g1gc") {
        instanceAutoOptimize = preset === "automatic";
        instanceJvmArgs = preset === "g1gc" ? "-XX:+UseG1GC -XX:MaxGCPauseMillis=200" : "";
    }

	let instanceJvmArgs = $state("");
	let instanceLoaderType = $state<string>("vanilla");
	let instanceLoaderVersion = $state<string>("0.15.11");
	let instanceWindowWidth = $state(1280);
	let instanceWindowHeight = $state(720);
	let instanceStartFullscreen = $state(false);
	let instanceJavaPath = $state("");
	let instanceEnableVulkanOpt = $state(false);
	let instanceAutoOptimize = $state(true);
	let instanceUseGameMode = $state(false);
	let instanceUseMangoHud = $state(false);
	let instanceForceDedicatedGpu = $state(false);
	let instanceUseGamescope = $state(false);
	let instanceGamescopeWidth = $state<number | null>(null);
	let instanceGamescopeHeight = $state<number | null>(null);
	let instanceGamescopeFsr = $state(false);
	let instanceForceFullVerification = $state(false);
	let gpuInfo = $state<GpuInfo | null>(null);
	let generatedAikarFlags = $state<string[]>([]);
	let instancePreLaunchHook = $state("");
	let instancePostExitHook = $state("");
	let systemRamMb = $state(8192);

	const ramPresets = $derived.by(() => {
		const total = systemRamMb || 8192;
		const gb = Math.floor(total / 1024);
		const options: { mb: number; label: string; desc: string }[] = [];

		if (gb <= 4) {
			options.push({ mb: 1024, label: uiText("ui.6c31b63a02708634"), desc: uiText("ui.5d61b4a122c009e1") });
			options.push({ mb: 2048, label: uiText("ui.b2dd6247c7a0cd98"), desc: uiText("settings.resolutionDefault") });
			options.push({ mb: 3072, label: uiText("ui.485f66b700108977"), desc: uiText("ui.5fa7218b756e8f6d") });
		} else if (gb <= 8) {
			options.push({ mb: 2048, label: uiText("ui.b2dd6247c7a0cd98"), desc: "Vanilla" });
			options.push({ mb: 4096, label: uiText("ui.64a214401e793659"), desc: uiText("ui.4e3cfaa334cbafb5") });
			options.push({ mb: 6144, label: uiText("ui.ba344e24e41f5b5c"), desc: uiText("ui.b8c4be44934868d3") });
		} else if (gb <= 16) {
			options.push({ mb: 4096, label: uiText("ui.64a214401e793659"), desc: "Vanilla" });
			options.push({ mb: 6144, label: uiText("ui.ba344e24e41f5b5c"), desc: uiText("ui.2dd0dd57667f7e0b") });
			options.push({ mb: 8192, label: uiText("ui.64b29b62d14dd300"), desc: "Modpacks" });
			options.push({ mb: 12288, label: uiText("ui.f2d32ef1e01403f8"), desc: uiText("ui.f8c78cf86fbff9cc") });
		} else {
			options.push({ mb: 4096, label: uiText("ui.64a214401e793659"), desc: uiText("ui.ba68d32e9584a982") });
			options.push({ mb: 6144, label: uiText("ui.ba344e24e41f5b5c"), desc: uiText("ui.4e3cfaa334cbafb5") });
			options.push({ mb: 8192, label: uiText("ui.64b29b62d14dd300"), desc: "Modpacks" });
			options.push({ mb: 12288, label: uiText("ui.f2d32ef1e01403f8"), desc: uiText("ui.f8c78cf86fbff9cc") });
			options.push({ mb: 16384, label: uiText("ui.c854bb56c3dc2210"), desc: uiText("ui.50ad1b4f7002320c") });
		}
		return options;
	});

	let settingsModalSyncedFor = $state<string | null>(null);
	$effect(() => {
		const open = settingsVisible;
		const profileId = activeProfile?.id ?? null;
		if (!open) {
			settingsModalSyncedFor = null;
			return;
		}
		if (settingsModalSyncedFor === profileId) return;
		settingsModalSyncedFor = profileId;
		if (activeProfile) {
			instanceNameInput = activeProfile.name || "Latest Release";
            instanceBanner = activeProfile.banner || "";
			instanceRamMb = activeProfile.ramMb || 4096;
			instanceJvmArgs = activeProfile.jvmArgs || "";
			const minimum = (activeProfile.jvmArgs || "").match(/-Xms(\d+)([mMgG])/);
			instanceMinRamMb = minimum ? Number(minimum[1]) * (minimum[2].toLowerCase() === "g" ? 1024 : 1) : Math.min(1024, activeProfile.ramMb || 4096);
			instanceAutoOptimize = activeProfile.autoOptimize !== false;
			instanceEnableVulkanOpt = activeProfile.useVulkan === true;
			instanceUseGameMode = activeProfile.useGamemode === true;
			instanceUseMangoHud = activeProfile.useMangohud === true;
			instanceForceDedicatedGpu = activeProfile.forceDedicatedGpu === true;
			instanceUseGamescope = activeProfile.useGamescope === true;
			instanceGamescopeWidth = activeProfile.gamescopeWidth ?? null;
			instanceGamescopeHeight = activeProfile.gamescopeHeight ?? null;
			instanceGamescopeFsr = activeProfile.gamescopeFsr === true;
			instanceForceFullVerification = activeProfile.forceFullVerification === true;
		instancePreLaunchHook = activeProfile.preLaunchHook || "";
		instancePostExitHook = activeProfile.postExitHook || "";
			instanceLoaderType = activeProfile.loader || "vanilla";
			instanceLoaderVersion = activeProfile.loaderVersion || "";
			instanceWindowWidth = activeProfile.resolutionW || activeProfile.resolution?.width || 1280;
			instanceWindowHeight = activeProfile.resolutionH || activeProfile.resolution?.height || 720;
			instanceStartFullscreen = activeProfile.fullscreen === true || activeProfile.resolution?.fullscreen === true;
			instanceJavaPath = activeProfile.javaPath || "";
		}
	});

	let aikarTimer: ReturnType<typeof setTimeout> | null = null;
	$effect(() => {
		const ram = instanceRamMb;
		const opt = instanceAutoOptimize;
		if (aikarTimer) clearTimeout(aikarTimer);
		aikarTimer = setTimeout(() => {
			optimizerGetFlags(ram, opt)
				.then(flags => generatedAikarFlags = flags ?? [])
				.catch(() => {});
		}, 300);
		return () => {
			if (aikarTimer) clearTimeout(aikarTimer);
		};
	});

	async function saveInstanceSettings() {
        if (savingInstanceSettings) return;
        const profile = activeProfile;
		if (profile) {
			if (instanceMinRamMb > instanceRamMb || instanceRamMb > systemRamMb) { toast(uiText("ui.064548a1a2e44056"), "error"); return; }
			instanceJvmArgs = instanceJvmArgs.split(/\s+/).filter(value => value && !/^-Xm[sx]/.test(value)).concat(`-Xms${instanceMinRamMb}M`, `-Xmx${instanceRamMb}M`).join(" ");

			const bannerValue = instanceBanner.trim();
			if (bannerValue && !isSafeBannerUrl(bannerValue)) {
				toast(uiText("ui.78acfb25f932a189"), "error");
				return;
			}

            const changes = {
                    id: profile.id,
					name: instanceNameInput,
					ramMb: instanceRamMb,
					jvmArgs: instanceJvmArgs,
					autoOptimize: instanceAutoOptimize,
					useVulkan: instanceEnableVulkanOpt,
					useGamemode: instanceUseGameMode,
					useMangohud: instanceUseMangoHud,
					forceDedicatedGpu: instanceForceDedicatedGpu,
					useGamescope: instanceUseGamescope,
					gamescopeWidth: instanceGamescopeWidth,
					gamescopeHeight: instanceGamescopeHeight,
					gamescopeFsr: instanceGamescopeFsr,
					forceFullVerification: instanceForceFullVerification,
					loader: instanceLoaderType,
					loaderVersion: instanceLoaderVersion || null,
					resolutionW: instanceWindowWidth,
					resolutionH: instanceWindowHeight,
					fullscreen: instanceStartFullscreen,
					javaPath: instanceJavaPath || null,
					preLaunchHook: instancePreLaunchHook,
					postExitHook: instancePostExitHook,
            };
			savingInstanceSettings = true;
            try {
                await profilesUpdate(changes);
                profiles.update(profile.id, {
                    ...changes,
                    loader: changes.loader as Profile["loader"],
                    loaderVersion: changes.loaderVersion ?? undefined,
                    resolution: { width: changes.resolutionW, height: changes.resolutionH, fullscreen: changes.fullscreen },
                    preLaunchHook: changes.preLaunchHook || null,
                    postExitHook: changes.postExitHook || null
                });
				profiles.setBanner(profile.id, bannerValue);
                toast(uiText("ui.3f84cec002ac344d"), "success");
			} catch (e) {
				toast(uiText("ui.6900afcd83738036") + String(e), "error");
				return;
			} finally { savingInstanceSettings = false; }
		}
		settingsModalSyncedFor = null;
	}


	let jvmValidation = $state<JvmValidationResult | null>(null);
	let isRepairing = $state(false);
	let isRepairingAll = $state(false);
	let isBackingUp = $state(false);
	let isExporting = $state(false);

	let jvmValidateTimer: ReturnType<typeof setTimeout> | null = null;
	$effect(() => {
		const args = instanceJvmArgs;
		if (jvmValidateTimer) clearTimeout(jvmValidateTimer);
		if (!args.trim()) {
			jvmValidation = null;
			return;
		}
		jvmValidateTimer = setTimeout(async () => {
			try {
				jvmValidation = await jvmArgsValidate(args);
			} catch {}
		}, 250);
		return () => {
			if (jvmValidateTimer) clearTimeout(jvmValidateTimer);
		};
	});

	async function handleRepairInstance() {
		if (!activeProfile) return;
		isRepairing = true;
		try {
			toast(uiText("ui.5691b2f0233aec20"), "info");
			await instanceRepair(activeProfile.id);
			toast(uiText("ui.bf6fa5ceff7715e1"), "success");
		} catch (e) {
			toast(uiText("ui.2da9423a63e2c0d8") + String(e), "error");
		} finally {
			isRepairing = false;
		}
	}

	async function handleRepairAll() {
		if (!activeProfile || isRepairingAll) return;
		isRepairingAll = true;
		try {
			toast(uiText("ui.9862d364182d7ac8"), "info");
			const outcome = await doctorRepairAll(activeProfile.id);
			const detail = outcome.repairedMods > 0 ? ` ${outcome.repairedMods} mod(s) recuperado(s).` : "";
			toast(uiText("ui.2564886d38d3a6f2", {arg0: (detail)}), "success");
			for (const warning of outcome.warnings) toast(warning, "info");
			await refreshAllData();
		} catch (e) {
			toast(uiText("ui.85465570a78c6a06") + String(e), "error");
		} finally {
			isRepairingAll = false;
		}
	}

	async function handleBackupSaves() {
		if (!activeProfile) return;
		isBackingUp = true;
		try {
			const safeName = activeProfile.name.toLowerCase().replace(/[^a-z0-9]/g, "_");
			const path = await save({
				defaultPath: `${safeName}_saves_backup.zip`,
				filters: [{ name: "Arquivo ZIP", extensions: ["zip"] }]
			});
			if (!path) {
				isBackingUp = false;
				return;
			}
			toast(uiText("ui.dff69f0e92b7de31"), "info");
			await instanceBackupSaves(activeProfile.id, path);
			toast(uiText("ui.bd713228121fc570", {arg0: (path)}), "success");
		} catch (e) {
			toast(uiText("ui.b75153e767c784f6") + String(e), "error");
		} finally {
			isBackingUp = false;
		}
	}

	async function handleExportZip() {
		if (!activeProfile) return;
		isExporting = true;
		try {
			const safeName = activeProfile.name.toLowerCase().replace(/[^a-z0-9]/g, "_");
			const path = await save({
				defaultPath: `${safeName}_export.zip`,
				filters: [{ name: "Arquivo ZIP", extensions: ["zip"] }]
			});
			if (!path) {
				isExporting = false;
				return;
			}
			toast(uiText("ui.13cf2456c0828d9b"), "info");
			await instanceExportZip(activeProfile.id, path);
			toast(uiText("ui.927050bd0e44fcd7", {arg0: (path)}), "success");
		} catch (e) {
			toast(uiText("ui.4028f2f1323faf44") + String(e), "error");
		} finally {
			isExporting = false;
		}
	}

	let generatedShareCode = $state<string | null>(null);
	let showShareCodeModal = $state(false);
	let isGeneratingShareCode = $state(false);

	async function handleExportShareCode() {
		if (!activeProfile) return;
		isGeneratingShareCode = true;
		try {
			const code = await instanceExportShareCode(activeProfile.id);
			generatedShareCode = code;
			showShareCodeModal = true;
			if (navigator?.clipboard) {
				await navigator.clipboard.writeText(code);
			}
			achievements.unlock("share_code");
			playSound("chime");
			toast(uiText("ui.a0e0eb21133c533e", {arg0: (code)}), "success");
		} catch (e) {
			toast(uiText("ui.5c2df8114e58e959") + String(e), "error");
		} finally {
			isGeneratingShareCode = false;
		}
	}

	let isCheckingUpdates = $state(false);
	let availableModUpdates = $state<ModUpdateItem[]>([]);
	let isUpdatingAllMods = $state(false);
	let updatingModProjects = $state<string[]>([]);

	async function handleCheckModUpdates() {
		if (!instanceId || isCheckingUpdates) return;
		const id = instanceId;
		isCheckingUpdates = true;
		toast(uiText("ui.b007b33cf6aea8c4"), "info");
		try {
			const updates = await modsCheckUpdates(id);
			if (id !== instanceId) return;
			availableModUpdates = updates || [];
			if (availableModUpdates.length === 0) {
				toast(uiText("ui.ec926c5c381f50e2"), "success");
			} else {
				playSound("chime");
				toast(uiText("ui.c535dc1fd400aa4d", {arg0: (availableModUpdates.length)}), "info");
			}
		} catch (e) {
			toast(uiText("ui.bf3e6e55e38be20f") + String(e), "error");
		} finally {
			isCheckingUpdates = false;
		}
	}

	async function handleUpdateAllMods() {
		if (!instanceId || availableModUpdates.length === 0 || isUpdatingAllMods) return;
		const id = instanceId;
		const toUpdate = availableModUpdates.filter(update => !isUpdateFrozen(update));
		if (!toUpdate.length) { toast(uiText("ui.3c4b5076f892bd3c"), "info"); return; }
		isUpdatingAllMods = true;
		toast(`Atualizando ${toUpdate.length} mods...`, "info");
		let count = 0;
		for (const u of toUpdate) {
			if (id !== instanceId) break;
			try {
				await modsUpdate(u.projectId, u.latestVersionId, id);
				count++;
				if (id === instanceId) availableModUpdates = availableModUpdates.filter(x => x.projectId !== u.projectId);
			} catch (err) {
				console.error(uiText("ui.3d6be3dde1520cbd"), u.projectId, err);
			}
		}
		isUpdatingAllMods = false;
		if (id !== instanceId) return;
		if (count > 0) {
			playSound("achievement");
			toast(uiText("ui.680c180aa27692cc", {arg0: (count)}), "success");
			await refreshAllData();
		} else {
			toast(uiText("ui.a316761ebd86ecb9"), "warning");
		}
	}

	async function handleUpdateSingleMod(update: ModUpdateItem) {
		if (!instanceId || updatingModProjects.includes(update.projectId) || isUpdatingAllMods) return;
		if (isUpdateFrozen(update)) { toast(uiText("ui.2bc9659ccd079dda"), "info"); return; }
		updatingModProjects = [...updatingModProjects, update.projectId];
		try {
			toast(`Atualizando ${update.projectName || update.projectTitle || 'mod'}...`, "info");
			await modsUpdate(update.projectId, update.latestVersionId, instanceId);
			playSound("chime");
			toast(`Mod atualizado com sucesso!`, "success");
			availableModUpdates = availableModUpdates.filter(x => x.projectId !== update.projectId);
			await refreshAllData();
		} catch (e) {
			toast(uiText("ui.f7262db82ed3f50a") + String(e), "error");
		} finally {
			updatingModProjects = updatingModProjects.filter(id => id !== update.projectId);
		}
	}

	let isRepairingModpack = $state(false);

	async function handleRepairModpack() {
		if (!instanceId) return;
		isRepairingModpack = true;
		try {
			toast(uiText("ui.aa1e8e9534687c4d"), "info");
			const count = await instanceRepairModpack(instanceId);
			if (count > 0) {
				toast(`Modpack reparado! ${count} mod(s) baixado(s).`, "success");
				playSound("achievement");
				await refreshAllData();
			} else {
				toast(uiText("ui.7e4c450d953a94b7"), "success");
			}
		} catch (e) {
			toast(uiText("ui.35c425e04f2b57c8") + String(e), "error");
		} finally {
			isRepairingModpack = false;
		}
	}

	let showHostModal = $state(false);
	let hostLinkInfo = $state<HostLinkInfo | null>(null);
	let customHostPort = $state(25565);
	let isCopiedHostLink = $state(false);
	let isCopiedDirectAddress = $state(false);
	let upnpResult = $state<UpnpPortMappingResult | null>(null);
	let isOpeningUpnp = $state(false);

	async function attemptUpnpOpen() {
		isOpeningUpnp = true;
		try {
			const res = await upnpOpenPort(customHostPort);
			upnpResult = res;
			if (res.success) {
				toast(uiText("ui.8960ce11b29dc29d"), "success");
			}
		} catch (e) {
			console.warn("UPnP automatic opening failed:", e);
		} finally {
			isOpeningUpnp = false;
		}
	}

	async function openHostWorldModal() {
		try {
			hostLinkInfo = await p2pGetHostLink(customHostPort);
			showHostModal = true;
			attemptUpnpOpen();
		} catch (e) {
			toast(uiText("ui.b41df1ca12481eb2") + String(e), "error");
		}
	}

	async function refreshHostLink() {
		try {
			hostLinkInfo = await p2pGetHostLink(customHostPort);
			await attemptUpnpOpen();
			toast(uiText("ui.8da72df805e1a937") + customHostPort, "success");
		} catch (e) {
			toast(uiText("ui.63825fe75c4da989") + String(e), "error");
		}
	}

	function copyHostLink() {
		if (!hostLinkInfo) return;
		navigator.clipboard.writeText(hostLinkInfo.shareLink);
		isCopiedHostLink = true;
		toast(uiText("ui.900c9ce79ac45e7d"), "success");
		setTimeout(() => (isCopiedHostLink = false), 2500);
	}

	function copyDirectAddress() {
		if (!hostLinkInfo) return;
		const addr = upnpResult?.externalIp ? `${upnpResult.externalIp}:${customHostPort}` : hostLinkInfo.directAddress;
		navigator.clipboard.writeText(addr);
		isCopiedDirectAddress = true;
		toast(uiText("ui.54b20bf5531a8a89"), "success");
		setTimeout(() => (isCopiedDirectAddress = false), 2000);
	}

	// Launch & Install state
	let isLaunching = $derived(appState.isLaunching);
	let isInstalled = $state(true);
	let launchStatusText = $derived(appState.launchStatusText);
	let downloadProgressPercent = $state(0);

	// Real Data from File System & Backend
	let worldsList = $state<WorldDetail[]>([]);
    let datapackWorld = $state("");
	let screenshotsList = $state<Array<{ name: string; path: string; modified: string; dataUrl?: string | null; thumbPath?: string | null }>>([]);
	let previewScreenshot = $state<{ name: string; path: string; dataUrl?: string | null } | null>(null);
	let screenshotZoom = $state(1);

	function zoomInScreenshot() {
		screenshotZoom = Math.min(3, +(screenshotZoom + 0.25).toFixed(2));
	}

	function zoomOutScreenshot() {
		screenshotZoom = Math.max(0.5, +(screenshotZoom - 0.25).toFixed(2));
	}

	function resetScreenshotZoom() {
		screenshotZoom = 1;
	}

	async function copyScreenshotImage() {
		if (!previewScreenshot) return;
		try {
			const src = previewScreenshot.dataUrl || convertFileSrc(previewScreenshot.path);
			const res = await fetch(src);
			const blob = await res.blob();
			const pngBlob = blob.type === "image/png" ? blob : new Blob([blob], { type: "image/png" });
			await navigator.clipboard.write([
				new ClipboardItem({ "image/png": pngBlob })
			]);
			toast(uiText("screenshots.copied"), "success");
			playSound("chime");
		} catch {
			try {
				await navigator.clipboard.writeText(previewScreenshot.path);
				toast("Caminho da captura copiado!", "info");
			} catch (e) {
				toast(String(e), "error");
			}
		}
	}

	$effect(() => {
		if (!previewScreenshot) return;
		function onKey(e: KeyboardEvent) {
			if (e.key === "Escape") {
				previewScreenshot = null;
			} else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "c") {
				e.preventDefault();
				copyScreenshotImage();
			} else if (e.key === "+" || e.key === "=") {
				zoomInScreenshot();
			} else if (e.key === "-") {
				zoomOutScreenshot();
			}
		}
		window.addEventListener("keydown", onKey);
		return () => window.removeEventListener("keydown", onKey);
	});
	let fileTree = $state<FileTreeEntry[]>([]);
	let instanceMods = $state<FileTreeEntry[]>([]);
	let resourcePacks = $state<FileTreeEntry[]>([]);
	let shaderPacks = $state<FileTreeEntry[]>([]);
	let dataPacks = $state<FileTreeEntry[]>([]);
	let isLoadingData = $state(false);

	// In-launcher File Manager states
	let fileSubPath = $state("");
	let fileBreadcrumbs = $state<string[]>([]);
	let activeEditorFile = $state<{ path: string; name: string; content: string } | null>(null);
	let isSavingEditor = $state(false);

	let selectedSnapshotWorld = $state<{ name: string; folder: string } | null>(null);
	let showConfigEditor = $state(false);
	let showP2PHost = $state(false);
	let shieldResult = $state<ShieldScanResult | null>(null);
	let isScanningShield = $state(false);

	async function runShieldScan() {
		if (!instanceId || isScanningShield) return;
		isScanningShield = true;
		try {
			shieldResult = await instanceShieldScan(instanceId);
			if (!shieldResult.isClean && shieldResult.threats.length > 0) {
				toast(uiText("ui.29f3471b4915d6e0", {arg0: (shieldResult.threats.length)}), "error");
			}
		} catch {
			// Silent scan: do not show error toast when clean or idle
		} finally {
			isScanningShield = false;
		}
	}

	let hasAutoScannedShield = $state(false);
	$effect(() => {
		if (mainTab === "conteudo" && subTab === "mods" && instanceMods.length > 0 && !shieldResult && !isScanningShield && !hasAutoScannedShield) {
            const timer = setTimeout(() => { hasAutoScannedShield = true; void runShieldScan(); }, 1200);
            return () => clearTimeout(timer);
        }
	});

    const packGroups = $derived([
        { id: 'resourcepacks', type: 'resourcepacks', label: uiText("ui.906d0853ece30a11"), icon: Box, items: resourcePacks },
        { id: 'shaders', type: 'shaderpacks', label: 'Shaders', icon: Sparkles, items: shaderPacks },
        { id: 'datapacks', type: 'datapacks', label: uiText("ui.f3260e4c97557038"), icon: Code, items: dataPacks }
    ] as const);
    const contentCatalogUrl = $derived(`/mods?instance=${encodeURIComponent(instanceId)}&type=mod`);
    const enabledModCount = $derived(instanceMods.filter(mod => !mod.name.endsWith('.disabled')).length);
    const normalizedSearch = $derived(debouncedSearch.trim().toLowerCase());
    const filteredMods = $derived(normalizedSearch ? instanceMods.filter(mod =>
        mod.name.toLowerCase().includes(normalizedSearch) || modRowInfo.get(mod.name)?.displayName.toLowerCase().includes(normalizedSearch)) : instanceMods);
    let isChangingMods = $state(false);

    async function toggleSelectedMods(enabled: boolean) {
        if (isChangingMods) return;
        const id = instanceId;
        const selected = instanceMods.filter(mod => selectedModSet.has(mod.name) && mod.name.endsWith('.disabled') === enabled);
        isChangingMods = true;
        let failed = 0;
        try {
            for (const mod of selected) {
                try { await instanceModToggle(id, mod.name, enabled); }
                catch { failed++; }
            }
            if (id !== instanceId) return;
            selectedModNames = [];
            await loadSection('mods', true);
            toast(failed ? uiText("ui.b610fee7d4ca3c7e", {arg0: (failed)}) : 'Mods selecionados atualizados.', failed ? 'error' : 'success');
        } finally { isChangingMods = false; }
    }

	let selectedModNames = $state<string[]>([]);
	let frozenModNames = $state<string[]>([]);
	let activeMenuMod = $state<string | null>(null);
	let modMenuAbove = $state(false);

	const allModsSelected = $derived(
		filteredMods.length > 0 && filteredMods.every(m => selectedModSet.has(m.name))
	);

	function toggleSelectAllMods() {
		if (allModsSelected) {
			const visible = new Set(filteredMods.map(mod => mod.name));
			selectedModNames = selectedModNames.filter(name => !visible.has(name));
		} else {
			selectedModNames = [...new Set([...selectedModNames, ...filteredMods.map(mod => mod.name)])];
		}
	}

	function toggleModSelection(name: string) {
		if (selectedModNames.includes(name)) {
			selectedModNames = selectedModNames.filter(n => n !== name);
		} else {
			selectedModNames = [...selectedModNames, name];
		}
	}

	function handleToggleFreeze(mod: FileTreeEntry) {
		const name = mod.name.replace(/\.disabled$/, '');
		if (frozenModNames.includes(name)) {
			frozenModNames = frozenModNames.filter(n => n !== name);
			toast(uiText("ui.1fb77f66fa869583", {arg0: (mod.name.replace('.disabled', ''))}), "info");
		} else {
			frozenModNames = [...frozenModNames, name];
			toast(uiText("ui.3ef436739a9288c8", {arg0: (mod.name.replace('.disabled', ''))}), "success");
		}
		try { localStorage.setItem(`luxmc:frozen-mods:${instanceId}`, JSON.stringify(frozenModNames)); }
		catch { toast(uiText("ui.1a50574dacc105fc"), "warning"); }
	}

	function isUpdateFrozen(update: ModUpdateItem) {
		return instanceMods.some(mod => frozenModSet.has(mod.name.replace(/\.disabled$/, '')) && modRowInfo.get(mod.name)?.update?.projectId === update.projectId);
	}

	function handleCopyModLink(mod: FileTreeEntry) {
		const raw = mod.name.replace(/\.disabled$/, '').replace(/\.(jar|zip)$/, '');
		const url = `https://modrinth.com/mods?q=${encodeURIComponent(parseModMeta(mod.name).title)}`;
		if (navigator.clipboard) {
			navigator.clipboard.writeText(url).then(() => {
				toast(uiText("ui.8e2a04648fa7795b"), "success");
			}).catch(() => {
				toast(`Mod: ${raw}`, "info");
			});
		}
	}

	function handleShowModFile(_mod: FileTreeEntry) {
		if (instanceId) {
			instanceModsOpenFolder(instanceId).catch(() => {});
		}
	}

	async function handleSyncMod(_mod: FileTreeEntry) {
		toast("Sincronizando mod...", "info");
		await refreshAllData();
		toast(uiText("ui.1a7c0fe3efaf18ca"), "success");
	}

	function handleSwapVersion(mod: FileTreeEntry) {
		const raw = mod.name.replace(/\.disabled$/, '').replace(/\.(jar|zip)$/, '');
		import("$app/navigation").then(({ goto }) => {
			goto(`${contentCatalogUrl}&search=${encodeURIComponent(raw)}`);
		});
	}

	function parseModMeta(fileName: string) {
		const clean = fileName.replace(/\.disabled$/, '').replace(/\.(jar|zip|mrpack)$/, '');
		const match = clean.match(/^(.*?)[-_+vV]?((\d+\.[\d.]+[a-zA-Z0-9_-]*))$/);
		if (match && match[1] && match[2]) {
			const title = match[1].replace(/[-_]/g, ' ').trim() || clean;
			return {
				title,
				version: match[2],
				author: "Mod Developer"
			};
		}
		return {
			title: clean.replace(/[-_]/g, ' '),
			version: "1.0.0",
			author: "Mod Developer"
		};
	}

	const selectedModSet = $derived(new Set(selectedModNames));
	const frozenModSet = $derived(new Set(frozenModNames));

	function findModUpdate(fileName: string, displayName: string): ModUpdateItem | null {
		const lowerName = displayName.toLowerCase();
		for (const u of availableModUpdates) {
			if (
				(u.fileName &&
					(fileName === u.fileName ||
						fileName === u.fileName + ".disabled" ||
						fileName.startsWith(u.fileName.replace(".jar", "")))) ||
				(u.projectName && lowerName.includes(u.projectName.toLowerCase())) ||
				(u.projectId && lowerName.includes(u.projectId.toLowerCase()))
			) {
				return u;
			}
		}
		return null;
	}

	interface ModRowInfo {
		displayName: string;
		author: string;
		version: string;
		badge: { initials: string; theme: string };
		update: ModUpdateItem | null;
	}

	const modRowInfo = $derived.by(() => {
		const map = new Map<string, ModRowInfo>();
		for (const mod of instanceMods) {
			const meta = parseModMeta(mod.name);
			const rawName = mod.name.replace(".disabled", "").replace(".jar", "");
			const displayName =
				rawName.includes("_") && /^\d+_\d+$/.test(rawName)
					? "Mod #" + rawName.split("_")[0]
					: meta.title;
			map.set(mod.name, {
				displayName,
				author: meta.author,
				version: meta.version,
				badge: getModBadge(displayName),
				update: findModUpdate(mod.name, displayName)
			});
		}
		return map;
	});

    type DataSection = "mods" | "resourcepacks" | "shaders" | "datapacks" | "mundos" | "galeria" | "ficheiros" | "configuracoes";
    const loadedSections = new Set<string>();
    const sectionRequests = new Map<string, Promise<void>>();
    const resolvedMetadata = new Set<string>();
    let dataGeneration = 0;
    let pendingLoads = 0;
    let metadataTimer: ReturnType<typeof setTimeout> | undefined;

    const iconQueue = new Map<string, { name: string; folder: string; id: string; key: string }>();
    const iconRequested = new Set<string>();
    let iconTimer: ReturnType<typeof setTimeout> | undefined;
    let iconBusy = false;
    const iconLookups: Array<{ id: string; folder: string; names: string[]; apply: (icons: Awaited<ReturnType<typeof instanceContentIcons>>) => void }> = [];
    let lookupBusy = false;
    async function flushIconLookups() {
        if (lookupBusy) return;
        lookupBusy = true;
        try {
            await Promise.all(Array.from({ length: 2 }, async () => {
                while (iconLookups.length) {
                    const lookup = iconLookups.shift()!;
                    if (lookup.id !== instanceId) continue;
                    await instanceContentIcons(lookup.id, lookup.folder, lookup.names, true).then(lookup.apply).catch(() => {});
                }
            }));
        } finally { lookupBusy = false; }
    }
    const warmedIcons = new Set<string>();
    function queueContentIcons(entries: FileTreeEntry[], folder: string) {
        for (const entry of entries) {
            if (!entry.iconKey || entry.iconResolved || entry.icon || iconRequested.has(entry.iconKey)) continue;
            iconRequested.add(entry.iconKey);
            iconQueue.set(entry.iconKey, { name: entry.name, folder, id: instanceId, key: entry.iconKey });
        }
        clearTimeout(iconTimer);
        iconTimer = setTimeout(() => void flushContentIcons(), 0);
    }
    function warmIcons(icons: Array<{icon?: string | null}>) {
        if (settings.value.preloadContentIcons === false) return;
        for (const icon of icons) {
            if (!icon.icon || warmedIcons.has(icon.icon)) continue;
            if (warmedIcons.size >= 2000) warmedIcons.delete(warmedIcons.values().next().value!);
            warmedIcons.add(icon.icon);
            const image = new window.Image(); image.decoding = 'async'; image.src = icon.icon;
        }
    }
    async function flushContentIcons() {
        if (iconBusy) return;
        iconBusy = true;
        try {
            await Promise.all(Array.from({ length: 2 }, async () => {
            while (iconQueue.size) {
                const first = iconQueue.values().next().value!;
                const batch = [...iconQueue.values()].filter(entry => entry.id === first.id && entry.folder === first.folder).slice(0, 32);
                for (const entry of batch) iconQueue.delete(entry.key);
                const apply = (icons: Awaited<ReturnType<typeof instanceContentIcons>>) => {
                    if (instanceId !== first.id) return;
                    warmIcons(icons);
                    const byKey = new Map(icons.map(icon => [icon.iconKey, icon]));
                    const update = (entries: FileTreeEntry[]) => entries.map(entry => {
                        const icon = entry.iconKey ? byKey.get(entry.iconKey) : undefined;
                        return icon ? { ...entry, icon: icon.icon, iconResolved: icon.resolved } : entry;
                    });
                    if (first.folder === 'mods') instanceMods = update(instanceMods);
                    else if (first.folder === 'resourcepacks') resourcePacks = update(resourcePacks);
                    else if (first.folder === 'shaderpacks') shaderPacks = update(shaderPacks);
                    else if (first.folder === `saves/${datapackWorld}/datapacks`) dataPacks = update(dataPacks);
                };
                try {
                    const local = await instanceContentIcons(first.id, first.folder, batch.map(entry => entry.name));
                    apply(local);
                    const missing = local.filter(entry => !entry.resolved).map(entry => entry.name);
                    if (missing.length) iconLookups.push({ id: first.id, folder: first.folder, names: missing, apply });
                } catch { for (const entry of batch) iconRequested.delete(entry.key); }
            }
            }));
        } finally { iconBusy = false; void flushIconLookups(); }
    }
    function loadContentIcon(node: HTMLElement, initial: { entry: FileTreeEntry; folder: string }) {
        let value = initial;
        const request = () => {
            const { entry, folder } = value;
            if (!entry.iconKey || entry.iconResolved || entry.icon || iconRequested.has(entry.iconKey)) return;
            iconRequested.add(entry.iconKey);
            iconQueue.set(entry.iconKey, { name: entry.name, folder, id: instanceId, key: entry.iconKey });
            clearTimeout(iconTimer);
            iconTimer = setTimeout(() => void flushContentIcons(), 30);
        };
        const observer = new IntersectionObserver(entries => { if (entries.some(entry => entry.isIntersecting)) request(); }, { rootMargin: '100px' });
        observer.observe(node);
        return { update(next: typeof initial) { value = next; if (node.getBoundingClientRect().top < window.innerHeight + 100) request(); }, destroy() { observer.disconnect(); } };
    }

    async function loadSection(section: DataSection, force = false) {
        const id = instanceId;
        if (!id) return;
        const path = fileSubPath;
        const world = datapackWorld;
        const key = `${id}:${section}:${section === "ficheiros" ? path : section === "datapacks" ? world : ""}`;
        const generation = dataGeneration;
        const pending = sectionRequests.get(key);
        if (pending) { await pending; if (!force && loadedSections.has(key)) return; }
        if (generation !== dataGeneration || (!force && loadedSections.has(key))) return;
        pendingLoads++;
        isLoadingData = true;
        const request = (async () => {
            const current = () => generation === dataGeneration && id === instanceId;
            if (section === "configuracoes") {
                const specs = await getSystemSpecs();
                if (!current()) return;
                if (specs?.totalRamMb > 0) systemRamMb = specs.totalRamMb;
                gpuInfo = await optimizerDetectGpu();
                if (!current()) return;
                if (!current()) return;
                await detectJava();
            } else if (section === "mundos") {
                const data = await instanceWorldsList(id);
                if (current()) worldsList = data;
            } else if (section === "galeria") {
                const data = await instancesScreenshots(id);
                if (current()) screenshotsList = data;
            } else {
                if (section === "datapacks") {
                    const worlds = await instanceWorldsList(id);
                    if (!current() || (world && world !== datapackWorld)) return;
                    worldsList = worlds;
                    if (!worldsList.some(world => world.folderName === datapackWorld)) datapackWorld = worldsList[0]?.folderName || "";
                    if (!datapackWorld) { dataPacks = []; loadedSections.add(key); return; }
                }
                const folder = section === "datapacks" ? `saves/${datapackWorld}/datapacks` : section === "ficheiros" ? path || undefined : section === "shaders" ? "shaderpacks" : section;
                const data = await instanceFileTree(id, folder);
                if (!current() || (section === "datapacks" && folder !== `saves/${datapackWorld}/datapacks`)) return;
                if (section === "mods") {
                    instanceMods = data;
                    if (!resolvedMetadata.has(id)) {
                        resolvedMetadata.add(id);
                        metadataTimer = setTimeout(() => {
                            void (async () => {
                                const results = await Promise.allSettled([
                                    data.some(mod => /^\d+(_\d+)?\.jar$/.test(mod.name.replace('.disabled', ''))) ? modsResolveNames(id) : Promise.resolve(0),
                                    Promise.resolve(0)
                                ]);
                                const changed = results.reduce((total,result) => total + (result.status === "fulfilled" ? result.value : 0), 0);
                                if (current() && changed > 0) await loadSection("mods", true);
                            })().catch(() => {});
                        }, 0);
                    }
                } else if (section === "resourcepacks") resourcePacks = data;
                else if (section === "shaders") shaderPacks = data;
                else if (section === "datapacks") dataPacks = data;
                else fileTree = data;
                if (folder && ['mods', 'resourcepacks', 'shaderpacks'].includes(folder) && settings.value.preloadContentIcons !== false) { warmIcons(data); queueContentIcons(data, folder); }
            }
            if (current()) loadedSections.add(key);
        })().catch(error => {
            if (generation === dataGeneration) toast(uiText("ui.24951c37dc7ac3cd") + String(error), "error");
        }).finally(() => {
            if (sectionRequests.get(key) === request) sectionRequests.delete(key);
            pendingLoads--;
            isLoadingData = pendingLoads > 0;
        });
        sectionRequests.set(key, request);
        await request;
    }

    $effect(() => {
        const id = instanceId;
        untrack(() => {
            dataGeneration++;
            iconQueue.clear(); iconLookups.length = 0; iconRequested.clear(); clearTimeout(iconTimer);
            loadedSections.clear();
            resolvedMetadata.clear();
            mainTab = "conteudo"; subTab = "mods";
            searchQuery = ""; debouncedSearch = ""; selectedModNames = []; activeMenuMod = null; availableModUpdates = [];
            try {
                const saved: unknown = JSON.parse(localStorage.getItem(`luxmc:frozen-mods:${id}`) ?? '[]');
                frozenModNames = Array.isArray(saved) ? saved.filter((name): name is string => typeof name === 'string').slice(0, 10000) : [];
            } catch { frozenModNames = []; }
            instanceMods = []; resourcePacks = []; shaderPacks = []; dataPacks = [];
            worldsList = []; datapackWorld = ""; screenshotsList = []; fileTree = [];
            fileSubPath = ""; fileBreadcrumbs = [];
            shieldResult = null; hasAutoScannedShield = false;
            const generation = dataGeneration;
            const version = activeProfile?.mcVersion;
            if (id && version) void versionsCheckInstalled(version).then(value => {
                if (generation === dataGeneration) isInstalled = value;
            }).catch(() => {});
        });
        return () => { dataGeneration++; clearTimeout(metadataTimer); };
    });

    $effect(() => {
        const id = instanceId;
        if (!id) return;
        untrack(() => { void loadSection("mods"); });
        const timer = setTimeout(() => {
            if (instanceId !== id) return;
            for (const section of ["resourcepacks", "shaders", "datapacks"] as const) void loadSection(section);
        }, 250);
        return () => clearTimeout(timer);
    });


    onMount(() => {
        const timer = setTimeout(() => { void checkModpackUpdate(); }, 2000);
        return () => clearTimeout(timer);
    });

    async function refreshAllData() {
        loadedSections.clear();
        if (!instanceId) return;
        await Promise.allSettled((["mods", "resourcepacks", "shaders", "datapacks", "mundos", "galeria", "ficheiros"] as const).map(section => loadSection(section, true)));
    }

	async function handleToggleMod(mod: FileTreeEntry) {
		if (isChangingMods) return;
		isChangingMods = true;
		const id = instanceId;
		const isCurrentlyDisabled = mod.name.endsWith(".disabled");
		try {
			await instanceModToggle(id, mod.name, isCurrentlyDisabled);
			if (id !== instanceId) return;
			toast(isCurrentlyDisabled ? `Mod ativado!` : `Mod desativado!`, "success");
			await loadSection("mods", true);
		} catch (e) {
			toast(uiText("ui.602701edd10dcc87") + String(e), "error");
		} finally { isChangingMods = false; }
	}

	async function handleDeleteMod(mod: FileTreeEntry) {
		try {
			await instanceModDelete(instanceId, mod.name);
			playSound("delete");
			toast(uiText("ui.76da1453ec791bf3", {arg0: (mod.name)}), "success");
			await refreshAllData();
		} catch (e) {
			toast(uiText("ui.882c8bee40eaf46f") + String(e), "error");
		}
	}

	async function handleAddModFile() {
		try {
			const selected = await open({
				title: uiText("ui.1811d33a8372f4f0"),
				multiple: true,
				filters: [{ name: "Mod do Minecraft (.jar)", extensions: ["jar"] }]
			});
			if (!selected) return;
			const paths = Array.isArray(selected) ? selected : [selected];
			let added = 0;
			const failed: string[] = [];
			for (const p of paths) {
				try {
					await instanceModAdd(instanceId, p);
					added++;
				} catch (error) {
					console.error("instance_mod_add failed:", p, error);
					failed.push(`${p.split(/[\\/]/).pop()}: ${String(error)}`);
				}
			}
			await refreshAllData();
			if (added) toast(uiText("ui.130c4ba5e16f85ca", {arg0: (added)}), "success");
			if (failed.length) toast(uiText("ui.a10afc7c4031c1c0", {arg0: (failed.length), arg1: (failed.slice(0, 2).join("; "))}), "error");
		} catch (e) {
			toast(uiText("ui.6de32492351b3bfe") + String(e), "error");
		}
	}

	async function handleOpenModsFolder() {
		try {
			await instanceModsOpenFolder(instanceId);
		} catch (e) {
			toast(uiText("ui.251b9aac7e9713f4") + String(e), "error");
		}
	}

	function isModsFileDrag(e: DragEvent): boolean {
		if (subTab !== "mods" || isImportingDrop) return false;
		const types = e.dataTransfer?.types;
		if (!types) return false;
		return Array.from(types).includes("Files");
	}

	function handleModsDragEnter(e: DragEvent) {
		if (!isModsFileDrag(e)) return;
		e.preventDefault();
		modsDragDepth += 1;
		modsDragOver = true;
	}

	function handleModsDragOver(e: DragEvent) {
		if (!isModsFileDrag(e)) return;
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
	}

	function handleModsDragLeave() {
		if (!modsDragOver) return;
		modsDragDepth = Math.max(0, modsDragDepth - 1);
		if (modsDragDepth === 0) modsDragOver = false;
	}

	function fileToBase64(file: Blob): Promise<string> {
		return new Promise((resolve, reject) => {
			const reader = new FileReader();
			reader.onload = () => {
				const raw = String(reader.result ?? "");
				const sep = raw.indexOf(",");
				resolve(sep >= 0 ? raw.slice(sep + 1) : raw);
			};
			reader.onerror = () => reject(reader.error ?? new Error(uiText("ui.4a07e3a9501f9478")));
			reader.readAsDataURL(file);
		});
	}

	async function handleModsDrop(e: DragEvent) {
		e.preventDefault();
		modsDragDepth = 0;
		modsDragOver = false;
		if (subTab !== "mods" || isImportingDrop) return;

		const files = Array.from(e.dataTransfer?.files ?? []);
		const jars = files.filter((f) => f.name.toLowerCase().endsWith(".jar"));
		if (!jars.length) {
			toast(uiText("ui.43da397ce0b54e88"), "error");
			return;
		}

		isImportingDrop = true;
		let added = 0;
		const failed: string[] = [];
		try {
			for (const file of jars) {
				if (file.size > 128 * 1024 * 1024) {
					failed.push(`${file.name}: maior que 128 MB`);
					continue;
				}
				try {
					const direct = (file as unknown as { path?: string }).path;
					let ok = false;
					if (direct) {
						try {
							await instanceModAdd(instanceId, direct);
							ok = true;
						} catch (err) {
							console.warn(uiText("ui.8f7680e47215af89"), err);
						}
					}
					if (!ok) {
						const data = await fileToBase64(file);
						await instanceModAddBytes(instanceId, file.name, data);
					}
					added += 1;
				} catch (error) {
					console.error("drop import failed:", file.name, error);
					failed.push(`${file.name}: ${String(error)}`);
				}
			}
			await refreshAllData();
			if (added) toast(uiText("ui.130c4ba5e16f85ca", {arg0: (added)}), "success");
			if (failed.length) toast(uiText("ui.a10afc7c4031c1c0", {arg0: (failed.length), arg1: (failed.slice(0, 2).join("; "))}), "error");
		} finally {
			isImportingDrop = false;
		}
	}

	async function navigateToFolder(folderName: string) {
		fileBreadcrumbs = [...fileBreadcrumbs, folderName];
		fileSubPath = fileBreadcrumbs.join("/");
		fileTree = await instanceFileTree(instanceId, fileSubPath).catch(() => []);
	}

	async function navigateBreadcrumb(index: number) {
		if (index < 0) {
			fileBreadcrumbs = [];
			fileSubPath = "";
		} else {
			fileBreadcrumbs = fileBreadcrumbs.slice(0, index + 1);
			fileSubPath = fileBreadcrumbs.join("/");
		}
		fileTree = await instanceFileTree(instanceId, fileSubPath || undefined).catch(() => []);
	}

	async function navigateUp() {
		if (fileBreadcrumbs.length > 0) {
			fileBreadcrumbs = fileBreadcrumbs.slice(0, -1);
			fileSubPath = fileBreadcrumbs.join("/");
			fileTree = await instanceFileTree(instanceId, fileSubPath || undefined).catch(() => []);
		}
	}

	async function handleOpenFile(file: FileTreeEntry) {
		if (file.isDir) {
			await navigateToFolder(file.name);
		} else {
			if (file.size > 300000) {
				toast(uiText("ui.21f8ab933ab2229b", {arg0: (Math.round(file.size / 1024))}), "info");
				return;
			}
			const isText = /\.(txt|json|properties|toml|log|cfg|mcmeta|yaml|yml|ini|csv|md|sh|lock|json5)$/i.test(file.name);
			if (isText) {
				try {
					const content = await instanceFileRead(instanceId, file.path);
					activeEditorFile = { path: file.path, name: file.name, content };
				} catch (e) {
					toast(uiText("ui.536cc065f204a1c1") + String(e), "error");
				}
			} else {
				toast(uiText("ui.e3785c74af0bd42f", {arg0: (Math.round(file.size / 1024))}), "info");
			}
		}
	}

	async function handleSaveEditorFile() {
		if (!activeEditorFile) return;
		isSavingEditor = true;
		try {
			await instanceFileWrite(instanceId, activeEditorFile.path, activeEditorFile.content);
			toast(uiText("ui.97f3464fc2a2ebe4", {arg0: (activeEditorFile.name)}), "success");
		} catch (e) {
			toast(uiText("ui.18e5596b3983ad06") + String(e), "error");
		} finally {
			isSavingEditor = false;
		}
	}

	async function handleDeleteFileEntry(file: FileTreeEntry) {
		if (!confirm(uiText("ui.3d4f9b19fb69e3dc", {arg0: (file.name)}))) return;
		try {
			await instanceFileDelete(instanceId, file.path);
			playSound("delete");
			toast(uiText("ui.977616b756fa0a24", {arg0: (file.name)}), "success");
			fileTree = await instanceFileTree(instanceId, fileSubPath || undefined).catch(() => []);
		} catch (e) {
			toast(uiText("ui.f1e8ee7a5d4f9972") + String(e), "error");
		}
	}

	async function handleDeleteWorld(folderName: string) {
		if (!instanceId) return;
		if (!confirm(uiText("ui.2538b779c071153e", {arg0: (folderName)}))) return;
		try {
			await instanceWorldDelete(instanceId, folderName);
			playSound("delete");
			toast(uiText("ui.6340a3cf6c8c240c", {arg0: (folderName)}), "success");
			worldsList = worldsList.filter(w => w.folderName !== folderName);
		} catch (e) {
			toast(uiText("ui.6cea4ae536ae622b") + String(e), "error");
		}
	}

	let isImportingWorld = $state(false);

	async function handleImportWorld() {
		if (!instanceId || isImportingWorld) return;
		try {
			const selected = await open({
				multiple: false,
				directory: false,
				title: uiText("ui.c2a914f873589639"),
				filters: [
					{ name: uiText("ui.fb1184809a30129e"), extensions: ["zip"] },
					{ name: uiText("ui.d2385f159b9df3d6"), extensions: ["*"] }
				]
			});
			if (!selected) return;
			const filePath = typeof selected === "string" ? selected : selected[0];
			if (!filePath) return;

			isImportingWorld = true;
			toast(uiText("ui.dfdec63dbc7386d1"), "info");
			const imported = await instanceWorldImport(instanceId, filePath);
			playSound("achievement");
			toast(uiText("ui.33f1b4853e2883a1", {arg0: (imported.name)}), "success");
			worldsList = await instanceWorldsList(instanceId);
		} catch (e) {
			toast(uiText("ui.3df571bd0d6c5ef9") + String(e), "error");
		} finally {
			isImportingWorld = false;
		}
	}

	async function handleImportWorldFolder() {
		if (!instanceId || isImportingWorld) return;
		try {
			const selected = await open({
				multiple: false,
				directory: true,
				title: uiText("ui.5e4b17f0588ebccd")
			});
			if (!selected) return;
			const folderPath = typeof selected === "string" ? selected : selected[0];
			if (!folderPath) return;

			isImportingWorld = true;
			toast(uiText("ui.3a897767fb77dda4"), "info");
			const imported = await instanceWorldImport(instanceId, folderPath);
			playSound("achievement");
			toast(uiText("ui.33f1b4853e2883a1", {arg0: (imported.name)}), "success");
			worldsList = await instanceWorldsList(instanceId);
		} catch (e) {
			toast(uiText("ui.3df571bd0d6c5ef9") + String(e), "error");
		} finally {
			isImportingWorld = false;
		}
	}

	async function checkDoctorConflictsManual() {
		const targetProfileId = activeProfile?.id || instanceId || "";
		try {
			const res = await doctorCheckInstanceConflicts(targetProfileId);
			if (res.hasConflicts) {
				pendingLaunchConflicts = res;
				showModConflictModal = true;
			} else {
				toast(uiText("ui.102a8ab0170c6104"), "success");
			}
		} catch (e) {
			toast(uiText("ui.3f8f4932b9580466") + String(e), "error");
		}
	}

	let launchResetTimer: ReturnType<typeof setTimeout> | null = null;
	let launchGuard = false;

	async function handlePlay(skipConflictCheck: boolean = false) {
		if (launchGuard || isLaunching) return;
		launchGuard = true;
		try {
			await runPlay(skipConflictCheck);
		} finally {
			launchGuard = false;
		}
	}

	async function runPlay(skipConflictCheck: boolean = false) {
		if (launchResetTimer) {
			clearTimeout(launchResetTimer);
			launchResetTimer = null;
		}
		if (isLaunching) return;

		const targetProfileId = activeProfile?.id || instanceId || "";
		if (!skipConflictCheck) {
			try {
				let readiness = await doctorInstanceReadiness(targetProfileId);
				if (!readiness.ready && readiness.conflicts.hasConflicts) {
					pendingLaunchConflicts = readiness.conflicts;
					showModConflictModal = true;
					return;
				}
				if (!readiness.ready && readiness.repairable) {
					appState.isLaunching = true;
					appState.launchStatusText = uiText("ui.b6d484d52a57df94");
					toast(uiText("ui.5d2502a90ec845c1"), "info");
					try {
						const outcome = await doctorRepairAll(targetProfileId);
						for (const warning of outcome.warnings) toast(warning, "warning");
					} catch (repairError) {
						console.error("Auto repair failed:", repairError);
						toast(uiText("ui.793b29f6f9f9c6e2") + String(repairError), "error");
						appState.isLaunching = false;
						return;
					}
					readiness = await doctorInstanceReadiness(targetProfileId);
					if (!readiness.ready) {
						toast(readiness.blockers[0] || uiText("ui.0a6aaa8e4cd3261f"), "error");
						appState.isLaunching = false;
						return;
					}
					appState.isLaunching = false;
				}
				if (!readiness.ready) {
					toast(uiText("ui.bb62da2d79a47c08", {arg0: (readiness.blockers[0] || uiText("ui.9e8298233a54f645"))}), "error");
					return;
				}
				for (const warning of readiness.warnings) toast(warning, "info");
				const conflicts = readiness.conflicts;
				if (conflicts.hasConflicts) {
					pendingLaunchConflicts = conflicts;
					showModConflictModal = true;
					return;
				}
			} catch (e) {
				console.warn("Pre-launch conflict check error:", e);
			}
		}

		appState.isLaunching = true;
		downloadProgressPercent = 15;
		appState.launchStatusText = uiText("ui.99900384cb24abb8");

		try {
			// Ensure valid account
			let userUuid = account.value?.uuid;
			let userAccId = account.value?.id || userUuid;
			if (!userUuid) {
				const devAcc = await authDevLogin();
				account.value = {
					id: devAcc.id,
					username: devAcc.username,
					uuid: devAcc.uuid,
					minecraftToken: devAcc.accessToken,
					expiresAt: devAcc.expiresAt ? (devAcc.expiresAt < 1e11 ? devAcc.expiresAt * 1000 : devAcc.expiresAt) : 0
				};
				userUuid = devAcc.uuid;
				userAccId = devAcc.id;
			}

			const verId = activeProfile?.mcVersion || "1.20.4";
			downloadProgressPercent = 35;
			appState.launchStatusText = uiText("ui.8fbc6893bdbc6efd");

			// If not installed, download version
			if (!isInstalled) {
				downloadProgressPercent = 55;
				appState.launchStatusText = uiText("ui.4d7cbc3453407632") + verId + "...";
				await versionsDownload(verId);
				isInstalled = true;
			}

			downloadProgressPercent = 85;
			appState.launchStatusText = uiText("ui.dc11fbd10f868b09");
			const targetProfileId = activeProfile?.id || instanceId || "";
			const globalVulkan = typeof window !== "undefined" && localStorage.getItem("luxmc_enable_vulkan") === "true";
			const isVulkan = globalVulkan || activeProfile?.useVulkan === true;
			const skinToPass = activeSkinStore.current.skinUrl || account.value?.skinUrl || null;
			const effectiveCape = activeSkinStore.current.hasCape
				? (activeSkinStore.current.customCapeUrl || (activeSkinStore.current.capeType && activeSkinStore.current.capeType !== "none" ? getFullCapeDataUrl(activeSkinStore.current.capeType) : null))
				: (account.value?.capeUrl || null);
			const result = await launchGame({
				versionId: verId,
				accountId: userUuid || "",
				profileId: targetProfileId,
				enableVulkan: isVulkan,
				skinUrl: skinToPass,
				skinVariant: activeSkinStore.current.type === "alex" ? "slim" : "classic",
				capeUrl: effectiveCape
			});

			gamingStats.onGameStart(targetProfileId);
			appState.isGameRunning = true;
			appState.activeGameDetails = { profileId: targetProfileId, name: activeProfile?.name || "Minecraft", version: verId, loader: activeProfile?.loader || "vanilla" };

			if (activeProfile) {
				profilesUpdate({
					id: activeProfile.id,
					lastPlayed: new Date().toISOString(),
					launchCount: (activeProfile.launchCount || 0) + 1,
				}).catch(() => {});
			}

			const pNameLower = (activeProfile?.name || "").toLowerCase();
			const modpackCover = (activeProfile?.icon && (activeProfile.icon.startsWith("http") || activeProfile.icon.startsWith("data:")))
				? activeProfile.icon
				: pNameLower.includes("better mc") || pNameLower.includes("bmc")
				? "https://raw.githubusercontent.com/predabr/luxmc/main/build/modpack_better_mc.webp"
				: pNameLower.includes("pixelmon") || pNameLower.includes("cobblemon")
				? "https://raw.githubusercontent.com/predabr/luxmc/main/build/modpack_cobblemon.webp"
				: pNameLower.includes("fabulously optimized") || pNameLower.includes("fo")
				? "https://raw.githubusercontent.com/predabr/luxmc/main/build/modpack_fo.webp"
				: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png";

			const loaderText = activeProfile?.loader ? activeProfile.loader.toUpperCase() : "Vanilla";
			const modCountText = instanceMods.length > 0 ? ` (${instanceMods.length} mods)` : "";

			discordSetActivity({
				inGame: true,
				details: activeProfile?.name || "Minecraft",
				state: `Minecraft ${verId} · ${loaderText}${modCountText}`,
				largeText: activeProfile?.name || `Minecraft ${verId}`,
				largeImage: modpackCover,
				smallImage: "grass",
				smallText: `Luxmc · ${loaderText}`,
				startTime: Math.floor(Date.now() / 1000),
				buttons: [
					{ label: uiText("ui.4c8757fab2a21345"), url: "https://luxmc-r92.pages.dev" },
					{ label: uiText("ui.d4fb4e24ae3c7a8f"), url: "https://luxmc-r92.pages.dev" }
				]
			}).catch(() => {});

			gamingStats.onGameStart(activeProfile?.id || "");
			appState.isGameRunning = true;
			appState.activeGameDetails = {
				profileId: activeProfile?.id,
				name: activeProfile?.name || "Minecraft",
				version: verId,
				loader: activeProfile?.loader || "vanilla"
			};
			if (activeProfile?.id) profiles.setLastPlayed(activeProfile.id);
			downloadProgressPercent = 100;
			appState.launchStatusText = uiText("ui.56374a8d934236e2", {arg0: (result.pid)});
			toast(uiText("ui.f9462c8ca5b0a2ac", {arg0: (verId), arg1: (result.pid)}), "success");
			void handlePostLaunchActions();
		} catch (e) {
			console.error("Launch error:", e);
			toast(uiText("ui.ea9a9c9188e1f662") + String(e), "error");
			appState.launchStatusText = "";
			downloadProgressPercent = 0;
			appState.isGameRunning = false;
		} finally {
			if (launchResetTimer) clearTimeout(launchResetTimer);
			launchResetTimer = setTimeout(() => {
				launchResetTimer = null;
				appState.isLaunching = false;
				appState.launchingProfileId = null;
				appState.launchStatusText = "";
				downloadProgressPercent = 0;
			}, 2000);
		}
	}

	async function handleStopGame() {
		try {
			appState.isStopping = true;
			appState.wasManuallyTerminated = true;
			await stopGame();
			appState.isGameRunning = false;
			gamingStats.onGameExit();
			toast(uiText("ui.ac75d5c7df6f61ef"), "info");
		} catch (e) {
			appState.isGameRunning = false;
			gamingStats.onGameExit();
			toast(uiText("ui.f635cd0acfe6def9") + String(e), "error");
		} finally {
			appState.isStopping = false;
		}
	}

	async function handleAddResourcePack(packType: "shaderpacks" | "datapacks" | "resourcepacks" = "resourcepacks") {
		const label = packType === 'shaderpacks' ? 'Shader' : packType === 'datapacks' ? 'Datapack' : uiText("ui.963f78de2df74f50");
		try {
			const selected = await open({
				title: uiText("ui.3afa608d3f7b29df", {arg0: (label)}),
				multiple: true,
				filters: [{ name: "Arquivo Compactado (.zip)", extensions: ["zip"] }]
			});
			if (!selected) return;
			const paths = Array.isArray(selected) ? selected : [selected];
			for (const p of paths) {
				await instancePackAdd(instanceId, packType, p, packType === "datapacks" ? datapackWorld : undefined);
			}
			toast(uiText("ui.fd5be19f7f80fe28", {arg0: (paths.length), arg1: (label.toLowerCase())}), "success");
			await refreshAllData();
		} catch (e) {
			toast(uiText("ui.9723ec0453e0c16f") + String(e), "error");
		}
	}

	async function handleDeletePack(fileName: string, packType: "shaderpacks" | "datapacks" | "resourcepacks") {
		try {
			await instancePackDelete(instanceId, packType, fileName, packType === "datapacks" ? datapackWorld : undefined);
			playSound("delete");
			toast(uiText("ui.f3e43f3065bf2a2b"), "success");
			await refreshAllData();
		} catch (e) {
			toast(uiText("ui.22fc4b84ee52a446") + String(e), "error");
		}
	}

	async function handleOpenPackFolder(packType: "shaderpacks" | "datapacks" | "resourcepacks") {
		try {
			await instancePackOpenFolder(instanceId, packType, packType === "datapacks" ? datapackWorld : undefined);
		} catch (e) {
			toast(uiText("ui.b9f5a23e16422403") + String(e), "error");
		}
	}

	async function handleDeleteScreenshot(path: string) {
		try {
			await screenshotDelete(path);
			playSound("delete");
			screenshotsList = screenshotsList.filter(s => s.path !== path);
			toast(uiText("ui.37c8bb8f8416b9a5"), "info");
		} catch (e) {
			toast(uiText("ui.2132eacd3690c6de") + String(e), "error");
		}
	}

	async function checkModpackUpdate() {
		if (!instanceId || isCheckingUpdate) return;
		isCheckingUpdate = true;
		try {
			const info = await modpackCheckUpdate(instanceId);
			if (info && info.hasUpdate) {
				modpackUpdate = info;
				modpackDiff = await modpackVersionDiff(instanceId, info.versionId || "", info.source || "").catch(() => null);
			} else {
				modpackUpdate = null;
				modpackDiff = null;
			}
		} catch (e) {
			console.debug("Modpack update check not applicable or failed:", e);
		} finally {
			isCheckingUpdate = false;
		}
	}

	async function handleApplyModpackUpdate() {
		if (!modpackUpdate || !instanceId || isUpdatingModpack) return;
        if (appState.isGameRunning) { toast(uiText("ui.5c121f6239aa0903"), "warning"); return; }
		isUpdatingModpack = true;
		updateStatusText = uiText("ui.b35e32e9887ce30f");
		updateProgressPercent = 5;

		let unlistenProgress: (() => void) | null = null;
		try {
			unlistenProgress = await listen<{ phase?: string; percent?: number; status?: string }>(
				"modpack-progress",
				(event) => {
					if (event.payload.status) updateStatusText = event.payload.status;
					if (typeof event.payload.percent === "number") updateProgressPercent = event.payload.percent;
				}
			);

			await modpackUpdateAtomic(
				instanceId,
				modpackUpdate.versionId || "",
				modpackUpdate.source || "modrinth",
				modpackUpdate.projectId || ""
			);

			toast(uiText("ui.5f480d299789b92c", {arg0: (modpackUpdate.latestVersion || "mais recente")}), "success");
			playSound("chime");
			modpackUpdate = null;
			modpackDiff = null;
			await profiles.refresh();
            await refreshAllData();
		} catch (e) {
			console.error(uiText("ui.5a393de14381bc7c"), e);
			toast(uiText("ui.f3b332eeaeaf27ca") + String(e), "error");
		} finally {
			if (unlistenProgress) unlistenProgress();
			isUpdatingModpack = false;
			updateStatusText = "";
			updateProgressPercent = 0;
		}
	}

	async function openInstanceFolder() {
		if (!instanceId) return;
		try {
			await instancesOpenFolder(instanceId);
		} catch (e) {
			toast(uiText("ui.b9f5a23e16422403") + String(e), "error");
		}
	}

	async function openScreenshotsFolder() {
		if (!instanceId) return;
		try {
			await screenshotsOpenFolder(instanceId);
		} catch (e) {
			toast(uiText("ui.4a03c8f6feb5584d") + String(e), "error");
		}
	}
</script>

<div class="flex gap-6 w-full select-none">

	<div class="flex-1 flex flex-col min-w-0 space-y-5">

		<a href="/instances" class={button({ variant: "ghost", size: "sm", class: "w-fit group" })}>
			<ArrowLeft class="w-3.5 h-3.5 transition-transform group-hover:-translate-x-1" /> {uiText("common.back")}
		</a>

        <InstanceHero profile={activeProfile} banner={heroBanner} launching={isLaunching} running={appState.isGameRunning} stopping={appState.isStopping} status={launchStatusText} progress={downloadProgressPercent}
            javaLabel={javaRuntimes.find(runtime => runtime.path === activeProfile?.javaPath)?.versionString || (activeProfile?.javaPath ? 'Personalizado' : uiText("ui.b803f24d52edd7bc"))}
            onPlay={() => handlePlay()} onStop={handleStopGame} onSettings={() => goToSection("configuracoes")} onHost={() => showP2PHost = true}>
            {#snippet actions()}
                <button type="button" class={button({ variant: 'ghost', size: 'sm' })} onclick={openInstanceFolder}><FolderOpen class="h-4 w-4" />{uiText("screenshots.openFolderBtn")}</button>
                <button type="button" class={button({ variant: 'ghost', size: 'sm' })} onclick={() => showP2PHost = true}><Radio class="h-4 w-4" />{uiText("ui.4cab082a07901725")}</button>
                <button type="button" class={button({ variant: toolsExpanded ? 'secondary' : 'ghost', size: 'sm' })} aria-expanded={toolsExpanded} aria-controls="instance-toolbox" onclick={() => toolsExpanded = !toolsExpanded}><Wrench class="h-4 w-4" />{uiText("settings.refinement.instanceTools")}</button>
            {/snippet}
        </InstanceHero>
        {#if toolsExpanded}
            <section id="instance-toolbox" class="surface-glass space-y-5 p-5 sm:p-6" aria-label={uiText("settings.refinement.instanceTools")} in:fade={{ duration: 150 }}>
                <header class="flex items-start justify-between gap-4">
                    <div class="space-y-1"><p class="page-eyebrow">{uiText("settings.refinement.instanceTools")}</p><h2 class="text-lg font-semibold text-fg">{uiText("ui.fa18e9c316c39701")}</h2><p class="text-sm text-fg-muted">{uiText("ui.5cccec99fd3f2ff4")}</p></div>
                    <button type="button" class={button({ variant: 'ghost', size: 'icon' })} aria-label={uiText("common.close")} onclick={() => toolsExpanded = false}><X class="h-4 w-4" /></button>
                </header>
<section class="instance-settings-group" aria-label={uiText("ui.fa18e9c316c39701")}>
                        <div class="space-y-5">
                            <div class="grid gap-3 sm:grid-cols-2">
                                <button type="button" class={button({ variant: "secondary", class: "h-auto justify-start whitespace-normal p-4 text-left" })} onclick={() => showKeybindEditorModal = true}><Keyboard class="h-5 w-5 shrink-0 text-brand-400" /><span><span class="block text-sm font-semibold">{uiText("ui.66098554bb521658")}</span><span class="mt-1 block text-xs font-normal text-fg-muted">{uiText("ui.2ed6f8926780ee51")}</span></span></button>
                                <button type="button" class={button({ variant: "secondary", class: "h-auto justify-start whitespace-normal p-4 text-left" })} onclick={() => showConfigEditor = true}><Code class="h-5 w-5 shrink-0 text-brand-400" /><span><span class="block text-sm font-semibold">{uiText("ui.4d54a8d687f8a2ff")}</span><span class="mt-1 block text-xs font-normal text-fg-muted">{uiText("ui.a7fbda48ffe03520")}</span></span></button>
                                <button type="button" class={button({ variant: "secondary", class: "h-auto justify-start whitespace-normal p-4 text-left" })} onclick={() => showJukeboxModal = true}><Disc class="h-5 w-5 shrink-0 text-brand-400" /><span><span class="block text-sm font-semibold">{uiText("ui.b2ced5c6bc6abca1")}</span><span class="mt-1 block text-xs font-normal text-fg-muted">{uiText("ui.71031146a0ac2853")}</span></span></button>
                                <button type="button" class={button({ variant: "secondary", class: "h-auto justify-start whitespace-normal p-4 text-left" })} onclick={() => showModpackExportModal = true}><Package class="h-5 w-5 shrink-0 text-brand-400" /><span><span class="block text-sm font-semibold">{uiText("ui.1cbf90a624d3ece6")}</span><span class="mt-1 block text-xs font-normal text-fg-muted">{uiText("ui.c2d63059db8afdbb")}</span></span></button>
                                <button type="button" class={button({ variant: "secondary", class: "h-auto justify-start whitespace-normal p-4 text-left" })} onclick={() => showWorldBackupModal = true}><HardDrive class="h-5 w-5 shrink-0 text-brand-400" /><span><span class="block text-sm font-semibold">{uiText("ui.adc2b8d01a4c7a79")}</span><span class="mt-1 block text-xs font-normal text-fg-muted">{uiText("ui.eb9382115d772ec9")}</span></span></button>
                                <button type="button" class={button({ variant: "secondary", class: "h-auto justify-start whitespace-normal p-4 text-left" })} onclick={() => showDeathDetectorModal = true}><Skull class="h-5 w-5 shrink-0 text-brand-400" /><span class="text-sm font-semibold">{uiText("ui.d88bf4aee9b1d26f")}</span></button>
                                <button type="button" class={button({ variant: "secondary", class: "h-auto justify-start whitespace-normal p-4 text-left" })} onclick={openInstanceFolder}><FolderOpen class="h-5 w-5 shrink-0 text-brand-400" /><span><span class="block text-sm font-semibold">{uiText("screenshots.openFolderBtn")}</span><span class="mt-1 block text-xs font-normal text-fg-muted">{uiText("ui.780c409845e4aae2")}</span></span></button>
                            </div>
                        </div>
                    </section>
                    <details class="instance-settings-group" ontoggle={(event) => labExpanded = event.currentTarget.open}>
                        <summary class="flex cursor-pointer items-center gap-3 text-sm font-semibold text-fg"><Activity class="h-5 w-5 text-brand-400" />{uiText("ui.ff0415cca9ac2e70")}<span class="ml-auto text-xs font-normal text-fg-muted">{uiText("ui.c57da88ed5af4812")}</span></summary>
                        {#if labExpanded}<div class="mt-5"><InstanceLab profileId={instanceId} /></div>{/if}
                    </details>
                                </section>
        {/if}
        {#if modpackUpdate?.projectId}<ModpackVersions profileId={instanceId} info={modpackUpdate} onChanged={refreshAllData} />{/if}

		{#if modpackUpdate?.hasUpdate || isUpdatingModpack}
			<div class="bg-gradient-to-r from-amber-500/15 via-orange-500/10 to-bg-elevated border border-amber-500/30 rounded-3xl p-5 shadow-xl relative overflow-hidden">
				<div class="absolute -right-10 -bottom-10 w-40 h-40 bg-[radial-gradient(circle_at_center,rgb(245_158_11/0.15),transparent_70%)] rounded-full pointer-events-none"></div>
				<div class="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 relative z-10">
					<div class="flex items-start gap-3.5">
						<div class="w-12 h-12 rounded-2xl bg-gradient-to-br from-amber-500 to-orange-600 flex items-center justify-center text-brand-foreground font-black shadow-lg shrink-0">
							<ArrowUpCircle class="w-6 h-6" />
						</div>
						<div class="space-y-1">
							<div class="flex items-center gap-2 flex-wrap">
								<span class="font-extrabold text-sm text-fg">{uiText("ui.5f428ec2d324f417")}</span>
								<span class="text-[10px] font-mono font-bold bg-amber-500/20 text-amber-300 border border-amber-500/30 px-2 py-0.5 rounded-full">
									{modpackUpdate?.latestVersion || uiText("ui.a23951e91db1e2cb")}
								</span>
								{#if modpackUpdate?.currentVersion}
									<span class="text-[10px] text-fg/40 font-mono">
										{uiText("ui.c70c18c86e1e73ea")} {modpackUpdate.currentVersion})
									</span>
								{/if}
							</div>
							<p class="text-xs text-fg/60">
								{uiText("ui.06ba47775007165a")} {modpackUpdate?.source === 'curseforge' ? 'CurseForge' : 'Modrinth'}.
							</p>
							<div class="flex items-center gap-1.5 text-[11px] font-bold text-emerald-400 pt-0.5">
								<ShieldCheck class="w-3.5 h-3.5 shrink-0" />
								<span>{uiText("ui.40411658f011d85a")}</span>
							</div>
							{#if modpackDiff?.available}
								<div class="mt-2 flex flex-wrap gap-1.5 text-[10px]">
									{#if modpackDiff.added.length}<span class="rounded-full bg-emerald-500/15 px-2 py-1 text-emerald-300">+{modpackDiff.added.length} {uiText("ui.b14d07bdc43e1b12")}</span>{/if}
									{#if modpackDiff.updated.length}<span class="rounded-full bg-amber-500/15 px-2 py-1 text-amber-300">↻{modpackDiff.updated.length} {uiText("ui.9dfce32f7678d26e")}</span>{/if}
									{#if modpackDiff.removed.length}<span class="rounded-full bg-rose-500/15 px-2 py-1 text-rose-300">−{modpackDiff.removed.length} {uiText("ui.538851c9c49de0bf")}</span>{/if}
								</div>
							{/if}
						</div>
					</div>

					<div class="shrink-0 flex items-center gap-3 w-full md:w-auto">
						{#if isUpdatingModpack}
							<div class="flex flex-col items-end gap-1.5 w-full min-w-[220px]">
								<div class="flex items-center justify-between w-full text-xs font-bold text-amber-300">
									<span class="flex items-center gap-1.5">
										<RefreshCw class="w-3 h-3 animate-spin" /> {updateStatusText || "Atualizando..."}
									</span>
									<span>{updateProgressPercent}%</span>
								</div>
								<div class="w-full h-2 bg-fg/10 rounded-full overflow-hidden">
									<div class="h-full bg-gradient-to-r from-amber-500 to-orange-500 transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-300 rounded-full" style="width: {updateProgressPercent}%"></div>
								</div>
							</div>
						{:else}
							<button
								type="button"
								class={launcherButton({ variant: "secondary", size: "lg", class: "w-full md:w-auto from-amber-500 to-orange-500 hover:from-amber-400 hover:to-orange-400 flex items-center justify-center gap-2" })}
								onclick={handleApplyModpackUpdate}
							>
								<Download class="w-4 h-4" /> {uiText("ui.5b3d667989437e73")}
							</button>
						{/if}
					</div>
				</div>
			</div>
		{/if}

        <div class="instance-overview-nav" role="tablist" tabindex="-1" onkeydown={handleTabKeys} aria-label={uiText("ui.afb02856c2b1242c")}>
            {#each [{ id: 'mods', label: 'Mods', icon: Puzzle }, { id: 'resourcepacks', label: uiText("ui.906d0853ece30a11"), icon: Box }, { id: 'shaders', label: 'Shaders', icon: Sparkles }, { id: 'datapacks', label: uiText("ui.f3260e4c97557038"), icon: Code }, { id: 'mundos', label: uiText("files.saves"), icon: Globe2 }, { id: 'galeria', label: uiText("ui.067348ce68943b63"), icon: Image }, { id: 'ficheiros', label: uiText("ui.c11445cb4fe129bf"), icon: Folder }, { id: 'configuracoes', label: uiText("ui.76b0fb6ad18939ac"), icon: SettingsIcon }] as section}
                {@const selected = mainTab === section.id || (mainTab === 'conteudo' && subTab === section.id)}
                <button type="button" role="tab" tabindex={selected ? 0 : -1} id={`instance-tab-${section.id}`} aria-selected={selected} aria-controls={`instance-${['mods', 'resourcepacks', 'shaders', 'datapacks'].includes(section.id) ? 'conteudo' : section.id}`} class={button({ variant: selected ? "secondary" : "ghost", size: "sm", class: selected ? 'text-brand-400 border-brand-400/30 bg-brand-500/10' : '' })} onclick={() => {
                    if (['mods', 'resourcepacks', 'shaders', 'datapacks'].includes(section.id)) {
                        subTab = section.id as typeof subTab;
                        goToSection('conteudo');
                    } else goToSection(section.id as typeof mainTab);
                }}><section.icon class="h-4 w-4" />{section.label}</button>
            {/each}
            <button type="button" class={button({ variant: 'ghost', size: 'icon' })} aria-label={uiText("ui.f289ac48d636afd5")} onclick={refreshAllData}><RefreshCw class="h-4 w-4 {isLoadingData ? 'animate-spin' : ''}" /></button>
        </div>
        {#if mainTab === 'conteudo'}

		<div use:pageMotion={subTab} id="instance-conteudo" role="tabpanel" aria-labelledby={`instance-tab-${subTab}`} class="instance-overview-section">
			<div
				class="relative flex flex-col gap-4"
				in:fade={{ easing: quintOut, duration: 220 }}
				role="group"
				aria-label={uiText("ui.afb02856c2b1242c")}
				ondragenter={handleModsDragEnter}
				ondragover={handleModsDragOver}
				ondragleave={handleModsDragLeave}
				ondrop={handleModsDrop}
			>
				{#if modsDragOver || isImportingDrop}
					<div class="pointer-events-none absolute inset-0 z-40 flex flex-col items-center justify-center gap-3 rounded-3xl border-2 border-dashed {isImportingDrop && !modsDragOver ? 'border-brand-400/40' : 'border-brand-400/70'} bg-bg-overlay/75 backdrop-blur-sm">
						<div class="h-14 w-14 rounded-2xl bg-brand-500/15 border border-brand-400/40 text-brand-300 flex items-center justify-center">
							<Puzzle class="w-7 h-7" />
						</div>
						<p class="text-sm font-black text-fg">{isImportingDrop ? uiText("ui.ce5c1c3f264d0643") : uiText("ui.daed1ac225487b5d")}</p>
						<p class="text-xs text-fg/60 max-w-sm text-center">{isImportingDrop ? uiText("ui.cc81a4152cf3d83b") : uiText("ui.2f9ee9f26673dcd3")}</p>
					</div>
				{/if}
                {#if subTab === "mods"}
                <section class="rounded-2xl border border-border bg-bg-elevated overflow-hidden">
                    <div class="flex flex-wrap items-center justify-between gap-3 p-5 border-b border-border">
                        <div>
                            <h2 class="text-base font-semibold text-fg">{uiText("home.installedMods")}</h2>
                            <p class="mt-1 text-xs text-fg-muted">{uiText("ui.38f7365f2d19005d")} {activeProfile?.name ?? uiText("ui.9c95cb0e794ba1f0")}.</p>
                        </div>
                        <a href={contentCatalogUrl} class={button({ variant: 'primary', size: 'sm' })}><Download class="h-4 w-4" />{uiText("ui.491774c8d38fada1")}</a>
                    </div>
                    <div class="flex flex-wrap items-center gap-2 p-4 border-t border-border">
                        <div class="relative min-w-48 flex-1">
                            <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-fg-subtle pointer-events-none" />
                            <input type="search" aria-label={uiText("ui.1bae8d6a32c4bf2f")} bind:value={searchQuery} placeholder={uiText("ui.958087c3687d84e7")} class="w-full h-10 rounded-xl border border-border bg-bg-subtle pl-10 pr-3 text-xs text-fg placeholder:text-fg-subtle focus:outline-none focus:border-brand-400" />
                        </div>
                        <button type="button" class={button({ variant: 'secondary', size: 'sm' })} onclick={handleOpenModsFolder}><FolderOpen class="h-4 w-4" />{uiText("screenshots.openFolderBtn")}</button>
                        <button type="button" class={button({ variant: 'secondary', size: 'sm' })} onclick={handleCheckModUpdates} disabled={isCheckingUpdates || isLoadingData}><RefreshCw class="h-4 w-4 {isCheckingUpdates ? 'animate-spin' : ''}" />{isCheckingUpdates ? 'Verificando…' : uiText("mods.checkUpdates")}</button>
                        <button type="button" class={button({ variant: 'secondary', size: 'sm' })} disabled={isImportingDrop} onclick={handleAddModFile}><Plus class="h-4 w-4" />{uiText("ui.21ce7cc4e47f9c45")}</button>
                    </div>
                    <div class="flex flex-wrap items-center justify-between gap-2 px-4 pb-4 text-xs text-fg-muted" aria-live="polite">
                        <span>{isLoadingData ? uiText("ui.318209e40ba847c9") : `${filteredMods.length} mods · ${resourcePacks.length + shaderPacks.length + dataPacks.length} pacotes`} · {enabledModCount} {uiText("ui.089963539b553aec")}</span>
                        <span>{uiText("ui.0129b013747c67a1")}</span>
                    </div>
                </section>
                {#if selectedModNames.length > 0}
                    <div class="flex flex-wrap items-center gap-2 rounded-xl border border-brand-500/20 bg-brand-500/5 px-4 py-3">
                        <span class="text-xs font-semibold text-fg mr-auto">{selectedModNames.length} {uiText("ui.b86f90bb0eaec7af")}</span>
                        <button type="button" class={button({ variant: 'secondary', size: 'sm' })} disabled={isChangingMods} onclick={() => toggleSelectedMods(true)}><ToggleRight class="h-4 w-4" />{uiText("ui.631205572827d2f7")}</button>
                        <button type="button" class={button({ variant: 'secondary', size: 'sm' })} disabled={isChangingMods} onclick={() => toggleSelectedMods(false)}><ToggleLeft class="h-4 w-4" />{uiText("ui.67afbeec3df01c59")}</button>
                        <button type="button" class={button({ variant: 'ghost', size: 'sm' })} onclick={() => selectedModNames = []}>{uiText("ui.33a042c0fd071ccc")}</button>
                    </div>
                {/if}


					{#if shieldResult && !shieldResult.isClean}
						<div class="bg-red-500/15 border border-red-500/40 rounded-2xl p-4 flex items-center justify-between gap-3 mb-4 text-red-200 shadow-md">
							<div class="flex items-center gap-3">
								<div class="w-9 h-9 rounded-xl bg-red-500/20 text-red-400 border border-red-500/30 flex items-center justify-center shrink-0">
									<ShieldAlert class="w-5 h-5" />
								</div>
								<div>
									<div class="text-xs font-black text-red-100 flex items-center gap-2">
										{uiText("ui.1154eb1419b0554c")} {shieldResult.threats.length} {uiText("ui.359a77d94bcc8b33")}
									</div>
									<div class="text-[11px] text-red-200/80 mt-0.5">
										{uiText("ui.16eb14fc759a4af8")}
									</div>
								</div>
							</div>
						</div>
					{/if}

					{#if availableModUpdates.length > 0}
						<div class="bg-gradient-to-r from-amber-500/15 via-orange-500/10 to-transparent border border-amber-500/30 p-4 rounded-2xl flex flex-col sm:flex-row sm:items-center justify-between gap-3 mb-4 shadow-lg shadow-amber-500/5">
							<div class="flex items-center gap-3">
								<div class="w-10 h-10 rounded-xl bg-amber-500/20 border border-amber-500/40 flex items-center justify-center text-amber-400 shrink-0">
									<ArrowUpCircle class="w-5 h-5 animate-pulse" />
								</div>
								<div>
									<h4 class="text-xs font-black text-fg uppercase tracking-wider">
										{availableModUpdates.length} {uiText("ui.2fe83d132e3db243")}{availableModUpdates.length > 1 ? 's' : ''} {uiText("ui.fd1ba6017af56df1")}
									</h4>
									<p class="text-[11px] text-fg-muted">
										{uiText("ui.c887f1209d82794b")}
									</p>
								</div>
							</div>
							<button
								type="button"
								class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2 disabled:opacity-50 shrink-0" })}
								disabled={isUpdatingAllMods}
								onclick={handleUpdateAllMods}
							>
								<RefreshCw class="w-4 h-4 {isUpdatingAllMods ? 'animate-spin' : ''}" />
								{isUpdatingAllMods ? 'Atualizando Mods...' : uiText("ui.b756d6a044e058bd", {arg0: (availableModUpdates.length)})}
							</button>
						</div>
					{/if}

                    {#if instanceMods.length > 0 && filteredMods.length === 0}
                        <p class="py-6 text-center text-sm text-fg-muted">{uiText("ui.96eec50d7e6f654e")}</p>
                    {:else if instanceMods.length > 0}
						<div class="instance-mod-row items-center gap-4 px-4 py-3 border-b border-fg/10 text-xs font-semibold text-fg/50 select-none bg-bg-elevated/80 rounded-t-2xl">
							<div class="flex items-center gap-3">
								<input
									type="checkbox"
									checked={allModsSelected}
                                    aria-label={uiText("ui.ee74c55ed89254ee")}
									onchange={toggleSelectAllMods}
									class="w-4 h-4 rounded border border-fg/20 bg-bg-subtle accent-emerald-500 cursor-pointer"
								/>
								<span>{uiText("ui.4c7877d7558fe41d")}</span>
							</div>
							<div class="text-left font-medium">{uiText("instances.sortVersion")}</div>
							<div class="text-right font-medium pr-2">{uiText("instances.actions")}</div>
						</div>

						<VirtualList items={filteredMods} getKey={(mod) => mod.name} itemHeight={76} fixedHeight overscan={2} height="600px" class="rounded-b-2xl border-x border-b border-fg/5 bg-bg-elevated/40">
							{#snippet children(mod: FileTreeEntry, _index: number)}
								{@const isDisabled = mod.name.endsWith('.disabled')}
								{@const info = modRowInfo.get(mod.name)}
								{@const displayName = info?.displayName ?? mod.name}
								{@const badge = info?.badge ?? { initials: "MD", theme: "" }}
								{@const isFrozen = frozenModSet.has(mod.name.replace(/\.disabled$/, ''))}
								{@const isSelected = selectedModSet.has(mod.name)}
								{@const modUpdate = info?.update ?? null}
								<div class="instance-mod-row h-full items-center gap-4 px-4 py-3 hover:bg-fg/[0.03] border-b border-fg/5 transition-colors group {isDisabled ? 'opacity-50' : ''}">
									<!-- Checkbox & Project Info -->
									<div class="flex items-center gap-3.5 min-w-0">
										<input
											type="checkbox"
											checked={isSelected}
                                            aria-label={uiText("ui.cdcc8c2293d065b1", {arg0: (displayName)})}
											onchange={() => toggleModSelection(mod.name)}
											class="w-4 h-4 rounded border border-fg/20 bg-bg-subtle accent-emerald-500 cursor-pointer shrink-0"
										/>
										<div use:loadContentIcon={{entry: mod, folder: "mods"}} class="w-10 h-10 rounded-xl overflow-hidden flex items-center justify-center shrink-0 bg-bg-subtle border border-fg/10 relative shadow-sm {isDisabled ? 'grayscale opacity-60' : ''}">
											<div class="absolute inset-0 bg-bg-subtle text-fg-muted flex items-center justify-center select-none">
												<span class="text-xs font-black tracking-tight drop-shadow-sm">{badge.initials}</span>
											</div>
											{#if mod.icon}
												<img decoding="async"
													src={mod.icon}
													alt={displayName}
													loading="eager"
													class="relative z-10 w-full h-full object-contain"
													onload={(e) => { (e.currentTarget as HTMLImageElement).style.visibility = "visible"; }} onerror={(e) => { (e.currentTarget as HTMLImageElement).style.visibility = "hidden"; }}
												/>
											{/if}
										</div>
										<div class="min-w-0">
											<div class="flex items-center gap-2">
												<h5 class="text-xs font-bold text-fg truncate max-w-[280px] group-hover:text-emerald-400 transition-colors" title={displayName}>
													{displayName}
												</h5>
												{#if isFrozen}<span title={uiText("ui.b37959a62adebf5c")}><Snowflake class="w-3.5 h-3.5 text-cyan-400 shrink-0" /></span>{/if}
												{#if modUpdate}
													<span class="text-[8px] font-black uppercase px-1.5 py-0.5 rounded-full bg-amber-500/15 text-amber-400 border border-amber-500/30 animate-pulse" title={uiText("ui.b9ef98f7cd528e18", {arg0: (modUpdate.latestVersionNumber)})}>
														v{modUpdate.latestVersionNumber}
													</span>
												{/if}
											</div>
											<div class="flex items-center gap-1.5 mt-0.5 text-[11px] text-fg/40">
												<span class="w-1.5 h-1.5 rounded-full bg-fg/30"></span>
												<span class="truncate max-w-[200px]">{info?.author ?? ''}</span>
											</div>
										</div>
									</div>


									<!-- Versão -->
									<div class="min-w-0">
										<div class="flex items-center gap-1.5">
											<span class="text-xs font-bold text-fg font-mono">{info?.version ?? ''}</span>
										</div>
										<div class="text-[10px] text-fg/40 font-mono truncate max-w-[160px]" title={mod.name}>
											{mod.name.replace('.disabled', '')}
										</div>
									</div>

									<div class="flex items-center gap-2 justify-end relative shrink-0">
										<button
											type="button"
											onclick={() => handleSwapVersion(mod)}
											class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
											title={uiText("ui.3a14ec141d94c628")}
										>
											<ArrowLeftRight class="w-4 h-4" />
										</button>

										<button
											type="button"
											onclick={() => handleToggleMod(mod)}
											class="w-10 h-5 shrink-0 rounded-full transition-colors relative cursor-pointer {isDisabled ? 'bg-fg/20' : 'bg-emerald-500'}"
											title={isDisabled ? uiText("ui.271352f466c82af2") : uiText("ui.c8452ddeb7bbcd26")}
											role="switch"
											aria-checked={!isDisabled}
                                            aria-label={`${isDisabled ? uiText("ui.631205572827d2f7") : uiText("ui.67afbeec3df01c59")} ${displayName}`}
                                            disabled={isChangingMods}
										>
											<span class="absolute top-0.5 w-4 h-4 rounded-full bg-white transition-all shadow-md {isDisabled ? 'left-0.5' : 'left-[22px]'}"></span>
										</button>

										<button
											type="button"
											onclick={() => handleDeleteMod(mod)}
											class={launcherButton({ variant: "ghostDanger", size: "icon", class: "" })}
											title={uiText("ui.fd78244374fec86b")} aria-label={uiText("ui.a03c3b676aa5e5d1", {arg0: (displayName)})}
										>
											<Trash2 class="w-4 h-4" />
										</button>

										<div class="relative">
											<button
												type="button"
												onclick={(e) => {
													e.stopPropagation();
													const viewport = e.currentTarget.closest('[data-virtual-list]');
													modMenuAbove = !!viewport && e.currentTarget.getBoundingClientRect().bottom + 210 > viewport.getBoundingClientRect().bottom;
													activeMenuMod = activeMenuMod === mod.name ? null : mod.name;
												}}
												class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
												title={uiText("ui.5d951376c42f3cf3")}
											>
												<MoreVertical class="w-4 h-4" />
											</button>

											{#if activeMenuMod === mod.name}
												<div
													class="absolute right-0 {modMenuAbove ? 'bottom-full mb-1.5' : 'top-full mt-1.5'} w-44 rounded-2xl bg-bg-elevated border border-fg/10 p-1.5 shadow-2xl z-30 space-y-0.5"
													transition:scale={{ easing: backOut, duration: 180, start: 0.95 }}
												>
													<button
														type="button"
														onclick={() => {
															activeMenuMod = null;
															handleSyncMod(mod);
														}}
														class={launcherButton({ variant: "secondary", size: "sm", class: "w-full flex items-center gap-2 text-left" })}
													>
														<RefreshCw class="w-3.5 h-3.5 text-emerald-400" /> {uiText("ui.87628281a7e1468e")}
													</button>
													<button
														type="button"
														onclick={() => {
															activeMenuMod = null;
															handleShowModFile(mod);
														}}
														class={launcherButton({ variant: "secondary", size: "sm", class: "w-full flex items-center gap-2 text-left" })}
													>
														<FolderOpen class="w-3.5 h-3.5 text-blue-400" /> {uiText("ui.364b8b026f67b5d6")}
													</button>
													<button
														type="button"
														onclick={() => {
															activeMenuMod = null;
															handleCopyModLink(mod);
														}}
														class={launcherButton({ variant: "secondary", size: "sm", class: "w-full flex items-center gap-2 text-left" })}
													>
														<Link class="w-3.5 h-3.5 text-amber-400" /> {uiText("ui.411f814c83c97626")}
													</button>
													<button
														type="button"
														onclick={() => {
															activeMenuMod = null;
															handleToggleFreeze(mod);
														}}
														class={launcherButton({ variant: "secondary", size: "sm", class: "w-full flex items-center gap-2 text-left" })}
													>
														<Snowflake class="w-3.5 h-3.5 text-cyan-400" /> {isFrozen ? uiText("ui.f44db915d1c4f82a") : uiText("ui.384f1f4bf26edf60")}
													</button>
												</div>
											{/if}
										</div>
									</div>
								</div>
							{/snippet}
						</VirtualList>
					{/if}
                {/if}
                {#each packGroups.filter(group => group.id === subTab) as group (group.id)}
                    {@const packs = normalizedSearch ? group.items.filter(pack => pack.name.toLowerCase().includes(normalizedSearch)) : group.items}
                    <section class="overflow-hidden rounded-2xl border border-border bg-bg-elevated" aria-label={group.label}>
                        <header class="flex flex-wrap items-center gap-3 p-4 border-border" class:border-b={packs.length > 0}>
                            <group.icon class="h-5 w-5 text-brand-400" /><h3 class="text-sm font-semibold text-fg">{group.label}</h3><span class="content-count text-xs text-fg-muted">{group.items.length}</span>
                            {#if group.id === "datapacks"}<select aria-label={uiText("files.saves")} bind:value={datapackWorld} onchange={() => void loadSection("datapacks", true)} class="max-w-56 rounded-xl border border-border bg-bg-elevated px-3 py-2 text-sm text-fg">{#if !worldsList.length}<option value="">{uiText("files.saves")}: 0</option>{/if}{#each worldsList as world}<option value={world.folderName}>{world.folderName}</option>{/each}</select>{/if}
                            <div class="ml-auto flex flex-wrap gap-2">
                                <a class={button({ variant: 'ghost', size: 'sm' })} href={`/mods?instance=${encodeURIComponent(instanceId)}&type=${group.id === 'resourcepacks' ? 'resourcepack' : group.id === 'shaders' ? 'shader' : 'datapack'}`}><Download class="h-4 w-4" />{uiText("ui.aa56664d77c69c69")}</a>
                                <button type="button" class={button({ variant: 'secondary', size: 'sm' })} onclick={() => handleOpenPackFolder(group.type)}><FolderOpen class="h-4 w-4" />{uiText("screenshots.openFolderBtn")}</button>
                                <button type="button" class={button({ variant: 'secondary', size: 'sm' })} onclick={() => handleAddResourcePack(group.type)}><Plus class="h-4 w-4" />{uiText("ui.e517797b6ffe1f2c")}</button>
                            </div>
                        </header>
                        {#if packs.length > 0}
                            <div class="grid gap-2 p-3 sm:grid-cols-2">
                                {#each packs as pack (pack.name)}
                                    <div class="flex min-w-0 items-center gap-3 rounded-xl border border-border p-3">
                                        <div use:loadContentIcon={{entry: pack, folder: group.id === "shaders" ? "shaderpacks" : group.id === "datapacks" ? `saves/${datapackWorld}/datapacks` : "resourcepacks"}} class="relative flex h-12 w-12 shrink-0 items-center justify-center overflow-hidden rounded-xl border border-border bg-bg-subtle">
                                            <group.icon class="absolute h-5 w-5 text-fg-muted" />
                                            {#if pack.icon}<img src={pack.icon} alt={pack.name} loading="eager" decoding="async" class="relative h-full w-full object-contain" onload={(event) => (event.currentTarget as HTMLImageElement).style.visibility = 'visible'} onerror={(event) => (event.currentTarget as HTMLImageElement).style.visibility = 'hidden'} />{/if}
                                        </div>
                                        <div class="min-w-0 flex-1"><p class="truncate text-xs font-semibold text-fg" title={pack.name}>{pack.name}</p><p class="mt-1 text-[10px] text-fg-muted">{(pack.size / 1048576).toFixed(2)} MB</p></div>
                                        <button type="button" class={button({ variant: 'ghost', size: 'icon' })} aria-label={`${uiText("screenshots.openFolderBtn")}: ${pack.name}`} title={uiText("screenshots.openFolderBtn")} onclick={() => handleOpenPackFolder(group.type)}><FolderOpen class="h-4 w-4" /></button>
                                        <button type="button" class={button({ variant: 'ghostDanger', size: 'icon' })} aria-label={uiText("ui.a03c3b676aa5e5d1", {arg0: (pack.name)})} onclick={() => handleDeletePack(pack.name, group.type)}><Trash2 class="h-4 w-4" /></button>
                                    </div>
                                {/each}
                            </div>
                        {/if}
                    </section>
                {/each}

			</div>

        </div>
        {:else if mainTab === "mundos"}
        <div use:pageMotion={mainTab} use:loadWhenVisible={{ id: instanceId, section: "mundos" }} id="instance-mundos" role="tabpanel" aria-labelledby="instance-tab-mundos" class="instance-overview-section" aria-label={uiText("ui.9ac3a6eca765a04a")}>
			<div class="space-y-4">
				<div class="flex items-center justify-between flex-wrap gap-2">
					<h3 class="text-sm font-bold text-fg">{uiText("ui.5df9580dc4ce92a4")}</h3>
					<div class="flex items-center flex-wrap gap-2">
						<button
							type="button"
							class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-1.5 disabled:opacity-50" })}
							onclick={handleImportWorld}
							disabled={isImportingWorld}
							title={uiText("ui.ef346900326a3128")}
						>
							<Download class="w-3.5 h-3.5" />
							<span>{isImportingWorld ? uiText("ui.eeb8c8bbcaab66e1") : uiText("ui.5825de6edcc4b718")}</span>
						</button>
						<button
							type="button"
							class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5 disabled:opacity-50" })}
							onclick={handleImportWorldFolder}
							disabled={isImportingWorld}
							title={uiText("ui.cf689b540d90f342")}
						>
							<FolderPlus class="w-3.5 h-3.5 text-amber-400" />
							<span>{uiText("ui.88d40f96b904a0b4")}</span>
						</button>
						<button
							type="button"
							class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })}
							onclick={() => showWorldBackupModal = true}
							title={uiText("ui.521b761de5247db8")}
						>
							<HardDrive class="w-3.5 h-3.5 text-cyan-400" /> {uiText("ui.50c6370959cfedfa")}{worldsList.length})
						</button>
						<button class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })} onclick={openInstanceFolder}>
							<FolderOpen class="w-3.5 h-3.5 text-emerald-400" /> {uiText("ui.0cac89af486b57d0")}
						</button>
					</div>
				</div>

				{#if worldsList.length === 0}
					<div class="bg-bg-elevated border border-fg/5 rounded-3xl p-16 flex flex-col items-center justify-center text-center gap-4">
						<Globe2 class="w-12 h-12 text-fg/20" />
						<div>
							<h4 class="text-sm font-bold text-fg">{uiText("ui.ffb9bcfef0dec831")}</h4>
							<p class="text-xs text-fg/40 mt-1">{uiText("ui.38175563e63f18a2")}</p>
						</div>
						<div class="flex items-center gap-2 mt-2">
							<button
								type="button"
								class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-2" })}
								onclick={handleImportWorld}
							>
								<Download class="w-4 h-4" /> {uiText("ui.5825de6edcc4b718")}
							</button>
							<button
								type="button"
								class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2" })}
								onclick={handleImportWorldFolder}
							>
								<FolderPlus class="w-4 h-4 text-amber-400" /> {uiText("ui.88d40f96b904a0b4")}
							</button>
						</div>
					</div>
				{:else}
					<div class="grid grid-cols-1 md:grid-cols-2 gap-3.5">
						{#each worldsList as world (world.folderName)}
							<div class="bg-bg-elevated border border-fg/5 p-4 rounded-2xl flex flex-col justify-between hover:border-fg/15 transition-[color,background-color,border-color,box-shadow,transform,opacity] group gap-3 ">
								<div class="flex items-start gap-3.5 min-w-0">
									<div class="h-14 w-14 rounded-xl bg-bg-overlay/40 border border-fg/10 overflow-hidden flex items-center justify-center shrink-0 shadow-md">
										{#if world.iconBase64}
											<img loading="lazy" decoding="async" src={world.iconBase64} alt={world.name} class="w-full h-full object-cover [image-rendering:pixelated]" />
										{:else}
											<img loading="lazy" decoding="async" src="/grass_block.png" alt={uiText("ui.43c6d4bb0a0dbcff")} class="w-8 h-8 object-contain drop-shadow" />
										{/if}
									</div>
									<div class="min-w-0 flex-1">
										<div class="flex items-center gap-2">
											<h5 class="text-xs font-bold text-fg truncate">{world.name}</h5>
											{#if world.hardcore}
												<span class="px-1.5 py-0.5 rounded text-[9px] font-black bg-red-500/20 text-red-400 border border-red-500/30">{uiText("ui.cd67305f256ee549")}</span>
											{/if}
										</div>
										<div class="flex flex-wrap items-center gap-1.5 mt-1 text-[10px] text-fg/50">
											<span class="text-emerald-400 font-semibold">{world.gameMode || uiText("ui.c7fb9a1ec7e64301")}</span>
											<span>·</span>
											<span>{(world.sizeBytes / (1024 * 1024)).toFixed(1)} MB</span>
											{#if world.versionName}
												<span>·</span>
												<span class="font-mono text-fg/40">{world.versionName}</span>
											{/if}
										</div>

										<div class="flex flex-wrap items-center gap-2 mt-2 pt-2 border-t border-fg/5 text-[10px] font-mono">
											{#if world.seed != null}
												<button
													type="button"
													class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1" })}
													onclick={() => {
														if (world.seed != null) {
															navigator.clipboard.writeText(world.seed.toString());
															toast(`Seed copiada: ${world.seed}`, "success");
														}
													}}
													title={uiText("ui.92051bdae9a22112")}
												>
													<Compass class="w-3 h-3 text-brand-500" />
													<span>{uiText("ui.b784ffe8db315c0a")} {world.seed}</span>
													<Copy class="w-2.5 h-2.5 opacity-50" />
												</button>
											{/if}
											{#if world.spawnX != null && world.spawnZ != null}
												<div class="flex items-center gap-1 text-fg/40" title={uiText("ui.159b06a44a0d05ce")}>
													<MapPin class="w-3 h-3 text-emerald-400" />
													<span>{uiText("ui.ea818c2605b6e30f")} {world.spawnX}, {world.spawnY ?? 64}, {world.spawnZ}</span>
												</div>
											{/if}
											{#if world.dayCount != null}
												<span class="text-amber-300/70">{uiText("ui.8ed570950d8312cd")} {world.dayCount}</span>
											{/if}
											{#if world.playerHealth != null}
												<span class="text-rose-400 font-semibold">❤️ {Math.round(world.playerHealth)}/20</span>
											{/if}
											{#if world.playerLevel != null && world.playerLevel > 0}
												<span class="text-emerald-300 font-semibold">{uiText("ui.6c82b07ec01cb52e")} {world.playerLevel}</span>
											{/if}
										</div>

										{#if world.playerInventory && world.playerInventory.length > 0}
											<div class="flex flex-wrap items-center gap-1.5 mt-2 pt-2 border-t border-fg/5">
												<span class="text-[9px] uppercase tracking-wider text-fg/40 font-bold mr-1">{uiText("ui.edf17c7751526329")}</span>
												{#each world.playerInventory.slice(0, 9) as item}
													<span class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-bg-overlay/40 border border-fg/10 text-[10px] font-mono text-fg/80" title={`Slot ${item.slot}: ${item.id} (x${item.count})`}>
														<span class="text-brand-400 font-bold">{item.id.replace(/^minecraft:/, '')}</span>
														{#if item.count > 1}
															<span class="text-[9px] px-1 rounded bg-brand-500/20 text-brand-300 font-bold">x{item.count}</span>
														{/if}
													</span>
												{/each}
												{#if world.playerInventory.length > 9}
													<span class="text-[9px] text-fg/40 font-mono">+{world.playerInventory.length - 9}</span>
												{/if}
											</div>
										{/if}
									</div>
								</div>

								<div class="flex items-center justify-between pt-1 border-t border-fg/5">
									<div class="flex items-center gap-1.5">
										<button
											type="button"
											class={launcherButton({ variant: "ghostBrand", size: "sm", class: "flex items-center gap-1.5" })}
											onclick={() => selectedSnapshotWorld = { name: world.name, folder: world.folderName }}
											title={uiText("ui.5b46d1550e2e65ae")}
										>
											<Archive class="w-3 h-3 text-brand-500" />
											<span>{uiText("ui.f187f78e07efb26e")} {world.snapshotsCount != null && world.snapshotsCount > 0 ? `(${world.snapshotsCount})` : ''}</span>
										</button>
										<button
											type="button"
											class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })}
											onclick={openHostWorldModal}
											title={uiText("ui.d5bb9c1b94510b4a")}
										>
											<Radio class="w-3 h-3 text-purple-400" />
											<span>{uiText("ui.79366e5761e99168")}</span>
										</button>
									</div>

									<div class="flex items-center gap-2">
										<button
											type="button"
											class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-1.5" })}
											onclick={() => handlePlay()}
											title={uiText("ui.0354cb2b38d6235c")}
										>
											<Play class="w-3 h-3 fill-current" /> {uiText("instances.play")}
										</button>
										<button
											type="button"
											class={launcherButton({ variant: "ghostDanger", size: "icon", class: "" })}
											onclick={() => handleDeleteWorld(world.folderName)}
											title={uiText("ui.36b29bbcb53deb0e")}
										>
											<Trash2 class="w-3.5 h-3.5" />
										</button>
									</div>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>

		</div>
        {:else if mainTab === "galeria"}
        <div use:pageMotion={mainTab} use:loadWhenVisible={{ id: instanceId, section: "galeria" }} id="instance-galeria" role="tabpanel" aria-labelledby="instance-tab-galeria" class="instance-overview-section" aria-label={uiText("ui.c66873c6ffa75714")}>
			<div class="space-y-4">
				<div class="flex items-center justify-between">
					<h3 class="text-sm font-bold text-fg">{uiText("ui.2cbb69c6cee5108c")}</h3>
					<button class={launcherButton({ variant: "ghost", size: "sm", class: "hover:underline flex items-center gap-1" })} style="color: rgb(var(--brand-500));" onclick={openScreenshotsFolder}>
						<FolderOpen class="w-3.5 h-3.5" /> {uiText("ui.c8afa99d42ab9309")}
					</button>
				</div>

				{#if screenshotsList.length === 0}
					<div class="bg-bg-elevated border border-fg/5 rounded-3xl p-16 flex flex-col items-center justify-center text-center">
						<Image class="w-12 h-12 text-fg/20 mb-3" />
						<h4 class="text-sm font-bold text-fg">{uiText("ui.7758915dd2dea905")}</h4>
						<p class="text-xs text-fg/40 mt-1">{uiText("ui.997499ecbd594d77")}</p>
					</div>
				{:else}
					<div class="columns-2 gap-4 xl:columns-3">
						{#each screenshotsList as shot (shot.path)}
							<div
								class="mb-4 break-inside-avoid bg-bg-elevated border border-fg/5 rounded-2xl overflow-hidden group relative cursor-pointer hover:border-brand-500/30 transition-[color,background-color,border-color,box-shadow,transform,opacity] shadow-soft "
								onclick={() => previewScreenshot = shot}
								role="button"
								tabindex="0"
								onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); previewScreenshot = shot; } }}
							>
								<div class="bg-bg-overlay/60 overflow-hidden">
									<img decoding="async"
										src={shot.thumbPath ? convertFileSrc(shot.thumbPath) : shot.dataUrl || convertFileSrc(shot.path)}
										alt={shot.name}
										class="w-full h-auto object-contain group-hover:scale-105 transition-transform duration-300"
										loading="lazy"
									/>
								</div>
								<div class="p-3 flex items-center justify-between bg-bg-elevated/90">
									<span class="text-xs font-medium text-fg truncate max-w-[80%]">{shot.name}</span>
									<button
										class={launcherButton({ variant: "ghostDanger", size: "icon", class: "" })}
										onclick={(e) => { e.stopPropagation(); handleDeleteScreenshot(shot.path); }}
										title={uiText("screenshots.deleteTitle")}
									>
										<Trash2 class="w-3.5 h-3.5" />
									</button>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>

		</div>
        {:else if mainTab === "ficheiros"}
        <div use:pageMotion={mainTab} use:loadWhenVisible={{ id: instanceId, section: "ficheiros" }} id="instance-ficheiros" role="tabpanel" aria-labelledby="instance-tab-ficheiros" class="instance-overview-section" aria-label={uiText("ui.99c85eb6aa2966e3")}>
			<div class="space-y-4">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						<h3 class="text-sm font-bold text-fg">{uiText("ui.9a52d147c628ff95")}</h3>
						<span class="text-[10px] text-fg/40 font-mono">({fileTree.length} {uiText("ui.01839461d49555e3")}</span>
					</div>
					<button class={launcherButton({ variant: "ghost", size: "sm", class: "hover:underline flex items-center gap-1" })} style="color: rgb(var(--brand-500));" onclick={openInstanceFolder}>
						<FolderOpen class="w-3.5 h-3.5" /> {runtimePlatform.isWindows ? uiText("ui.ecc618681b1eadd0") : uiText("screenshots.openFolderBtn")}
					</button>
				</div>

				<div class="bg-bg-elevated border border-fg/10 rounded-2xl p-3 flex items-center justify-between gap-3 shadow-md">
					<div class="flex items-center gap-1 text-xs font-mono overflow-x-auto custom-scrollbar py-0.5">
						<button
							type="button"
							class={launcherButton({ variant: "secondary", size: "sm", class: "shrink-0" })}
							onclick={() => navigateBreadcrumb(-1)}
						>
							{uiText("ui.aec5f2db004d631f")}
						</button>
						{#each fileBreadcrumbs as seg, idx}
							<ChevronRight class="w-3.5 h-3.5 text-fg/30 shrink-0" />
							<button
								type="button"
								class="text-xs font-bold px-2.5 py-1 rounded-full transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer shrink-0 {idx === fileBreadcrumbs.length - 1 ? 'bg-fg/10 text-fg' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}"
								onclick={() => navigateBreadcrumb(idx)}
							>
								{seg}
							</button>
						{/each}
					</div>

					<div class="flex items-center gap-1 shrink-0">
						{#if fileBreadcrumbs.length > 0}
							<button
								type="button"
								class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
								title={uiText("ui.c73dd525d1706a85")}
								onclick={navigateUp}
							>
								<ArrowUp class="w-4 h-4" />
							</button>
						{/if}
						<button
							type="button"
							class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
							title={uiText("ui.65532457b454b2c6")}
							onclick={refreshAllData}
						>
							<RefreshCw class="w-4 h-4 {isLoadingData ? 'animate-spin' : ''}" />
						</button>
					</div>
				</div>

				<div class="bg-bg-elevated border border-fg/5 rounded-2xl p-2 space-y-1 shadow-md max-h-[500px] overflow-y-auto custom-scrollbar">
					{#if fileTree.length === 0}
						<div class="text-xs text-fg/40 py-8 text-center">{uiText("ui.04b603403f5f1b60")}</div>
					{:else}
						{#each fileTree as file (file.path)}
							<div
								class="flex items-center justify-between p-2.5 hover:bg-fg/5 rounded-xl text-xs transition-colors group cursor-pointer "
								onclick={() => handleOpenFile(file)}
								role="button"
								tabindex="0"
								onkeydown={(e) => { if (e.key === 'Enter') handleOpenFile(file); }}
							>
								<div class="flex items-center gap-3 min-w-0">
									{#if file.isDir}
										<Folder class="w-4 h-4 text-amber-400 shrink-0" />
									{:else}
										<FileText class="w-4 h-4 text-fg/40 shrink-0 group-hover:text-emerald-400 transition-colors" />
									{/if}
									<span class="font-medium text-fg truncate">{file.name}</span>
								</div>

								<div class="flex items-center gap-3 shrink-0">
									<span class="text-[10px] text-fg/30 font-mono">
										{file.isDir ? uiText("ui.518e479153a320df") : `${Math.round(file.size / 1024)} KB`}
									</span>
									<button
										type="button"
										class={launcherButton({ variant: "danger", size: "icon", class: "opacity-0 group-hover:opacity-100" })}
										onclick={(e) => { e.stopPropagation(); handleDeleteFileEntry(file); }}
										title={uiText("screenshots.deleteBtn")}
									>
										<Trash2 class="w-3.5 h-3.5" />
									</button>
								</div>
							</div>
						{/each}
					{/if}
				</div>
			</div>
        </div>

        {:else if mainTab === "configuracoes"}
        <div use:pageMotion={mainTab} use:loadWhenVisible={{ id: instanceId, section: "configuracoes" }} id="instance-configuracoes" role="tabpanel" aria-labelledby="instance-tab-configuracoes" class="instance-overview-section">
            <section aria-label={uiText("ui.6c5b8d86c6726795")} class="rounded-2xl border border-border bg-bg-elevated p-5 sm:p-6 space-y-6">
            <header class="relative shrink-0 overflow-hidden rounded-2xl border border-border bg-bg-subtle">
                <img loading="lazy" decoding="async" src={instanceBanner || heroBanner} alt="" class="absolute inset-0 h-full w-full object-cover opacity-30" />
                <div class="relative flex items-center gap-4 bg-gradient-to-r from-bg-elevated via-bg-elevated/80 to-transparent p-5">
                    <img loading="lazy" decoding="async"
                        src={settingsIcon}
                        alt=""
                        class="h-14 w-14 rounded-xl object-contain border border-fg/10 bg-bg-elevated p-1"
                        onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/grass_block.png'; }}
                    />
                    <div class="min-w-0 flex-1"><p class="text-xs text-brand-400 font-semibold">{uiText("ui.6c5b8d86c6726795")}</p><h2 class="mt-1 text-xl font-bold text-fg truncate">{instanceNameInput}</h2><p class="mt-1 text-xs text-fg-muted">Minecraft {activeProfile?.mcVersion} {uiText("ui.0347a27f4ae5be7b")}</p></div>

                </div>
            </header>
			<div inert={savingInstanceSettings} class="space-y-6">

				<div class="min-w-0 flex-1 space-y-6">

					<section id="instance-settings-geral" class="instance-settings-group">
						<div class="space-y-6">
							<div>
								<div class="flex items-center gap-2">
									<Box class="w-4 h-4 text-emerald-400" />
									<h3 class="text-xs font-bold text-fg uppercase tracking-wider">{uiText("settings.general")}</h3>
								</div>
								<p class="text-[11px] text-fg/40 mt-0.5">{uiText("ui.d5de5a5144fadbfa")}</p>
							</div>

							<div class="space-y-3">
								<span class="text-xs font-bold text-fg/70 block">{uiText("instances.name")}</span>
								<div class="flex items-center gap-4">
									<div class="h-14 w-14 rounded-2xl bg-bg-elevated border border-fg/10 flex items-center justify-center shrink-0 p-1">
										<img loading="lazy" decoding="async" src={settingsIcon} alt={uiText("ui.e76907efa549eca8")} class="w-10 h-10 rounded-lg object-contain" />
									</div>
									<input
										type="text"
										aria-label={uiText("instances.namePlaceholder")} bind:value={instanceNameInput}
										class="flex-1 bg-bg-elevated border border-fg/10 rounded-2xl px-4 py-3 text-xs font-bold text-fg outline-none focus:border-brand-500 transition-colors"
									/>
								</div>
							</div>

							<label class="block space-y-2 text-xs text-fg-muted">
                                <span>{uiText("ui.94318b08bcae561f")}</span>
                                <input type="url" aria-label={uiText("ui.94318b08bcae561f")} bind:value={instanceBanner} placeholder="https://…/banner.webp" class="w-full rounded-xl border border-border bg-bg-elevated px-4 py-3 text-fg focus:border-brand-500" />
                            </label>
                            <div class="space-y-3 pt-2">
								<div>
									<span class="text-xs font-bold text-fg/80 block">{uiText("ui.064fc30a68b2488a")}</span>
									<p class="text-[11px] text-fg/40 mt-0.5">{uiText("ui.a22fca1bf720bf8b")}</p>
								</div>

								<button
									type="button"
									class={button({ variant: "secondary", class: "w-full h-auto justify-start gap-4 whitespace-normal p-4 text-left group border-amber-500/25" })}
									onclick={handleRepairModpack}
                                    disabled={isRepairingModpack}
								>
									<div class="h-10 w-10 rounded-xl bg-amber-500/20 border border-amber-500/40 flex items-center justify-center shrink-0 text-amber-400 group-hover:scale-110 transition-transform">
										<Sparkles class="w-5 h-5" />
									</div>
									<div>
										<h4 class="text-xs font-bold text-amber-400">{uiText("ui.598a4670601ddf97")}</h4>
										<p class="text-[11px] text-fg/40 mt-0.5">{uiText("ui.fb2ba1a357ce9b2d")}</p>
									</div>
								</button>

								<button
									type="button"
									class={button({ variant: "danger", class: "w-full h-auto justify-start gap-4 whitespace-normal p-4 text-left group" })}
									onclick={deleteCurrentInstance} disabled={deletingInstance || appState.isGameRunning}
								>
									<div class="h-10 w-10 rounded-xl bg-rose-500/20 border border-rose-500/40 flex items-center justify-center shrink-0 text-rose-400 group-hover:scale-110 transition-transform">
										<Trash2 class="w-5 h-5" />
									</div>
									<div>
										<h4 class="text-xs font-bold text-rose-400">{uiText("ui.79ef883cba2c401e")}</h4>
										<p class="text-[11px] text-fg/40 mt-0.5">{uiText("ui.58df83b9244fd69a")}</p>
									</div>
								</button>
							</div>
						</div>

					</section><section id="instance-settings-instalacao" class="instance-settings-group">
						<div class="space-y-6">
							<div>
								<h3 class="text-xs font-bold text-fg uppercase tracking-wider">{uiText("ui.c30103f0b0461371")}</h3>
								<p class="text-[11px] text-fg/40 mt-0.5">{uiText("ui.c77441d30550b560")}</p>
							</div>

							<div class="space-y-3">
								<span class="text-xs font-bold text-fg/70 block">{uiText("ui.afd4e4e867ce4d76")}</span>
								<div class="grid grid-cols-2 gap-3">
									{#each ["fabric", "forge", "neoforge", "quilt", "vanilla"] as loader}
										<button
											type="button"
											aria-pressed={instanceLoaderType === loader}
                                            class={button({ variant: instanceLoaderType === loader ? "primary" : "secondary", class: "justify-between" })}
											onclick={() => instanceLoaderType = loader}
										>
											<span class="capitalize">{loader}</span>
											{#if instanceLoaderType === loader}<Check class="w-4 h-4" />{/if}
										</button>
									{/each}
								</div>
							</div>

							<div class="space-y-2">
								<span class="text-xs font-bold text-fg/70 block">{uiText("ui.b927179f42538736")}</span>
								<input type="text" bind:value={instanceLoaderVersion} class="w-full bg-bg-elevated border border-fg/10 rounded-2xl px-4 py-2.5 text-xs text-fg font-mono outline-none focus:border-brand-500" />
							</div>
						</div>

					</section><section id="instance-settings-otimizacao" class="instance-settings-group">
						<div class="space-y-5">
							<div>
								<div class="flex items-center gap-2">
									<Zap class="w-4 h-4 text-emerald-400" />
									<h3 class="text-xs font-bold text-fg uppercase tracking-wider">{uiText("ui.a38614a12123f0eb")}</h3>
								</div>
								<p class="text-[11px] text-fg/40 mt-0.5">{uiText("ui.ae23252a007d7244")}</p>
							</div>

							<div class="bg-bg-elevated border border-fg/5 rounded-2xl p-4 space-y-3">
								<div class="flex items-center justify-between">
									<span class="text-xs font-bold text-fg/90 flex items-center gap-2">
										<Cpu class="w-3.5 h-3.5 text-brand-500" /> {uiText("ui.4c49e2c6fdf378cf")}
									</span>
									<span class="text-[10px] font-mono text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-md border border-emerald-500/20">
										{uiText("ui.1fc12aa36d25d2fa")}
									</span>
								</div>
								<div class="grid grid-cols-2 gap-2 text-xs">
									<div class="bg-bg-overlay/30 p-2.5 rounded-xl border border-fg/5">
										<span class="text-[10px] text-fg/40 block">{uiText("ui.d4b6159f5b365816")}</span>
										<span class="text-xs font-bold text-fg truncate block mt-0.5" title={gpuInfo?.renderer || 'Buscando...'}>
											{gpuInfo?.renderer || uiText("ui.eff6696501ec9ecd")}
										</span>
										<span class="text-[10px] text-brand-500 font-mono block mt-0.5">
											{uiText("ui.8bc4a5ebed894826")} {gpuInfo?.driver || uiText("ui.eff6696501ec9ecd")}
										</span>
									</div>
									<div class="bg-bg-overlay/30 p-2.5 rounded-xl border border-fg/5">
										<span class="text-[10px] text-fg/40 block">{uiText("ui.2bd57c28deb45064")}</span>
										<span class="text-xs font-bold text-fg block mt-0.5">
											{Math.round(systemRamMb / 1024)} {uiText("ui.1fdb78b55e9d4eb5")}
										</span>
										<span class="text-[10px] text-fg/40 font-mono block mt-0.5">
											{uiText("ui.80c56ce36ecb212b")} {(instanceRamMb / 1024).toFixed(1)} GB
										</span>
									</div>
								</div>
							</div>

							<div class="bg-bg-elevated border border-fg/5 rounded-2xl p-4 space-y-3">
								<div class="flex items-center justify-between">
									<div>
										<div class="flex items-center gap-2">
											<span class="text-xs font-bold text-fg">{uiText("ui.8ff821a4aea0985d")}</span>
											<span class="text-[9px] bg-brand-500/20 text-brand-500 px-1.5 py-0.5 rounded font-bold">{uiText("ui.2aa20679b65d4368")}</span>
										</div>
										<span class="text-[10px] text-fg/40 block mt-0.5">
											{uiText("ui.465353a6e568f0af")} {instanceRamMb} {uiText("ui.cd328bf785bc5db7")}
										</span>
									</div>
									<button
										type="button"
										role="switch" aria-checked={instanceAutoOptimize} aria-label={uiText("ui.2223c19072c10de7")}
										class="w-10 h-5 rounded-full transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-200 relative flex items-center px-0.5 cursor-pointer {instanceAutoOptimize ? 'bg-emerald-500 shadow-glow' : 'bg-bg-subtle'}"
										onclick={() => instanceAutoOptimize = !instanceAutoOptimize}
									>
										<span class="w-4 h-4 rounded-full bg-fg transition-transform duration-200 shadow-md {instanceAutoOptimize ? 'translate-x-5' : 'translate-x-0'}"></span>
									</button>
								</div>

								<div class="space-y-1.5">
									<div class="flex items-center justify-between text-[10px]">
										<span class="text-fg/50">{uiText("ui.602df047702d4df1")}</span>
										<span class="font-mono text-fg/30">{generatedAikarFlags.length} {uiText("ui.4069611a089d3052")}</span>
									</div>
									<div class="bg-bg-overlay/50 p-2.5 rounded-xl border border-fg/5 max-h-24 overflow-y-auto custom-scrollbar font-mono text-[10px] text-emerald-400 leading-relaxed break-all">
										{generatedAikarFlags.join(" ")}
									</div>
								</div>
							</div>

							<div class="bg-bg-elevated border border-fg/5 rounded-2xl p-4 flex items-center justify-between">
								<div>
									<div class="flex items-center gap-2">
										<span class="text-xs font-bold text-fg">{uiText("ui.7cde29d5642062fa")}</span>
										<span class="text-[9px] bg-cyan-500/20 text-cyan-400 px-1.5 py-0.5 rounded font-bold">{uiText("ui.b74b99727ed37f3a")}</span>
									</div>
									<span class="text-[10px] text-fg/40 block mt-0.5">
										{uiText("ui.fa9cc525f5941eec")}
									</span>
								</div>
								<button
									type="button"
									role="switch" aria-checked={instanceEnableVulkanOpt} aria-label={uiText("ui.13bfca46695c5ffe")}
									class="w-10 h-5 rounded-full transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-200 relative flex items-center px-0.5 cursor-pointer {instanceEnableVulkanOpt ? 'bg-emerald-500 shadow-glow' : 'bg-bg-subtle'}"
									onclick={() => instanceEnableVulkanOpt = !instanceEnableVulkanOpt}
								>
									<span class="w-4 h-4 rounded-full bg-fg transition-transform duration-200 shadow-md {instanceEnableVulkanOpt ? 'translate-x-5' : 'translate-x-0'}"></span>
								</button>
							</div>
						</div>

							<div class="bg-bg-elevated border border-fg/5 rounded-2xl p-4 space-y-3">
								<div><span class="text-xs font-bold text-fg">{uiText("ui.cf5346ca62930ed6")}</span><span class="mt-0.5 block text-[10px] text-fg/40">{uiText("ui.110aacf7b0065c8b")}</span></div>
								<div class="grid gap-2 sm:grid-cols-2">
{#if runtimePlatform.isLinux}
									<button type="button" class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center justify-between text-left" })} aria-pressed={instanceUseGameMode} onclick={() => instanceUseGameMode = !instanceUseGameMode}><span><span class="block font-bold text-fg">{uiText("ui.d0fba50ac6075a55")}</span><span class="block text-[10px] text-fg/40">{uiText("ui.3dbb98af2279bcb7")}</span></span><span class="h-2.5 w-2.5 rounded-full {instanceUseGameMode ? 'bg-emerald-400' : 'bg-fg/20'}"></span></button>
									<button type="button" class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center justify-between text-left" })} aria-pressed={instanceUseMangoHud} onclick={() => instanceUseMangoHud = !instanceUseMangoHud}><span><span class="block font-bold text-fg">{uiText("ui.b424331359cf27e5")}</span><span class="block text-[10px] text-fg/40">{uiText("ui.8cac59e97e95d402")}</span></span><span class="h-2.5 w-2.5 rounded-full {instanceUseMangoHud ? 'bg-emerald-400' : 'bg-fg/20'}"></span></button>
									<button type="button" class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center justify-between text-left" })} aria-pressed={instanceForceDedicatedGpu} onclick={() => instanceForceDedicatedGpu = !instanceForceDedicatedGpu}><span><span class="block font-bold text-fg">{uiText("ui.ed43d0e547b9621c")}</span><span class="block text-[10px] text-fg/40">{uiText("ui.e1f6fe8e2a2809b6")}</span></span><span class="h-2.5 w-2.5 rounded-full {instanceForceDedicatedGpu ? 'bg-emerald-400' : 'bg-fg/20'}"></span></button>
									{/if}
<button type="button" class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center justify-between text-left" })} aria-pressed={instanceForceFullVerification} onclick={() => instanceForceFullVerification = !instanceForceFullVerification}><span><span class="block font-bold text-fg">{uiText("ui.760daec3c6095582")}</span><span class="block text-[10px] text-fg/40">{uiText("ui.e5d1aaf3521787c0")}</span></span><span class="h-2.5 w-2.5 rounded-full {instanceForceFullVerification ? 'bg-emerald-400' : 'bg-fg/20'}"></span></button>
								</div>
								{#if runtimePlatform.isLinux}
<div class="flex items-center justify-between rounded-xl border border-fg/10 bg-bg-overlay/30 p-3"><div><span class="block text-xs font-bold text-fg">{uiText("ui.bbc209cb23b5013e")}</span><span class="block text-[10px] text-fg/40">{uiText("ui.f6bd97809dcbafe8")}</span></div><button type="button" role="switch" aria-checked={instanceUseGamescope} aria-label={uiText("ui.590148ebf7358843")} class="w-10 h-5 rounded-full px-0.5 {instanceUseGamescope ? 'bg-emerald-500' : 'bg-bg-subtle'}" onclick={() => instanceUseGamescope = !instanceUseGamescope}><span class="block h-4 w-4 rounded-full bg-fg transition-transform {instanceUseGamescope ? 'translate-x-5' : ''}"></span></button></div>
								{#if instanceUseGamescope}<div class="grid grid-cols-2 gap-2"><input type="number" bind:value={instanceGamescopeWidth} placeholder={uiText("ui.09b171027051a68c")} class="rounded-xl border border-fg/10 bg-bg-overlay/30 px-3 py-2 text-xs text-fg outline-none" /><input type="number" bind:value={instanceGamescopeHeight} placeholder={uiText("ui.3031040a64870128")} class="rounded-xl border border-fg/10 bg-bg-overlay/30 px-3 py-2 text-xs text-fg outline-none" /></div><label class="flex items-center gap-2 text-xs text-fg/70"><input type="checkbox" bind:checked={instanceGamescopeFsr} class="accent-brand-500" /> {uiText("ui.c5ca752901a0f010")}</label>{/if}
							{/if}
</div>
					</section><section id="instance-settings-janela" class="instance-settings-group">
						<div class="space-y-6">
							<div>
								<h3 class="text-xs font-bold text-fg uppercase tracking-wider">{uiText("ui.56342ebec923727e")}</h3>
								<p class="text-[11px] text-fg/40 mt-0.5">{uiText("ui.9a9496fbd02e9c15")}</p>
							</div>

							<div class="grid grid-cols-2 gap-4">
								<div class="space-y-1.5">
									<span class="text-xs font-bold text-fg/70 block">{uiText("ui.b11e7ca5545ba841")}</span>
									<input type="number" bind:value={instanceWindowWidth} class="w-full bg-bg-elevated border border-fg/10 rounded-2xl px-4 py-2 text-xs text-fg font-mono outline-none" />
								</div>
								<div class="space-y-1.5">
									<span class="text-xs font-bold text-fg/70 block">{uiText("ui.de73d011d72d3759")}</span>
									<input type="number" bind:value={instanceWindowHeight} class="w-full bg-bg-elevated border border-fg/10 rounded-2xl px-4 py-2 text-xs text-fg font-mono outline-none" />
								</div>
							</div>

							<div class="flex items-center justify-between bg-bg-elevated border border-fg/5 rounded-2xl p-4">
								<div>
									<span class="text-xs font-bold text-fg block">{uiText("ui.7c88f13b0bafe58a")}</span>
									<span class="text-[10px] text-fg/40 block mt-0.5">{uiText("ui.ed605bc2ba679639")}</span>
								</div>
								<button
									type="button"
									role="switch" aria-checked={instanceStartFullscreen} aria-label={uiText("ui.3eb3041b45e2c37e")}
									class="w-10 h-5 rounded-full transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-200 relative flex items-center px-0.5 cursor-pointer {instanceStartFullscreen ? 'bg-brand-500 shadow-glow' : 'bg-bg-subtle'}"
									onclick={() => instanceStartFullscreen = !instanceStartFullscreen}
								>
									<span class="w-4 h-4 rounded-full bg-fg transition-transform duration-200 shadow-md {instanceStartFullscreen ? 'translate-x-5' : 'translate-x-0'}"></span>
								</button>
							</div>
						</div>

					</section><section id="instance-settings-java" class="instance-settings-group">
						<div class="space-y-6">
							<div>
								<h3 class="text-xs font-bold text-fg uppercase tracking-wider">{uiText("settings.javaMemory")}</h3>
								<p class="text-[11px] text-fg/40 mt-0.5">{uiText("ui.3b67a722513ae251")}</p>
							</div>

							<div class="bg-bg-elevated border border-emerald-500/25 rounded-2xl p-4 space-y-2">
								<div class="flex items-center justify-between text-xs">
									<div class="flex items-center gap-2.5">
										<div class="w-8 h-8 rounded-xl bg-emerald-500/10 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
											<Cpu class="w-4 h-4" />
										</div>
										<div>
											<span class="font-bold text-fg block">{uiText("ui.5dfcd498d1d1a355")}</span>
											<span class="text-[10px] text-fg/50 block">{uiText("ui.1a2bd1d28cb68c00")} {Math.round(systemRamMb / 1024)} {uiText("ui.87f3475ed0cc3d56")}</span>
										</div>
									</div>
									<span class="text-[10px] font-mono font-bold text-emerald-400 bg-emerald-500/10 px-2.5 py-1 rounded-full border border-emerald-500/20 flex items-center gap-1">
										<Sparkles class="w-3 h-3" /> {instanceAutoOptimize ? uiText("ui.05a7d7f600ea7daa") : uiText("ui.eb2e7856f69c6e4d")}
									</span>
								</div>
								<p class="text-[11px] text-fg/50 leading-relaxed">
									{uiText("ui.1a6d4e6c6b0bab9c")}
								</p>
							</div>

							<div class="rounded-2xl border border-border bg-bg-subtle p-4 space-y-4">
                            <h3 class="text-sm font-semibold text-fg">{uiText("ui.d6558f3825d66efd")}</h3>
                            <label for="instance-ram-min" class="flex justify-between text-xs text-fg-muted"><span>{uiText("ui.40b336d098b84bde")}</span><span>{(instanceMinRamMb / 1024).toFixed(1)} GB</span></label>
                            <input id="instance-ram-min" type="range" min="512" max={instanceRamMb} step="256" bind:value={instanceMinRamMb} class="w-full accent-brand-500" />
                            <label for="instance-ram-max" class="flex justify-between text-xs text-fg-muted"><span>{uiText("ui.6c14c547e5a5577b")}</span><span>{(instanceRamMb / 1024).toFixed(1)} GB</span></label>
                            <input id="instance-ram-max" type="range" min="1024" max={Math.max(1024, Math.floor(systemRamMb / 256) * 256)} step="256" bind:value={instanceRamMb} class="w-full accent-brand-500" />
                            <p class="text-xs text-fg-muted">{uiText("ui.26d3b594b40df6af")}</p>
                        </div>
                        <div class="rounded-2xl border border-border bg-bg-subtle p-4 space-y-3">
                            <label for="instance-java" class="block text-sm font-semibold text-fg">{uiText("ui.6980b7f402766d81")}</label>
                            <select id="instance-java" bind:value={instanceJavaPath} class="w-full rounded-xl border border-border bg-bg-elevated px-3 py-3 text-xs text-fg"><option value="">{uiText("ui.1cd6b3e855b28ecc")}</option>{#each javaRuntimes as runtime}{#if runtime.path}<option value={runtime.path}>Java {runtime.major} · {runtime.versionString || runtime.path}</option>{/if}{/each}{#if instanceJavaPath && !javaRuntimes.some(runtime => runtime.path === instanceJavaPath)}<option value={instanceJavaPath}>{instanceJavaPath}</option>{/if}</select>
                            <button type="button" class={button({ variant: "secondary", size: "sm" })} onclick={detectJava} disabled={scanningJava}><RefreshCw class="h-4 w-4 {scanningJava ? 'animate-spin' : ''}" />{scanningJava ? 'Detectando…' : uiText("ui.e78537280b2f6241")}</button>
                            <div class="flex flex-wrap gap-2" aria-label={uiText("ui.2afd44d7e0ca9df4")}>{#each ['automatic', 'g1gc'] as preset}<button type="button" class={button({ variant: (preset === 'automatic' ? instanceAutoOptimize : instanceJvmArgs.includes('-XX:+UseG1GC')) ? "primary" : "secondary", size: "sm" })} onclick={() => applyJvmPreset(preset as 'automatic' | 'g1gc')}>{preset === 'automatic' ? uiText("settings.refinement.automaticJvm") : 'G1GC'}</button>{/each}</div>
                        </div>
                        <div class="space-y-2 bg-bg-elevated border border-fg/5 rounded-2xl p-4">
								<div class="flex items-center justify-between">
									<span class="text-xs font-bold text-fg/80">{uiText("ui.e033a695dae655ab")}</span>
									<span class="text-[10px] text-fg/40">{instanceJavaPath ? 'Customizado' : uiText("ui.bd6464b216ad9deb")}</span>
								</div>
								<div class="flex gap-2">
									<input
										type="text"
										bind:value={instanceJavaPath}
										placeholder={uiText("ui.2ef129a3d9f14f60")}
										class="flex-1 bg-bg-elevated border border-fg/10 rounded-xl px-4 py-2.5 text-xs text-fg font-mono outline-none focus:border-emerald-500"
									/>
									<button
										type="button"
										class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5 shrink-0" })}
										onclick={async () => {
											const selected = await open({
												title: uiText("ui.7161f9f39f03b19c"),
												multiple: false,
												directory: false
											});
											if (typeof selected === "string") {
												instanceJavaPath = selected;
											}
										}}
									>
										<FolderOpen class="w-3.5 h-3.5" /> {uiText("ui.6e7780ca6921c098")}
									</button>
								</div>
							</div>

							<div class="space-y-2 bg-bg-elevated border border-fg/5 rounded-2xl p-4">
								<div class="flex items-center justify-between">
									<span class="text-xs font-bold text-fg/80">{uiText("ui.ee15867ed5aa0c3e")}</span>
									{#if jvmValidation}
										{#if jvmValidation.valid}
											<span class="text-[10px] font-bold text-emerald-400 flex items-center gap-1">
												<Check class="w-3 h-3 text-emerald-400" /> {uiText("ui.5ef52a0077620be1")} {runtimePlatform.label}
											</span>
										{:else}
											<span class="text-[10px] font-bold text-amber-400 flex items-center gap-1">
												<span class="w-2 h-2 rounded-full bg-amber-400 animate-pulse"></span> {jvmValidation.rejected.length} {uiText("ui.3955baf1163f00d5")}
											</span>
										{/if}
									{/if}
								</div>

								<input
									type="text"
									aria-label={uiText("ui.1a419508e060780b")} bind:value={instanceJvmArgs}
									placeholder={uiText("ui.5854817f5f7815b4")}
									class="w-full bg-bg-elevated border border-fg/10 rounded-xl px-4 py-2.5 text-xs text-fg font-mono outline-none focus:border-emerald-500"
								/>

								{#if jvmValidation && !jvmValidation.valid}
									<div class="p-3 bg-amber-500/10 border border-amber-500/25 rounded-xl space-y-2 mt-2">
										<div class="text-[11px] font-bold text-amber-300">
											{uiText("ui.eb46f0dd3f439727")} {jvmValidation.rejected.join(", ")}
										</div>
										<ul class="text-[10px] text-fg/70 space-y-1 list-disc pl-4">
											{#each jvmValidation.suggestions as sug}
												<li>{sug}</li>
											{/each}
										</ul>
										<button
											type="button"
											class={launcherButton({ variant: "primary", size: "sm", class: "mt-1" })}
											onclick={() => {
												if (jvmValidation) {
													instanceJvmArgs = jvmValidation.normalized;
												}
											}}
										>
											{uiText("ui.2f6cbbb8bf61ec90")}
										</button>
									</div>
								{/if}
							</div>

							<div class="space-y-3 bg-bg-elevated border border-fg/5 rounded-2xl p-4">
								<div>
									<h4 class="text-xs font-bold text-fg/80">{uiText("ui.1e8ac000017d6279")}</h4>
									<p class="text-[10px] text-fg/40 mt-0.5">{uiText("ui.091d6d888c29fde3")}</p>
								</div>

								<div class="grid grid-cols-3 gap-2 pt-1">
									<button
										type="button"
										class="p-3 rounded-xl bg-bg-subtle hover:bg-bg-subtle border border-fg/5 hover:border-fg/20 text-left transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer flex flex-col justify-between group disabled:opacity-50"
										onclick={handleRepairInstance}
										disabled={isRepairing}
									>
										<div class="flex items-center justify-between w-full">
											<RefreshCw class="w-4 h-4 text-emerald-400 {isRepairing ? 'animate-spin' : 'group-hover:rotate-180 transition-transform duration-500'}" />
											{#if isRepairing}
												<span class="text-[9px] text-emerald-400 font-bold">{uiText("ui.0c3d47e9c33210d2")}</span>
											{/if}
										</div>
										<div class="mt-2">
											<p class="text-xs font-bold text-fg">{uiText("ui.598a4670601ddf97")}</p>
											<p class="text-[10px] text-fg/40">{uiText("ui.7db65939dfe2db30")}</p>
										</div>
									</button>

									<button
										type="button"
										class="p-3 rounded-xl bg-bg-subtle hover:bg-emerald-500/10 border border-emerald-500/20 hover:border-emerald-500/40 text-left transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer flex flex-col justify-between group disabled:opacity-50"
										onclick={handleRepairAll}
										disabled={isRepairingAll}
									>
										<div class="flex items-center justify-between w-full">
											<ShieldCheck class="w-4 h-4 text-emerald-400 {isRepairingAll ? 'animate-pulse' : 'group-hover:scale-110 transition-transform'}" />
											{#if isRepairingAll}
												<span class="text-[9px] text-emerald-400 font-bold">{uiText("ui.dc0546b3e22c8f9e")}</span>
											{/if}
										</div>
										<div class="mt-2">
											<p class="text-xs font-bold text-fg">{uiText("ui.96653c8b1e652f01")}</p>
											<p class="text-[10px] text-fg/40">{uiText("ui.671f31669fabe344")}</p>
										</div>
									</button>

									<button
										type="button"
										class="p-3 rounded-xl bg-bg-subtle hover:bg-bg-subtle border border-fg/5 hover:border-fg/20 text-left transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer flex flex-col justify-between group disabled:opacity-50"
										onclick={handleBackupSaves}
										disabled={isBackingUp}
									>
										<div class="flex items-center justify-between w-full">
											<Save class="w-4 h-4 text-emerald-400 group-hover:scale-110 transition-transform" />
											{#if isBackingUp}
												<span class="text-[9px] text-emerald-400 font-bold">{uiText("ui.f16f04e767446b97")}</span>
											{/if}
										</div>
										<div class="mt-2">
											<p class="text-xs font-bold text-fg">{uiText("ui.b19db652c4f0c998")}</p>
											<p class="text-[10px] text-fg/40">{uiText("ui.d2210529056f4ca9")}</p>
										</div>
									</button>

									<button
										type="button"
										class="p-3 rounded-xl bg-bg-subtle hover:bg-bg-subtle border border-fg/5 hover:border-fg/20 text-left transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer flex flex-col justify-between group disabled:opacity-50"
										onclick={handleExportZip}
										disabled={isExporting}
									>
										<div class="flex items-center justify-between w-full">
											<Share2 class="w-4 h-4 text-purple-400 group-hover:scale-110 transition-transform" />
											{#if isExporting}
												<span class="text-[9px] text-purple-400 font-bold">{uiText("ui.e50a7ac800f3396c")}</span>
											{/if}
										</div>
										<div class="mt-2">
											<p class="text-xs font-bold text-fg">{uiText("ui.ff014bfdd1352508")}</p>
											<p class="text-[10px] text-fg/40">{uiText("ui.7525e73f46d28721")}</p>
										</div>
									</button>

									<button
										type="button"
										class="p-3 rounded-xl bg-bg-subtle hover:bg-bg-subtle border border-fg/5 hover:border-brand-500/40 text-left transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer flex flex-col justify-between group disabled:opacity-50"
										onclick={handleExportShareCode}
										disabled={isGeneratingShareCode}
									>
										<div class="flex items-center justify-between w-full">
											<Sparkles class="w-4 h-4 text-brand-500 group-hover:scale-110 transition-transform" />
											{#if isGeneratingShareCode}
												<span class="text-[9px] text-brand-500 font-bold">{uiText("ui.f16f04e767446b97")}</span>
											{/if}
										</div>
										<div class="mt-2">
											<p class="text-xs font-bold text-fg">{uiText("ui.e401c3353dbf4918")}</p>
											<p class="text-[10px] text-fg/40">{uiText("ui.e223b0257858f237")}</p>
										</div>
									</button>
								</div>
							</div>
						</div>

					</section><section id="instance-settings-hooks" class="instance-settings-group">
						<div class="space-y-6">
							<div>
								<h3 class="text-xs font-bold text-fg uppercase tracking-wider">{uiText("ui.b9dac8db6dc5de93")}</h3>
								<p class="text-[11px] text-fg/40 mt-0.5">{uiText("ui.d7fc7773fcae43a9")}</p>
							</div>

							<div class="space-y-2">
								<span class="text-xs font-bold text-fg/70 block">{uiText("ui.a5d2b4869274d3b7")}</span>
								<input type="text" bind:value={instancePreLaunchHook} placeholder={uiText("ui.020ca0f4c7f03f88")} class="w-full bg-bg-elevated border border-fg/10 rounded-2xl px-4 py-2.5 text-xs text-fg font-mono outline-none" />
							</div>

							<div class="space-y-2">
								<span class="text-xs font-bold text-fg/70 block">{uiText("ui.c353a1ff17495f64")}</span>
								<input type="text" bind:value={instancePostExitHook} placeholder={uiText("ui.020ca0f4c7f03f88")} class="w-full bg-bg-elevated border border-fg/10 rounded-2xl px-4 py-2.5 text-xs text-fg font-mono outline-none" />
							</div>
						</div>
                    </section>

				</div>
			</div>

			<div class="flex items-center justify-end gap-3 pt-4 border-t border-fg/5">
				<button
					type="button"
					class={button({ variant: "secondary" })}
					onclick={() => { settingsModalSyncedFor = null; }}
				>
					{uiText("common.cancel")}
				</button>
				<button
					type="button"
					class={button({ variant: "primary" })}
                    disabled={savingInstanceSettings}
                    aria-busy={savingInstanceSettings}
					onclick={saveInstanceSettings}
				>
					<Check class="w-4 h-4 stroke-[3]" /> {uiText("ui.fdc83a1f4dac7cb0")}
				</button>
			</div>

            </section>
		</div>
        {/if}

	</div>

	<RightSidebar />

</div>

{#if previewScreenshot}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="fixed inset-0 z-50 bg-bg-overlay/95 backdrop-blur-2xl flex flex-col items-center justify-center p-4 select-none" in:fade={{ easing: quintOut, duration: 220 }} onclick={() => previewScreenshot = null}>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="absolute top-6 left-6 right-6 flex items-center justify-between z-10" onclick={(e) => e.stopPropagation()}>
			<div class="flex items-center gap-2 min-w-0">
				<Image class="w-4 h-4 text-purple-400 shrink-0" />
				<span class="text-xs font-bold text-fg truncate max-w-sm drop-shadow">{previewScreenshot.name}</span>
			</div>
			<div class="flex items-center gap-2">
				<div class="flex items-center gap-1 bg-fg/10 backdrop-blur-md rounded-full px-2 py-1 border border-fg/10">
					<button type="button" class={launcherButton({ variant: "secondary", size: "icon", class: "" })} onclick={zoomOutScreenshot} title={uiText("ui.f433fd692956a9ac")}>
						<ZoomOut class="w-3.5 h-3.5" />
					</button>
					<button type="button" class={launcherButton({ variant: "ghost", size: "sm", class: "" })} onclick={resetScreenshotZoom} title={uiText("ui.b3c135b66a00b0ef")}>
						{Math.round(screenshotZoom * 100)}%
					</button>
					<button type="button" class={launcherButton({ variant: "secondary", size: "icon", class: "" })} onclick={zoomInScreenshot} title={uiText("ui.dbdc887294502696")}>
						<ZoomIn class="w-3.5 h-3.5" />
					</button>
				</div>
				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
					onclick={() => previewScreenshot = null}
				>
					<X class="w-4 h-4" />
				</button>
			</div>
		</div>

		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="flex-1 w-full flex items-center justify-center overflow-hidden p-6" onclick={(e) => e.stopPropagation()}>
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
			<img loading="lazy" decoding="async"
				src={previewScreenshot.dataUrl || convertFileSrc(previewScreenshot.path)}
				alt={previewScreenshot.name}
				class="max-w-full max-h-[75vh] object-contain rounded-2xl shadow-2xl transition-transform duration-150 ease-out"
				style="transform: scale({screenshotZoom});"
				onclick={(e) => e.stopPropagation()}
			/>
		</div>

		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="absolute bottom-6 flex flex-wrap items-center gap-3 z-10" onclick={(e) => e.stopPropagation()}>
			<button
				type="button"
				class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-2" })}
				onclick={copyScreenshotImage}
			>
				<Copy class="w-3.5 h-3.5" /> {uiText("ui.fcfa0a511cf841dd")}
			</button>
			<button
				type="button"
				class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })}
				onclick={() => {
					navigator.clipboard.writeText(previewScreenshot!.path);
					toast("Caminho copiado!", "success");
				}}
			>
				<Copy class="w-3.5 h-3.5" /> {uiText("ui.1c031de96f1a554d")}
			</button>
			<button
				type="button"
				class={launcherButton({ variant: "danger", size: "sm", class: "flex items-center gap-1.5" })}
				onclick={async () => {
					await handleDeleteScreenshot(previewScreenshot!.path);
					previewScreenshot = null;
				}}
			>
				<Trash2 class="w-3.5 h-3.5" /> {uiText("screenshots.deleteBtn")}
			</button>
		</div>
	</div>
{/if}

{#if showShareCodeModal && generatedShareCode}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/80 backdrop-blur-md" in:fade={{ easing: quintOut, duration: 220 }}>
		<div class="w-full max-w-md bg-bg-elevated border border-brand-500/30 rounded-3xl p-6 shadow-2xl">
			<div class="flex items-center justify-between mb-4">
				<div class="flex items-center gap-2">
					<Sparkles class="w-5 h-5 text-brand-500" />
					<h3 class="text-sm font-black text-fg">{uiText("ui.4a39ff112f3ab0c4")}</h3>
				</div>
				<button type="button" class={launcherButton({ variant: "ghost", size: "icon", class: "" })} onclick={() => showShareCodeModal = false}>
					<X class="w-4 h-4" />
				</button>
			</div>

			<p class="text-xs text-fg/60 mb-4 leading-relaxed">
				{uiText("ui.6eec56b2ea50c77b")} <strong>{uiText("ui.715776ad6c3fa0be")}</strong> {uiText("ui.04b80ba98201a877")}
			</p>

			<div class="bg-bg-overlay/50 border border-brand-500/40 rounded-2xl p-4 flex items-center justify-between mb-5">
				<span class="font-mono text-xl font-black text-brand-500 tracking-wider select-all">{generatedShareCode}</span>
				<button
					type="button"
					class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-1.5" })}
					onclick={() => {
						navigator.clipboard.writeText(generatedShareCode!);
						toast(uiText("ui.02de8de8a052c934"), "success");
						playSound("chime");
					}}
				>
					<Copy class="w-3.5 h-3.5" /> {uiText("common.copy")}
				</button>
			</div>

			<button
				type="button"
				class={launcherButton({ variant: "secondary", size: "sm", class: "w-full" })}
				onclick={() => showShareCodeModal = false}
			>
				{uiText("statusBanner.dismiss")}
			</button>
		</div>
	</div>
{/if}

{#if activeEditorFile}
	<div class="fixed inset-0 z-50 bg-bg-overlay/55 flex items-center justify-center p-4 sm:p-6" in:fade={{ easing: quintOut, duration: 220 }}>
		<div class="max-w-4xl w-full h-[80vh] bg-bg-elevated border border-fg/10 rounded-3xl overflow-hidden shadow-2xl flex flex-col">
			<div class="p-4 border-b border-fg/10 flex items-center justify-between">
				<div class="flex items-center gap-2 min-w-0">
					<FileText class="w-4 h-4 text-emerald-400 shrink-0" />
					<span class="text-xs font-bold text-fg truncate">{activeEditorFile.name}</span>
					<span class="text-[10px] font-mono text-fg/40 truncate">({activeEditorFile.path})</span>
				</div>
				<div class="flex items-center gap-2 shrink-0">
					<button
						type="button"
						class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })}
						style="background-color: rgb(var(--brand-500));"
						disabled={isSavingEditor}
						onclick={handleSaveEditorFile}
					>
						<Save class="w-3.5 h-3.5 stroke-[2.5]" /> {isSavingEditor ? 'Salvando...' : uiText("ui.67a528de00957b92")}
					</button>
					<button
						type="button"
						class={launcherButton({ variant: "secondary", size: "icon", class: "ml-2" })}
						onclick={() => activeEditorFile = null}
					>
						<X class="w-4 h-4" />
					</button>
				</div>
			</div>
			<div class="flex-1 p-4 bg-bg-elevated overflow-hidden">
				<textarea
					bind:value={activeEditorFile.content}
					class="w-full h-full bg-transparent border-0 outline-none font-mono text-xs text-fg/90 leading-relaxed resize-none custom-scrollbar p-2"
					spellcheck="false"
				></textarea>
			</div>
			<div class="p-2.5 bg-bg-elevated border-t border-fg/5 text-[10px] text-fg/40 font-mono px-4 flex justify-between">
				<span>{uiText("ui.999e8aab92fc9678")} {Math.round(activeEditorFile.content.length / 1024)} KB</span>
				<span>{uiText("ui.d19967ef17fbcb07")}</span>
			</div>
		</div>
	</div>
{/if}

{#if showHostModal && hostLinkInfo}
	<div class="fixed inset-0 z-50 bg-bg-overlay/80 backdrop-blur-md flex items-center justify-center p-6" in:fade={{ easing: quintOut, duration: 220 }}>
		<div class="max-w-md w-full bg-bg-elevated border border-brand-500/30 rounded-3xl p-6 shadow-2xl space-y-5 relative overflow-hidden select-none">
			<div class="absolute -top-10 -right-10 w-36 h-36 bg-[radial-gradient(circle_at_center,rgb(var(--brand-500)/0.15),transparent_70%)] rounded-full pointer-events-none"></div>

			<div class="flex items-center justify-between">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-brand-500/15 border border-brand-500/30 flex items-center justify-center">
						<Share2 class="w-5 h-5 text-brand-500" />
					</div>
					<div>
						<h3 class="font-extrabold text-fg text-base">{uiText("ui.c8bf774680dc473c")}</h3>
						<p class="text-xs text-fg/50">{uiText("ui.839eed5aebf9543e")}</p>
					</div>
				</div>
				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
					onclick={() => showHostModal = false}
				>
					<X class="w-4 h-4" />
				</button>
			</div>

			<!-- UPnP Status Badge -->
			<div class="flex items-center justify-between px-3.5 py-2 rounded-2xl border {upnpResult?.success ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-300' : isOpeningUpnp ? 'bg-blue-500/10 border-blue-500/30 text-blue-300' : 'bg-amber-500/10 border-amber-500/30 text-amber-300'}">
				<div class="flex items-center gap-2">
					{#if isOpeningUpnp}
						<RefreshCw class="w-4 h-4 animate-spin text-blue-400" />
						<span class="text-xs font-bold">{uiText("ui.8990bc947c33eed4")}</span>
					{:else if upnpResult?.success}
						<Globe2 class="w-4 h-4 text-emerald-400" />
						<span class="text-xs font-extrabold">{uiText("ui.0ebfbaca769305a7")}</span>
					{:else}
						<Radio class="w-4 h-4 text-amber-400" />
						<span class="text-xs font-bold">{uiText("ui.cce4fa137bd9cfc9")}</span>
					{/if}
				</div>
				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "sm", class: "disabled:opacity-50" })}
					disabled={isOpeningUpnp}
					onclick={attemptUpnpOpen}
				>
					{isOpeningUpnp ? 'Aguarde...' : 'Reabrir UPnP'}
				</button>
			</div>

			<div class="bg-bg-elevated border border-fg/10 rounded-2xl p-4 space-y-3 shadow-inner">
				<div>
					<span class="text-[10px] font-extrabold text-brand-500 uppercase tracking-wider block mb-1">{uiText("ui.d7cb6fd6ba75193c")}</span>
					<div class="flex items-center gap-2">
						<input
							type="text"
							readonly
							value={hostLinkInfo.shareLink}
							class="flex-1 bg-bg-overlay/50 border border-fg/10 rounded-xl px-3.5 py-2 text-xs font-mono text-fg/90 outline-none select-all"
						/>
						<button
							type="button"
							class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-1.5" })}
							onclick={copyHostLink}
						>
							{#if isCopiedHostLink}
								<Check class="w-3.5 h-3.5 stroke-[3]" /> {uiText("ui.a8fe0fc805d5fd50")}
							{:else}
								<Copy class="w-3.5 h-3.5 stroke-[3]" /> {uiText("ui.b9cfc7837360c373")}
							{/if}
						</button>
					</div>
				</div>

				{#if upnpResult?.success && upnpResult.externalIp}
					<div class="pt-2 border-t border-fg/5">
						<span class="text-[10px] font-extrabold text-emerald-400 uppercase tracking-wider block mb-1">{uiText("ui.976577f5da8962bc")}</span>
						<div class="flex items-center gap-2">
							<input
								type="text"
								readonly
								value={`${upnpResult.externalIp}:${customHostPort}`}
								class="flex-1 bg-bg-overlay/50 border border-emerald-500/20 rounded-xl px-3.5 py-2 text-xs font-mono text-emerald-300 outline-none select-all"
							/>
							<button
								type="button"
								class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })}
								onclick={copyDirectAddress}
							>
								{#if isCopiedDirectAddress}
									<Check class="w-3.5 h-3.5" /> {uiText("ui.a8fe0fc805d5fd50")}
								{:else}
									<Copy class="w-3.5 h-3.5" /> {uiText("ui.50dbccc70acc7869")}
								{/if}
							</button>
						</div>
					</div>
				{/if}

				<div class="grid grid-cols-2 gap-2 pt-2 border-t border-fg/5">
					<div>
						<span class="text-[10px] text-fg/40 block font-bold">{uiText("ui.a661481fb87eb8cf")}</span>
						<span class="text-xs font-mono text-emerald-400 font-bold">{hostLinkInfo.directAddress}</span>
					</div>
					<div>
						<span class="text-[10px] text-fg/40 block font-bold">{uiText("ui.77b322e21110fd34")}</span>
						<div class="flex items-center gap-1 mt-0.5">
							<input
								type="number"
								bind:value={customHostPort}
								class="w-20 bg-bg-overlay/50 border border-fg/10 rounded-lg px-2 py-0.5 text-xs font-mono text-fg"
								onchange={refreshHostLink}
							/>
						</div>
					</div>
				</div>
			</div>

			<div class="text-[11px] text-fg/50 leading-relaxed space-y-1 bg-amber-500/10 border border-amber-500/20 p-3 rounded-2xl">
				<div class="font-bold text-amber-300">{uiText("ui.0cca698beb04034a")}</div>
				<div>{uiText("ui.2c687c3fb44a7abd")} <b>{uiText("ui.0bf00d4fe11a2575")}</b> {uiText("ui.c39f5640596dfdd2")} <b>{customHostPort}</b>.</div>
				<div>{uiText("ui.b914be67fab72283")} <b>{uiText("ui.50fcf243a3406348")}</b> {uiText("ui.fc74181ece96ac0d")} <b>{uiText("ui.2834161ad11b509f")}</b> {uiText("ui.74332bae5db24084")}</div>
			</div>
		</div>
	</div>
{/if}




{#if selectedSnapshotWorld}
	<WorldSnapshotsModal
		open={!!selectedSnapshotWorld}
		profileId={instanceId}
		worldName={selectedSnapshotWorld.name}
		folderName={selectedSnapshotWorld.folder}
		onClose={() => selectedSnapshotWorld = null}
		onRestored={() => refreshAllData()}
	/>
{/if}

{#if showConfigEditor}
<InstanceConfigEditorModal
	open={showConfigEditor}
	profileId={instanceId}
	onClose={() => showConfigEditor = false}
/>
{/if}

{#if showP2PHost}
<P2PHostModal
	profileId={instanceId}
	open={showP2PHost}
	onClose={() => showP2PHost = false}
/>
{/if}

{#if showModConflictModal}
<ModConflictModal
	isOpen={showModConflictModal}
	profileId={instanceId}
	conflictsResult={pendingLaunchConflicts}
	onResolved={() => {
		showModConflictModal = false;
		refreshAllData();
		handlePlay(true);
	}}
	onProceedAnyway={() => {
		showModConflictModal = false;
		handlePlay(true);
	}}
	onClose={() => showModConflictModal = false}
/>
{/if}

{#if showModpackExportModal}
<ModpackExportModal
	isOpen={showModpackExportModal}
	profileId={instanceId}
	instanceName={activeProfile?.name || "Modpack"}
	onClose={() => showModpackExportModal = false}
/>
{/if}

{#if showWorldBackupModal}
<WorldBackupModal
	isOpen={showWorldBackupModal}
	profileId={instanceId}
	worldsList={worldsList}
	onClose={() => showWorldBackupModal = false}
/>
{/if}

{#if showKeybindEditorModal}
<KeybindEditorModal
	isOpen={showKeybindEditorModal}
	profileId={instanceId}
	onClose={() => showKeybindEditorModal = false}
/>
{/if}

{#if showDeathDetectorModal}
<DeathDetectorModal
	open={showDeathDetectorModal}
	profileId={instanceId}
	onClose={() => showDeathDetectorModal = false}
	onOpenSnapshots={() => {
		showDeathDetectorModal = false;
		if (worldsList && worldsList.length > 0) {
			const w = worldsList[0];
			selectedSnapshotWorld = { name: w.name, folder: w.folderName };
		}
	}}
/>
{/if}

{#if showJukeboxModal}
<JukeboxModal
	open={showJukeboxModal}
	onClose={() => showJukeboxModal = false}
/>
{/if}
