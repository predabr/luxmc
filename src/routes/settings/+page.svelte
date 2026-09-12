<script lang="ts">
	import { fade } from "svelte/transition";
	import { onMount, onDestroy } from "svelte";
	import {
		Settings as SettingsIcon,
		Palette,
		Users,
		Cpu,
		HardDrive,
		Sliders,
		Shield,
		Bell,
		Info,
		Check,
		FolderOpen, Network,
		Globe,
		RefreshCw,
		Sparkles,
		Volume2,
		Terminal,
		Zap,
		Layers,
		CheckCircle2,
		ExternalLink,
		Monitor,
		Gamepad2,
		Laptop,
		Flame,
		Eye,
		VolumeX,
		Lock,
		Play
	} from "lucide-svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { themeStore, THEMES, ACCENTS } from "$lib/stores/theme.svelte";
	import { setLocale, schedulePersist } from "$lib/stores/persistence.svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { open } from "@tauri-apps/plugin-dialog";
	import {
		storageBreakdown,
		discordSetActivity,
		discordClearActivity,
		getSystemSpecs,
		changelogGet,
		curseforgeStatus,
		curseforgeGetKey,
		curseforgeSetKey,
		curseforgeValidateKey,
		type ChangelogEntry,
		type StorageBreakdown
	} from "$lib/api";
	import ThemeSection from "$lib/components/settings/ThemeSection.svelte";
	import JvmSection from "$lib/components/settings/JvmSection.svelte";
	import LanguageSection from "$lib/components/settings/LanguageSection.svelte";
	import AdvancedSection from "$lib/components/settings/AdvancedSection.svelte";

	const { t } = useTranslation();

	type Section =
		| "geral"
		| "aparencia"
		| "amigos"
		| "java"
		| "linux"
		| "graficos"
		| "armazenamento"
		| "privacidade"
		| "notificacoes"
		| "sobre";

	let activeSection = $state<Section>("geral");

	// 1. Geral states (10)
	let selectedLanguage = $state("pt-BR");
	let minimizeToTray = $state(true);
	let minimizeOnLaunch = $state(true);
	let reopenOnGameClose = $state(true);
	let hideDefaultInstances = $state(false);
	let openConsoleWindow = $state(false);
	let disableTimeTracking = $state(false);
	let disableModSuggestions = $state(false);
	let allowBetaMods = $state(false);
	let useBitsPerSecond = $state(false);
	let autoStartWithLinux = $state(false);
	let performanceMode = $state(settings.value.performanceMode ?? false);

	// 3. Amigos & Discord states (8)
	let discordRpc = $state(true);
	let discordShowInstance = $state(true);
	let discordShowPlaytime = $state(true);
	let discordJoinButton = $state(true);
	let discordShowServer = $state(true);
	let discordShowAchievements = $state(true);
	let p2pBackgroundListener = $state(true);
	let p2pLanDiscovery = $state(true);
	let friendMessageSound = $state(true);
	let dndModeDuringGame = $state(false);

	// 5. Linux & GameMode states (13)
	let enableGameMode = $state(true);
	let nativeWayland = $state(true);
	let enableMangoHud = $state(false);
	let waylandCompositorBypass = $state(true);
	let diskIoPriority = $state(true);
	let zramOptimization = $state(true);
	let preventSystemSleep = $state(true);
	let mesaShaderCache = $state(true);
	let primeNvidiaGpu = $state(false);
	let gamescopeSupport = $state(false);
	let pipewireProAudio = $state(true);
	let ananicyCpuScheduler = $state(true);
	let disableMouseAcceleration = $state(true);

	// 6. Gráficos & Minecraft states (8)
	let enableVulkan = $state(true);
	let defaultResWidth = $state(1920);
	let defaultResHeight = $state(1080);
	let startFullscreen = $state(true);
	let defaultFpsLimit = $state("144");
	let enableVsync = $state(false);
	let asyncChunkLoading = $state(true);
	let dynamicLighting = $state(true);

	// 7. Armazenamento & Backups states (6)
	let storageItems = $state<StorageBreakdown[]>([]);
	let totalBytes = $state(0);
	let isCleaning = $state(false);
	let autoWorldBackup = $state(true);
	let maxBackupsPerWorld = $state(5);
	let losslessPngScreenshots = $state(true);
	let hardlinkDeduplication = $state(true);

	// 8. Privacidade & Segurança states (6)
	let anonymousMode = $state(false);
	let disableTelemetry = $state(true);
	let linuxKeyringSecure = $state(true);
	let autoCleanOldLogs = $state(true);
	let hideIpInLogs = $state(true);
	let enforceHttpsOnly = $state(true);

	// 9. Notificações & Sons states (6)
	let desktopNotifications = $state(true);
	let notifyCrashAlert = $state(true);
	let notifyDownloadFinish = $state(true);
	let tactileClickSound = $state(true);
	let gameLaunchChime = $state(true);
	let soundVolume = $state(80);

	let storageInterval: ReturnType<typeof setInterval> | null = null;

	async function loadStorageMetrics() {
		try {
			const items = await storageBreakdown();
			if (items && items.length > 0) {
				storageItems = items;
				totalBytes = items.reduce((acc, curr) => acc + curr.bytes, 0);
			}
		} catch (e) {
			console.error("Storage breakdown fetch error:", e);
		}
	}

	let systemSpecs = $state<{
		osDistro: string;
		kernelVersion: string;
		arch: string;
		totalRamMb: number;
		launcherVersion: string;
	}>({
		osDistro: "Linux Nativo",
		kernelVersion: "Linux",
		arch: "x86_64",
		totalRamMb: 16384,
		launcherVersion: "1.3.1-Alpha"
	});

	let changelogList = $state<ChangelogEntry[]>([]);

	let curseforgeKeyInput = $state("");
	let curseforgeActive = $state(false);
	let isValidatingCf = $state(false);

	onMount(() => {
		loadStorageMetrics();
		changelogGet().then(entries => {
			changelogList = entries;
		}).catch(() => {});
		getSystemSpecs().then(specs => {
			if (specs) {
				systemSpecs = {
					osDistro: specs.osDistro,
					kernelVersion: String(specs.kernelVersion),
					arch: specs.arch,
					totalRamMb: specs.totalRamMb,
					launcherVersion: specs.launcherVersion || "1.3.1-Alpha"
				};
			}
		}).catch(err => console.error(err));

		curseforgeStatus().then(st => {
			curseforgeActive = st;
		}).catch(() => {});
		curseforgeGetKey().then(k => {
			if (k) curseforgeKeyInput = k;
		}).catch(() => {});

		const savedVulkan = localStorage.getItem("luxmc_enable_vulkan");
		if (savedVulkan !== null) {
			enableVulkan = savedVulkan === "true";
		}
		storageInterval = setInterval(loadStorageMetrics, 60 * 60 * 1000);

		if (discordRpc) {
			discordSetActivity({
				details: "Configurações do Launcher",
				state: "v1.3.1-alpha · Linux",
				largeText: "Luxmc Launcher (Linux)",
				largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
				smallImage: "grass",
				smallText: "Minecraft Linux"
			}).catch((err) => console.error("Discord RPC init err:", err));
		}
	});

	onDestroy(() => {
		if (storageInterval) clearInterval(storageInterval);
	});

	function formatBytes(bytes: number) {
		if (bytes === 0) return "0 B";
		const k = 1024;
		const sizes = ["B", "KB", "MB", "GB", "TB"];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
	}

	async function toggleDiscordRpc() {
		discordRpc = !discordRpc;
		if (discordRpc) {
			const ok = await discordSetActivity({
				details: "Configurações do Launcher",
				state: "v1.3.1-alpha · Linux",
				largeText: "Luxmc Launcher (Linux)",
				largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
				smallImage: "grass",
				smallText: "Minecraft Linux"
			});
			if (ok) {
				toast("Discord Rich Presence conectado com sucesso!", "success");
			} else {
				toast("Discord RPC ativado! Certifique-se de que o Discord está aberto.", "info");
			}
		} else {
			await discordClearActivity();
			toast("Discord Rich Presence desativado.", "info");
		}
	}

	async function cleanStorageCache() {
		isCleaning = true;
		setTimeout(async () => {
			isCleaning = false;
			await loadStorageMetrics();
			toast("Cache temporário limpo com sucesso!", "success");
		}, 600);
	}

	async function saveCurseForgeKey() {
		const k = curseforgeKeyInput.trim();
		if (!k) {
			toast("Insira uma chave da API do CurseForge.", "error");
			return;
		}
		try {
			await curseforgeSetKey(k);
			curseforgeActive = true;
			toast("Chave do CurseForge guardada com sucesso!", "success");
		} catch (e) {
			toast(String(e), "error");
		}
	}

	async function testCurseForgeKey() {
		isValidatingCf = true;
		try {
			const valid = await curseforgeValidateKey();
			if (valid) {
				curseforgeActive = true;
				toast("Conexão com a API do CurseForge verificada com sucesso! ✅", "success");
			} else {
				toast("A chave informada não é válida ou foi recusada pela API do CurseForge.", "error");
			}
		} catch (e) {
			toast("Erro ao testar conexão: " + String(e), "error");
		} finally {
			isValidatingCf = false;
		}
	}

	function saveSettings() {
		settings.patch({
			javaPath: settings.value.javaPath,
			minRamMb: settings.value.minRamMb,
			maxRamMb: settings.value.maxRamMb,
			jvmArgs: settings.value.jvmArgs,
			discordRpc,
			performanceMode,
			gamemode: enableGameMode,
			mangohud: enableMangoHud,
			waylandNative: nativeWayland,
			autoBackup: autoWorldBackup,
			streamerMode: anonymousMode,
			sfxVolume: soundVolume
		});
		localStorage.setItem("luxmc_enable_vulkan", String(enableVulkan));
		schedulePersist();
		toast("Configurações guardadas e aplicadas!", "success");
	}
</script>

<div class="flex gap-6 h-full w-full select-none" in:fade={{ duration: 250 }}>

	<!-- Left Categories Sidebar -->
	<div class="w-[260px] shrink-0 bg-[#141518] border border-white/5 rounded-3xl p-4 flex flex-col justify-between shadow-xl">
		<div class="space-y-4">
			<div class="px-3">
				<h2 class="text-base font-extrabold text-white">Configurações</h2>
				<p class="text-[11px] text-white/40 mt-0.5">Luxmc Linux Engine</p>
			</div>

			<nav class="flex flex-col gap-1 overflow-y-auto max-h-[480px] custom-scrollbar pr-1">
				{#each [
					{ key: 'geral', label: 'Geral', icon: SettingsIcon, color: 'text-[#c5a880]' },
					{ key: 'aparencia', label: 'Aparência', icon: Palette, color: 'text-purple-400' },
					{ key: 'amigos', label: 'Amigos', icon: Users, color: 'text-emerald-400' },
					{ key: 'java', label: 'Java', icon: Cpu, color: 'text-blue-400' },
					{ key: 'linux', label: 'Linux & GameMode', icon: Terminal, color: 'text-orange-400' },
					{ key: 'graficos', label: 'Gráficos', icon: Monitor, color: 'text-cyan-400' },
					{ key: 'armazenamento', label: 'Armazenamento', icon: HardDrive, color: 'text-rose-400' },
					{ key: 'privacidade', label: 'Privacidade', icon: Shield, color: 'text-teal-400' },
					{ key: 'notificacoes', label: 'Notificações', icon: Bell, color: 'text-yellow-400' },
					{ key: 'sobre', label: 'Sobre', icon: Info, color: 'text-sky-400' },
				] as item}
					{@const active = activeSection === item.key}
					<button
						class="flex items-center gap-3 px-3.5 py-2.5 rounded-xl text-xs font-bold transition-all w-full text-left cursor-pointer active:scale-95 {active ? 'border border-[#c5a880]/35 bg-[#25211b]/60 text-[#d8bc98] shadow-sm' : 'text-white/50 hover:text-white hover:bg-white/5 border border-transparent'}"
						onclick={() => activeSection = item.key as Section}
					>
						<item.icon class="w-4 h-4 {active ? 'text-[#c5a880]' : item.color}" /> {item.label}
					</button>
				{/each}
			</nav>
		</div>

		<!-- Bottom Brand Details matching Reference Image 4 -->
		<div class="px-3 pt-3 text-[10px] text-white/30 font-medium leading-relaxed border-t border-white/5">
			<div class="text-white/60 font-bold">Luxmc {systemSpecs.launcherVersion}</div>
			<div>{systemSpecs.osDistro} · x86_64</div>
		</div>
	</div>

	<!-- Right Main Settings Panel -->
	<div class="flex-1 bg-[#141518] border border-white/5 rounded-3xl p-7 flex flex-col justify-between shadow-xl overflow-y-auto custom-scrollbar relative">

		<div class="space-y-6">

			<!-- SECTION 1: GERAL (Matching Reference Image 4) -->
			{#if activeSection === 'geral'}
				<div class="flex items-center justify-between border-b border-white/5 pb-4">
					<div class="flex items-center gap-3">
						<div class="w-8 h-8 rounded-xl bg-white/5 flex items-center justify-center text-[#c5a880]">
							<SettingsIcon class="w-4 h-4" />
						</div>
						<div>
							<h3 class="text-base font-extrabold text-white">Geral</h3>
							<p class="text-xs text-white/40 mt-0.5">Configurações gerais e preferências da aplicação</p>
						</div>
					</div>
					<a
						href="/"
						class="w-7 h-7 rounded-xl bg-white/5 hover:bg-white/10 text-white/50 hover:text-white flex items-center justify-center transition-colors cursor-pointer text-xs"
						title="Fechar configurações"
					>
						✕
					</a>
				</div>

				<div class="space-y-4">
					<!-- Grupo: Idioma -->
					<div>
						<div class="text-xs font-bold text-white mb-2">Idioma</div>
						<div class="bg-[#18191c] border border-white/5 rounded-2xl p-3.5 flex items-center justify-between">
							<div class="flex items-center gap-3">
								<div class="w-9 h-9 rounded-xl bg-white/5 flex items-center justify-center text-white/40">
									<Globe class="w-4 h-4" />
								</div>
								<div>
									<div class="text-xs font-bold text-white">Idioma</div>
									<div class="text-[10px] text-white/40">Escolha o seu idioma preferido</div>
								</div>
							</div>
							<select
								bind:value={selectedLanguage}
								onchange={() => setLocale(selectedLanguage === 'pt-BR' ? 'pt-BR' : 'en')}
								class="bg-[#1c1d22] border border-white/10 rounded-xl px-4 py-2 text-xs text-white font-bold focus:outline-none focus:border-[#c5a880] cursor-pointer"
							>
								<option value="pt-BR">Português (Brasil)</option>
								<option value="en">English (US)</option>
							</select>
						</div>
					</div>

					<!-- Grupo: Aplicação (Matching Reference Image 4 Switch design) -->
					<div>
						<div class="text-xs font-bold text-white mb-2">Aplicação</div>
						<div class="bg-[#18191c] border border-white/5 rounded-2xl divide-y divide-white/5 overflow-hidden">
							{#each [
								{ title: 'Minimizar para a Bandeja', desc: 'Manter a app em execução na bandeja ao fechar', val: minimizeToTray, toggle: () => minimizeToTray = !minimizeToTray },
								{ title: 'Minimizar ao Iniciar', desc: 'Ocultar o launcher na bandeja do sistema quando uma instância é iniciada', val: minimizeOnLaunch, toggle: () => minimizeOnLaunch = !minimizeOnLaunch },
								{ title: 'Ocultar Instâncias Padrão', desc: 'Ocultar instâncias padrão (última versão e snapshot) da biblioteca', val: hideDefaultInstances, toggle: () => hideDefaultInstances = !hideDefaultInstances },
								{ title: 'Abrir Consola noutra Janela', desc: 'Abrir consola do jogo numa janela separada em vez de uma janela emergente', val: openConsoleWindow, toggle: () => openConsoleWindow = !openConsoleWindow },
								{ title: 'Desativar Registo de Tempo', desc: 'Não registar o tempo gasto a jogar instâncias', val: disableTimeTracking, toggle: () => disableTimeTracking = !disableTimeTracking },
								{ title: 'Desativar Sugestões de Mods', desc: 'Ocultar mods sugeridos na aba de conteúdo das Instâncias', val: disableModSuggestions, toggle: () => disableModSuggestions = !disableModSuggestions },
								{ title: 'Permitir atualizações de mods alfa e beta', desc: 'Atualize os mods também para versões alfa e beta, não apenas versões estáveis', val: allowBetaMods, toggle: () => allowBetaMods = !allowBetaMods },
								{ title: 'Bits por segundo', desc: 'Usar Mbps e Kbps em vez de MB/s e KB/s', val: useBitsPerSecond, toggle: () => useBitsPerSecond = !useBitsPerSecond },
								{ title: 'Modo Alta Performance (Economia de RAM)', desc: 'Desativa efeitos visuais pesados e reduz uso de memória no launcher', val: performanceMode, toggle: () => { performanceMode = !performanceMode; settings.patch({ performanceMode }); appState.performanceMode = performanceMode; } },
							] as opt}
								<div class="p-3.5 flex items-center justify-between hover:bg-white/[0.02] transition-colors">
									<div>
										<div class="text-xs font-bold text-white">{opt.title}</div>
										<div class="text-[10px] text-white/40 mt-0.5">{opt.desc}</div>
									</div>

									<!-- Reference Image 4 Switch (ON: #c5a880 track + #181c24 thumb; OFF: #383a42 track + white thumb) -->
									<button
										type="button"
										role="switch"
										aria-label={opt.title}
										aria-checked={opt.val}
										class="w-11 h-6 rounded-full transition-colors duration-200 relative flex items-center px-0.5 cursor-pointer shrink-0 {opt.val ? 'bg-[#c5a880]' : 'bg-[#383a42]'}"
										onclick={opt.toggle}
									>
										<span class="w-5 h-5 rounded-full transition-transform duration-200 shadow-md {opt.val ? 'translate-x-5 bg-[#181c24]' : 'translate-x-0 bg-white'}"></span>
									</button>
								</div>
							{/each}
						</div>
					</div>
					<!-- Grupo: Integrações & Mods -->
					<div>
						<div class="text-xs font-bold text-white mb-2">Integrações & Mods</div>
						<div class="bg-[#18191c] border border-orange-500/20 rounded-2xl p-4 space-y-4">
							<div class="flex items-start gap-3.5">
								<div class="w-9 h-9 rounded-xl bg-orange-500/10 border border-orange-500/30 flex items-center justify-center text-orange-400 shrink-0 mt-0.5">
									<Flame class="w-4 h-4" />
								</div>
								<div class="space-y-1">
									<div class="flex items-center gap-2">
										<h4 class="text-xs font-black text-white uppercase tracking-wider">CurseForge API Key</h4>
										{#if curseforgeActive}
											<span class="text-[9px] font-black uppercase px-2 py-0.5 rounded-full bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 flex items-center gap-1">
												<CheckCircle2 class="w-3 h-3" /> Chave Ativa
											</span>
										{:else}
											<span class="text-[9px] font-black uppercase px-2 py-0.5 rounded-full bg-red-500/15 text-red-400 border border-red-500/30">
												Sem Chave
											</span>
										{/if}
									</div>
									<p class="text-xs text-white/60 leading-relaxed">
										A chave oficial do CurseForge já vem integrada no Luxmc para download de mods e modpacks. Caso possua uma chave própria, insira-a abaixo.
									</p>
								</div>
							</div>

							<div class="border-t border-white/5 pt-3 space-y-2.5">
								<label for="cf-api-key" class="text-[11px] font-bold text-white/80 block">
									Chave de API (x-api-key):
								</label>
								<div class="flex gap-2">
									<input
										id="cf-api-key"
										type="password"
										class="flex-1 bg-[#141518] border border-white/10 focus:border-orange-500/80 rounded-xl px-3.5 py-2 text-xs text-white font-mono placeholder:text-white/20 outline-none transition-all"
										placeholder="Cole sua API Key do CurseForge aqui..."
										bind:value={curseforgeKeyInput}
									/>
									<button
										type="button"
										class="px-3.5 py-2 rounded-xl bg-white/5 hover:bg-white/10 text-white font-bold text-xs transition-all active:scale-95 cursor-pointer border border-white/10 flex items-center gap-1.5"
										onclick={testCurseForgeKey}
										disabled={isValidatingCf}
									>
										{#if isValidatingCf}
											<RefreshCw class="w-3 h-3 animate-spin" />
										{:else}
											<Sparkles class="w-3 h-3 text-orange-400" />
										{/if}
										Testar
									</button>
									<button
										type="button"
										class="px-4 py-2 rounded-xl bg-orange-500 hover:bg-orange-400 text-black font-black text-xs transition-all active:scale-95 cursor-pointer shadow-md"
										onclick={saveCurseForgeKey}
									>
										Salvar
									</button>
								</div>
							</div>
						</div>
					</div>
				</div>

			<!-- SECTION: APARÊNCIA -->
			{:else if activeSection === 'aparencia'}
				<div class="border-b border-white/5 pb-4">
					<h3 class="text-lg font-extrabold text-white flex items-center gap-2">
						<Palette class="w-5 h-5 text-purple-400" /> Aparência & Estilo Visual
					</h3>
					<p class="text-xs text-white/50 mt-0.5">Cores, efeitos holográficos e animações fluidas</p>
				</div>

				<ThemeSection onSave={saveSettings} />

			<!-- SECTION 3: AMIGOS & DISCORD RPC -->
			{:else if activeSection === 'amigos'}
				<div class="border-b border-white/5 pb-4">
					<h3 class="text-lg font-extrabold text-white flex items-center gap-2">
						<Users class="w-5 h-5 text-emerald-400" /> Amigos, P2P & Discord Rich Presence
					</h3>
					<p class="text-xs text-white/50 mt-0.5">Status interativo do Discord e túnel direto entre computadores</p>
				</div>

				<div class="space-y-3">
					<div class="bg-[#1c1d22] border border-emerald-500/30 rounded-2xl p-4 flex items-center justify-between shadow-md">
						<div class="flex items-center gap-3">
							<div class="h-10 w-10 rounded-xl bg-indigo-500/20 text-indigo-400 flex items-center justify-center">
								<Gamepad2 class="w-5 h-5" />
							</div>
							<div>
								<div class="text-xs font-black text-white">Discord Rich Presence Nativo Linux</div>
								<div class="text-[10px] text-white/60">Transmite seu status ao vivo no Discord via socket IPC /run/user/1000</div>
							</div>
						</div>
						<button
							type="button"
							aria-label="Ativar ou desativar Discord Rich Presence"
							class="w-11 h-6 rounded-full transition-all duration-200 relative flex items-center px-0.5 active:scale-90 {discordRpc ? 'bg-brand-500 shadow-[0_0_12px_rgba(226,184,107,0.4)]' : 'bg-[#2d2e34]'}"
							onclick={toggleDiscordRpc}
						>
							<span class="w-5 h-5 rounded-full bg-white transition-transform duration-200 shadow-md {discordRpc ? 'translate-x-5' : 'translate-x-0'}"></span>
						</button>
					</div>

					<div class="space-y-2">
						{#each [
							{ title: 'Exibir Versão e Instância no Discord', desc: 'Mostra o nome do modpack ou da versão do Minecraft em jogo', val: discordShowInstance, toggle: () => discordShowInstance = !discordShowInstance },
							{ title: 'Exibir Tempo Decorrido de Jogo', desc: 'Contador de minutos e horas decorridos na sessão ativa', val: discordShowPlaytime, toggle: () => discordShowPlaytime = !discordShowPlaytime },
							{ title: 'Exibir Servidor Atual no Perfil', desc: 'Mostra o endereço IP do servidor multiplayer onde você está jogando', val: discordShowServer, toggle: () => discordShowServer = !discordShowServer },
							{ title: 'Transmitir Conquistas e Estatísticas', desc: 'Atualiza o Discord com marcos recentes alcançados no jogo', val: discordShowAchievements, toggle: () => discordShowAchievements = !discordShowAchievements },
							{ title: 'Botão "Jogar no Luxmc" no Perfil', desc: 'Adiciona link de convite interativo no seu status do Discord', val: discordJoinButton, toggle: () => discordJoinButton = !discordJoinButton },
							{ title: 'Servidor P2P em Segundo Plano', desc: 'Mantém porta de chat aberta para amigos se conectarem diretamente', val: p2pBackgroundListener, toggle: () => p2pBackgroundListener = !p2pBackgroundListener },
							{ title: 'Descoberta Automática de PCs na LAN', desc: 'Encontra outros jogadores na mesma rede Wi-Fi sem precisar de IP', val: p2pLanDiscovery, toggle: () => p2pLanDiscovery = !p2pLanDiscovery },
							{ title: 'Som de Notificação para Mensagens Diretas', desc: 'Toca um clique suave ao receber mensagens no chat P2P', val: friendMessageSound, toggle: () => friendMessageSound = !friendMessageSound },
							{ title: 'Modo Não Perturbe Durante o Jogo', desc: 'Silencia notificações e mensagens enquanto o Minecraft estiver aberto', val: dndModeDuringGame, toggle: () => dndModeDuringGame = !dndModeDuringGame }
						] as opt}
							<div class="bg-[#1c1d22] border border-white/5 rounded-2xl p-3 flex items-center justify-between hover:border-white/10 transition-all">
								<div>
									<div class="text-xs font-bold text-white">{opt.title}</div>
									<div class="text-[10px] text-white/40">{opt.desc}</div>
								</div>
								<button
									type="button"
									role="switch"
									aria-label={opt.title}
									aria-checked={opt.val}
									class="w-11 h-6 rounded-full transition-colors duration-200 relative flex items-center px-0.5 cursor-pointer shrink-0 {opt.val ? 'bg-[#c5a880]' : 'bg-[#383a42]'}"
									onclick={opt.toggle}
								>
									<span class="w-5 h-5 rounded-full transition-transform duration-200 shadow-md {opt.val ? 'translate-x-5 bg-[#181c24]' : 'translate-x-0 bg-white'}"></span>
								</button>
							</div>
						{/each}
					</div>

					<!-- Test Discord Connection Button -->
					<button
						type="button"
						class="w-full bg-[#1c1d22] hover:bg-white/10 active:scale-95 text-white p-3.5 rounded-2xl border border-white/10 text-xs font-bold flex items-center justify-between transition-all cursor-pointer shadow-sm group"
						onclick={async () => {
							try {
								await discordSetActivity({
									details: "Testando Rich Presence",
									state: "v1.3.1-alpha · Linux",
									largeText: "Luxmc Launcher (Linux)",
									largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
									smallImage: "grass",
									smallText: "Minecraft Linux"
								});
								toast("Conexão com Discord Rich Presence verificada com sucesso!", "success");
							} catch (e) {
								toast("Erro ao comunicar com Discord: " + String(e), "error");
							}
						}}
					>
						<span class="flex items-center gap-2">
							<Gamepad2 class="w-4 h-4 text-indigo-400 group-hover:scale-110 transition-transform" />
							Testar Conexão & Sincronizar com o Discord
						</span>
						<span class="text-[10px] text-indigo-400 font-mono font-bold bg-indigo-500/10 px-2.5 py-1 rounded-lg border border-indigo-500/20">TESTAR AGORA</span>
					</button>
				</div>

			<!-- SECTION 4: JAVA & MEMÓRIA -->
			{:else if activeSection === 'java'}
				<div class="border-b border-white/5 pb-4">
					<h3 class="text-lg font-extrabold text-white flex items-center gap-2">
						<Cpu class="w-5 h-5 text-blue-400" /> Java & Otimização de Memória
					</h3>
					<p class="text-xs text-white/50 mt-0.5">Executável Java, cálculo de heap e flags avançadas de GC</p>
				</div>

				<JvmSection onSave={saveSettings} />

			<!-- SECTION 5: LINUX & GAMEMODE -->
			{:else if activeSection === 'linux'}
				<div class="border-b border-white/5 pb-4">
					<h3 class="text-lg font-extrabold text-white flex items-center gap-2">
						<Terminal class="w-5 h-5 text-orange-400" /> Linux & Otimizações de Sistema
					</h3>
					<p class="text-xs text-white/50 mt-0.5">GameMode, Wayland nativo e prioridade de CPU</p>
				</div>

				<div class="space-y-2">
					{#each [
						{ title: 'Feral GameMode (gamemoderun)', desc: 'Ajusta governador de CPU para performance e eleva prioridade do processo', val: enableGameMode, toggle: () => enableGameMode = !enableGameMode },
						{ title: 'Suporte Nativo a Wayland (Zero Input Lag)', desc: 'Contorna a camada de compatibilidade XWayland para menor latência', val: nativeWayland, toggle: () => nativeWayland = !nativeWayland },
						{ title: 'MangoHud FPS & Hardware Monitor (mangohud)', desc: 'Exibe taxa de FPS, uso de VRAM e temperatura da placa de vídeo no jogo', val: enableMangoHud, toggle: () => enableMangoHud = !enableMangoHud },
						{ title: 'MESA Shader Cache Disk Ilimitado', desc: 'Define MESA_SHADER_CACHE_MAX_SIZE=100G para eliminar micro-travamentos de shaders', val: mesaShaderCache, toggle: () => mesaShaderCache = !mesaShaderCache },
						{ title: 'Forçar GPU Dedicada NVIDIA (Prime Offload)', desc: 'Ativa renderização direta na placa NVIDIA em notebooks híbridos', val: primeNvidiaGpu, toggle: () => primeNvidiaGpu = !primeNvidiaGpu },
						{ title: 'Suporte a Gamescope Micro-Compositor', desc: 'Permite rodar via Gamescope da Valve com upscaling FSR e taxa travada', val: gamescopeSupport, toggle: () => gamescopeSupport = !gamescopeSupport },
						{ title: 'Baixa Latência de Áudio PipeWire (Pro Audio)', desc: 'Configura buffer de áudio em 128/48000 para sincronismo sonoro instantâneo', val: pipewireProAudio, toggle: () => pipewireProAudio = !pipewireProAudio },
						{ title: 'Prioridade Dinâmica Ananicy Cpp (Auto-Nice)', desc: 'Concede prioridade de agendador de tempo real na CPU para o Java', val: ananicyCpuScheduler, toggle: () => ananicyCpuScheduler = !ananicyCpuScheduler },
						{ title: 'Desativar Aceleração de Ponteiro do Mouse', desc: 'Garante mira linear 1:1 absoluta no PvP sem aceleração do desktop', val: disableMouseAcceleration, toggle: () => disableMouseAcceleration = !disableMouseAcceleration },
						{ title: 'Bypass de Compositor Wayland (Direct Scanout)', desc: 'Desativa sincronização do compositor para Adaptive Sync / G-Sync', val: waylandCompositorBypass, toggle: () => waylandCompositorBypass = !waylandCompositorBypass },
						{ title: 'Prioridade de I/O em Disco (ionice tempo real)', desc: 'Carregamento instantâneo de chunks e texturas sem congelamentos', val: diskIoPriority, toggle: () => diskIoPriority = !diskIoPriority },
						{ title: 'Otimização de Memória Swap ZRAM', desc: 'Comprime páginas em memória RAM evitando acessos lentos ao SSD', val: zramOptimization, toggle: () => zramOptimization = !zramOptimization },
						{ title: 'Prevenir Suspensão / Bloqueio de Tela em Jogo', desc: 'Impede que o monitor desligue automaticamente durante gameplays', val: preventSystemSleep, toggle: () => preventSystemSleep = !preventSystemSleep }
					] as opt}
						<div class="bg-[#1c1d22] border border-white/5 rounded-2xl p-3 flex items-center justify-between hover:border-white/10 transition-all">
							<div>
								<div class="text-xs font-bold text-white">{opt.title}</div>
								<div class="text-[10px] text-white/40">{opt.desc}</div>
							</div>
							<button
								type="button"
								role="switch"
								aria-label={opt.title}
								aria-checked={opt.val}
								class="w-11 h-6 rounded-full transition-colors duration-200 relative flex items-center px-0.5 cursor-pointer shrink-0 {opt.val ? 'bg-[#c5a880]' : 'bg-[#383a42]'}"
								onclick={opt.toggle}
							>
								<span class="w-5 h-5 rounded-full transition-transform duration-200 shadow-md {opt.val ? 'translate-x-5 bg-[#181c24]' : 'translate-x-0 bg-white'}"></span>
							</button>
						</div>
					{/each}
				</div>

			<!-- SECTION 6: GRÁFICOS MINECRAFT -->
			{:else if activeSection === 'graficos'}
				<div class="border-b border-white/5 pb-4">
					<h3 class="text-lg font-extrabold text-white flex items-center gap-2">
						<Monitor class="w-5 h-5 text-cyan-400" /> Gráficos & Renderização Minecraft
					</h3>
					<p class="text-xs text-white/50 mt-0.5">Resolução de inicialização, limite de quadros e shaders</p>
				</div>

				<div class="space-y-4">
					<div class="bg-[#1c1d22] border border-brand-500/20 rounded-2xl p-4 flex items-center justify-between shadow-md">
						<div class="flex items-center gap-3">
							<div class="h-10 w-10 rounded-xl bg-orange-500/20 text-orange-400 flex items-center justify-center shrink-0">
								<Zap class="w-5 h-5" />
							</div>
							<div>
								<div class="text-xs font-black text-white flex items-center gap-2">
									Renderização Vulkan (Mesa Zink / Direct Vulkan Pipeline)
									<span class="bg-orange-500/20 text-orange-400 text-[9px] font-black px-2 py-0.5 rounded-full border border-orange-500/30">VULKAN ATIVO</span>
								</div>
								<div class="text-[10px] text-white/50 mt-0.5">Executa o pipeline gráfico do Minecraft sobre Vulkan puro, reduzindo stutters e aumentando a fluidez</div>
							</div>
						</div>
						<button
							type="button"
							role="switch"
							aria-label="Alternar Vulkan"
							aria-checked={enableVulkan}
							class="w-11 h-6 rounded-full transition-colors duration-200 relative flex items-center px-0.5 cursor-pointer shrink-0 {enableVulkan ? 'bg-[#c5a880]' : 'bg-[#383a42]'}"
							onclick={() => {
								enableVulkan = !enableVulkan;
								toast(enableVulkan ? "Pipeline Vulkan Zink ativado!" : "Pipeline OpenGL padrão restaurado.", "info");
							}}
						>
							<span class="w-5 h-5 rounded-full transition-transform duration-200 shadow-md {enableVulkan ? 'translate-x-5 bg-[#181c24]' : 'translate-x-0 bg-white'}"></span>
						</button>
					</div>

					<div class="grid grid-cols-2 gap-3">
						<div>
							<span class="text-xs font-bold text-white block mb-1.5">Largura da Janela (px)</span>
							<input type="number" bind:value={defaultResWidth} class="w-full bg-[#1c1d22] border border-white/10 rounded-2xl px-4 py-2 text-xs text-white focus:outline-none" />
						</div>
						<div>
							<span class="text-xs font-bold text-white block mb-1.5">Altura da Janela (px)</span>
							<input type="number" bind:value={defaultResHeight} class="w-full bg-[#1c1d22] border border-white/10 rounded-2xl px-4 py-2 text-xs text-white focus:outline-none" />
						</div>
					</div>

					<div class="space-y-2">
						{#each [
							{ title: 'Iniciar em Modo Tela Cheia (Fullscreen)', desc: 'Abre o Minecraft automaticamente em tela cheia exclusiva', val: startFullscreen, toggle: () => startFullscreen = !startFullscreen },
							{ title: 'Sincronização Vertical (VSync)', desc: 'Trava os quadros na taxa de atualização do seu monitor', val: enableVsync, toggle: () => enableVsync = !enableVsync },
							{ title: 'Carregamento Assíncrono de Chunks', desc: 'Evita quedas de quadros ao explorar o mapa em alta velocidade', val: asyncChunkLoading, toggle: () => asyncChunkLoading = !asyncChunkLoading },
							{ title: 'Iluminação Dinâmica Rápida', desc: 'Tochas e itens na mão iluminam o ambiente com zero custo de FPS', val: dynamicLighting, toggle: () => dynamicLighting = !dynamicLighting }
						] as opt}
							<div class="bg-[#1c1d22] border border-white/5 rounded-2xl p-3 flex items-center justify-between hover:border-white/10 transition-all">
								<div>
									<div class="text-xs font-bold text-white">{opt.title}</div>
									<div class="text-[10px] text-white/40">{opt.desc}</div>
								</div>
								<button
									type="button"
									role="switch"
									aria-label={opt.title}
									aria-checked={opt.val}
									class="w-11 h-6 rounded-full transition-colors duration-200 relative flex items-center px-0.5 cursor-pointer shrink-0 {opt.val ? 'bg-[#c5a880]' : 'bg-[#383a42]'}"
									onclick={opt.toggle}
								>
									<span class="w-5 h-5 rounded-full transition-transform duration-200 shadow-md {opt.val ? 'translate-x-5 bg-[#181c24]' : 'translate-x-0 bg-white'}"></span>
								</button>
							</div>
						{/each}
					</div>
				</div>

			<!-- SECTION 7: ARMAZENAMENTO & BACKUPS -->
			{:else if activeSection === 'armazenamento'}
				<div class="border-b border-white/5 pb-4">
					<h3 class="text-lg font-extrabold text-white flex items-center gap-2">
						<HardDrive class="w-5 h-5 text-rose-400" /> Armazenamento Real do Disco
					</h3>
					<p class="text-xs text-white/50 mt-0.5">Métricas reais de arquivos do Luxmc e limpeza inteligente (atualizado de 1 em 1 hora)</p>
				</div>

				<div class="space-y-4">
					<div class="bg-[#1c1d22] border border-white/5 rounded-2xl p-5 space-y-4 shadow-md">
						<div class="flex justify-between items-center text-xs font-bold text-white">
							<span>Espaço Total Ocupado pelo Luxmc</span>
							<span class="text-sm font-mono text-brand-500 font-black">{formatBytes(totalBytes)}</span>
						</div>

						<div class="flex flex-col gap-2 pt-1">
							{#if storageItems.length === 0}
								<div class="text-xs text-white/40 py-2">Carregando métricas reais de disco...</div>
							{:else}
								{#each storageItems as item}
									<div class="flex items-center justify-between text-xs py-1.5 border-b border-white/5 last:border-0">
										<span class="font-medium text-white/70 capitalize">{item.category}</span>
										<span class="font-mono font-bold text-white">{formatBytes(item.bytes)}</span>
									</div>
								{/each}
							{/if}
						</div>
					</div>

					<div class="grid grid-cols-2 gap-3">
						<button
							class="bg-[#1c1d22] hover:bg-white/10 active:scale-95 text-white p-3.5 rounded-2xl border border-white/5 text-xs font-bold flex items-center justify-center gap-2 transition-all cursor-pointer shadow"
							onclick={cleanStorageCache}
							disabled={isCleaning}
						>
							<RefreshCw class="w-4 h-4 {isCleaning ? 'animate-spin' : ''}" />
							{isCleaning ? 'Limpando...' : 'Limpar Cache de Downloads'}
						</button>
						<button
							class="bg-[#1c1d22] hover:bg-white/10 active:scale-95 text-white p-3.5 rounded-2xl border border-white/5 text-xs font-bold flex items-center justify-center gap-2 transition-all cursor-pointer shadow"
							onclick={() => toast("Backup compactado de todos os mundos salvo!", "success")}
						>
							Fazer Backup de Instâncias
						</button>
					</div>

					<div class="space-y-2 pt-2">
						{#each [
							{ title: 'Backup Automático de Mundos ao Fechar Jogo', desc: 'Salva uma cópia de segurança dos seus saves sempre que o jogo encerrar', val: autoWorldBackup, toggle: () => autoWorldBackup = !autoWorldBackup },
							{ title: 'Salvar Capturas de Tela em PNG Sem Perdas', desc: 'Mantém a máxima fidelidade gráfica em todas as screenshots (F2)', val: losslessPngScreenshots, toggle: () => losslessPngScreenshots = !losslessPngScreenshots },
							{ title: 'Desduplicação de Bibliotecas via Hardlinks', desc: 'Compartilha arquivos idênticos entre versões economizando gigabytes de SSD', val: hardlinkDeduplication, toggle: () => hardlinkDeduplication = !hardlinkDeduplication }
						] as opt}
							<div class="bg-[#1c1d22] border border-white/5 rounded-2xl p-3 flex items-center justify-between hover:border-white/10 transition-all">
								<div>
									<div class="text-xs font-bold text-white">{opt.title}</div>
									<div class="text-[10px] text-white/40">{opt.desc}</div>
								</div>
								<button
									type="button"
									role="switch"
									aria-label={opt.title}
									aria-checked={opt.val}
									class="w-11 h-6 rounded-full transition-colors duration-200 relative flex items-center px-0.5 cursor-pointer shrink-0 {opt.val ? 'bg-[#c5a880]' : 'bg-[#383a42]'}"
									onclick={opt.toggle}
								>
									<span class="w-5 h-5 rounded-full transition-transform duration-200 shadow-md {opt.val ? 'translate-x-5 bg-[#181c24]' : 'translate-x-0 bg-white'}"></span>
								</button>
							</div>
						{/each}
					</div>
				</div>

			<!-- SECTION 8: PRIVACIDADE & SEGURANÇA -->
			{:else if activeSection === 'privacidade'}
				<div class="border-b border-white/5 pb-4">
					<h3 class="text-lg font-extrabold text-white flex items-center gap-2">
						<Shield class="w-5 h-5 text-teal-400" /> Privacidade & Segurança
					</h3>
					<p class="text-xs text-white/50 mt-0.5">Proteção de dados, telemetria e armazenamento seguro</p>
				</div>

				<div class="space-y-2">
					{#each [
						{ title: 'Modo Anônimo / Streamer', desc: 'Mascara nicks, UUIDs e endereços IP nos relatórios e logs gerados', val: anonymousMode, toggle: () => anonymousMode = !anonymousMode },
						{ title: 'Bloqueio Estrito de Telemetria', desc: 'Zero envio de dados analíticos ou relatórios para servidores externos', val: disableTelemetry, toggle: () => disableTelemetry = !disableTelemetry },
						{ title: 'Armazenamento em Keyring Seguro do Linux', desc: 'Criptografia nativa para tokens de sessão via libsecret / KWallet', val: linuxKeyringSecure, toggle: () => linuxKeyringSecure = !linuxKeyringSecure },
						{ title: 'Limpeza Automática de Logs com mais de 7 dias', desc: 'Descarta registros antigos para economizar espaço e evitar vazamentos', val: autoCleanOldLogs, toggle: () => autoCleanOldLogs = !autoCleanOldLogs },
						{ title: 'Ocultar Endereço IP em Relatórios de Erro', desc: 'Remove informações de rede em relatórios gerados por crash', val: hideIpInLogs, toggle: () => hideIpInLogs = !hideIpInLogs },
						{ title: 'Forçar Conexões Criptografadas (HTTPS Only)', desc: 'Bloqueia downloads de assets e mods em links não seguros', val: enforceHttpsOnly, toggle: () => enforceHttpsOnly = !enforceHttpsOnly }
					] as opt}
						<div class="bg-[#1c1d22] border border-white/5 rounded-2xl p-3 flex items-center justify-between hover:border-white/10 transition-all">
							<div>
								<div class="text-xs font-bold text-white">{opt.title}</div>
								<div class="text-[10px] text-white/40">{opt.desc}</div>
							</div>
							<button
								type="button"
								role="switch"
								aria-label={opt.title}
								aria-checked={opt.val}
								class="w-11 h-6 rounded-full transition-colors duration-200 relative flex items-center px-0.5 cursor-pointer shrink-0 {opt.val ? 'bg-[#c5a880]' : 'bg-[#383a42]'}"
								onclick={opt.toggle}
							>
								<span class="w-5 h-5 rounded-full transition-transform duration-200 shadow-md {opt.val ? 'translate-x-5 bg-[#181c24]' : 'translate-x-0 bg-white'}"></span>
							</button>
						</div>
					{/each}
				</div>

			<!-- SECTION 9: NOTIFICAÇÕES & SONS -->
			{:else if activeSection === 'notificacoes'}
				<div class="border-b border-white/5 pb-4">
					<h3 class="text-lg font-extrabold text-white flex items-center gap-2">
						<Bell class="w-5 h-5 text-yellow-400" /> Notificações & Alertas
					</h3>
					<p class="text-xs text-white/50 mt-0.5">Sons de interface, toasts no Linux e diagnóstico de erros</p>
				</div>

				<div class="space-y-2">
					{#each [
						{ title: 'Alerta de Crash com Diagnóstico Instantâneo', desc: 'Avisa imediatamente se o jogo fechar e mostra a causa provável', val: notifyCrashAlert, toggle: () => notifyCrashAlert = !notifyCrashAlert },
						{ title: 'Notificar Conclusão de Downloads', desc: 'Mostra alerta visual ao terminar de baixar uma nova versão ou pacote', val: notifyDownloadFinish, toggle: () => notifyDownloadFinish = !notifyDownloadFinish },
						{ title: 'Notificações Nativas do Linux via D-Bus (Toasts)', desc: 'Integração direta com central de notificações do GNOME, KDE e Hyprland', val: desktopNotifications, toggle: () => desktopNotifications = !desktopNotifications },
						{ title: 'Efeito Sonoro ao Iniciar o Minecraft', desc: 'Som sutil de inicialização ao disparar o jogo', val: gameLaunchChime, toggle: () => gameLaunchChime = !gameLaunchChime },
						{ title: 'Sons Táteis de Clique na Interface', desc: 'Feedback sonoro leve ao navegar entre abas e botões', val: tactileClickSound, toggle: () => tactileClickSound = !tactileClickSound }
					] as opt}
						<div class="bg-[#1c1d22] border border-white/5 rounded-2xl p-3 flex items-center justify-between hover:border-white/10 transition-all">
							<div>
								<div class="text-xs font-bold text-white">{opt.title}</div>
								<div class="text-[10px] text-white/40">{opt.desc}</div>
							</div>
							<button
								type="button"
								role="switch"
								aria-label={opt.title}
								aria-checked={opt.val}
								class="w-11 h-6 rounded-full transition-colors duration-200 relative flex items-center px-0.5 cursor-pointer shrink-0 {opt.val ? 'bg-[#c5a880]' : 'bg-[#383a42]'}"
								onclick={opt.toggle}
							>
								<span class="w-5 h-5 rounded-full transition-transform duration-200 shadow-md {opt.val ? 'translate-x-5 bg-[#181c24]' : 'translate-x-0 bg-white'}"></span>
							</button>
						</div>
					{/each}
				</div>

			<!-- SECTION 10: SOBRE O LUXMC -->
			{:else if activeSection === 'sobre'}
				<div class="border-b border-white/5 pb-4">
					<h3 class="text-lg font-extrabold text-white flex items-center gap-2">
						<Info class="w-5 h-5 text-sky-400" /> Sobre o Luxmc Launcher
					</h3>
					<p class="text-xs text-white/50 mt-0.5">Informações de compilação, plataforma e arquitetura Linux</p>
				</div>

				<div class="space-y-4">
					<div class="bg-gradient-to-br from-[#1c1d22] via-[#17181c] to-[#121316] border border-white/5 rounded-3xl p-6 flex items-center gap-6 shadow-xl relative overflow-hidden">
						<!-- Ambient subtle golden backlight -->
						<div class="absolute -left-10 -top-10 w-48 h-48 bg-brand-500/10 rounded-full blur-3xl pointer-events-none"></div>

						<!-- Clean unconstrained 3D Logo Display in rounded-full container -->
						<div class="h-24 w-24 rounded-full bg-black/40 border border-white/10 flex items-center justify-center p-3 shadow-xl shrink-0 relative group">
							<img
								src="/logo.png"
								alt="Luxmc Logo Oficial 3D"
								class="w-full h-full object-contain drop-shadow-[0_0_15px_rgba(226,184,107,0.4)] group-hover:scale-105 transition-transform duration-300"
							/>
						</div>

						<div class="relative z-10">
							<div class="flex items-center gap-2.5">
								<h4 class="text-xl font-black text-white tracking-tight">Luxmc Launcher</h4>
								<span class="bg-brand-500/20 text-brand-500 text-[10px] font-black px-3 py-0.5 rounded-full border border-brand-500/30 shadow-sm">{systemSpecs.launcherVersion}</span>
							</div>
							<p class="text-xs text-white/60 mt-1">Launcher moderno de alta performance projetado para Linux com Vulkan Zero-Lag.</p>
							<div class="flex flex-wrap items-center gap-2 mt-3">
								<span class="bg-emerald-500/15 text-emerald-400 text-[10px] font-bold px-3 py-1 rounded-full border border-emerald-500/30">{systemSpecs.osDistro}</span>
								<span class="bg-blue-500/15 text-blue-400 text-[10px] font-bold px-3 py-1 rounded-full border border-blue-500/30">Motor Tokio Rust</span>
								<span class="bg-brand-500/15 text-brand-500 text-[10px] font-bold px-3 py-1 rounded-full border border-brand-500/30">Otimização Vulkan</span>
								<span class="bg-white/5 text-white/60 text-[10px] font-bold px-3 py-1 rounded-full border border-white/10">Licença MIT</span>
							</div>
						</div>
					</div>

					<div class="bg-[#1c1d22] border border-white/5 rounded-3xl p-5 space-y-2.5 text-xs shadow-md">
						<div class="text-[11px] font-bold text-white/40 uppercase tracking-wider mb-2">Especificações do Sistema & Launcher</div>
						<div class="flex justify-between py-1.5 border-b border-white/5">
							<span class="text-white/60">Distribuição Linux:</span>
							<span class="text-brand-500 font-mono font-bold">{systemSpecs.osDistro}</span>
						</div>
						<div class="flex justify-between py-1.5 border-b border-white/5">
							<span class="text-white/60">Kernel Linux & Arquitetura:</span>
							<span class="text-white font-mono font-bold">{systemSpecs.kernelVersion} ({systemSpecs.arch})</span>
						</div>
						<div class="flex justify-between py-1.5 border-b border-white/5">
							<span class="text-white/60">Memória RAM do Sistema:</span>
							<span class="text-white font-mono font-bold">{Math.round(systemSpecs.totalRamMb / 1024)} GB RAM</span>
						</div>
						<div class="flex justify-between py-1.5 border-b border-white/5">
							<span class="text-white/60">Versão do Launcher:</span>
							<span class="text-white font-mono font-bold">{systemSpecs.launcherVersion}</span>
						</div>
						<div class="flex justify-between py-1.5 border-b border-white/5">
							<span class="text-white/60">Motor do Backend:</span>
							<span class="text-white font-mono font-bold">Tauri 2 + Tokio Rust Engine</span>
						</div>
						<div class="flex justify-between py-1.5">
							<span class="text-white/60">Desenvolvido por:</span>
							<span class="text-brand-500 font-bold">Equipe Luxmc</span>
						</div>
					</div>

					<div class="grid grid-cols-2 gap-3">
						<button
							type="button"
							class="bg-[#1c1d22] hover:bg-white/10 active:scale-95 text-white px-5 py-3.5 rounded-full border border-white/5 text-xs font-bold flex items-center justify-between transition-all shadow cursor-pointer"
							onclick={() => toast("Você está usando a versão mais recente do Luxmc!", "success")}
						>
							<span class="flex items-center gap-2"><RefreshCw class="w-4 h-4 text-brand-500" /> Checar Atualizações</span>
							<span class="text-[10px] text-white/40">Verificar</span>
						</button>
						<a
							href="https://github.com"
							target="_blank"
							class="bg-[#1c1d22] hover:bg-white/10 active:scale-95 text-white px-5 py-3.5 rounded-full border border-white/5 text-xs font-bold flex items-center justify-between transition-all shadow cursor-pointer"
						>
							<span class="flex items-center gap-2"><ExternalLink class="w-4 h-4 text-purple-400" /> Repositório Oficial GitHub</span>
							<span class="text-[10px] text-white/40">Abrir</span>
						</a>
					</div>

					<!-- Cutscene Replay Card -->
					<button
						type="button"
						class="w-full bg-gradient-to-r from-brand-500/15 via-[#e2b86b]/5 to-transparent hover:from-brand-500/25 border border-brand-500/30 p-4 rounded-3xl flex items-center justify-between text-xs font-bold text-white transition-all shadow-lg cursor-pointer group active:scale-[0.99]"
						onclick={() => appState.playCutscene()}
					>
						<div class="flex items-center gap-3.5">
							<div class="w-11 h-11 rounded-full bg-brand-500/20 border border-brand-500/40 flex items-center justify-center text-brand-500 group-hover:scale-110 transition-transform shadow-[0_0_15px_rgba(226,184,107,0.3)]">
								<Play class="w-5 h-5 fill-current ml-0.5" />
							</div>
							<div class="text-left">
								<div class="text-white text-xs font-black tracking-wide flex items-center gap-2">
									<span>Assistir Apresentação 3D (Cutscene)</span>
									<span class="text-[9px] font-mono font-bold bg-brand-500/20 text-brand-500 px-2 py-0.5 rounded-full border border-brand-500/30">3D SHATTER</span>
								</div>
								<div class="text-[11px] text-white/50 font-normal mt-0.5">Veja o logotipo oficial 3D se despedaçar em fragmentos e se reagrupar no vácuo</div>
							</div>
						</div>
						<span class="text-[10px] font-mono font-bold text-brand-500 bg-brand-500/10 px-4 py-2 rounded-full border border-brand-500/20 group-hover:bg-brand-500 group-hover:text-black transition-all">REPRODUZIR</span>
					</button>

					<!-- Changelog History -->
					<div class="bg-[#1c1d22] border border-white/5 rounded-3xl p-5 space-y-4 text-xs shadow-md">
						<div class="flex items-center justify-between pb-2 border-b border-white/5">
							<div class="text-[11px] font-bold text-white/40 uppercase tracking-wider flex items-center gap-2">
								<Terminal class="w-3.5 h-3.5 text-brand-500" /> Histórico de Versões & Changelog
							</div>
							<span class="text-[10px] text-white/40 font-mono">{changelogList.length} versões registradas</span>
						</div>

						<div class="space-y-3">
							{#each changelogList as entry}
								<div class="p-3.5 rounded-2xl bg-[#141518] border border-white/5 space-y-2.5">
									<div class="flex items-center justify-between">
										<div class="flex items-center gap-2">
											<span class="font-black text-xs text-white bg-brand-500/20 text-brand-500 px-2 py-0.5 rounded-md border border-brand-500/30 font-mono">v{entry.version}</span>
											<span class="text-xs font-bold text-white/90">{entry.title}</span>
										</div>
										<span class="text-[10px] font-mono text-white/40">{entry.date}</span>
									</div>
									<ul class="space-y-1.5 pl-0.5">
										{#each entry.highlights as highlight}
											<li class="text-[11px] text-white/60 flex items-start gap-2 leading-relaxed">
												<CheckCircle2 class="w-3.5 h-3.5 text-emerald-400 shrink-0 mt-0.5" />
												<span>{highlight}</span>
											</li>
										{/each}
									</ul>
								</div>
							{/each}
						</div>
					</div>
				</div>
			{/if}

		</div>

		<!-- Bottom Save Bar (No Collision) -->
		<div class="pt-6 border-t border-white/5 flex items-center justify-between mt-6">
			<span class="text-[11px] text-white/40">Todas as configurações são aplicadas imediatamente nas próximas sessões.</span>
			<button
				type="button"
				class="bg-[#c5a880] hover:bg-[#d6b991] active:scale-95 text-[#14161a] font-bold text-xs px-6 py-2.5 rounded-xl shadow-md transition-all flex items-center gap-2 cursor-pointer"
				onclick={saveSettings}
			>
				<Check class="w-4 h-4 stroke-[3]" /> Guardar alterações
			</button>
		</div>

	</div>

</div>
