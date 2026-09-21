<script lang="ts">
    import InstanceHero from "$lib/components/instances/InstanceHero.svelte";
	import { button } from "$lib/components/ui/button";
	import LoaderBadge from "$lib/components/instances/LoaderBadge.svelte";
	import { javaScan } from "$lib/api/java";
	import type { JavaInstallStatus } from "$lib/api/types";
	import { page } from "$app/state";
	import { fade, scale } from "svelte/transition";
	import { onMount } from "svelte";
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
		Link
	} from "lucide-svelte";
	import RightSidebar from "$lib/components/layout/RightSidebar.svelte";
	import VirtualList from "$lib/components/ui/VirtualList.svelte";
	import WorldSnapshotsModal from "$lib/components/ui/WorldSnapshotsModal.svelte";
	import InstanceConfigEditorModal from "$lib/components/ui/InstanceConfigEditorModal.svelte";
	import P2PHostModal from "$lib/components/ui/P2PHostModal.svelte";
	import KeybindEditorModal from "$lib/components/instances/KeybindEditorModal.svelte";
	import ModConflictModal from "$lib/components/instances/ModConflictModal.svelte";
	import ModpackExportModal from "$lib/components/instances/ModpackExportModal.svelte";
	import WorldBackupModal from "$lib/components/instances/WorldBackupModal.svelte";
	import DeathDetectorModal from "$lib/components/instances/DeathDetectorModal.svelte";
	import JukeboxModal from "$lib/components/instances/JukeboxModal.svelte";
	import { profiles, type Profile } from "$lib/stores/profiles.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { getFullCapeDataUrl } from "$lib/utils/capeTextures";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { open, save } from "@tauri-apps/plugin-dialog";
	import { convertFileSrc } from "@tauri-apps/api/core";
	import { handlePostLaunchActions } from "$lib/utils/launcherLifecycle";
	import {
		launchGame,
		stopGame,
		versionsCheckInstalled,
		versionsDownload,
		instanceFileTree,
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
		instanceModsOpenFolder,
		instancePackAdd,
		instancePackDelete,
		instancePackOpenFolder,
		profilesUpdate,
		discordSetActivity,
		readTextFile,
		writeTextFile,
		deleteFileOrDir,
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
		optimizerGetPerfPack,
		optimizerInstallPerfPack,
		optimizerDetectGpu,
		modsResolveNames,
		modsResolveIcons,
		type GpuInfo,
		type PerformancePackInfo,
		type JvmValidationResult,
		type FileTreeEntry,
		type WorldDetail,
		type HostLinkInfo,
		instanceShieldScan,
		type ShieldScanResult,
		modpackCheckUpdate,
		modpackUpdateAtomic,
		type ModpackUpdateInfo,
		doctorCheckInstanceConflicts,
		type PreLaunchCheckResult,
		listen,
		upnpOpenPort,
		upnpClosePort,
		type UpnpPortMappingResult,
		type ModUpdateItem
	} from "$lib/api";
	import { achievements } from "$lib/stores/achievements.svelte";
	import { playSound } from "$lib/utils/sound";

	const instanceId = $derived(page.params.id ?? "");
	const activeProfile = $derived(profiles.list.find(p => p.id === instanceId) || profiles.active);

	const heroBanner = $derived.by(() => {
		if (activeProfile?.banner) return activeProfile.banner;
		if (activeProfile?.icon && (activeProfile.icon.startsWith("http") || activeProfile.icon.startsWith("data:"))) {
			return activeProfile.icon;
		}
		const nameLower = (activeProfile?.name || "").toLowerCase();
		if (nameLower.includes("better mc") || nameLower.includes("better minecraft") || nameLower.includes("bmc")) {
			return "/modpack_better_mc.webp";
		}
		if (nameLower.includes("pixelmon") || nameLower.includes("cobblemon")) {
			return "/modpack_cobblemon.webp";
		}
		if (nameLower.includes("fabulously optimized") || nameLower.includes("fo")) {
			return "/modpack_fo.webp";
		}
		return "/bg_day.jpg";
	});

	let mainTab = $state<"conteudo" | "mundos" | "galeria" | "ficheiros" | "configuracoes">("conteudo");
	let subTab = $state<"mods" | "resourcepacks" | "shaders" | "datapacks">("mods");
	let searchQuery = $state("");

	let modpackUpdate = $state<ModpackUpdateInfo | null>(null);
	let isCheckingUpdate = $state(false);
	let isUpdatingModpack = $state(false);
	let updateStatusText = $state("");
	let updateProgressPercent = $state(0);

	let showModConflictModal = $state(false);
	let pendingLaunchConflicts = $state<PreLaunchCheckResult | null>(null);
	let showModpackExportModal = $state(false);
	let showWorldBackupModal = $state(false);
	let showKeybindEditorModal = $state(false);
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

	let showInstanceSettingsModal = $state(false);
	let activeInstanceSection = $state<"geral" | "instalacao" | "otimizacao" | "janela" | "controlos" | "java" | "hooks">("geral");
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
	function applyJvmPreset(preset: "aikar" | "zgc" | "shenandoah") {
		instanceAutoOptimize = preset === "aikar";
		instanceJvmArgs = preset === "aikar" ? "" : preset === "zgc" ? "-XX:+UseZGC" : "-XX:+UseShenandoahGC";
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
	let gpuInfo = $state<GpuInfo | null>(null);
	let perfPackInfo = $state<PerformancePackInfo | null>(null);
	let generatedAikarFlags = $state<string[]>([]);
	let installingPerfPack = $state(false);
	let instancePreLaunchHook = $state("");
	let instancePostExitHook = $state("");
	let systemRamMb = $state(8192);

	const ramPresets = $derived.by(() => {
		const total = systemRamMb || 8192;
		const gb = Math.floor(total / 1024);
		const options: { mb: number; label: string; desc: string }[] = [];

		if (gb <= 4) {
			options.push({ mb: 1024, label: "1 GB", desc: "Mínimo" });
			options.push({ mb: 2048, label: "2 GB", desc: "Padrão" });
			options.push({ mb: 3072, label: "3 GB", desc: "Recomendado" });
		} else if (gb <= 8) {
			options.push({ mb: 2048, label: "2 GB", desc: "Vanilla" });
			options.push({ mb: 4096, label: "4 GB", desc: "Ideal" });
			options.push({ mb: 6144, label: "6 GB", desc: "Mods leves" });
		} else if (gb <= 16) {
			options.push({ mb: 4096, label: "4 GB", desc: "Vanilla" });
			options.push({ mb: 6144, label: "6 GB", desc: "Ideal mods" });
			options.push({ mb: 8192, label: "8 GB", desc: "Modpacks" });
			options.push({ mb: 12288, label: "12 GB", desc: "Pesado" });
		} else {
			options.push({ mb: 4096, label: "4 GB", desc: "Leve" });
			options.push({ mb: 6144, label: "6 GB", desc: "Ideal" });
			options.push({ mb: 8192, label: "8 GB", desc: "Modpacks" });
			options.push({ mb: 12288, label: "12 GB", desc: "Pesado" });
			options.push({ mb: 16384, label: "16 GB", desc: "Extremo" });
		}
		return options;
	});

	$effect(() => {
		if (activeProfile) {
			instanceNameInput = activeProfile.name || "Latest Release";
            instanceBanner = activeProfile.banner || "";
			instanceRamMb = activeProfile.ramMb || 4096;
			instanceJvmArgs = activeProfile.jvmArgs || "";
			const minimum = (activeProfile.jvmArgs || "").match(/-Xms(\d+)([mMgG])/);
			instanceMinRamMb = minimum ? Number(minimum[1]) * (minimum[2].toLowerCase() === "g" ? 1024 : 1) : Math.min(1024, activeProfile.ramMb || 4096);
			instanceAutoOptimize = activeProfile.autoOptimize !== false;
			instanceEnableVulkanOpt = activeProfile.useVulkan === true;
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
				.then(flags => generatedAikarFlags = flags)
				.catch(() => {});
		}, 300);
		return () => {
			if (aikarTimer) clearTimeout(aikarTimer);
		};
	});

	async function saveInstanceSettings() {
		if (activeProfile) {
			if (instanceMinRamMb > instanceRamMb || instanceRamMb > systemRamMb) { toast("A RAM mínima deve ser menor que a máxima e caber na memória do sistema.", "error"); return; }
			instanceJvmArgs = instanceJvmArgs.split(/\s+/).filter(value => value && !/^-Xm[sx]/.test(value)).concat(`-Xms${instanceMinRamMb}M`, `-Xmx${instanceRamMb}M`).join(" ");

			try {
				if (instanceBanner.trim() && new URL(instanceBanner.trim()).protocol !== "https:") throw new Error("Use uma imagem HTTPS para o banner.");
                await profilesUpdate({
                    id: activeProfile.id,
					name: instanceNameInput,
					ramMb: instanceRamMb,
					jvmArgs: instanceJvmArgs,
					autoOptimize: instanceAutoOptimize,
					useVulkan: instanceEnableVulkanOpt,
					loader: instanceLoaderType,
					loaderVersion: instanceLoaderVersion || null,
					resolutionW: instanceWindowWidth,
					resolutionH: instanceWindowHeight,
					fullscreen: instanceStartFullscreen,
					javaPath: instanceJavaPath || null,
				});
				profiles.update(activeProfile.id, {
					name: instanceNameInput,
					ramMb: instanceRamMb,
					jvmArgs: instanceJvmArgs,
					autoOptimize: instanceAutoOptimize,
					useVulkan: instanceEnableVulkanOpt,
					loader: instanceLoaderType as Profile["loader"],
					loaderVersion: instanceLoaderVersion,
					resolution: { width: instanceWindowWidth, height: instanceWindowHeight, fullscreen: instanceStartFullscreen },
					resolutionW: instanceWindowWidth,
					resolutionH: instanceWindowHeight,
					fullscreen: instanceStartFullscreen,
					javaPath: instanceJavaPath || null,
				});
				profiles.setBanner(activeProfile.id, instanceBanner.trim());
                toast("Configurações salvas com sucesso!", "success");
			} catch (e) {
				toast("Erro ao salvar no banco de dados: " + String(e), "error");
				return;
			}
		}
		showInstanceSettingsModal = false;
	}

	async function handleInstallPerfPack() {
		if (!activeProfile) return;
		installingPerfPack = true;
		try {
			const installed = await optimizerInstallPerfPack(activeProfile.id);
			toast(`Pacote de performance instalado: ${installed.join(", ")}`, "success");
			await refreshAllData();
		} catch (e) {
			toast(`Falha ao instalar pacote: ${String(e)}`, "error");
		} finally {
			installingPerfPack = false;
		}
	}

	let jvmValidation = $state<JvmValidationResult | null>(null);
	let isRepairing = $state(false);
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
			toast("Verificando integridade e reparando arquivos...", "info");
			await instanceRepair(activeProfile.id);
			toast("Instância e bibliotecas verificadas e reparadas com sucesso!", "success");
		} catch (e) {
			toast("Erro ao reparar instância: " + String(e), "error");
		} finally {
			isRepairing = false;
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
			toast("Criando backup dos mundos (saves)...", "info");
			await instanceBackupSaves(activeProfile.id, path);
			toast(`Backup salvo com sucesso em: ${path}`, "success");
		} catch (e) {
			toast("Erro ao criar backup: " + String(e), "error");
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
			toast("Exportando instância completa...", "info");
			await instanceExportZip(activeProfile.id, path);
			toast(`Instância exportada com sucesso em: ${path}`, "success");
		} catch (e) {
			toast("Erro ao exportar instância: " + String(e), "error");
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
			toast(`Código ${code} gerado e copiado!`, "success");
		} catch (e) {
			toast("Erro ao gerar código de compartilhamento: " + String(e), "error");
		} finally {
			isGeneratingShareCode = false;
		}
	}

	let isCheckingUpdates = $state(false);
	let availableModUpdates = $state<ModUpdateItem[]>([]);
	let isUpdatingAllMods = $state(false);
	let updatingModProjects = $state<string[]>([]);

	async function handleCheckModUpdates() {
		if (!instanceId) return;
		isCheckingUpdates = true;
		toast("Verificando atualizações de mods...", "info");
		try {
			const updates = await modsCheckUpdates(instanceId);
			availableModUpdates = updates || [];
			if (availableModUpdates.length === 0) {
				toast("Todos os mods estão atualizados!", "success");
			} else {
				playSound("chime");
				toast(`${availableModUpdates.length} atualização(ões) de mods encontrada(s)!`, "info");
			}
		} catch (e) {
			toast("Erro ao verificar atualizações: " + String(e), "error");
		} finally {
			isCheckingUpdates = false;
		}
	}

	async function handleUpdateAllMods() {
		if (!instanceId || availableModUpdates.length === 0 || isUpdatingAllMods) return;
		isUpdatingAllMods = true;
		toast(`Atualizando ${availableModUpdates.length} mods...`, "info");
		let count = 0;
		const toUpdate = [...availableModUpdates];
		for (const u of toUpdate) {
			try {
				await modsUpdate(u.projectId, u.latestVersionId, instanceId);
				count++;
				availableModUpdates = availableModUpdates.filter(x => x.projectId !== u.projectId);
			} catch (err) {
				console.error("Falha ao atualizar mod:", u.projectId, err);
			}
		}
		isUpdatingAllMods = false;
		if (count > 0) {
			playSound("achievement");
			toast(`${count} mod(s) atualizado(s) com sucesso!`, "success");
			await refreshAllData();
		} else {
			toast("Nenhum mod pôde ser atualizado no momento.", "warning");
		}
	}

	async function handleUpdateSingleMod(update: ModUpdateItem) {
		if (!instanceId || updatingModProjects.includes(update.projectId) || isUpdatingAllMods) return;
		updatingModProjects = [...updatingModProjects, update.projectId];
		try {
			toast(`Atualizando ${update.projectName || update.projectTitle || 'mod'}...`, "info");
			await modsUpdate(update.projectId, update.latestVersionId, instanceId);
			playSound("chime");
			toast(`Mod atualizado com sucesso!`, "success");
			availableModUpdates = availableModUpdates.filter(x => x.projectId !== update.projectId);
			await refreshAllData();
		} catch (e) {
			toast("Erro ao atualizar mod: " + String(e), "error");
		} finally {
			updatingModProjects = updatingModProjects.filter(id => id !== update.projectId);
		}
	}

	let isRepairingModpack = $state(false);

	async function handleRepairModpack() {
		if (!instanceId) return;
		isRepairingModpack = true;
		try {
			toast("Verificando integridade e baixando mods faltantes...", "info");
			const count = await instanceRepairModpack(instanceId);
			if (count > 0) {
				toast(`Modpack reparado! ${count} mod(s) baixado(s).`, "success");
				playSound("achievement");
				await refreshAllData();
			} else {
				toast("Todos os mods do modpack já estão íntegros!", "success");
			}
		} catch (e) {
			toast("Erro ao reparar modpack: " + String(e), "error");
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
				toast("Porta UPnP aberta com sucesso no roteador!", "success");
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
			toast("Erro ao gerar link de host: " + String(e), "error");
		}
	}

	async function refreshHostLink() {
		try {
			hostLinkInfo = await p2pGetHostLink(customHostPort);
			await attemptUpnpOpen();
			toast("Link de conexão atualizado com a porta " + customHostPort, "success");
		} catch (e) {
			toast("Erro ao atualizar porta: " + String(e), "error");
		}
	}

	function copyHostLink() {
		if (!hostLinkInfo) return;
		navigator.clipboard.writeText(hostLinkInfo.shareLink);
		isCopiedHostLink = true;
		toast("Link próprio de conexão copiado! Envie para seus amigos.", "success");
		setTimeout(() => (isCopiedHostLink = false), 2500);
	}

	function copyDirectAddress() {
		if (!hostLinkInfo) return;
		const addr = upnpResult?.externalIp ? `${upnpResult.externalIp}:${customHostPort}` : hostLinkInfo.directAddress;
		navigator.clipboard.writeText(addr);
		isCopiedDirectAddress = true;
		toast("Endereço IP copiado!", "success");
		setTimeout(() => (isCopiedDirectAddress = false), 2000);
	}

	// Launch & Install state
	let isLaunching = $state(false);
	let isInstalled = $state(true);
	let launchStatusText = $state("");
	let downloadProgressPercent = $state(0);

	// Real Data from File System & Backend
	let worldsList = $state<WorldDetail[]>([]);
	let screenshotsList = $state<Array<{ name: string; path: string; modified: string; dataUrl?: string | null }>>([]);
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
			toast("Imagem copiada para a área de transferência!", "success");
			playSound("chime");
		} catch {
			await navigator.clipboard.writeText(previewScreenshot.path);
			toast("Caminho da captura copiado!", "info");
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
				toast(`⚠️ Luxmc Shield detectou ${shieldResult.threats.length} ameaça(s) nos mods!`, "error");
			}
		} catch {
			// Silent scan: do not show error toast when clean or idle
		} finally {
			isScanningShield = false;
		}
	}

	let hasAutoScannedShield = $state(false);
	$effect(() => {
		if (subTab === "mods" && instanceMods.length > 0 && !shieldResult && !isScanningShield && !hasAutoScannedShield) {
			hasAutoScannedShield = true;
			runShieldScan();
		}
	});

	const currentPacksList = $derived(
		subTab === "resourcepacks" ? resourcePacks : subTab === "shaders" ? shaderPacks : dataPacks
	);

	const filteredMods = $derived(
		searchQuery
			? instanceMods.filter(m => m.name.toLowerCase().includes(searchQuery.toLowerCase()))
			: instanceMods
	);

	let selectedModNames = $state<string[]>([]);
	let frozenModNames = $state<string[]>([]);
	let activeMenuMod = $state<string | null>(null);

	const allModsSelected = $derived(
		filteredMods.length > 0 && filteredMods.every(m => selectedModNames.includes(m.name))
	);

	function toggleSelectAllMods() {
		if (allModsSelected) {
			selectedModNames = [];
		} else {
			selectedModNames = filteredMods.map(m => m.name);
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
		if (frozenModNames.includes(mod.name)) {
			frozenModNames = frozenModNames.filter(n => n !== mod.name);
			toast(`Versão de "${mod.name.replace('.disabled', '')}" descongelada.`, "info");
		} else {
			frozenModNames = [...frozenModNames, mod.name];
			toast(`Versão de "${mod.name.replace('.disabled', '')}" congelada! Atualizações pausadas.`, "success");
		}
	}

	function handleCopyModLink(mod: FileTreeEntry) {
		const raw = mod.name.replace(/\.disabled$/, '').replace(/\.(jar|zip)$/, '');
		const url = `https://modrinth.com/mod/${raw.toLowerCase().replace(/[^a-z0-9_-]/g, '-')}`;
		if (navigator.clipboard) {
			navigator.clipboard.writeText(url).then(() => {
				toast("Link do mod copiado para a área de transferência!", "success");
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
		toast("Mod sincronizado com sucesso!", "success");
	}

	function handleSwapVersion(mod: FileTreeEntry) {
		const raw = mod.name.replace(/\.disabled$/, '').replace(/\.(jar|zip)$/, '');
		import("$app/navigation").then(({ goto }) => {
			goto(`/mods?search=${encodeURIComponent(raw)}`);
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

	onMount(async () => {
        void detectJava();
			try {
			const specs = await getSystemSpecs();
			if (specs && specs.totalRamMb > 0) {
				systemRamMb = specs.totalRamMb;
			}
			gpuInfo = await optimizerDetectGpu();
			if (activeProfile) {
				perfPackInfo = await optimizerGetPerfPack(activeProfile.loader, activeProfile.mcVersion);
			}
		} catch {}
		await refreshAllData();
		if (instanceId) {
			checkModpackUpdate();
		}

		if (instanceId) {
			const hasNumericNames = instanceMods.some(m => /^\d+_\d+\.jar$/.test(m.name.replace('.disabled', '')));
			if (hasNumericNames) {
				modsResolveNames(instanceId).then(renamed => {
					if (renamed > 0) {
						refreshAllData();
					}
				}).catch(() => {});
			}
		}
	});

	async function refreshAllData() {
		if (!instanceId) return;
		isLoadingData = true;
		try {
			const ver = activeProfile?.mcVersion || "1.20.4";

			const [newIsInstalled, newWorlds, newScreenshots, newFileTree, newMods, newResources, newShaders, newDataPacks] = await Promise.all([
				versionsCheckInstalled(ver).catch(() => true),
				instanceWorldsList(instanceId).catch(() => []),
				instancesScreenshots(instanceId).catch(() => []),
				instanceFileTree(instanceId, fileSubPath || undefined).catch(() => []),
				instanceFileTree(instanceId, "mods").catch(() => []),
				instanceFileTree(instanceId, "resourcepacks").catch(() => []),
				instanceFileTree(instanceId, "shaderpacks").catch(() => []),
				instanceFileTree(instanceId, "datapacks").catch(() => []),
			]);

			isInstalled = newIsInstalled;
			worldsList = newWorlds;
			screenshotsList = newScreenshots;
			fileTree = newFileTree;
			instanceMods = newMods;
			resourcePacks = newResources;
			shaderPacks = newShaders;
			dataPacks = newDataPacks;

			if (instanceId && newMods.some(m => /^\d+(_\d+)?\.jar$/.test(m.name.replace('.disabled', '')))) {
				modsResolveNames(instanceId).then(renamed => {
					if (renamed > 0) {
						instanceFileTree(instanceId, "mods").then(updated => {
							instanceMods = updated;
						}).catch(() => {});
					}
				}).catch(() => {});
			}

			if (instanceId && newMods.some(m => !m.icon)) {
				modsResolveIcons(instanceId).then(resolved => {
					if (resolved > 0) {
						instanceFileTree(instanceId, "mods").then(updated => {
							instanceMods = updated;
						}).catch(() => {});
					}
				}).catch(() => {});
			}
		} catch (e) {
			console.error(e);
			toast("Falha ao carregar dados da instância: " + String(e), "error");
		} finally {
			isLoadingData = false;
		}
	}

	async function handleToggleMod(mod: FileTreeEntry) {
		const isCurrentlyDisabled = mod.name.endsWith(".disabled");
		try {
			await instanceModToggle(instanceId, mod.name, isCurrentlyDisabled);
			toast(isCurrentlyDisabled ? `Mod ativado!` : `Mod desativado!`, "success");
			await refreshAllData();
		} catch (e) {
			toast("Erro ao alternar mod: " + String(e), "error");
		}
	}

	async function handleDeleteMod(mod: FileTreeEntry) {
		try {
			await instanceModDelete(instanceId, mod.name);
			playSound("delete");
			toast(`Mod "${mod.name}" removido com sucesso!`, "success");
			await refreshAllData();
		} catch (e) {
			toast("Erro ao remover mod: " + String(e), "error");
		}
	}

	async function handleAddModFile() {
		try {
			const selected = await open({
				title: "Selecione o arquivo .JAR do Mod",
				multiple: true,
				filters: [{ name: "Mod do Minecraft (.jar)", extensions: ["jar"] }]
			});
			if (!selected) return;
			const paths = Array.isArray(selected) ? selected : [selected];
			for (const p of paths) {
				await instanceModAdd(instanceId, p);
			}
			toast(`${paths.length} mod(s) adicionado(s) com sucesso!`, "success");
			await refreshAllData();
		} catch (e) {
			toast("Erro ao adicionar mod: " + String(e), "error");
		}
	}

	async function handleOpenModsFolder() {
		try {
			await instanceModsOpenFolder(instanceId);
		} catch (e) {
			toast("Erro ao abrir pasta de mods: " + String(e), "error");
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
				toast(`Arquivo grande (${Math.round(file.size / 1024)} KB). Abra com o gerenciador do sistema para melhor performance.`, "info");
				return;
			}
			const isText = /\.(txt|json|properties|toml|log|cfg|mcmeta|yaml|yml|ini|csv|md|sh|lock|json5)$/i.test(file.name);
			if (isText) {
				try {
					const content = await readTextFile(file.path);
					activeEditorFile = { path: file.path, name: file.name, content };
				} catch (e) {
					toast("Não foi possível ler este arquivo: " + String(e), "error");
				}
			} else {
				toast(`Arquivo binário (${Math.round(file.size / 1024)} KB). Abra com o gerenciador do sistema.`, "info");
			}
		}
	}

	async function handleSaveEditorFile() {
		if (!activeEditorFile) return;
		isSavingEditor = true;
		try {
			await writeTextFile(activeEditorFile.path, activeEditorFile.content);
			toast(`Arquivo "${activeEditorFile.name}" guardado com sucesso!`, "success");
		} catch (e) {
			toast("Erro ao salvar arquivo: " + String(e), "error");
		} finally {
			isSavingEditor = false;
		}
	}

	async function handleDeleteFileEntry(file: FileTreeEntry) {
		if (!confirm(`Tem certeza que deseja excluir "${file.name}" permanentemente?`)) return;
		try {
			await deleteFileOrDir(file.path);
			playSound("delete");
			toast(`"${file.name}" excluído com sucesso!`, "success");
			fileTree = await instanceFileTree(instanceId, fileSubPath || undefined).catch(() => []);
		} catch (e) {
			toast("Erro ao excluir arquivo: " + String(e), "error");
		}
	}

	async function handleDeleteWorld(folderName: string) {
		if (!instanceId) return;
		if (!confirm(`Tem certeza que deseja excluir o mundo "${folderName}" permanentemente? Esta ação não pode ser desfeita.`)) return;
		try {
			await instanceWorldDelete(instanceId, folderName);
			playSound("delete");
			toast(`Mundo "${folderName}" excluído com sucesso!`, "success");
			worldsList = worldsList.filter(w => w.folderName !== folderName);
		} catch (e) {
			toast("Erro ao excluir mundo: " + String(e), "error");
		}
	}

	let isImportingWorld = $state(false);

	async function handleImportWorld() {
		if (!instanceId || isImportingWorld) return;
		try {
			const selected = await open({
				multiple: false,
				directory: false,
				title: "Selecionar Mapa do Minecraft (.zip)",
				filters: [
					{ name: "Mundo Minecraft Comprimido (*.zip)", extensions: ["zip"] },
					{ name: "Todos os Arquivos (*.*)", extensions: ["*"] }
				]
			});
			if (!selected) return;
			const filePath = typeof selected === "string" ? selected : selected[0];
			if (!filePath) return;

			isImportingWorld = true;
			toast("Importando e descompactando mapa...", "info");
			const imported = await instanceWorldImport(instanceId, filePath);
			playSound("achievement");
			toast(`Mundo "${imported.name}" importado com sucesso!`, "success");
			worldsList = await instanceWorldsList(instanceId);
		} catch (e) {
			toast("Erro ao importar mundo: " + String(e), "error");
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
				title: "Selecionar Pasta de Mundo (contendo level.dat)"
			});
			if (!selected) return;
			const folderPath = typeof selected === "string" ? selected : selected[0];
			if (!folderPath) return;

			isImportingWorld = true;
			toast("Importando pasta do mundo...", "info");
			const imported = await instanceWorldImport(instanceId, folderPath);
			playSound("achievement");
			toast(`Mundo "${imported.name}" importado com sucesso!`, "success");
			worldsList = await instanceWorldsList(instanceId);
		} catch (e) {
			toast("Erro ao importar mundo: " + String(e), "error");
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
				toast("Nenhum conflito de mods detectado! Seus mods estão compatíveis.", "success");
			}
		} catch (e) {
			toast("Falha ao checar conflitos: " + String(e), "error");
		}
	}

	async function handlePlay(skipConflictCheck: boolean = false) {
		if (isLaunching) return;

		const targetProfileId = activeProfile?.id || instanceId || "";
		if (!skipConflictCheck) {
			try {
				const conflicts = await doctorCheckInstanceConflicts(targetProfileId);
				if (conflicts.hasConflicts) {
					pendingLaunchConflicts = conflicts;
					showModConflictModal = true;
					return;
				}
			} catch (e) {
				console.warn("Pre-launch conflict check error:", e);
			}
		}

		isLaunching = true;
		downloadProgressPercent = 15;
		launchStatusText = "Preparando autenticação da conta...";

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
			launchStatusText = "Verificando bibliotecas e integridade do jogo...";

			// If not installed, download version
			if (!isInstalled) {
				downloadProgressPercent = 55;
				launchStatusText = "Baixando client.jar do Minecraft " + verId + "...";
				await versionsDownload(verId);
				isInstalled = true;
			}

			downloadProgressPercent = 85;
			launchStatusText = "Injetando parâmetros JVM, flags e inicializando Minecraft...";
			const targetProfileId = activeProfile?.id || instanceId || "";
			const isVulkan = typeof window !== "undefined" ? localStorage.getItem("luxmc_enable_vulkan") === "true" : false;
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
				smallImage: activeProfile?.loader === "fabric" ? "fabric" : (activeProfile?.loader === "forge" ? "curse" : "grass"),
				smallText: `Luxmc v1.9.2`,
				startTime: Math.floor(Date.now() / 1000),
				buttons: [
					{ label: "Baixar Luxmc", url: "https://luxmc-r92.pages.dev" },
					{ label: "Site Oficial", url: "https://luxmc-r92.pages.dev" }
				]
			}).catch(() => {});

			downloadProgressPercent = 100;
			launchStatusText = `Minecraft em execução (PID: ${result.pid})`;
			toast(`🎮 Minecraft ${verId} iniciado com sucesso! (PID: ${result.pid})`, "success");
			void handlePostLaunchActions();
		} catch (e) {
			console.error("Launch error:", e);
			toast("Falha ao iniciar o jogo: " + String(e), "error");
			launchStatusText = "";
			downloadProgressPercent = 0;
			appState.isGameRunning = false;
		} finally {
			setTimeout(() => {
				isLaunching = false;
				launchStatusText = "";
				downloadProgressPercent = 0;
			}, 2000);
		}
	}

	async function handleStopGame() {
		try {
			await stopGame();
			appState.isGameRunning = false;
			gamingStats.onGameExit();
			toast("Instância encerrada com sucesso.", "info");
		} catch (e) {
			toast("Erro ao tentar encerrar o jogo: " + String(e), "error");
		}
	}

	async function handleAddResourcePack() {
		const packType = subTab === 'shaders' ? 'shaderpacks' : subTab === 'datapacks' ? 'datapacks' : 'resourcepacks';
		const label = subTab === 'shaders' ? 'Shader' : subTab === 'datapacks' ? 'Datapack' : 'Pacote de Recursos';
		try {
			const selected = await open({
				title: `Selecionar ${label} (.zip)`,
				multiple: true,
				filters: [{ name: "Arquivo Compactado (.zip)", extensions: ["zip"] }]
			});
			if (!selected) return;
			const paths = Array.isArray(selected) ? selected : [selected];
			for (const p of paths) {
				await instancePackAdd(instanceId, packType, p);
			}
			toast(`${paths.length} ${label.toLowerCase()}(s) adicionado(s) com sucesso!`, "success");
			await refreshAllData();
		} catch (e) {
			toast("Erro ao adicionar arquivo: " + String(e), "error");
		}
	}

	async function handleDeletePack(fileName: string) {
		const packType = subTab === 'shaders' ? 'shaderpacks' : subTab === 'datapacks' ? 'datapacks' : 'resourcepacks';
		try {
			await instancePackDelete(instanceId, packType, fileName);
			playSound("delete");
			toast("Item removido com sucesso!", "success");
			await refreshAllData();
		} catch (e) {
			toast("Erro ao remover: " + String(e), "error");
		}
	}

	async function handleOpenPackFolder() {
		const packType = subTab === 'shaders' ? 'shaderpacks' : subTab === 'datapacks' ? 'datapacks' : 'resourcepacks';
		try {
			await instancePackOpenFolder(instanceId, packType);
		} catch (e) {
			toast("Erro ao abrir pasta: " + String(e), "error");
		}
	}

	async function handleDeleteScreenshot(path: string) {
		try {
			await screenshotDelete(path);
			playSound("delete");
			screenshotsList = screenshotsList.filter(s => s.path !== path);
			toast("Captura de tela removida!", "info");
		} catch (e) {
			toast("Erro ao excluir captura: " + String(e), "error");
		}
	}

	async function checkModpackUpdate() {
		if (!instanceId || isCheckingUpdate) return;
		isCheckingUpdate = true;
		try {
			const info = await modpackCheckUpdate(instanceId);
			if (info && info.hasUpdate) {
				modpackUpdate = info;
			} else {
				modpackUpdate = null;
			}
		} catch (e) {
			console.debug("Modpack update check not applicable or failed:", e);
		} finally {
			isCheckingUpdate = false;
		}
	}

	async function handleApplyModpackUpdate() {
		if (!modpackUpdate || !instanceId || isUpdatingModpack) return;
        if (appState.isGameRunning) { toast("Feche o Minecraft antes de atualizar o modpack.", "warning"); return; }
		isUpdatingModpack = true;
		updateStatusText = "Iniciando atualização protegida...";
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

			toast(`🎉 Modpack atualizado com sucesso para a versão ${modpackUpdate.latestVersion || "mais recente"}! Seus mundos, prints e opções foram 100% preservados.`, "success");
			playSound("chime");
			modpackUpdate = null;
			await profiles.refresh();
            await refreshAllData();
		} catch (e) {
			console.error("Falha ao atualizar modpack:", e);
			toast("Erro ao atualizar modpack: " + String(e), "error");
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
			toast("Erro ao abrir pasta: " + String(e), "error");
		}
	}

	async function openScreenshotsFolder() {
		if (!instanceId) return;
		try {
			await screenshotsOpenFolder(instanceId);
		} catch (e) {
			toast("Erro ao abrir pasta de capturas: " + String(e), "error");
		}
	}
</script>

<div class="flex gap-8 h-full w-full select-none">

	<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-2 space-y-5">

		<a href="/instances" class="flex items-center gap-2 text-xs font-bold text-fg/50 hover:text-fg transition-colors w-fit group">
			<ArrowLeft class="w-3.5 h-3.5 transition-transform group-hover:-translate-x-1" /> Voltar
		</a>

        <InstanceHero profile={activeProfile} banner={heroBanner} launching={isLaunching} running={appState.isGameRunning} status={launchStatusText} progress={downloadProgressPercent}
            javaLabel={javaRuntimes.find(runtime => runtime.path === activeProfile?.javaPath)?.versionString || (activeProfile?.javaPath ? 'Personalizado' : 'Automático')}
            onPlay={() => handlePlay()} onStop={handleStopGame} onSettings={() => showInstanceSettingsModal = true} onHost={openHostWorldModal}>
            {#snippet actions()}
                <button type="button" class={button({ variant: 'ghost', size: 'sm' })} onclick={openInstanceFolder}><FolderOpen class="h-4 w-4" />Abrir pasta</button>
                <button type="button" class={button({ variant: 'ghost', size: 'sm' })} onclick={() => showP2PHost = true}><Radio class="h-4 w-4" />Host P2P</button>
                <button type="button" class={button({ variant: 'ghost', size: 'sm' })} onclick={() => showModpackExportModal = true}><Package class="h-4 w-4" />Exportar pack</button>
                <button type="button" class={button({ variant: 'ghost', size: 'sm' })} onclick={() => showWorldBackupModal = true}><HardDrive class="h-4 w-4" />Backups</button>
                <button type="button" class={button({ variant: 'ghost', size: 'sm' })} onclick={() => showDeathDetectorModal = true}><Skull class="h-4 w-4" />Última morte</button>
            {/snippet}
        </InstanceHero>

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
								<span class="font-extrabold text-sm text-fg">Nova Versão do Modpack Disponível!</span>
								<span class="text-[10px] font-mono font-bold bg-amber-500/20 text-amber-300 border border-amber-500/30 px-2 py-0.5 rounded-full">
									{modpackUpdate?.latestVersion || "Atualização Oficial"}
								</span>
								{#if modpackUpdate?.currentVersion}
									<span class="text-[10px] text-fg/40 font-mono">
										(Instalado: {modpackUpdate.currentVersion})
									</span>
								{/if}
							</div>
							<p class="text-xs text-fg/60">
								Uma atualização oficial foi detectada no {modpackUpdate?.source === 'curseforge' ? 'CurseForge' : 'Modrinth'}.
							</p>
							<div class="flex items-center gap-1.5 text-[11px] font-bold text-emerald-400 pt-0.5">
								<ShieldCheck class="w-3.5 h-3.5 shrink-0" />
								<span>Seus mundos (saves/), prints (screenshots/) e opções (options.txt) serão 100% preservados.</span>
							</div>
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
									<div class="h-full bg-gradient-to-r from-amber-500 to-orange-500 transition-all duration-300 rounded-full" style="width: {updateProgressPercent}%"></div>
								</div>
							</div>
						{:else}
							<button
								type="button"
								class="w-full md:w-auto px-6 py-3 rounded-2xl bg-gradient-to-r from-amber-500 to-orange-500 hover:from-amber-400 hover:to-orange-400 text-brand-foreground font-black text-xs shadow-lg hover:shadow-amber-500/25 transition-all hover:scale-105 active:scale-[0.98] flex items-center justify-center gap-2 cursor-pointer"
								onclick={handleApplyModpackUpdate}
							>
								<Download class="w-4 h-4" /> Atualizar em 1 Clique
							</button>
						{/if}
					</div>
				</div>
			</div>
		{/if}

		<div class="flex flex-wrap items-center justify-between gap-2">
            <div class="section-tabs" role="tablist" aria-label="Conteúdo da instância">
                {#each [{ id: 'conteudo', label: 'Mods e conteúdo', icon: Layers }, { id: 'mundos', label: `Mundos · ${worldsList.length}`, icon: Globe2 }, { id: 'galeria', label: `Screenshots · ${screenshotsList.length}`, icon: Image }, { id: 'ficheiros', label: 'Arquivos', icon: Folder }] as tab}
                    <button type="button" role="tab" class="section-tab" aria-selected={mainTab === tab.id} onclick={() => mainTab = tab.id as typeof mainTab}><tab.icon class="h-4 w-4" />{tab.label}</button>
                {/each}
                <button type="button" role="tab" class="section-tab" aria-selected={mainTab === "configuracoes"} onclick={() => mainTab = "configuracoes"}><SettingsIcon class="h-4 w-4" />Configurações</button>
            </div>
            <button type="button" class={button({ variant: 'ghost', size: 'icon' })} aria-label="Atualizar dados" onclick={refreshAllData}><RefreshCw class="h-4 w-4 {isLoadingData ? 'animate-spin' : ''}" /></button>
        </div>

		{#if mainTab === 'conteudo'}
			<div class="flex flex-col gap-4" in:fade={{ duration: 150 }}>
				<div class="flex flex-wrap items-center justify-between gap-3">
					<div class="flex bg-bg-elevated border border-fg/10 rounded-full p-1 gap-1">
						<button
							class="px-4 py-1.5 rounded-full text-xs font-bold transition-all flex items-center gap-1.5 {subTab === 'mods' ? 'bg-bg-subtle text-fg shadow-sm border border-fg/10' : 'text-fg/70 hover:text-fg hover:bg-bg-subtle'}"
							onclick={() => subTab = 'mods'}
						>
							<Puzzle class="w-3.5 h-3.5 text-blue-400" /> Mods ({instanceMods.length})
						</button>
						<button
							class="px-4 py-1.5 rounded-full text-xs font-bold transition-all flex items-center gap-1.5 {subTab === 'resourcepacks' ? 'bg-bg-subtle text-fg shadow-sm border border-fg/10' : 'text-fg/70 hover:text-fg hover:bg-bg-subtle'}"
							onclick={() => subTab = 'resourcepacks'}
						>
							<Box class="w-3.5 h-3.5 text-amber-400" /> Pacotes de recursos ({resourcePacks.length})
						</button>
						<button
							class="px-4 py-1.5 rounded-full text-xs font-bold transition-all flex items-center gap-1.5 {subTab === 'shaders' ? 'bg-bg-subtle text-fg shadow-sm border border-fg/10' : 'text-fg/70 hover:text-fg hover:bg-bg-subtle'}"
							onclick={() => subTab = 'shaders'}
						>
							<Sparkles class="w-3.5 h-3.5 text-purple-400" /> Shaders ({shaderPacks.length})
						</button>
						<button
							class="px-4 py-1.5 rounded-full text-xs font-bold transition-all flex items-center gap-1.5 {subTab === 'datapacks' ? 'bg-bg-subtle text-fg shadow-sm border border-fg/10' : 'text-fg/70 hover:text-fg hover:bg-bg-subtle'}"
							onclick={() => subTab = 'datapacks'}
						>
							<Code class="w-3.5 h-3.5 text-emerald-400" /> {"{}"} Datapacks ({dataPacks.length})
						</button>
					</div>

					<div class="flex items-center gap-2 flex-wrap">
						{#if subTab === 'mods'}
							<button
								class="bg-bg-subtle hover:bg-fg/10 text-fg/80 hover:text-fg px-4 py-2 rounded-full border border-fg/10 text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer"
								onclick={handleOpenModsFolder}
							>
								<FolderOpen class="w-3.5 h-3.5" /> Abrir Pasta mods/
							</button>
							<button
								type="button"
								class="bg-bg-subtle hover:bg-fg/10 text-fg/80 hover:text-fg px-4 py-2 rounded-full border border-fg/10 text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer disabled:opacity-50"
								onclick={handleCheckModUpdates}
								disabled={isCheckingUpdates}
							>
								<RefreshCw class="w-3.5 h-3.5 {isCheckingUpdates ? 'animate-spin' : ''}" />
								{isCheckingUpdates ? 'Verificando...' : 'Verificar Atualizações'}
							</button>
							<a
								href="/mods"
								class="bg-bg-subtle hover:bg-fg/10 text-fg/80 hover:text-fg px-4 py-2 rounded-full border border-fg/10 text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer"
							>
								<Search class="w-3.5 h-3.5" /> Obter Mais Mods
							</a>
							<button
								class="text-brand-foreground px-5 py-2 rounded-full text-xs font-black flex items-center gap-1.5 shadow-sm cursor-pointer hover:scale-105 active:scale-[0.98] transition-all"
								style="background-color: rgb(var(--brand-500));"
								onclick={handleAddModFile}
							>
								<Plus class="w-4 h-4 stroke-[3]" /> Adicionar .JAR
							</button>
						{:else}
							<button
								class="bg-bg-subtle hover:bg-fg/10 text-fg/70 hover:text-fg px-4 py-2 rounded-full text-xs font-bold flex items-center gap-1.5 border border-fg/5 transition-all cursor-pointer"
								onclick={handleOpenPackFolder}
							>
								<FolderOpen class="w-3.5 h-3.5" /> Abrir Pasta
							</button>
							<button
								class="text-brand-foreground px-5 py-2 rounded-full text-xs font-black flex items-center gap-1.5 shadow-sm cursor-pointer hover:scale-105 active:scale-[0.98] transition-all"
								style="background-color: rgb(var(--brand-500));"
								onclick={handleAddResourcePack}
							>
								<Plus class="w-4 h-4" /> Adicionar .ZIP
							</button>
						{/if}
					</div>
				</div>

				{#if subTab === 'mods'}
					{#if shieldResult && !shieldResult.isClean}
						<div class="bg-red-500/15 border border-red-500/40 rounded-2xl p-4 flex items-center justify-between gap-3 mb-4 text-red-200 shadow-md">
							<div class="flex items-center gap-3">
								<div class="w-9 h-9 rounded-xl bg-red-500/20 text-red-400 border border-red-500/30 flex items-center justify-center shrink-0">
									<ShieldAlert class="w-5 h-5" />
								</div>
								<div>
									<div class="text-xs font-black text-red-100 flex items-center gap-2">
										Luxmc Shield · {shieldResult.threats.length} Ameaça(s) Crítica(s) Detectada(s)!
									</div>
									<div class="text-[11px] text-red-200/80 mt-0.5">
										Foram encontrados arquivos maliciosos ou suspeitos de roubo de sessão / CVEs. Desative ou remova os mods indicados.
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
										{availableModUpdates.length} Mod{availableModUpdates.length > 1 ? 's' : ''} com Atualização Disponível
									</h4>
									<p class="text-[11px] text-fg-muted">
										Atualize todos de uma vez ou atualize individualmente os que desejar na lista abaixo.
									</p>
								</div>
							</div>
							<button
								type="button"
								class="bg-amber-500 hover:bg-amber-400 text-bg font-extrabold text-xs px-4 py-2 rounded-xl transition-all shadow-md active:scale-95 flex items-center gap-2 cursor-pointer disabled:opacity-50 shrink-0"
								disabled={isUpdatingAllMods}
								onclick={handleUpdateAllMods}
							>
								<RefreshCw class="w-4 h-4 {isUpdatingAllMods ? 'animate-spin' : ''}" />
								{isUpdatingAllMods ? 'Atualizando Mods...' : `Atualizar Todos os Mods (${availableModUpdates.length})`}
							</button>
						</div>
					{/if}

					{#if instanceMods.length === 0}
						<div class="bg-bg-elevated border border-fg/5 rounded-3xl p-16 flex flex-col items-center justify-center text-center">
							<div class="h-16 w-16 rounded-full bg-blue-500/10 border border-blue-500/20 flex items-center justify-center mb-4 text-blue-400">
								<Puzzle class="w-8 h-8" />
							</div>
							<h3 class="text-base font-extrabold text-fg">Nenhum mod instalado nesta instância</h3>
							<p class="text-xs text-fg/40 mt-1 max-w-md">
								Você pode instalar mods incríveis diretamente pela Central de Conteúdo ou importar arquivos .jar do seu computador.
							</p>
							<div class="flex items-center gap-3 mt-6">
								<a
									href="/mods"
									class="text-brand-foreground px-6 py-2.5 rounded-full text-xs font-black transition-all hover:scale-105 active:scale-[0.98] shadow-md flex items-center gap-2"
									style="background-color: rgb(var(--brand-500));"
								>
									<Sparkles class="w-4 h-4" /> Baixar Mods na Central
								</a>
								<button
									class="bg-bg-subtle hover:bg-fg/10 border border-fg/10 text-fg text-xs font-bold px-6 py-2.5 rounded-full transition-all flex items-center gap-2 cursor-pointer"
									onclick={handleAddModFile}
								>
									<Plus class="w-4 h-4" /> Importar .JAR Local
								</button>
							</div>
						</div>
					{:else}
						<!-- Modrinth Table Header -->
						<div class="grid grid-cols-[auto_1fr_180px_160px] items-center gap-4 px-4 py-3 border-b border-fg/10 text-xs font-semibold text-fg/50 select-none bg-bg-elevated/80 rounded-t-2xl">
							<div class="flex items-center gap-3">
								<input
									type="checkbox"
									checked={allModsSelected}
									onchange={toggleSelectAllMods}
									class="w-4 h-4 rounded border border-fg/20 bg-bg-subtle accent-emerald-500 cursor-pointer"
								/>
								<span>Projeto</span>
							</div>
							<div></div>
							<div class="text-left font-medium">Versão</div>
							<div class="text-right font-medium pr-2">Ações</div>
						</div>

						<VirtualList items={filteredMods} itemHeight={76} height="600px" class="rounded-b-2xl border-x border-b border-fg/5 bg-bg-elevated/40">
							{#snippet children(mod: FileTreeEntry, _index: number)}
								{@const isDisabled = mod.name.endsWith('.disabled')}
								{@const rawName = mod.name.replace('.disabled', '').replace('.jar', '')}
								{@const meta = parseModMeta(mod.name)}
								{@const displayName = rawName.includes('_') && /^\d+_\d+$/.test(rawName) ? 'Mod #' + rawName.split('_')[0] : meta.title}
								{@const badge = getModBadge(displayName)}
								{@const isFrozen = frozenModNames.includes(mod.name)}
								{@const isSelected = selectedModNames.includes(mod.name)}
								{@const modUpdate = availableModUpdates.find(u => 
									(u.fileName && (mod.name === u.fileName || mod.name === u.fileName + '.disabled' || mod.name.startsWith(u.fileName.replace('.jar', '')))) ||
									(u.projectName && displayName.toLowerCase().includes(u.projectName.toLowerCase())) ||
									(u.projectId && displayName.toLowerCase().includes(u.projectId.toLowerCase()))
								)}
								<div class="grid grid-cols-[auto_1fr_180px_160px] items-center gap-4 px-4 py-3 hover:bg-fg/[0.03] border-b border-fg/5 transition-colors group {isDisabled ? 'opacity-50' : ''}" style="content-visibility: auto;">
									<!-- Checkbox & Project Info -->
									<div class="flex items-center gap-3.5 min-w-0">
										<input
											type="checkbox"
											checked={isSelected}
											onchange={() => toggleModSelection(mod.name)}
											class="w-4 h-4 rounded border border-fg/20 bg-bg-subtle accent-emerald-500 cursor-pointer shrink-0"
										/>
										<div class="w-10 h-10 rounded-xl overflow-hidden flex items-center justify-center shrink-0 bg-bg-subtle border border-fg/10 relative shadow-sm {isDisabled ? 'grayscale opacity-60' : ''}">
											{#if mod.icon}
												<img
													src={mod.icon}
													alt={displayName}
													loading="lazy"
													class="w-full h-full object-cover [image-rendering:pixelated]"
													onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
												/>
											{:else}
												<div class="w-full h-full bg-gradient-to-br {badge.theme} flex items-center justify-center select-none">
													<span class="text-xs font-black tracking-tight drop-shadow-sm">{badge.initials}</span>
												</div>
											{/if}
										</div>
										<div class="min-w-0">
											<div class="flex items-center gap-2">
												<h5 class="text-xs font-bold text-fg truncate max-w-[280px] group-hover:text-emerald-400 transition-colors" title={displayName}>
													{displayName}
												</h5>
												{#if modUpdate}
													<span class="text-[8px] font-black uppercase px-1.5 py-0.5 rounded-full bg-amber-500/15 text-amber-400 border border-amber-500/30 animate-pulse" title={`Atualização disponível: ${modUpdate.latestVersionNumber}`}>
														v{modUpdate.latestVersionNumber}
													</span>
												{/if}
											</div>
											<div class="flex items-center gap-1.5 mt-0.5 text-[11px] text-fg/40">
												<span class="w-1.5 h-1.5 rounded-full bg-fg/30"></span>
												<span class="truncate max-w-[200px]">{meta.author}</span>
											</div>
										</div>
									</div>

									<div></div>

									<!-- Versão -->
									<div class="min-w-0">
										<div class="flex items-center gap-1.5">
											<span class="text-xs font-bold text-fg font-mono">{meta.version}</span>
											{#if isFrozen}
												<span title="Versão congelada">
													<Snowflake class="w-3.5 h-3.5 text-cyan-400 shrink-0" />
												</span>
											{/if}
										</div>
										<div class="text-[10px] text-fg/40 font-mono truncate max-w-[160px]" title={mod.name}>
											{mod.name.replace('.disabled', '')}
										</div>
									</div>

									<div class="flex items-center gap-2 justify-end relative shrink-0">
										<button
											type="button"
											onclick={() => handleSwapVersion(mod)}
											class="p-1.5 rounded-lg text-fg/50 hover:text-fg hover:bg-fg/10 transition-colors cursor-pointer"
											title="Trocar versão"
										>
											<ArrowLeftRight class="w-4 h-4" />
										</button>

										<button
											type="button"
											onclick={() => handleToggleMod(mod)}
											class="w-10 h-5 rounded-full transition-colors relative cursor-pointer shadow-inner {isDisabled ? 'bg-fg/20' : 'bg-emerald-500'}"
											title={isDisabled ? 'Ativar mod' : 'Desativar mod'}
											role="switch"
											aria-checked={!isDisabled}
										>
											<span class="absolute top-0.5 w-4 h-4 rounded-full bg-white transition-all shadow-md {isDisabled ? 'left-0.5' : 'left-[22px]'}"></span>
										</button>

										<button
											type="button"
											onclick={() => handleDeleteMod(mod)}
											class="p-1.5 rounded-lg text-fg/50 hover:text-red-400 hover:bg-red-500/10 transition-colors cursor-pointer"
											title="Excluir mod"
										>
											<Trash2 class="w-4 h-4" />
										</button>

										<div class="relative">
											<button
												type="button"
												onclick={(e) => {
													e.stopPropagation();
													activeMenuMod = activeMenuMod === mod.name ? null : mod.name;
												}}
												class="p-1.5 rounded-lg text-fg/50 hover:text-fg hover:bg-fg/10 transition-colors cursor-pointer"
												title="Mais opções"
											>
												<MoreVertical class="w-4 h-4" />
											</button>

											{#if activeMenuMod === mod.name}
												<div
													class="absolute right-0 top-full mt-1.5 w-44 rounded-2xl bg-bg-elevated/95 backdrop-blur-xl border border-fg/10 p-1.5 shadow-2xl z-30 space-y-0.5"
													transition:scale={{ duration: 120, start: 0.95 }}
												>
													<button
														type="button"
														onclick={() => {
															activeMenuMod = null;
															handleSyncMod(mod);
														}}
														class="w-full flex items-center gap-2 px-3 py-2 rounded-xl text-xs font-medium text-fg/80 hover:text-fg hover:bg-fg/5 transition-colors text-left cursor-pointer"
													>
														<RefreshCw class="w-3.5 h-3.5 text-emerald-400" /> Sincronizar
													</button>
													<button
														type="button"
														onclick={() => {
															activeMenuMod = null;
															handleShowModFile(mod);
														}}
														class="w-full flex items-center gap-2 px-3 py-2 rounded-xl text-xs font-medium text-fg/80 hover:text-fg hover:bg-fg/5 transition-colors text-left cursor-pointer"
													>
														<FolderOpen class="w-3.5 h-3.5 text-blue-400" /> Exibir arquivo
													</button>
													<button
														type="button"
														onclick={() => {
															activeMenuMod = null;
															handleCopyModLink(mod);
														}}
														class="w-full flex items-center gap-2 px-3 py-2 rounded-xl text-xs font-medium text-fg/80 hover:text-fg hover:bg-fg/5 transition-colors text-left cursor-pointer"
													>
														<Link class="w-3.5 h-3.5 text-amber-400" /> Copiar link
													</button>
													<button
														type="button"
														onclick={() => {
															activeMenuMod = null;
															handleToggleFreeze(mod);
														}}
														class="w-full flex items-center gap-2 px-3 py-2 rounded-xl text-xs font-medium text-fg/80 hover:text-fg hover:bg-fg/5 transition-colors text-left cursor-pointer"
													>
														<Snowflake class="w-3.5 h-3.5 text-cyan-400" /> {isFrozen ? "Descongelar versão" : "Congelar versão"}
													</button>
												</div>
											{/if}
										</div>
									</div>
								</div>
							{/snippet}
						</VirtualList>
					{/if}
				{:else}
					<div class="flex items-center justify-between mb-3">
						<h4 class="text-xs font-bold text-fg uppercase tracking-wider">
							{subTab === 'resourcepacks' ? 'Pacotes de Textura Instalados' : subTab === 'shaders' ? 'Shaders Instalados' : 'Datapacks Instalados'} ({currentPacksList.length})
						</h4>
						<div class="flex items-center gap-2">
							<button
								type="button"
								class="bg-fg/5 hover:bg-fg/10 text-fg/80 hover:text-fg px-3 py-1.5 rounded-xl border border-fg/10 text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer shadow-sm active:scale-[0.98]"
								onclick={handleAddResourcePack}
							>
								<Plus class="w-3.5 h-3.5" /> Importar .ZIP Local
							</button>
							<button
								class="text-xs font-bold text-fg/40 hover:text-fg flex items-center gap-1 cursor-pointer transition-colors"
								onclick={handleOpenPackFolder}
							>
								<FolderOpen class="w-3.5 h-3.5" /> Abrir pasta
							</button>
						</div>
					</div>

					{#if currentPacksList.length === 0}
						<div class="bg-bg-elevated border border-fg/5 rounded-3xl p-12 flex flex-col items-center justify-center text-center">
							<div class="h-14 w-14 rounded-full bg-fg/5 flex items-center justify-center mb-3 text-fg/20">
								<Box class="w-7 h-7" />
							</div>
							<h3 class="text-sm font-extrabold text-fg">
								{subTab === 'resourcepacks' ? 'Nenhum pacote de textura instalado' : subTab === 'shaders' ? 'Nenhum shader instalado' : 'Nenhum datapack instalado'}
							</h3>
							<p class="text-xs text-fg/40 mt-1 max-w-sm">
								{subTab === 'shaders' ? 'Clique em Importar .ZIP acima ou adicione shaders na pasta shaderpacks.' : subTab === 'resourcepacks' ? 'Clique em Importar .ZIP acima ou importe pacotes de textura do seu computador.' : 'Adicione datapacks para modificar o comportamento do jogo.'}
							</p>
						</div>
					{:else}
						<div class="grid grid-cols-1 md:grid-cols-2 gap-3">
							{#each currentPacksList as pack}
								<div class="bg-bg-elevated border border-fg/5 hover:border-fg/15 p-4 rounded-2xl flex items-center justify-between transition-all group shadow-sm">
									<div class="flex items-center gap-3.5 min-w-0">
										<div class="w-10 h-10 rounded-xl bg-amber-500/15 text-amber-400 border border-amber-500/30 flex items-center justify-center shrink-0 shadow-sm">
											<Box class="w-5 h-5" />
										</div>
										<div class="min-w-0">
											<span class="text-xs font-bold text-fg truncate block max-w-[220px]" title={pack.name}>{pack.name}</span>
											<span class="text-[10px] text-fg/40 font-mono mt-0.5 block">{pack.size > 1048576 ? (pack.size / (1024 * 1024)).toFixed(2) + ' MB' : Math.round(pack.size / 1024) + ' KB'}</span>
										</div>
									</div>
									<button
										type="button"
										class="p-2 rounded-xl text-fg/70 hover:text-red-300 hover:bg-red-500/20 border border-fg/10 hover:border-red-500/40 bg-bg-subtle transition-all cursor-pointer active:scale-[0.98] shadow-sm"
										onclick={() => handleDeletePack(pack.name)}
										title="Excluir arquivo"
									>
										<Trash2 class="w-4 h-4" />
									</button>
								</div>
							{/each}
						</div>
					{/if}
				{/if}

			</div>

        {:else if mainTab === 'configuracoes'}
            <section class="surface-glass space-y-6 p-6" in:fade={{ duration: 150 }}>
                <div class="flex flex-wrap items-center justify-between gap-4"><div><h2 class="text-lg font-semibold text-fg">Do seu jeito</h2><p class="mt-1 text-xs text-fg-muted">Memória e Java exclusivos desta instância.</p></div><button type="button" class={button({ variant: 'secondary', size: 'sm' })} onclick={() => showInstanceSettingsModal = true}><SettingsIcon class="h-4 w-4" />Todas as configurações</button></div>
                <div class="grid gap-6 md:grid-cols-2">
                    <div class="space-y-4 rounded-2xl border border-fg/5 bg-bg/30 p-5">
                        <h3 class="text-sm font-semibold text-fg">Alocação de memória</h3>
                        <label for="inline-ram-min" class="flex justify-between text-xs text-fg-muted"><span>Mínima</span><span>{(instanceMinRamMb / 1024).toFixed(1)} GB</span></label>
                        <input id="inline-ram-min" type="range" min="512" max={instanceRamMb} step="256" bind:value={instanceMinRamMb} class="w-full accent-brand-500" />
                        <label for="inline-ram-max" class="flex justify-between text-xs text-fg-muted"><span>Máxima</span><span>{(instanceRamMb / 1024).toFixed(1)} GB</span></label>
                        <input id="inline-ram-max" type="range" min="1024" max={Math.max(1024, Math.floor(systemRamMb / 256) * 256)} step="256" bind:value={instanceRamMb} class="w-full accent-brand-500" />
                        <p class="text-xs text-fg-subtle">{(systemRamMb / 1024).toFixed(1)} GB no sistema. Reserve memória para os outros aplicativos.</p>
                    </div>
                    <div class="space-y-4 rounded-2xl border border-fg/5 bg-bg/30 p-5">
                        <label for="inline-java" class="block text-sm font-semibold text-fg">Instalação Java</label>
                        <select id="inline-java" bind:value={instanceJavaPath} class="w-full rounded-xl border border-border bg-bg-subtle px-3 py-3 text-xs text-fg"><option value="">Automático · compatível com o Minecraft</option>{#each javaRuntimes as runtime}{#if runtime.path}<option value={runtime.path}>Java {runtime.major} · {runtime.versionString || runtime.path}</option>{/if}{/each}{#if instanceJavaPath && !javaRuntimes.some(runtime => runtime.path === instanceJavaPath)}<option value={instanceJavaPath}>{instanceJavaPath}</option>{/if}</select>
                        <button type="button" class={button({ variant: 'secondary', size: 'sm' })} onclick={detectJava} disabled={scanningJava}><RefreshCw class="h-3.5 w-3.5 {scanningJava ? 'animate-spin' : ''}" />{scanningJava ? 'Detectando…' : 'Detectar instalações'}</button>
                        <p class="text-xs text-fg-subtle">O modo automático escolhe o Java necessário para a versão do jogo.</p>
                    </div>
                </div>
                <div class="space-y-3"><label for="inline-jvm" class="text-sm font-semibold text-fg">Argumentos JVM</label><div class="flex flex-wrap gap-2">{#each ['aikar', 'zgc', 'shenandoah'] as preset}<button type="button" class={button({ variant: 'outline', size: 'sm' })} onclick={() => applyJvmPreset(preset as 'aikar' | 'zgc' | 'shenandoah')}>{preset.toUpperCase()}</button>{/each}</div><textarea id="inline-jvm" bind:value={instanceJvmArgs} rows="3" class="w-full rounded-xl border-border bg-bg-subtle font-mono text-xs text-fg" placeholder="Argumentos adicionais da JVM"></textarea><p class="text-xs text-fg-subtle">Aikar usa G1GC. ZGC e Shenandoah dependem do suporte da instalação Java.</p></div>
                <div class="flex justify-end border-t border-fg/5 pt-4"><button type="button" class={button({ variant: 'primary' })} onclick={saveInstanceSettings}>Salvar configurações</button></div>
            </section>
		{:else if mainTab === 'mundos'}
			<div class="space-y-4">
				<div class="flex items-center justify-between flex-wrap gap-2">
					<h3 class="text-sm font-bold text-fg">Mundos Salvos nesta Instância</h3>
					<div class="flex items-center flex-wrap gap-2">
						<button
							type="button"
							class="bg-brand-500 hover:bg-brand-400 text-brand-foreground px-3.5 py-1.5 rounded-full text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer shadow-md active:scale-95 disabled:opacity-50"
							onclick={handleImportWorld}
							disabled={isImportingWorld}
							title="Importar Mapa baixado em arquivo .zip"
						>
							<Download class="w-3.5 h-3.5" />
							<span>{isImportingWorld ? "Importando..." : "Importar Mundo (.zip)"}</span>
						</button>
						<button
							type="button"
							class="bg-bg-subtle hover:bg-fg/10 text-fg/90 hover:text-fg px-3 py-1.5 rounded-full border border-fg/15 text-xs font-bold flex items-center gap-1.5 cursor-pointer transition-all shadow-sm active:scale-95 disabled:opacity-50"
							onclick={handleImportWorldFolder}
							disabled={isImportingWorld}
							title="Importar pasta descompactada de save"
						>
							<FolderPlus class="w-3.5 h-3.5 text-amber-400" />
							<span>Importar Pasta</span>
						</button>
						<button
							type="button"
							class="bg-bg-subtle hover:bg-cyan-500/20 text-fg/90 hover:text-cyan-300 px-3.5 py-1.5 rounded-full border border-fg/15 hover:border-cyan-500/40 text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer shadow-sm"
							onclick={() => showWorldBackupModal = true}
							title="Nuvem Pessoal & Backups Automáticos de Saves"
						>
							<HardDrive class="w-3.5 h-3.5 text-cyan-400" /> Backups de Saves ({worldsList.length})
						</button>
						<button class="bg-bg-subtle hover:bg-fg/10 text-fg/90 hover:text-fg px-3.5 py-1.5 rounded-full border border-fg/15 text-xs font-bold flex items-center gap-1.5 cursor-pointer transition-all shadow-sm" onclick={openInstanceFolder}>
							<FolderOpen class="w-3.5 h-3.5 text-emerald-400" /> Abrir pasta saves/
						</button>
					</div>
				</div>

				{#if worldsList.length === 0}
					<div class="bg-bg-elevated border border-fg/5 rounded-3xl p-16 flex flex-col items-center justify-center text-center gap-4">
						<Globe2 class="w-12 h-12 text-fg/20" />
						<div>
							<h4 class="text-sm font-bold text-fg">Nenhum mundo encontrado</h4>
							<p class="text-xs text-fg/40 mt-1">Abra o Minecraft e crie um mundo, ou importe um mapa baixado (.zip ou pasta)!</p>
						</div>
						<div class="flex items-center gap-2 mt-2">
							<button
								type="button"
								class="bg-brand-500 hover:bg-brand-400 text-brand-foreground px-4 py-2 rounded-xl text-xs font-bold flex items-center gap-2 cursor-pointer shadow-md active:scale-95"
								onclick={handleImportWorld}
							>
								<Download class="w-4 h-4" /> Importar Mundo (.zip)
							</button>
							<button
								type="button"
								class="bg-bg-subtle hover:bg-fg/10 text-fg px-4 py-2 rounded-xl text-xs font-bold border border-fg/10 flex items-center gap-2 cursor-pointer active:scale-95"
								onclick={handleImportWorldFolder}
							>
								<FolderPlus class="w-4 h-4 text-amber-400" /> Importar Pasta
							</button>
						</div>
					</div>
				{:else}
					<div class="grid grid-cols-1 md:grid-cols-2 gap-3.5">
						{#each worldsList as world}
							<div class="bg-bg-elevated border border-fg/5 p-4 rounded-2xl flex flex-col justify-between hover:border-fg/15 transition-all group gap-3">
								<div class="flex items-start gap-3.5 min-w-0">
									<div class="h-14 w-14 rounded-xl bg-bg-overlay/40 border border-fg/10 overflow-hidden flex items-center justify-center shrink-0 shadow-md">
										{#if world.iconBase64}
											<img src={world.iconBase64} alt={world.name} class="w-full h-full object-cover [image-rendering:pixelated]" />
										{:else}
											<img src="/grass_block.png" alt="Mundo" class="w-8 h-8 object-contain drop-shadow" />
										{/if}
									</div>
									<div class="min-w-0 flex-1">
										<div class="flex items-center gap-2">
											<h5 class="text-xs font-bold text-fg truncate">{world.name}</h5>
											{#if world.hardcore}
												<span class="px-1.5 py-0.5 rounded text-[9px] font-black bg-red-500/20 text-red-400 border border-red-500/30">HARDCORE</span>
											{/if}
										</div>
										<div class="flex flex-wrap items-center gap-1.5 mt-1 text-[10px] text-fg/50">
											<span class="text-emerald-400 font-semibold">{world.gameMode || 'Sobrevivência'}</span>
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
													class="flex items-center gap-1 bg-fg/5 hover:bg-fg/10 text-fg/70 hover:text-fg px-2 py-0.5 rounded-md border border-fg/5 transition-colors cursor-pointer"
													onclick={() => {
														if (world.seed != null) {
															navigator.clipboard.writeText(world.seed.toString());
															toast(`Seed copiada: ${world.seed}`, "success");
														}
													}}
													title="Copiar Seed"
												>
													<Compass class="w-3 h-3 text-brand-500" />
													<span>Seed: {world.seed}</span>
													<Copy class="w-2.5 h-2.5 opacity-50" />
												</button>
											{/if}
											{#if world.spawnX != null && world.spawnZ != null}
												<div class="flex items-center gap-1 text-fg/40" title="Coordenadas de Spawn">
													<MapPin class="w-3 h-3 text-emerald-400" />
													<span>Spawn: {world.spawnX}, {world.spawnY ?? 64}, {world.spawnZ}</span>
												</div>
											{/if}
											{#if world.dayCount != null}
												<span class="text-amber-300/70">Dia {world.dayCount}</span>
											{/if}
											{#if world.playerHealth != null}
												<span class="text-rose-400 font-semibold">❤️ {Math.round(world.playerHealth)}/20</span>
											{/if}
											{#if world.playerLevel != null && world.playerLevel > 0}
												<span class="text-emerald-300 font-semibold">⭐ Nvl {world.playerLevel}</span>
											{/if}
										</div>

										{#if world.playerInventory && world.playerInventory.length > 0}
											<div class="flex flex-wrap items-center gap-1.5 mt-2 pt-2 border-t border-fg/5">
												<span class="text-[9px] uppercase tracking-wider text-fg/40 font-bold mr-1">Inventário:</span>
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
											class="bg-bg-subtle hover:bg-brand-500/20 text-fg/80 hover:text-brand-400 px-3 py-1.5 rounded-full text-[11px] font-bold transition-all cursor-pointer flex items-center gap-1.5 border border-fg/10 hover:border-brand-500/30 active:scale-[0.98]"
											onclick={() => selectedSnapshotWorld = { name: world.name, folder: world.folderName }}
											title="Time Machine: Gerenciar snapshots e backups deste mundo"
										>
											<Archive class="w-3 h-3 text-brand-500" />
											<span>Snapshots {world.snapshotsCount != null && world.snapshotsCount > 0 ? `(${world.snapshotsCount})` : ''}</span>
										</button>
										<button
											type="button"
											class="bg-bg-subtle hover:bg-purple-500/20 text-fg/80 hover:text-purple-300 px-2.5 py-1.5 rounded-full text-[11px] font-bold transition-all cursor-pointer flex items-center gap-1.5 border border-fg/10 hover:border-purple-500/30 active:scale-[0.98]"
											onclick={openHostWorldModal}
											title="Hospedar este mundo para amigos via P2P / UPnP"
										>
											<Radio class="w-3 h-3 text-purple-400" />
											<span>Hospedar</span>
										</button>
									</div>

									<div class="flex items-center gap-2">
										<button
											type="button"
											class="bg-emerald-500/20 hover:bg-emerald-500 text-emerald-300 hover:text-brand-foreground border border-emerald-500/40 px-4 py-1.5 rounded-full text-[11px] font-bold transition-all cursor-pointer flex items-center gap-1.5 active:scale-[0.98] shadow-sm"
											onclick={() => handlePlay()}
											title="Jogar este mundo"
										>
											<Play class="w-3 h-3 fill-current" /> Jogar
										</button>
										<button
											type="button"
											class="p-1.5 rounded-xl bg-bg-subtle text-fg/70 hover:text-red-400 hover:bg-red-500/20 border border-fg/10 hover:border-red-500/40 transition-all cursor-pointer active:scale-[0.98] shadow-sm"
											onclick={() => handleDeleteWorld(world.folderName)}
											title="Excluir este mundo"
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

		{:else if mainTab === 'galeria'}
			<div class="space-y-4">
				<div class="flex items-center justify-between">
					<h3 class="text-sm font-bold text-fg">Capturas de Tela (F2)</h3>
					<button class="text-xs font-bold hover:underline flex items-center gap-1 cursor-pointer" style="color: rgb(var(--brand-500));" onclick={openScreenshotsFolder}>
						<FolderOpen class="w-3.5 h-3.5" /> Abrir pasta screenshots/
					</button>
				</div>

				{#if screenshotsList.length === 0}
					<div class="bg-bg-elevated border border-fg/5 rounded-3xl p-16 flex flex-col items-center justify-center text-center">
						<Image class="w-12 h-12 text-fg/20 mb-3" />
						<h4 class="text-sm font-bold text-fg">Nenhuma captura de tela</h4>
						<p class="text-xs text-fg/40 mt-1">Pressione F2 dentro do jogo para capturar momentos épicos!</p>
					</div>
				{:else}
					<div class="columns-2 gap-4 xl:columns-3">
						{#each screenshotsList as shot}
							<div
								class="mb-4 break-inside-avoid bg-bg-elevated border border-fg/5 rounded-2xl overflow-hidden group relative cursor-pointer hover:border-brand-500/30 transition-all shadow-soft"
								onclick={() => previewScreenshot = shot}
								role="button"
								tabindex="0"
								onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); previewScreenshot = shot; } }}
							>
								<div class="bg-bg-overlay/60 overflow-hidden">
									<img
										src={shot.dataUrl || convertFileSrc(shot.path)}
										alt={shot.name}
										class="w-full h-auto object-contain group-hover:scale-105 transition-transform duration-300"
										loading="lazy"
									/>
								</div>
								<div class="p-3 flex items-center justify-between bg-bg-elevated/90">
									<span class="text-xs font-medium text-fg truncate max-w-[80%]">{shot.name}</span>
									<button
										class="text-fg/40 hover:text-red-400 transition-colors p-1 rounded-full hover:bg-fg/5"
										onclick={(e) => { e.stopPropagation(); handleDeleteScreenshot(shot.path); }}
										title="Excluir captura"
									>
										<Trash2 class="w-3.5 h-3.5" />
									</button>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>

		{:else if mainTab === 'ficheiros'}
			<div class="space-y-4">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						<h3 class="text-sm font-bold text-fg">Explorador de Arquivos</h3>
						<span class="text-[10px] text-fg/40 font-mono">({fileTree.length} itens)</span>
					</div>
					<button class="text-xs font-bold hover:underline flex items-center gap-1 cursor-pointer" style="color: rgb(var(--brand-500));" onclick={openInstanceFolder}>
						<FolderOpen class="w-3.5 h-3.5" /> Abrir no Gerenciador Linux
					</button>
				</div>

				<div class="bg-bg-elevated border border-fg/10 rounded-2xl p-3 flex items-center justify-between gap-3 shadow-md">
					<div class="flex items-center gap-1 text-xs font-mono overflow-x-auto custom-scrollbar py-0.5">
						<button
							type="button"
							class="text-xs font-bold px-2.5 py-1 rounded-full hover:bg-fg/10 text-fg/60 hover:text-fg transition-all cursor-pointer shrink-0"
							onclick={() => navigateBreadcrumb(-1)}
						>
							~ raiz
						</button>
						{#each fileBreadcrumbs as seg, idx}
							<ChevronRight class="w-3.5 h-3.5 text-fg/30 shrink-0" />
							<button
								type="button"
								class="text-xs font-bold px-2.5 py-1 rounded-full transition-all cursor-pointer shrink-0 {idx === fileBreadcrumbs.length - 1 ? 'bg-fg/10 text-fg' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}"
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
								class="p-1.5 rounded-full bg-fg/5 hover:bg-fg/15 text-fg transition-colors cursor-pointer"
								title="Subir nível"
								onclick={navigateUp}
							>
								<ArrowUp class="w-4 h-4" />
							</button>
						{/if}
						<button
							type="button"
							class="p-1.5 rounded-full bg-fg/5 hover:bg-fg/15 text-fg transition-colors cursor-pointer"
							title="Atualizar pasta"
							onclick={refreshAllData}
						>
							<RefreshCw class="w-4 h-4 {isLoadingData ? 'animate-spin' : ''}" />
						</button>
					</div>
				</div>

				<div class="bg-bg-elevated border border-fg/5 rounded-2xl p-2 space-y-1 shadow-md max-h-[500px] overflow-y-auto custom-scrollbar">
					{#if fileTree.length === 0}
						<div class="text-xs text-fg/40 py-8 text-center">Nenhum arquivo nesta pasta.</div>
					{:else}
						{#each fileTree as file}
							<div
								class="flex items-center justify-between p-2.5 hover:bg-fg/5 rounded-xl text-xs transition-colors group cursor-pointer"
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
										{file.isDir ? 'Pasta' : `${Math.round(file.size / 1024)} KB`}
									</span>
									<button
										type="button"
										class="p-1.5 rounded-lg bg-bg-subtle text-fg/70 hover:text-red-400 hover:bg-red-500/20 border border-fg/10 hover:border-red-500/40 transition-all opacity-0 group-hover:opacity-100 cursor-pointer shadow-sm"
										onclick={(e) => { e.stopPropagation(); handleDeleteFileEntry(file); }}
										title="Excluir"
									>
										<Trash2 class="w-3.5 h-3.5" />
									</button>
								</div>
							</div>
						{/each}
					{/if}
				</div>
			</div>
		{/if}

	</div>

	<RightSidebar />

</div>

{#if previewScreenshot}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="fixed inset-0 z-50 bg-bg-overlay/95 backdrop-blur-2xl flex flex-col items-center justify-center p-4 select-none" in:fade={{ duration: 150 }} onclick={() => previewScreenshot = null}>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="absolute top-6 left-6 right-6 flex items-center justify-between z-10" onclick={(e) => e.stopPropagation()}>
			<div class="flex items-center gap-2 min-w-0">
				<Image class="w-4 h-4 text-purple-400 shrink-0" />
				<span class="text-xs font-bold text-fg truncate max-w-sm drop-shadow">{previewScreenshot.name}</span>
			</div>
			<div class="flex items-center gap-2">
				<div class="flex items-center gap-1 bg-fg/10 backdrop-blur-md rounded-full px-2 py-1 border border-fg/10">
					<button type="button" class="p-1.5 text-fg/70 hover:text-fg rounded-full hover:bg-fg/10 cursor-pointer" onclick={zoomOutScreenshot} title="Diminuir Zoom (-)">
						<ZoomOut class="w-3.5 h-3.5" />
					</button>
					<button type="button" class="px-2 text-[10px] font-mono text-fg/90 hover:text-fg cursor-pointer" onclick={resetScreenshotZoom} title="Resetar Zoom">
						{Math.round(screenshotZoom * 100)}%
					</button>
					<button type="button" class="p-1.5 text-fg/70 hover:text-fg rounded-full hover:bg-fg/10 cursor-pointer" onclick={zoomInScreenshot} title="Aumentar Zoom (+)">
						<ZoomIn class="w-3.5 h-3.5" />
					</button>
				</div>
				<button
					type="button"
					class="p-2 rounded-full bg-fg/10 hover:bg-fg/20 text-fg transition-colors cursor-pointer"
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
			<img
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
				class="px-5 py-2 rounded-full bg-brand-500 hover:bg-brand-400 text-brand-foreground text-xs font-black flex items-center gap-2 shadow-lg shadow-brand-500/20 cursor-pointer transition-all"
				onclick={copyScreenshotImage}
			>
				<Copy class="w-3.5 h-3.5" /> Copiar Imagem (Ctrl+C)
			</button>
			<button
				type="button"
				class="px-4 py-2 rounded-full bg-fg/10 hover:bg-fg/20 text-fg text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer border border-fg/10"
				onclick={() => {
					navigator.clipboard.writeText(previewScreenshot!.path);
					toast("Caminho copiado!", "success");
				}}
			>
				<Copy class="w-3.5 h-3.5" /> Copiar Caminho
			</button>
			<button
				type="button"
				class="px-4 py-2 rounded-full bg-red-500/20 hover:bg-red-500/30 text-red-300 text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer border border-red-500/20"
				onclick={async () => {
					await handleDeleteScreenshot(previewScreenshot!.path);
					previewScreenshot = null;
				}}
			>
				<Trash2 class="w-3.5 h-3.5" /> Excluir
			</button>
		</div>
	</div>
{/if}

{#if showShareCodeModal && generatedShareCode}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/80 backdrop-blur-md" in:fade={{ duration: 150 }}>
		<div class="w-full max-w-md bg-bg-elevated border border-brand-500/30 rounded-3xl p-6 shadow-2xl">
			<div class="flex items-center justify-between mb-4">
				<div class="flex items-center gap-2">
					<Sparkles class="w-5 h-5 text-brand-500" />
					<h3 class="text-sm font-black text-fg">Compartilhar Instância</h3>
				</div>
				<button type="button" class="text-fg/40 hover:text-fg p-1 rounded-lg cursor-pointer" onclick={() => showShareCodeModal = false}>
					<X class="w-4 h-4" />
				</button>
			</div>

			<p class="text-xs text-fg/60 mb-4 leading-relaxed">
				Envie este código rápido para seus amigos. Eles só precisam clicar em <strong>"Importar por Código"</strong> na tela de instâncias para baixar a mesma versão, loader e mods!
			</p>

			<div class="bg-bg-overlay/50 border border-brand-500/40 rounded-2xl p-4 flex items-center justify-between mb-5">
				<span class="font-mono text-xl font-black text-brand-500 tracking-wider select-all">{generatedShareCode}</span>
				<button
					type="button"
					class="px-3.5 py-1.5 bg-brand-500 hover:bg-brand-400 text-brand-foreground text-xs font-black rounded-xl cursor-pointer flex items-center gap-1.5 transition-all"
					onclick={() => {
						navigator.clipboard.writeText(generatedShareCode!);
						toast("Código copiado para a área de transferência!", "success");
						playSound("chime");
					}}
				>
					<Copy class="w-3.5 h-3.5" /> Copiar
				</button>
			</div>

			<button
				type="button"
				class="w-full py-2.5 rounded-xl bg-fg/10 hover:bg-fg/15 text-xs font-bold text-fg transition-colors cursor-pointer"
				onclick={() => showShareCodeModal = false}
			>
				Fechar
			</button>
		</div>
	</div>
{/if}

{#if activeEditorFile}
	<div class="fixed inset-0 z-50 bg-bg-overlay/85 backdrop-blur-md flex items-center justify-center p-6" in:fade={{ duration: 150 }}>
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
						class="px-5 py-2 rounded-full text-xs font-black text-brand-foreground flex items-center gap-1.5 transition-all cursor-pointer hover:scale-105 active:scale-[0.98] shadow-md"
						style="background-color: rgb(var(--brand-500));"
						disabled={isSavingEditor}
						onclick={handleSaveEditorFile}
					>
						<Save class="w-3.5 h-3.5 stroke-[2.5]" /> {isSavingEditor ? 'Salvando...' : 'Guardar Alterações'}
					</button>
					<button
						type="button"
						class="p-1.5 rounded-full bg-fg/5 hover:bg-fg/15 text-fg/60 hover:text-fg transition-colors cursor-pointer ml-2"
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
				<span>Tamanho: {Math.round(activeEditorFile.content.length / 1024)} KB</span>
				<span>Editor de Arquivos do Luxmc</span>
			</div>
		</div>
	</div>
{/if}

{#if showHostModal && hostLinkInfo}
	<div class="fixed inset-0 z-50 bg-bg-overlay/80 backdrop-blur-md flex items-center justify-center p-6" in:fade={{ duration: 150 }}>
		<div class="max-w-md w-full bg-bg-elevated border border-brand-500/30 rounded-3xl p-6 shadow-2xl space-y-5 relative overflow-hidden select-none">
			<div class="absolute -top-10 -right-10 w-36 h-36 bg-[radial-gradient(circle_at_center,rgb(var(--brand-500)/0.15),transparent_70%)] rounded-full pointer-events-none"></div>

			<div class="flex items-center justify-between">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-brand-500/15 border border-brand-500/30 flex items-center justify-center">
						<Share2 class="w-5 h-5 text-brand-500" />
					</div>
					<div>
						<h3 class="font-extrabold text-fg text-base">Hostear Mundo com Link Próprio</h3>
						<p class="text-xs text-fg/50">Compartilhe com amigos para entrarem no seu mundo</p>
					</div>
				</div>
				<button
					type="button"
					class="p-1.5 rounded-full bg-fg/5 hover:bg-fg/15 text-fg/60 hover:text-fg transition-colors cursor-pointer"
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
						<span class="text-xs font-bold">Abrindo porta no roteador (UPnP)...</span>
					{:else if upnpResult?.success}
						<Globe2 class="w-4 h-4 text-emerald-400" />
						<span class="text-xs font-extrabold">UPnP Ativo · Aberto para Internet</span>
					{:else}
						<Radio class="w-4 h-4 text-amber-400" />
						<span class="text-xs font-bold">Modo LAN (Rede Local)</span>
					{/if}
				</div>
				<button
					type="button"
					class="text-[10px] font-extrabold px-2.5 py-1 rounded-lg bg-fg/10 hover:bg-fg/15 text-fg transition-all cursor-pointer disabled:opacity-50"
					disabled={isOpeningUpnp}
					onclick={attemptUpnpOpen}
				>
					{isOpeningUpnp ? 'Aguarde...' : 'Reabrir UPnP'}
				</button>
			</div>

			<div class="bg-bg-elevated border border-fg/10 rounded-2xl p-4 space-y-3 shadow-inner">
				<div>
					<span class="text-[10px] font-extrabold text-brand-500 uppercase tracking-wider block mb-1">Link Próprio do Luxmc (Compartilhável)</span>
					<div class="flex items-center gap-2">
						<input
							type="text"
							readonly
							value={hostLinkInfo.shareLink}
							class="flex-1 bg-bg-overlay/50 border border-fg/10 rounded-xl px-3.5 py-2 text-xs font-mono text-fg/90 outline-none select-all"
						/>
						<button
							type="button"
							class="px-4 py-2 rounded-xl bg-brand-500 hover:brightness-110 text-brand-foreground font-black text-xs flex items-center gap-1.5 transition-all cursor-pointer shadow-md active:scale-[0.98]"
							onclick={copyHostLink}
						>
							{#if isCopiedHostLink}
								<Check class="w-3.5 h-3.5 stroke-[3]" /> Copiado!
							{:else}
								<Copy class="w-3.5 h-3.5 stroke-[3]" /> Copiar Link
							{/if}
						</button>
					</div>
				</div>

				{#if upnpResult?.success && upnpResult.externalIp}
					<div class="pt-2 border-t border-fg/5">
						<span class="text-[10px] font-extrabold text-emerald-400 uppercase tracking-wider block mb-1">IP Público (Amigos fora da rede local)</span>
						<div class="flex items-center gap-2">
							<input
								type="text"
								readonly
								value={`${upnpResult.externalIp}:${customHostPort}`}
								class="flex-1 bg-bg-overlay/50 border border-emerald-500/20 rounded-xl px-3.5 py-2 text-xs font-mono text-emerald-300 outline-none select-all"
							/>
							<button
								type="button"
								class="px-3.5 py-2 rounded-xl bg-emerald-500/20 hover:bg-emerald-500/30 text-emerald-300 border border-emerald-500/30 font-extrabold text-xs flex items-center gap-1.5 transition-all cursor-pointer active:scale-[0.98]"
								onclick={copyDirectAddress}
							>
								{#if isCopiedDirectAddress}
									<Check class="w-3.5 h-3.5" /> Copiado!
								{:else}
									<Copy class="w-3.5 h-3.5" /> Copiar IP
								{/if}
							</button>
						</div>
					</div>
				{/if}

				<div class="grid grid-cols-2 gap-2 pt-2 border-t border-fg/5">
					<div>
						<span class="text-[10px] text-fg/40 block font-bold">Endereço IP Local:</span>
						<span class="text-xs font-mono text-emerald-400 font-bold">{hostLinkInfo.directAddress}</span>
					</div>
					<div>
						<span class="text-[10px] text-fg/40 block font-bold">Porta do Servidor:</span>
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
				<div class="font-bold text-amber-300">Como funciona?</div>
				<div>1. Abra seu mundo no Minecraft e clique em <b>"Aberto para LAN"</b> na porta <b>{customHostPort}</b>.</div>
				<div>2. Envie o <b>Link Próprio</b> ou <b>IP Público</b> para seus amigos colarem no Luxmc.</div>
			</div>
		</div>
	</div>
{/if}



{#if showInstanceSettingsModal}
	<div class="fixed inset-0 z-50 bg-bg-overlay/85 backdrop-blur-md flex items-center justify-center p-6" in:fade={{ duration: 150 }}>
		<div class="w-full max-w-3xl bg-bg-elevated border border-fg/10 rounded-3xl p-6 shadow-2xl space-y-6 flex flex-col justify-between select-none h-[560px]">

			<div class="flex gap-6 h-full overflow-hidden">
				<div class="w-56 shrink-0 border-r border-fg/5 pr-4 flex flex-col justify-between">
					<div class="space-y-4">
						<h2 class="text-sm font-extrabold text-fg px-2">Configurações da Instância</h2>
						<nav class="flex flex-col gap-1">
							<button
								type="button"
								class="flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-xs font-bold transition-all w-full text-left cursor-pointer {activeInstanceSection === 'geral' ? 'bg-bg-subtle text-fg border border-fg/10 shadow-sm' : 'text-fg/70 hover:text-fg hover:bg-fg/5'}"
								onclick={() => activeInstanceSection = 'geral'}
							>
								<Box class="w-4 h-4 text-emerald-400" /> Geral
							</button>
							<button
								type="button"
								class="flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-xs font-bold transition-all w-full text-left cursor-pointer {activeInstanceSection === 'instalacao' ? 'bg-bg-subtle text-fg border border-fg/10 shadow-sm' : 'text-fg/70 hover:text-fg hover:bg-fg/5'}"
								onclick={() => activeInstanceSection = 'instalacao'}
							>
								<Download class="w-4 h-4 text-cyan-400" /> Instalação
							</button>
							<button
								type="button"
								class="flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-xs font-bold transition-all w-full text-left cursor-pointer {activeInstanceSection === 'otimizacao' ? 'bg-bg-subtle text-emerald-400 border border-emerald-500/30 shadow-sm' : 'text-fg/70 hover:text-fg hover:bg-fg/5'}"
								onclick={() => activeInstanceSection = 'otimizacao'}
							>
								<Zap class="w-4 h-4 text-emerald-400" /> Otimização Luxmc
							</button>
							<button
								type="button"
								class="flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-xs font-bold transition-all w-full text-left cursor-pointer {activeInstanceSection === 'janela' ? 'bg-bg-subtle text-fg border border-fg/10 shadow-sm' : 'text-fg/70 hover:text-fg hover:bg-fg/5'}"
								onclick={() => activeInstanceSection = 'janela'}
							>
								<Layers class="w-4 h-4 text-purple-400" /> Janela
							</button>
							<button
								type="button"
								class="flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-xs font-bold transition-all w-full text-left cursor-pointer {activeInstanceSection === 'java' ? 'bg-bg-subtle text-fg border border-fg/10 shadow-sm' : 'text-fg/70 hover:text-fg hover:bg-fg/5'}"
								onclick={() => activeInstanceSection = 'java'}
							>
								<Sparkles class="w-4 h-4 text-amber-400" /> Java e Memória
							</button>
							<button
								type="button"
								class="flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-xs font-bold transition-all w-full text-left cursor-pointer {activeInstanceSection === 'hooks' ? 'bg-bg-subtle text-fg border border-fg/10 shadow-sm' : 'text-fg/70 hover:text-fg hover:bg-fg/5'}"
								onclick={() => activeInstanceSection = 'hooks'}
							>
								<Code class="w-4 h-4 text-rose-400" /> Launch Hooks
							</button>
						</nav>
					</div>

					<div class="text-[10px] text-fg/30 font-mono px-2">
						Minecraft {activeProfile?.mcVersion || '1.21.4'}
					</div>
				</div>

				<div class="flex-1 flex flex-col justify-between overflow-y-auto custom-scrollbar pr-1 space-y-6">

					{#if activeInstanceSection === 'geral'}
						<div class="space-y-6">
							<div>
								<div class="flex items-center gap-2">
									<Box class="w-4 h-4 text-emerald-400" />
									<h3 class="text-xs font-bold text-fg uppercase tracking-wider">Geral</h3>
								</div>
								<p class="text-[11px] text-fg/40 mt-0.5">Nome, ícone e ações da instância</p>
							</div>

							<div class="space-y-3">
								<span class="text-xs font-bold text-fg/70 block">Nome da Instância</span>
								<div class="flex items-center gap-4">
									<div class="h-14 w-14 rounded-2xl bg-bg-elevated border border-fg/10 flex items-center justify-center shrink-0 p-1">
										<img src="/grass_block.png" alt="Minecraft" class="w-10 h-10 object-contain [image-rendering:pixelated]" />
									</div>
									<input
										type="text"
										bind:value={instanceNameInput}
										class="flex-1 bg-bg-elevated border border-fg/10 rounded-2xl px-4 py-3 text-xs font-bold text-fg outline-none focus:border-brand-500 transition-colors"
									/>
								</div>
							</div>

							<label class="block space-y-2 text-xs text-fg-muted">
                                <span>Banner da instância</span>
                                <input type="url" bind:value={instanceBanner} placeholder="https://…/banner.webp" class="w-full rounded-xl border border-border bg-bg-elevated px-4 py-3 text-fg focus:border-brand-500" />
                            </label>
                            <div class="space-y-3 pt-2">
								<div>
									<span class="text-xs font-bold text-fg/80 block">Ações da Instância</span>
									<p class="text-[11px] text-fg/40 mt-0.5">Verifique os arquivos do modpack ou remova a instância.</p>
								</div>

								<button
									type="button"
									class="w-full bg-bg-elevated hover:bg-amber-500/10 border border-amber-500/30 rounded-2xl p-4 flex items-center gap-4 transition-all cursor-pointer text-left group"
									onclick={handleRepairModpack}
                                    disabled={isRepairingModpack}
								>
									<div class="h-10 w-10 rounded-xl bg-amber-500/20 border border-amber-500/40 flex items-center justify-center shrink-0 text-amber-400 group-hover:scale-110 transition-transform">
										<Sparkles class="w-5 h-5" />
									</div>
									<div>
										<h4 class="text-xs font-bold text-amber-400">Reparar Instância</h4>
										<p class="text-[11px] text-fg/40 mt-0.5">Verificar e recuperar arquivos do manifesto do modpack</p>
									</div>
								</button>

								<button
									type="button"
									class="w-full bg-bg-elevated hover:bg-rose-500/10 border border-rose-500/30 rounded-2xl p-4 flex items-center gap-4 transition-all cursor-pointer text-left group"
									onclick={() => toast("Dados da instância removidos.", "info")}
								>
									<div class="h-10 w-10 rounded-xl bg-rose-500/20 border border-rose-500/40 flex items-center justify-center shrink-0 text-rose-400 group-hover:scale-110 transition-transform">
										<Trash2 class="w-5 h-5" />
									</div>
									<div>
										<h4 class="text-xs font-bold text-rose-400">Apagar dados da instância</h4>
										<p class="text-[11px] text-fg/40 mt-0.5">Remover permanentemente os ficheiros desta Instância e começar de novo</p>
									</div>
								</button>
							</div>
						</div>

					{:else if activeInstanceSection === 'instalacao'}
						<div class="space-y-6">
							<div>
								<h3 class="text-xs font-bold text-fg uppercase tracking-wider">Instalação & Mod Loaders</h3>
								<p class="text-[11px] text-fg/40 mt-0.5">Gerencie o loader (Fabric, Forge, NeoForge, Quilt) e versão do jogo</p>
							</div>

							<div class="space-y-3">
								<span class="text-xs font-bold text-fg/70 block">Mod Loader Ativo</span>
								<div class="grid grid-cols-2 gap-3">
									{#each ["fabric", "forge", "neoforge", "vanilla"] as loader}
										<button
											type="button"
											class="p-3 rounded-2xl border text-xs font-bold flex items-center justify-between transition-all cursor-pointer {instanceLoaderType === loader ? 'bg-brand-500/20 border-brand-500 text-brand-500' : 'bg-bg-elevated border-fg/10 text-fg/60 hover:text-fg'}"
											onclick={() => instanceLoaderType = loader}
										>
											<span class="capitalize">{loader}</span>
											{#if instanceLoaderType === loader}<Check class="w-4 h-4" />{/if}
										</button>
									{/each}
								</div>
							</div>

							<div class="space-y-2">
								<span class="text-xs font-bold text-fg/70 block">Versão do Mod Loader</span>
								<input type="text" bind:value={instanceLoaderVersion} class="w-full bg-bg-elevated border border-fg/10 rounded-2xl px-4 py-2.5 text-xs text-fg font-mono outline-none focus:border-brand-500" />
							</div>
						</div>

					{:else if activeInstanceSection === 'otimizacao'}
						<div class="space-y-5">
							<div>
								<div class="flex items-center gap-2">
									<Zap class="w-4 h-4 text-emerald-400" />
									<h3 class="text-xs font-bold text-fg uppercase tracking-wider">Sistema de Otimização Luxmc</h3>
								</div>
								<p class="text-[11px] text-fg/40 mt-0.5">Tuning inteligente de JVM, detecção de hardware e aceleração gráfica Linux</p>
							</div>

							<div class="bg-bg-elevated border border-fg/5 rounded-2xl p-4 space-y-3">
								<div class="flex items-center justify-between">
									<span class="text-xs font-bold text-fg/90 flex items-center gap-2">
										<Cpu class="w-3.5 h-3.5 text-brand-500" /> Hardware Detectado no Sistema
									</span>
									<span class="text-[10px] font-mono text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-md border border-emerald-500/20">
										Linux-First
									</span>
								</div>
								<div class="grid grid-cols-2 gap-2 text-xs">
									<div class="bg-bg-overlay/30 p-2.5 rounded-xl border border-fg/5">
										<span class="text-[10px] text-fg/40 block">GPU & Renderizador</span>
										<span class="text-xs font-bold text-fg truncate block mt-0.5" title={gpuInfo?.renderer || 'Buscando...'}>
											{gpuInfo?.renderer || 'AMD Radeon / Mesa RADV'}
										</span>
										<span class="text-[10px] text-brand-500 font-mono block mt-0.5">
											Driver: {gpuInfo?.driver || 'amdgpu'}
										</span>
									</div>
									<div class="bg-bg-overlay/30 p-2.5 rounded-xl border border-fg/5">
										<span class="text-[10px] text-fg/40 block">Memória RAM do Sistema</span>
										<span class="text-xs font-bold text-fg block mt-0.5">
											{Math.round(systemRamMb / 1024)} GB Totais
										</span>
										<span class="text-[10px] text-fg/40 font-mono block mt-0.5">
											Alocado p/ instância: {(instanceRamMb / 1024).toFixed(1)} GB
										</span>
									</div>
								</div>
							</div>

							<div class="bg-bg-elevated border border-fg/5 rounded-2xl p-4 space-y-3">
								<div class="flex items-center justify-between">
									<div>
										<div class="flex items-center gap-2">
											<span class="text-xs font-bold text-fg">Flags de JVM Inteligentes (Aikar G1GC)</span>
											<span class="text-[9px] bg-brand-500/20 text-brand-500 px-1.5 py-0.5 rounded font-bold">Base do Sistema</span>
										</div>
										<span class="text-[10px] text-fg/40 block mt-0.5">
											Ajusta dinamicamente tamanhos de região e new-generation para os {instanceRamMb} MB alocados
										</span>
									</div>
									<button
										type="button"
										aria-label="Alternar Flags Aikar"
										class="w-10 h-5 rounded-full transition-all duration-200 relative flex items-center px-0.5 cursor-pointer {instanceAutoOptimize ? 'bg-emerald-500 shadow-glow' : 'bg-bg-subtle'}"
										onclick={() => instanceAutoOptimize = !instanceAutoOptimize}
									>
										<span class="w-4 h-4 rounded-full bg-fg transition-transform duration-200 shadow-md {instanceAutoOptimize ? 'translate-x-5' : 'translate-x-0'}"></span>
									</button>
								</div>

								<div class="space-y-1.5">
									<div class="flex items-center justify-between text-[10px]">
										<span class="text-fg/50">Flags aplicadas em tempo real:</span>
										<span class="font-mono text-fg/30">{generatedAikarFlags.length} parâmetros</span>
									</div>
									<div class="bg-bg-overlay/50 p-2.5 rounded-xl border border-fg/5 max-h-24 overflow-y-auto custom-scrollbar font-mono text-[10px] text-emerald-400 leading-relaxed break-all">
										{generatedAikarFlags.join(" ")}
									</div>
								</div>
							</div>

							<div class="bg-bg-elevated border border-fg/5 rounded-2xl p-4 space-y-3">
								<div class="flex items-center justify-between">
									<div>
										<div class="flex items-center gap-2">
											<span class="text-xs font-bold text-fg">Pacote de Mods de Performance</span>
											{#if perfPackInfo?.available}
												<span class="text-[9px] bg-emerald-500/20 text-emerald-400 px-1.5 py-0.5 rounded font-bold">Disponível</span>
											{:else}
												<span class="text-[9px] bg-fg/10 text-fg/40 px-1.5 py-0.5 rounded font-bold">Indisponível</span>
											{/if}
										</div>
										<span class="text-[10px] text-fg/40 block mt-0.5">
											{perfPackInfo?.available ? 'Conjunto homologado de mods de taxa de quadros e redução de RAM' : (perfPackInfo?.reason || 'Requer modloader')}
										</span>
									</div>
									{#if perfPackInfo?.available}
										<button
											type="button"
											disabled={installingPerfPack}
											class="px-3 py-1.5 bg-emerald-500 hover:bg-emerald-400 disabled:opacity-50 text-brand-foreground rounded-xl font-bold text-xs transition-all flex items-center gap-1.5 cursor-pointer shadow-md shadow-emerald-500/20"
											onclick={handleInstallPerfPack}
										>
											{#if installingPerfPack}
												<RefreshCw class="w-3.5 h-3.5 animate-spin" /> Instalando...
											{:else}
												<Download class="w-3.5 h-3.5" /> Aplicar Pacote
											{/if}
										</button>
									{/if}
								</div>

								{#if perfPackInfo?.available && perfPackInfo.mods.length > 0}
									<div class="grid grid-cols-3 gap-2 pt-1">
										{#each perfPackInfo.mods as mod}
											<div class="bg-bg-overlay/30 p-2 rounded-xl border border-fg/5">
												<span class="font-bold text-[11px] text-fg block">{mod.title}</span>
												<span class="text-[9px] text-fg/40 block line-clamp-2 mt-0.5">{mod.description}</span>
											</div>
										{/each}
									</div>
								{/if}
							</div>

							<div class="bg-bg-elevated border border-fg/5 rounded-2xl p-4 flex items-center justify-between">
								<div>
									<div class="flex items-center gap-2">
										<span class="text-xs font-bold text-fg">Mesa Zink / Vulkan (Linux)</span>
										<span class="text-[9px] bg-cyan-500/20 text-cyan-400 px-1.5 py-0.5 rounded font-bold">Opt-in</span>
									</div>
									<span class="text-[10px] text-fg/40 block mt-0.5">
										Executa o OpenGL sobre Vulkan via Mesa Zink no Linux (recomendado para AMD RADV / Intel)
									</span>
								</div>
								<button
									type="button"
									aria-label="Alternar Aceleração Vulkan"
									class="w-10 h-5 rounded-full transition-all duration-200 relative flex items-center px-0.5 cursor-pointer {instanceEnableVulkanOpt ? 'bg-emerald-500 shadow-glow' : 'bg-bg-subtle'}"
									onclick={() => instanceEnableVulkanOpt = !instanceEnableVulkanOpt}
								>
									<span class="w-4 h-4 rounded-full bg-fg transition-transform duration-200 shadow-md {instanceEnableVulkanOpt ? 'translate-x-5' : 'translate-x-0'}"></span>
								</button>
							</div>
						</div>

					{:else if activeInstanceSection === 'janela'}
						<div class="space-y-6">
							<div>
								<h3 class="text-xs font-bold text-fg uppercase tracking-wider">Janela & Display</h3>
								<p class="text-[11px] text-fg/40 mt-0.5">Dimensões da janela e modo de exibição do Minecraft</p>
							</div>

							<div class="grid grid-cols-2 gap-4">
								<div class="space-y-1.5">
									<span class="text-xs font-bold text-fg/70 block">Largura (px)</span>
									<input type="number" bind:value={instanceWindowWidth} class="w-full bg-bg-elevated border border-fg/10 rounded-2xl px-4 py-2 text-xs text-fg font-mono outline-none" />
								</div>
								<div class="space-y-1.5">
									<span class="text-xs font-bold text-fg/70 block">Altura (px)</span>
									<input type="number" bind:value={instanceWindowHeight} class="w-full bg-bg-elevated border border-fg/10 rounded-2xl px-4 py-2 text-xs text-fg font-mono outline-none" />
								</div>
							</div>

							<div class="flex items-center justify-between bg-bg-elevated border border-fg/5 rounded-2xl p-4">
								<div>
									<span class="text-xs font-bold text-fg block">Iniciar em Tela Cheia (Fullscreen)</span>
									<span class="text-[10px] text-fg/40 block mt-0.5">Abre o Minecraft ocupando todo o monitor nativamente</span>
								</div>
								<button
									type="button"
									aria-label="Alternar Tela Cheia"
									class="w-10 h-5 rounded-full transition-all duration-200 relative flex items-center px-0.5 cursor-pointer {instanceStartFullscreen ? 'bg-brand-500 shadow-glow' : 'bg-bg-subtle'}"
									onclick={() => instanceStartFullscreen = !instanceStartFullscreen}
								>
									<span class="w-4 h-4 rounded-full bg-fg transition-transform duration-200 shadow-md {instanceStartFullscreen ? 'translate-x-5' : 'translate-x-0'}"></span>
								</button>
							</div>
						</div>

					{:else if activeInstanceSection === 'java'}
						<div class="space-y-6">
							<div>
								<h3 class="text-xs font-bold text-fg uppercase tracking-wider">Java e Memória</h3>
								<div class="mt-4 space-y-3 rounded-2xl border border-border bg-bg-elevated p-4">
									<label for="ram-min" class="block text-xs text-fg-muted">RAM mínima · {instanceMinRamMb} MB</label>
									<input id="ram-min" class="w-full accent-brand-500" type="range" min="512" max={instanceRamMb} step="256" bind:value={instanceMinRamMb} />
									<label for="ram-max" class="block text-xs text-fg-muted">RAM máxima · {instanceRamMb} MB</label>
									<input id="ram-max" class="w-full accent-brand-500" type="range" min="1024" max={Math.max(1024, Math.floor(systemRamMb / 256) * 256)} step="256" bind:value={instanceRamMb} />
									<div class="flex flex-wrap gap-2">{#each ["aikar", "zgc", "shenandoah"] as preset}<button type="button" class={button({ variant: "secondary", size: "sm" })} onclick={() => applyJvmPreset(preset as "aikar" | "zgc" | "shenandoah")}>{preset.toUpperCase()}</button>{/each}</div>
									<p class="text-xs text-fg-subtle">ZGC e Shenandoah exigem um Java compatível. Aikar usa o G1GC.</p>
									<button type="button" class={button({ variant: "secondary", size: "sm" })} onclick={detectJava} disabled={scanningJava}>{scanningJava ? "Detectando..." : "Detectar instalações Java"}</button>
									<label class="block text-xs text-fg-muted" for="java-runtime">Java da instância</label>
									<select id="java-runtime" bind:value={instanceJavaPath} class="w-full rounded-xl border border-border bg-bg-subtle px-3 py-3 text-xs text-fg"><option value="">Automático</option>{#each javaRuntimes as runtime}{#if runtime.path}<option value={runtime.path}>Java {runtime.major} · {runtime.versionString || runtime.path}</option>{/if}{/each}{#if instanceJavaPath && !javaRuntimes.some(runtime => runtime.path === instanceJavaPath)}<option value={instanceJavaPath}>{instanceJavaPath}</option>{/if}</select>
								</div>

								<p class="text-[11px] text-fg/40 mt-0.5">Alocação de RAM, presets rápidos e validação de flags JVM</p>
							</div>

							<div class="flex items-center justify-between bg-bg-elevated border border-emerald-500/30 rounded-2xl p-4">
								<div>
									<span class="text-xs font-bold text-emerald-400 block">Luxmc Vulkan Zero-Lag Optimizer</span>
									<span class="text-[10px] text-fg/40 block mt-0.5">Otimização própria de renderização Mesa Zink e flags G1GC sem bugs visuais</span>
								</div>
								<button
									type="button"
									aria-label="Alternar Otimização Vulkan"
									class="w-10 h-5 rounded-full transition-all duration-200 relative flex items-center px-0.5 cursor-pointer {instanceEnableVulkanOpt ? 'bg-emerald-500 shadow-glow' : 'bg-bg-subtle'}"
									onclick={() => instanceEnableVulkanOpt = !instanceEnableVulkanOpt}
								>
									<span class="w-4 h-4 rounded-full bg-fg transition-transform duration-200 shadow-md {instanceEnableVulkanOpt ? 'translate-x-5' : 'translate-x-0'}"></span>
								</button>
							</div>

							<div class="bg-bg-elevated border border-emerald-500/25 rounded-2xl p-4 space-y-2">
								<div class="flex items-center justify-between text-xs">
									<div class="flex items-center gap-2.5">
										<div class="w-8 h-8 rounded-xl bg-emerald-500/10 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
											<Cpu class="w-4 h-4" />
										</div>
										<div>
											<span class="font-bold text-fg block">Memória e otimização</span>
											<span class="text-[10px] text-fg/50 block">Hardware: {Math.round(systemRamMb / 1024)} GB Totais detectados</span>
										</div>
									</div>
									<span class="text-[10px] font-mono font-bold text-emerald-400 bg-emerald-500/10 px-2.5 py-1 rounded-full border border-emerald-500/20 flex items-center gap-1">
										<Sparkles class="w-3 h-3" /> {instanceAutoOptimize ? "Otimização ativa" : "Configuração manual"}
									</span>
								</div>
								<p class="text-[11px] text-fg/50 leading-relaxed">
									O Luxmc calcula e aloca dinamicamente a quantidade ótima de RAM ao iniciar com base no peso dos mods da instância e na memória livre do Linux, prevenindo travamentos e otimizando o Garbage Collector.
								</p>
							</div>

							<div class="space-y-2 bg-bg-elevated border border-fg/5 rounded-2xl p-4">
								<div class="flex items-center justify-between">
									<span class="text-xs font-bold text-fg/80">Executável Java do Perfil</span>
									<span class="text-[10px] text-fg/40">{instanceJavaPath ? 'Customizado' : 'Auto-detectar (Padrão)'}</span>
								</div>
								<div class="flex gap-2">
									<input
										type="text"
										bind:value={instanceJavaPath}
										placeholder="Deixe vazio para usar a versão recomendada automaticamente"
										class="flex-1 bg-bg-elevated border border-fg/10 rounded-xl px-4 py-2.5 text-xs text-fg font-mono outline-none focus:border-emerald-500"
									/>
									<button
										type="button"
										class="px-3.5 py-2 rounded-xl bg-fg/5 hover:bg-fg/10 text-fg text-xs font-bold transition-all cursor-pointer flex items-center gap-1.5 shrink-0"
										onclick={async () => {
											const selected = await open({
												title: "Selecionar Executável Java",
												multiple: false,
												directory: false
											});
											if (typeof selected === "string") {
												instanceJavaPath = selected;
											}
										}}
									>
										<FolderOpen class="w-3.5 h-3.5" /> Procurar
									</button>
								</div>
							</div>

							<div class="space-y-2 bg-bg-elevated border border-fg/5 rounded-2xl p-4">
								<div class="flex items-center justify-between">
									<span class="text-xs font-bold text-fg/80">Argumentos JVM Customizados</span>
									{#if jvmValidation}
										{#if jvmValidation.valid}
											<span class="text-[10px] font-bold text-emerald-400 flex items-center gap-1">
												<Check class="w-3 h-3 text-emerald-400" /> Compatível com Linux
											</span>
										{:else}
											<span class="text-[10px] font-bold text-amber-400 flex items-center gap-1">
												<span class="w-2 h-2 rounded-full bg-amber-400 animate-pulse"></span> {jvmValidation.rejected.length} flag(s) inseguras
											</span>
										{/if}
									{/if}
								</div>

								<input
									type="text"
									bind:value={instanceJvmArgs}
									placeholder="-XX:+UseG1GC -XX:+AlwaysPreTouch"
									class="w-full bg-bg-elevated border border-fg/10 rounded-xl px-4 py-2.5 text-xs text-fg font-mono outline-none focus:border-emerald-500"
								/>

								{#if jvmValidation && !jvmValidation.valid}
									<div class="p-3 bg-amber-500/10 border border-amber-500/25 rounded-xl space-y-2 mt-2">
										<div class="text-[11px] font-bold text-amber-300">
											Flags rejeitadas para Linux: {jvmValidation.rejected.join(", ")}
										</div>
										<ul class="text-[10px] text-fg/70 space-y-1 list-disc pl-4">
											{#each jvmValidation.suggestions as sug}
												<li>{sug}</li>
											{/each}
										</ul>
										<button
											type="button"
											class="text-[10px] font-bold text-brand-foreground bg-emerald-500 hover:bg-emerald-400 px-3 py-1 rounded-lg transition-all cursor-pointer mt-1"
											onclick={() => {
												if (jvmValidation) {
													instanceJvmArgs = jvmValidation.normalized;
												}
											}}
										>
											Aplicar Sugestão e Limpar Incompatíveis
										</button>
									</div>
								{/if}
							</div>

							<div class="space-y-3 bg-bg-elevated border border-fg/5 rounded-2xl p-4">
								<div>
									<h4 class="text-xs font-bold text-fg/80">Manutenção & Backup da Instância</h4>
									<p class="text-[10px] text-fg/40 mt-0.5">Verifique integridade de arquivos ou exporte backups com segurança</p>
								</div>

								<div class="grid grid-cols-3 gap-2 pt-1">
									<button
										type="button"
										class="p-3 rounded-xl bg-bg-subtle hover:bg-bg-subtle border border-fg/5 hover:border-fg/20 text-left transition-all cursor-pointer flex flex-col justify-between group disabled:opacity-50"
										onclick={handleRepairInstance}
										disabled={isRepairing}
									>
										<div class="flex items-center justify-between w-full">
											<RefreshCw class="w-4 h-4 text-emerald-400 {isRepairing ? 'animate-spin' : 'group-hover:rotate-180 transition-transform duration-500'}" />
											{#if isRepairing}
												<span class="text-[9px] text-emerald-400 font-bold">Reparando...</span>
											{/if}
										</div>
										<div class="mt-2">
											<p class="text-xs font-bold text-fg">Reparar Instância</p>
											<p class="text-[10px] text-fg/40">Checar SHA1 e baixar arquivos faltantes</p>
										</div>
									</button>

									<button
										type="button"
										class="p-3 rounded-xl bg-bg-subtle hover:bg-bg-subtle border border-fg/5 hover:border-fg/20 text-left transition-all cursor-pointer flex flex-col justify-between group disabled:opacity-50"
										onclick={handleBackupSaves}
										disabled={isBackingUp}
									>
										<div class="flex items-center justify-between w-full">
											<Save class="w-4 h-4 text-emerald-400 group-hover:scale-110 transition-transform" />
											{#if isBackingUp}
												<span class="text-[9px] text-emerald-400 font-bold">Gerando...</span>
											{/if}
										</div>
										<div class="mt-2">
											<p class="text-xs font-bold text-fg">Backup dos Mundos</p>
											<p class="text-[10px] text-fg/40">Compactar saves em arquivo .zip</p>
										</div>
									</button>

									<button
										type="button"
										class="p-3 rounded-xl bg-bg-subtle hover:bg-bg-subtle border border-fg/5 hover:border-fg/20 text-left transition-all cursor-pointer flex flex-col justify-between group disabled:opacity-50"
										onclick={handleExportZip}
										disabled={isExporting}
									>
										<div class="flex items-center justify-between w-full">
											<Share2 class="w-4 h-4 text-purple-400 group-hover:scale-110 transition-transform" />
											{#if isExporting}
												<span class="text-[9px] text-purple-400 font-bold">Exportando...</span>
											{/if}
										</div>
										<div class="mt-2">
											<p class="text-xs font-bold text-fg">Exportar Instância</p>
											<p class="text-[10px] text-fg/40">Criar pacote completo .zip</p>
										</div>
									</button>

									<button
										type="button"
										class="p-3 rounded-xl bg-bg-subtle hover:bg-bg-subtle border border-fg/5 hover:border-brand-500/40 text-left transition-all cursor-pointer flex flex-col justify-between group disabled:opacity-50"
										onclick={handleExportShareCode}
										disabled={isGeneratingShareCode}
									>
										<div class="flex items-center justify-between w-full">
											<Sparkles class="w-4 h-4 text-brand-500 group-hover:scale-110 transition-transform" />
											{#if isGeneratingShareCode}
												<span class="text-[9px] text-brand-500 font-bold">Gerando...</span>
											{/if}
										</div>
										<div class="mt-2">
											<p class="text-xs font-bold text-fg">Compartilhar Código</p>
											<p class="text-[10px] text-fg/40">Gerar código LUX-XXXX</p>
										</div>
									</button>
								</div>
							</div>
						</div>

					{:else if activeInstanceSection === 'hooks'}
						<div class="space-y-6">
							<div>
								<h3 class="text-xs font-bold text-fg uppercase tracking-wider">Launch Hooks</h3>
								<p class="text-[11px] text-fg/40 mt-0.5">Executar scripts pré e pós inicialização do Minecraft</p>
							</div>

							<div class="space-y-2">
								<span class="text-xs font-bold text-fg/70 block">Script Pré-Inicialização (Pre-Launch)</span>
								<input type="text" bind:value={instancePreLaunchHook} placeholder="/path/to/script.sh" class="w-full bg-bg-elevated border border-fg/10 rounded-2xl px-4 py-2.5 text-xs text-fg font-mono outline-none" />
							</div>

							<div class="space-y-2">
								<span class="text-xs font-bold text-fg/70 block">Script Pós-Encerramento (Post-Exit)</span>
								<input type="text" bind:value={instancePostExitHook} placeholder="/path/to/script.sh" class="w-full bg-bg-elevated border border-fg/10 rounded-2xl px-4 py-2.5 text-xs text-fg font-mono outline-none" />
							</div>
						</div>
					{/if}

				</div>
			</div>

			<div class="flex items-center justify-end gap-3 pt-4 border-t border-fg/5">
				<button
					type="button"
					class="px-6 py-2.5 rounded-full text-xs font-bold text-fg/60 hover:text-fg hover:bg-fg/5 transition-all cursor-pointer"
					onclick={() => showInstanceSettingsModal = false}
				>
					Cancelar
				</button>
				<button
					type="button"
					class="px-7 py-2.5 rounded-full text-xs font-black text-brand-foreground transition-all cursor-pointer shadow-lg hover:scale-105 active:scale-[0.98] flex items-center gap-2"
					style="background-color: rgb(var(--brand-500));"
					onclick={saveInstanceSettings}
				>
					<Check class="w-4 h-4 stroke-[3]" /> Guardar alterações
				</button>
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

<InstanceConfigEditorModal
	open={showConfigEditor}
	profileId={instanceId}
	onClose={() => showConfigEditor = false}
/>

<P2PHostModal
	open={showP2PHost}
	onClose={() => showP2PHost = false}
/>

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

<ModpackExportModal
	isOpen={showModpackExportModal}
	profileId={instanceId}
	instanceName={activeProfile?.name || "Modpack"}
	onClose={() => showModpackExportModal = false}
/>

<WorldBackupModal
	isOpen={showWorldBackupModal}
	profileId={instanceId}
	worldsList={worldsList}
	onClose={() => showWorldBackupModal = false}
/>

<KeybindEditorModal
	isOpen={showKeybindEditorModal}
	profileId={instanceId}
	onClose={() => showKeybindEditorModal = false}
/>

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

<JukeboxModal
	open={showJukeboxModal}
	onClose={() => showJukeboxModal = false}
/>
