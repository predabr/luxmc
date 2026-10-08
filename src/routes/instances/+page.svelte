<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { quintOut } from "svelte/easing";
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";
	import { fade } from "svelte/transition";
	import Heading from "$lib/components/ui/Heading.svelte";
	import Card from "$lib/components/ui/Card.svelte";
	import Button from "$lib/components/ui/Button.svelte";
	import Input from "$lib/components/ui/Input.svelte";
	import Skeleton from "$lib/components/ui/Skeleton.svelte";
	import Modal from "$lib/components/ui/Modal.svelte";
	import InstanceGrid from "$lib/components/instances/InstanceGrid.svelte";
	import InstanceFilters from "$lib/components/instances/InstanceFilters.svelte";
	import CreateInstanceModal from "$lib/components/instances/CreateInstanceModal.svelte";
	import UniversalImporterModal from "$lib/components/instances/UniversalImporterModal.svelte";
	import FilterableVersionSelect from "$lib/components/ui/FilterableVersionSelect.svelte";
	import {
		Plus,
		Trash2,
		Check,
		Boxes,
		Copy,
		FolderOpen,
		Image,
		Import,
		HeartPulse,
		Sparkles,
		FolderTree,
		StickyNote,
		X,
		SquareCheck,
		Cpu
	} from "lucide-svelte";
	import { open } from "@tauri-apps/plugin-dialog";
	import { convertFileSrc } from "@tauri-apps/api/core";
	import { getIconSrc } from "$lib/utils/icons";
	import { profiles, type Profile } from "$lib/stores/profiles.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { getFullCapeDataUrl } from "$lib/utils/capeTextures";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { fireModpackSuccessConfetti } from "$lib/utils/confetti";
	import {
		api,
		versionsList,
		instancesDuplicate,
		instancesOpenFolder,
		instancesScreenshots,
		instanceImportModpack,
		instanceImportMrpack,
		instanceCancelImport,
		instanceHealthCheck,
		instanceFileTree,
		instanceSetNotes,
		getSystemSpecs,
		optimizerInstallPerfPack,
		launchGame,
		versionsCheckInstalled,
		versionsDownload,
		authDevLogin,
		discordSetActivity,
		type HealthCheckResult,
		type FileTreeEntry,
		instanceImportShareCode
	} from "$lib/api";
	import { handlePostLaunchActions } from "$lib/utils/launcherLifecycle";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { achievements } from "$lib/stores/achievements.svelte";
	import { playSound } from "$lib/utils/sound";

	const { t } = useTranslation();

	let showCreate = $state(false);
	let showUniversalImport = $state(false);
	let showImport = $state(false);
	let showImportMrpack = $state(false);
	let showImportCode = $state(false);
	let shareCodeInput = $state("");
	let isImportingCode = $state(false);

	async function handleImportShareCode() {
		const code = shareCodeInput.trim();
		if (!code) {
			toast(uiText("ui.199a854cf6c43a1f"), "warning");
			return;
		}
		isImportingCode = true;
		try {
			const profile = await instanceImportShareCode(code);
			toast(uiText("ui.fd91cd11f65f22fb", {arg0: (profile.name)}), "success");
			showImportCode = false;
			shareCodeInput = "";
			achievements.unlock("share_code");
			playSound("chime");
			await profiles.refresh();
		} catch (e) {
			toast(uiText("ui.bd7b4afa8b3f0462") + String(e), "error");
		} finally {
			isImportingCode = false;
		}
	}
	let importFile = $state<string | null>(null);
	let importMrpackFile = $state<string | null>(null);
	let importName = $state("");
	let importMrpackName = $state("");
	let importVersion = $state("1.21.4");
	let importLoader = $state("fabric");
	let importing = $state(false);
	let importingMrpack = $state(false);
	let importProgress = $state<{ phase: string; current: number; total: number; percent: number; status: string } | null>(null);
	let lastError = $state<string | null>(null);

	let systemRamMb = $state(8192);
	let availableVersions = $state<Array<{ id: string; versionType: string; releaseTime: string }>>([]);
	let versionsLoading = $state(false);

	let viewMode = $state<"grid" | "list">("grid");
	let searchQuery = $state("");
	let sortBy = $state<"name" | "version" | "date" | "lastPlayed">("name");
	let groupFilter = $state<string>("all");

	let screenshotsId = $state<string | null>(null);
	let screenshots = $state<Array<{ name: string; path: string; modified: string; dataUrl?: string | null; thumbPath?: string | null }>>([]);
	let screenshotsLoading = $state(false);

	let healthCheckId = $state<string | null>(null);
	let healthResult = $state<HealthCheckResult | null>(null);
	let healthChecking = $state(false);

	let fileBrowserId = $state<string | null>(null);
	let fileTree = $state<FileTreeEntry[]>([]);
	let fileTreeLoading = $state(false);
	let fileTreePath = $state<string | null>(null);

	let notesId = $state<string | null>(null);
	let notesText = $state("");
	let savingNotes = $state(false);

	let instanceColors = $state<Record<string, string>>({});
	let selectedIds = $state<Set<string>>(new Set());
	let selectionMode = $state(false);
	let searchInput = $state<HTMLInputElement | null>(null);
	let colorPickerId = $state<string | null>(null);
	let navigatingId = $state<string | null>(null);

	const colorOptions = [
		{ value: "red", color: "rgb(var(--danger))" },
		{ value: "blue", color: "rgb(59, 130, 246)" },
		{ value: "green", color: "rgb(var(--success))" },
		{ value: "yellow", color: "rgb(234, 179, 8)" },
		{ value: "purple", color: "rgb(168, 85, 247)" },
		{ value: "orange", color: "rgb(249, 115, 22)" },
	];

	const groups = ["all", "Modded", "Vanilla", "Servers", "Favorites"];

	let activeSearch = $state("");
	let searchDebounce: ReturnType<typeof setTimeout> | null = null;

	$effect(() => {
		const query = searchQuery;
		if (searchDebounce) clearTimeout(searchDebounce);
		searchDebounce = setTimeout(() => {
			activeSearch = query;
		}, 150);
		return () => {
			if (searchDebounce) clearTimeout(searchDebounce);
		};
	});

	const normalizedSearch = $derived(activeSearch.trim().toLowerCase());

	const filteredInstances = $derived(
		profiles.list
			.filter((p) => !normalizedSearch || p.name.toLowerCase().includes(normalizedSearch))
			.filter((p) => {
				if (groupFilter === "all") return true;
				if (groupFilter === "Favorites") return p.favorite;
				if (groupFilter === "Modded") return p.loader !== "vanilla";
				if (groupFilter === "Vanilla") return p.loader === "vanilla";
				if (groupFilter === "Servers") return p.group === "Servers";
				return true;
			})
			.toSorted((a, b) => {
				if (sortBy === "name") return (a.name || "").localeCompare(b.name || "");
				if (sortBy === "version") return (a.mcVersion || "").localeCompare(b.mcVersion || "");
				if (sortBy === "lastPlayed") return (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0);
				return (b.createdAt ?? 0) - (a.createdAt ?? 0);
			}),
	);

	function formatBytes(bytes: number): string {
		if (bytes === 0) return "0 B";
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
		return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
	}

	function formatTimeAgo(timestamp: number): string {
		if (!timestamp) return uiText("ui.307a3b6fb9276530");
		const diff = Date.now() - timestamp;
		const minutes = Math.floor(diff / 60000);
		if (minutes < 1) return "Agora";
		if (minutes < 60) return uiText("ui.203ff49e070cca53", {arg0: (minutes)});
		const hours = Math.floor(minutes / 60);
		if (hours < 24) return uiText("ui.aea1c7314c6793d9", {arg0: (hours)});
		const days = Math.floor(hours / 24);
		return uiText("ui.48a43c7197b246ae", {arg0: (days)});
	}

	function loadColors() {
		try {
			const stored = localStorage.getItem("luxmc.instanceColors");
			if (stored) instanceColors = JSON.parse(stored);
		} catch {}
	}

	function toggleSelect(id: string) {
		const next = new Set(selectedIds);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		selectedIds = next;
		if (next.size === 0) selectionMode = false;
	}

	function toggleSelectionMode() {
		if (selectionMode) {
			selectedIds = new Set();
			selectionMode = false;
		} else {
			selectionMode = true;
		}
	}

	function selectInstance(id: string) {
		if (selectionMode) {
			toggleSelect(id);
			return;
		}
		navigatingId = id;
		profiles.activeId = id;
		void goto("/instances/" + id);
	}

	function handleInstanceKeydown(e: KeyboardEvent) {
		if (e.ctrlKey && e.key === "f") {
			e.preventDefault();
			searchInput?.focus();
		}
		if (e.key === "Escape") {
			searchQuery = "";
			searchInput?.blur();
			if (selectionMode) toggleSelectionMode();
		}
	}

	let versionsRequested = false;
    let versionsUpdatedAt = 0;
	let specsRequested = false;

	onMount(() => {
		loadColors();
		window.addEventListener("keydown", handleInstanceKeydown);
        const refreshVersions = () => { if (!document.hidden) ensureVersions(); };
        window.addEventListener("focus", refreshVersions);
        const timer = setInterval(refreshVersions, 60000);
        return () => { window.removeEventListener("keydown", handleInstanceKeydown); window.removeEventListener("focus", refreshVersions); clearInterval(timer); };
	});

	function ensureSystemSpecs() {
		if (specsRequested) return;
		specsRequested = true;
		void loadSystemSpecs();
	}

	async function loadSystemSpecs() {
		try {
			const specs = await getSystemSpecs();
			if (specs?.totalRamMb) systemRamMb = specs.totalRamMb;
		} catch {}
	}

	function ensureModalData() {
		ensureVersions();
		ensureSystemSpecs();
	}

	function ensureVersions() {
		if (versionsRequested && Date.now() - versionsUpdatedAt < 60000) return;
		versionsRequested = true;
        versionsUpdatedAt = Date.now();
		void loadVersions();
	}

	async function loadVersions() {
		try {
			const cached = localStorage.getItem("luxmc_cached_versions");
			if (cached) {
				const parsed = JSON.parse(cached);
				if (parsed?.versions?.length > 0) availableVersions = parsed.versions;
			}
		} catch {}
		versionsLoading = availableVersions.length === 0;
		try {
			const response = await versionsList();
			availableVersions = response.versions;
			const payload = JSON.stringify(response);
			setTimeout(() => {
				try { localStorage.setItem("luxmc_cached_versions", payload); } catch {}
			}, 0);
		} catch (error) {
			if (availableVersions.length === 0) {
				lastError = `Could not load Minecraft versions: ${String(error)}`;
				toast(lastError, "error");
			}
		} finally {
			versionsLoading = false;
		}
	}

	async function handleCreateInstance(input: {
		name: string; version: string; loader: string; loaderVersion?: string; icon: string;
		ramGb: number; autoOptimize: boolean; useVulkan: boolean; installPerfPack: boolean;
		resolutionW?: number; resolutionH?: number; fullscreen?: boolean;
		javaPath?: string; jvmArgs?: string; gameDir?: string;
	}) {
		const p = await api.invoke<{
			id: string; name: string; icon: string; mcVersion: string;
			loader: string; loaderVersion: string | null; gameDir: string;
			resolutionW: number | null; resolutionH: number | null; fullscreen: boolean;
			javaPath: string | null; jvmArgs: string | null;
			createdAt: string; updatedAt: string;
		}>("profiles_create", {
			input: {
				name: input.name, mcVersion: input.version, loader: input.loader,
				loaderVersion: input.loaderVersion ?? null,
				icon: input.icon, ramMb: input.ramGb * 1024,
				autoOptimize: input.autoOptimize, useVulkan: input.useVulkan,
				resolutionW: input.resolutionW ?? null,
				resolutionH: input.resolutionH ?? null,
				fullscreen: input.fullscreen ?? false,
				javaPath: input.javaPath ?? null,
				jvmArgs: input.jvmArgs ?? null,
				gameDir: input.gameDir ?? null,
			},
		});

		if (input.installPerfPack && input.loader !== "vanilla") {
			try {
				await optimizerInstallPerfPack(p.id);
			} catch (err) {
				const message = String(err);
				const withoutMods =
					message.includes("Nenhum mod de desempenho compatível") ||
					message.includes("O pacote de otimização requer");
				toast(
					withoutMods
						? uiText("ui.40614d4fd6779ff5", {arg0: (input.version), arg1: (input.loader)})
						: uiText("ui.c523fdbb3da2b7ee", {arg0: (message)}),
					withoutMods ? "info" : "error"
				);
			}
		}

		profiles.add({
			id: p.id, name: p.name, icon: p.icon || input.icon,
			mcVersion: p.mcVersion,
			loader: p.loader as "vanilla" | "fabric" | "forge" | "neoforge" | "quilt",
			loaderVersion: p.loaderVersion ?? undefined,
			gameDir: p.gameDir,
			createdAt: new Date(p.createdAt).getTime(),
			updatedAt: new Date(p.updatedAt).getTime(),
			group: undefined,
			ramMb: input.ramGb * 1024,
			autoOptimize: input.autoOptimize,
			useVulkan: input.useVulkan,
			resolution: p.resolutionW && p.resolutionH ? { width: p.resolutionW, height: p.resolutionH, fullscreen: p.fullscreen } : undefined,
			resolutionW: p.resolutionW ?? undefined,
			resolutionH: p.resolutionH ?? undefined,
			fullscreen: p.fullscreen,
			javaPath: p.javaPath ?? undefined,
			jvmArgs: p.jvmArgs ?? undefined,
		});
		profiles.activeId = p.id;
	}

	let launchingInstanceId = $state<string | null>(null);

	async function quickPlay(p: Profile) {
		if (launchingInstanceId || appState.isLaunching) return;
		launchingInstanceId = p.id;
		appState.isLaunching = true;
		appState.launchingProfileId = p.id;
		appState.launchStatusText = uiText("ui.897c3c2e65f8c77d");
		profiles.activeId = p.id;
		try {
			let accountId = account.value?.uuid;
			if (!accountId) {
				const dev = await authDevLogin();
				accountId = dev.uuid;
			}
			const verId = p.mcVersion || "1.21.4";
			appState.launchStatusText = uiText("ui.a4446717f1d89dbb", {arg0: (verId)});
			const installed = await versionsCheckInstalled(verId).catch(() => false);
			if (!installed) {
				appState.launchStatusText = uiText("ui.1d9893e36eabb20a", {arg0: (verId)});
				toast(uiText("ui.1d9893e36eabb20a", {arg0: (verId)}), "info");
				await versionsDownload(verId);
			}
			const globalVulkan = typeof window !== "undefined" && localStorage.getItem("luxmc_enable_vulkan") === "true";
			const isVulkan = globalVulkan || p.useVulkan === true;
			const skinToPass = activeSkinStore.current.skinUrl || account.value?.skinUrl || null;
			const effectiveCape = activeSkinStore.current.hasCape
				? (activeSkinStore.current.customCapeUrl || (activeSkinStore.current.capeType && activeSkinStore.current.capeType !== "none" ? getFullCapeDataUrl(activeSkinStore.current.capeType) : null))
				: (account.value?.capeUrl || null);
			appState.launchStatusText = uiText("ui.d3144d88da48de3b");
			const res = await launchGame({
				versionId: verId,
				accountId: accountId || "",
				profileId: p.id,
				enableVulkan: isVulkan,
				skinUrl: skinToPass,
				skinVariant: activeSkinStore.current.type === "alex" ? "slim" : "classic",
				capeUrl: effectiveCape
			});
			gamingStats.onGameStart(p.id);
            appState.activeGameDetails = { profileId: p.id, name: p.name, version: verId, loader: p.loader };
			appState.isGameRunning = true;
			const pNameLower = p.name.toLowerCase();
			const modpackCover = (p.icon && (p.icon.startsWith("http://") || p.icon.startsWith("https://")))
				? p.icon
				: pNameLower.includes("better mc") || pNameLower.includes("bmc")
				? "https://raw.githubusercontent.com/predabr/luxmc/main/build/modpack_better_mc.webp"
				: pNameLower.includes("pixelmon") || pNameLower.includes("cobblemon")
				? "https://raw.githubusercontent.com/predabr/luxmc/main/build/modpack_cobblemon.webp"
				: pNameLower.includes("fabulously optimized") || pNameLower.includes("fo")
				? "https://raw.githubusercontent.com/predabr/luxmc/main/build/modpack_fo.webp"
				: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png";

			discordSetActivity({
				inGame: true,
				details: p.name,
				state: `Minecraft ${verId} · ${p.loader ? p.loader.toUpperCase() : "Vanilla"}`,
				largeText: p.name,
				largeImage: modpackCover,
				smallImage: "grass",
				smallText: `Luxmc · ${p.loader || "Vanilla"}`,
				startTime: Math.floor(Date.now() / 1000)
			}).catch(() => {});
			profiles.setLastPlayed(p.id);
			toast(`🎮 Minecraft ${verId} (${p.name}) iniciado! (PID: ${res.pid})`, "success");
			void handlePostLaunchActions();
		} catch (e) {
			toast(uiText("ui.ca8c81cc4b1149af") + String(e), "error");
		} finally {
			appState.isLaunching = false;
			appState.launchingProfileId = null;
			appState.launchStatusText = "";
			launchingInstanceId = null;
		}
	}

	let editingInstance = $state<{
		id: string; name: string; mcVersion: string; loader: string;
		icon: string; ramMb: number; jvmArgs: string;
	} | null>(null);
	let editName = $state("");
	let editVersion = $state("");
	let editIcon = $state("grass_block");
	let editRamGb = $state(4);
	let editJvmArgs = $state("");
	let editSaving = $state(false);

	function openEditInstance(p: Profile) {
		ensureModalData();
		editingInstance = {
			id: p.id, name: p.name, mcVersion: p.mcVersion, loader: p.loader,
			icon: p.icon || "grass_block", ramMb: p.ramMb || 4096, jvmArgs: p.jvmArgs || ""
		};
		editName = p.name;
		editVersion = p.mcVersion;
		editIcon = p.icon || "grass_block";
		editRamGb = Math.max(1, Math.round((p.ramMb || 4096) / 1024));
		editJvmArgs = p.jvmArgs || "";
	}

	async function saveEditInstance() {
		if (!editingInstance || !editName.trim()) return;
		editSaving = true;
		try {
			await api.invoke("profiles_update", {
				input: {
					id: editingInstance.id, name: editName.trim(), mcVersion: editVersion,
					icon: editIcon, ramMb: editRamGb * 1024, jvmArgs: editJvmArgs
				}
			});
			profiles.update(editingInstance.id, {
				name: editName.trim(), mcVersion: editVersion,
				icon: editIcon, ramMb: editRamGb * 1024, jvmArgs: editJvmArgs
			});
			toast(uiText("ui.97db9771d03340dc"), "success");
			editingInstance = null;
		} catch (e) {
			lastError = uiText("ui.5187b2d53dfa23cd") + String(e);
			toast(t("instances.failedUpdate", { error: String(e) }), "error");
		} finally {
			editSaving = false;
		}
	}

	let confirmDeleteInstance = $state<{ id: string; name: string; gameDir: string } | null>(null);
	let deleting = $state(false);

	async function deleteInstance(id: string) {
		try {
			await api.invoke("profiles_delete", { id });
			profiles.remove(id);
			playSound("delete");
		} catch (e) {
			toast(t("instances.failedDelete", { error: String(e) }), "error");
		}
	}

	async function bulkDelete() {
		let count = 0;
		for (const id of selectedIds) {
			try {
				await api.invoke("profiles_delete", { id });
				profiles.remove(id);
				count++;
			} catch (e) {
				toast(t("instances.failedDelete", { error: String(e) }), "error");
			}
		}
		selectedIds = new Set();
		selectionMode = false;
		if (count > 0) {
			playSound("delete");
			toast(uiText("ui.0a0f81bf7b3cf421", {arg0: (count)}), "success");
		}
	}

	async function bulkDuplicate() {
		for (const id of selectedIds) {
			try {
				const p = await instancesDuplicate(id);
				profiles.add({
					id: p.id, name: p.name, icon: p.icon, mcVersion: p.mcVersion,
					loader: p.loader as "vanilla" | "fabric" | "forge" | "neoforge" | "quilt",
					loaderVersion: p.loaderVersion ?? undefined,
					gameDir: p.gameDir,
					createdAt: new Date(p.createdAt).getTime(),
					updatedAt: new Date(p.updatedAt).getTime(),
				});
			} catch (e) {
				toast(t("instances.failedDuplicate", { error: String(e) }), "error");
			}
		}
		selectedIds = new Set();
		selectionMode = false;
	}

	async function duplicateInstance(id: string) {
		try {
			const p = await instancesDuplicate(id);
			profiles.add({
				id: p.id, name: p.name, icon: p.icon, mcVersion: p.mcVersion,
				loader: p.loader as "vanilla" | "fabric" | "forge" | "neoforge" | "quilt",
				loaderVersion: p.loaderVersion ?? undefined,
				gameDir: p.gameDir,
				createdAt: new Date(p.createdAt).getTime(),
				updatedAt: new Date(p.updatedAt).getTime(),
			});
			toast(t("instances.duplicate"), "success");
		} catch (e) {
			toast(t("instances.failedDuplicate", { error: String(e) }), "error");
		}
	}

	async function openFolder(id: string) {
		try { await instancesOpenFolder(id); } catch (e) {
			toast(t("instances.failedOpenFolder", { error: String(e) }), "error");
		}
	}

	async function openScreenshots(id: string) {
		screenshotsId = id;
		screenshotsLoading = true;
		try { screenshots = await instancesScreenshots(id); } catch (e) {
			toast(t("screenshots.loadFailed", { error: String(e) }), "error");
			screenshots = [];
		} finally { screenshotsLoading = false; }
	}

	function closeScreenshots() { screenshotsId = null; screenshots = []; }

	async function checkHealth(id: string) {
		healthCheckId = id;
		healthChecking = true;
		healthResult = null;
		try { healthResult = await instanceHealthCheck(id); } catch (e) {
			toast(t("instances.failedHealthCheck", { error: String(e) }), "error");
			healthResult = { clientJar: false, natives: false, modsOk: false, issues: ["Health check failed: " + String(e)] };
		} finally { healthChecking = false; }
	}

	function closeHealthCheck() { healthCheckId = null; healthResult = null; }

	async function browseFiles(id: string) {
		fileBrowserId = id;
		fileTreeLoading = true;
		fileTreePath = null;
		try { fileTree = await instanceFileTree(id); } catch (e) {
			toast(t("instances.failedFileTree", { error: String(e) }), "error");
			fileTree = [];
		} finally { fileTreeLoading = false; }
	}

	async function navigateFileTree(subPath: string | null) {
		if (!fileBrowserId) return;
		fileTreeLoading = true;
		fileTreePath = subPath;
		try { fileTree = await instanceFileTree(fileBrowserId, subPath ?? undefined); } catch (e) {
			toast(t("instances.failedFileTree", { error: String(e) }), "error");
			fileTree = [];
		} finally { fileTreeLoading = false; }
	}

	function closeFileBrowser() { fileBrowserId = null; fileTree = []; fileTreePath = null; }

	function openNotes(id: string) {
		const p = profiles.list.find((x) => x.id === id);
		if (p) { notesId = id; notesText = p.notes ?? ""; }
	}

	async function saveNotes() {
		if (!notesId) return;
		savingNotes = true;
		profiles.update(notesId, { notes: notesText });
		const id = notesId;
		const notes = notesText || null;
		try { await instanceSetNotes(id, notes); toast(t("instances.notesSaved"), "success"); }
		catch (e) { toast(t("instances.failedSaveNotes", { error: String(e) }), "error"); }
		finally { savingNotes = false; notesId = null; notesText = ""; }
	}

	function closeNotes() { notesId = null; notesText = ""; }

	async function pickModpackFile() {
		const file = await open({ filters: [{ name: "Modpack", extensions: ["zip"] }] });
		if (typeof file === "string") {
			importFile = file;
			if (!importName) { importName = (file.split(/[/\\]/).pop() ?? "Imported Modpack").replace(/\.zip$/i, ""); }
			showImport = true;
		}
	}

	async function importModpack() {
		if (!importFile || !importName.trim()) return;
		importing = true;
		importProgress = null;
		let unlisten: (() => void) | null = null;
		let wasCancelled = false;
		try {
			const { listen } = await import("@tauri-apps/api/event");
			unlisten = await listen<{ phase: string; current: number; total: number; percent: number; status: string }>(
				"modpack-progress",
				(event) => {
					importProgress = event.payload;
					if (event.payload.phase === "cancelled") {
						wasCancelled = true;
					}
				}
			);
			const p = await instanceImportModpack(importFile, importName.trim(), importVersion, importLoader);
			if (wasCancelled) {
				toast(uiText("ui.e6c50b7bfe042bcf"), "info");
			} else {
				profiles.add({
					id: p.id, name: p.name, icon: p.icon || "",
					mcVersion: p.mcVersion,
					loader: (p.loader.toLowerCase() as "vanilla" | "fabric" | "forge" | "neoforge" | "quilt") || "fabric",
					loaderVersion: p.loaderVersion ?? undefined,
					gameDir: p.gameDir, createdAt: Date.now(), updatedAt: Date.now(),
				});
				profiles.activeId = p.id;
				showImport = false; importFile = null; importName = "";
				fireModpackSuccessConfetti();
				toast(t("instances.createdSuccess"), "success");
			}
		} catch (e) {
			if (wasCancelled || String(e).toLowerCase().includes("cancelad")) toast(uiText("ui.e6c50b7bfe042bcf"), "info");
			else toast(t("instances.failedImportModpack", { error: String(e) }), "error");
		}
		finally { importing = false; importProgress = null; unlisten?.(); }
	}

	async function cancelImport() {
		try { await instanceCancelImport(); } catch {}
	}

	async function pickMrpackFile() {
		const file = await open({ filters: [{ name: "Modrinth Modpack", extensions: ["mrpack"] }] });
		if (typeof file === "string") {
			importMrpackFile = file;
			if (!importMrpackName) { importMrpackName = (file.split(/[/\\]/).pop() ?? "Imported Modpack").replace(/\.mrpack$/i, ""); }
			showImportMrpack = true;
		}
	}

	async function importMrpack() {
		if (!importMrpackFile || !importMrpackName.trim()) return;
		importingMrpack = true;
		importProgress = null;
		let unlisten: (() => void) | null = null;
		let wasCancelled = false;
		try {
			const { listen } = await import("@tauri-apps/api/event");
			unlisten = await listen<{ phase: string; current: number; total: number; percent: number; status: string }>(
				"modpack-progress",
				(event) => {
					importProgress = event.payload;
					if (event.payload.phase === "cancelled") {
						wasCancelled = true;
					}
				}
			);
			const p = await instanceImportMrpack(importMrpackFile, importMrpackName.trim());
			if (wasCancelled) {
				toast(uiText("ui.e6c50b7bfe042bcf"), "info");
			} else {
				profiles.add({
					id: p.id,
					name: p.name,
					icon: p.icon || "",
					mcVersion: p.mcVersion,
					loader: (p.loader.toLowerCase() as "vanilla" | "fabric" | "forge" | "neoforge" | "quilt") || "fabric",
					loaderVersion: p.loaderVersion ?? undefined,
					gameDir: p.gameDir,
					createdAt: Date.now(),
					updatedAt: Date.now(),
				});
				profiles.activeId = p.id;
				showImportMrpack = false; importMrpackFile = null; importMrpackName = "";
				fireModpackSuccessConfetti();
				toast(t("instances.createdSuccess"), "success");
			}
		} catch (e) {
			if (wasCancelled || String(e).toLowerCase().includes("cancelad")) toast(uiText("ui.e6c50b7bfe042bcf"), "info");
			else toast(t("instances.failedImportMrpack", { error: String(e) }), "error");
		}
		finally { importingMrpack = false; importProgress = null; unlisten?.(); }
	}
</script>

<div class="mx-auto flex min-h-full w-full max-w-7xl flex-col gap-6">
	<div class="flex flex-wrap items-end justify-between gap-5 pb-2">
		<div class="shrink-0"><p class="page-eyebrow mb-2">{uiText("ui.d6e609c3c2349aac")}</p><h1 class="page-title">{uiText("ui.dd0b0bf4b8398672")}<span class="ml-3 text-lg font-medium text-fg-subtle">{profiles.list.length}</span></h1><p class="page-description">{uiText("ui.dd78c3a38366515f")}</p></div>
		<div class="flex flex-wrap items-center gap-2">
			{#if selectionMode && selectedIds.size > 0}
				<Button variant="danger" size="sm" onclick={bulkDelete}>
					<Trash2 class="h-4 w-4" />
					{t("instances.deleteSelected", { count: selectedIds.size })}
				</Button>
				<Button variant="secondary" size="sm" onclick={bulkDuplicate}>
					<Copy class="h-4 w-4" />
					{t("instances.duplicateSelected", { count: selectedIds.size })}
				</Button>
				<Button variant="ghost" size="sm" onclick={toggleSelectionMode}>
					<X class="h-4 w-4" />
					{t("common.cancel")}
				</Button>
			{:else}
				<Button variant="secondary" size="sm" onclick={toggleSelectionMode}>
					<SquareCheck class="h-4 w-4" />
					{t("instances.select")}
				</Button>
			{/if}

			<div class="flex flex-wrap items-center gap-2">
				<Button
					variant="secondary"
					size="sm"
					onclick={() => showUniversalImport = true}
				>
					<FolderTree class="h-3.5 w-3.5 text-brand-400" />
					{uiText("ui.4bd2faff3cc4d942")}
				</Button>

				<Button
					variant="secondary"
					size="sm"
					onclick={() => showImportCode = true}
				>
					<Sparkles class="h-3.5 w-3.5 text-brand-400" />
					{uiText("ui.f58b85570398c0cc")}
				</Button>

				<Button
					variant="secondary"
					size="sm"
					onclick={pickModpackFile}
				>
					<Import class="h-3.5 w-3.5 text-brand-400" />
					{uiText("ui.679e0f3cda397a79")}
				</Button>

				<Button
					variant="secondary"
					size="sm"
					onclick={pickMrpackFile}
				>
					<div class="w-3 h-3 rounded-full bg-emerald-500/20 border border-emerald-500/40 flex items-center justify-center">
						<div class="w-1 h-1 rounded-full bg-emerald-400"></div>
					</div>
					{uiText("ui.ed8aff80815983e4")}
				</Button>

				<Button
					variant="solid"
					size="sm"
					onclick={() => { showCreate = !showCreate; lastError = null; if (showCreate) ensureModalData(); }}
				>
					<Plus class="h-4 w-4 stroke-[2.5]" />
					{uiText("instances.newInstance")}
				</Button>
			</div>
		</div>
	</div>

	<CreateInstanceModal
		bind:isOpen={showCreate}
		onClose={() => showCreate = false}
		versions={availableVersions}
		{versionsLoading}
		{systemRamMb}
		onCreate={handleCreateInstance}
	/>

	<UniversalImporterModal
		open={showUniversalImport}
		onClose={() => showUniversalImport = false}
	/>

	{#if showImport}
		<Card>
			<div class="flex flex-col gap-3">
				<p class="text-sm" style="color: rgb(var(--fg-muted));">{t("instances.importModpackDesc")}</p>
				<p class="truncate text-xs" style="color: rgb(var(--fg-subtle));">{importFile}</p>
				{#if importing && importProgress}
					<div class="flex flex-col gap-2">
						<div class="flex items-center justify-between text-xs" style="color: rgb(var(--fg-muted));">
							<span>{importProgress.status}</span>
							{#if importProgress.total}<span>{importProgress.current ?? 0}/{importProgress.total}</span>{/if}
						</div>
						<div class="h-2 w-full overflow-hidden rounded-full" style="background: rgb(var(--bg-elevated));">
							<div
								class="h-full rounded-full transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-300"
								style="width: {importProgress.percent ?? 0}%; background: linear-gradient(90deg, rgb(var(--brand-500)), rgb(var(--brand-400)));"
							></div>
						</div>
						<Button variant="danger" size="sm" onclick={cancelImport}>
							<X class="h-4 w-4" /> {uiText("ui.7cdc678d82b8a405")}
						</Button>
					</div>
				{:else}
					<div class="flex gap-3">
						<div class="flex-1">
							<label for="import-name" class="mb-1 block text-xs" style="color: rgb(var(--fg-subtle));">{t("instances.name")}</label>
							<Input id="import-name" bind:value={importName} placeholder={t("instances.modpackName")} />
						</div>
						<div class="w-36">
							<label for="import-loader" class="mb-1 block text-xs" style="color: rgb(var(--fg-subtle));">{t("instances.loader")}</label>
							<select id="import-loader" class="h-9 w-full rounded-md px-2 text-sm transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-300" style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg)); color: rgb(var(--fg));" bind:value={importLoader}>
								<option value="fabric">Fabric</option>
								<option value="forge">Forge</option>
								<option value="neoforge">NeoForge</option>
								<option value="quilt">Quilt</option>
							</select>
						</div>
					</div>
					<div class="flex gap-2">
						<Button variant="solid" onclick={importModpack} loading={importing}>
							<Import class="h-4 w-4" /> {t("instances.importModpack")}
						</Button>
						<Button variant="secondary" onclick={() => { showImport = false; importFile = null; }}>
							<X class="h-4 w-4" /> {t("common.cancel")}
						</Button>
					</div>
				{/if}
			</div>
		</Card>
	{/if}

	{#if showImportMrpack}
		<Card>
			<div class="flex flex-col gap-3">
				<p class="text-sm" style="color: rgb(var(--fg-muted));">{t("instances.importMrpackDesc")}</p>
				<p class="truncate text-xs" style="color: rgb(var(--fg-subtle));">{importMrpackFile}</p>
				{#if importingMrpack && importProgress}
					<div class="flex flex-col gap-2">
						<div class="flex items-center justify-between text-xs" style="color: rgb(var(--fg-muted));">
							<span>{importProgress.status ?? uiText("ui.31e6fe00c185a198")}</span>
							<span>{importProgress.percent ?? 0}%</span>
						</div>
						<div class="h-2 w-full overflow-hidden rounded-full" style="background: rgb(var(--bg-elevated));">
							<div
								class="h-full rounded-full transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-300"
								style="width: {importProgress.percent ?? 0}%; background: linear-gradient(90deg, rgb(var(--brand-500)), rgb(var(--brand-400)));"
							></div>
						</div>
						<Button variant="danger" size="sm" onclick={cancelImport}>
							<X class="h-4 w-4" /> {uiText("ui.7cdc678d82b8a405")}
						</Button>
					</div>
				{:else}
					<div class="flex gap-3">
						<div class="flex-1">
							<label for="import-mrpack-name" class="mb-1 block text-xs" style="color: rgb(var(--fg-subtle));">{t("instances.name")}</label>
							<Input id="import-mrpack-name" bind:value={importMrpackName} placeholder={t("instances.modpackName")} />
						</div>
					</div>
					<div class="flex gap-2">
						<Button variant="solid" onclick={importMrpack} loading={importingMrpack}>
							<Import class="h-4 w-4" /> {t("instances.importMrpack")}
						</Button>
						<Button variant="secondary" onclick={() => { showImportMrpack = false; importMrpackFile = null; }}>
							<X class="h-4 w-4" /> {t("common.cancel")}
						</Button>
					</div>
				{/if}
			</div>
		</Card>
	{/if}

	<InstanceFilters
		bind:searchQuery
		bind:sortBy
		bind:groupFilter
		bind:viewMode
		{groups}
		bind:searchInput
	/>

	<InstanceGrid
		instances={filteredInstances}
		{viewMode}
		searchQuery={activeSearch}
		{selectionMode}
		{selectedIds}
		{instanceColors}
		{launchingInstanceId}
		navigatingInstanceId={navigatingId}
		{colorOptions}
		{getIconSrc}
		{formatTimeAgo}
		{formatBytes}
		activeId={profiles.activeId}
		onSelect={selectInstance}
		onToggleSelect={toggleSelect}
		onQuickPlay={quickPlay}
		onEdit={openEditInstance}
		onOpenFolder={openFolder}
		onDelete={(p) => { confirmDeleteInstance = { id: p.id, name: p.name, gameDir: p.gameDir }; }}
		onDuplicate={duplicateInstance}
		onScreenshots={openScreenshots}
		onNotes={openNotes}
		onHealthCheck={checkHealth}
	/>

	<div class="mt-4 mb-6 rounded-3xl bg-bg-elevated border border-fg/[0.06] p-4 flex flex-col sm:flex-row items-center justify-between gap-4 shadow-sm">
		<div class="flex items-center gap-4">
			<div class="w-10 h-10 rounded-full bg-brand-500/10 border border-brand-500/20 flex items-center justify-center shrink-0">
				<Boxes class="w-5 h-5 text-brand-500" />
			</div>
			<div>
				<div class="text-xs font-black text-fg flex items-center gap-2">
					<span>{uiText("ui.a8e9a7046fb6f751")}</span>
					<span class="text-[9px] font-black uppercase px-2.5 py-0.5 rounded-full bg-fg/5 text-fg/60 border border-fg/[0.12]">
						{filteredInstances.length} {filteredInstances.length === 1 ? uiText("ui.a48ea8949d3ec2ab") : uiText("ui.3e9ebd1203add9dc")}
					</span>
				</div>
				<p class="text-[11px] text-fg/35 mt-0.5">{uiText("ui.2f63f5e1db3d4813")}</p>
			</div>
		</div>
		<div class="flex items-center gap-2.5 w-full sm:w-auto justify-end">
			<button type="button" class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2" })}
				onclick={() => { if (profiles.active) openFolder(profiles.active.id); else toast(uiText("ui.5c273c589e2cff6c"), "info"); }}
			>
				<FolderOpen class="w-4 h-4 text-fg/35" /> {uiText("ui.421131d86551a38d")}
			</button>
		</div>
	</div>

	{#if showImportCode}
		<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/80" transition:fade={{ easing: quintOut, duration: 220 }}>
			<div class="w-full max-w-md bg-bg-elevated border border-brand-500/30 rounded-3xl p-6 shadow-2xl">
				<div class="flex items-center justify-between mb-4">
					<div class="flex items-center gap-2">
						<Sparkles class="w-5 h-5 text-brand-500" />
						<h3 class="text-sm font-black text-fg">{uiText("ui.0032f36998624ab3")}</h3>
					</div>
					<button type="button" class={launcherButton({ variant: "ghost", size: "icon", class: "" })} onclick={() => showImportCode = false}>
						<X class="w-4 h-4" />
					</button>
				</div>

				<p class="text-xs text-fg/60 mb-4 leading-relaxed">
					{uiText("ui.b7932d55d1d821ed")} <span class="font-mono text-brand-400">{uiText("ui.3b76dd3f39108941")}</span>{uiText("ui.99905187d486e33d")}
				</p>

				<div class="mb-5">
					<input 
						type="text" 
						bind:value={shareCodeInput} 
						placeholder={uiText("ui.3b76dd3f39108941")} 
						class="w-full bg-bg-elevated border border-fg/[0.06] focus:border-emerald-500/50 rounded-2xl px-4 py-3 text-sm text-fg font-mono uppercase tracking-widest outline-none transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-300"
					/>
				</div>

				<div class="flex items-center justify-end gap-3">
					<button 
						type="button" 
						class={launcherButton({ variant: "secondary", size: "sm", class: "" })}
						onclick={() => showImportCode = false}
					>
						{uiText("common.cancel")}
					</button>
					<button 
						type="button" 
						class={launcherButton({ variant: "secondary", size: "sm", class: "from-emerald-500 via-teal-500 to-emerald-600 flex items-center gap-2 disabled:opacity-50" })}
						disabled={isImportingCode}
						onclick={handleImportShareCode}
					>
						<Sparkles class="w-3.5 h-3.5 fill-current" />
						{isImportingCode ? uiText("ui.eeb8c8bbcaab66e1") : uiText("settings.import")}
					</button>
				</div>
			</div>
		</div>
	{/if}

	{#if healthCheckId}
		<div class="fixed inset-0 z-50 flex items-center justify-center bg-bg-overlay/80" transition:fade={{ easing: quintOut, duration: 220 }}
			onclick={(e) => { if (e.target === e.currentTarget) closeHealthCheck(); }}
			onkeydown={(e) => { if (e.key === "Escape") closeHealthCheck(); }}
			role="dialog" aria-modal="true" aria-label={t("health.title")} tabindex="-1"
		>
			<Card class="w-full max-w-md">
				<div class="mb-4 flex items-center justify-between">
					<div class="flex items-center gap-2">
						<HeartPulse class="h-5 w-5" style="color: rgb(var(--fg-muted));" />
						<h2 class="text-lg font-medium">{t("health.title")}</h2>
					</div>
					<button class={launcherButton({ variant: "ghost", size: "icon", class: "grid place-items-center" })} style="color: rgb(var(--fg-subtle));" onclick={closeHealthCheck} aria-label={t("common.close")}>
						<X class="h-4 w-4" />
					</button>
				</div>
				{#if healthChecking}
					<div class="flex items-center justify-center py-8">
						<div class="h-6 w-6 animate-spin rounded-full border-2 border-t-transparent" style="border-color: rgb(var(--brand-500)); border-top-color: transparent;"></div>
						<span class="ml-2 text-sm" style="color: rgb(var(--fg-muted));">{t("health.checking")}</span>
					</div>
				{:else if healthResult}
					<div class="flex flex-col gap-3">
						<div class="flex items-center gap-2 rounded-lg px-3 py-2" style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg));">
							{#if healthResult.clientJar}<Check class="h-4 w-4" style="color: rgb(var(--success));" />{:else}<HeartPulse class="h-4 w-4" style="color: rgb(var(--danger));" />{/if}
							<span class="text-sm">{t("health.clientJar")}</span>
							<span class="ml-auto text-xs" style="color: {healthResult.clientJar ? 'rgb(var(--success))' : 'rgb(var(--danger))'};">
								{healthResult.clientJar ? t("health.present") : t("health.missing")}
							</span>
						</div>
						<div class="flex items-center gap-2 rounded-lg px-3 py-2" style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg));">
							{#if healthResult.natives}<Check class="h-4 w-4" style="color: rgb(var(--success));" />{:else}<HeartPulse class="h-4 w-4" style="color: rgb(var(--warning));" />{/if}
							<span class="text-sm">{t("health.natives")}</span>
							<span class="ml-auto text-xs" style="color: {healthResult.natives ? 'rgb(var(--success))' : 'rgb(var(--warning))'};">
								{healthResult.natives ? t("health.present") : t("health.missing")}
							</span>
						</div>
						<div class="flex items-center gap-2 rounded-lg px-3 py-2" style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg));">
							{#if healthResult.modsOk}<Check class="h-4 w-4" style="color: rgb(var(--success));" />{:else}<HeartPulse class="h-4 w-4" style="color: rgb(var(--warning));" />{/if}
							<span class="text-sm">{t("health.mods")}</span>
							<span class="ml-auto text-xs" style="color: {healthResult.modsOk ? 'rgb(var(--success))' : 'rgb(var(--warning))'};">
								{healthResult.modsOk ? t("health.present") : t("health.missing")}
							</span>
						</div>
						{#if healthResult.issues.length > 0}
							<div class="rounded-lg px-3 py-2" style="border: 1px solid rgb(var(--danger) / 0.3); background: rgb(var(--danger) / 0.1);">
								<p class="text-xs font-medium" style="color: rgb(var(--danger));">{t("health.issuesFound")}</p>
								{#each healthResult.issues as issue}<p class="mt-1 text-xs" style="color: rgb(var(--danger));">- {issue}</p>{/each}
							</div>
						{:else}
							<div class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm" style="border: 1px solid rgb(var(--success) / 0.3); background: rgb(var(--success) / 0.1); color: rgb(var(--success));">
								<Check class="h-4 w-4" /> {t("health.healthy")}
							</div>
						{/if}
					</div>
				{/if}
			</Card>
		</div>
	{/if}

	{#if fileBrowserId}
		<div class="fixed inset-0 z-50 flex items-center justify-center bg-bg-overlay/80" transition:fade={{ easing: quintOut, duration: 220 }}
			onclick={(e) => { if (e.target === e.currentTarget) closeFileBrowser(); }}
			onkeydown={(e) => { if (e.key === "Escape") closeFileBrowser(); }}
			role="dialog" aria-modal="true" aria-label={t("files.title")} tabindex="-1"
		>
			<Card class="flex max-h-[80vh] w-full max-w-lg flex-col">
				<div class="mb-4 flex items-center justify-between">
					<div class="flex items-center gap-2">
						<FolderTree class="h-5 w-5" style="color: rgb(var(--fg-muted));" />
						<h2 class="text-lg font-medium">{t("files.title")}</h2>
					</div>
					<button class={launcherButton({ variant: "ghost", size: "icon", class: "grid place-items-center" })} style="color: rgb(var(--fg-subtle));" onclick={closeFileBrowser} aria-label={t("common.close")}>
						<X class="h-4 w-4" />
					</button>
				</div>
				{#if fileTreePath}
					<button class={launcherButton({ variant: "ghost", size: "sm", class: "mb-2 text-left" })} style="color: rgb(var(--brand-500));" onclick={() => navigateFileTree(null)}>
						.. / {fileTreePath}
					</button>
				{/if}
				<div class="flex-1 overflow-y-auto">
					{#if fileTreeLoading}
						<div class="space-y-2">{#each Array(6) as _}<Skeleton variant="text" />{/each}</div>
					{:else if fileTree.length === 0}
						<div class="flex flex-col items-center gap-2 py-8 text-center">
							<FolderTree class="h-8 w-8" style="color: rgb(var(--fg-subtle));" />
							<p class="text-sm" style="color: rgb(var(--fg-muted));">{t("files.empty")}</p>
						</div>
					{:else}
						<div class="flex flex-col gap-0.5">
							{#each fileTree as entry (entry.path)}
								<button class={launcherButton({ variant: "ghost", size: "sm", class: "flex items-center gap-2 text-left" })}
									onclick={() => { if (entry.isDir) { navigateFileTree(fileTreePath ? `${fileTreePath}/${entry.name}` : entry.name); } }}
								>
									{#if entry.isDir}<FolderOpen class="h-4 w-4 shrink-0" style="color: rgb(var(--brand-500));" />{:else}<span class="h-4 w-4 shrink-0"></span>{/if}
									<span class="truncate">{entry.name}</span>
									{#if !entry.isDir}
										<span class="ml-auto text-xs" style="color: rgb(var(--fg-subtle));">
											{entry.size < 1024 ? `${entry.size} B` : entry.size < 1024 * 1024 ? `${(entry.size / 1024).toFixed(1)} KB` : `${(entry.size / (1024 * 1024)).toFixed(1)} MB`}
										</span>
									{/if}
								</button>
							{/each}
						</div>
					{/if}
				</div>
			</Card>
		</div>
	{/if}

	{#if screenshotsId}
		<div class="fixed inset-0 z-50 flex items-center justify-center bg-bg-overlay/80" transition:fade={{ easing: quintOut, duration: 220 }}
			onclick={(e) => { if (e.target === e.currentTarget) closeScreenshots(); }}
			onkeydown={(e) => { if (e.key === "Escape") closeScreenshots(); }}
			role="dialog" aria-modal="true" aria-label={t("screenshots.title")} tabindex="-1"
		>
			<Card class="flex max-h-[80vh] w-full max-w-2xl flex-col">
				<div class="mb-4 flex items-center justify-between">
					<div class="flex items-center gap-2">
						<Image class="h-5 w-5" style="color: rgb(var(--fg-muted));" />
						<h2 class="text-lg font-medium">{t("screenshots.title")}</h2>
					</div>
					<button class={launcherButton({ variant: "ghost", size: "icon", class: "grid place-items-center" })} style="color: rgb(var(--fg-subtle));" onclick={closeScreenshots} aria-label={t("common.close")}>
						<X class="h-4 w-4" />
					</button>
				</div>
				<div class="flex-1 overflow-y-auto">
					{#if screenshotsLoading}
						<div class="space-y-2">{#each Array(6) as _}<Skeleton variant="text" />{/each}</div>
					{:else if screenshots.length === 0}
						<div class="flex flex-col items-center gap-2 py-8 text-center">
							<Image class="h-8 w-8" style="color: rgb(var(--fg-subtle));" />
							<p class="text-sm" style="color: rgb(var(--fg-muted));">{t("screenshots.noScreenshots")}</p>
						</div>
					{:else}
						<div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
							{#each screenshots as shot (shot.path)}
								<div class="overflow-hidden rounded-xl border border-fg/[0.12] bg-bg-overlay/40 shadow-sm group">
									<div class="aspect-video overflow-hidden flex items-center justify-center bg-bg-overlay/60">
										<img decoding="async" src={shot.thumbPath ? convertFileSrc(shot.thumbPath) : convertFileSrc(shot.path)} alt={shot.name} class="w-full h-full object-cover group-hover:scale-105 transition-transform" loading="lazy" />
									</div>
									<div class="px-2.5 py-1.5 flex items-center justify-between">
										<div class="min-w-0 flex-1">
											<p class="truncate text-[11px] font-bold text-fg/90">{shot.name}</p>
											<p class="text-[10px] text-fg/35">{new Date(shot.modified).toLocaleDateString()}</p>
										</div>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			</Card>
		</div>
	{/if}

	{#if notesId}
		<div class="fixed inset-0 z-50 flex items-center justify-center bg-bg-overlay/80" transition:fade={{ easing: quintOut, duration: 220 }}
			onclick={(e) => { if (e.target === e.currentTarget) closeNotes(); }}
			onkeydown={(e) => { if (e.key === "Escape") closeNotes(); }}
			role="dialog" aria-modal="true" aria-label={t("instances.notes")} tabindex="-1"
		>
			<Card class="w-full max-w-md">
				<div class="mb-4 flex items-center justify-between">
					<div class="flex items-center gap-2">
						<StickyNote class="h-5 w-5" style="color: rgb(var(--fg-muted));" />
						<h2 class="text-lg font-medium">{t("instances.notes")}</h2>
					</div>
					<button class={launcherButton({ variant: "ghost", size: "icon", class: "grid place-items-center" })} style="color: rgb(var(--fg-subtle));" onclick={closeNotes} aria-label={t("common.close")}>
						<X class="h-4 w-4" />
					</button>
				</div>
				<textarea class="min-h-[200px] w-full resize-y rounded-md p-3 text-sm outline-none" style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg)); color: rgb(var(--fg));"
					placeholder={t("instances.notesPlaceholder")} aria-label={t("instances.notes")} bind:value={notesText}
				></textarea>
				<div class="mt-3 flex justify-end gap-2">
					<Button variant="secondary" onclick={closeNotes}>{t("common.cancel")}</Button>
					<Button variant="solid" onclick={saveNotes}>{t("common.save")}</Button>
				</div>
			</Card>
		</div>
	{/if}

	<Modal isOpen={editingInstance !== null} onClose={() => (editingInstance = null)} title={t("instances.edit")} maxWidth="max-w-xl">
		{#if editingInstance}
			<div class="flex flex-col gap-4 text-xs select-none">
				<div class="space-y-1.5">
					<label for="edit-instance-name" class="block text-xs font-bold text-fg/70 uppercase tracking-widest">{t("instances.name")}</label>
					<div class="flex items-center gap-3">
						<div class="h-11 w-11 rounded-2xl bg-bg-elevated border border-fg/[0.12] flex items-center justify-center shrink-0 p-1">
							<img loading="lazy" decoding="async" src={getIconSrc(editIcon)} alt={uiText("ui.665ba1908fbc76bf")} class="w-8 h-8 object-contain [image-rendering:pixelated]" />
						</div>
						<Input id="edit-instance-name" bind:value={editName} placeholder={t("instances.namePlaceholder")} />
					</div>
				</div>

				<div class="space-y-1.5">
					<span class="block text-xs font-bold text-fg/70 uppercase tracking-widest">{uiText("ui.f15f8f19139853cf")}</span>
					<div class="flex items-center gap-1.5 bg-bg-elevated p-1.5 rounded-2xl border border-fg/[0.12]">
						{#each [{ id: "grass_block", label: uiText("ui.40d7347a8c8fa495"), src: "/grass_block.png" }, { id: "modpack_fo", label: uiText("ui.e6e64f0d1eeaced1"), src: "/modpack_fo_icon.png" }, { id: "modpack_better_mc", label: uiText("ui.aaa7a30b891a76cc"), src: "/modpack_bmc_icon.webp" }, { id: "modpack_cobblemon", label: uiText("ui.7f1c6e0fb0701436"), src: "/modpack_cobblemon_icon.png" }, { id: "logo", label: uiText("ui.d707dc2f1936efd8"), src: "/logo.png" }, { id: "grass_head", label: "Steve", src: "/grass_head.png" }] as ip}
							<button type="button" class="w-8 h-8 rounded-xl p-1 transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer flex items-center justify-center {editIcon === ip.id ? 'bg-brand-500/20 border border-brand-500 scale-105' : 'hover:bg-fg/5 opacity-60 hover:opacity-100'}"
								onclick={() => editIcon = ip.id} title={ip.label}
							>
								<img loading="lazy" decoding="async" src={ip.src} alt={ip.label} class="w-6 h-6 object-contain [image-rendering:pixelated]" />
							</button>
						{/each}
					</div>
				</div>

				<div class="space-y-1.5">
					<span class="block text-xs font-bold text-fg/70 uppercase tracking-widest">{t("instances.version")}</span>
					<FilterableVersionSelect versions={availableVersions} bind:value={editVersion} loading={versionsLoading} />
				</div>

				<div class="bg-bg-elevated border border-emerald-500/20 rounded-2xl p-3.5 space-y-2">
					<div class="flex items-center justify-between text-xs">
						<div class="flex items-center gap-2.5">
							<div class="w-7 h-7 rounded-lg bg-emerald-500/10 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
								<Cpu class="w-3.5 h-3.5" />
							</div>
							<div>
								<span class="font-bold text-fg block">{uiText("ui.2583ad9ceffb7cc1")}</span>
								<span class="text-[10px] text-fg/35 block">{uiText("ui.1a2bd1d28cb68c00")} {Math.round(systemRamMb / 1024)} {uiText("ui.1fdb78b55e9d4eb5")}</span>
							</div>
						</div>
						<span class="text-[10px] font-mono font-bold text-emerald-400 bg-emerald-500/10 px-2.5 py-0.5 rounded-full border border-emerald-500/20">{uiText("ui.dde4960e8bbb2c8b")}</span>
					</div>
					<p class="text-[10px] text-fg/35 leading-relaxed">
						{uiText("ui.c19e0e6e3c39cf15")}
					</p>
				</div>

				<div class="space-y-1.5">
					<label for="edit-jvm-args" class="block text-xs font-bold text-fg/70 uppercase tracking-widest">{uiText("ui.ee15867ed5aa0c3e")}</label>
					<input id="edit-jvm-args" type="text" bind:value={editJvmArgs} placeholder={uiText("ui.5854817f5f7815b4")}
						class="w-full bg-bg-elevated border border-fg/[0.06] focus:border-emerald-500/50 rounded-xl px-4 py-2.5 text-xs text-fg font-mono outline-none transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-300"
					/>
					<p class="text-[10px] text-fg/35">{uiText("ui.cc1ba216a6957757")}</p>
				</div>

				<div class="flex justify-end gap-2 border-t pt-4" style="border-color: rgb(var(--border));">
					<Button variant="secondary" onclick={() => (editingInstance = null)}>{t("common.cancel")}</Button>
					<Button variant="solid" onclick={saveEditInstance} loading={editSaving} disabled={!editName.trim() || !editVersion}>
						<Check class="h-4 w-4" /> {t("instances.saveChanges")}
					</Button>
				</div>
			</div>
		{/if}
	</Modal>

	{#if confirmDeleteInstance}
		<div class="fixed inset-0 z-50 flex items-center justify-center bg-bg-overlay/80 p-4" transition:fade={{ easing: quintOut, duration: 220 }}>
			<div class="w-full max-w-md bg-bg-elevated border border-red-500/30 rounded-3xl p-6 shadow-2xl space-y-4 select-none">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-red-500/15 border border-red-500/30 flex items-center justify-center text-red-400 shrink-0">
						<Trash2 class="w-5 h-5" />
					</div>
					<div>
						<h3 class="text-sm font-black text-fg">{uiText("instances.deleteConfirmTitle")}</h3>
						<p class="text-[11px] text-fg/35">{uiText("ui.4d2a771f48f9cbaa")}</p>
					</div>
				</div>
				<div class="bg-bg-elevated p-3.5 rounded-2xl border border-fg/[0.06] text-xs text-fg/70 space-y-1.5">
					<p>{uiText("ui.1af399ce0ececeb2")} <strong class="text-fg">"{confirmDeleteInstance.name}"</strong>?</p>
					<p class="text-[10px] text-fg/35 font-mono break-all">{uiText("ui.cc9f3b6a13a0719a")} {confirmDeleteInstance.gameDir}</p>
					<p class="text-[11px] text-red-400/90 font-medium">{uiText("ui.fb29cb4d3bcf8336")}</p>
				</div>
				<div class="flex justify-end gap-2.5 pt-2">
					<button type="button" class={launcherButton({ variant: "secondary", size: "sm", class: "" })}
						onclick={() => (confirmDeleteInstance = null)}
					>{uiText("common.cancel")}</button>
					<button type="button" class={launcherButton({ variant: "danger", size: "sm", class: "flex items-center gap-2" })}
						disabled={deleting}
						onclick={async () => {
							if (!confirmDeleteInstance) return;
							deleting = true;
							const id = confirmDeleteInstance.id;
							const name = confirmDeleteInstance.name;
							confirmDeleteInstance = null;
							await deleteInstance(id);
							deleting = false;
							toast(uiText("ui.bc3196ede9748fff", {arg0: (name)}), "success");
						}}
					>
						<Trash2 class="w-3.5 h-3.5" /> {uiText("ui.37045d507711e562")}
					</button>
				</div>
			</div>
		</div>
	{/if}
</div>
