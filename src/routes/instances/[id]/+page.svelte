<script lang="ts">
	import { page } from "$app/stores";
	import { fade } from "svelte/transition";
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
		Gauge
	} from "lucide-svelte";
	import RightSidebar from "$lib/components/layout/RightSidebar.svelte";
	import VirtualList from "$lib/components/ui/VirtualList.svelte";
	import { profiles, type Profile } from "$lib/stores/profiles.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { open, save } from "@tauri-apps/plugin-dialog";
	import { convertFileSrc } from "@tauri-apps/api/core";
	import { 
		launchGame, 
		versionsCheckInstalled, 
		versionsDownload,
		instanceFileTree, 
		instancesScreenshots, 
		instancesOpenFolder, 
		screenshotDelete,
		authDevLogin,
		instanceWorldsList,
		instanceWorldDelete,
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
		instanceExportZip,
		instanceBackupSaves,
		instanceRestoreSaves,
		jvmArgsValidate,
		getSystemSpecs,
		optimizerGetFlags,
		optimizerGetPerfPack,
		optimizerInstallPerfPack,
		optimizerDetectGpu,
		modsResolveNames,
		type GpuInfo,
		type PerformancePackInfo,
		type JvmValidationResult,
		type FileTreeEntry,
		type WorldDetail,
		type HostLinkInfo
	} from "$lib/api";

	const instanceId = $derived($page.params.id ?? "");
	const activeProfile = $derived(profiles.list.find(p => p.id === instanceId) || profiles.active);

	let mainTab = $state<"conteudo" | "mundos" | "galeria" | "ficheiros">("conteudo");
	let subTab = $state<"mods" | "resourcepacks" | "shaders" | "datapacks">("mods");
	let searchQuery = $state("");

	let showInstanceSettingsModal = $state(false);
	let activeInstanceSection = $state<"geral" | "instalacao" | "otimizacao" | "janela" | "controlos" | "java" | "hooks">("geral");
	let instanceNameInput = $state("Latest Release");
	let instanceRamMb = $state(4096);
	let instanceJvmArgs = $state("");
	let instanceLoaderType = $state<string>("vanilla");
	let instanceLoaderVersion = $state<string>("0.15.11");
	let instanceWindowWidth = $state(1280);
	let instanceWindowHeight = $state(720);
	let instanceStartFullscreen = $state(false);
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
			instanceRamMb = activeProfile.ramMb || 4096;
			instanceJvmArgs = activeProfile.jvmArgs || "";
			instanceAutoOptimize = activeProfile.autoOptimize !== false;
			instanceEnableVulkanOpt = activeProfile.useVulkan === true;
			instanceLoaderType = activeProfile.loader || "vanilla";
			instanceLoaderVersion = activeProfile.loaderVersion || "";
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
			activeProfile.name = instanceNameInput;
			activeProfile.ramMb = instanceRamMb;
			activeProfile.jvmArgs = instanceJvmArgs;
			activeProfile.autoOptimize = instanceAutoOptimize;
			activeProfile.useVulkan = instanceEnableVulkanOpt;
			(activeProfile as any).loader = instanceLoaderType;
			(activeProfile as any).loaderVersion = instanceLoaderVersion;

			try {
				await profilesUpdate({
					id: activeProfile.id,
					name: instanceNameInput,
					ramMb: instanceRamMb,
					jvmArgs: instanceJvmArgs,
					autoOptimize: instanceAutoOptimize,
					useVulkan: instanceEnableVulkanOpt,
					loader: instanceLoaderType,
					loaderVersion: instanceLoaderVersion || null,
				});
				profiles.update(activeProfile.id, {
					name: instanceNameInput,
					ramMb: instanceRamMb,
					jvmArgs: instanceJvmArgs,
					autoOptimize: instanceAutoOptimize,
					useVulkan: instanceEnableVulkanOpt,
					loader: instanceLoaderType as Profile["loader"],
					loaderVersion: instanceLoaderVersion,
				});
				toast("Configurações salvas com sucesso!", "success");
			} catch (e) {
				toast("Erro ao salvar no banco de dados: " + String(e), "error");
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

	
	// Host World State
	let showHostModal = $state(false);
	let hostLinkInfo = $state<HostLinkInfo | null>(null);
	let customHostPort = $state(25565);
	let isCopiedHostLink = $state(false);

	async function openHostWorldModal() {
		try {
			hostLinkInfo = await p2pGetHostLink(customHostPort);
			showHostModal = true;
		} catch (e) {
			toast("Erro ao gerar link de host: " + String(e), "error");
		}
	}

	async function refreshHostLink() {
		try {
			hostLinkInfo = await p2pGetHostLink(customHostPort);
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
	
	// Launch & Install state
	let isLaunching = $state(false);
	let isInstalled = $state(true);
	let launchStatusText = $state("");
	let downloadProgressPercent = $state(0);

	// Real Data from File System & Backend
	let worldsList = $state<WorldDetail[]>([]);
	let screenshotsList = $state<Array<{ name: string; path: string; modified: string; dataUrl?: string | null }>>([]);
	let previewScreenshot = $state<{ name: string; path: string; dataUrl?: string | null } | null>(null);
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

	const currentPacksList = $derived(
		subTab === "resourcepacks" ? resourcePacks : subTab === "shaders" ? shaderPacks : dataPacks
	);

	const filteredMods = $derived(
		searchQuery
			? instanceMods.filter(m => m.name.toLowerCase().includes(searchQuery.toLowerCase()))
			: instanceMods
	);

	onMount(async () => {
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
		} catch (e) {
			console.error(e);
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
			toast(`Mundo "${folderName}" excluído com sucesso!`, "success");
			worldsList = worldsList.filter(w => w.folderName !== folderName);
		} catch (e) {
			toast("Erro ao excluir mundo: " + String(e), "error");
		}
	}

	async function handlePlay() {
		if (isLaunching) return;
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
					expiresAt: devAcc.expiresAt
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
			const isVulkan = typeof window !== "undefined" ? localStorage.getItem("luxmc_enable_vulkan") !== "false" : true;
			const result = await launchGame({
				versionId: verId,
				accountId: userUuid || "",
				profileId: targetProfileId,
				enableVulkan: isVulkan
			});

			gamingStats.onGameStart();
			appState.isGameRunning = true;
			appState.activeGameDetails = { name: activeProfile?.name || "Minecraft", version: verId, loader: activeProfile?.loader || "vanilla" };

			if (activeProfile) {
				profilesUpdate({
					id: activeProfile.id,
					lastPlayed: new Date().toISOString(),
					launchCount: (activeProfile.launchCount || 0) + 1,
				}).catch(() => {});
			}

			discordSetActivity({
				inGame: true,
				details: `Jogando ${activeProfile?.name || "Minecraft"}`,
				state: `Minecraft ${verId} · ${activeProfile?.loader ? activeProfile.loader.toUpperCase() : "Vanilla"}`,
				largeText: `Minecraft ${verId}`,
				largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
				smallImage: activeProfile?.loader === "fabric" ? "fabric" : (activeProfile?.loader === "forge" ? "curse" : "grass"),
				smallText: `Luxmc · ${activeProfile?.loader || "Vanilla"}`,
				startTime: Math.floor(Date.now() / 1000)
			}).catch(() => {});

			downloadProgressPercent = 100;
			launchStatusText = `Minecraft em execução (PID: ${result.pid})`;
			toast(`🎮 Minecraft ${verId} iniciado com sucesso! (PID: ${result.pid})`, "success");
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
			screenshotsList = screenshotsList.filter(s => s.path !== path);
			toast("Captura de tela removida!", "info");
		} catch (e) {
			toast("Erro ao excluir captura: " + String(e), "error");
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
</script>

<div class="flex gap-8 h-full w-full select-none" in:fade={{ duration: 300 }}>
	
	<!-- Center Main Instance View -->
	<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-2 space-y-5">
		
		<!-- Top Voltar Link -->
		<a href="/instances" class="flex items-center gap-2 text-xs font-bold text-white/50 hover:text-white transition-colors w-fit group">
			<ArrowLeft class="w-3.5 h-3.5 transition-transform group-hover:-translate-x-1" /> Voltar
		</a>

		<!-- Instance Hero Card -->
		<div class="bg-[#18191c] border border-white/5 rounded-3xl p-6 flex flex-col justify-between shadow-xl relative group">
			
			<!-- Minecraft Background Artwork -->
			<div class="absolute inset-0 pointer-events-none z-0 overflow-hidden rounded-3xl">
				<img 
					src={activeProfile?.loader === 'vanilla' ? '/vanilla_banner.png' : '/modpack_fo.webp'} 
					alt="Minecraft Banner" 
					class="w-full h-full object-cover opacity-35 group-hover:scale-105 transition-transform duration-700" 
				/>
				<div class="absolute inset-0 bg-gradient-to-t from-[#18191c] via-[#18191c]/80 to-[#18191c]/40"></div>
			</div>

			<div class="flex items-center justify-between relative z-10">
				<!-- Icon & Badges & Title -->
				<div class="flex items-center gap-4">
				<div class="h-16 w-16 rounded-2xl bg-[#222328] border border-white/10 flex items-center justify-center p-2 shadow-inner overflow-hidden">
					{#if activeProfile?.icon && (activeProfile.icon.startsWith("http") || activeProfile.icon.startsWith("/") || activeProfile.icon.startsWith("data:"))}
						<img src={activeProfile.icon} alt={activeProfile.name} class="w-12 h-12 object-cover rounded-xl" />
					{:else}
						<img src="/grass_block.png" alt="Minecraft" class="w-12 h-12 object-contain drop-shadow [image-rendering:pixelated]" />
					{/if}
				</div>

					<div>
						<div class="flex items-center gap-2">
							<span class="bg-emerald-500/20 text-emerald-400 text-[9px] font-bold px-2 py-0.5 rounded-md uppercase border border-emerald-500/30 flex items-center gap-1">
								<span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
								{activeProfile?.loader || 'Vanilla'}
							</span>
							<span class="bg-white/10 text-white/70 text-[9px] font-bold px-2 py-0.5 rounded-md">
								MC {activeProfile?.mcVersion || '1.20.4'}
							</span>
						</div>

						<h1 class="text-2xl font-black text-white mt-1 tracking-tight">
							{activeProfile?.name || 'Latest Release'}
						</h1>
					</div>
				</div>

				<div class="flex items-center gap-3">
					<!-- Prominent Instance Settings Button -->
					<button 
						type="button"
						class="bg-[#222328] hover:bg-brand-500/20 text-white hover:text-brand-500 text-xs font-black px-6 py-3.5 rounded-full border border-brand-500/40 hover:border-brand-500 transition-all flex items-center gap-2.5 shadow-xl cursor-pointer shrink-0 hover:scale-105 active:scale-95 group"
						onclick={() => showInstanceSettingsModal = true}
						title="Abrir todas as configurações da instância"
					>
						<SettingsIcon class="w-4 h-4 text-brand-500 group-hover:rotate-45 transition-transform" />
						<span>Configurações da Instância</span>
					</button>

					<button 
						type="button"
						class="hover:bg-white/15 active:scale-95 text-white bg-white/10 text-xs font-bold px-5 py-3.5 rounded-full border border-white/15 transition-all flex items-center gap-2 shadow-lg cursor-pointer shrink-0"
						onclick={openHostWorldModal}
						title="Gerar link próprio de conexão para amigos jogarem no seu mundo"
					>
						<Share2 class="w-4 h-4 text-brand-500" />
						<span>Hostear Mundo</span>
					</button>

					<!-- Big Metallic Action Button (JOGAR / Instalar) -->
					<button 
						class="hover:brightness-110 active:scale-95 text-black text-xs font-black px-9 py-3.5 rounded-full border border-white/20 transition-all flex items-center gap-2.5 cursor-pointer shrink-0 hover:scale-105 {appState.isGameRunning ? 'shadow-[0_0_25px_rgba(34,197,94,0.5)] bg-emerald-500' : 'shadow-[0_0_25px_rgba(226,184,107,0.4)]'}"
						style={appState.isGameRunning ? '' : "background-color: var(--accent-color, #e2b86b);"}
						onclick={handlePlay}
						disabled={isLaunching}
					>
						{#if isLaunching}
							<RefreshCw class="w-4 h-4 animate-spin" /> {launchStatusText || 'Iniciando...'}
						{:else if appState.isGameRunning}
							<span class="relative flex h-3 w-3">
								<span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
								<span class="relative inline-flex rounded-full h-3 w-3 bg-green-500"></span>
							</span>
							JOGANDO
						{:else if isInstalled}
							<Play class="w-4 h-4 fill-current" /> JOGAR MINECRAFT
						{:else}
							<Download class="w-4 h-4" /> Instalar e Jogar
						{/if}
					</button>
				</div>
			</div>

			<!-- Visual Download & Launch Progress Bar -->
			{#if isLaunching}
				<div class="mt-4 p-3.5 rounded-2xl bg-[#141518] border border-white/10 space-y-2 shadow-inner" in:fade={{ duration: 150 }}>
					<div class="flex items-center justify-between text-xs font-bold">
						<span class="text-white/80 flex items-center gap-2">
							<RefreshCw class="w-3.5 h-3.5 animate-spin text-brand-500" />
							{launchStatusText}
						</span>
						<span class="font-mono text-brand-500 font-black">{downloadProgressPercent}%</span>
					</div>
					<div class="w-full h-2 rounded-full bg-white/5 overflow-hidden relative">
						<div 
							class="h-full bg-gradient-to-r from-brand-500 to-emerald-400 rounded-full transition-all duration-300 shadow-[0_0_12px_rgba(226,184,107,0.6)]"
							style="width: {downloadProgressPercent}%;"
						></div>
					</div>
				</div>
			{/if}

			<!-- Status Bar Info -->
			<div class="flex items-center justify-between border-t border-white/5 mt-6 pt-4 text-xs font-medium">
				<div class="flex items-center gap-8">
					<div>
						<span class="text-[10px] font-bold text-white/40 uppercase block">Status</span>
						<span class="text-white flex items-center gap-1.5 font-bold mt-0.5">
							{#if isLaunching}
								<span class="w-2 h-2 rounded-full bg-amber-400 animate-ping"></span> {launchStatusText}
							{:else}
								<Check class="w-3.5 h-3.5 text-emerald-400" /> Pronto para jogar
							{/if}
						</span>
					</div>

					<div>
						<span class="text-[10px] font-bold text-white/40 uppercase block">Tempo de Jogo</span>
						<span class="text-white font-bold mt-0.5 block">0m</span>
					</div>
				</div>

				<div class="flex items-center gap-2">
					<button 
						class="bg-[#222328] hover:bg-white/10 text-white/80 hover:text-white px-4 py-2 rounded-full border border-white/10 text-xs font-bold flex items-center gap-2 transition-all cursor-pointer"
						onclick={openInstanceFolder}
					>
						<FolderOpen class="w-3.5 h-3.5" /> Abrir Pasta da Instância
					</button>
					<button 
						type="button"
						class="bg-[#1c1d22] border border-brand-500/40 hover:border-brand-500 hover:bg-brand-500/10 text-brand-500 hover:text-white px-5 py-2.5 rounded-2xl text-xs font-black flex items-center gap-2 transition-all shadow-md hover:scale-[1.02] active:scale-95 cursor-pointer"
						onclick={() => { instanceNameInput = activeProfile?.name || "Latest Release"; showInstanceSettingsModal = true; }}
					>
						<SettingsIcon class="w-4 h-4 text-brand-500" /> Configurações da Instância
					</button>
				</div>
			</div>

		</div>

		<!-- Main Navigation Tabs -->
		<div class="flex items-center justify-between border-b border-white/5 pb-2">
			<div class="flex gap-2">
				<button 
					class="px-5 py-2.5 rounded-full text-xs font-bold transition-all flex items-center gap-2 relative {mainTab === 'conteudo' ? 'bg-[#222328] text-white border border-white/10 shadow-sm' : 'text-white/40 hover:text-white'}"
					onclick={() => mainTab = 'conteudo'}
				>
					<Layers class="w-4 h-4 text-amber-400" /> Conteúdo
					{#if mainTab === 'conteudo'}
						<div class="absolute bottom-[-9px] left-3 right-3 h-0.5 rounded-full" style="background-color: var(--accent-color, #e2b86b);"></div>
					{/if}
				</button>

				<button 
					class="px-5 py-2.5 rounded-full text-xs font-bold transition-all flex items-center gap-2 {mainTab === 'mundos' ? 'bg-[#222328] text-white border border-white/10 shadow-sm' : 'text-white/40 hover:text-white'}"
					onclick={() => mainTab = 'mundos'}
				>
					<Globe2 class="w-4 h-4 text-emerald-400" /> Mundos ({worldsList.length})
				</button>

				<button 
					class="px-5 py-2.5 rounded-full text-xs font-bold transition-all flex items-center gap-2 {mainTab === 'galeria' ? 'bg-[#222328] text-white border border-white/10 shadow-sm' : 'text-white/40 hover:text-white'}"
					onclick={() => mainTab = 'galeria'}
				>
					<Image class="w-4 h-4 text-purple-400" /> Galeria ({screenshotsList.length})
				</button>

				<button 
					class="px-5 py-2.5 rounded-full text-xs font-bold transition-all flex items-center gap-2 {mainTab === 'ficheiros' ? 'bg-[#222328] text-white border border-white/10 shadow-sm' : 'text-white/40 hover:text-white'}"
					onclick={() => mainTab = 'ficheiros'}
				>
					<Folder class="w-4 h-4 text-blue-400" /> Ficheiros
				</button>
			</div>

			<button class="text-white/40 hover:text-white p-2.5 rounded-full transition-colors cursor-pointer" title="Atualizar dados" onclick={refreshAllData}>
				<RefreshCw class="w-4 h-4 {isLoadingData ? 'animate-spin' : ''}" />
			</button>
		</div>

		<!-- TAB 1: Conteúdo -->
		{#if mainTab === 'conteudo'}
			<div class="flex flex-col gap-4">
				<div class="flex flex-wrap items-center justify-between gap-3">
					<div class="flex bg-[#18191c] border border-white/10 rounded-full p-1 gap-1">
						<button 
							class="px-4 py-1.5 rounded-full text-xs font-bold transition-all flex items-center gap-1.5 {subTab === 'mods' ? 'bg-[#25262c] text-white shadow-sm border border-white/10' : 'text-white/40 hover:text-white'}"
							onclick={() => subTab = 'mods'}
						>
							<Puzzle class="w-3.5 h-3.5 text-blue-400" /> Mods ({instanceMods.length})
						</button>
						<button 
							class="px-4 py-1.5 rounded-full text-xs font-bold transition-all flex items-center gap-1.5 {subTab === 'resourcepacks' ? 'bg-[#25262c] text-white shadow-sm border border-white/10' : 'text-white/40 hover:text-white'}"
							onclick={() => subTab = 'resourcepacks'}
						>
							<Box class="w-3.5 h-3.5 text-amber-400" /> Pacotes de recursos ({resourcePacks.length})
						</button>
						<button 
							class="px-4 py-1.5 rounded-full text-xs font-bold transition-all flex items-center gap-1.5 {subTab === 'shaders' ? 'bg-[#25262c] text-white shadow-sm border border-white/10' : 'text-white/40 hover:text-white'}"
							onclick={() => subTab = 'shaders'}
						>
							<Sparkles class="w-3.5 h-3.5 text-purple-400" /> Shaders ({shaderPacks.length})
						</button>
						<button 
							class="px-4 py-1.5 rounded-full text-xs font-bold transition-all flex items-center gap-1.5 {subTab === 'datapacks' ? 'bg-[#25262c] text-white shadow-sm border border-white/10' : 'text-white/40 hover:text-white'}"
							onclick={() => subTab = 'datapacks'}
						>
							<Code class="w-3.5 h-3.5 text-emerald-400" /> {"{}"} Datapacks ({dataPacks.length})
						</button>
					</div>

					<div class="flex items-center gap-2">
						{#if subTab === 'mods'}
							<button 
								class="bg-[#222328] hover:bg-white/10 text-white/80 hover:text-white px-4 py-2 rounded-full border border-white/10 text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer"
								onclick={handleOpenModsFolder}
							>
								<FolderOpen class="w-3.5 h-3.5" /> Abrir Pasta mods/
							</button>
							<a 
								href="/mods"
								class="bg-[#222328] hover:bg-white/10 text-white/80 hover:text-white px-4 py-2 rounded-full border border-white/10 text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer"
							>
								<Search class="w-3.5 h-3.5" /> Obter Mais Mods
							</a>
							<button 
								class="text-black px-5 py-2 rounded-full text-xs font-black flex items-center gap-1.5 shadow-sm cursor-pointer hover:scale-105 active:scale-95 transition-all"
								style="background-color: var(--accent-color, #e2b86b);"
								onclick={handleAddModFile}
							>
								<Plus class="w-4 h-4 stroke-[3]" /> Adicionar .JAR
							</button>
						{:else}
							<button 
								class="bg-[#222328] hover:bg-white/10 text-white/70 hover:text-white px-4 py-2 rounded-full text-xs font-bold flex items-center gap-1.5 border border-white/5 transition-all cursor-pointer"
								onclick={handleOpenPackFolder}
							>
								<FolderOpen class="w-3.5 h-3.5" /> Abrir Pasta
							</button>
							<button 
								class="text-black px-5 py-2 rounded-full text-xs font-black flex items-center gap-1.5 shadow-sm cursor-pointer hover:scale-105 active:scale-95 transition-all"
								style="background-color: var(--accent-color, #e2b86b);"
								onclick={handleAddResourcePack}
							>
								<Plus class="w-4 h-4" /> Adicionar .ZIP
							</button>
						{/if}
					</div>
				</div>

				{#if subTab === 'mods'}
					{#if instanceMods.length === 0}
						<div class="bg-[#18191c] border border-white/5 rounded-3xl p-16 flex flex-col items-center justify-center text-center">
							<div class="h-16 w-16 rounded-full bg-blue-500/10 border border-blue-500/20 flex items-center justify-center mb-4 text-blue-400">
								<Puzzle class="w-8 h-8" />
							</div>
							<h3 class="text-base font-extrabold text-white">Nenhum mod instalado nesta instância</h3>
							<p class="text-xs text-white/40 mt-1 max-w-md">
								Você pode instalar mods incríveis diretamente pela Central de Conteúdo ou importar arquivos .jar do seu computador.
							</p>
							<div class="flex items-center gap-3 mt-6">
								<a 
									href="/mods"
									class="text-black px-6 py-2.5 rounded-full text-xs font-black transition-all hover:scale-105 active:scale-95 shadow-md flex items-center gap-2"
									style="background-color: var(--accent-color, #e2b86b);"
								>
									<Sparkles class="w-4 h-4" /> Baixar Mods na Central
								</a>
								<button 
									class="bg-[#222328] hover:bg-white/10 border border-white/10 text-white text-xs font-bold px-6 py-2.5 rounded-full transition-all flex items-center gap-2 cursor-pointer"
									onclick={handleAddModFile}
								>
									<Plus class="w-4 h-4" /> Importar .JAR Local
								</button>
							</div>
						</div>
					{:else}
						<VirtualList items={filteredMods} itemHeight={72} height="600px" class="rounded-2xl">
							{#snippet children(mod: FileTreeEntry, _index: number)}
								{@const isDisabled = mod.name.endsWith('.disabled')}
								{@const rawName = mod.name.replace('.disabled', '').replace('.jar', '')}
								{@const displayName = rawName.includes('_') && /^\d+_\d+$/.test(rawName) ? 'Mod #' + rawName.split('_')[0] : rawName.replace(/_/g, ' ')}
								<div class="bg-[#18191c] border border-white/5 hover:border-white/15 p-4 flex items-center justify-between transition-all group {isDisabled ? 'opacity-50' : ''}" style="content-visibility: auto;">
									<div class="flex items-center gap-3.5 min-w-0">
										<div class="w-10 h-10 rounded-xl {isDisabled ? 'bg-white/5 text-white/30' : 'bg-blue-500/15 text-blue-400 border border-blue-500/30'} flex items-center justify-center shrink-0 shadow-sm">
											<Puzzle class="w-5 h-5" />
										</div>
										<div class="min-w-0">
											<div class="flex items-center gap-2">
												<h5 class="text-xs font-bold text-white truncate max-w-[320px]" title={displayName}>{displayName}</h5>
												<span class="text-[9px] font-black uppercase px-2 py-0.5 rounded-full {isDisabled ? 'bg-red-500/10 text-red-400 border border-red-500/20' : 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'}">
													{isDisabled ? 'Desativado' : 'Ativo'}
												</span>
											</div>
											<div class="flex items-center gap-2 mt-1 text-[10px] text-white/40 font-mono">
												<span>{mod.size > 1048576 ? (mod.size / (1024 * 1024)).toFixed(2) + ' MB' : Math.round(mod.size / 1024) + ' KB'}</span>
												<span>·</span>
												<span class="truncate max-w-[250px]">{mod.name}</span>
											</div>
										</div>
									</div>

									<div class="flex items-center gap-2 shrink-0">
										<button 
											type="button"
											class="p-2 rounded-xl transition-all cursor-pointer {isDisabled ? 'bg-white/5 text-white/40 hover:text-white hover:bg-white/10' : 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 hover:bg-emerald-500/30'}"
											onclick={() => handleToggleMod(mod)}
											title={isDisabled ? 'Ativar mod' : 'Desativar mod'}
										>
											{#if isDisabled}
												<ToggleLeft class="w-4 h-4" />
											{:else}
												<ToggleRight class="w-4 h-4" />
											{/if}
										</button>

										<button 
											type="button"
											class="p-2 rounded-xl text-white/40 hover:text-red-400 hover:bg-red-500/10 transition-all cursor-pointer"
											onclick={() => handleDeleteMod(mod)}
											title="Excluir mod"
										>
											<Trash2 class="w-4 h-4" />
										</button>
									</div>
								</div>
							{/snippet}
						</VirtualList>
					{/if}
				{:else}
					{#if currentPacksList.length === 0}
						<div class="bg-[#18191c] border border-white/5 rounded-3xl p-16 flex flex-col items-center justify-center text-center">
							<div class="h-16 w-16 rounded-full bg-white/5 flex items-center justify-center mb-4 text-white/20">
								<Box class="w-8 h-8" />
							</div>
							<h3 class="text-base font-extrabold text-white">
								{subTab === 'resourcepacks' ? 'Nenhum pacote de recursos' : subTab === 'shaders' ? 'Nenhum shader instalado' : 'Nenhum datapack instalado'}
							</h3>
							<p class="text-xs text-white/40 mt-1 max-w-sm">
								Esta instância ainda não possui {subTab === 'resourcepacks' ? 'texturas' : subTab === 'shaders' ? 'shaders' : 'datapacks'} adicionados.
							</p>
							<button 
								class="mt-6 bg-[#222328] hover:bg-white/10 border border-white/10 text-white text-xs font-bold px-6 py-2.5 rounded-full transition-all flex items-center gap-2 cursor-pointer"
								onclick={handleAddResourcePack}
							>
								<Plus class="w-4 h-4" /> Importar Arquivo .ZIP
							</button>
						</div>
					{:else}
						<div class="grid grid-cols-1 md:grid-cols-2 gap-3">
							{#each currentPacksList as pack}
								<div class="bg-[#18191c] border border-white/5 hover:border-white/15 p-4 rounded-2xl flex items-center justify-between transition-all group">
									<div class="flex items-center gap-3.5 min-w-0">
										<div class="w-10 h-10 rounded-xl bg-amber-500/15 text-amber-400 border border-amber-500/30 flex items-center justify-center shrink-0 shadow-sm">
											<Box class="w-5 h-5" />
										</div>
										<div class="min-w-0">
											<span class="text-xs font-bold text-white truncate block max-w-[220px]" title={pack.name}>{pack.name}</span>
											<span class="text-[10px] text-white/40 font-mono mt-0.5 block">{pack.size > 1048576 ? (pack.size / (1024 * 1024)).toFixed(2) + ' MB' : Math.round(pack.size / 1024) + ' KB'}</span>
										</div>
									</div>
									<button 
										type="button" 
										class="p-2 rounded-xl text-white/40 hover:text-red-400 hover:bg-red-500/10 transition-all cursor-pointer"
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

		<!-- TAB 2: Mundos Reais -->
		{:else if mainTab === 'mundos'}
			<div class="space-y-4">
				<div class="flex items-center justify-between">
					<h3 class="text-sm font-bold text-white">Mundos Salvos nesta Instância</h3>
					<button class="text-xs font-bold hover:underline flex items-center gap-1 cursor-pointer" style="color: var(--accent-color, #e2b86b);" onclick={openInstanceFolder}>
						<FolderOpen class="w-3.5 h-3.5" /> Abrir pasta saves/
					</button>
				</div>

				{#if worldsList.length === 0}
					<div class="bg-[#18191c] border border-white/5 rounded-3xl p-16 flex flex-col items-center justify-center text-center">
						<Globe2 class="w-12 h-12 text-white/20 mb-3" />
						<h4 class="text-sm font-bold text-white">Nenhum mundo encontrado</h4>
						<p class="text-xs text-white/40 mt-1">Abra o Minecraft e crie seu primeiro mundo singleplayer!</p>
					</div>
				{:else}
					<div class="grid grid-cols-1 md:grid-cols-2 gap-3">
						{#each worldsList as world}
							<div class="bg-[#18191c] border border-white/5 p-4 rounded-2xl flex items-center justify-between hover:border-white/15 transition-all group">
								<div class="flex items-center gap-3.5 min-w-0">
									<div class="h-12 w-12 rounded-xl bg-black/40 border border-white/10 overflow-hidden flex items-center justify-center shrink-0 shadow-md">
										{#if world.iconBase64}
											<img src={world.iconBase64} alt={world.name} class="w-full h-full object-cover [image-rendering:pixelated]" />
										{:else}
											<img src="/grass_block.png" alt="Mundo" class="w-7 h-7 object-contain drop-shadow" />
										{/if}
									</div>
									<div class="min-w-0">
										<h5 class="text-xs font-bold text-white truncate">{world.name}</h5>
										<div class="flex items-center gap-2 mt-0.5 text-[10px] text-white/40">
											<span class="text-emerald-400 font-semibold">{world.gameMode || 'Sobrevivência'}</span>
											<span>·</span>
											<span>{(world.sizeBytes / (1024 * 1024)).toFixed(1)} MB</span>
										</div>
									</div>
								</div>

								<div class="flex items-center gap-1.5 shrink-0">
									<button 
										class="bg-white/5 hover:brightness-110 text-white px-4 py-2 rounded-full text-xs font-bold transition-all cursor-pointer flex items-center gap-1.5 opacity-80 group-hover:opacity-100 active:scale-95"
										onclick={handlePlay}
										title="Jogar este mundo"
									>
										<Play class="w-3 h-3 fill-current" /> Jogar
									</button>
									<button 
										class="p-2 rounded-full text-white/40 hover:text-red-400 hover:bg-red-500/10 transition-colors cursor-pointer"
										onclick={() => handleDeleteWorld(world.folderName)}
										title="Excluir este mundo"
									>
										<Trash2 class="w-3.5 h-3.5" />
									</button>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>

		<!-- TAB 3: Galeria Real de Screenshots -->
		{:else if mainTab === 'galeria'}
			<div class="space-y-4">
				<div class="flex items-center justify-between">
					<h3 class="text-sm font-bold text-white">Capturas de Tela (F2)</h3>
					<button class="text-xs font-bold hover:underline flex items-center gap-1 cursor-pointer" style="color: var(--accent-color, #e2b86b);" onclick={openInstanceFolder}>
						<FolderOpen class="w-3.5 h-3.5" /> Abrir pasta screenshots/
					</button>
				</div>

				{#if screenshotsList.length === 0}
					<div class="bg-[#18191c] border border-white/5 rounded-3xl p-16 flex flex-col items-center justify-center text-center">
						<Image class="w-12 h-12 text-white/20 mb-3" />
						<h4 class="text-sm font-bold text-white">Nenhuma captura de tela</h4>
						<p class="text-xs text-white/40 mt-1">Pressione F2 dentro do jogo para capturar momentos épicos!</p>
					</div>
				{:else}
					<div class="grid grid-cols-2 md:grid-cols-3 gap-4">
						{#each screenshotsList as shot}
							<div 
								class="bg-[#18191c] border border-white/5 rounded-2xl overflow-hidden group relative cursor-pointer hover:border-white/20 transition-all shadow-md"
								onclick={() => previewScreenshot = shot}
								role="button"
								tabindex="0"
								onkeydown={(e) => { if (e.key === 'Enter') previewScreenshot = shot; }}
							>
								<div class="h-36 bg-black/60 flex items-center justify-center overflow-hidden">
									<img 
										src={shot.dataUrl || convertFileSrc(shot.path)} 
										alt={shot.name} 
										class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
										loading="lazy"
									/>
								</div>
								<div class="p-3 flex items-center justify-between bg-[#141518]/90">
									<span class="text-xs font-medium text-white truncate max-w-[80%]">{shot.name}</span>
									<button 
										class="text-white/40 hover:text-red-400 transition-colors p-1 rounded-full hover:bg-white/5"
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

		<!-- TAB 4: Ficheiros Reais do Mine (Explorador Interativo de Arquivos) -->
		{:else if mainTab === 'ficheiros'}
			<div class="space-y-4">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						<h3 class="text-sm font-bold text-white">Explorador de Arquivos</h3>
						<span class="text-[10px] text-white/40 font-mono">({fileTree.length} itens)</span>
					</div>
					<button class="text-xs font-bold hover:underline flex items-center gap-1 cursor-pointer" style="color: var(--accent-color, #e2b86b);" onclick={openInstanceFolder}>
						<FolderOpen class="w-3.5 h-3.5" /> Abrir no Gerenciador Linux
					</button>
				</div>

				<!-- Breadcrumbs & Navigation Toolbar -->
				<div class="bg-[#18191c] border border-white/10 rounded-2xl p-3 flex items-center justify-between gap-3 shadow-md">
					<div class="flex items-center gap-1 text-xs font-mono overflow-x-auto custom-scrollbar py-0.5">
						<button 
							type="button" 
							class="text-xs font-bold px-2.5 py-1 rounded-full hover:bg-white/10 text-white/60 hover:text-white transition-all cursor-pointer shrink-0"
							onclick={() => navigateBreadcrumb(-1)}
						>
							~ raiz
						</button>
						{#each fileBreadcrumbs as seg, idx}
							<ChevronRight class="w-3.5 h-3.5 text-white/30 shrink-0" />
							<button 
								type="button" 
								class="text-xs font-bold px-2.5 py-1 rounded-full transition-all cursor-pointer shrink-0 {idx === fileBreadcrumbs.length - 1 ? 'bg-white/10 text-white' : 'text-white/60 hover:text-white hover:bg-white/5'}"
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
								class="p-1.5 rounded-full bg-white/5 hover:bg-white/15 text-white transition-colors cursor-pointer"
								title="Subir nível"
								onclick={navigateUp}
							>
								<ArrowUp class="w-4 h-4" />
							</button>
						{/if}
						<button 
							type="button" 
							class="p-1.5 rounded-full bg-white/5 hover:bg-white/15 text-white transition-colors cursor-pointer"
							title="Atualizar pasta"
							onclick={refreshAllData}
						>
							<RefreshCw class="w-4 h-4 {isLoadingData ? 'animate-spin' : ''}" />
						</button>
					</div>
				</div>

				<!-- Files Table / List -->
				<div class="bg-[#18191c] border border-white/5 rounded-2xl p-2 space-y-1 shadow-md max-h-[500px] overflow-y-auto custom-scrollbar">
					{#if fileTree.length === 0}
						<div class="text-xs text-white/40 py-8 text-center">Nenhum arquivo nesta pasta.</div>
					{:else}
						{#each fileTree as file}
							<div 
								class="flex items-center justify-between p-2.5 hover:bg-white/5 rounded-xl text-xs transition-colors group cursor-pointer"
								onclick={() => handleOpenFile(file)}
								role="button"
								tabindex="0"
								onkeydown={(e) => { if (e.key === 'Enter') handleOpenFile(file); }}
							>
								<div class="flex items-center gap-3 min-w-0">
									{#if file.isDir}
										<Folder class="w-4 h-4 text-amber-400 shrink-0" />
									{:else}
										<FileText class="w-4 h-4 text-white/40 shrink-0 group-hover:text-emerald-400 transition-colors" />
									{/if}
									<span class="font-medium text-white truncate">{file.name}</span>
								</div>

								<div class="flex items-center gap-3 shrink-0">
									<span class="text-[10px] text-white/30 font-mono">
										{file.isDir ? 'Pasta' : `${Math.round(file.size / 1024)} KB`}
									</span>
									<button 
										type="button" 
										class="p-1.5 rounded-full text-white/20 hover:text-red-400 hover:bg-red-500/10 transition-colors opacity-0 group-hover:opacity-100"
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

	<!-- Right Sidebar (Notícias Luxmc & Comunidade) -->
	<RightSidebar />

</div>

<!-- Screenshot Fullscreen Preview Modal -->
{#if previewScreenshot}
	<div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-md flex items-center justify-center p-6" in:fade={{ duration: 150 }}>
		<div class="max-w-4xl w-full bg-[#18191c] border border-white/10 rounded-3xl overflow-hidden shadow-2xl flex flex-col">
			<div class="p-4 border-b border-white/10 flex items-center justify-between">
				<div class="flex items-center gap-2 min-w-0">
					<Image class="w-4 h-4 text-purple-400 shrink-0" />
					<span class="text-xs font-bold text-white truncate">{previewScreenshot.name}</span>
				</div>
				<div class="flex items-center gap-2 shrink-0">
					<button 
						type="button" 
						class="px-3.5 py-1.5 rounded-full bg-white/10 hover:bg-white/20 text-white text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer"
						onclick={() => {
							navigator.clipboard.writeText(previewScreenshot!.path);
							toast("Caminho da captura copiado!", "success");
						}}
					>
						<Copy class="w-3.5 h-3.5" /> Copiar Caminho
					</button>
					<button 
						type="button" 
						class="px-3.5 py-1.5 rounded-full bg-red-500/20 hover:bg-red-500/30 text-red-300 text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer"
						onclick={async () => {
							await handleDeleteScreenshot(previewScreenshot!.path);
							previewScreenshot = null;
						}}
					>
						<Trash2 class="w-3.5 h-3.5" /> Excluir
					</button>
					<button 
						type="button" 
						class="p-1.5 rounded-full bg-white/5 hover:bg-white/15 text-white/60 hover:text-white transition-colors cursor-pointer ml-2"
						onclick={() => previewScreenshot = null}
					>
						<X class="w-4 h-4" />
					</button>
				</div>
			</div>
			<div class="p-4 flex items-center justify-center bg-black/70 max-h-[70vh] overflow-hidden">
				<img 
					src={previewScreenshot.dataUrl || convertFileSrc(previewScreenshot.path)} 
					alt={previewScreenshot.name} 
					class="max-h-[65vh] max-w-full object-contain rounded-xl shadow-lg"
				/>
			</div>
			<div class="p-3 bg-[#141518] border-t border-white/5 text-[10px] text-white/40 font-mono truncate px-4">
				{previewScreenshot.path}
			</div>
		</div>
	</div>
{/if}

<!-- In-Launcher File Text Editor Modal -->
{#if activeEditorFile}
	<div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-md flex items-center justify-center p-6" in:fade={{ duration: 150 }}>
		<div class="max-w-4xl w-full h-[80vh] bg-[#18191c] border border-white/10 rounded-3xl overflow-hidden shadow-2xl flex flex-col">
			<div class="p-4 border-b border-white/10 flex items-center justify-between">
				<div class="flex items-center gap-2 min-w-0">
					<FileText class="w-4 h-4 text-emerald-400 shrink-0" />
					<span class="text-xs font-bold text-white truncate">{activeEditorFile.name}</span>
					<span class="text-[10px] font-mono text-white/40 truncate">({activeEditorFile.path})</span>
				</div>
				<div class="flex items-center gap-2 shrink-0">
					<button 
						type="button" 
						class="px-5 py-2 rounded-full text-xs font-black text-black flex items-center gap-1.5 transition-all cursor-pointer hover:scale-105 active:scale-95 shadow-md"
						style="background-color: var(--accent-color, #e2b86b);"
						disabled={isSavingEditor}
						onclick={handleSaveEditorFile}
					>
						<Save class="w-3.5 h-3.5 stroke-[2.5]" /> {isSavingEditor ? 'Salvando...' : 'Guardar Alterações'}
					</button>
					<button 
						type="button" 
						class="p-1.5 rounded-full bg-white/5 hover:bg-white/15 text-white/60 hover:text-white transition-colors cursor-pointer ml-2"
						onclick={() => activeEditorFile = null}
					>
						<X class="w-4 h-4" />
					</button>
				</div>
			</div>
			<div class="flex-1 p-4 bg-[#121316] overflow-hidden">
				<textarea 
					bind:value={activeEditorFile.content}
					class="w-full h-full bg-transparent border-0 outline-none font-mono text-xs text-white/90 leading-relaxed resize-none custom-scrollbar p-2"
					spellcheck="false"
				></textarea>
			</div>
			<div class="p-2.5 bg-[#141518] border-t border-white/5 text-[10px] text-white/40 font-mono px-4 flex justify-between">
				<span>Tamanho: {Math.round(activeEditorFile.content.length / 1024)} KB</span>
				<span>Editor de Arquivos do Luxmc</span>
			</div>
		</div>
	</div>
{/if}

<!-- Modal de Hostear Mundo com Link Próprio -->
{#if showHostModal && hostLinkInfo}
	<div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-md flex items-center justify-center p-6" in:fade={{ duration: 150 }}>
		<div class="max-w-md w-full bg-[#18191c] border border-brand-500/30 rounded-3xl p-6 shadow-2xl space-y-5 relative overflow-hidden select-none">
			<div class="absolute -top-10 -right-10 w-36 h-36 bg-brand-500/15 rounded-full blur-2xl pointer-events-none"></div>

			<div class="flex items-center justify-between">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-brand-500/15 border border-brand-500/30 flex items-center justify-center">
						<Share2 class="w-5 h-5 text-brand-500" />
					</div>
					<div>
						<h3 class="font-extrabold text-white text-base">Hostear Mundo com Link Próprio</h3>
						<p class="text-xs text-white/50">Compartilhe com amigos para entrarem no seu mundo</p>
					</div>
				</div>
				<button 
					type="button" 
					class="p-1.5 rounded-full bg-white/5 hover:bg-white/15 text-white/60 hover:text-white transition-colors cursor-pointer"
					onclick={() => showHostModal = false}
				>
					<X class="w-4 h-4" />
				</button>
			</div>

			<div class="bg-[#121316] border border-white/10 rounded-2xl p-4 space-y-3 shadow-inner">
				<div>
					<span class="text-[10px] font-extrabold text-brand-500 uppercase tracking-wider block mb-1">Link Próprio do Luxmc (Compartilhável)</span>
					<div class="flex items-center gap-2">
						<input 
							type="text" 
							readonly 
							value={hostLinkInfo.shareLink} 
							class="flex-1 bg-black/50 border border-white/10 rounded-xl px-3.5 py-2 text-xs font-mono text-white/90 outline-none select-all"
						/>
						<button 
							type="button" 
							class="px-4 py-2 rounded-xl bg-brand-500 hover:brightness-110 text-black font-black text-xs flex items-center gap-1.5 transition-all cursor-pointer shadow-md active:scale-95"
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

				<div class="grid grid-cols-2 gap-2 pt-2 border-t border-white/5">
					<div>
						<span class="text-[10px] text-white/40 block font-bold">Endereço IP Local:</span>
						<span class="text-xs font-mono text-emerald-400 font-bold">{hostLinkInfo.directAddress}</span>
					</div>
					<div>
						<span class="text-[10px] text-white/40 block font-bold">Porta do Servidor:</span>
						<div class="flex items-center gap-1 mt-0.5">
							<input 
								type="number" 
								bind:value={customHostPort} 
								class="w-20 bg-black/50 border border-white/10 rounded-lg px-2 py-0.5 text-xs font-mono text-white" 
								onchange={refreshHostLink}
							/>
						</div>
					</div>
				</div>
			</div>

			<div class="text-[11px] text-white/50 leading-relaxed space-y-1 bg-amber-500/10 border border-amber-500/20 p-3 rounded-2xl">
				<div class="font-bold text-amber-300">Como funciona?</div>
				<div>1. Abra seu mundo no Minecraft e clique em <b>"Aberto para LAN"</b>.</div>
				<div>2. Envie o <b>Link Próprio</b> acima para seus amigos colarem no Luxmc.</div>
			</div>
		</div>
	</div>
{/if}



<!-- SKlauncher-Style Instance Settings Modal -->
{#if showInstanceSettingsModal}
	<div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-md flex items-center justify-center p-6" in:fade={{ duration: 150 }}>
		<div class="w-full max-w-3xl bg-[#141518] border border-white/10 rounded-3xl p-6 shadow-2xl space-y-6 flex flex-col justify-between select-none h-[560px]">
			
			<div class="flex gap-6 h-full overflow-hidden">
				<!-- Left Category List -->
				<div class="w-56 shrink-0 border-r border-white/5 pr-4 flex flex-col justify-between">
					<div class="space-y-4">
						<h2 class="text-sm font-extrabold text-white px-2">Configurações da Instância</h2>
						<nav class="flex flex-col gap-1">
							<button 
								type="button"
								class="flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-xs font-bold transition-all w-full text-left cursor-pointer {activeInstanceSection === 'geral' ? 'bg-[#222328] text-white border border-white/10 shadow-sm' : 'text-white/40 hover:text-white'}"
								onclick={() => activeInstanceSection = 'geral'}
							>
								<Box class="w-4 h-4 text-emerald-400" /> Geral
							</button>
							<button 
								type="button"
								class="flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-xs font-bold transition-all w-full text-left cursor-pointer {activeInstanceSection === 'instalacao' ? 'bg-[#222328] text-white border border-white/10 shadow-sm' : 'text-white/40 hover:text-white'}"
								onclick={() => activeInstanceSection = 'instalacao'}
							>
								<Download class="w-4 h-4 text-cyan-400" /> Instalação
							</button>
							<button 
								type="button"
								class="flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-xs font-bold transition-all w-full text-left cursor-pointer {activeInstanceSection === 'otimizacao' ? 'bg-[#222328] text-[#caa97c] border border-[#caa97c]/30 shadow-sm' : 'text-white/40 hover:text-white'}"
								onclick={() => activeInstanceSection = 'otimizacao'}
							>
								<Zap class="w-4 h-4 text-[#caa97c]" /> Otimização Luxmc
							</button>
							<button 
								type="button"
								class="flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-xs font-bold transition-all w-full text-left cursor-pointer {activeInstanceSection === 'janela' ? 'bg-[#222328] text-white border border-white/10 shadow-sm' : 'text-white/40 hover:text-white'}"
								onclick={() => activeInstanceSection = 'janela'}
							>
								<Layers class="w-4 h-4 text-purple-400" /> Janela
							</button>
							<button 
								type="button"
								class="flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-xs font-bold transition-all w-full text-left cursor-pointer {activeInstanceSection === 'java' ? 'bg-[#222328] text-white border border-white/10 shadow-sm' : 'text-white/40 hover:text-white'}"
								onclick={() => activeInstanceSection = 'java'}
							>
								<Sparkles class="w-4 h-4 text-amber-400" /> Java e Memória
							</button>
							<button 
								type="button"
								class="flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-xs font-bold transition-all w-full text-left cursor-pointer {activeInstanceSection === 'hooks' ? 'bg-[#222328] text-white border border-white/10 shadow-sm' : 'text-white/40 hover:text-white'}"
								onclick={() => activeInstanceSection = 'hooks'}
							>
								<Code class="w-4 h-4 text-rose-400" /> Launch Hooks
							</button>
						</nav>
					</div>

					<div class="text-[10px] text-white/30 font-mono px-2">
						Minecraft {activeProfile?.mcVersion || '1.21.4'}
					</div>
				</div>

				<!-- Right Panel Content -->
				<div class="flex-1 flex flex-col justify-between overflow-y-auto custom-scrollbar pr-1 space-y-6">
					
					{#if activeInstanceSection === 'geral'}
						<div class="space-y-6">
							<div>
								<div class="flex items-center gap-2">
									<Box class="w-4 h-4 text-emerald-400" />
									<h3 class="text-xs font-bold text-white uppercase tracking-wider">Geral</h3>
								</div>
								<p class="text-[11px] text-white/40 mt-0.5">Nome, ícone e ações da instância</p>
							</div>

							<!-- Icon & Name Row -->
							<div class="space-y-3">
								<span class="text-xs font-bold text-white/70 block">Nome da Instância</span>
								<div class="flex items-center gap-4">
									<div class="h-14 w-14 rounded-2xl bg-[#1c1d22] border border-white/10 flex items-center justify-center shrink-0 p-1">
										<img src="/grass_block.png" alt="Minecraft" class="w-10 h-10 object-contain [image-rendering:pixelated]" />
									</div>
									<input 
										type="text"
										bind:value={instanceNameInput}
										class="flex-1 bg-[#1c1d22] border border-white/10 rounded-2xl px-4 py-3 text-xs font-bold text-white outline-none focus:border-brand-500 transition-colors"
									/>
								</div>
							</div>

							<!-- Instance Actions -->
							<div class="space-y-3 pt-2">
								<div>
									<span class="text-xs font-bold text-white/80 block">Ações da Instância</span>
									<p class="text-[11px] text-white/40 mt-0.5">Reparar ou apagar esta Instância. Estas ações não podem ser desfeitas.</p>
								</div>

								<!-- Repair Button -->
								<button 
									type="button"
									class="w-full bg-[#18191c] hover:bg-amber-500/10 border border-amber-500/30 rounded-2xl p-4 flex items-center gap-4 transition-all cursor-pointer text-left group"
									onclick={() => toast("Recursos da instância reparados com sucesso!", "success")}
								>
									<div class="h-10 w-10 rounded-xl bg-amber-500/20 border border-amber-500/40 flex items-center justify-center shrink-0 text-amber-400 group-hover:scale-110 transition-transform">
										<Sparkles class="w-5 h-5" />
									</div>
									<div>
										<h4 class="text-xs font-bold text-amber-400">Reparar Instância</h4>
										<p class="text-[11px] text-white/40 mt-0.5">Corrigir ficheiros corrompidos e descarregar recursos em falta</p>
									</div>
								</button>

								<!-- Delete Button -->
								<button 
									type="button"
									class="w-full bg-[#18191c] hover:bg-rose-500/10 border border-rose-500/30 rounded-2xl p-4 flex items-center gap-4 transition-all cursor-pointer text-left group"
									onclick={() => toast("Dados da instância removidos.", "info")}
								>
									<div class="h-10 w-10 rounded-xl bg-rose-500/20 border border-rose-500/40 flex items-center justify-center shrink-0 text-rose-400 group-hover:scale-110 transition-transform">
										<Trash2 class="w-5 h-5" />
									</div>
									<div>
										<h4 class="text-xs font-bold text-rose-400">Apagar dados da instância</h4>
										<p class="text-[11px] text-white/40 mt-0.5">Remover permanentemente os ficheiros desta Instância e começar de novo</p>
									</div>
								</button>
							</div>
						</div>

					{:else if activeInstanceSection === 'instalacao'}
						<div class="space-y-6">
							<div>
								<h3 class="text-xs font-bold text-white uppercase tracking-wider">Instalação & Mod Loaders</h3>
								<p class="text-[11px] text-white/40 mt-0.5">Gerencie o loader (Fabric, Forge, NeoForge, Quilt) e versão do jogo</p>
							</div>

							<div class="space-y-3">
								<span class="text-xs font-bold text-white/70 block">Mod Loader Ativo</span>
								<div class="grid grid-cols-2 gap-3">
									{#each ["fabric", "forge", "neoforge", "vanilla"] as loader}
										<button 
											type="button"
											class="p-3 rounded-2xl border text-xs font-bold flex items-center justify-between transition-all cursor-pointer {instanceLoaderType === loader ? 'bg-brand-500/20 border-brand-500 text-brand-500' : 'bg-[#1c1d22] border-white/10 text-white/60 hover:text-white'}"
											onclick={() => instanceLoaderType = loader}
										>
											<span class="capitalize">{loader}</span>
											{#if instanceLoaderType === loader}<Check class="w-4 h-4" />{/if}
										</button>
									{/each}
								</div>
							</div>

							<div class="space-y-2">
								<span class="text-xs font-bold text-white/70 block">Versão do Mod Loader</span>
								<input type="text" bind:value={instanceLoaderVersion} class="w-full bg-[#1c1d22] border border-white/10 rounded-2xl px-4 py-2.5 text-xs text-white font-mono outline-none focus:border-brand-500" />
							</div>
						</div>

					{:else if activeInstanceSection === 'otimizacao'}
						<div class="space-y-5">
							<div>
								<div class="flex items-center gap-2">
									<Zap class="w-4 h-4 text-[#caa97c]" />
									<h3 class="text-xs font-bold text-white uppercase tracking-wider">Sistema de Otimização Luxmc</h3>
								</div>
								<p class="text-[11px] text-white/40 mt-0.5">Tuning inteligente de JVM, detecção de hardware e aceleração gráfica Linux</p>
							</div>

							<!-- Hardware Detected Card -->
							<div class="bg-[#18191f] border border-white/5 rounded-2xl p-4 space-y-3">
								<div class="flex items-center justify-between">
									<span class="text-xs font-bold text-white/90 flex items-center gap-2">
										<Cpu class="w-3.5 h-3.5 text-brand-500" /> Hardware Detectado no Sistema
									</span>
									<span class="text-[10px] font-mono text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-md border border-emerald-500/20">
										Linux-First
									</span>
								</div>
								<div class="grid grid-cols-2 gap-2 text-xs">
									<div class="bg-black/30 p-2.5 rounded-xl border border-white/5">
										<span class="text-[10px] text-white/40 block">GPU & Renderizador</span>
										<span class="text-xs font-bold text-white truncate block mt-0.5" title={gpuInfo?.renderer || 'Buscando...'}>
											{gpuInfo?.renderer || 'AMD Radeon / Mesa RADV'}
										</span>
										<span class="text-[10px] text-brand-500 font-mono block mt-0.5">
											Driver: {gpuInfo?.driver || 'amdgpu'}
										</span>
									</div>
									<div class="bg-black/30 p-2.5 rounded-xl border border-white/5">
										<span class="text-[10px] text-white/40 block">Memória RAM do Sistema</span>
										<span class="text-xs font-bold text-white block mt-0.5">
											{Math.round(systemRamMb / 1024)} GB Totais
										</span>
										<span class="text-[10px] text-white/40 font-mono block mt-0.5">
											Alocado p/ instância: {(instanceRamMb / 1024).toFixed(1)} GB
										</span>
									</div>
								</div>
							</div>

							<!-- Aikar's Flags Smart Optimization Card -->
							<div class="bg-[#18191f] border border-white/5 rounded-2xl p-4 space-y-3">
								<div class="flex items-center justify-between">
									<div>
										<div class="flex items-center gap-2">
											<span class="text-xs font-bold text-white">Flags de JVM Inteligentes (Aikar G1GC)</span>
											<span class="text-[9px] bg-brand-500/20 text-brand-500 px-1.5 py-0.5 rounded font-bold">Base do Sistema</span>
										</div>
										<span class="text-[10px] text-white/40 block mt-0.5">
											Ajusta dinamicamente tamanhos de região e new-generation para os {instanceRamMb} MB alocados
										</span>
									</div>
									<button 
										type="button"
										aria-label="Alternar Flags Aikar"
										class="w-10 h-5 rounded-full transition-all duration-200 relative flex items-center px-0.5 cursor-pointer {instanceAutoOptimize ? 'bg-[#caa97c] shadow-[0_0_10px_rgba(202,169,124,0.35)]' : 'bg-[#2d2e34]'}"
										onclick={() => instanceAutoOptimize = !instanceAutoOptimize}
									>
										<span class="w-4 h-4 rounded-full bg-white transition-transform duration-200 shadow-md {instanceAutoOptimize ? 'translate-x-5' : 'translate-x-0'}"></span>
									</button>
								</div>

								<!-- Live Visible JVM Flags Preview -->
								<div class="space-y-1.5">
									<div class="flex items-center justify-between text-[10px]">
										<span class="text-white/50">Flags aplicadas em tempo real:</span>
										<span class="font-mono text-white/30">{generatedAikarFlags.length} parâmetros</span>
									</div>
									<div class="bg-black/50 p-2.5 rounded-xl border border-white/5 max-h-24 overflow-y-auto custom-scrollbar font-mono text-[10px] text-[#caa97c] leading-relaxed break-all">
										{generatedAikarFlags.join(" ")}
									</div>
								</div>
							</div>

							<!-- Performance Modpack Card -->
							<div class="bg-[#18191f] border border-white/5 rounded-2xl p-4 space-y-3">
								<div class="flex items-center justify-between">
									<div>
										<div class="flex items-center gap-2">
											<span class="text-xs font-bold text-white">Pacote de Mods de Performance</span>
											{#if perfPackInfo?.available}
												<span class="text-[9px] bg-emerald-500/20 text-emerald-400 px-1.5 py-0.5 rounded font-bold">Disponível</span>
											{:else}
												<span class="text-[9px] bg-white/10 text-white/40 px-1.5 py-0.5 rounded font-bold">Indisponível</span>
											{/if}
										</div>
										<span class="text-[10px] text-white/40 block mt-0.5">
											{perfPackInfo?.available ? 'Conjunto homologado de mods de taxa de quadros e redução de RAM' : (perfPackInfo?.reason || 'Requer modloader')}
										</span>
									</div>
									{#if perfPackInfo?.available}
										<button 
											type="button"
											disabled={installingPerfPack}
											class="px-3 py-1.5 bg-[#caa97c] hover:bg-[#d8bc98] disabled:opacity-50 text-black rounded-xl font-bold text-xs transition-all flex items-center gap-1.5 cursor-pointer shadow-md"
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
											<div class="bg-black/30 p-2 rounded-xl border border-white/5">
												<span class="font-bold text-[11px] text-white block">{mod.title}</span>
												<span class="text-[9px] text-white/40 block line-clamp-2 mt-0.5">{mod.description}</span>
											</div>
										{/each}
									</div>
								{/if}
							</div>

							<!-- Linux Mesa Zink / Vulkan Toggle -->
							<div class="bg-[#18191f] border border-white/5 rounded-2xl p-4 flex items-center justify-between">
								<div>
									<div class="flex items-center gap-2">
										<span class="text-xs font-bold text-white">Mesa Zink / Vulkan (Linux)</span>
										<span class="text-[9px] bg-cyan-500/20 text-cyan-400 px-1.5 py-0.5 rounded font-bold">Opt-in</span>
									</div>
									<span class="text-[10px] text-white/40 block mt-0.5">
										Executa o OpenGL sobre Vulkan via Mesa Zink no Linux (recomendado para AMD RADV / Intel)
									</span>
								</div>
								<button 
									type="button"
									aria-label="Alternar Aceleração Vulkan"
									class="w-10 h-5 rounded-full transition-all duration-200 relative flex items-center px-0.5 cursor-pointer {instanceEnableVulkanOpt ? 'bg-[#caa97c] shadow-[0_0_10px_rgba(202,169,124,0.35)]' : 'bg-[#2d2e34]'}"
									onclick={() => instanceEnableVulkanOpt = !instanceEnableVulkanOpt}
								>
									<span class="w-4 h-4 rounded-full bg-white transition-transform duration-200 shadow-md {instanceEnableVulkanOpt ? 'translate-x-5' : 'translate-x-0'}"></span>
								</button>
							</div>
						</div>

					{:else if activeInstanceSection === 'janela'}
						<div class="space-y-6">
							<div>
								<h3 class="text-xs font-bold text-white uppercase tracking-wider">Janela & Display</h3>
								<p class="text-[11px] text-white/40 mt-0.5">Dimensões da janela e modo de exibição do Minecraft</p>
							</div>

							<div class="grid grid-cols-2 gap-4">
								<div class="space-y-1.5">
									<span class="text-xs font-bold text-white/70 block">Largura (px)</span>
									<input type="number" bind:value={instanceWindowWidth} class="w-full bg-[#1c1d22] border border-white/10 rounded-2xl px-4 py-2 text-xs text-white font-mono outline-none" />
								</div>
								<div class="space-y-1.5">
									<span class="text-xs font-bold text-white/70 block">Altura (px)</span>
									<input type="number" bind:value={instanceWindowHeight} class="w-full bg-[#1c1d22] border border-white/10 rounded-2xl px-4 py-2 text-xs text-white font-mono outline-none" />
								</div>
							</div>

							<div class="flex items-center justify-between bg-[#1c1d22] border border-white/5 rounded-2xl p-4">
								<div>
									<span class="text-xs font-bold text-white block">Iniciar em Tela Cheia (Fullscreen)</span>
									<span class="text-[10px] text-white/40 block mt-0.5">Abre o Minecraft ocupando todo o monitor nativamente</span>
								</div>
								<button 
									type="button"
									aria-label="Alternar Tela Cheia"
									class="w-10 h-5 rounded-full transition-all duration-200 relative flex items-center px-0.5 cursor-pointer {instanceStartFullscreen ? 'bg-brand-500 shadow-[0_0_10px_rgba(226,184,107,0.35)]' : 'bg-[#2d2e34]'}"
									onclick={() => instanceStartFullscreen = !instanceStartFullscreen}
								>
									<span class="w-4 h-4 rounded-full bg-white transition-transform duration-200 shadow-md {instanceStartFullscreen ? 'translate-x-5' : 'translate-x-0'}"></span>
								</button>
							</div>
						</div>

					{:else if activeInstanceSection === 'java'}
						<div class="space-y-6">
							<div>
								<h3 class="text-xs font-bold text-white uppercase tracking-wider">Java e Memória</h3>
								<p class="text-[11px] text-white/40 mt-0.5">Alocação de RAM, presets rápidos e validação de flags JVM</p>
							</div>

							<div class="flex items-center justify-between bg-[#1c1d22] border border-[#caa97c]/30 rounded-2xl p-4">
								<div>
									<span class="text-xs font-bold text-[#caa97c] block">Luxmc Vulkan Zero-Lag Optimizer</span>
									<span class="text-[10px] text-white/40 block mt-0.5">Otimização própria de renderização Mesa Zink e flags G1GC sem bugs visuais</span>
								</div>
								<button 
									type="button"
									aria-label="Alternar Otimização Vulkan"
									class="w-10 h-5 rounded-full transition-all duration-200 relative flex items-center px-0.5 cursor-pointer {instanceEnableVulkanOpt ? 'bg-[#caa97c] shadow-[0_0_10px_rgba(202,169,124,0.35)]' : 'bg-[#2d2e34]'}"
									onclick={() => instanceEnableVulkanOpt = !instanceEnableVulkanOpt}
								>
									<span class="w-4 h-4 rounded-full bg-white transition-transform duration-200 shadow-md {instanceEnableVulkanOpt ? 'translate-x-5' : 'translate-x-0'}"></span>
								</button>
							</div>

							<!-- RAM Presets & Slider -->
							<div class="space-y-3 bg-[#18191f] border border-white/5 rounded-2xl p-4">
								<div class="flex items-center justify-between">
									<div class="flex items-center gap-2">
										<span class="text-xs font-bold text-white/80">Alocação de Memória RAM</span>
										<span class="text-[10px] text-white/40">(Sistema: {Math.round(systemRamMb / 1024)} GB)</span>
									</div>
									<span class="text-xs font-mono font-black text-[#caa97c] bg-[#caa97c]/10 px-2.5 py-0.5 rounded-lg border border-[#caa97c]/20">
										{instanceRamMb} MB ({(instanceRamMb / 1024).toFixed(1)} GB)
									</span>
								</div>

								<!-- Quick RAM Presets based on real system specs -->
								<div class="grid grid-cols-3 sm:grid-cols-5 gap-1.5 pt-1">
									{#each ramPresets as preset}
										<button 
											type="button"
											class="py-1.5 px-2 rounded-xl text-center border transition-all cursor-pointer {instanceRamMb === preset.mb ? 'bg-[#caa97c] text-black border-[#caa97c] font-black shadow-md' : 'bg-[#202128] text-white/70 border-white/5 hover:border-white/20 hover:text-white'}"
											onclick={() => instanceRamMb = preset.mb}
										>
											<div class="text-[11px] font-bold leading-tight">{preset.label}</div>
											<div class="text-[9px] opacity-60 leading-tight">{preset.desc}</div>
										</button>
									{/each}
								</div>

								<input 
									type="range" 
									min="1024" 
									max={Math.max(4096, Math.min(Math.floor(systemRamMb * 0.9 / 1024) * 1024, 32768))} 
									step="512" 
									bind:value={instanceRamMb} 
									class="w-full accent-[#caa97c] cursor-pointer mt-2" 
								/>
								<div class="flex justify-between text-[10px] font-mono text-white/40">
									<span>1024 MB (1 GB)</span>
									<span>{Math.round(systemRamMb * 0.45 / 1024) * 1024} MB</span>
									<span>{Math.max(4096, Math.min(Math.floor(systemRamMb * 0.9 / 1024) * 1024, 32768))} MB ({Math.round(Math.max(4096, Math.min(Math.floor(systemRamMb * 0.9 / 1024) * 1024, 32768)) / 1024)} GB)</span>
								</div>
							</div>

							<!-- Custom JVM Arguments with Live OS-Safe Validator -->
							<div class="space-y-2 bg-[#18191f] border border-white/5 rounded-2xl p-4">
								<div class="flex items-center justify-between">
									<span class="text-xs font-bold text-white/80">Argumentos JVM Customizados</span>
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
									class="w-full bg-[#14151a] border border-white/10 rounded-xl px-4 py-2.5 text-xs text-white font-mono outline-none focus:border-[#caa97c]" 
								/>

								{#if jvmValidation && !jvmValidation.valid}
									<div class="p-3 bg-amber-500/10 border border-amber-500/25 rounded-xl space-y-2 mt-2">
										<div class="text-[11px] font-bold text-amber-300">
											Flags rejeitadas para Linux: {jvmValidation.rejected.join(", ")}
										</div>
										<ul class="text-[10px] text-white/70 space-y-1 list-disc pl-4">
											{#each jvmValidation.suggestions as sug}
												<li>{sug}</li>
											{/each}
										</ul>
										<button 
											type="button" 
											class="text-[10px] font-bold text-black bg-[#caa97c] hover:bg-[#d8bc98] px-3 py-1 rounded-lg transition-all cursor-pointer mt-1"
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

							<!-- Instance Maintenance & Backup Tools -->
							<div class="space-y-3 bg-[#18191f] border border-white/5 rounded-2xl p-4">
								<div>
									<h4 class="text-xs font-bold text-white/80">Manutenção & Backup da Instância</h4>
									<p class="text-[10px] text-white/40 mt-0.5">Verifique integridade de arquivos ou exporte backups com segurança</p>
								</div>

								<div class="grid grid-cols-3 gap-2 pt-1">
									<!-- Repair Button -->
									<button 
										type="button"
										class="p-3 rounded-xl bg-[#202128] hover:bg-[#282933] border border-white/5 hover:border-white/20 text-left transition-all cursor-pointer flex flex-col justify-between group disabled:opacity-50"
										onclick={handleRepairInstance}
										disabled={isRepairing}
									>
										<div class="flex items-center justify-between w-full">
											<RefreshCw class="w-4 h-4 text-[#caa97c] {isRepairing ? 'animate-spin' : 'group-hover:rotate-180 transition-transform duration-500'}" />
											{#if isRepairing}
												<span class="text-[9px] text-[#caa97c] font-bold">Reparando...</span>
											{/if}
										</div>
										<div class="mt-2">
											<p class="text-xs font-bold text-white">Reparar Instância</p>
											<p class="text-[10px] text-white/40">Checar SHA1 e baixar arquivos faltantes</p>
										</div>
									</button>

									<!-- Backup Saves -->
									<button 
										type="button"
										class="p-3 rounded-xl bg-[#202128] hover:bg-[#282933] border border-white/5 hover:border-white/20 text-left transition-all cursor-pointer flex flex-col justify-between group disabled:opacity-50"
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
											<p class="text-xs font-bold text-white">Backup dos Mundos</p>
											<p class="text-[10px] text-white/40">Compactar saves em arquivo .zip</p>
										</div>
									</button>

									<!-- Export Instance -->
									<button 
										type="button"
										class="p-3 rounded-xl bg-[#202128] hover:bg-[#282933] border border-white/5 hover:border-white/20 text-left transition-all cursor-pointer flex flex-col justify-between group disabled:opacity-50"
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
											<p class="text-xs font-bold text-white">Exportar Instância</p>
											<p class="text-[10px] text-white/40">Criar pacote completo .zip</p>
										</div>
									</button>
								</div>
							</div>
						</div>

					{:else if activeInstanceSection === 'hooks'}
						<div class="space-y-6">
							<div>
								<h3 class="text-xs font-bold text-white uppercase tracking-wider">Launch Hooks</h3>
								<p class="text-[11px] text-white/40 mt-0.5">Executar scripts pré e pós inicialização do Minecraft</p>
							</div>

							<div class="space-y-2">
								<span class="text-xs font-bold text-white/70 block">Script Pré-Inicialização (Pre-Launch)</span>
								<input type="text" bind:value={instancePreLaunchHook} placeholder="/path/to/script.sh" class="w-full bg-[#1c1d22] border border-white/10 rounded-2xl px-4 py-2.5 text-xs text-white font-mono outline-none" />
							</div>

							<div class="space-y-2">
								<span class="text-xs font-bold text-white/70 block">Script Pós-Encerramento (Post-Exit)</span>
								<input type="text" bind:value={instancePostExitHook} placeholder="/path/to/script.sh" class="w-full bg-[#1c1d22] border border-white/10 rounded-2xl px-4 py-2.5 text-xs text-white font-mono outline-none" />
							</div>
						</div>
					{/if}

				</div>
			</div>

			<!-- Footer Buttons -->
			<div class="flex items-center justify-end gap-3 pt-4 border-t border-white/5">
				<button 
					type="button"
					class="px-6 py-2.5 rounded-full text-xs font-bold text-white/60 hover:text-white hover:bg-white/5 transition-all cursor-pointer"
					onclick={() => showInstanceSettingsModal = false}
				>
					Cancelar
				</button>
				<button 
					type="button"
					class="px-7 py-2.5 rounded-full text-xs font-black text-black transition-all cursor-pointer shadow-lg hover:scale-105 active:scale-95 flex items-center gap-2"
					style="background-color: var(--accent-color, #e2b86b);"
					onclick={saveInstanceSettings}
				>
					<Check class="w-4 h-4 stroke-[3]" /> Guardar alterações
				</button>
			</div>

		</div>
	</div>
{/if}
