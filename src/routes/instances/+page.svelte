<script lang="ts">
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
		FolderTree,
		StickyNote,
		X,
		SquareCheck
	} from "lucide-svelte";
	import { open } from "@tauri-apps/plugin-dialog";
	import { convertFileSrc } from "@tauri-apps/api/core";
	import { profiles, type Profile } from "$lib/stores/profiles.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import {
		api,
		versionsList,
		instancesDuplicate,
		instancesOpenFolder,
		instancesScreenshots,
		instanceImportModpack,
		instanceImportMrpack,
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
	} from "$lib/api";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { toast } from "$lib/stores/toasts.svelte";

	const { t } = useTranslation();

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
	let lastError = $state<string | null>(null);

	let systemRamMb = $state(8192);
	let availableVersions = $state<Array<{ id: string; versionType: string; releaseTime: string }>>([]);
	let versionsLoading = $state(false);

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
			.filter((p) => !searchQuery || p.name.toLowerCase().includes(searchQuery.toLowerCase()))
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
		profiles.activeId = id;
		goto("/instances/" + id);
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
			if (specs?.totalRamMb) systemRamMb = specs.totalRamMb;
		} catch {}
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
			try { localStorage.setItem("luxmc_cached_versions", JSON.stringify(response)); } catch {}
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
		name: string; version: string; loader: string; icon: string;
		ramGb: number; autoOptimize: boolean; useVulkan: boolean; installPerfPack: boolean;
	}) {
		const p = await api.invoke<{
			id: string; name: string; icon: string; mcVersion: string;
			loader: string; loaderVersion: string | null; gameDir: string;
			createdAt: string; updatedAt: string;
		}>("profiles_create", {
			input: {
				name: input.name, mcVersion: input.version, loader: input.loader,
				icon: input.icon, ramMb: input.ramGb * 1024,
				autoOptimize: input.autoOptimize, useVulkan: input.useVulkan,
			},
		});

		if (input.installPerfPack && input.loader !== "vanilla") {
			try { await optimizerInstallPerfPack(p.id); } catch (err) { console.warn("Could not install perf pack:", err); }
		}

		profiles.add({
			id: p.id, name: p.name, icon: p.icon || input.icon,
			mcVersion: p.mcVersion,
			loader: p.loader as "vanilla" | "fabric" | "forge" | "neoforge" | "quilt",
			loaderVersion: p.loaderVersion ?? undefined,
			gameDir: p.gameDir,
			createdAt: new Date(p.createdAt).getTime(),
			updatedAt: new Date(p.updatedAt).getTime(),
			group: input.loader === "vanilla" ? "Vanilla" : "Modded",
			ramMb: input.ramGb * 1024,
			autoOptimize: input.autoOptimize,
			useVulkan: input.useVulkan,
		});
		profiles.activeId = p.id;
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
				versionId: verId, accountId: accountId || "",
				profileId: p.id, enableVulkan: isVulkan
			});
			gamingStats.onGameStart();
			appState.isGameRunning = true;
			appState.activeGameDetails = { name: p.name, version: verId, loader: p.loader };
			discordSetActivity({
				inGame: true,
				details: `Jogando ${p.name}`,
				state: `Minecraft ${verId} · ${p.loader ? p.loader.toUpperCase() : "Vanilla"}`,
				largeText: `Minecraft ${verId}`,
				largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
				smallImage: p.loader === "fabric" ? "fabric" : (p.loader === "forge" ? "curse" : "grass"),
				smallText: `Luxmc · ${p.loader || "Vanilla"}`,
				startTime: Math.floor(Date.now() / 1000)
			}).catch(() => {});
			profiles.setLastPlayed(p.id);
			toast(`🎮 Minecraft ${verId} (${p.name}) iniciado! (PID: ${res.pid})`, "success");
		} catch (e) {
			toast("Falha ao iniciar jogo: " + String(e), "error");
		} finally {
			setTimeout(() => { launchingInstanceId = null; }, 2500);
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
			toast("Instância atualizada com sucesso!", "success");
			editingInstance = null;
		} catch (e) {
			lastError = "Falha ao atualizar instância: " + String(e);
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
		} catch (e) {
			toast(t("instances.failedDelete", { error: String(e) }), "error");
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

	async function navigateFileTree(subPath: string) {
		if (!fileBrowserId) return;
		fileTreeLoading = true;
		fileTreePath = subPath;
		try { fileTree = await instanceFileTree(fileBrowserId, subPath); } catch (e) {
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
			if (!importName) { importName = (file.split("/").pop() ?? "Imported Modpack").replace(/\.zip$/i, ""); }
			showImport = true;
		}
	}

	async function importModpack() {
		if (!importFile || !importName.trim()) return;
		importing = true;
		try {
			const p = await instanceImportModpack(importFile, importName.trim(), importVersion, importLoader);
			profiles.add({
				id: p.id, name: p.name, icon: "", mcVersion: importVersion,
				loader: importLoader as "vanilla" | "fabric" | "forge" | "neoforge" | "quilt",
				gameDir: "", createdAt: Date.now(), updatedAt: Date.now(),
			});
			profiles.activeId = p.id;
			showImport = false; importFile = null; importName = "";
			toast(t("instances.createdSuccess"), "success");
		} catch (e) { toast(t("instances.failedImportModpack", { error: String(e) }), "error"); }
		finally { importing = false; }
	}

	async function pickMrpackFile() {
		const file = await open({ filters: [{ name: "Modrinth Modpack", extensions: ["mrpack"] }] });
		if (typeof file === "string") {
			importMrpackFile = file;
			if (!importMrpackName) { importMrpackName = (file.split("/").pop() ?? "Imported Modpack").replace(/\.mrpack$/i, ""); }
			showImportMrpack = true;
		}
	}

	async function importMrpack() {
		if (!importMrpackFile || !importMrpackName.trim()) return;
		importingMrpack = true;
		try {
			const p = await instanceImportMrpack(importMrpackFile, importMrpackName.trim());
			profiles.add({
				id: p.id, name: p.name, icon: "", mcVersion: p.mcVersion,
				loader: p.loader as "vanilla" | "fabric" | "forge" | "neoforge" | "quilt",
				gameDir: p.gameDir, createdAt: Date.now(), updatedAt: Date.now(),
			});
			profiles.activeId = p.id;
			showImportMrpack = false; importMrpackFile = null; importMrpackName = "";
			toast(t("instances.createdSuccess"), "success");
		} catch (e) { toast(t("instances.failedImportMrpack", { error: String(e) }), "error"); }
		finally { importingMrpack = false; }
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
					onclick={() => { showCreate = !showCreate; lastError = null; }}
				>
					<Plus class="h-4 w-4 stroke-[3]" />
					Nova Instância
				</button>
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
						<select id="import-loader" class="h-9 w-full rounded-md px-2 text-sm" style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg)); color: rgb(var(--fg));" bind:value={importLoader}>
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
						<Import class="h-4 w-4" /> {t("instances.importMrpack")}
					</Button>
					<Button variant="secondary" onclick={() => { showImportMrpack = false; importMrpackFile = null; }}>
						<X class="h-4 w-4" /> {t("common.cancel")}
					</Button>
				</div>
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
		{searchQuery}
		{selectionMode}
		{selectedIds}
		{instanceColors}
		{launchingInstanceId}
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

	<!-- Bottom Bar -->
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
				<p class="text-[11px] text-white/40 mt-0.5">Gerencie suas versões com carregamento rápido e perfis isolados.</p>
			</div>
		</div>
		<div class="flex items-center gap-2.5 w-full sm:w-auto justify-end">
			<button type="button" class="px-5 py-2.5 rounded-full bg-white/5 hover:bg-white/10 border border-white/10 text-white/80 hover:text-white text-xs font-bold transition-all cursor-pointer flex items-center gap-2"
				onclick={() => { if (profiles.active) openFolder(profiles.active.id); else toast("Nenhuma instância ativa selecionada", "info"); }}
			>
				<FolderOpen class="w-4 h-4 text-white/50" /> Abrir Pasta
			</button>
			<button type="button" class="px-6 py-2.5 rounded-full bg-brand-500 hover:bg-[#ebd095] text-black font-black text-xs transition-all active:scale-95 shadow-md cursor-pointer flex items-center gap-2"
				onclick={() => showCreate = true}
			>
				<Plus class="w-4 w-4 stroke-[3]" /> Criar Instância
			</button>
		</div>
	</div>

	<!-- Health Check Modal -->
	{#if healthCheckId}
		<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" transition:fade={{ duration: 150 }}
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
					<button class="grid h-8 w-8 place-items-center rounded-md transition-colors" style="color: rgb(var(--fg-subtle));" onclick={closeHealthCheck} aria-label={t("common.close")}>
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
							{#if healthResult.clientJar}<Check class="h-4 w-4" style="color: rgb(74, 222, 128);" />{:else}<HeartPulse class="h-4 w-4" style="color: rgb(248, 113, 113);" />{/if}
							<span class="text-sm">{t("health.clientJar")}</span>
							<span class="ml-auto text-xs" style="color: {healthResult.clientJar ? 'rgb(74, 222, 128)' : 'rgb(248, 113, 113)'};">
								{healthResult.clientJar ? t("health.present") : t("health.missing")}
							</span>
						</div>
						<div class="flex items-center gap-2 rounded-lg px-3 py-2" style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg));">
							{#if healthResult.natives}<Check class="h-4 w-4" style="color: rgb(74, 222, 128);" />{:else}<HeartPulse class="h-4 w-4" style="color: rgb(250, 204, 21);" />{/if}
							<span class="text-sm">{t("health.natives")}</span>
							<span class="ml-auto text-xs" style="color: {healthResult.natives ? 'rgb(74, 222, 128)' : 'rgb(250, 204, 21)'};">
								{healthResult.natives ? t("health.present") : t("health.missing")}
							</span>
						</div>
						<div class="flex items-center gap-2 rounded-lg px-3 py-2" style="border: 1px solid rgb(var(--border)); background: rgb(var(--bg));">
							{#if healthResult.modsOk}<Check class="h-4 w-4" style="color: rgb(74, 222, 128);" />{:else}<HeartPulse class="h-4 w-4" style="color: rgb(250, 204, 21);" />{/if}
							<span class="text-sm">{t("health.mods")}</span>
							<span class="ml-auto text-xs" style="color: {healthResult.modsOk ? 'rgb(74, 222, 128)' : 'rgb(250, 204, 21)'};">
								{healthResult.modsOk ? t("health.present") : t("health.missing")}
							</span>
						</div>
						{#if healthResult.issues.length > 0}
							<div class="rounded-lg px-3 py-2" style="border: 1px solid rgba(239, 68, 68, 0.3); background: rgba(239, 68, 68, 0.1);">
								<p class="text-xs font-medium" style="color: rgb(248, 113, 113);">{t("health.issuesFound")}</p>
								{#each healthResult.issues as issue}<p class="mt-1 text-xs" style="color: rgb(252, 165, 165);">- {issue}</p>{/each}
							</div>
						{:else}
							<div class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm" style="border: 1px solid rgba(34, 197, 94, 0.3); background: rgba(34, 197, 94, 0.1); color: rgb(74, 222, 128);">
								<Check class="h-4 w-4" /> {t("health.healthy")}
							</div>
						{/if}
					</div>
				{/if}
			</Card>
		</div>
	{/if}

	<!-- File Browser Modal -->
	{#if fileBrowserId}
		<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" transition:fade={{ duration: 150 }}
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
					<button class="grid h-8 w-8 place-items-center rounded-md transition-colors" style="color: rgb(var(--fg-subtle));" onclick={closeFileBrowser} aria-label={t("common.close")}>
						<X class="h-4 w-4" />
					</button>
				</div>
				{#if fileTreePath}
					<button class="mb-2 text-left text-xs" style="color: rgb(45, 212, 191);" onclick={() => navigateFileTree(null as any)}>
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
							{#each fileTree as entry}
								<button class="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm transition-colors"
									onclick={() => { if (entry.isDir) { navigateFileTree(fileTreePath ? `${fileTreePath}/${entry.name}` : entry.name); } }}
								>
									{#if entry.isDir}<FolderOpen class="h-4 w-4 shrink-0" style="color: rgb(45, 212, 191);" />{:else}<span class="h-4 w-4 shrink-0"></span>{/if}
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

	<!-- Screenshots Modal -->
	{#if screenshotsId}
		<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" transition:fade={{ duration: 150 }}
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
					<button class="grid h-8 w-8 place-items-center rounded-md transition-colors" style="color: rgb(var(--fg-subtle));" onclick={closeScreenshots} aria-label={t("common.close")}>
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
							{#each screenshots as shot}
								<div class="overflow-hidden rounded-xl border border-white/10 bg-black/40 shadow-sm group">
									<div class="aspect-video overflow-hidden flex items-center justify-center bg-black/60">
										<img src={convertFileSrc(shot.path)} alt={shot.name} class="w-full h-full object-cover group-hover:scale-105 transition-transform" loading="lazy" />
									</div>
									<div class="px-2.5 py-1.5 flex items-center justify-between">
										<div class="min-w-0 flex-1">
											<p class="truncate text-[11px] font-bold text-white/90">{shot.name}</p>
											<p class="text-[10px] text-white/40">{new Date(shot.modified).toLocaleDateString()}</p>
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

	<!-- Notes Modal -->
	{#if notesId}
		<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" transition:fade={{ duration: 150 }}
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
					<button class="grid h-8 w-8 place-items-center rounded-md transition-colors" style="color: rgb(var(--fg-subtle));" onclick={closeNotes} aria-label={t("common.close")}>
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

	<!-- Edit Instance Modal -->
	<Modal isOpen={editingInstance !== null} onClose={() => (editingInstance = null)} title={t("instances.edit")} maxWidth="max-w-xl">
		{#if editingInstance}
			<div class="flex flex-col gap-4 text-xs select-none">
				<div class="space-y-1.5">
					<label for="edit-instance-name" class="block text-xs font-bold text-white/70 uppercase tracking-wider">{t("instances.name")}</label>
					<div class="flex items-center gap-3">
						<div class="h-11 w-11 rounded-2xl bg-[#18191c] border border-white/10 flex items-center justify-center shrink-0 p-1">
							<img src={getIconSrc(editIcon)} alt="Ícone" class="w-8 h-8 object-contain [image-rendering:pixelated]" />
						</div>
						<Input id="edit-instance-name" bind:value={editName} placeholder={t("instances.namePlaceholder")} />
					</div>
				</div>

				<div class="space-y-1.5">
					<span class="block text-xs font-bold text-white/70 uppercase tracking-wider">Ícone da Instância</span>
					<div class="flex items-center gap-1.5 bg-[#18191c] p-1.5 rounded-2xl border border-white/10">
						{#each [{ id: "grass_block", label: "Grama", src: "/grass_block.png" }, { id: "modpack_fo", label: "FO", src: "/modpack_fo_icon.png" }, { id: "modpack_better_mc", label: "BMC", src: "/modpack_bmc_icon.webp" }, { id: "modpack_cobblemon", label: "Cobblemon", src: "/modpack_cobblemon_icon.png" }, { id: "logo", label: "Logo", src: "/logo.png" }, { id: "grass_head", label: "Steve", src: "/grass_head.png" }] as ip}
							<button type="button" class="w-8 h-8 rounded-xl p-1 transition-all cursor-pointer flex items-center justify-center {editIcon === ip.id ? 'bg-brand-500/20 border border-brand-500 scale-105' : 'hover:bg-white/5 opacity-60 hover:opacity-100'}"
								onclick={() => editIcon = ip.id} title={ip.label}
							>
								<img src={ip.src} alt={ip.label} class="w-6 h-6 object-contain [image-rendering:pixelated]" />
							</button>
						{/each}
					</div>
				</div>

				<div class="space-y-1.5">
					<span class="block text-xs font-bold text-white/70 uppercase tracking-wider">{t("instances.version")}</span>
					<FilterableVersionSelect versions={availableVersions} bind:value={editVersion} loading={versionsLoading} />
				</div>

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
						{#each [2, 4, 6, 8, 12, 16] as ram}
							{@const isRecommended = (systemRamMb >= 12288 && ram === 6) || (systemRamMb < 12288 && ram === 4)}
							<button type="button" class="flex-1 py-1.5 rounded-xl text-xs font-black transition-all cursor-pointer relative {editRamGb === ram ? 'bg-brand-500 text-black shadow-md' : 'bg-[#222328] text-white/60 hover:text-white border border-white/5 hover:border-white/20'}"
								onclick={() => editRamGb = ram}
							>
								<span>{ram} GB</span>
								{#if isRecommended}
									<span class="absolute -top-2 left-1/2 -translate-x-1/2 text-[7px] font-extrabold uppercase px-1 rounded bg-emerald-500 text-black">Ideal</span>
								{/if}
							</button>
						{/each}
					</div>
				</div>

				<div class="space-y-1.5">
					<label for="edit-jvm-args" class="block text-xs font-bold text-white/70 uppercase tracking-wider">Argumentos JVM Customizados</label>
					<input id="edit-jvm-args" type="text" bind:value={editJvmArgs} placeholder="-XX:+UseG1GC -XX:+AlwaysPreTouch"
						class="w-full bg-[#18191c] border border-white/10 focus:border-brand-500 rounded-xl px-4 py-2.5 text-xs text-white font-mono outline-none transition-colors"
					/>
					<p class="text-[10px] text-white/40">Parâmetros extras passados diretamente para a máquina virtual Java.</p>
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

	<!-- Delete Confirmation Modal -->
	{#if confirmDeleteInstance}
		<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-md p-4" transition:fade={{ duration: 150 }}>
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
					<button type="button" class="px-5 py-2.5 rounded-full text-xs font-bold text-white/60 hover:text-white hover:bg-white/10 transition-all cursor-pointer"
						onclick={() => (confirmDeleteInstance = null)}
					>Cancelar</button>
					<button type="button" class="px-5 py-2.5 rounded-full bg-red-500 hover:bg-red-600 text-white font-black text-xs transition-all active:scale-95 shadow-md cursor-pointer flex items-center gap-2"
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
						<Trash2 class="w-3.5 h-3.5" /> Excluir Definitivamente
					</button>
				</div>
			</div>
		</div>
	{/if}
</div>
