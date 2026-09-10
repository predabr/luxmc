<script lang="ts">
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";
	import { fade } from "svelte/transition";
	import Heading from "$lib/components/ui/Heading.svelte";
	import Card from "$lib/components/ui/Card.svelte";
	import Button from "$lib/components/ui/Button.svelte";
	import Input from "$lib/components/ui/Input.svelte";
	import Badge from "$lib/components/ui/Badge.svelte";
	import Skeleton from "$lib/components/ui/Skeleton.svelte";
	import {
		Plus,
		Trash2,
		Check,
		Boxes,
		Box,
		LayoutGrid,
		List,
		Copy,
		FolderOpen,
		Image,
		Search,
		ArrowUpDown,
		X,
		Import,
		RefreshCw,
		Heart,
		HeartPulse,
		FolderTree,
		ShieldCheck,
		AlertTriangle,
		Star,
		StickyNote,
		Clock,
		HardDrive,
		Download,
		Group,
		Paintbrush,
		SquareCheck,
		Play,
		Pencil,
		Sparkles,
		Zap,
		Loader2,
		MoreVertical
	} from "lucide-svelte";
	import FilterableVersionSelect from "$lib/components/ui/FilterableVersionSelect.svelte";
	import Modal from "$lib/components/ui/Modal.svelte";
	import { open } from "@tauri-apps/plugin-dialog";
	import { convertFileSrc } from "@tauri-apps/api/core";
	import { profiles, type Profile } from "$lib/stores/profiles.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import {
		api,
		fetchVersionsDirect,
		versionsList,
		instancesDuplicate,
		instancesOpenFolder,
		instancesScreenshots,
		instanceImportModpack,
		instanceImportMrpack,
		instanceHealthCheck,
		instanceFileTree,
		instanceSetNotes,
		instanceSetFavorite,
		instanceExportZip,
		instanceBackupSaves,
		instanceRestoreSaves,
		instanceRepair,
		getSystemSpecs,
		profilesCreate,
		optimizerInstallPerfPack,
		launchGame,
		versionsCheckInstalled,
		versionsDownload,
		authDevLogin,
		discordSetActivity,
		type HealthCheckResult,
		type FileTreeEntry,
	} from "$lib/api";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { toast } from "$lib/stores/toasts.svelte";

	const { t } = useTranslation();

	let openCardMenuId = $state<string | null>(null);
	let showCreate = $state(false);
	let showImport = $state(false);
	let showImportMrpack = $state(false);
	let importFile = $state<string | null>(null);
	let importMrpackFile = $state<string | null>(null);
	let importName = $state("");
	let importMrpackName = $state("");
	let importVersion = $state("1.21.4");
	let importLoader = $state("fabric");
	let importing = $state(false);
	let importingMrpack = $state(false);
	let newName = $state("My Instance");
	let newVersion = $state("1.21.4");
	let newLoader = $state("vanilla");
	let newIcon = $state("grass_block");
	let selectedRamGb = $state(4);
	let newAutoOptimize = $state(true);
	let newUseVulkan = $state(false);
	let newInstallPerfPack = $state(true);
	let creating = $state(false);
	let lastError = $state<string | null>(null);
	let createSuccess = $state(false);

	let systemRamMb = $state(8192);

	const ramPresets = $derived.by(() => {
		const totalGb = Math.floor(systemRamMb / 1024);
		if (totalGb <= 4) return [1, 2, 3];
		if (totalGb <= 8) return [2, 4, 6];
		if (totalGb <= 16) return [2, 4, 6, 8, 12];
		if (totalGb <= 32) return [2, 4, 6, 8, 12, 16, 24];
		return [2, 4, 6, 8, 12, 16, 24, 32];
	});

	const iconPresets = [
		{ id: "grass_block", label: "Bloco de Grama", src: "/grass_block.png" },
		{ id: "modpack_fo", label: "Fabulously Optimized", src: "/modpack_fo_icon.png" },
		{ id: "modpack_better_mc", label: "Better MC", src: "/modpack_bmc_icon.webp" },
		{ id: "modpack_cobblemon", label: "Cobblemon", src: "/modpack_cobblemon_icon.png" },
		{ id: "logo", label: "Luxmc Logo", src: "/logo.png" },
		{ id: "grass_head", label: "Steve", src: "/grass_head.png" }
	];

	function getIconSrc(iconStr?: string): string {
		if (!iconStr || iconStr === "grass_block") return "/grass_block.png";
		if (iconStr === "modpack_fo") return "/modpack_fo_icon.png";
		if (iconStr === "modpack_better_mc") return "/modpack_bmc_icon.webp";
		if (iconStr === "modpack_cobblemon") return "/modpack_cobblemon_icon.png";
		if (iconStr === "logo") return "/logo.png";
		if (iconStr === "grass_head") return "/grass_head.png";
		if (iconStr.startsWith("/") || iconStr.startsWith("http") || iconStr.startsWith("data:")) return iconStr;
		try {
			return convertFileSrc(iconStr);
		} catch {
			return "/grass_block.png";
		}
	}

	async function pickCustomIcon(isEdit: boolean = false) {
		const file = await open({
			title: "Selecionar Ícone da Instância",
			filters: [{ name: "Imagens", extensions: ["png", "jpg", "jpeg", "webp", "svg"] }]
		});
		if (typeof file === "string") {
			if (isEdit) {
				editIcon = file;
			} else {
				newIcon = file;
			}
			toast("Ícone personalizado selecionado!", "success");
		}
	}

	let editingInstance = $state<{
		id: string;
		name: string;
		mcVersion: string;
		loader: string;
		icon: string;
		ramMb: number;
		jvmArgs: string;
	} | null>(null);
	let editName = $state("");
	let editVersion = $state("");
	let editIcon = $state("grass_block");
	let editRamGb = $state(4);
	let editJvmArgs = $state("");
	let editSaving = $state(false);

	let confirmDeleteInstance = $state<{ id: string; name: string; gameDir: string } | null>(null);
	let deleting = $state(false);

	function openEditInstance(p: typeof profiles.list[0]) {
		editingInstance = {
			id: p.id,
			name: p.name,
			mcVersion: p.mcVersion,
			loader: p.loader,
			icon: p.icon || "grass_block",
			ramMb: p.ramMb || 4096,
			jvmArgs: p.jvmArgs || ""
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
					id: editingInstance.id,
					name: editName.trim(),
					mcVersion: editVersion,
					icon: editIcon,
					ramMb: editRamGb * 1024,
					jvmArgs: editJvmArgs
				}
			});
			profiles.update(editingInstance.id, {
				name: editName.trim(),
				mcVersion: editVersion,
				icon: editIcon,
				ramMb: editRamGb * 1024,
				jvmArgs: editJvmArgs
			});
			toast("Instância atualizada com sucesso!", "success");
			editingInstance = null;
		} catch (e) {
			lastError = "Falha ao atualizar instância: " + String(e);
			toast(t("instances.failedUpdate", { error: String(e) }), "error");
		} finally {
			editSaving = false;
		}
	}

	let viewMode = $state<"grid" | "list">("grid");
	let searchQuery = $state("");
	let sortBy = $state<"name" | "version" | "date" | "lastPlayed">("name");
	let groupFilter = $state<string>("all");

	let screenshotsId = $state<string | null>(null);
	let screenshots = $state<Array<{ name: string; path: string; modified: string }>>([]);
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
	let availableVersions = $state<Array<{ id: string; versionType: string; releaseTime: string }>>([]);
	let versionsLoading = $state(false);

	const colorOptions = [
		{ value: "red", color: "rgb(239, 68, 68)" },
		{ value: "blue", color: "rgb(59, 130, 246)" },
		{ value: "green", color: "rgb(34, 197, 94)" },
		{ value: "yellow", color: "rgb(234, 179, 8)" },
		{ value: "purple", color: "rgb(168, 85, 247)" },
		{ value: "orange", color: "rgb(249, 115, 22)" },
	];

	const groups = ["all", "Modded", "Vanilla", "Servers", "Favorites"];

	const filteredInstances = $derived(
		profiles.list
			.filter(
				(p) =>
					!searchQuery ||
					p.name.toLowerCase().includes(searchQuery.toLowerCase()),
			)
			.filter((p) => {
				if (groupFilter === "all") return true;
				if (groupFilter === "Favorites") return p.favorite;
				if (groupFilter === "Modded") return p.loader !== "vanilla";
				if (groupFilter === "Vanilla") return p.loader === "vanilla";
				if (groupFilter === "Servers") return p.group === "Servers";
				return true;
			})
			.toSorted((a, b) => {
				if (sortBy === "name") return a.name.localeCompare(b.name);
				if (sortBy === "version") return a.mcVersion.localeCompare(b.mcVersion);
				if (sortBy === "lastPlayed") return (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0);
				return b.createdAt - a.createdAt;
			}),
	);

	function getRamRecommendation(modCount: number, mcVersion: string): number {
		const isModern = mcVersion >= "1.18";
		if (modCount === 0) return isModern ? 2048 : 1024;
		if (modCount <= 10) return isModern ? 3072 : 2048;
		if (modCount <= 30) return isModern ? 4096 : 3072;
		if (modCount <= 60) return isModern ? 6144 : 4096;
		return isModern ? 8192 : 6144;
	}

	function formatBytes(bytes: number): string {
		if (bytes === 0) return "0 B";
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
		return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
	}

	function formatTimeAgo(timestamp: number): string {
		if (!timestamp) return "Never";
		const diff = Date.now() - timestamp;
		const minutes = Math.floor(diff / 60000);
		if (minutes < 1) return "Just now";
		if (minutes < 60) return `${minutes}m ago`;
		const hours = Math.floor(minutes / 60);
		if (hours < 24) return `${hours}h ago`;
		const days = Math.floor(hours / 24);
		return `${days}d ago`;
	}

	function loadColors() {
		try {
			const stored = localStorage.getItem("luxmc.instanceColors");
			if (stored) instanceColors = JSON.parse(stored);
		} catch {}
	}

	function saveColor(id: string, color: string) {
		instanceColors = { ...instanceColors, [id]: color };
		localStorage.setItem("luxmc.instanceColors", JSON.stringify(instanceColors));
	}

	function removeColor(id: string) {
		const next = { ...instanceColors };
		delete next[id];
		instanceColors = next;
		localStorage.setItem("luxmc.instanceColors", JSON.stringify(instanceColors));
	}

	function toggleSelect(id: string) {
		const next = new Set(selectedIds);
		if (next.has(id)) {
			next.delete(id);
		} else {
			next.add(id);
		}
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

	async function bulkDelete() {
		for (const id of selectedIds) {
			await api.invoke("profiles_delete", { id });
			profiles.remove(id);
		}
		selectedIds = new Set();
		selectionMode = false;
	}

	async function bulkDuplicate() {
		for (const id of selectedIds) {
			try {
				const p = await instancesDuplicate(id);
				profiles.add({
					id: p.id,
					name: p.name,
					icon: p.icon,
					mcVersion: p.mcVersion,
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

	onMount(() => {
		loadColors();
		void loadVersions();
		void loadSystemSpecs();
		window.addEventListener("keydown", handleInstanceKeydown);
		return () => window.removeEventListener("keydown", handleInstanceKeydown);
	});

	async function loadSystemSpecs() {
		try {
			const specs = await getSystemSpecs();
			if (specs?.totalRamMb) {
				systemRamMb = specs.totalRamMb;
				if (systemRamMb >= 16384 && selectedRamGb === 4) {
					selectedRamGb = 6;
				}
			}
		} catch {}
	}

	async function loadVersions() {
		try {
			const cached = localStorage.getItem("luxmc_cached_versions");
			if (cached) {
				const parsed = JSON.parse(cached);
				if (parsed?.versions?.length > 0) {
					availableVersions = parsed.versions;
					newVersion = parsed.latestRelease || availableVersions[0].id;
				}
			}
		} catch {}

		versionsLoading = availableVersions.length === 0;
		try {
			const response = await versionsList();
			availableVersions = response.versions;
			if (availableVersions.length > 0) {
				newVersion = response.latestRelease || availableVersions[0].id;
				try {
					localStorage.setItem("luxmc_cached_versions", JSON.stringify(response));
				} catch {}
			}
		} catch (error) {
			if (availableVersions.length === 0) {
				lastError = `Could not load Minecraft versions: ${String(error)}`;
				toast(lastError, "error");
			}
		} finally {
			versionsLoading = false;
		}
	}

	async function createProfile() {
		if (!newName.trim()) return;
		creating = true;
		lastError = null;
		createSuccess = false;
		try {
			const p = await api.invoke<{
				id: string;
				name: string;
				icon: string;
				mcVersion: string;
				loader: string;
				loaderVersion: string | null;
				gameDir: string;
				createdAt: string;
				updatedAt: string;
			}>("profiles_create", {
				input: {
					name: newName.trim(),
					mcVersion: newVersion,
					loader: newLoader,
					icon: newIcon,
					ramMb: selectedRamGb * 1024,
					autoOptimize: newAutoOptimize,
					useVulkan: newUseVulkan,
				},
			});

			if (newInstallPerfPack && newLoader !== "vanilla") {
				try {
					await optimizerInstallPerfPack(p.id);
				} catch (err) {
					console.warn("Could not install perf pack:", err);
				}
			}

			profiles.add({
				id: p.id,
				name: p.name,
				icon: p.icon || newIcon,
				mcVersion: p.mcVersion,
				loader: p.loader as "vanilla" | "fabric" | "forge" | "neoforge" | "quilt",
				loaderVersion: p.loaderVersion ?? undefined,
				gameDir: p.gameDir,
				createdAt: new Date(p.createdAt).getTime(),
				updatedAt: new Date(p.updatedAt).getTime(),
				group: newLoader === "vanilla" ? "Vanilla" : "Modded",
				ramMb: selectedRamGb * 1024,
				autoOptimize: newAutoOptimize,
				useVulkan: newUseVulkan,
			});
			profiles.activeId = p.id;
			createSuccess = true;
			toast(t("instances.createdSuccess"), "success");
			setTimeout(() => {
				showCreate = false;
				createSuccess = false;
				newName = "My Instance";
				newIcon = "grass_block";
				newAutoOptimize = true;
				newUseVulkan = false;
				newInstallPerfPack = true;
			}, 1500);
		} catch (e) {
			lastError = "Failed to create instance: " + String(e);
			toast(lastError, "error");
		} finally {
			creating = false;
		}
	}

	async function deleteInstance(id: string) {
		try {
			await api.invoke("profiles_delete", { id });
			profiles.remove(id);
		} catch (e) {
			toast(t("instances.failedDelete", { error: String(e) }), "error");
		}
	}

	function openNotes(id: string) {
		const p = profiles.list.find((x) => x.id === id);
		if (p) {
			notesId = id;
			notesText = p.notes ?? "";
		}
	}

	async function saveNotes() {
		if (!notesId) return;
		savingNotes = true;
		profiles.update(notesId, { notes: notesText });
		const id = notesId;
		const notes = notesText || null;
		try {
			await instanceSetNotes(id, notes);
			toast(t("instances.notesSaved"), "success");
		} catch (e) {
			toast(t("instances.failedSaveNotes", { error: String(e) }), "error");
		} finally {
			savingNotes = false;
			notesId = null;
			notesText = "";
		}
	}

	function closeNotes() {
		notesId = null;
		notesText = "";
	}

	async function pickModpackFile() {
		const file = await open({
			filters: [{ name: "Modpack", extensions: ["zip"] }],
		});
		if (typeof file === "string") {
			importFile = file;
			if (!importName) {
				const basename = file.split("/").pop() ?? "Imported Modpack";
				importName = basename.replace(/\.zip$/i, "");
			}
			showImport = true;
		}
	}

	async function importModpack() {
		if (!importFile || !importName.trim()) return;
		importing = true;
		try {
			const p = await instanceImportModpack(importFile, importName.trim(), importVersion, importLoader);
			profiles.add({
				id: p.id,
				name: p.name,
				icon: "",
				mcVersion: importVersion,
				loader: importLoader as "vanilla" | "fabric" | "forge" | "neoforge" | "quilt",
				gameDir: "",
				createdAt: Date.now(),
				updatedAt: Date.now(),
			});
			profiles.activeId = p.id;
			showImport = false;
			importFile = null;
			importName = "";
			toast(t("instances.createdSuccess"), "success");
		} catch (e) {
			toast(t("instances.failedImportModpack", { error: String(e) }), "error");
		} finally {
			importing = false;
		}
	}

	async function pickMrpackFile() {
		const file = await open({
			filters: [{ name: "Modrinth Modpack", extensions: ["mrpack"] }],
		});
		if (typeof file === "string") {
			importMrpackFile = file;
			if (!importMrpackName) {
				const basename = file.split("/").pop() ?? "Imported Modpack";
				importMrpackName = basename.replace(/\.mrpack$/i, "");
			}
			showImportMrpack = true;
		}
	}

	async function importMrpack() {
		if (!importMrpackFile || !importMrpackName.trim()) return;
		importingMrpack = true;
		try {
			const p = await instanceImportMrpack(importMrpackFile, importMrpackName.trim());
			profiles.add({
				id: p.id,
				name: p.name,
				icon: "",
				mcVersion: p.mcVersion,
				loader: p.loader as "vanilla" | "fabric" | "forge" | "neoforge" | "quilt",
				gameDir: p.gameDir,
				createdAt: Date.now(),
				updatedAt: Date.now(),
			});
			profiles.activeId = p.id;
			showImportMrpack = false;
			importMrpackFile = null;
			importMrpackName = "";
			toast(t("instances.createdSuccess"), "success");
		} catch (e) {
			toast(t("instances.failedImportMrpack", { error: String(e) }), "error");
		} finally {
			importingMrpack = false;
		}
	}

	function selectInstance(id: string) {
		if (selectionMode) {
			toggleSelect(id);
			return;
		}
		profiles.activeId = id;
		goto("/instances/" + id);
	}

	let launchingInstanceId = $state<string | null>(null);

	async function quickPlay(p: Profile) {
		if (launchingInstanceId) return;
		launchingInstanceId = p.id;
		profiles.activeId = p.id;
		try {
			let accountId = account.value?.uuid;
			if (!accountId) {
				const dev = await authDevLogin();
				accountId = dev.uuid;
			}
			const verId = p.mcVersion || "1.21.4";
			const installed = await versionsCheckInstalled(verId).catch(() => false);
			if (!installed) {
				toast(`Baixando Minecraft ${verId}...`, "info");
				await versionsDownload(verId);
			}
			const isVulkan = p.useVulkan === true;
			const res = await launchGame({
				versionId: verId,
				accountId: accountId || "",
				profileId: p.id,
				enableVulkan: isVulkan
			});
			gamingStats.onGameStart();
			appState.isGameRunning = true;
			appState.activeGameDetails = { name: p.name, version: verId, loader: p.loader };
			discordSetActivity({
				inGame: true,
				details: `Jogando ${p.name}`,
				state: `Minecraft ${verId} · Luxmc`,
				largeText: `Minecraft ${verId}`,
				largeImage: "default",
				startTime: Math.floor(Date.now() / 1000)
			}).catch(() => {});
			profiles.setLastPlayed(p.id);
			toast(`🎮 Minecraft ${verId} (${p.name}) iniciado! (PID: ${res.pid})`, "success");
		} catch (e) {
			toast("Falha ao iniciar jogo: " + String(e), "error");
		} finally {
			setTimeout(() => {
				launchingInstanceId = null;
			}, 2500);
		}
	}

	async function duplicateInstance(id: string) {
		try {
			const p = await instancesDuplicate(id);
			profiles.add({
				id: p.id,
				name: p.name,
				icon: p.icon,
				mcVersion: p.mcVersion,
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
		try {
			await instancesOpenFolder(id);
		} catch (e) {
			toast(t("instances.failedOpenFolder", { error: String(e) }), "error");
		}
	}

	async function openScreenshots(id: string) {
		screenshotsId = id;
		screenshotsLoading = true;
		try {
			screenshots = await instancesScreenshots(id);
		} catch (e) {
			toast(t("screenshots.loadFailed", { error: String(e) }), "error");
			screenshots = [];
		} finally {
			screenshotsLoading = false;
		}
	}

	function closeScreenshots() {
		screenshotsId = null;
		screenshots = [];
	}

	async function checkHealth(id: string) {
		healthCheckId = id;
		healthChecking = true;
		healthResult = null;
		try {
			healthResult = await instanceHealthCheck(id);
		} catch (e) {
			toast(t("instances.failedHealthCheck", { error: String(e) }), "error");
			healthResult = {
				clientJar: false,
				natives: false,
				modsOk: false,
				issues: ["Health check failed: " + String(e)],
			};
		} finally {
			healthChecking = false;
		}
	}

	function closeHealthCheck() {
		healthCheckId = null;
		healthResult = null;
	}

	async function browseFiles(id: string) {
		fileBrowserId = id;
		fileTreeLoading = true;
		fileTreePath = null;
		try {
			fileTree = await instanceFileTree(id);
		} catch (e) {
			toast(t("instances.failedFileTree", { error: String(e) }), "error");
			fileTree = [];
		} finally {
			fileTreeLoading = false;
		}
	}

	async function navigateFileTree(subPath: string) {
		if (!fileBrowserId) return;
		fileTreeLoading = true;
		fileTreePath = subPath;
		try {
			fileTree = await instanceFileTree(fileBrowserId, subPath);
		} catch (e) {
			toast(t("instances.failedFileTree", { error: String(e) }), "error");
			fileTree = [];
		} finally {
			fileTreeLoading = false;
		}
	}

	function closeFileBrowser() {
		fileBrowserId = null;
		fileTree = [];
		fileTreePath = null;
	}
</script>

<div class="mx-auto flex h-full max-w-6xl flex-col gap-6">
	<div class="flex items-center justify-between">
		<Heading>{t("instances.title")}</Heading>
		<div class="flex gap-2">
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

			<div class="flex items-center gap-2.5">
				<button 
					type="button"
					class="px-5 py-2.5 rounded-full bg-[#18191c] hover:bg-[#202127] border border-white/10 hover:border-white/20 text-xs font-bold text-white flex items-center gap-2 transition-all active:scale-95 shadow-sm cursor-pointer"
					onclick={pickModpackFile}
				>
					<Import class="h-4 w-4 text-brand-500" />
					Importar .zip
				</button>

				<button 
					type="button"
					class="px-5 py-2.5 rounded-full bg-[#18191c] hover:bg-[#202127] border border-white/10 hover:border-[#1bd96a]/40 text-xs font-bold text-white flex items-center gap-2 transition-all active:scale-95 shadow-sm cursor-pointer"
					onclick={pickMrpackFile}
				>
					<div class="w-3.5 h-3.5 rounded-full bg-[#1bd96a]/20 border border-[#1bd96a]/40 flex items-center justify-center">
						<div class="w-1.5 h-1.5 rounded-full bg-[#1bd96a]"></div>
					</div>
					Importar Modrinth .mrpack
				</button>

				<button 
					type="button"
					class="px-6 py-2.5 rounded-full bg-gradient-to-r from-brand-500 to-[#cba358] hover:brightness-110 active:scale-95 text-black font-black text-xs flex items-center gap-2 transition-all shadow-[0_0_20px_rgba(226,184,107,0.35)] cursor-pointer"
					onclick={() => { showCreate = !showCreate; lastError = null; createSuccess = false; }}
				>
					<Plus class="h-4 w-4 stroke-[3]" />
					Nova Instância
				</button>
			</div>
		</div>
	</div>

	{#if showCreate}
		<div class="rounded-3xl bg-[#141518] border border-white/10 p-6 shadow-2xl space-y-6 select-none" in:fade={{ duration: 200 }}>
			<!-- Header -->
			<div class="flex items-center justify-between border-b border-white/5 pb-4">
				<div>
					<h3 class="text-lg font-black text-white flex items-center gap-2">
						<Sparkles class="w-5 h-5 text-brand-500" /> Criar Nova Instância
					</h3>
					<p class="text-xs text-white/50 mt-0.5">Selecione o modloader e a versão desejada para configurar sua instância com alto desempenho.</p>
				</div>
				<button 
					type="button"
					class="h-8 w-8 rounded-full bg-white/5 hover:bg-white/10 text-white/60 hover:text-white flex items-center justify-center transition-all cursor-pointer"
					onclick={() => showCreate = false}
					title="Fechar"
				>
					<X class="w-4 h-4" />
				</button>
			</div>

			{#if createSuccess}
				<div class="flex items-center gap-3 rounded-2xl p-4 text-sm bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 font-bold">
					<Check class="h-5 w-5" />
					Instância "{newName}" criada com sucesso! Redirecionando...
				</div>
			{:else}
				<!-- Presets de 1-Clique -->
				<div class="space-y-2.5 bg-[#18191c]/60 p-4 rounded-3xl border border-white/5">
					<div class="flex items-center justify-between">
						<span class="text-xs font-bold text-brand-500 uppercase tracking-wider flex items-center gap-1.5">
							<Zap class="w-3.5 h-3.5 fill-current" /> Modelos Prontos (1-Clique)
						</span>
						<span class="text-[10px] text-white/40 font-medium">Configurações pré-otimizadas</span>
					</div>

					<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-2.5">
						{#each [
							{
								title: 'Vanilla Otimizado',
								version: '1.21.4',
								loader: 'fabric',
								ram: 4,
								desc: 'Fabric + pronto para Sodium, Iris & Lithium.',
								tag: 'Máximo FPS',
								tagColor: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20'
							},
							{
								title: 'PvP Clássico 1.8.9',
								version: '1.8.9',
								loader: 'vanilla',
								ram: 3,
								desc: 'Otimizado para Hypixel, Mush e servidores de PvP.',
								tag: 'Competitivo',
								tagColor: 'text-amber-400 bg-amber-500/10 border-amber-500/20'
							},
							{
								title: 'Survival 1.21.4',
								version: '1.21.4',
								loader: 'vanilla',
								ram: 4,
								desc: 'Última versão vanilla pura e estável da Mojang.',
								tag: 'Exploração',
								tagColor: 'text-blue-400 bg-blue-500/10 border-blue-500/20'
							},
							{
								title: 'Modded NeoForge',
								version: '1.21.1',
								loader: 'neoforge',
								ram: 6,
								desc: 'Perfil com 6 GB de RAM para modpacks pesados.',
								tag: 'Heavy Mods',
								tagColor: 'text-purple-400 bg-purple-500/10 border-purple-500/20'
							}
						] as preset}
							<button
								type="button"
								class="p-3.5 rounded-2xl border text-left transition-all flex flex-col justify-between h-28 cursor-pointer bg-[#141518] border-white/5 hover:border-brand-500/50 hover:bg-[#1f2026] group relative overflow-hidden active:scale-98"
								onclick={() => {
									newLoader = preset.loader;
									newVersion = preset.version;
									newName = preset.title;
									selectedRamGb = preset.ram;
									toast(`Modelo "${preset.title}" aplicado com sucesso!`, "success");
								}}
							>
								<div class="flex items-center justify-between w-full">
									<span class="text-xs font-black text-white group-hover:text-brand-500 transition-colors truncate">{preset.title}</span>
									<span class="text-[8px] font-extrabold uppercase px-1.5 py-0.5 rounded border {preset.tagColor} shrink-0">{preset.tag}</span>
								</div>
								<p class="text-[10px] text-white/40 leading-snug">{preset.desc}</p>
								<div class="flex items-center gap-2 text-[9px] font-mono text-white/30">
									<span>{preset.version}</span>
									<span>·</span>
									<span class="capitalize">{preset.loader}</span>
									<span>·</span>
									<span>{preset.ram} GB</span>
								</div>
							</button>
						{/each}
					</div>
				</div>

				<!-- Step 1: Select Loader -->
				<div class="space-y-2">
					<span class="text-xs font-bold text-white/70 uppercase tracking-wider block">1. Selecione o Modloader</span>
					<div class="grid grid-cols-2 sm:grid-cols-5 gap-3">
						{#each [
							{ id: 'vanilla', name: 'Vanilla', desc: 'Minecraft Oficial Puro', badge: 'Estável', color: 'border-emerald-500/30' },
							{ id: 'fabric', name: 'Fabric', desc: 'Mais Leve & Alto FPS', badge: 'Recomendado', color: 'border-blue-500/30' },
							{ id: 'neoforge', name: 'NeoForge', desc: 'Mods Modernos 1.20+', badge: 'Novo', color: 'border-amber-500/30' },
							{ id: 'forge', name: 'Forge', desc: 'Maior Biblioteca Clássica', badge: 'Clássico', color: 'border-orange-500/30' },
							{ id: 'quilt', name: 'Quilt', desc: 'Ecossistema Aberto', badge: 'Comunitário', color: 'border-purple-500/30' }
						] as ldr}
							<button
								type="button"
								class="p-3.5 rounded-2xl border text-left transition-all flex flex-col justify-between h-24 cursor-pointer {newLoader === ldr.id ? 'bg-[#222328] border-brand-500 shadow-[0_0_15px_rgba(226,184,107,0.25)]' : 'bg-[#18191c] border-white/5 hover:border-white/15'}"
								onclick={() => {
									newLoader = ldr.id;
									newName = `Minecraft ${newVersion} (${ldr.name})`;
								}}
							>
								<div class="flex items-center justify-between w-full">
									<span class="text-xs font-black text-white">{ldr.name}</span>
									<span class="text-[8px] font-extrabold uppercase px-1.5 py-0.5 rounded bg-white/5 text-white/60">{ldr.badge}</span>
								</div>
								<span class="text-[10px] text-white/40 leading-snug">{ldr.desc}</span>
							</button>
						{/each}
					</div>
				</div>

				<!-- Step 2: Searchable Version Picker -->
				<div class="space-y-2">
					<span class="text-xs font-bold text-white/70 uppercase tracking-wider block">2. Versão do Minecraft</span>
					<FilterableVersionSelect
						versions={availableVersions}
						bind:value={newVersion}
						loading={versionsLoading}
						onChange={(ver) => {
							const ldrName = newLoader.charAt(0).toUpperCase() + newLoader.slice(1);
							newName = `Minecraft ${ver} (${ldrName})`;
						}}
					/>
				</div>

				<!-- Step 3: Instance Name, Icon & RAM Allocation -->
				<div class="space-y-4">
					<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
						<!-- Instance Name -->
						<div class="md:col-span-2 space-y-1.5">
							<label for="instance-name" class="block text-xs font-bold text-white/70 uppercase tracking-wider">{t("instances.name")}</label>
							<div class="flex items-center gap-3">
								<div class="h-11 w-11 rounded-2xl bg-[#18191c] border border-white/10 flex items-center justify-center shrink-0 p-1">
									<img src={getIconSrc(newIcon)} alt="Ícone da Instância" class="w-8 h-8 object-contain [image-rendering:pixelated]" />
								</div>
								<input
									id="instance-name"
									type="text"
									class="h-11 flex-1 rounded-2xl px-5 text-xs font-bold text-white bg-[#18191c] border border-white/10 focus:border-brand-500 outline-none transition-all"
									placeholder={t("instances.namePlaceholder")}
									bind:value={newName}
								/>
							</div>
						</div>

						<!-- Quick Icon Picker -->
						<div class="space-y-1.5">
							<span class="block text-xs font-bold text-white/70 uppercase tracking-wider">Ícone da Instância</span>
							<div class="flex items-center gap-1.5 bg-[#18191c] p-1.5 rounded-2xl border border-white/10">
								{#each iconPresets as ip}
									<button
										type="button"
										class="w-8 h-8 rounded-xl p-1 transition-all cursor-pointer flex items-center justify-center {newIcon === ip.id ? 'bg-brand-500/20 border border-brand-500 scale-105' : 'hover:bg-white/5 opacity-60 hover:opacity-100'}"
										onclick={() => newIcon = ip.id}
										title={ip.label}
									>
										<img src={ip.src} alt={ip.label} class="w-6 h-6 object-contain [image-rendering:pixelated]" />
									</button>
								{/each}
								<button
									type="button"
									class="h-8 px-2 rounded-xl text-[10px] font-bold text-brand-500 hover:bg-brand-500/10 border border-brand-500/30 transition-all cursor-pointer shrink-0 ml-auto"
									onclick={() => pickCustomIcon(false)}
									title="Carregar imagem do PC"
								>
									+ Imagem
								</button>
							</div>
						</div>
					</div>

					<div class="bg-[#18191c] border border-white/5 rounded-3xl p-4 space-y-2.5">
						<div class="flex items-center justify-between text-xs">
							<div class="flex items-center gap-2">
								<span class="font-bold text-white/80">Alocação de Memória RAM</span>
								<span class="text-[10px] text-white/40 font-mono bg-white/5 px-2 py-0.5 rounded-full border border-white/5">
									Detectado no PC: {Math.round(systemRamMb / 1024)} GB
								</span>
							</div>
							<span class="font-mono font-black text-brand-500 bg-brand-500/10 px-3 py-0.5 rounded-full border border-brand-500/20">
								{selectedRamGb} GB ({selectedRamGb * 1024} MB)
							</span>
						</div>

						<!-- Dynamic System RAM Presets -->
						<div class="flex items-center gap-2">
							{#each ramPresets as ram}
								{@const isRecommended = (systemRamMb >= 12288 && ram === 6) || (systemRamMb < 12288 && ram === 4)}
								<button
									type="button"
									class="flex-1 py-2 rounded-2xl text-xs font-black transition-all cursor-pointer relative {selectedRamGb === ram ? 'bg-brand-500 text-black shadow-md scale-[1.02]' : 'bg-[#222328] text-white/60 hover:text-white border border-white/5 hover:border-white/20'}"
									onclick={() => selectedRamGb = ram}
								>
									<span>{ram} GB</span>
									{#if isRecommended}
										<span class="absolute -top-2 left-1/2 -translate-x-1/2 text-[8px] font-extrabold uppercase px-1 rounded bg-emerald-500 text-black shadow-xs">
											Ideal
										</span>
									{/if}
								</button>
							{/each}
						</div>
						<p class="text-[10px] text-white/40">
							{#if selectedRamGb <= 2}
								Ideal para versões clássicas e hardware básico.
							{:else if selectedRamGb === 4}
								Recomendado para Minecraft moderno (1.20+) Vanilla ou com poucos mods.
							{:else if selectedRamGb === 6}
								Excelente para modpacks médios (Fabulously Optimized, shaders moderados).
							{:else if selectedRamGb === 8}
								Configuração para modpacks pesados (Better MC, Cobblemon, All The Mods).
							{:else}
								Alocação extrema para modpacks com centenas de mods e shaders ultra realistas.
							{/if}
						</p>
					</div>

					<!-- Luxmc Optimization Presets for New Instance -->
					<div class="bg-[#18191c] border border-white/5 rounded-3xl p-4 space-y-3">
						<div class="flex items-center justify-between">
							<div class="flex items-center gap-2">
								<Zap class="w-4 h-4 text-brand-500" />
								<span class="text-xs font-bold text-white/90">Otimizações Nativas Luxmc</span>
							</div>
							<span class="text-[9px] font-bold uppercase tracking-wider text-brand-500 bg-brand-500/10 px-2 py-0.5 rounded-full border border-brand-500/20">
								Auto Tuning
							</span>
						</div>

						<div class="space-y-2">
							<!-- Aikar Flags Toggle -->
							<label class="flex items-start gap-3 p-2.5 rounded-2xl bg-[#222328]/60 hover:bg-[#222328] border border-white/5 cursor-pointer transition-colors">
								<input type="checkbox" bind:checked={newAutoOptimize} class="mt-0.5 accent-brand-500 rounded" />
								<div class="text-xs space-y-0.5">
									<div class="font-bold text-white/90 flex items-center gap-1.5">
										<span>Flags de GC Inteligentes (Aikar G1GC)</span>
										<span class="text-[9px] font-bold text-emerald-400 bg-emerald-500/10 px-1.5 py-0.5 rounded border border-emerald-500/20">Recomendado</span>
									</div>
									<p class="text-[10px] text-white/50">
										Ajusta dinamicamente as regiões de heap e threads do garbage collector da JVM para eliminar microtravamentos.
									</p>
								</div>
							</label>

							<!-- Performance Modpack Opt-in (if mod loader chosen) -->
							{#if newLoader !== "vanilla"}
								<label class="flex items-start gap-3 p-2.5 rounded-2xl bg-[#222328]/60 hover:bg-[#222328] border border-white/5 cursor-pointer transition-colors">
									<input type="checkbox" bind:checked={newInstallPerfPack} class="mt-0.5 accent-brand-500 rounded" />
									<div class="text-xs space-y-0.5">
										<div class="font-bold text-white/90 flex items-center gap-1.5">
											<Sparkles class="w-3.5 h-3.5 text-amber-400" />
											<span>Pacote de Otimização Essencial</span>
										</div>
										<p class="text-[10px] text-white/50">
											Baixa automaticamente Sodium, Lithium e FerriteCore oficiais compatíveis com a versão escolhida.
										</p>
									</div>
								</label>
							{/if}

							<!-- Mesa Zink / Vulkan Toggle -->
							<label class="flex items-start gap-3 p-2.5 rounded-2xl bg-[#222328]/60 hover:bg-[#222328] border border-white/5 cursor-pointer transition-colors">
								<input type="checkbox" bind:checked={newUseVulkan} class="mt-0.5 accent-brand-500 rounded" />
								<div class="text-xs space-y-0.5">
									<div class="font-bold text-white/90 flex items-center gap-1.5">
										<span>Aceleração Gráfica Mesa Zink / Vulkan (Linux)</span>
										<span class="text-[9px] font-mono text-white/40 bg-white/5 px-1.5 py-0.5 rounded">Experimental</span>
									</div>
									<p class="text-[10px] text-white/50">
										Redireciona o pipeline OpenGL para o driver Vulkan nativo da sua GPU via Gallium Zink.
									</p>
								</div>
							</label>
						</div>
					</div>
				</div>

				<!-- Action Buttons -->
				<div class="flex gap-3 pt-2">
					<button 
						type="button"
						class="px-8 py-3 rounded-full bg-brand-500 hover:bg-[#ebd095] text-black font-black text-xs flex items-center gap-2 transition-all active:scale-95 shadow-lg cursor-pointer"
						onclick={createProfile}
						disabled={creating}
					>
						{#if creating}
							<div class="w-4 h-4 rounded-full border-2 border-black border-t-transparent animate-spin"></div>
							Criando Instância...
						{:else}
							<Check class="h-4 w-4 stroke-[3]" />
							Criar Instância Agora
						{/if}
					</button>

					<button 
						type="button"
						class="px-6 py-3 rounded-full bg-white/5 hover:bg-white/10 text-white/60 hover:text-white font-bold text-xs flex items-center gap-1.5 transition-all cursor-pointer"
						onclick={() => { showCreate = false; lastError = null; }}
					>
						<X class="h-4 w-4" />
						Cancelar
					</button>
				</div>
			{/if}

			{#if lastError}
				<div class="flex items-center justify-between rounded-full px-5 py-3 text-xs bg-red-500/10 border border-red-500/30 text-red-400 font-bold">
					<div class="flex items-center gap-2">
						<AlertTriangle class="h-4 w-4 shrink-0" />
						{lastError}
					</div>
					<button
						type="button"
						class="p-1 rounded hover:bg-white/10 cursor-pointer"
						onclick={() => (lastError = null)}
					>
						<X class="h-3.5 w-3.5" />
					</button>
				</div>
			{/if}
		</div>
	{/if}

	{#if showImport}
		<Card>
			<div class="flex flex-col gap-3">
				<p class="text-sm" style="color: rgb(var(--fg-muted));">{t("instances.importModpackDesc")}</p>
				<p class="truncate text-xs" style="color: rgb(var(--fg-subtle));">{importFile}</p>
				<div class="flex gap-3">
					<div class="flex-1">
						<label for="import-name" class="mb-1 block text-xs" style="color: rgb(var(--fg-subtle));">{t("instances.name")}</label>
						<Input id="import-name" bind:value={importName} placeholder={t("instances.modpackName")} />
					</div>
					<div class="w-36">
						<label for="import-loader" class="mb-1 block text-xs" style="color: rgb(var(--fg-subtle));">{t("instances.loader")}</label>
						<select
							id="import-loader"
							class="h-9 w-full rounded-md px-2 text-sm"
							style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg)); color: rgb(var(--fg));"
							bind:value={importLoader}
						>
							<option value="fabric">Fabric</option>
							<option value="forge">Forge</option>
							<option value="neoforge">NeoForge</option>
							<option value="quilt">Quilt</option>
						</select>
					</div>
				</div>
				<div class="flex gap-2">
					<Button variant="solid" onclick={importModpack} loading={importing}>
						<Import class="h-4 w-4" />
						{t("instances.importModpack")}
					</Button>
					<Button variant="secondary" onclick={() => { showImport = false; importFile = null; }}>
						<X class="h-4 w-4" />
						{t("common.cancel")}
					</Button>
				</div>
			</div>
		</Card>
	{/if}

	{#if showImportMrpack}
		<Card>
			<div class="flex flex-col gap-3">
				<p class="text-sm" style="color: rgb(var(--fg-muted));">{t("instances.importMrpackDesc")}</p>
				<p class="truncate text-xs" style="color: rgb(var(--fg-subtle));">{importMrpackFile}</p>
				<div class="flex gap-3">
					<div class="flex-1">
						<label for="import-mrpack-name" class="mb-1 block text-xs" style="color: rgb(var(--fg-subtle));">{t("instances.name")}</label>
						<Input id="import-mrpack-name" bind:value={importMrpackName} placeholder={t("instances.modpackName")} />
					</div>
				</div>
				<div class="flex gap-2">
					<Button variant="solid" onclick={importMrpack} loading={importingMrpack}>
						<Import class="h-4 w-4" />
						{t("instances.importMrpack")}
					</Button>
					<Button variant="secondary" onclick={() => { showImportMrpack = false; importMrpackFile = null; }}>
						<X class="h-4 w-4" />
						{t("common.cancel")}
					</Button>
				</div>
			</div>
		</Card>
	{/if}

	<div class="flex items-center gap-3">
		<div class="relative flex-1">
			<Search
				class="absolute left-3.5 top-1/2 h-4 w-4 -translate-y-1/2 text-white/40"
			/>
			<input
				bind:this={searchInput}
				class="h-10 w-full rounded-full pl-10 pr-4 text-xs font-bold outline-none placeholder:text-white/30 bg-[#18191c] border border-white/10 focus:border-brand-500 text-white transition-all"
				placeholder={t("instances.searchPlaceholder")}
				bind:value={searchQuery}
			/>
		</div>

		<div class="flex items-center gap-1.5 bg-[#18191c] border border-white/10 rounded-full px-3.5 h-10">
			<ArrowUpDown class="h-3.5 w-3.5 text-white/40" />
			<select
				class="bg-transparent text-xs font-bold text-white outline-none cursor-pointer pr-1"
				bind:value={sortBy}
			>
				<option value="name" class="bg-[#18191c]">{t("instances.sortName")}</option>
				<option value="version" class="bg-[#18191c]">{t("instances.sortVersion")}</option>
				<option value="date" class="bg-[#18191c]">{t("instances.sortDate")}</option>
				<option value="lastPlayed" class="bg-[#18191c]">{t("instances.sortLastPlayed")}</option>
			</select>
		</div>

		<div class="flex items-center gap-1.5 bg-[#18191c] border border-white/10 rounded-full px-3.5 h-10">
			<Group class="h-3.5 w-3.5 text-white/40" />
			<select
				class="bg-transparent text-xs font-bold text-white outline-none cursor-pointer pr-1"
				bind:value={groupFilter}
			>
				{#each groups as g}
					<option value={g} class="bg-[#18191c]">{g === "all" ? t("instances.groupAll") : g === "Modded" ? t("instances.groupModded") : g === "Vanilla" ? t("instances.groupVanilla") : g === "Servers" ? t("instances.groupServers") : g === "Favorites" ? t("instances.groupFavorites") : g}</option>
				{/each}
			</select>
		</div>

		<div class="flex items-center gap-1 p-1 bg-[#18191c] border border-white/10 rounded-full h-10">
			<button
				class="grid h-8 w-8 place-items-center rounded-full transition-all cursor-pointer {viewMode === 'grid' ? 'bg-white/10 text-brand-500 shadow-sm' : 'text-white/40 hover:text-white'}"
				onclick={() => (viewMode = "grid")}
				aria-label={t("instances.gridView")}
			>
				<LayoutGrid class="h-3.5 w-3.5" />
			</button>
			<button
				class="grid h-8 w-8 place-items-center rounded-full transition-all cursor-pointer {viewMode === 'list' ? 'bg-white/10 text-brand-500 shadow-sm' : 'text-white/40 hover:text-white'}"
				onclick={() => (viewMode = "list")}
				aria-label={t("instances.listView")}
			>
				<List class="h-3.5 w-3.5" />
			</button>
		</div>
	</div>

	{#if filteredInstances.length === 0}
		<Card>
			<div class="flex flex-col items-center gap-3 py-8 text-center">
				<div class="relative mb-2">
					<div class="mc-creeper-face h-16 w-16"></div>
				</div>
				<p class="text-sm" style="color: rgb(var(--fg-muted));">
					{#if searchQuery}
						{t("instances.noInstancesMatch", { query: searchQuery })}
					{:else}
						{t("instances.createFirst")}
					{/if}
				</p>
			</div>
		</Card>
	{:else if viewMode === "grid"}
		<div in:fade={{ duration: 150 }} class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
			{#each filteredInstances as p (p.id)}
				{@const isActive = profiles.activeId === p.id}
				{@const tagColor = instanceColors[p.id]}
				<div class="relative group">
					{#if selectionMode}
						<button
							class="absolute left-2 top-2 z-10 grid h-5 w-5 place-items-center rounded transition-colors"
							style="border: 1px solid {selectedIds.has(p.id) ? 'rgb(45, 212, 191)' : 'rgb(var(--border))'}; background: {selectedIds.has(p.id) ? 'rgba(45, 212, 191, 0.2)' : 'rgb(var(--bg-elevated))'};"
							onclick={(e) => { e.stopPropagation(); toggleSelect(p.id); }}
							role="checkbox"
							aria-checked={selectedIds.has(p.id)}
							aria-label={`${t("instances.select")}: ${p.name}`}
						>
							{#if selectedIds.has(p.id)}
								<Check class="h-3 w-3" style="color: rgb(45, 212, 191);" />
							{/if}
						</button>
					{/if}
					<Card interactive onclick={() => selectInstance(p.id)}>
						<!-- Top Minecraft Artwork Banner -->
						<div class="h-36 -mx-4 -mt-4 mb-3 rounded-t-2xl overflow-hidden relative bg-[#1c1d22]">
							<img 
								src={p.loader === 'fabric' ? '/modpack_fo.webp' : p.loader === 'forge' ? '/modpack_better_mc.webp' : p.loader === 'neoforge' ? '/modpack_cobblemon.webp' : '/vanilla_banner.png'} 
								alt="Minecraft Artwork" 
								class="w-full h-full object-cover opacity-85 group-hover:scale-105 transition-transform duration-500" 
							/>
							<div class="absolute inset-0 bg-gradient-to-t from-[#141518] via-transparent to-transparent"></div>
							
							<!-- Top-Right Loader Pill -->
							<div class="absolute top-2.5 right-2.5 bg-black/70 backdrop-blur-md text-[#caa97c] text-[10px] font-black uppercase px-2.5 py-1 rounded-full border border-white/10 shadow-lg flex items-center gap-1.5">
								<span class="w-1.5 h-1.5 rounded-full bg-[#caa97c]"></span>
								<span>{p.loader.toUpperCase()} · {p.mcVersion}</span>
							</div>

							<!-- Bottom-Left Instance Icon -->
							<div class="absolute bottom-2.5 left-3 h-10 w-10 rounded-xl bg-[#14151a] border-2 border-white/15 p-0.5 shadow-xl flex items-center justify-center overflow-hidden">
								<img src={getIconSrc(p.icon)} alt="Minecraft" class="w-full h-full object-cover rounded-lg [image-rendering:pixelated]" />
							</div>
						</div>

						{#if tagColor}
							<div class="absolute left-0 top-0 h-full w-1 rounded-l-xl" style="background: {colorOptions.find((c) => c.value === tagColor)?.color ?? 'rgb(45, 212, 191)'};"></div>
						{/if}
						<div class="flex items-start justify-between">
							<div class="min-w-0 flex-1">
								<div class="flex items-center gap-2">
									<button
										class="grid h-5 w-5 shrink-0 place-items-center rounded transition-colors cursor-pointer"
										style="color: {p.favorite ? 'rgb(250, 204, 21)' : 'rgb(var(--fg-subtle))'};"
										onclick={(e) => {
											e.stopPropagation();
											profiles.toggleFavorite(p.id);
											instanceSetFavorite(p.id, !p.favorite).catch(() => {});
										}}
										aria-label={t("servers.favorite")}
									>
										<Star class="h-3.5 w-3.5 {p.favorite ? 'fill-current' : ''}" />
									</button>
									<p class="truncate font-black text-white text-sm">{p.name}</p>
									{#if isActive}
										<span class="px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 text-[9px] font-black uppercase flex items-center gap-1">
											<span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span> Ativa
										</span>
									{:else}
										<span class="px-2 py-0.5 rounded-full bg-white/5 text-white/50 text-[9px] font-bold">
											Pronta
										</span>
									{/if}
								</div>
								<p class="mt-0.5 flex items-center gap-1.5 text-xs text-white/50">
									<span class="inline-block mc-diamond-shape" style="width: 6px; height: 6px;"></span>
									{p.mcVersion} · {p.loader.toUpperCase()}
									{#if p.loaderVersion}<span class="text-white/30">({p.loaderVersion})</span>{/if}
								</p>
							</div>
							<button
								class="grid h-7 w-7 shrink-0 place-items-center rounded-md transition-all duration-150 text-white/40 hover:text-red-400 hover:bg-red-500/10 cursor-pointer"
								onclick={(e) => {
									e.stopPropagation();
									confirmDeleteInstance = { id: p.id, name: p.name, gameDir: p.gameDir };
								}}
								aria-label={t("common.delete")}
								title="Excluir Instância"
							>
								<Trash2 class="h-3.5 w-3.5" />
							</button>
						</div>

						<div class="mt-2 flex flex-wrap gap-2 text-[11px]" style="color: rgb(var(--fg-subtle));">
							{#if p.lastPlayed}
								<span class="flex items-center gap-1">
									<Clock class="h-3 w-3" />
									{formatTimeAgo(p.lastPlayed)}
								</span>
							{/if}
							{#if p.modCount !== undefined && p.modCount > 0}
								<span class="flex items-center gap-1">
									<Box class="h-3 w-3" />
									{t("instances.modsCount", { count: p.modCount })}
								</span>
							{/if}
							{#if p.diskUsage}
								<span class="flex items-center gap-1">
									<HardDrive class="h-3 w-3" />
									{formatBytes(p.diskUsage)}
								</span>
							{/if}
							<span class="flex items-center gap-1">
								<Download class="h-3 w-3" />
								{t("instances.ramCount", { ram: (p.ramMb || 4096) / 1024 })}
							</span>
						</div>

						{#if p.notes}
							<p class="mt-1 text-[11px] italic line-clamp-2" style="color: rgb(var(--fg-subtle));">{p.notes}</p>
						{/if}

						<!-- Card Footer: Clean SKlauncher Layout -->
						<div class="mt-4 flex items-center justify-between border-t border-white/5 pt-3 relative">
							<!-- Left: Secondary Tools (Edit, Folder, More Options) -->
							<div class="flex items-center gap-1.5">
								<button
									type="button"
									class="h-8 w-8 rounded-xl flex items-center justify-center text-white/50 hover:text-white bg-white/[0.03] hover:bg-white/10 border border-white/5 transition-all cursor-pointer shadow-sm active:scale-95"
									onclick={(e) => { e.stopPropagation(); openEditInstance(p); }}
									aria-label={t("instances.edit")}
									title="Configurar Instância"
								>
									<Pencil class="h-3.5 w-3.5" />
								</button>
								<button
									type="button"
									class="h-8 w-8 rounded-xl flex items-center justify-center text-white/50 hover:text-white bg-white/[0.03] hover:bg-white/10 border border-white/5 transition-all cursor-pointer shadow-sm active:scale-95"
									onclick={(e) => { e.stopPropagation(); openFolder(p.id); }}
									aria-label={t("instances.openFolder")}
									title="Abrir Pasta de Arquivos"
								>
									<FolderOpen class="h-3.5 w-3.5" />
								</button>
								<div class="relative">
									<button
										type="button"
										class="h-8 w-8 rounded-xl flex items-center justify-center text-white/50 hover:text-white bg-white/[0.03] hover:bg-white/10 border border-white/5 transition-all cursor-pointer shadow-sm active:scale-95 {openCardMenuId === p.id ? 'bg-[#caa97c]/20 text-[#caa97c] border-[#caa97c]/40' : ''}"
										onclick={(e) => { e.stopPropagation(); openCardMenuId = openCardMenuId === p.id ? null : p.id; }}
										title="Mais Opções"
									>
										<MoreVertical class="h-3.5 w-3.5" />
									</button>

									{#if openCardMenuId === p.id}
										<!-- Backdrop click to dismiss -->
										<!-- svelte-ignore a11y_click_events_have_key_events -->
										<!-- svelte-ignore a11y_no_static_element_interactions -->
										<div class="fixed inset-0 z-40" onclick={(e) => { e.stopPropagation(); openCardMenuId = null; }}></div>

										<!-- Popover Dropdown Menu -->
										<!-- svelte-ignore a11y_click_events_have_key_events -->
										<!-- svelte-ignore a11y_no_static_element_interactions -->
										<div 
											class="absolute bottom-10 left-0 z-50 w-48 bg-[#181920] border border-white/10 rounded-2xl shadow-2xl p-1.5 space-y-0.5 text-xs font-semibold text-white/80"
											onclick={(e) => e.stopPropagation()}
										>
											<button 
												type="button" 
												class="w-full flex items-center gap-2 px-3 py-2 rounded-xl hover:bg-white/5 hover:text-white transition-colors cursor-pointer text-left" 
												onclick={() => { openCardMenuId = null; duplicateInstance(p.id); }}
											>
												<Copy class="w-3.5 h-3.5 text-[#caa97c]" />
												<span>Duplicar Instância</span>
											</button>
											<button 
												type="button" 
												class="w-full flex items-center gap-2 px-3 py-2 rounded-xl hover:bg-white/5 hover:text-white transition-colors cursor-pointer text-left" 
												onclick={() => { openCardMenuId = null; openScreenshots(p.id); }}
											>
												<Image class="w-3.5 h-3.5 text-sky-400" />
												<span>Capturas de Tela</span>
											</button>
											<button 
												type="button" 
												class="w-full flex items-center gap-2 px-3 py-2 rounded-xl hover:bg-white/5 hover:text-white transition-colors cursor-pointer text-left" 
												onclick={() => { openCardMenuId = null; openNotes(p.id); }}
											>
												<StickyNote class="w-3.5 h-3.5 text-amber-400" />
												<span>Anotações</span>
											</button>
											<button 
												type="button" 
												class="w-full flex items-center gap-2 px-3 py-2 rounded-xl hover:bg-white/5 hover:text-white transition-colors cursor-pointer text-left" 
												onclick={() => { openCardMenuId = null; checkHealth(p.id); }}
											>
												<HeartPulse class="w-3.5 h-3.5 text-emerald-400" />
												<span>Diagnóstico</span>
											</button>
											<div class="border-t border-white/5 my-1"></div>
											<button 
												type="button" 
												class="w-full flex items-center gap-2 px-3 py-2 rounded-xl hover:bg-red-500/15 text-red-400 hover:text-red-300 transition-colors cursor-pointer text-left" 
												onclick={() => { openCardMenuId = null; confirmDeleteInstance = { id: p.id, name: p.name, gameDir: p.gameDir }; }}
											>
												<Trash2 class="w-3.5 h-3.5" />
												<span>Excluir Instância</span>
											</button>
										</div>
									{/if}
								</div>
							</div>

							<!-- Right: Primary JOGAR Button (Hero champagne/gold pill) -->
							<button
								type="button"
								class="flex items-center gap-2 rounded-xl px-6 py-2.5 text-xs font-black text-black transition-all hover:brightness-105 active:scale-95 shadow-[0_4px_20px_rgba(202,169,124,0.35)] cursor-pointer bg-gradient-to-r from-[#caa97c] via-[#ddbe93] to-[#ebd095] disabled:opacity-60"
								onclick={(e) => { e.stopPropagation(); quickPlay(p); }}
								disabled={launchingInstanceId === p.id}
							>
								{#if launchingInstanceId === p.id}
									<Loader2 class="h-4 w-4 animate-spin text-black" />
									<span>Iniciando...</span>
								{:else}
									<Play class="h-4 w-4 fill-current stroke-[2.5]" />
									<span class="tracking-wide uppercase">JOGAR</span>
								{/if}
							</button>
						</div>
					</Card>
				</div>
			{/each}
		</div>
	{:else}
		<div in:fade={{ duration: 150 }} class="flex flex-col gap-1">
			<div class="grid grid-cols-[auto_1fr_8rem_6rem_5.5rem] gap-3 border-b px-4 py-2 text-xs font-medium" style="border-color: rgb(var(--border)); color: rgb(var(--fg-subtle));">
				<span></span>
				<span>{t("instances.sortName")}</span>
				<span>{t("instances.sortVersion")}</span>
				<span>{t("instances.loader")}</span>
				<span class="text-right">{t("instances.actions")}</span>
			</div>
			{#each filteredInstances as p (p.id)}
				{@const isActive = profiles.activeId === p.id}
				{@const tagColor = instanceColors[p.id]}
				<div
					class="grid grid-cols-[auto_1fr_8rem_6rem_5.5rem] items-center gap-3 rounded-md px-4 py-2.5 text-left transition-colors cursor-pointer"
					style="{isActive ? 'background: rgba(45, 212, 191, 0.05);' : ''}{tagColor ? ' border-left: 3px solid ' + (colorOptions.find((c) => c.value === tagColor)?.color ?? 'rgb(45, 212, 191)') + ';' : ''}"
					onclick={() => selectInstance(p.id)}
					onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectInstance(p.id); }}
					role="button"
					tabindex="0"
				>
					{#if selectionMode}
						<button
							class="grid h-5 w-5 shrink-0 place-items-center rounded transition-colors"
							style="border: 1px solid {selectedIds.has(p.id) ? 'rgb(45, 212, 191)' : 'rgb(var(--border))'}; background: {selectedIds.has(p.id) ? 'rgba(45, 212, 191, 0.2)' : 'rgb(var(--bg-elevated))'};"
							onclick={(e) => { e.stopPropagation(); toggleSelect(p.id); }}
							role="checkbox"
							aria-checked={selectedIds.has(p.id)}
							aria-label={`${t("instances.select")}: ${p.name}`}
						>
							{#if selectedIds.has(p.id)}
								<Check class="h-3 w-3" style="color: rgb(45, 212, 191);" />
							{/if}
						</button>
					{:else}
						<button
							class="grid h-5 w-5 shrink-0 place-items-center rounded transition-colors"
							style="color: {p.favorite ? 'rgb(250, 204, 21)' : 'rgb(var(--fg-subtle))'};"
							onclick={(e) => {
								e.stopPropagation();
								profiles.toggleFavorite(p.id);
								instanceSetFavorite(p.id, !p.favorite).catch(() => {});
							}}
							aria-label={t("servers.favorite")}
						>
							<Star class="h-3.5 w-3.5 {p.favorite ? 'fill-current' : ''}" />
						</button>
					{/if}
					<div class="flex min-w-0 items-center gap-3">
						<img src={getIconSrc(p.icon)} alt="Minecraft" class="w-5 h-5 rounded object-contain [image-rendering:pixelated] drop-shadow-xs shrink-0 bg-black/40 border border-white/10" />
						<span class="truncate text-sm font-medium">{p.name}</span>
						{#if isActive}
							<span class="px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 text-[9px] font-black uppercase flex items-center gap-1">
								<span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span> Ativa
							</span>
						{/if}
					</div>
					<span class="flex items-center gap-1 truncate font-mono text-xs" style="color: rgb(var(--fg-muted));">
						<span class="inline-block mc-diamond-shape" style="width: 5px; height: 5px;"></span>
						{p.mcVersion}
					</span>
					<span class="truncate text-xs font-bold uppercase" style="color: rgb(var(--fg-muted));">{p.loader}</span>
					<div class="flex items-center justify-end gap-1.5">
						<button
							class="flex items-center gap-1.5 rounded-xl px-4 py-1.5 text-xs font-black text-black transition-all hover:brightness-105 active:scale-95 shadow-[0_2px_10px_rgba(202,169,124,0.3)] cursor-pointer bg-gradient-to-r from-[#caa97c] via-[#ddbe93] to-[#ebd095] disabled:opacity-75"
							onclick={(e) => { e.stopPropagation(); quickPlay(p); }}
							disabled={launchingInstanceId === p.id}
						>
							{#if launchingInstanceId === p.id}
								<Loader2 class="h-3 w-3 animate-spin text-black" />
								<span>Iniciando...</span>
							{:else}
								<Play class="h-3 w-3 fill-current stroke-[2.5]" />
								<span class="tracking-wide uppercase">JOGAR</span>
							{/if}
						</button>
						<button
							class="h-7 w-7 rounded-full flex items-center justify-center hover:bg-white/10 text-white/50 hover:text-white transition-all cursor-pointer"
							onclick={(e) => { e.stopPropagation(); openEditInstance(p); }}
							aria-label={t("instances.edit")}
							title={t("instances.editTooltip")}
						>
							<Pencil class="h-3.5 w-3.5" />
						</button>
						<button
							class="h-7 w-7 rounded-full flex items-center justify-center hover:bg-white/10 text-white/50 hover:text-white transition-all cursor-pointer"
							onclick={(e) => { e.stopPropagation(); openFolder(p.id); }}
							aria-label={t("instances.openFolder")}
						>
							<FolderOpen class="h-3.5 w-3.5" />
						</button>
						<button
							class="h-7 w-7 rounded-full flex items-center justify-center hover:bg-red-500/20 text-white/50 hover:text-red-400 transition-all cursor-pointer"
							onclick={(e) => { e.stopPropagation(); confirmDeleteInstance = { id: p.id, name: p.name, gameDir: p.gameDir }; }}
							aria-label={t("common.delete")}
						>
							<Trash2 class="h-3.5 w-3.5" />
						</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}

	<!-- Sleek Bottom Bar for Instances Library -->
	<div class="mt-4 mb-6 rounded-3xl bg-[#141518] border border-white/5 p-4 flex flex-col sm:flex-row items-center justify-between gap-4 shadow-xl">
		<div class="flex items-center gap-4">
			<div class="w-10 h-10 rounded-full bg-brand-500/10 border border-brand-500/20 flex items-center justify-center shrink-0">
				<Boxes class="w-5 h-5 text-brand-500" />
			</div>
			<div>
				<div class="text-xs font-black text-white flex items-center gap-2">
					<span>Biblioteca Luxmc</span>
					<span class="text-[9px] font-black uppercase px-2.5 py-0.5 rounded-full bg-white/5 text-white/60 border border-white/10">
						{filteredInstances.length} {filteredInstances.length === 1 ? 'Instância instalada' : 'Instâncias instaladas'}
					</span>
				</div>
				<p class="text-[11px] text-white/40 mt-0.5">
					Gerencie suas versões com carregamento rápido e perfis isolados.
				</p>
			</div>
		</div>

		<div class="flex items-center gap-2.5 w-full sm:w-auto justify-end">
			<button
				type="button"
				class="px-5 py-2.5 rounded-full bg-white/5 hover:bg-white/10 border border-white/10 text-white/80 hover:text-white text-xs font-bold transition-all cursor-pointer flex items-center gap-2"
				onclick={() => {
					if (profiles.active) openFolder(profiles.active.id);
					else toast("Nenhuma instância ativa selecionada", "info");
				}}
			>
				<FolderOpen class="w-4 h-4 text-white/50" />
				Abrir Pasta
			</button>

			<button
				type="button"
				class="px-6 py-2.5 rounded-full bg-brand-500 hover:bg-[#ebd095] text-black font-black text-xs transition-all active:scale-95 shadow-md cursor-pointer flex items-center gap-2"
				onclick={() => { showCreate = true; }}
			>
				<Plus class="w-4 h-4 stroke-[3]" />
				Criar Instância
			</button>
		</div>
	</div>

	{#if healthCheckId}
		<div
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
			transition:fade={{ duration: 150 }}
			onclick={(e) => { if (e.target === e.currentTarget) closeHealthCheck(); }}
			onkeydown={(e) => { if (e.key === "Escape") closeHealthCheck(); }}
			role="dialog"
			aria-modal="true"
			aria-label={t("health.title")}
			tabindex="-1"
		>
			<Card class="w-full max-w-md">
				<div class="mb-4 flex items-center justify-between">
					<div class="flex items-center gap-2">
						<HeartPulse class="h-5 w-5" style="color: rgb(var(--fg-muted));" />
						<h2 class="text-lg font-medium">{t("health.title")}</h2>
					</div>
					<button
						class="grid h-8 w-8 place-items-center rounded-md transition-colors"
						style="color: rgb(var(--fg-subtle));"
						onclick={closeHealthCheck}
						aria-label={t("common.close")}
					>
						<X class="h-4 w-4" />
					</button>
				</div>
				{#if healthChecking}
					<div class="flex items-center justify-center py-8">
						<div class="h-6 w-6 animate-spin rounded-full border-2 border-t-transparent" style="border-color: rgb(45, 212, 191); border-top-color: transparent;"></div>
						<span class="ml-2 text-sm" style="color: rgb(var(--fg-muted));">{t("health.checking")}</span>
					</div>
				{:else if healthResult}
					<div class="flex flex-col gap-3">
						<div class="flex items-center gap-2 rounded-lg px-3 py-2" style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg));">
							{#if healthResult.clientJar}
								<Check class="h-4 w-4" style="color: rgb(74, 222, 128);" />
							{:else}
								<AlertTriangle class="h-4 w-4" style="color: rgb(248, 113, 113);" />
							{/if}
							<span class="text-sm">{t("health.clientJar")}</span>
							<span class="ml-auto text-xs" style="color: {healthResult.clientJar ? 'rgb(74, 222, 128)' : 'rgb(248, 113, 113)'};">
								{healthResult.clientJar ? t("health.present") : t("health.missing")}
							</span>
						</div>
						<div class="flex items-center gap-2 rounded-lg px-3 py-2" style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg));">
							{#if healthResult.natives}
								<Check class="h-4 w-4" style="color: rgb(74, 222, 128);" />
							{:else}
								<AlertTriangle class="h-4 w-4" style="color: rgb(250, 204, 21);" />
							{/if}
							<span class="text-sm">{t("health.natives")}</span>
							<span class="ml-auto text-xs" style="color: {healthResult.natives ? 'rgb(74, 222, 128)' : 'rgb(250, 204, 21)'};">
								{healthResult.natives ? t("health.present") : t("health.missing")}
							</span>
						</div>
						<div class="flex items-center gap-2 rounded-lg px-3 py-2" style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg));">
							{#if healthResult.modsOk}
								<Check class="h-4 w-4" style="color: rgb(74, 222, 128);" />
							{:else}
								<AlertTriangle class="h-4 w-4" style="color: rgb(250, 204, 21);" />
							{/if}
							<span class="text-sm">{t("health.mods")}</span>
							<span class="ml-auto text-xs" style="color: {healthResult.modsOk ? 'rgb(74, 222, 128)' : 'rgb(250, 204, 21)'};">
								{healthResult.modsOk ? t("health.present") : t("health.missing")}
							</span>
						</div>
						{#if healthResult.issues.length > 0}
							<div class="rounded-lg px-3 py-2" style="border: 1px solid rgba(239, 68, 68, 0.3); background: rgba(239, 68, 68, 0.1);">
								<p class="text-xs font-medium" style="color: rgb(248, 113, 113);">{t("health.issuesFound")}</p>
								{#each healthResult.issues as issue}
									<p class="mt-1 text-xs" style="color: rgb(252, 165, 165);">- {issue}</p>
								{/each}
							</div>
						{:else}
							<div class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm" style="border: 1px solid rgba(34, 197, 94, 0.3); background: rgba(34, 197, 94, 0.1); color: rgb(74, 222, 128);">
								<Check class="h-4 w-4" />
								{t("health.healthy")}
							</div>
						{/if}
					</div>
				{/if}
			</Card>
		</div>
	{/if}

	{#if fileBrowserId}
		<div
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
			transition:fade={{ duration: 150 }}
			onclick={(e) => { if (e.target === e.currentTarget) closeFileBrowser(); }}
			onkeydown={(e) => { if (e.key === "Escape") closeFileBrowser(); }}
			role="dialog"
			aria-modal="true"
			aria-label={t("files.title")}
			tabindex="-1"
		>
			<Card class="flex max-h-[80vh] w-full max-w-lg flex-col">
				<div class="mb-4 flex items-center justify-between">
					<div class="flex items-center gap-2">
						<FolderTree class="h-5 w-5" style="color: rgb(var(--fg-muted));" />
						<h2 class="text-lg font-medium">{t("files.title")}</h2>
					</div>
					<button
						class="grid h-8 w-8 place-items-center rounded-md transition-colors"
						style="color: rgb(var(--fg-subtle));"
						onclick={closeFileBrowser}
						aria-label={t("common.close")}
					>
						<X class="h-4 w-4" />
					</button>
				</div>
				{#if fileTreePath}
					<button
						class="mb-2 text-left text-xs"
						style="color: rgb(45, 212, 191);"
						onclick={() => navigateFileTree(null as any)}
					>
						.. / {fileTreePath}
					</button>
				{/if}
				<div class="flex-1 overflow-y-auto">
					{#if fileTreeLoading}
						<Skeleton lines={6} />
					{:else if fileTree.length === 0}
						<div class="flex flex-col items-center gap-2 py-8 text-center">
							<FolderTree class="h-8 w-8" style="color: rgb(var(--fg-subtle));" />
							<p class="text-sm" style="color: rgb(var(--fg-muted));">{t("files.empty")}</p>
						</div>
					{:else}
						<div class="flex flex-col gap-0.5">
							{#each fileTree as entry}
								<button
									class="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm transition-colors"
									onclick={() => {
										if (entry.isDir) {
											const relPath = fileTreePath ? `${fileTreePath}/${entry.name}` : entry.name;
											navigateFileTree(relPath);
										}
									}}
								>
									{#if entry.isDir}
										<FolderOpen class="h-4 w-4 shrink-0" style="color: rgb(45, 212, 191);" />
									{:else}
										<span class="h-4 w-4 shrink-0"></span>
									{/if}
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
		<div
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
			transition:fade={{ duration: 150 }}
			onclick={(e) => { if (e.target === e.currentTarget) closeScreenshots(); }}
			onkeydown={(e) => { if (e.key === "Escape") closeScreenshots(); }}
			role="dialog"
			aria-modal="true"
			aria-label={t("screenshots.title")}
			tabindex="-1"
		>
			<Card class="flex max-h-[80vh] w-full max-w-2xl flex-col">
				<div class="mb-4 flex items-center justify-between">
					<div class="flex items-center gap-2">
						<Image class="h-5 w-5" style="color: rgb(var(--fg-muted));" />
						<h2 class="text-lg font-medium">{t("screenshots.title")}</h2>
					</div>
					<button
						class="grid h-8 w-8 place-items-center rounded-md transition-colors"
						style="color: rgb(var(--fg-subtle));"
						onclick={closeScreenshots}
						aria-label={t("common.close")}
					>
						<X class="h-4 w-4" />
					</button>
				</div>
				<div class="flex-1 overflow-y-auto">
					{#if screenshotsLoading}
						<Skeleton lines={6} />
					{:else if screenshots.length === 0}
						<div class="flex flex-col items-center gap-2 py-8 text-center">
							<Image class="h-8 w-8" style="color: rgb(var(--fg-subtle));" />
							<p class="text-sm" style="color: rgb(var(--fg-muted));">{t("screenshots.noScreenshots")}</p>
						</div>
					{:else}
						<div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
							{#each screenshots as shot}
								<div class="overflow-hidden rounded-xl border border-white/10 bg-black/40 shadow-sm group">
									<div class="aspect-video overflow-hidden flex items-center justify-center bg-black/60">
										<img 
											src={convertFileSrc(shot.path)} 
											alt={shot.name} 
											class="w-full h-full object-cover group-hover:scale-105 transition-transform" 
											loading="lazy" 
										/>
									</div>
									<div class="px-2.5 py-1.5 flex items-center justify-between">
										<div class="min-w-0 flex-1">
											<p class="truncate text-[11px] font-bold text-white/90">{shot.name}</p>
											<p class="text-[10px] text-white/40">
												{new Date(shot.modified).toLocaleDateString()}
											</p>
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
		<div
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
			transition:fade={{ duration: 150 }}
			onclick={(e) => { if (e.target === e.currentTarget) closeNotes(); }}
			onkeydown={(e) => { if (e.key === "Escape") closeNotes(); }}
			role="dialog"
			aria-modal="true"
			aria-label={t("instances.notes")}
			tabindex="-1"
		>
			<Card class="w-full max-w-md">
				<div class="mb-4 flex items-center justify-between">
					<div class="flex items-center gap-2">
						<StickyNote class="h-5 w-5" style="color: rgb(var(--fg-muted));" />
						<h2 class="text-lg font-medium">{t("instances.notes")}</h2>
					</div>
					<button
						class="grid h-8 w-8 place-items-center rounded-md transition-colors"
						style="color: rgb(var(--fg-subtle));"
						onclick={closeNotes}
						aria-label={t("common.close")}
					>
						<X class="h-4 w-4" />
					</button>
				</div>
				<textarea
					class="min-h-[200px] w-full resize-y rounded-md p-3 text-sm outline-none"
					style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg)); color: rgb(var(--fg));"
					placeholder={t("instances.notesPlaceholder")}
					aria-label={t("instances.notes")}
					bind:value={notesText}
				></textarea>
				<div class="mt-3 flex justify-end gap-2">
					<Button variant="secondary" onclick={closeNotes}>{t("common.cancel")}</Button>
					<Button variant="solid" onclick={saveNotes}>{t("common.save")}</Button>
				</div>
			</Card>
		</div>
	{/if}

	<Modal
		isOpen={editingInstance !== null}
		onClose={() => (editingInstance = null)}
		title={t("instances.edit")}
		maxWidth="max-w-xl"
	>
		{#if editingInstance}
			<div class="flex flex-col gap-4 text-xs select-none">
				<!-- Instance Name and Icon -->
				<div class="space-y-1.5">
					<label for="edit-instance-name" class="block text-xs font-bold text-white/70 uppercase tracking-wider">{t("instances.name")}</label>
					<div class="flex items-center gap-3">
						<div class="h-11 w-11 rounded-2xl bg-[#18191c] border border-white/10 flex items-center justify-center shrink-0 p-1">
							<img src={getIconSrc(editIcon)} alt="Ícone da Instância" class="w-8 h-8 object-contain [image-rendering:pixelated]" />
						</div>
						<Input id="edit-instance-name" bind:value={editName} placeholder={t("instances.namePlaceholder")} />
					</div>
				</div>

				<!-- Icon Selector -->
				<div class="space-y-1.5">
					<span class="block text-xs font-bold text-white/70 uppercase tracking-wider">Ícone da Instância</span>
					<div class="flex items-center gap-1.5 bg-[#18191c] p-1.5 rounded-2xl border border-white/10">
						{#each iconPresets as ip}
							<button
								type="button"
								class="w-8 h-8 rounded-xl p-1 transition-all cursor-pointer flex items-center justify-center {editIcon === ip.id ? 'bg-brand-500/20 border border-brand-500 scale-105' : 'hover:bg-white/5 opacity-60 hover:opacity-100'}"
								onclick={() => editIcon = ip.id}
								title={ip.label}
							>
								<img src={ip.src} alt={ip.label} class="w-6 h-6 object-contain [image-rendering:pixelated]" />
							</button>
						{/each}
						<button
							type="button"
							class="h-8 px-2.5 rounded-xl text-[10px] font-bold text-brand-500 hover:bg-brand-500/10 border border-brand-500/30 transition-all cursor-pointer shrink-0 ml-auto"
							onclick={() => pickCustomIcon(true)}
							title="Carregar imagem do PC"
						>
							+ Carregar Arquivo
						</button>
					</div>
				</div>

				<!-- Minecraft Version -->
				<div class="space-y-1.5">
					<span class="block text-xs font-bold text-white/70 uppercase tracking-wider">{t("instances.version")}</span>
					<FilterableVersionSelect
						versions={availableVersions}
						bind:value={editVersion}
						loading={versionsLoading}
					/>
				</div>

				<!-- RAM Allocation with System RAM Presets -->
				<div class="bg-[#18191c] border border-white/5 rounded-2xl p-3.5 space-y-2">
					<div class="flex items-center justify-between text-xs">
						<div class="flex items-center gap-2">
							<span class="font-bold text-white/80">Alocação de Memória RAM</span>
							<span class="text-[10px] text-white/40 font-mono bg-white/5 px-2 py-0.5 rounded-full border border-white/5">
								Detectado no PC: {Math.round(systemRamMb / 1024)} GB
							</span>
						</div>
						<span class="font-mono font-black text-brand-500 bg-brand-500/10 px-2.5 py-0.5 rounded-full border border-brand-500/20">
							{editRamGb} GB ({editRamGb * 1024} MB)
						</span>
					</div>

					<div class="flex items-center gap-1.5">
						{#each ramPresets as ram}
							{@const isRecommended = (systemRamMb >= 12288 && ram === 6) || (systemRamMb < 12288 && ram === 4)}
							<button
								type="button"
								class="flex-1 py-1.5 rounded-xl text-xs font-black transition-all cursor-pointer relative {editRamGb === ram ? 'bg-brand-500 text-black shadow-md' : 'bg-[#222328] text-white/60 hover:text-white border border-white/5 hover:border-white/20'}"
								onclick={() => editRamGb = ram}
							>
								<span>{ram} GB</span>
								{#if isRecommended}
									<span class="absolute -top-2 left-1/2 -translate-x-1/2 text-[7px] font-extrabold uppercase px-1 rounded bg-emerald-500 text-black">
										Ideal
									</span>
								{/if}
							</button>
						{/each}
					</div>
				</div>

				<!-- JVM Arguments -->
				<div class="space-y-1.5">
					<label for="edit-jvm-args" class="block text-xs font-bold text-white/70 uppercase tracking-wider">Argumentos JVM Customizados</label>
					<input
						id="edit-jvm-args"
						type="text"
						bind:value={editJvmArgs}
						placeholder="-XX:+UseG1GC -XX:+AlwaysPreTouch"
						class="w-full bg-[#18191c] border border-white/10 focus:border-brand-500 rounded-xl px-4 py-2.5 text-xs text-white font-mono outline-none transition-colors"
					/>
					<p class="text-[10px] text-white/40">Parâmetros extras passados diretamente para a máquina virtual Java.</p>
				</div>

				<!-- Footer Buttons -->
				<div class="flex justify-end gap-2 border-t pt-4" style="border-color: rgb(var(--border));">
					<Button variant="secondary" onclick={() => (editingInstance = null)}>
						{t("common.cancel")}
					</Button>
					<Button variant="solid" onclick={saveEditInstance} loading={editSaving} disabled={!editName.trim() || !editVersion}>
						<Check class="h-4 w-4" />
						{t("instances.saveChanges")}
					</Button>
				</div>
			</div>
		{/if}
	</Modal>

	<!-- Confirmation Modal for Deletion -->
	{#if confirmDeleteInstance}
		<div
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-md p-4"
			transition:fade={{ duration: 150 }}
		>
			<div class="w-full max-w-md bg-[#141518] border border-red-500/30 rounded-3xl p-6 shadow-2xl space-y-4 select-none">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-red-500/15 border border-red-500/30 flex items-center justify-center text-red-400 shrink-0">
						<Trash2 class="w-5 h-5" />
					</div>
					<div>
						<h3 class="text-sm font-black text-white">Excluir Instância</h3>
						<p class="text-[11px] text-white/50">Esta ação não poderá ser desfeita</p>
					</div>
				</div>

				<div class="bg-[#18191c] p-3.5 rounded-2xl border border-white/5 text-xs text-white/70 space-y-1.5">
					<p>Tem certeza que deseja apagar a instância <strong class="text-white">"{confirmDeleteInstance.name}"</strong>?</p>
					<p class="text-[10px] text-white/40 font-mono break-all">Pasta: {confirmDeleteInstance.gameDir}</p>
					<p class="text-[11px] text-red-400/90 font-medium">Todos os mundos, saves, mods e arquivos salvos serão removidos permanentemente do disco.</p>
				</div>

				<div class="flex justify-end gap-2.5 pt-2">
					<button
						type="button"
						class="px-5 py-2.5 rounded-full text-xs font-bold text-white/60 hover:text-white hover:bg-white/10 transition-all cursor-pointer"
						onclick={() => (confirmDeleteInstance = null)}
					>
						Cancelar
					</button>
					<button
						type="button"
						class="px-5 py-2.5 rounded-full bg-red-500 hover:bg-red-600 text-white font-black text-xs transition-all active:scale-95 shadow-md cursor-pointer flex items-center gap-2"
						disabled={deleting}
						onclick={async () => {
							if (!confirmDeleteInstance) return;
							deleting = true;
							const id = confirmDeleteInstance.id;
							const name = confirmDeleteInstance.name;
							confirmDeleteInstance = null;
							await deleteInstance(id);
							deleting = false;
							toast(`Instância "${name}" excluída com sucesso!`, "success");
						}}
					>
						<Trash2 class="w-3.5 h-3.5" />
						Excluir Definitivamente
					</button>
				</div>
			</div>
		</div>
	{/if}
</div>
