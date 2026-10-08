<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { backOut, quintOut } from "svelte/easing";
    import { Cpu, ImagePlus, X, LoaderCircle } from "lucide-svelte";
    import { loadFolderImages, saveFolderImage, prepareFolderImage } from "$lib/utils/folderImages";
    import LuxAccountForm from "$lib/components/profile/LuxAccountForm.svelte";
    let localProfile = $state(false);
    import { appState } from "$lib/stores/app.svelte";
	import { fade, slide, fly } from "svelte/transition";
	import { onMount } from "svelte";
    import { listenDownloadProgress } from "$lib/api/events";
	import { goto } from "$app/navigation";
	import { preloadRoute } from "$lib/utils/preloadRoute";
	import {
		Play,
		Plus,
		Clock,
		Users,
		ArrowRight,
		Gamepad2,
		Loader2,
		Lock,
		ExternalLink,
		CheckCircle2,
		FolderOpen,
		ChevronDown,
		ChevronUp,
		Search,
		Trash2,
		User,
		MoreVertical,
		Settings,
		Sparkles,
		Layers,
		ArrowLeft,
		ChevronRight,
		FolderPlus,
		SlidersHorizontal,
		Filter,
		ArrowUpDown,
		Home,
		Square
	} from "lucide-svelte";
	import GlassSelect from "$lib/components/ui/GlassSelect.svelte";
	import MicrosoftLogo from "$lib/components/ui/MicrosoftLogo.svelte";
	import RightSidebar from "$lib/components/layout/RightSidebar.svelte";
	import CreateInstanceModal from "$lib/components/instances/CreateInstanceModal.svelte";
	import { account, saveCurrentAccount, loadCurrentAccount } from "$lib/stores/account.svelte";
	import { profiles, type Profile } from "$lib/stores/profiles.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { getFullCapeDataUrl } from "$lib/utils/capeTextures";
	import { createSkinAvatar } from "$lib/utils/textureImage";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playClick, playSuccess } from "$lib/utils/sounds";
	import {
		authDevLogin,
		authOfflineLogin,
		authLogin,
		authGetClientId,
		authSetClientId,
		launchGame,
		stopGame,
		versionsCheckInstalled,
		versionsDownload,
		discordSetActivity,
		instancesOpenFolder,
        profilesUpdate,
		versionsList,
		api,
		optimizerInstallPerfPack
	} from "$lib/api";
	import { handlePostLaunchActions } from "$lib/utils/launcherLifecycle";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";

	const { t } = useTranslation();

	let loginTab = $state<"microsoft" | "offline">("microsoft");
	let offlineName = $state("");
	let isLoggingIn = $state(false);
	let isLoggingInMicrosoft = $state(false);
	let savedAccounts = $state<string[]>([]);
	let showMsClientIdModal = $state(false);
	let msClientIdInput = $state("");
	let isSavingClientId = $state(false);

	let searchQuery = $state("");
	let selectedGroup = $state<string | null>(null);
	let sortBy = $state<"lastPlayed" | "name" | "version">("lastPlayed");
	let jumpInExpanded = $state(true);
	let isLaunching = $derived(appState.isLaunching);
	let launchingProfileId = $derived(appState.launchingProfileId);
	let launchStatusText = $derived(appState.launchStatusText);
	let activeContextMenuId = $state<string | null>(null);

	let showCreateModal = $state(false);
	let availableVersions = $state<Array<{ id: string; versionType: string; releaseTime: string }>>([]);
	let versionsLoading = $state(false);
	let systemRamMb = $state(8192);

	let customGroups = $state<string[]>([]);
    let folderImages = $state<Record<string, string>>({});
    let folderImageInput = $state<HTMLInputElement | null>(null);
    let folderImageTarget = $state<string | null>(null);
    let folderImageBusy = $state(false);
    function folderImage(name: string): string { return Object.hasOwn(folderImages, name) ? folderImages[name] : ''; }
    function chooseFolderImage(name: string) {
        if (folderImageBusy) return;
        folderImageTarget = name;
        folderImageInput?.click();
    }
    async function uploadFolderImage(event: Event) {
        const input = event.currentTarget as HTMLInputElement;
        const file = input.files?.[0];
        const folder = folderImageTarget;
        input.value = '';
        if (!file || folder === null || folderImageBusy) return;
        folderImageBusy = true;
        try {
            const image = await prepareFolderImage(file);
            try { folderImages = saveFolderImage(folderImages, folder, image); }
            catch { toast(uiText('library.imageStorageError'), 'error'); return; }
            toast(uiText('library.imageSaved'), 'success');
        } catch (error) { toast(uiText(error instanceof Error && error.message === 'limit' ? 'library.imageLimit' : 'library.imageInvalid'), 'error'); }
        finally { folderImageBusy = false; }
    }
    function removeFolderImage(name: string) {
        try { folderImages = saveFolderImage(folderImages, name, null); toast(uiText('library.imageRemoved'), 'success'); }
        catch { toast(uiText('library.imageStorageError'), 'error'); }
    }
	let showNewGroupPrompt = $state(false);
	let newGroupName = $state("");
    let groupHover = $state<string | null>(null);
    let movingProfile = $state<string | null>(null);
    const libraryGroups = $derived(customGroups);
    const visibleFolders = $derived(selectedGroup === null ? libraryGroups.filter(name => !searchQuery || name.toLowerCase().includes(searchQuery.toLowerCase())) : []);
    function groupMembers(group: string) {
        return profiles.list.filter(profile => profile.group === group);
    }
    async function moveToGroup(profileId: string, group: string) {
        if (movingProfile || !profiles.list.some(profile => profile.id === profileId) || (group !== '' && !libraryGroups.includes(group))) return;
        movingProfile = profileId;
        try { await profilesUpdate({id: profileId, instanceGroup: group}); profiles.update(profileId, {group}); }
        catch (cause) { toast(String(cause), 'error'); }
        finally { movingProfile = null; groupHover = null; }
    }
    function dropIntoGroup(event: DragEvent, group: string) {
        event.preventDefault();
        const id = event.dataTransfer?.getData('application/x-luxmc-instance');
        if (id) void moveToGroup(id, group);
    }

	const tilePalette = $derived([
		{ bg: "bg-bg-subtle border border-fg/[0.08]", text: uiText("ui.6beab8bced7f50bb") },
		{ bg: "bg-bg-subtle border border-fg/[0.07]", text: uiText("ui.6beab8bced7f50bb") },
		{ bg: "bg-bg-subtle border border-fg/[0.08]", text: uiText("ui.6beab8bced7f50bb") },
		{ bg: "bg-bg-subtle border border-fg/[0.06]", text: uiText("ui.6beab8bced7f50bb") },
		{ bg: "bg-bg-subtle border border-fg/[0.07]", text: uiText("ui.6beab8bced7f50bb") },
		{ bg: "bg-bg-subtle border border-fg/[0.08]", text: uiText("ui.6beab8bced7f50bb") },
		{ bg: "bg-bg-subtle border border-fg/[0.07]", text: uiText("ui.6beab8bced7f50bb") },
		{ bg: "bg-bg-subtle border border-fg/[0.06]", text: uiText("ui.6beab8bced7f50bb") }
	]);

	function getInstanceTileColor(id: string): { bg: string; text: string } {
		let hash = 0;
		for (let i = 0; i < id.length; i++) hash = id.charCodeAt(i) + ((hash << 5) - hash);
		return tilePalette[Math.abs(hash) % tilePalette.length];
	}

    $effect(() => {
        if (!isLaunching) return;
        let disposed = false;
        let unsubscribe: (() => void) | undefined;
        void listenDownloadProgress(progress => {
            if (disposed) return;
            const labels: Record<string, string> = { assets: uiText("download.phaseAssets"), libraries: uiText("ui.6de2d03fd96e5b9b"), client: uiText("ui.595883002f54bbf0"), java: uiText("ui.6839bd679d968a68"), done: uiText("ui.62ccae77aeae6db8") };
            const label = labels[progress.phase] || uiText("ui.29e59575e38eaf03");
            const percent = progress.total > 0 ? Math.min(100, Math.round(progress.completed / progress.total * 100)) : 0;
            appState.launchStatusText = `${label} · ${percent}%`;
        }).then(unlisten => { if (disposed) unlisten(); else unsubscribe = unlisten; }).catch(() => {});
        return () => { disposed = true; unsubscribe?.(); };
    });

	onMount(() => {
        try {
            const folders = localStorage.getItem('luxmc_library_folders_v2');
            const saved = JSON.parse(folders ?? localStorage.getItem('luxmc_library_groups') ?? '[]');
            if (Array.isArray(saved)) customGroups = [...new Set(saved.filter((value): value is string => typeof value === 'string' && value.trim().length > 0 && value.length <= 64 && (folders !== null || !['Vanilla', 'Modded'].includes(value))))];
            localStorage.setItem('luxmc_library_folders_v2', JSON.stringify(customGroups));
            folderImages = loadFolderImages(customGroups);
        } catch {}
		const savedAccsRaw = localStorage.getItem("luxmc_saved_nicknames");
		if (savedAccsRaw) {
			try {
				savedAccounts = JSON.parse(savedAccsRaw);
			} catch {}
		} else {
			const legacyMap = localStorage.getItem("luxmc_offline_passwords");
			if (legacyMap) {
				try {
					savedAccounts = Object.keys(JSON.parse(legacyMap));
				} catch {}
			}
		}

		if (!account.value) {
			void loadCurrentAccount();
		}

		loadVersions();

		const handleClickOutside = () => {
			activeContextMenuId = null;
		};
		window.addEventListener("click", handleClickOutside);
		const refreshVersions = () => { if (!document.hidden) void loadVersions(); };
        const timer = setInterval(refreshVersions, 60000);
        window.addEventListener("focus", refreshVersions);
        return () => { window.removeEventListener("click", handleClickOutside); window.removeEventListener("focus", refreshVersions); clearInterval(timer); };
	});

	async function loadVersions() {
		try {
			const cached = localStorage.getItem("luxmc_cached_versions");
			if (cached) {
				const parsed = JSON.parse(cached);
				if (parsed?.versions?.length > 0) availableVersions = parsed.versions;
			}
		} catch {}
		try {
			const response = await versionsList();
			if (response?.versions?.length > 0) {
				availableVersions = response.versions;
				const payload = JSON.stringify(response);
				setTimeout(() => {
					try { localStorage.setItem("luxmc_cached_versions", payload); } catch {}
				}, 0);
			}
		} catch {}
	}

	function saveSavedAccounts(list: string[]) {
		savedAccounts = list;
		if (typeof window !== "undefined") {
			localStorage.setItem("luxmc_saved_nicknames", JSON.stringify(list));
		}
	}

	function deleteSavedAccount(name: string) {
		const updated = savedAccounts.filter(n => n.toLowerCase() !== name.toLowerCase());
		saveSavedAccounts(updated);
		toast(uiText("ui.e73a0a4974f82317", {arg0: (name)}), "info");
	}

	async function handleOfflineAuth(customNick?: string) {
		if (isLoggingIn || isLoggingInMicrosoft) return;
		const name = (customNick || offlineName).trim();
		if (!name) {
			toast(uiText("ui.2ea8d2dc1d25389a"), "error");
			return;
		}

		isLoggingIn = true;
		try {
			const backendAcc = await authOfflineLogin(name);
			const skinUrl = backendAcc?.skinUrl || `https://minotar.net/skin/${name}`;
			const newAcc = {
				id: backendAcc?.id || ("offline_" + Date.now()),
				username: name,
				uuid: backendAcc?.uuid || ("offline-" + name.toLowerCase()),
				minecraftToken: "",
				expiresAt: 0,
				skinUrl,
				skinVariant: "classic",
				capeUrl: null
			};

			void saveCurrentAccount(newAcc);
			activeSkinStore.setSkin({
				id: newAcc.uuid,
				name: newAcc.username,
				url: `https://mc-heads.net/body/${newAcc.username}/300`,
				skinUrl,
				avatarUrl: `https://mc-heads.net/avatar/${newAcc.username}/100`,
				type: "steve"
			});

			if (!savedAccounts.some(n => n.toLowerCase() === name.toLowerCase())) {
				saveSavedAccounts([name, ...savedAccounts]);
			}

			toast(`Bem-vindo, ${name}!`, "success");
			playSuccess();
			account.value = newAcc;
		} catch (e) {
			toast(uiText("ui.86b8a84c140df857") + String(e), "error");
		} finally {
			isLoggingIn = false;
		}
	}

	async function handleMicrosoftLogin() {
		if (isLoggingIn || isLoggingInMicrosoft) return;
		isLoggingInMicrosoft = true;
		try {
			const existingId = await authGetClientId().catch(() => "");
			if (!existingId || existingId === "00000000-0000-0000-0000-000000000000") {
				await authSetClientId("9750ebbe-21e9-4a4d-b808-f451a3e0af7f").catch(() => null);
			}
			await startMsLogin();
		} catch (e) {
			toast(String(e), "error");
		} finally {
			isLoggingInMicrosoft = false;
		}
	}

	async function startMsLogin() {
		isLoggingInMicrosoft = true;
		try {
			toast(uiText("ui.9e5f8df7532c38f7"), "info");
			const acc = await authLogin();
			const skinModel = acc.skinVariant?.toLowerCase() === "slim" ? "alex" : "steve";
			const skinUrl = acc.skinUrl || `https://minotar.net/skin/${acc.username}`;
			const initialAvatar = `https://mc-heads.net/avatar/${acc.username}/100`;

			const newAcc = {
				id: acc.id,
				username: acc.username,
				uuid: acc.uuid,
				minecraftToken: acc.accessToken || "",
				expiresAt: acc.expiresAt ? (acc.expiresAt < 1e11 ? acc.expiresAt * 1000 : acc.expiresAt) : 0,
				skinUrl: acc.skinUrl || null,
				skinVariant: acc.skinVariant || "classic",
				capeUrl: acc.capeUrl || null,
				avatarUrl: initialAvatar
			};

			void saveCurrentAccount(newAcc);
			activeSkinStore.setSkin({
				id: newAcc.uuid,
				name: newAcc.username,
				url: `https://mc-heads.net/body/${newAcc.username}/300`,
				skinUrl,
				avatarUrl: initialAvatar,
				type: skinModel,
				hasCape: Boolean(newAcc.capeUrl),
				capeType: newAcc.capeUrl ? "custom" : "none",
				customCapeUrl: newAcc.capeUrl || ""
			});

			if (skinUrl) {
				void createSkinAvatar(skinUrl, new AbortController().signal, skinModel)
					.then((avatar) => {
						if (avatar) {
							activeSkinStore.setSkin({ avatarUrl: avatar });
							if (account.value && account.value.id === newAcc.id) {
								const updated = { ...account.value, avatarUrl: avatar };
								account.value = updated;
								void saveCurrentAccount(updated);
							}
						}
					})
					.catch(() => {});
			}

			if (!savedAccounts.some(n => n.toLowerCase() === newAcc.username.toLowerCase())) {
				saveSavedAccounts([newAcc.username, ...savedAccounts]);
			}

			toast(uiText("ui.d4d05ee9d7210443", {arg0: (newAcc.username)}), "success");
			playSuccess();
			account.value = newAcc;
		} catch (e) {
			toast(uiText("ui.03393fa87c1c9fd7") + String(e), "error");
		} finally {
			isLoggingInMicrosoft = false;
		}
	}

	async function saveAndLoginWithClientId() {
		const val = msClientIdInput.trim();
		if (!val) {
			toast(uiText("ui.45642234f6e7ced3"), "error");
			return;
		}
		isSavingClientId = true;
		try {
			await authSetClientId(val);
			showMsClientIdModal = false;
			toast(uiText("ui.69958948b529e62f"), "success");
			await startMsLogin();
		} catch (e) {
			toast(uiText("ui.7fa73e80e61671ca") + String(e), "error");
		} finally {
			isSavingClientId = false;
		}
	}

	async function handleDevLogin() {
		if (isLoggingIn || isLoggingInMicrosoft) return;
		isLoggingIn = true;
		try {
			const devAcc = await authDevLogin();
			const newAcc = {
				id: devAcc.id,
				username: devAcc.username,
				uuid: devAcc.uuid,
				minecraftToken: devAcc.accessToken,
				expiresAt: devAcc.expiresAt ? (devAcc.expiresAt < 1e11 ? devAcc.expiresAt * 1000 : devAcc.expiresAt) : 0
			};
			void saveCurrentAccount(newAcc);
			toast(`Modo Dev Ativo: ${devAcc.username}`, "info");
			playSuccess();
			account.value = newAcc;
		} catch (e) {
			toast(uiText("ui.d4d9fa06ecf42be8") + String(e), "error");
		} finally {
			isLoggingIn = false;
		}
	}

	async function handleLaunch(targetProfile: Profile, serverIp?: string, serverPort?: number) {
		if (appState.isLaunching) return;
		appState.isLaunching = true;
		appState.launchingProfileId = targetProfile.id;
		appState.launchStatusText = serverIp ? uiText("ui.31b4e9ae9f49ca91", {arg0: (serverIp)}) : "Iniciando...";

		try {
			let userUuid = account.value?.uuid;
			if (!userUuid) {
				const devAcc = await authDevLogin();
				userUuid = devAcc.uuid;
			}

			const verId = targetProfile.mcVersion || "1.20.4";
			appState.launchStatusText = uiText("ui.778d3006d213fe60");

			const installed = await versionsCheckInstalled(verId).catch(() => false);
			if (!installed) {
				appState.launchStatusText = uiText("ui.1d9893e36eabb20a", {arg0: (verId)});
				await versionsDownload(verId);
			}

			appState.launchStatusText = serverIp ? uiText("ui.31b4e9ae9f49ca91", {arg0: (serverIp)}) : "Iniciando Minecraft...";
			const globalVulkan = typeof window !== "undefined" && localStorage.getItem("luxmc_enable_vulkan") === "true";
			const isVulkan = globalVulkan || targetProfile.useVulkan === true;
			const skinToPass = activeSkinStore.current.skinUrl || account.value?.skinUrl || null;
			const effectiveCape = activeSkinStore.current.hasCape
				? (activeSkinStore.current.customCapeUrl || (activeSkinStore.current.capeType && activeSkinStore.current.capeType !== "none" ? getFullCapeDataUrl(activeSkinStore.current.capeType) : null))
				: (account.value?.capeUrl || null);

			const result = await launchGame({
				versionId: verId,
				accountId: userUuid || "",
				profileId: targetProfile.id,
				enableVulkan: isVulkan,
				skinUrl: skinToPass,
				skinVariant: activeSkinStore.current.type === "alex" ? "slim" : "classic",
				capeUrl: effectiveCape,
				serverIp: serverIp || null,
				serverPort: serverPort || null
			});

			gamingStats.onGameStart(targetProfile.id);
            appState.isGameRunning = true;
            appState.activeGameDetails = { profileId: targetProfile.id, name: targetProfile.name, version: verId, loader: targetProfile.loader };
			profiles.setLastPlayed(targetProfile.id);

			discordSetActivity({
				inGame: true,
				details: targetProfile.name,
				state: `Minecraft ${verId} · ${targetProfile.loader.toUpperCase()}`,
				largeText: targetProfile.name,
				largeImage: targetProfile.icon || "grass",
				smallImage: "grass",
				smallText: `Luxmc · ${targetProfile.loader.toUpperCase()}`,
				startTime: Math.floor(Date.now() / 1000),
				buttons: [
					{ label: uiText("ui.4c8757fab2a21345"), url: "https://luxmc-r92.pages.dev" },
					{ label: uiText("ui.d4fb4e24ae3c7a8f"), url: "https://luxmc-r92.pages.dev" }
				]
			}).catch(() => {});

			toast(`🎮 Minecraft ${verId} iniciado! (PID: ${result.pid})`, "success");
			void handlePostLaunchActions();
		} catch (e) {
			toast(uiText("ui.5a696268f07e8812") + String(e), "error");
		} finally {
			appState.isLaunching = false;
			appState.launchingProfileId = null;
			appState.launchStatusText = "";
		}
	}

	async function handleStopGame() {
		try {
			appState.isStopping = true;
			appState.wasManuallyTerminated = true;
			await stopGame();
			appState.isGameRunning = false;
			gamingStats.onGameExit();
			toast(uiText("ui.69429e8fd4528906"), "info");
		} catch (e) {
			appState.isGameRunning = false;
			gamingStats.onGameExit();
			toast(uiText("ui.f635cd0acfe6def9") + String(e), "error");
		} finally {
			appState.isStopping = false;
		}
	}

	async function handleCreateInstance(input: {
		name: string; version: string; loader: string; icon: string;
		ramGb: number; autoOptimize: boolean; useVulkan: boolean; installPerfPack: boolean;
		resolutionW?: number; resolutionH?: number; fullscreen?: boolean;
		javaPath?: string; jvmArgs?: string; gameDir?: string;
	}) {
		try {
			const p = await api.invoke<{
				id: string; name: string; icon: string; mcVersion: string;
				loader: string; loaderVersion: string | null; gameDir: string;
				resolutionW: number | null; resolutionH: number | null; fullscreen: boolean;
				javaPath: string | null; jvmArgs: string | null;
				createdAt: string; updatedAt: string;
			}>("profiles_create", {
				input: {
					name: input.name, mcVersion: input.version, loader: input.loader,
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
				try { await optimizerInstallPerfPack(p.id); } catch {}
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

			showCreateModal = false;
			toast(uiText("ui.1b6566167e91c704", {arg0: (input.name)}), "success");
		} catch (e) {
			toast(uiText("ui.e9e569706d7fbe7a") + String(e), "error");
		}
	}

	function handleAddGroup() {
		const name = newGroupName.trim().slice(0,64);
		if (!name) return;
		if (!customGroups.includes(name)) {
			customGroups = [...customGroups, name];
            localStorage.setItem("luxmc_library_folders_v2", JSON.stringify(customGroups));
			toast(uiText("ui.4e66ec1ac77df784", {arg0: (name)}), "success");
		}
		newGroupName = "";
        selectedGroup = null;
		showNewGroupPrompt = false;
	}

	function getLoaderBadgeColor(loader: string) {
		const l = (loader || "").toLowerCase();
		if (l.includes("fabric")) return "bg-sky-500/15 text-sky-400 border-sky-500/30";
		if (l.includes("neoforge")) return "bg-orange-500/15 text-orange-400 border-orange-500/30";
		if (l.includes("forge")) return "bg-amber-500/15 text-amber-400 border-amber-500/30";
		if (l.includes("quilt")) return "bg-purple-500/15 text-purple-400 border-purple-500/30";
		return "bg-emerald-500/15 text-emerald-400 border-emerald-500/30";
	}

	const filteredProfiles = $derived(
		profiles.list
			.filter(p => {
				const matchesSearch = !searchQuery || p.name.toLowerCase().includes(searchQuery.toLowerCase()) || p.mcVersion.includes(searchQuery);
				const matchesGroup = selectedGroup === null ? (!!searchQuery || !p.group || !libraryGroups.includes(p.group)) : p.group === selectedGroup;
				return matchesSearch && matchesGroup;
			})
			.sort((a, b) => {
				if (sortBy === "lastPlayed") {
					return (b.lastPlayed || 0) - (a.lastPlayed || 0);
				}
				if (sortBy === "name") {
					return a.name.localeCompare(b.name);
				}
				return b.mcVersion.localeCompare(a.mcVersion);
			})
	);

	const jumpInInstances = $derived(
		[...profiles.list]
			.sort((a, b) => (b.lastPlayed || 0) - (a.lastPlayed || 0))
			.slice(0, 3)
	);
</script>

{#if !account.value}

	<div
		class="relative min-h-[calc(100dvh-2rem)] w-full flex items-center justify-center gap-14 p-6 select-none overflow-hidden"
	>
		<div class="absolute inset-0 bg-[radial-gradient(ellipse_80%_80%_at_50%_-20%,rgb(var(--brand-500)/0.18),transparent)] pointer-events-none"></div>
		<div class="absolute -top-40 left-1/2 -translate-x-1/2 w-[700px] h-[350px] bg-[radial-gradient(ellipse_at_center,rgb(var(--brand-500)/0.12),transparent_70%)] rounded-full pointer-events-none"></div>

        <aside class="relative z-10 hidden w-full max-w-sm flex-col gap-7 lg:flex">
            <img src="/logo.png" alt="Luxmc" class="h-28 w-28 object-contain" />
            <div><p class="mb-4 font-sans text-xs font-semibold uppercase tracking-[.18em] text-brand-400">Luxmc / Launcher</p><h2 class="font-sans text-5xl font-semibold leading-[1.08] tracking-tight text-fg">{uiText('loginDesign.headline')}</h2><p class="mt-6 font-sans text-sm leading-relaxed text-fg-muted">{uiText('loginDesign.story')}</p></div>
            <div class="flex items-center gap-3 border-t border-border pt-6 text-xs text-fg-muted"><Layers class="h-4 w-4 text-brand-400" />Vanilla · Fabric · Quilt · Forge · NeoForge</div>
        </aside>
		<div
			class="auth-surface w-full max-w-[520px] bg-bg-elevated border border-border rounded-3xl p-6 sm:p-8 relative z-10 space-y-6 shadow-elevated"
			in:fly={{ easing: backOut, y: 20, duration: 260 }}
		>
			<div class="flex flex-col items-center text-center space-y-3">
				<div class="w-16 h-16 rounded-2xl bg-gradient-to-b from-bg-subtle to-bg-elevated border border-fg/15 p-2.5 shadow-2xl flex items-center justify-center relative group">
					<div class="absolute inset-0 bg-blue-500/15 rounded-2xl pointer-events-none group-hover:bg-blue-500/25 transition-[color,background-color,border-color,box-shadow,transform,opacity]"></div>
					<img loading="lazy" decoding="async" src="/logo.png" alt="Luxmc" class="w-full h-full object-contain relative z-10" />
				</div>
				<div>
					<h1 class="text-xl font-black text-fg tracking-tight">{uiText("ui.d3094029aa08ec2c")}</h1>
					<p class="text-sm text-fg/60 mt-2 leading-relaxed">{uiText("design.loginDescription")}</p>
				</div>
			</div>

            <div class="grid grid-cols-3 gap-2" aria-label={uiText('loginDesign.method')}>
                {#each ['microsoft','luxmc','offline'] as method}
                    {@const selected = method === 'microsoft' ? loginTab === 'microsoft' : loginTab === 'offline' && localProfile === (method === 'offline')}
                    <button type="button" disabled={isLoggingIn || isLoggingInMicrosoft} aria-label={method === 'microsoft' ? 'Microsoft' : method === 'luxmc' ? 'Luxmc' : uiText('loginDesign.offline')} aria-pressed={selected} onclick={() => { loginTab = method === 'microsoft' ? 'microsoft' : 'offline'; localProfile = method === 'offline'; }} class="flex min-w-0 flex-col items-start gap-3 rounded-2xl border p-3 text-left {selected ? 'border-brand-400/50 bg-brand-500/10' : 'border-border bg-bg-subtle hover:border-fg/25'}">
                        {#if method === 'microsoft'}<MicrosoftLogo size={19} />{:else if method === 'luxmc'}<img src="/logo.png" alt="" class="h-6 w-6 object-contain" />{:else}<User class="h-5 w-5 text-fg-muted" />{/if}<span class="font-sans text-xs font-semibold text-fg">{method === 'microsoft' ? 'Microsoft' : method === 'luxmc' ? 'Luxmc' : uiText('loginDesign.offline')}</span>
                    </button>
                {/each}
            </div>

			{#if loginTab === "microsoft"}
				<div class="flex flex-col items-center text-center space-y-5 pt-1" in:fade={{ easing: quintOut, duration: 220 }}>
					<div class="space-y-1">
						<h2 class="text-base font-bold text-fg">{uiText("ui.e5b61bd78d85758f")}</h2>
						<p class="text-xs text-fg/50 max-w-xs leading-relaxed">
							{uiText("ui.95d14a94c1dc9be5")}
						</p>
					</div>

					{#if isLoggingInMicrosoft}
						<div role="status" aria-live="polite" class="w-full bg-bg/35 border border-brand-400/25 rounded-2xl p-6 text-left space-y-4">
							<div class="flex items-center justify-between">
								<MicrosoftLogo size={28} />
								<Loader2 class="w-5 h-5 text-brand-400 animate-spin motion-reduce:animate-none" />
							</div>

							<div class="space-y-1">
								<p class="text-sm font-semibold text-fg">{uiText("design.browserLogin")}</p>
								<p class="text-xs text-fg/60 leading-relaxed">
									{uiText("design.browserLoginHint")}
								</p>
							</div>

						</div>
					{:else}
						<div class="w-full space-y-2">
							<button
								type="button"
								class={launcherButton({ variant: "microsoft", size: "hero", class: "w-full" })}
								onclick={() => { playClick(); handleMicrosoftLogin(); }}
								disabled={isLoggingIn || isLoggingInMicrosoft}
							>
								<MicrosoftLogo size={20} />
								<span>{uiText("auth.signIn")}</span>
							</button>
							<span class="text-[11px] text-fg/40 font-medium block">{uiText("ui.9d92ae200eb167d0")}</span>
						</div>
					{/if}


				</div>

			{:else}

				{#if !localProfile}
                    <LuxAccountForm bind:busy={isLoggingIn} />
                    <button type="button" class={launcherButton({ variant: "ghost", size: "sm", class: "" })} disabled={isLoggingIn} onclick={() => localProfile = true}>{uiText("ui.8d82fb65b8a91eab")}</button>
                {:else}
                    <button type="button" class={launcherButton({ variant: "ghost", size: "sm", class: "" })} disabled={isLoggingIn} onclick={() => localProfile = false}>{uiText("ui.0cf9a9775845e7f8")}</button>
				<div class="space-y-4 pt-1" in:fade={{ easing: quintOut, duration: 220 }}>
					<div class="text-center space-y-1">
						<h2 class="text-base font-bold text-fg">{uiText("ui.d90b39bcf2e9d5be")}</h2>
						<p class="text-xs text-fg/50">{uiText("ui.1e6ebd3ae27cc1c6")}</p>
					</div>

					<div class="space-y-1.5">
						<label for="offline-nick-input" class="text-[11px] font-bold text-fg/60 block text-left">
							{uiText("ui.2e2972baf919c7b6")}
						</label>
						<div class="relative flex items-center">
							<div class="w-7 h-7 rounded-lg bg-bg-overlay/40 border border-fg/10 overflow-hidden absolute left-2.5 flex items-center justify-center pointer-events-none">
								<img loading="lazy" decoding="async"
									src={`https://mc-heads.net/avatar/${offlineName.trim() || 'Steve'}/32`}
									alt={uiText("ui.ca8e826d9c2ec401")}
									class="w-full h-full object-contain"
								/>
							</div>
							<input
								id="offline-nick-input"
								type="text"
								placeholder={uiText("ui.44c104681ef49837")}
								bind:value={offlineName}
								maxlength="16"
								class="w-full bg-bg-overlay/40 border border-fg/10 focus:border-blue-500/60 rounded-2xl pl-12 pr-4 py-3 text-xs font-bold text-fg outline-none transition-[color,background-color,border-color,box-shadow,transform,opacity] placeholder:text-fg/20"
								onkeydown={(e) => { if (e.key === "Enter") handleOfflineAuth(); }}
							/>
						</div>
					</div>

					<button
						type="button"
						class={launcherButton({ variant: "primary", size: "lg", class: "w-full uppercase tracking-wider flex items-center justify-center gap-2 disabled:opacity-50" })}
						onclick={() => { playClick(); handleOfflineAuth(); }}
						disabled={isLoggingIn || isLoggingInMicrosoft}
					>
						{#if isLoggingIn}
							<Loader2 class="w-4 h-4 animate-spin text-fg" /> {uiText("ui.a89d514549530e6e")}
						{:else}
							<Play class="w-3.5 h-3.5 fill-current" /> {uiText("home.offlineCta")}
						{/if}
					</button>

					{#if savedAccounts.length > 0}
						<div class="space-y-2 pt-1 border-t border-fg/5">
							<span class="text-[10px] font-bold text-fg/40 uppercase tracking-wider block text-left">
								{uiText("ui.e7f8cb3ab06ffcd0")}
							</span>
							<div class="flex flex-wrap gap-1.5">
								{#each savedAccounts as accName (accName)}
									<div class="inline-flex items-center gap-1.5 pl-1.5 pr-2 py-1 rounded-xl bg-fg/5 hover:bg-fg/10 border border-fg/5 transition-[color,background-color,border-color,box-shadow,transform,opacity] group">
										<button
											type="button"
											onclick={() => handleOfflineAuth(accName)}
											class="flex items-center gap-1.5 text-left cursor-pointer"
										>
											<img loading="lazy" decoding="async"
												src={`https://mc-heads.net/avatar/${accName}/32`}
												alt={accName}
												class="w-4 h-4 rounded object-cover"
											/>
											<span class="text-xs font-bold text-fg/80 group-hover:text-blue-300 transition-colors">{accName}</span>
										</button>
										<button
											type="button"
											onclick={() => deleteSavedAccount(accName)}
											class={launcherButton({ variant: "danger", size: "icon", class: "opacity-0 group-hover:opacity-100" })}
											title={uiText("mods.remove")}
										>
											<Trash2 class="w-2.5 h-2.5" />
										</button>
									</div>
								{/each}
							</div>
						</div>
					{/if}
				</div>

				{/if}
			{/if}

			<div class="pt-2 border-t border-fg/5 flex flex-col items-center gap-2">
				<button
					type="button"
					onclick={() => openUrl("https://luxmc-r92.pages.dev/#skin-studio")}
					class={launcherButton({ variant: "ghost", size: "sm", class: "flex items-center gap-1" })}
				>
					{uiText("ui.24f0ab7c60925e32")} <ExternalLink class="w-3 h-3" />
				</button>

				<div class="flex items-center justify-between w-full pt-1">
					<button
						type="button"
						class={launcherButton({ variant: "ghost", size: "sm", class: "" })}
						onclick={() => showMsClientIdModal = true}
					>
						{uiText("ui.563dfb14d03e1d7f")}
					</button>
					<button
						type="button"
						class={launcherButton({ variant: "ghost", size: "sm", class: "" })}
						onclick={handleDevLogin}
					>
						{uiText("ui.ab7d9b0dcfe1bfa6")}
					</button>
				</div>
			</div>
		</div>
	</div>

	{#if showMsClientIdModal}
		<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/80 backdrop-blur-sm" in:fade={{ easing: quintOut, duration: 220 }}>
			<div class="w-full max-w-md rounded-3xl bg-bg/35 backdrop-blur-xl border border-amber-500/30 p-6 shadow-2xl space-y-4" in:fly={{ easing: backOut, y: 20, duration: 260 }}>
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2.5">
						<Lock class="w-4 h-4 text-amber-400" />
						<h3 class="text-sm font-bold text-fg">{uiText("ui.8321c188043638f2")}</h3>
					</div>
					<button
						type="button"
						class={launcherButton({ variant: "ghost", size: "sm", class: "" })}
						onclick={() => showMsClientIdModal = false}
					>
						✕
					</button>
				</div>

				<p class="text-xs text-fg/60 leading-relaxed">
					{uiText("ui.99f83892f7beaf79")} <span class="text-amber-400 font-bold">{uiText("ui.8726db013948f070")}</span> {uiText("ui.5100f327ab279e1d")} <code class="text-amber-300">http://localhost:8453/callback</code>).
				</p>

				<input
					type="text"
					placeholder={uiText("ui.8fc44ad7924e6bda")}
					bind:value={msClientIdInput}
					class="w-full bg-bg-overlay/40 border border-fg/10 rounded-xl px-4 py-2.5 text-xs text-fg font-mono outline-none focus:border-amber-500/50"
				/>

				<div class="flex items-center gap-2 pt-1">
					<button
						type="button"
						class={launcherButton({ variant: "secondary", size: "sm", class: "flex-1" })}
						onclick={() => showMsClientIdModal = false}
					>
						{uiText("common.cancel")}
					</button>
					<button
						type="button"
						class={launcherButton({ variant: "secondary", size: "sm", class: "flex-1" })}
						onclick={saveAndLoginWithClientId}
						disabled={isSavingClientId}
					>
						{uiText("ui.c967d69697ef24b8")}
					</button>
				</div>
			</div>
		</div>
	{/if}

{:else}

	<div class="flex min-h-full w-full select-none" in:fade={{ easing: quintOut, duration: 220 }}>

		<div class="flex-1 flex flex-col min-w-0 space-y-6 p-6">

			<header class="flex items-center justify-between gap-4 py-1">
				<div class="flex items-center gap-3">
					<div class="flex items-center gap-1 bg-bg/35 backdrop-blur-xl border border-fg/[0.08] rounded-xl p-1 shadow-sm">
						<button
							type="button"
							class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
							title={uiText("common.back")}
							onclick={() => history.back()}
						>
							<ArrowLeft class="w-3.5 h-3.5" />
						</button>
						<button
							type="button"
							class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
							title={uiText("ui.859e48c3c9333657")}
							onclick={() => history.forward()}
						>
							<ChevronRight class="w-3.5 h-3.5" />
						</button>
					</div>

					<div class="flex items-center gap-2 text-xs font-bold text-fg/80">
						<Home class="w-3.5 h-3.5 text-fg/60" />
						<span class="text-fg font-extrabold">{t("nav.home")}</span>
					</div>
				</div>

				<div class="flex items-center gap-2">
					<div class="flex items-center gap-2 px-3 py-1.5 rounded-xl bg-bg/35 backdrop-blur-xl border border-fg/[0.08] text-[11px] text-fg/70 shadow-sm">
						{#if isLaunching}
							<span class="w-2 h-2 rounded-full bg-amber-400 shadow-[0_0_6px_rgba(251,191,36,0.8)]"></span>
							<span class="text-amber-300 font-bold">{launchStatusText || "Iniciando..."}</span>
						{:else if appState.isGameRunning}
							<span class="w-2 h-2 rounded-full bg-brand-500 shadow-[0_0_6px_rgba(59,130,246,0.8)]"></span>
							<span class="text-brand-500 font-bold">{uiText("ui.476e526adee711e8")} {appState.activeGameDetails?.name || "Minecraft"}</span>
						{:else}
							<span class="w-2 h-2 rounded-full bg-fg/30"></span>
							<span>{t("library.noneRunning")}</span>
						{/if}
					</div>
					{#if appState.isGameRunning}
						<button
							type="button"
							disabled={appState.isStopping}
							onclick={handleStopGame}
							class={launcherButton({ variant: "danger", size: "sm", class: "flex items-center gap-1.5 disabled:opacity-50" })}
							title={uiText("ui.6e69939266949462")}
						>
							{#if appState.isStopping}
								<Loader2 class="w-3.5 h-3.5 animate-spin" />
								<span>{uiText("ui.ea90df15a5385905")}</span>
							{:else}
								<Square class="w-3.5 h-3.5 fill-current" />
								<span>{uiText("ui.e4027f055ce88dd0")}</span>
							{/if}
						</button>
					{/if}
				</div>
			</header>

			<section class="space-y-3">
				<div class="flex items-center justify-between">
					<button
						type="button"
						onclick={() => jumpInExpanded = !jumpInExpanded}
						class={launcherButton({ variant: "ghost", size: "sm", class: "flex items-center gap-2 uppercase tracking-wider" })}
					>
						<span>{t("library.jumpIn")}</span>
						{#if jumpInExpanded}
							<ChevronUp class="w-4 h-4 text-fg/40" />
						{:else}
							<ChevronDown class="w-4 h-4 text-fg/40" />
						{/if}
					</button>
				</div>

				{#if jumpInExpanded}
					{#if jumpInInstances.length === 0}
						<div class="p-6 rounded-2xl bg-bg/35 backdrop-blur-xl border border-fg/[0.06] text-center space-y-2">
							<p class="text-xs text-fg/40">{uiText("ui.88282960c9bdaa99")}</p>
							<button
								type="button"
								onclick={() => showCreateModal = true}
								class={launcherButton({ variant: "primary", size: "sm", class: "" })}
							>
								{uiText("ui.8f652a01b5f78123")}
							</button>
						</div>
					{:else}
						<div class="space-y-2.5" transition:slide={{ easing: quintOut, duration: 240 }}>
							{#each jumpInInstances as inst (inst.id)}
								{@const tileCol = getInstanceTileColor(inst.id || inst.name)}
								<div class="flex flex-wrap items-center justify-between gap-3 p-3 rounded-2xl bg-bg/35 hover:bg-fg/10 backdrop-blur-xl border border-fg/[0.06] hover:border-fg/[0.14] transition-[color,background-color,border-color,box-shadow,transform,opacity] group shadow-sm ">
									<div class="flex flex-1 items-center gap-3.5 min-w-[220px]">
										<div class="w-12 h-12 rounded-xl overflow-hidden shrink-0 flex items-center justify-center shadow-inner {inst.icon && !inst.icon.includes('grass_block') ? 'bg-black/40 border border-fg/10' : tileCol.bg}">
											{#if inst.icon && inst.icon !== '/grass_block.png' && !inst.icon.includes('grass_block')}
												<img loading="lazy" decoding="async"
													src={inst.icon}
													alt={inst.name}
													class="w-full h-full object-contain"
													onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/grass_block.png'; (e.currentTarget as HTMLImageElement).className = 'w-10 h-10 object-contain [image-rendering:pixelated] drop-shadow'; }}
												/>
											{:else}
												<img loading="lazy" decoding="async"
													src="/grass_block.png"
													alt={inst.name}
													class="w-10 h-10 object-contain [image-rendering:pixelated] drop-shadow"
												/>
											{/if}
										</div>

										<div class="min-w-0 space-y-0.5">
											<div class="flex flex-wrap items-center gap-2">
												<h3 class="text-sm font-bold text-fg truncate leading-tight group-hover:text-brand-500 transition-colors">
													{inst.name}
												</h3>
												{#if !inst.lastPlayed}
													<span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-extrabold bg-brand-500/15 text-brand-500 border border-brand-500/30">
														<Sparkles class="w-2.5 h-2.5" /> {t("library.newInstanceBadge")}
													</span>
												{/if}
											</div>

											<div class="flex flex-wrap items-center gap-2 text-[11px] text-fg/50">
												<span class="font-semibold text-fg/70">
													{inst.loader} {inst.mcVersion}
												</span>
												<span>•</span>
												{#if inst.lastPlayed}
													<span>{uiText("ui.9183147acd1e7a10")}</span>
												{:else}
													<span>{t("library.neverPlayed")}</span>
												{/if}
											</div>
										</div>
									</div>

									<div class="ml-auto flex items-center gap-2 shrink-0">
										{#if isLaunching && launchingProfileId === inst.id}
											<button
												type="button"
												disabled
												class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-1.5 opacity-70" })}
											>
												<Loader2 class="w-3.5 h-3.5 animate-spin text-brand-foreground" />
												<span aria-live="polite">{launchStatusText || "Iniciando…"}</span>
											</button>
										{:else if appState.isGameRunning && appState.activeGameDetails?.profileId === inst.id}
											<button
												type="button"
												disabled={appState.isStopping}
												onclick={handleStopGame}
												class={launcherButton({ variant: "danger", size: "sm", class: "flex items-center gap-1.5 disabled:opacity-50" })}
												title={uiText("ui.e4027f055ce88dd0")}
											>
												{#if appState.isStopping}
													<Loader2 class="w-3.5 h-3.5 animate-spin" />
													<span>{uiText("ui.ea90df15a5385905")}</span>
												{:else}
													<Square class="w-3.5 h-3.5 fill-current" />
													<span>{uiText("ui.d36c5e504eb7c04c")}</span>
												{/if}
											</button>
										{:else}
											{@const isThisLaunching = (isLaunching || appState.isLaunching) && (launchingProfileId === inst.id || appState.launchingProfileId === inst.id)}
											<button
												type="button"
												onclick={() => handleLaunch(inst)}
												disabled={isLaunching || appState.isLaunching}
												class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-1.5 disabled:opacity-50" })}
												title={isThisLaunching ? (appState.launchStatusText || uiText("ui.dc0546b3e22c8f9e")) : uiText("instances.play")}
											>
												{#if isThisLaunching}
													<Loader2 class="w-3.5 h-3.5 animate-spin" />
													<span>{appState.launchStatusText || uiText("ui.dc0546b3e22c8f9e")}</span>
												{:else}
													<Play class="w-3.5 h-3.5 fill-current" />
													<span>{uiText("instances.play")}</span>
												{/if}
											</button>
										{/if}

										<div class="relative">
											<button
												type="button"
												onclick={(e) => {
													e.stopPropagation();
													activeContextMenuId = activeContextMenuId === inst.id ? null : inst.id;
												}}
												aria-label={uiText("ui.1bf82bb374333f9d", {arg0: (inst.name)})}
                                                aria-expanded={activeContextMenuId === inst.id}
                                                class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
											>
												<MoreVertical class="w-4 h-4" />
											</button>

											{#if activeContextMenuId === inst.id}
												<div
													class="home-instance-menu absolute right-0 top-full z-50 mt-2 flex w-56 flex-col gap-1 rounded-2xl border border-border bg-bg-elevated p-2 shadow-elevated"
													transition:fly={{ easing: backOut, y: -6, duration: 180 }}
												>
													<button
														type="button"
														onpointerenter={() => preloadRoute(`/instances/${inst.id}`)}
														onpointerdown={() => preloadRoute(`/instances/${inst.id}`, true)}
														onclick={() => { activeContextMenuId = null; void goto(`/instances/${inst.id}`); }}
														class={launcherButton({ variant: "ghost", size: "sm", class: "w-full justify-start text-left gap-2 whitespace-nowrap" })}
													>
														<Settings class="w-3.5 h-3.5 text-fg/60" /> {uiText("ui.286eb0604961b4d1")}
													</button>
													<button
														type="button"
														onclick={() => void instancesOpenFolder(inst.id).catch((e) => toast(String(e), "error"))}
														class={launcherButton({ variant: "ghost", size: "sm", class: "w-full justify-start text-left gap-2 whitespace-nowrap" })}
													>
														<FolderOpen class="w-3.5 h-3.5 text-fg/60" /> {uiText("screenshots.openFolder")}
													</button>
													<button
														type="button"
														onclick={() => {
															void api
																.invoke("profiles_delete", { id: inst.id })
																.then(() => profiles.remove(inst.id))
																.catch((e) => toast(String(e), "error"));
														}}
														class={launcherButton({ variant: "ghostDanger", size: "sm", class: "w-full justify-start text-left gap-2 whitespace-nowrap" })}
													>
														<Trash2 class="w-3.5 h-3.5" /> {uiText("instances.deleteConfirmTitle")}
													</button>
												</div>
											{/if}
										</div>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				{/if}
			</section>

			<section class="space-y-4 pb-8">

				<div class="space-y-3">
					<h2 class="text-lg font-black text-fg tracking-tight">
						{t("library.title")}
					</h2>

					<div class="flex flex-wrap items-center gap-2.5">
						<div class="relative min-w-[180px] flex-1">
							<Search class="w-3.5 h-3.5 absolute left-3.5 top-1/2 -translate-y-1/2 text-fg/30" />
							<input
								type="text"
								placeholder={t("common.search")}
								bind:value={searchQuery}
								class="w-full bg-bg/35 backdrop-blur-xl border border-fg/[0.06] rounded-xl pl-9 pr-3 py-2.5 text-xs text-fg placeholder:text-fg/30 outline-none focus:border-brand-500/60 transition-[color,background-color,border-color,box-shadow,transform,opacity]"
							/>
						</div>

						<button
							type="button"
							onclick={() => showNewGroupPrompt = true}
							class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5 shrink-0" })}
						>
							<FolderPlus class="w-3.5 h-3.5 text-fg/50" />
							<span>{t("library.newGroup")}</span>
						</button>

						<button
							type="button"
							onclick={() => showCreateModal = true}
							class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-1.5 shrink-0" })}
						>
							<Plus class="w-4 h-4 stroke-[3]" />
							<span>{t("library.newInstance")}</span>
						</button>
					</div>

					<div class="flex items-center gap-2 flex-wrap">
                        <GlassSelect icon={ArrowUpDown} bind:value={sortBy} label={uiText("ui.bffb787c611dc6a7")} options={[{value:"lastPlayed",label:uiText("ui.3068eba03d591c16")},{value:"name",label:uiText("instances.sortName")},{value:"version",label:uiText("instances.sortVersion")}]} />


						<button
							type="button"
							class={launcherButton({ variant: showNewGroupPrompt ? "ghostBrand" : "secondary", size: "sm" })}
							onclick={() => showNewGroupPrompt = !showNewGroupPrompt}
							title={uiText("ui.206f6d5e7122233e")}
						>
							<Filter class="w-3.5 h-3.5 text-fg/50" />
							<span>{t("library.addFilter")}</span>
						</button>
					</div>
				</div>


                <input bind:this={folderImageInput} data-folder-image-input type="file" accept="image/png,image/jpeg,image/webp,image/gif" class="hidden" tabindex="-1" onchange={uploadFolderImage} />
                {#if selectedGroup !== null}
                    <div class="flex items-center gap-3" aria-label={uiText('library.folderPath')}>
                        <button type="button" data-library-back class={launcherButton({variant:'secondary',size:'sm'})}
                            ondragover={event => { if (event.dataTransfer?.types.includes('application/x-luxmc-instance')) { event.preventDefault(); event.dataTransfer.dropEffect='move'; } }}
                            ondrop={event => dropIntoGroup(event,'')} onclick={() => selectedGroup=null}><ArrowLeft class="h-4 w-4" />{uiText('library.title')}</button>
                        <ChevronRight class="h-4 w-4 text-fg-muted" /><span class="truncate text-sm font-semibold text-fg">{selectedGroup}</span>
                    </div>
                {/if}

				{#if showNewGroupPrompt}
					<div class="p-3.5 rounded-2xl bg-bg/35 backdrop-blur-xl border border-brand-500/30 flex items-center gap-2.5" in:slide={{ easing: quintOut, duration: 220 }}>
						<input
							type="text"
							placeholder={uiText("ui.bf96010119f10b2f")}
							bind:value={newGroupName}
							class="flex-1 bg-black/40 border border-fg/10 rounded-xl px-3 py-2 text-xs text-fg outline-none focus:border-brand-500"
							onkeydown={(e) => { if (e.key === "Enter") handleAddGroup(); }}
						/>
						<button
							type="button"
							onclick={handleAddGroup}
							class={launcherButton({ variant: "primary", size: "sm", class: "" })}
						>
							{uiText("ui.967bcf34a9138e10")}
						</button>
						<button
							type="button"
							onclick={() => showNewGroupPrompt = false}
							class={launcherButton({ variant: "secondary", size: "sm", class: "" })}
						>
							{uiText("common.cancel")}
						</button>
					</div>
				{/if}

				{#if filteredProfiles.length === 0 && !visibleFolders.length}
					<div class="p-12 rounded-3xl bg-bg/35 backdrop-blur-xl border border-fg/[0.06] text-center space-y-3">
						<Layers class="w-8 h-8 text-fg/20 mx-auto" />
						<p class="text-sm text-fg/40">{uiText("ui.7c552b2c03b6eb16")}</p>
						<button
							type="button"
							onclick={() => { searchQuery = ""; selectedGroup = null; }}
							class={launcherButton({ variant: "ghost", size: "sm", class: "hover:underline" })}
						>
							{uiText("ui.fade927f9c6169bd")}
						</button>
					</div>
				{:else}
					<div class="home-instance-grid grid grid-cols-[repeat(auto-fill,minmax(min(220px,100%),1fr))] gap-5">
                        {#if selectedGroup === null}
                            {#each visibleFolders as folder (folder)}
                                <div class="relative">
                                <button type="button" data-library-folder={folder} aria-label={uiText('library.openFolder', {name:folder})}
                                    class="library-folder-card h-full w-full min-h-[232px] rounded-3xl border p-5 text-center shadow-soft transition-[background-color,border-color,box-shadow] {groupHover === folder ? 'border-brand-500 bg-brand-500/10' : 'border-fg/10 bg-bg/35 hover:border-brand-500/35 hover:bg-fg/10'}"
                                    disabled={!!movingProfile} ondragover={event => { if (event.dataTransfer?.types.includes('application/x-luxmc-instance')) { event.preventDefault(); groupHover=folder; event.dataTransfer.dropEffect='move'; } }}
                                    ondragleave={() => groupHover=null} ondrop={event => dropIntoGroup(event,folder)} onclick={() => selectedGroup=folder}>
                                    <div class="relative mx-auto flex h-28 w-28 items-center justify-center overflow-hidden rounded-2xl bg-brand-500/10">
                                        <FolderOpen class="h-16 w-16 text-brand-400" />
                                        {#if folderImage(folder)}<img data-folder-image src={folderImage(folder)} alt="" loading="lazy" decoding="async" class="absolute inset-0 h-full w-full rounded-2xl bg-bg-elevated object-contain" onload={event => { (event.currentTarget as HTMLImageElement).style.visibility='visible'; }} onerror={event => { (event.currentTarget as HTMLImageElement).style.visibility='hidden'; }} />{/if}
                                    </div>
                                    <span class="mt-5 block line-clamp-2 text-base font-bold text-fg">{folder}</span>
                                    <span class="mt-2 block text-xs text-fg-muted">{groupMembers(folder).length} · {uiText('library.folderItems')}</span>
                                    <span class="mt-2 block text-[10px] text-fg-subtle">{uiText('library.dropHere')}</span>
                                </button>
                                <div class="absolute right-2 top-2 flex gap-1">
                                    <button type="button" class={launcherButton({variant:'secondary',size:'icon',class:'h-9 w-9'})} disabled={folderImageBusy} aria-label={uiText('library.chooseFolderImage', {name:folder})} title={uiText('library.chooseFolderImage', {name:folder})} onclick={() => chooseFolderImage(folder)}>{#if folderImageBusy && folderImageTarget === folder}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<ImagePlus class="h-4 w-4" />{/if}</button>
                                    {#if folderImage(folder)}<button type="button" class={launcherButton({variant:'ghostDanger',size:'icon',class:'h-9 w-9'})} disabled={folderImageBusy} aria-label={uiText('library.removeFolderImage', {name:folder})} title={uiText('library.removeFolderImage', {name:folder})} onclick={() => removeFolderImage(folder)}><X class="h-4 w-4" /></button>{/if}
                                </div>
                                {#if folderImageBusy && folderImageTarget === folder}<p role="status" class="absolute inset-x-3 bottom-2 rounded-lg bg-bg-elevated px-2 py-1 text-center text-[10px] text-fg-muted">{uiText('publicProfile.processingImage')}</p>{/if}
                                </div>
                            {/each}
                        {/if}
						{#each filteredProfiles as inst (inst.id)}
							{@const tileCol = getInstanceTileColor(inst.id || inst.name)}
							{@const isThisLaunching = (isLaunching || appState.isLaunching) && (launchingProfileId === inst.id || appState.launchingProfileId === inst.id)}
							<div
								role="button"
								tabindex="0"
								draggable={!movingProfile}
                                ondragstart={event => { if (event.dataTransfer) { event.dataTransfer.setData('application/x-luxmc-instance',inst.id); event.dataTransfer.effectAllowed='move'; } }}
                                ondragend={() => groupHover=null}
                                class="home-instance-card focus-within:z-30 rounded-3xl bg-bg/35 hover:bg-fg/10 backdrop-blur-xl border border-fg/[0.08] hover:border-brand-500/35 transition-[background-color,border-color,box-shadow,transform] p-5 flex flex-col justify-between group relative shadow-soft hover:shadow-elevated cursor-pointer min-h-[232px]"
								onpointerenter={() => preloadRoute(`/instances/${inst.id}`)}
								onpointerdown={() => preloadRoute(`/instances/${inst.id}`, true)}
								onclick={event => { if (event.target instanceof Element && event.target.closest("button,select,label,input,a")) return; void goto(`/instances/${inst.id}`); }}
								onkeydown={(e) => { if (e.target === e.currentTarget && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); void goto(`/instances/${inst.id}`); } }}
							>

								<div class="w-full flex-1 flex items-center justify-center relative my-2">
									<div class="w-28 h-28 rounded-2xl flex items-center justify-center overflow-hidden transition-transform motion-reduce:transition-none group-hover:scale-[1.03] {inst.icon && !inst.icon.includes('grass_block') ? 'bg-fg/5 border border-fg/10' : tileCol.bg}">
										{#if inst.icon && inst.icon !== '/grass_block.png' && !inst.icon.includes('grass_block')}
											<img loading="lazy" decoding="async"
												src={inst.icon}
												alt={inst.name}
												class="w-full h-full object-contain"
												onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/grass_block.png'; (e.currentTarget as HTMLImageElement).className = 'w-24 h-24 object-contain [image-rendering:pixelated] drop-shadow-md'; }}
											/>
										{:else}
											<img loading="lazy" decoding="async"
												src="/grass_block.png"
												alt={inst.name}
												class="w-24 h-24 object-contain [image-rendering:pixelated] drop-shadow-md"
											/>
										{/if}
									</div>

									<button
										type="button"
										onclick={(e) => {
											e.stopPropagation();
											handleLaunch(inst);
										}}
										disabled={isLaunching || appState.isLaunching}
										class={launcherButton({ variant: "play", size: "icon", class: "absolute bottom-0 right-0 h-11 w-11 rounded-2xl" })}
										aria-label={`${uiText("instances.play")} ${inst.name}`}
										title={isThisLaunching ? (appState.launchStatusText || uiText("ui.dc0546b3e22c8f9e")) : uiText("instances.play")}
									>
										{#if isThisLaunching}
											<Loader2 class="w-4 h-4 animate-spin" />
										{:else}
											<Play class="w-4 h-4 fill-current ml-0.5" />
										{/if}
									</button>
								</div>

								<div class="w-full space-y-1 text-center mt-2 px-1">
									<h3 title={inst.name} class="text-base font-bold text-fg transition-colors line-clamp-2 leading-snug">
										{inst.name}
									</h3>
									<p class="text-xs text-fg-muted truncate font-medium">
										{inst.loader} • {inst.mcVersion}
									</p>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</section>

		</div>

		<RightSidebar />

	</div>

	<CreateInstanceModal
		bind:isOpen={showCreateModal}
		onClose={() => showCreateModal = false}
		versions={availableVersions}
		{versionsLoading}
		{systemRamMb}
		onCreate={handleCreateInstance}
	/>

{/if}
