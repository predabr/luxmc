<script lang="ts">
	import { fade, slide, fly } from "svelte/transition";
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";
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
		Filter
	} from "lucide-svelte";
	import RightSidebar from "$lib/components/layout/RightSidebar.svelte";
	import CreateInstanceModal from "$lib/components/instances/CreateInstanceModal.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { profiles, type Profile } from "$lib/stores/profiles.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { getFullCapeDataUrl } from "$lib/utils/capeTextures";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playClick, playSuccess } from "$lib/utils/sounds";
	import { 
		authDevLogin, 
		authOfflineLogin, 
		authLogin, 
		authGetClientId, 
		authSetClientId, 
		launchGame, 
		versionsCheckInstalled, 
		versionsDownload, 
		discordSetActivity, 
		instancesOpenFolder,
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
	let selectedGroup = $state("all");
	let sortBy = $state<"lastPlayed" | "name" | "version">("lastPlayed");
	let jumpInExpanded = $state(true);
	let isLaunching = $state(false);
	let launchingProfileId = $state<string | null>(null);
	let launchStatusText = $state("");
	let activeContextMenuId = $state<string | null>(null);

	let showCreateModal = $state(false);
	let availableVersions = $state<Array<{ id: string; versionType: string; releaseTime: string }>>([]);
	let versionsLoading = $state(false);
	let systemRamMb = $state(8192);

	let customGroups = $state<string[]>(["Vanilla", "Modded"]);
	let showNewGroupPrompt = $state(false);
	let newGroupName = $state("");

	onMount(() => {
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
			const saved = localStorage.getItem("luxmc_current_account");
			if (saved) {
				try {
					account.value = JSON.parse(saved);
				} catch {}
			}
		}

		loadVersions();

		const handleClickOutside = () => {
			activeContextMenuId = null;
		};
		window.addEventListener("click", handleClickOutside);
		return () => window.removeEventListener("click", handleClickOutside);
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
				localStorage.setItem("luxmc_cached_versions", JSON.stringify(response));
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
		toast(`Conta "${name}" removida.`, "info");
	}

	async function handleOfflineAuth(customNick?: string) {
		const name = (customNick || offlineName).trim();
		if (!name) {
			toast("Digite seu Nickname para jogar.", "error");
			return;
		}

		isLoggingIn = true;
		try {
			const backendAcc = await authOfflineLogin(name).catch(() => null);
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

			localStorage.setItem("luxmc_current_account", JSON.stringify(newAcc));
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
			toast("Erro ao entrar: " + String(e), "error");
		} finally {
			isLoggingIn = false;
		}
	}

	async function handleMicrosoftLogin() {
		try {
			const existingId = await authGetClientId().catch(() => "");
			if (!existingId || existingId === "00000000-0000-0000-0000-000000000000") {
				await authSetClientId("9750ebbe-21e9-4a4d-b808-f451a3e0af7f").catch(() => null);
			}
			await startMsLogin();
		} catch (e) {
			toast(String(e), "error");
		}
	}

	async function startMsLogin() {
		isLoggingInMicrosoft = true;
		try {
			toast("Iniciando autenticação com a Microsoft no navegador...", "info");
			const acc = await authLogin();
			const newAcc = {
				id: acc.id,
				username: acc.username,
				uuid: acc.uuid,
				minecraftToken: acc.accessToken || "",
				expiresAt: acc.expiresAt ? (acc.expiresAt < 1e11 ? acc.expiresAt * 1000 : acc.expiresAt) : 0,
				skinUrl: acc.skinUrl || null,
				skinVariant: acc.skinVariant || "classic",
				capeUrl: acc.capeUrl || null
			};

			localStorage.setItem("luxmc_current_account", JSON.stringify(newAcc));
			activeSkinStore.setSkin({
				id: newAcc.uuid,
				name: newAcc.username,
				url: `https://mc-heads.net/body/${newAcc.username}/300`,
				skinUrl: newAcc.skinUrl || `https://minotar.net/skin/${newAcc.username}`,
				avatarUrl: `https://mc-heads.net/avatar/${newAcc.username}/100`,
				type: newAcc.skinVariant === "slim" ? "alex" : "steve",
				hasCape: Boolean(newAcc.capeUrl),
				capeType: newAcc.capeUrl ? "custom" : "none",
				customCapeUrl: newAcc.capeUrl || ""
			});

			if (!savedAccounts.some(n => n.toLowerCase() === newAcc.username.toLowerCase())) {
				saveSavedAccounts([newAcc.username, ...savedAccounts]);
			}

			toast(`Conectado com a conta Microsoft: ${newAcc.username}`, "success");
			playSuccess();
			account.value = newAcc;
		} catch (e) {
			toast("Falha na autenticação Microsoft: " + String(e), "error");
		} finally {
			isLoggingInMicrosoft = false;
		}
	}

	async function saveAndLoginWithClientId() {
		const val = msClientIdInput.trim();
		if (!val) {
			toast("Digite um Client ID válido", "error");
			return;
		}
		isSavingClientId = true;
		try {
			await authSetClientId(val);
			showMsClientIdModal = false;
			toast("Azure Client ID salvo com sucesso!", "success");
			await startMsLogin();
		} catch (e) {
			toast("Erro ao salvar Client ID: " + String(e), "error");
		} finally {
			isSavingClientId = false;
		}
	}

	async function handleDevLogin() {
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
			localStorage.setItem("luxmc_current_account", JSON.stringify(newAcc));
			toast(`Modo Dev Ativo: ${devAcc.username}`, "info");
			playSuccess();
			account.value = newAcc;
		} catch (e) {
			toast("Erro no modo dev: " + String(e), "error");
		} finally {
			isLoggingIn = false;
		}
	}

	async function handleLaunch(targetProfile: Profile, serverIp?: string, serverPort?: number) {
		if (isLaunching) return;
		isLaunching = true;
		launchingProfileId = targetProfile.id;
		launchStatusText = serverIp ? `Conectando a ${serverIp}...` : "Iniciando...";

		try {
			let userUuid = account.value?.uuid;
			if (!userUuid) {
				const devAcc = await authDevLogin();
				userUuid = devAcc.uuid;
			}

			const verId = targetProfile.mcVersion || "1.20.4";
			launchStatusText = "Verificando arquivos do jogo...";

			const installed = await versionsCheckInstalled(verId).catch(() => false);
			if (!installed) {
				launchStatusText = `Baixando Minecraft ${verId}...`;
				await versionsDownload(verId);
			}

			launchStatusText = serverIp ? `Conectando a ${serverIp}...` : "Iniciando Minecraft...";
			const isVulkan = typeof window !== "undefined" ? localStorage.getItem("luxmc_enable_vulkan") === "true" : false;
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

			gamingStats.onGameStart();
			profiles.setLastPlayed(targetProfile.id);

			discordSetActivity({
				inGame: true,
				details: targetProfile.name,
				state: `Minecraft ${verId} · ${targetProfile.loader.toUpperCase()}`,
				largeText: targetProfile.name,
				largeImage: targetProfile.icon || "grass",
				smallImage: targetProfile.loader === "fabric" ? "fabric" : "grass",
				smallText: `Luxmc · ${targetProfile.loader}`,
				startTime: Math.floor(Date.now() / 1000)
			}).catch(() => {});

			toast(`🎮 Minecraft ${verId} iniciado! (PID: ${result.pid})`, "success");
			void handlePostLaunchActions();
		} catch (e) {
			toast("Falha ao iniciar o Minecraft: " + String(e), "error");
		} finally {
			isLaunching = false;
			launchingProfileId = null;
			launchStatusText = "";
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
				group: input.loader === "vanilla" ? "Vanilla" : "Modded",
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
			toast(`Instância "${input.name}" criada com sucesso!`, "success");
		} catch (e) {
			toast("Erro ao criar instância: " + String(e), "error");
		}
	}

	function handleAddGroup() {
		const name = newGroupName.trim();
		if (!name) return;
		if (!customGroups.includes(name)) {
			customGroups = [...customGroups, name];
			toast(`Grupo "${name}" criado!`, "success");
		}
		newGroupName = "";
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
				const matchesGroup = selectedGroup === "all" || p.group?.toLowerCase() === selectedGroup.toLowerCase() || (selectedGroup === "Vanilla" && p.loader === "vanilla") || (selectedGroup === "Modded" && p.loader !== "vanilla");
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
		class="relative min-h-screen w-full flex items-center justify-center p-6 select-none bg-[#0c0c0e] overflow-hidden"
	>
		<!-- Cosmic background mesh and glow -->
		<div class="absolute inset-0 bg-[radial-gradient(ellipse_80%_80%_at_50%_-20%,rgba(59,130,246,0.18),rgba(255,255,255,0))] pointer-events-none"></div>
		<div class="absolute -top-40 left-1/2 -translate-x-1/2 w-[700px] h-[350px] bg-blue-500/10 blur-[130px] rounded-full pointer-events-none"></div>

		<div 
			class="w-full max-w-md bg-[#14151a]/95 backdrop-blur-2xl border border-white/10 rounded-3xl p-8 shadow-[0_32px_90px_rgba(0,0,0,0.85)] relative z-10 space-y-6"
			in:fly={{ y: 20, duration: 300 }}
		>
			<div class="flex flex-col items-center text-center space-y-3">
				<div class="w-16 h-16 rounded-2xl bg-gradient-to-b from-[#1e2029] to-[#121318] border border-white/15 p-2.5 shadow-2xl flex items-center justify-center relative group">
					<div class="absolute inset-0 bg-blue-500/20 rounded-2xl blur-lg pointer-events-none group-hover:bg-blue-500/30 transition-all"></div>
					<img src="/logo.png" alt="Luxmc" class="w-full h-full object-contain relative z-10" />
				</div>
				<div>
					<h1 class="text-xl font-black text-white tracking-tight">Luxmc Launcher</h1>
					<p class="text-xs text-white/40 mt-0.5">Selecione o método de autenticação</p>
				</div>
			</div>

			<div class="flex rounded-2xl bg-black/50 border border-white/10 p-1">
				<button
					type="button"
					onclick={() => loginTab = "microsoft"}
					class="flex-1 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer flex items-center justify-center gap-2 {loginTab === 'microsoft' ? 'bg-blue-600 text-white shadow-md' : 'text-white/50 hover:text-white'}"
				>
					<svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none">
						<rect x="1" y="1" width="10" height="10" fill="#f25022" rx="1"/>
						<rect x="13" y="1" width="10" height="10" fill="#7fba00" rx="1"/>
						<rect x="1" y="13" width="10" height="10" fill="#00a4ef" rx="1"/>
						<rect x="13" y="13" width="10" height="10" fill="#ffb900" rx="1"/>
					</svg>
					<span>Microsoft</span>
				</button>

				<button
					type="button"
					onclick={() => loginTab = "offline"}
					class="flex-1 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer flex items-center justify-center gap-2 {loginTab === 'offline' ? 'bg-white/15 text-white shadow-md border border-white/10' : 'text-white/50 hover:text-white'}"
				>
					<User class="w-3.5 h-3.5" />
					<span>Modo Offline</span>
				</button>
			</div>

			{#if loginTab === "microsoft"}
				<div class="flex flex-col items-center text-center space-y-5 pt-1" in:fade={{ duration: 150 }}>
					<div class="space-y-1">
						<h2 class="text-base font-bold text-white">Conta Microsoft Oficial</h2>
						<p class="text-xs text-white/50 max-w-xs leading-relaxed">
							Autenticação segura via OAuth2. É necessário possuir o Minecraft original na conta.
						</p>
					</div>

					{#if isLoggingInMicrosoft}
						<!-- Futuristic Holographic Auth Portal -->
						<div class="w-full bg-[#181a24]/90 border border-blue-500/30 rounded-2xl p-6 text-center space-y-4 shadow-xl relative overflow-hidden">
							<div class="relative w-16 h-16 mx-auto flex items-center justify-center">
								<div class="absolute inset-0 rounded-full border-2 border-blue-500/20 animate-ping pointer-events-none"></div>
								<div class="w-14 h-14 rounded-full border-2 border-dashed border-blue-400 animate-spin flex items-center justify-center"></div>
								<Loader2 class="w-6 h-6 text-blue-400 animate-spin absolute" />
							</div>

							<div class="space-y-1">
								<p class="text-xs font-black text-white tracking-wide">Aguardando no Navegador...</p>
								<p class="text-[11px] text-white/40 leading-relaxed">
									Conclua a autorização na janela segura da Microsoft que se abriu.
								</p>
							</div>

							<button 
								type="button" 
								onclick={() => isLoggingInMicrosoft = false}
								class="px-4 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-[11px] font-bold text-white/60 hover:text-white cursor-pointer transition-colors"
							>
								Cancelar
							</button>
						</div>
					{:else}
						<div class="w-20 h-20 rounded-2xl bg-[#181a24] border border-white/10 flex items-center justify-center shadow-inner relative group">
							<div class="absolute inset-0 bg-blue-500/10 rounded-2xl blur-md pointer-events-none"></div>
							<svg class="w-10 h-10 relative z-10" viewBox="0 0 24 24" fill="none">
								<rect x="2" y="2" width="9" height="9" fill="#f25022" rx="1.5"/>
								<rect x="13" y="2" width="9" height="9" fill="#7fba00" rx="1.5"/>
								<rect x="2" y="13" width="9" height="9" fill="#00a4ef" rx="1.5"/>
								<rect x="13" y="13" width="9" height="9" fill="#ffb900" rx="1.5"/>
							</svg>
						</div>

						<div class="w-full space-y-2">
							<button
								type="button"
								class="w-full py-3.5 rounded-2xl bg-blue-600 hover:bg-blue-500 active:scale-95 text-white font-black text-xs uppercase tracking-wider flex items-center justify-center gap-2.5 transition-all shadow-lg shadow-blue-600/25 cursor-pointer disabled:opacity-50"
								onclick={() => { playClick(); handleMicrosoftLogin(); }}
								disabled={isLoggingIn || isLoggingInMicrosoft}
							>
								<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
									<path d="M1 1h10v10H1V1zm12 0h10v10H13V1zM1 13h10v10H1V13zm12 0h10v10H13V13z"/>
								</svg>
								<span>Entrar com Microsoft</span>
							</button>
							<span class="text-[11px] text-white/40 font-medium block">Conexão oficial criptografada</span>
						</div>
					{/if}

					<div class="pt-1">
						<button
							type="button"
							class="text-[11px] text-white/40 hover:text-blue-400 transition-colors cursor-pointer"
							onclick={() => showMsClientIdModal = true}
						>
							Problemas com o login? Configurar Azure ID
						</button>
					</div>
				</div>

			{:else}

				<div class="space-y-4 pt-1" in:fade={{ duration: 150 }}>
					<div class="text-center space-y-1">
						<h2 class="text-base font-bold text-white">Entrar com Nickname</h2>
						<p class="text-xs text-white/50">Jogue instantaneamente com qualquer nome.</p>
					</div>

					<div class="space-y-1.5">
						<label for="offline-nick-input" class="text-[11px] font-bold text-white/60 block text-left">
							Nickname do Jogador:
						</label>
						<div class="relative flex items-center">
							<div class="w-7 h-7 rounded-lg bg-black/40 border border-white/10 overflow-hidden absolute left-2.5 flex items-center justify-center pointer-events-none">
								<img 
									src={`https://mc-heads.net/avatar/${offlineName.trim() || 'Steve'}/32`} 
									alt="Avatar" 
									class="w-full h-full object-cover" 
								/>
							</div>
							<input 
								id="offline-nick-input"
								type="text" 
								placeholder="Ex: SteveGamer" 
								bind:value={offlineName}
								maxlength="16"
								class="w-full bg-black/40 border border-white/10 focus:border-blue-500/60 rounded-2xl pl-12 pr-4 py-3 text-xs font-bold text-white outline-none transition-all placeholder:text-white/20"
								onkeydown={(e) => { if (e.key === "Enter") handleOfflineAuth(); }}
							/>
						</div>
					</div>

					<button 
						type="button" 
						class="w-full py-3.5 rounded-2xl bg-blue-600 hover:bg-blue-500 text-white font-black text-xs uppercase tracking-wider flex items-center justify-center gap-2 transition-all shadow-lg shadow-blue-600/25 hover:scale-[1.01] active:scale-[0.98] cursor-pointer disabled:opacity-50"
						onclick={() => { playClick(); handleOfflineAuth(); }}
						disabled={isLoggingIn || isLoggingInMicrosoft}
					>
						{#if isLoggingIn}
							<Loader2 class="w-4 h-4 animate-spin text-white" /> Entrando...
						{:else}
							<Play class="w-3.5 h-3.5 fill-current" /> Jogar Offline
						{/if}
					</button>

					{#if savedAccounts.length > 0}
						<div class="space-y-2 pt-1 border-t border-white/5">
							<span class="text-[10px] font-bold text-white/40 uppercase tracking-wider block text-left">
								Contas recentes:
							</span>
							<div class="flex flex-wrap gap-1.5">
								{#each savedAccounts as accName (accName)}
									<div class="inline-flex items-center gap-1.5 pl-1.5 pr-2 py-1 rounded-xl bg-white/5 hover:bg-white/10 border border-white/5 transition-all group">
										<button 
											type="button" 
											onclick={() => handleOfflineAuth(accName)}
											class="flex items-center gap-1.5 text-left cursor-pointer"
										>
											<img 
												src={`https://mc-heads.net/avatar/${accName}/32`} 
												alt={accName} 
												class="w-4 h-4 rounded object-cover" 
											/>
											<span class="text-xs font-bold text-white/80 group-hover:text-blue-300 transition-colors">{accName}</span>
										</button>
										<button 
											type="button" 
											onclick={() => deleteSavedAccount(accName)}
											class="opacity-0 group-hover:opacity-100 text-white/30 hover:text-red-400 transition-opacity cursor-pointer p-0.5"
											title="Remover"
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

			<div class="pt-2 border-t border-white/5 flex flex-col items-center gap-2">
				<button 
					type="button" 
					onclick={() => openUrl("https://luxmc-r92.pages.dev/#skin-studio")}
					class="text-[11px] text-white/50 hover:text-blue-400 transition-colors flex items-center gap-1 cursor-pointer"
				>
					Personalizar skins e capas no Studio 3D Web <ExternalLink class="w-3 h-3" />
				</button>

				<div class="flex items-center justify-between w-full pt-1">
					<button
						type="button"
						class="text-[10px] text-white/30 hover:text-amber-400 transition-colors cursor-pointer"
						onclick={() => showMsClientIdModal = true}
					>
						Configurar Azure ID
					</button>
					<button
						type="button"
						class="text-[10px] text-white/30 hover:text-white transition-colors cursor-pointer"
						onclick={handleDevLogin}
					>
						Acesso Desenvolvedor
					</button>
				</div>
			</div>
		</div>
	</div>

	{#if showMsClientIdModal}
		<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm" in:fade={{ duration: 150 }}>
			<div class="w-full max-w-md rounded-3xl bg-[#13141a] border border-amber-500/30 p-6 shadow-2xl space-y-4" in:fly={{ y: 20, duration: 200 }}>
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2.5">
						<Lock class="w-4 h-4 text-amber-400" />
						<h3 class="text-sm font-bold text-white">Configurar Microsoft Azure</h3>
					</div>
					<button 
						type="button" 
						class="text-white/40 hover:text-white text-xs cursor-pointer"
						onclick={() => showMsClientIdModal = false}
					>
						✕
					</button>
				</div>

				<p class="text-xs text-white/60 leading-relaxed">
					Insira o <span class="text-amber-400 font-bold">Client ID</span> do seu registro de aplicativo no Azure Portal (Redirect URI: <code class="text-amber-300">http://localhost:8453/callback</code>).
				</p>

				<input 
					type="text" 
					placeholder="9750ebbe-21e9-4a4d-b808-f451a3e0af7f" 
					bind:value={msClientIdInput}
					class="w-full bg-black/40 border border-white/10 rounded-xl px-4 py-2.5 text-xs text-white font-mono outline-none focus:border-amber-500/50"
				/>

				<div class="flex items-center gap-2 pt-1">
					<button 
						type="button" 
						class="flex-1 py-2 rounded-xl bg-white/5 text-white/60 text-xs font-bold hover:bg-white/10 cursor-pointer"
						onclick={() => showMsClientIdModal = false}
					>
						Cancelar
					</button>
					<button 
						type="button" 
						class="flex-1 py-2 rounded-xl bg-amber-500 hover:bg-amber-400 text-black text-xs font-bold cursor-pointer"
						onclick={saveAndLoginWithClientId}
						disabled={isSavingClientId}
					>
						Salvar e Conectar
					</button>
				</div>
			</div>
		</div>
	{/if}

{:else}

	<div class="flex gap-6 min-h-full w-full select-none" in:fade={{ duration: 150 }}>
		
		<div class="flex-1 flex flex-col min-w-0 space-y-6">

			<header class="flex items-center justify-between gap-4 py-1">
				<div class="flex items-center gap-3">
					<div class="flex items-center gap-1 bg-[#121318] border border-white/5 rounded-xl p-1 shadow-sm">
						<button 
							type="button" 
							class="p-1.5 rounded-lg text-white/40 hover:text-white hover:bg-white/5 transition-colors cursor-pointer"
							title="Voltar"
							onclick={() => history.back()}
						>
							<ArrowLeft class="w-3.5 h-3.5" />
						</button>
						<button 
							type="button" 
							class="p-1.5 rounded-lg text-white/40 hover:text-white hover:bg-white/5 transition-colors cursor-pointer"
							title="Avançar"
							onclick={() => history.forward()}
						>
							<ChevronRight class="w-3.5 h-3.5" />
						</button>
					</div>

					<div class="flex items-center gap-2 text-xs font-bold text-white/70">
						<span class="text-white/30">▷</span>
						<span class="text-white">Home</span>
					</div>
				</div>

				<div class="flex items-center gap-2">
					<div class="flex items-center gap-2 px-3 py-1.5 rounded-xl bg-[#121318] border border-white/5 text-[11px] text-white/60">
						{#if isLaunching}
							<span class="w-2 h-2 rounded-full bg-amber-400 animate-ping"></span>
							<span class="text-amber-300 font-bold">{launchStatusText || "Iniciando..."}</span>
						{:else}
							<span class="w-2 h-2 rounded-full bg-neutral-600"></span>
							<span>No instances running</span>
						{/if}
					</div>
				</div>
			</header>

			<section class="space-y-3">
				<div class="flex items-center justify-between">
					<button 
						type="button" 
						onclick={() => jumpInExpanded = !jumpInExpanded}
						class="flex items-center gap-2 text-sm font-black text-white uppercase tracking-wider hover:text-emerald-400 transition-colors cursor-pointer"
					>
						<span>Jump in</span>
						{#if jumpInExpanded}
							<ChevronUp class="w-4 h-4 text-white/40" />
						{:else}
							<ChevronDown class="w-4 h-4 text-white/40" />
						{/if}
					</button>
				</div>

				{#if jumpInExpanded}
					{#if jumpInInstances.length === 0}
						<div class="p-6 rounded-2xl bg-[#121318] border border-white/5 text-center space-y-2">
							<p class="text-xs text-white/40">Nenhuma instância encontrada para início rápido.</p>
							<button 
								type="button" 
								onclick={() => showCreateModal = true}
								class="px-4 py-2 rounded-xl bg-emerald-500 text-black text-xs font-bold hover:bg-emerald-400 transition-colors cursor-pointer"
							>
								+ Criar Primeira Instância
							</button>
						</div>
					{:else}
						<div class="space-y-2" transition:slide={{ duration: 180 }}>
							{#each jumpInInstances as inst (inst.id)}
								<div class="flex items-center justify-between p-3 rounded-2xl bg-[#121318] hover:bg-[#16171f] border border-white/5 hover:border-white/10 transition-all group shadow-sm">
									<div class="flex items-center gap-3.5 min-w-0">
										<div class="w-12 h-12 rounded-xl overflow-hidden bg-black/40 border border-white/10 shrink-0 p-1 flex items-center justify-center">
											<img 
												src={inst.icon || "/grass_block.png"} 
												alt={inst.name}
												class="w-full h-full object-contain"
												onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/grass_block.png'; }}
											/>
										</div>

										<div class="min-w-0 space-y-0.5">
											<div class="flex items-center gap-2">
												<h3 class="text-sm font-bold text-white truncate leading-tight group-hover:text-emerald-300 transition-colors">
													{inst.name}
												</h3>
												{#if !inst.lastPlayed}
													<span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[9px] font-bold bg-emerald-500/15 text-emerald-400 border border-emerald-500/30">
														<Sparkles class="w-2.5 h-2.5" /> New instance
													</span>
												{/if}
											</div>

											<div class="flex items-center gap-2 text-[11px] text-white/40">
												<span class="px-2 py-0.5 rounded-md font-bold uppercase text-[9px] border {getLoaderBadgeColor(inst.loader)}">
													{inst.loader} {inst.mcVersion}
												</span>
												{#if inst.lastPlayed}
													<span>Jogado recentemente</span>
												{:else}
													<span>Never played</span>
												{/if}
											</div>
										</div>
									</div>

									<div class="flex items-center gap-2 shrink-0">
										<button
											type="button"
											onclick={() => handleLaunch(inst)}
											disabled={isLaunching && launchingProfileId === inst.id}
											class="px-5 py-2 rounded-full bg-emerald-500 hover:bg-emerald-400 text-black font-extrabold text-xs flex items-center gap-1.5 shadow-md transition-all hover:scale-105 active:scale-95 cursor-pointer disabled:opacity-50"
											title="Jogar Instância"
										>
											{#if isLaunching && launchingProfileId === inst.id}
												<Loader2 class="w-3.5 h-3.5 animate-spin" />
												<span>Iniciando...</span>
											{:else}
												<Play class="w-3.5 h-3.5 fill-current" />
												<span>Play</span>
											{/if}
										</button>

										<div class="relative">
											<button
												type="button"
												onclick={(e) => {
													e.stopPropagation();
													activeContextMenuId = activeContextMenuId === inst.id ? null : inst.id;
												}}
												class="p-2 rounded-xl text-white/40 hover:text-white hover:bg-white/5 transition-colors cursor-pointer"
											>
												<MoreVertical class="w-4 h-4" />
											</button>

											{#if activeContextMenuId === inst.id}
												<div 
													class="absolute right-0 top-10 w-44 rounded-xl bg-[#1c1d25] border border-white/10 shadow-2xl py-1 z-30 space-y-0.5"
													transition:fly={{ y: -6, duration: 120 }}
												>
													<button
														type="button"
														onclick={() => goto(`/instances/${inst.id}`)}
														class="w-full px-3 py-2 text-left text-xs text-white hover:bg-white/5 flex items-center gap-2 cursor-pointer"
													>
														<Settings class="w-3.5 h-3.5 text-white/60" /> Detalhes & Mods
													</button>
													<button
														type="button"
														onclick={() => instancesOpenFolder(inst.id)}
														class="w-full px-3 py-2 text-left text-xs text-white hover:bg-white/5 flex items-center gap-2 cursor-pointer"
													>
														<FolderOpen class="w-3.5 h-3.5 text-white/60" /> Abrir Pasta
													</button>
													<button
														type="button"
														onclick={() => profiles.remove(inst.id)}
														class="w-full px-3 py-2 text-left text-xs text-red-400 hover:bg-red-500/10 flex items-center gap-2 cursor-pointer"
													>
														<Trash2 class="w-3.5 h-3.5" /> Excluir Instância
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

			<div class="flex items-center justify-center py-1">
				<div class="w-12 h-1 bg-white/10 rounded-full"></div>
			</div>

			<section class="space-y-4 pb-8">
				
				<div class="space-y-3">
					<h2 class="text-lg font-black text-white tracking-tight">
						Library
					</h2>

					<div class="flex items-center gap-2.5">
						<div class="relative flex-1">
							<Search class="w-3.5 h-3.5 absolute left-3.5 top-1/2 -translate-y-1/2 text-white/30" />
							<input 
								type="text" 
								placeholder="Search" 
								bind:value={searchQuery}
								class="w-full bg-[#121318] border border-white/5 rounded-xl pl-9 pr-3 py-2 text-xs text-white placeholder:text-white/30 outline-none focus:border-white/20 transition-all"
							/>
						</div>

						<button 
							type="button" 
							onclick={() => showNewGroupPrompt = true}
							class="px-3.5 py-2 rounded-xl bg-[#121318] hover:bg-[#181920] text-white/70 hover:text-white border border-white/5 text-xs font-bold transition-all flex items-center gap-1.5 cursor-pointer shrink-0"
						>
							<FolderPlus class="w-3.5 h-3.5 text-white/50" />
							<span>+ New group</span>
						</button>

						<button 
							type="button" 
							onclick={() => showCreateModal = true}
							class="px-4 py-2 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-black text-xs font-black transition-all flex items-center gap-1.5 cursor-pointer shadow-md shrink-0"
						>
							<Plus class="w-4 h-4 stroke-[3]" />
							<span>+ New instance</span>
						</button>
					</div>

					<div class="flex items-center gap-2">
						<div class="flex items-center gap-1 bg-[#121318] border border-white/5 rounded-xl px-2.5 py-1">
							<span class="text-white/40 text-xs">⇅</span>
							<select 
								bind:value={sortBy}
								class="bg-transparent text-xs text-white/70 outline-none cursor-pointer pr-1"
							>
								<option value="lastPlayed" class="bg-[#181920]">Last played</option>
								<option value="name" class="bg-[#181920]">Name</option>
								<option value="version" class="bg-[#181920]">Version</option>
							</select>
						</div>

						<div class="flex items-center gap-1 bg-[#121318] border border-white/5 rounded-xl px-2.5 py-1">
							<span class="text-white/40 text-xs">⊞</span>
							<select 
								bind:value={selectedGroup}
								class="bg-transparent text-xs text-white/70 outline-none cursor-pointer pr-1"
							>
								<option value="all" class="bg-[#181920]">Custom group</option>
								{#each customGroups as grp}
									<option value={grp} class="bg-[#181920]">{grp}</option>
								{/each}
							</select>
						</div>

						<button 
							type="button"
							class="flex items-center gap-1 bg-[#121318] hover:bg-[#181920] border border-white/5 rounded-xl px-3 py-1.5 text-xs text-white/60 hover:text-white transition-colors cursor-pointer"
						>
							<Filter class="w-3 h-3 text-white/40" />
							<span>Add filter</span>
						</button>
					</div>
				</div>

				{#if showNewGroupPrompt}
					<div class="p-3.5 rounded-2xl bg-[#161720] border border-emerald-500/30 flex items-center gap-2.5" in:slide={{ duration: 150 }}>
						<input 
							type="text" 
							placeholder="Nome do novo grupo..."
							bind:value={newGroupName}
							class="flex-1 bg-black/40 border border-white/10 rounded-xl px-3 py-1.5 text-xs text-white outline-none focus:border-emerald-400"
							onkeydown={(e) => { if (e.key === "Enter") handleAddGroup(); }}
						/>
						<button 
							type="button" 
							onclick={handleAddGroup}
							class="px-3 py-1.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-black text-xs font-bold cursor-pointer"
						>
							Adicionar
						</button>
						<button 
							type="button" 
							onclick={() => showNewGroupPrompt = false}
							class="px-3 py-1.5 rounded-xl bg-white/5 text-white/50 hover:text-white text-xs cursor-pointer"
						>
							Cancelar
						</button>
					</div>
				{/if}

				{#if filteredProfiles.length === 0}
					<div class="p-12 rounded-3xl bg-[#121318] border border-white/5 text-center space-y-3">
						<Layers class="w-8 h-8 text-white/20 mx-auto" />
						<p class="text-sm text-white/40">Nenhuma instância encontrada para os filtros aplicados.</p>
						<button 
							type="button" 
							onclick={() => { searchQuery = ""; selectedGroup = "all"; }}
							class="text-xs text-emerald-400 hover:underline font-bold cursor-pointer"
						>
							Limpar filtros
						</button>
					</div>
				{:else}
					<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3.5">
						{#each filteredProfiles as inst (inst.id)}
							<div 
								role="button"
								tabindex="0"
								class="rounded-2xl bg-[#121318] hover:bg-[#161720] border border-white/5 hover:border-white/15 transition-all p-3 flex flex-col justify-between group relative shadow-sm cursor-pointer min-h-[160px]"
								onclick={() => goto(`/instances/${inst.id}`)}
								onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") goto(`/instances/${inst.id}`); }}
							>
								<div class="w-full flex-1 flex items-center justify-center relative my-1">
									<div class="w-16 h-16 rounded-2xl bg-black/40 border border-white/10 p-2 flex items-center justify-center overflow-hidden transition-transform group-hover:scale-105">
										<img 
											src={inst.icon || "/grass_block.png"} 
											alt={inst.name}
											class="w-full h-full object-contain"
											onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/grass_block.png'; }}
										/>
									</div>

									<button
										type="button"
										onclick={(e) => {
											e.stopPropagation();
											handleLaunch(inst);
										}}
										class="absolute bottom-0 right-3 w-9 h-9 rounded-full bg-emerald-500 hover:bg-emerald-400 text-black flex items-center justify-center shadow-lg opacity-0 group-hover:opacity-100 transition-all transform scale-90 group-hover:scale-100 cursor-pointer"
										title="Play"
									>
										<Play class="w-3.5 h-3.5 fill-current ml-0.5" />
									</button>
								</div>

								<div class="w-full space-y-0.5 text-center mt-2">
									<h3 class="text-xs font-bold text-white group-hover:text-emerald-300 transition-colors truncate">
										{inst.name}
									</h3>
									<p class="text-[10px] text-white/40 truncate">
										{inst.loader} {inst.mcVersion}
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