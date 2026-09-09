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
	} from "lucide-svelte";
	import FilterableVersionSelect from "$lib/components/ui/FilterableVersionSelect.svelte";
	import Modal from "$lib/components/ui/Modal.svelte";
	import { open } from "@tauri-apps/plugin-dialog";
	import { convertFileSrc } from "@tauri-apps/api/core";
	import { profiles } from "$lib/stores/profiles.svelte";
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
	let newName = $state("My Instance");
	let newVersion = $state("1.21.4");
	let newLoader = $state("vanilla");
	let selectedRamGb = $state(4);
	let creating = $state(false);
	let lastError = $state<string | null>(null);
	let createSuccess = $state(false);

	let editingInstance = $state<{ id: string; name: string; mcVersion: string; loader: string } | null>(null);
	let editName = $state("");
	let editVersion = $state("");
	let editSaving = $state(false);

	function openEditInstance(p: typeof profiles.list[0]) {
		editingInstance = { id: p.id, name: p.name, mcVersion: p.mcVersion, loader: p.loader };
		editName = p.name;
		editVersion = p.mcVersion;
	}

	async function saveEditInstance() {
		if (!editingInstance || !editName.trim()) return;
		editSaving = true;
		try {
			await api.invoke("profiles_update", {
				input: {
					id: editingInstance.id,
					name: editName.trim(),
					mcVersion: editVersion
				}
			});
			profiles.update(editingInstance.id, {
				name: editName.trim(),
				mcVersion: editVersion
			});
			editingInstance = null;
		} catch (e) {
			lastError = "Failed to update instance: " + String(e);
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
		window.addEventListener("keydown", handleInstanceKeydown);
		return () => window.removeEventListener("keydown", handleInstanceKeydown);
	});

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
				},
			});
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
				group: newLoader === "vanilla" ? "Vanilla" : "Modded",
				ramMb: selectedRamGb * 1024,
			});
			profiles.activeId = p.id;
			createSuccess = true;
			toast(t("instances.createdSuccess"), "success");
			setTimeout(() => {
				showCreate = false;
				createSuccess = false;
				newName = "My Instance";
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

				<!-- Step 3: Instance Name & RAM Allocation Selector -->
				<div class="space-y-4">
					<div>
						<label for="instance-name" class="mb-1.5 block text-xs font-bold text-white/70 uppercase tracking-wider">{t("instances.name")}</label>
						<input
							id="instance-name"
							type="text"
							class="h-11 w-full rounded-full px-5 text-xs font-bold text-white bg-[#18191c] border border-white/10 focus:border-brand-500 outline-none transition-all"
							placeholder={t("instances.namePlaceholder")}
							bind:value={newName}
						/>
					</div>

					<div class="bg-[#18191c] border border-white/5 rounded-3xl p-4 space-y-2.5">
						<div class="flex items-center justify-between text-xs">
							<span class="font-bold text-white/80">Alocação de Memória RAM</span>
							<span class="font-mono font-black text-brand-500 bg-brand-500/10 px-3 py-0.5 rounded-full border border-brand-500/20">
								{selectedRamGb} GB ({selectedRamGb * 1024} MB)
							</span>
						</div>
						<div class="flex items-center gap-2">
							{#each [2, 4, 6, 8, 12, 16] as ram}
								<button
									type="button"
									class="flex-1 py-2 rounded-full text-xs font-black transition-all cursor-pointer {selectedRamGb === ram ? 'bg-brand-500 text-black shadow-md scale-[1.02]' : 'bg-[#222328] text-white/60 hover:text-white border border-white/5 hover:border-white/20'}"
									onclick={() => selectedRamGb = ram}
								>
									{ram} GB
								</button>
							{/each}
						</div>
						<p class="text-[10px] text-white/40">
							{#if selectedRamGb === 2}
								Ideal para versões antigas Vanilla e hardware mais básico.
							{:else if selectedRamGb === 4}
								Recomendado para Minecraft moderno (1.20+) com ótimo equilíbrio.
							{:else if selectedRamGb === 6}
								Ideal para shaders moderados e modpacks médios.
							{:else}
								Máxima performance para modpacks pesados (Cobblemon, Better MC, Shaders Ultra).
							{/if}
						</p>
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
						<div class="h-32 -mx-4 -mt-4 mb-3 rounded-t-xl overflow-hidden relative bg-[#1c1d22]">
							<img 
								src={p.loader === 'fabric' ? '/modpack_fo.webp' : p.loader === 'forge' ? '/modpack_better_mc.webp' : p.loader === 'neoforge' ? '/modpack_cobblemon.webp' : '/vanilla_banner.png'} 
								alt="Minecraft Artwork" 
								class="w-full h-full object-cover opacity-85 group-hover:scale-105 transition-transform duration-500" 
							/>
							<div class="absolute inset-0 bg-gradient-to-t from-[#141518] via-transparent to-transparent"></div>
							<div class="absolute bottom-2.5 left-3 flex items-center gap-2">
								<img src="/grass_block.png" alt="Minecraft" class="w-5 h-5 object-contain [image-rendering:pixelated] drop-shadow-md" />
								<span class="bg-black/80 backdrop-blur-md text-brand-500 text-[9px] font-black uppercase px-2 py-0.5 rounded-md border border-brand-500/30 shadow-md">
									{p.loader.toUpperCase()} · MC {p.mcVersion}
								</span>
							</div>
						</div>

						{#if tagColor}
							<div class="absolute left-0 top-0 h-full w-1 rounded-l-xl" style="background: {colorOptions.find((c) => c.value === tagColor)?.color ?? 'rgb(45, 212, 191)'};"></div>
						{/if}
						<div class="flex items-start justify-between">
							<div class="min-w-0 flex-1">
								<div class="flex items-center gap-2">
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
									<p class="truncate font-medium">{p.name}</p>
									{#if isActive}
										<Badge>{t("common.active")}</Badge>
									{/if}
								</div>
							<p class="mt-0.5 flex items-center gap-1.5 text-xs" style="color: rgb(var(--fg-subtle));">
								<span class="inline-block mc-diamond-shape" style="width: 6px; height: 6px;"></span>
								{p.mcVersion} · {p.loader}
								{#if p.loaderVersion}{p.loaderVersion}{/if}
							</p>
							</div>
							<button
								class="grid h-7 w-7 shrink-0 place-items-center rounded-md transition-all duration-150 hover:scale-110 active:scale-90"
								style="color: rgb(var(--fg-subtle));"
								onclick={(e) => {
									e.stopPropagation();
									deleteInstance(p.id);
								}}
								aria-label={t("common.delete")}
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
							{#if p.ramMb}
								<span class="flex items-center gap-1">
									<Download class="h-3 w-3" />
									{t("instances.ramCount", { ram: p.ramMb / 1024 })}
								</span>
							{/if}
						</div>

						{#if p.notes}
							<p class="mt-1 text-[11px] italic line-clamp-2" style="color: rgb(var(--fg-subtle));">{p.notes}</p>
						{/if}

						<!-- Card Footer Actions (Sleek single row with pill buttons) -->
						<div class="mt-4 flex items-center justify-between border-t border-white/5 pt-3">
							<div class="flex items-center gap-1">
								<button
									class="h-8 w-8 rounded-full flex items-center justify-center text-white/50 hover:text-white hover:bg-white/10 transition-all cursor-pointer"
									onclick={(e) => { e.stopPropagation(); openEditInstance(p); }}
									aria-label={t("instances.edit")}
									title={t("instances.editTooltip")}
								>
									<Pencil class="h-3.5 w-3.5" />
								</button>
								<button
									class="h-8 w-8 rounded-full flex items-center justify-center text-white/50 hover:text-white hover:bg-white/10 transition-all cursor-pointer"
									onclick={(e) => { e.stopPropagation(); duplicateInstance(p.id); }}
									aria-label={t("instances.duplicate")}
									title="Duplicar Instância"
								>
									<Copy class="h-3.5 w-3.5" />
								</button>
								<button
									class="h-8 w-8 rounded-full flex items-center justify-center text-white/50 hover:text-white hover:bg-white/10 transition-all cursor-pointer"
									onclick={(e) => { e.stopPropagation(); openFolder(p.id); }}
									aria-label={t("instances.openFolder")}
									title="Abrir Pasta"
								>
									<FolderOpen class="h-3.5 w-3.5" />
								</button>
								<button
									class="h-8 w-8 rounded-full flex items-center justify-center text-white/50 hover:text-white hover:bg-white/10 transition-all cursor-pointer"
									onclick={(e) => { e.stopPropagation(); openScreenshots(p.id); }}
									aria-label={t("nav.screenshots")}
									title="Capturas de Tela"
								>
									<Image class="h-3.5 w-3.5" />
								</button>
								<button
									class="h-8 w-8 rounded-full flex items-center justify-center text-white/50 hover:text-white hover:bg-white/10 transition-all cursor-pointer"
									onclick={(e) => { e.stopPropagation(); openNotes(p.id); }}
									aria-label={t("instances.notes")}
									title="Anotações"
								>
									<StickyNote class="h-3.5 w-3.5" />
								</button>
								<div class="relative">
									<button
										class="h-8 w-8 rounded-full flex items-center justify-center text-white/50 hover:text-white hover:bg-white/10 transition-all cursor-pointer"
										style="color: {tagColor ? colorOptions.find((c) => c.value === tagColor)?.color ?? 'rgba(255,255,255,0.7)' : ''};"
										onclick={(e) => { e.stopPropagation(); colorPickerId = colorPickerId === p.id ? null : p.id; }}
										aria-label={t("instances.colorTag")}
										title="Cor da Tag"
									>
										<Paintbrush class="h-3.5 w-3.5" />
									</button>
									{#if colorPickerId === p.id}
										<div class="absolute bottom-full left-0 z-30 mb-1.5 flex gap-1 rounded-full p-1.5 bg-[#1e1f24] border border-white/10 shadow-xl">
											{#each colorOptions as c}
												<button
													class="h-5 w-5 rounded-full border-2 transition-transform hover:scale-110"
													style="background: {c.color}; border-color: {tagColor === c.value ? 'white' : 'transparent'};"
													onclick={(e) => { e.stopPropagation(); saveColor(p.id, c.value); colorPickerId = null; }}
													aria-label={t("instances.setThemeColor", { color: c.value })}
												></button>
											{/each}
											{#if tagColor}
												<button
													class="h-5 w-5 rounded-full border border-white/20 bg-white/5 hover:bg-white/15 flex items-center justify-center transition-transform hover:scale-110"
													onclick={(e) => { e.stopPropagation(); removeColor(p.id); colorPickerId = null; }}
													aria-label={t("instances.removeColor")}
												>
													<X class="h-3 w-3 text-white/60" />
												</button>
											{/if}
										</div>
									{/if}
								</div>
							</div>

							<button
								class="flex items-center gap-1.5 rounded-full px-4 py-2 text-xs font-black text-black transition-all hover:brightness-110 active:scale-95 shadow-md cursor-pointer"
								style="background-color: var(--accent-color, #e2b86b);"
								onclick={(e) => { e.stopPropagation(); selectInstance(p.id); goto('/'); }}
							>
								<Play class="h-3.5 w-3.5 fill-current" /> {t("instances.play")}
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
						{#if tagColor}
							<span class="h-2.5 w-2.5 shrink-0 rounded-full" style="background: {colorOptions.find((c) => c.value === tagColor)?.color ?? 'rgb(45, 212, 191)'};"></span>
						{:else if isActive}
							<span class="h-2 w-2 shrink-0 rounded-full" style="background: rgb(45, 212, 191);"></span>
						{:else}
							<span class="h-2 w-2 shrink-0 rounded-full" style="background: rgb(var(--border));"></span>
						{/if}
						<span class="truncate text-sm font-medium">{p.name}</span>
					</div>
					<span class="flex items-center gap-1 truncate font-mono text-xs" style="color: rgb(var(--fg-muted));">
					<span class="inline-block mc-diamond-shape" style="width: 5px; height: 5px;"></span>
					{p.mcVersion}
				</span>
					<span class="truncate text-xs" style="color: rgb(var(--fg-muted));">{p.loader}</span>
					<div class="flex items-center justify-end gap-1.5">
						<button
							class="flex items-center gap-1 rounded-full px-3 py-1.5 text-xs font-black text-black transition-all hover:brightness-110 active:scale-95 shadow-sm"
							style="background-color: var(--accent-color, #e2b86b);"
							onclick={(e) => { e.stopPropagation(); selectInstance(p.id); goto('/'); }}
						>
							<Play class="h-3 w-3 fill-current" /> {t("instances.play")}
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
							onclick={(e) => { e.stopPropagation(); duplicateInstance(p.id); }}
							aria-label={t("instances.duplicate")}
						>
							<Copy class="h-3.5 w-3.5" />
						</button>
						<button
							class="h-7 w-7 rounded-full flex items-center justify-center hover:bg-white/10 text-white/50 hover:text-white transition-all cursor-pointer"
							onclick={(e) => { e.stopPropagation(); openFolder(p.id); }}
							aria-label={t("instances.openFolder")}
						>
							<FolderOpen class="h-3.5 w-3.5" />
						</button>
						<button
							class="h-7 w-7 rounded-full flex items-center justify-center hover:bg-white/10 text-white/50 hover:text-white transition-all cursor-pointer"
							onclick={(e) => { e.stopPropagation(); openScreenshots(p.id); }}
							aria-label={t("nav.screenshots")}
						>
							<Image class="h-3.5 w-3.5" />
						</button>
						<button
							class="h-7 w-7 rounded-full flex items-center justify-center hover:bg-white/10 text-white/50 hover:text-white transition-all cursor-pointer"
							onclick={(e) => { e.stopPropagation(); openNotes(p.id); }}
							aria-label={t("instances.notes")}
						>
							<StickyNote class="h-3.5 w-3.5" />
						</button>
						<div class="relative">
							<button
								class="h-7 w-7 rounded-full flex items-center justify-center hover:bg-white/10 text-white/50 hover:text-white transition-all cursor-pointer"
								style="color: {tagColor ? colorOptions.find((c) => c.value === tagColor)?.color ?? 'rgb(var(--fg-subtle))' : 'rgb(var(--fg-subtle))'};"
								onclick={(e) => { e.stopPropagation(); colorPickerId = colorPickerId === p.id ? null : p.id; }}
								aria-label={t("instances.colorTag")}
							>
								<Paintbrush class="h-3.5 w-3.5" />
							</button>
							{#if colorPickerId === p.id}
								<div class="absolute bottom-full right-0 z-30 mb-1 flex gap-1 rounded-full p-1.5 bg-[#1e1f24] border border-white/10 shadow-xl">
									{#each colorOptions as c}
										<button
											class="h-5 w-5 rounded-full border-2 transition-transform hover:scale-110"
											style="background: {c.color}; border-color: {tagColor === c.value ? 'white' : 'transparent'};"
											onclick={(e) => { e.stopPropagation(); saveColor(p.id, c.value); colorPickerId = null; }}
											aria-label={t("instances.setThemeColor", { color: c.value })}
										></button>
									{/each}
									{#if tagColor}
										<button
											class="h-5 w-5 rounded-full border border-white/20 bg-white/5 hover:bg-white/15 flex items-center justify-center transition-transform hover:scale-110"
											onclick={(e) => { e.stopPropagation(); removeColor(p.id); colorPickerId = null; }}
											aria-label={t("instances.removeColor")}
										>
											<X class="h-3 w-3 text-white/60" />
										</button>
									{/if}
								</div>
							{/if}
						</div>
						<button
							class="h-7 w-7 rounded-full flex items-center justify-center hover:bg-red-500/20 text-white/50 hover:text-red-400 transition-all cursor-pointer"
							onclick={(e) => { e.stopPropagation(); deleteInstance(p.id); }}
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
		maxWidth="max-w-lg"
	>
		{#if editingInstance}
			<div class="flex flex-col gap-4">
				<div>
					<label for="edit-instance-name" class="mb-1.5 block text-xs font-medium" style="color: rgb(var(--fg-muted));">{t("instances.name")}</label>
					<Input id="edit-instance-name" bind:value={editName} placeholder={t("instances.namePlaceholder")} />
				</div>

				<div>
					<label for="edit-instance-version" class="mb-1.5 block text-xs font-medium" style="color: rgb(var(--fg-muted));">{t("instances.version")}</label>
					<FilterableVersionSelect
						versions={availableVersions}
						bind:value={editVersion}
						loading={versionsLoading}
					/>
				</div>

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
</div>
