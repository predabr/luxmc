<script lang="ts">
	import { onMount } from "svelte";
	import {
		Home,
		Users,
		Globe,
		Palette,
		Coffee,
		Terminal,
		ShieldCheck,
		FolderOpen,
		Check,
		ExternalLink,
		RefreshCw,
		Trash2,
		AlertCircle,
		Download,
		Sparkles
	} from "lucide-svelte";
	import { settings, type AppSettings } from "$lib/stores/settings.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { updaterStore } from "$lib/stores/updater.svelte";
	import { themeStore, THEMES, ACCENTS } from "$lib/stores/theme.svelte";
	import { setLocale, schedulePersist } from "$lib/stores/persistence.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { open } from "@tauri-apps/plugin-dialog";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import { appDataDir } from "@tauri-apps/api/path";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import ThemeSection from "$lib/components/settings/ThemeSection.svelte";

	const { t } = useTranslation();

	type SettingsTab =
		| "general"
		| "accounts"
		| "language"
		| "appearance"
		| "java"
		| "commands"
		| "privacy"
		| "runtime";

	let activeTab = $state<SettingsTab>("general");

	let releaseChannel = $state("stable");
	let concurrentDownloads = $state(settings.value.animations ? 5 : 3);
	let gameResolution = $state(settings.value.startFullscreen ? "fullscreen" : "1920x1080");
	let discordIntegration = $state(settings.value.discordRpc !== false);
	let launcherAction = $state<"keep_open" | "hide_reopen" | "close">(
		settings.value.launcherActionOnLaunch ?? "hide_reopen"
	);
	let showCloseWarning = $state(true);
	let performanceMode = $state(settings.value.performanceMode ?? false);

	const currentUsername = $derived(account.value?.username || "Nenhuma conta");
	const isMicrosoft = $derived(
		Boolean(
			account.value?.minecraftToken &&
			!account.value?.id.startsWith("offline_") &&
			!account.value?.id.startsWith("offline-")
		)
	);

	let currentLang = $state<"pt-BR" | "en" | "es">(settings.value.language === "pt-BR" ? "pt-BR" : settings.value.language === "es" ? "es" : "en");

	let selectedTheme = $state(settings.value.theme || "default-dark");
	let selectedAccent = $state(settings.value.accentTheme || themeStore.accent || "blue");
	let density = $state(settings.value.density || "comfortable");

	let maxRamGb = $state(Math.round((settings.value.maxRamMb || 4096) / 1024));
	let javaPathInput = $state(settings.value.javaPath || "");
	let jvmArgsInput = $state(settings.value.jvmArgs || "-XX:+UseG1GC -Dsun.rmi.dgc.server.gcInterval=2147483646");
	let useVulkan = $state(false);
	let waylandNative = $state(settings.value.waylandNative ?? false);

	const ramPresets = [2, 4, 6, 8, 10, 12, 16];

	function setRamPreset(gb: number) {
		maxRamGb = gb;
		saveJava();
	}

	function setJvmPreset(preset: "g1gc" | "aikar" | "zgc" | "shenandoah") {
		if (preset === "g1gc") {
			jvmArgsInput = "-XX:+UseG1GC -Dsun.rmi.dgc.server.gcInterval=2147483646";
		} else if (preset === "aikar") {
			jvmArgsInput = "-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20 -XX:G1HeapWastePercent=5 -XX:G1MixedGCCountTarget=4 -XX:InitiatingHeapOccupancyPercent=15 -XX:G1MixedGCLiveThresholdPercent=90 -XX:G1RSetUpdatingPauseTimePercent=5 -XX:SurvivorRatio=32 -XX:+PerfDisableSharedMem -XX:MaxTenuringThreshold=1";
		} else if (preset === "zgc") {
			jvmArgsInput = "-XX:+UseZGC -XX:+UnlockExperimentalVMOptions -XX:+ZGenerational";
		} else if (preset === "shenandoah") {
			jvmArgsInput = "-XX:+UseShenandoahGC -XX:+UnlockExperimentalVMOptions -XX:ShenandoahGCHeuristics=adaptive";
		}
		saveJava();
	}

	let preLaunchCmd = $state("");
	let postExitCmd = $state("");
	let gameWrapper = $state(settings.value.gamemode ? "gamemoderun" : "");

	let streamerMode = $state(settings.value.streamerMode ?? false);
	let hideDiscordDetails = $state(settings.value.hideDiscordDetails ?? false);
	let anonymousTelemetry = $state(settings.value.anonymousTelemetry ?? true);

	let runtimePath = $state("~/.local/share/luxmc");

	onMount(async () => {
		try {
			const dataDir = await appDataDir();
			if (dataDir) runtimePath = dataDir;
		} catch {}
		if (typeof window !== "undefined") {
			useVulkan = localStorage.getItem("luxmc_enable_vulkan") === "true";
		}
	});

	function saveGeneral() {
		settings.patch({
			discordRpc: discordIntegration,
			launcherActionOnLaunch: launcherAction,
			startFullscreen: gameResolution === "fullscreen",
			performanceMode: performanceMode,
		});
		schedulePersist();
	}

	function savePrivacy() {
		settings.patch({
			streamerMode,
			hideDiscordDetails,
			anonymousTelemetry,
		});
		schedulePersist();
	}

	function handleLangChange(lang: "pt-BR" | "en" | "es") {
		currentLang = lang;
		setLocale(lang);
		settings.patch({ language: lang });
		schedulePersist();
		toast("Idioma alterado com sucesso!", "info");
	}

	function handleThemeChange(tName: AppSettings["theme"]) {
		selectedTheme = tName;
		themeStore.setTheme(tName);
		settings.patch({ theme: tName });
		schedulePersist();
	}

	function handleAccentChange(acc: AppSettings["accentTheme"]) {
		selectedAccent = acc;
		themeStore.setAccent(acc);
		settings.patch({ accentTheme: acc });
		schedulePersist();
	}

	function handleDensityChange(d: AppSettings["density"]) {
		density = d;
		settings.patch({ density: d });
		schedulePersist();
	}

	function saveJava() {
		settings.patch({
			maxRamMb: maxRamGb * 1024,
			javaPath: javaPathInput ? javaPathInput.trim() : undefined,
			jvmArgs: jvmArgsInput ? jvmArgsInput.trim() : undefined,
			waylandNative: waylandNative
		});
		if (typeof window !== "undefined") {
			localStorage.setItem("luxmc_enable_vulkan", String(useVulkan));
		}
		schedulePersist();
		toast("Configurações do Java salvas!", "success");
	}

	async function browseJavaPath() {
		try {
			const sel = await open({
				multiple: false,
				title: "Selecionar Executável Java"
			});
			if (sel && typeof sel === "string") {
				javaPathInput = sel;
				saveJava();
			}
		} catch (e) {
			toast("Erro ao selecionar caminho: " + String(e), "error");
		}
	}

	async function handleClearCache() {
		try {
			const keys = ["luxmc_cache_mods", "luxmc_cache_search", "luxmc_temp_skins"];
			for (const k of keys) localStorage.removeItem(k);
		} catch {}
		toast(t("settings.cacheCleared") || "Cache limpo com sucesso!", "success");
	}
</script>

<div class="h-full flex flex-col select-none overflow-y-auto custom-scrollbar pr-2 pb-16 max-w-5xl mx-auto w-full">
	
	<div class="flex items-center gap-1.5 p-1.5 rounded-2xl bg-bg-elevated border border-fg/5 mb-8 overflow-x-auto custom-scrollbar shrink-0 shadow-md">
		
		<button
			type="button"
			onclick={() => activeTab = "general"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'general' ? 'bg-brand-500 text-fg shadow-md' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}"
		>
			<Home class="w-4 h-4" />
			<span>{t("settings.tabs.general")}</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "accounts"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'accounts' ? 'bg-brand-500 text-fg shadow-md' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}"
		>
			<Users class="w-4 h-4" />
			<span>{t("settings.tabs.accounts")}</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "language"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'language' ? 'bg-brand-500 text-fg shadow-md' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}"
		>
			<Globe class="w-4 h-4" />
			<span>{t("settings.tabs.language")}</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "appearance"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap relative {activeTab === 'appearance' ? 'bg-brand-500 text-fg shadow-md' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}"
		>
			<Palette class="w-4 h-4" />
			<span>{t("settings.tabs.appearance")}</span>
			<span class="text-[9px] font-black uppercase px-1.5 py-0.5 rounded bg-emerald-400 text-brand-foreground ml-0.5">PRO</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "java"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'java' ? 'bg-brand-500 text-fg shadow-md' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}"
		>
			<Coffee class="w-4 h-4" />
			<span>{t("settings.tabs.java")}</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "commands"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'commands' ? 'bg-brand-500 text-fg shadow-md' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}"
		>
			<Terminal class="w-4 h-4" />
			<span>{t("settings.tabs.commands")}</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "privacy"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'privacy' ? 'bg-brand-500 text-fg shadow-md' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}"
		>
			<ShieldCheck class="w-4 h-4" />
			<span>{t("settings.tabs.privacy")}</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "runtime"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'runtime' ? 'bg-brand-500 text-fg shadow-md' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}"
		>
			<FolderOpen class="w-4 h-4" />
			<span>{t("settings.tabs.runtime")}</span>
		</button>

	</div>

	{#if activeTab === "general"}
		<div class="space-y-6">
			<div class="flex items-center justify-between">
				<h2 class="text-2xl font-bold text-fg tracking-tight">{t("settings.general")}</h2>
				<span class="text-xs font-mono font-semibold px-2.5 py-1 rounded-lg bg-fg/5 text-fg/60 border border-fg/10">
					Versão 1.9.1
				</span>
			</div>

			<div class="bg-gradient-to-r from-brand-500/10 via-bg-elevated to-blue-500/10 border border-brand-500/20 rounded-3xl p-6 shadow-sm flex flex-col md:flex-row md:items-center justify-between gap-4">
				<div class="space-y-1.5">
					<div class="flex items-center gap-2">
						<span class="text-xs font-bold uppercase tracking-wider text-brand-400 flex items-center gap-1.5">
							<Sparkles class="w-3.5 h-3.5" /> Atualizador Automático Integrado
						</span>
						{#if updaterStore.updateAvailable}
							<span class="text-[10px] font-bold px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 animate-pulse">
								Nova Versão Disponível
							</span>
						{:else}
							<span class="text-[10px] font-semibold px-2 py-0.5 rounded-full bg-fg/5 text-fg/60 border border-fg/10">
								Sistema Atualizado
							</span>
						{/if}
					</div>
					<h3 class="text-base font-bold text-fg">
						{#if updaterStore.updateAvailable}
							Luxmc v{updaterStore.newVersion || "1.9.1"} pronto para instalar
						{:else}
							Você está executando a versão mais recente do Luxmc (v1.9.1)
						{/if}
					</h3>
					<p class="text-xs text-fg/50">
						{#if updaterStore.lastChecked}
							Última verificação: {new Date(updaterStore.lastChecked).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
						{:else}
							Verificação rápida e automática de novas versões e correções
						{/if}
					</p>
				</div>

				<div class="flex items-center gap-3 shrink-0">
					{#if updaterStore.updateAvailable}
						<button
							type="button"
							onclick={() => updaterStore.downloadAndInstall()}
							disabled={updaterStore.isDownloading}
							class="px-4 py-2.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-bg-deep font-bold text-xs transition-all shadow-md active:scale-95 flex items-center gap-2 cursor-pointer"
						>
							{#if updaterStore.isDownloading}
								<RefreshCw class="w-4 h-4 animate-spin" /> Baixando {updaterStore.downloadProgress}%
							{:else}
								<Download class="w-4 h-4" /> Atualizar Agora
							{/if}
						</button>
					{:else}
						<button
							type="button"
							onclick={() => updaterStore.check(true)}
							disabled={updaterStore.isChecking}
							class="px-4 py-2.5 rounded-xl bg-bg border border-fg/10 hover:border-brand-500/40 text-fg text-xs font-bold transition-all shadow-sm active:scale-95 flex items-center gap-2 cursor-pointer"
						>
							<RefreshCw class="w-4 h-4 {updaterStore.isChecking ? 'animate-spin text-brand-400' : 'text-fg/60'}" />
							{updaterStore.isChecking ? "Verificando..." : "Verificar Atualizações"}
						</button>
					{/if}
				</div>
			</div>

			<div class="bg-bg-elevated border border-fg/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.releaseChannel")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.releaseChannelDesc")}</p>
					</div>
					<select
						bind:value={releaseChannel}
						class="bg-bg-elevated border border-fg/10 rounded-xl px-4 py-2 text-xs font-semibold text-fg outline-none focus:border-blue-500 cursor-pointer min-w-[140px]"
					>
						<option value="stable">Stable ↕</option>
						<option value="beta">Beta ↕</option>
						<option value="nightly">Nightly ↕</option>
					</select>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">Downloads Simultâneos</h3>
						<p class="text-xs text-fg/50 leading-relaxed">Número de conexões paralelas ao baixar mods e bibliotecas</p>
					</div>
					<select
						bind:value={concurrentDownloads}
						class="bg-bg-elevated border border-fg/10 rounded-xl px-4 py-2 text-xs font-semibold text-fg outline-none focus:border-blue-500 cursor-pointer min-w-[100px]"
					>
						<option value={2}>2 ↕</option>
						<option value={3}>3 ↕</option>
						<option value={5}>5 ↕</option>
						<option value={10}>10 ↕</option>
					</select>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.gameResolution")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.gameResolutionDesc")}</p>
					</div>
					<select
						bind:value={gameResolution}
						onchange={saveGeneral}
						class="bg-bg-elevated border border-fg/10 rounded-xl px-4 py-2 text-xs font-semibold text-fg outline-none focus:border-blue-500 cursor-pointer min-w-[140px]"
					>
						<option value="default">Padrão ↕</option>
						<option value="1920x1080">1920x1080 ↕</option>
						<option value="1280x720">1280x720 ↕</option>
						<option value="fullscreen">Tela Cheia ↕</option>
					</select>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<div class="flex items-center gap-2">
							<h3 class="text-sm font-bold text-fg">Discord Rich Presence</h3>
							<span class="text-[10px] font-bold px-1.5 py-0.5 rounded bg-blue-500/15 text-blue-400">v1.9.1</span>
						</div>
						<p class="text-xs text-fg/50 leading-relaxed">Exibe seu status, mundo, modpack e tempo de jogo no Discord com botões de conexão direta</p>
					</div>
					<button
						type="button"
						aria-label="Toggle Discord Integration"
						onclick={() => { discordIntegration = !discordIntegration; saveGeneral(); }}
						class="w-12 h-6 rounded-full transition-colors relative cursor-pointer {discordIntegration ? 'bg-blue-600' : 'bg-fg/15'}"
					>
						<span class="absolute top-1 left-1 w-4 h-4 rounded-full bg-fg transition-transform {discordIntegration ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">Ação do Launcher ao Iniciar</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.launcherActionDesc")}</p>
					</div>
					<select
						bind:value={launcherAction}
						onchange={saveGeneral}
						class="bg-bg-elevated border border-fg/10 rounded-xl px-4 py-2 text-xs font-semibold text-fg outline-none focus:border-blue-500 cursor-pointer min-w-[150px]"
					>
						<option value="keep_open">{t("settings.launcherActionNone")} ↕</option>
						<option value="hide_reopen">{t("settings.launcherActionHide")} ↕</option>
						<option value="close">{t("settings.launcherActionClose")} ↕</option>
					</select>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.closeWarningTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.closeWarningDesc")}</p>
					</div>
					<button
						type="button"
						aria-label="Toggle Window Close Warning"
						onclick={() => showCloseWarning = !showCloseWarning}
						class="w-12 h-6 rounded-full transition-colors relative cursor-pointer {showCloseWarning ? 'bg-blue-600' : 'bg-fg/15'}"
					>
						<span class="absolute top-1 left-1 w-4 h-4 rounded-full bg-fg transition-transform {showCloseWarning ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.perfModeTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.perfModeDesc")}</p>
					</div>
					<button
						type="button"
						aria-label="Toggle Ultra Performance Mode"
						onclick={() => { performanceMode = !performanceMode; appState.performanceMode = performanceMode; saveGeneral(); }}
						class="w-12 h-6 rounded-full transition-colors relative cursor-pointer {performanceMode ? 'bg-emerald-500' : 'bg-fg/15'}"
					>
						<span class="absolute top-1 left-1 w-4 h-4 rounded-full bg-fg transition-transform {performanceMode ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

			</div>
		</div>

	{:else if activeTab === "accounts"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-fg tracking-tight">{t("settings.accountsTitle")}</h2>

			<div class="bg-bg-elevated border border-fg/5 rounded-3xl p-6 shadow-sm space-y-6">
				<div class="flex items-center justify-between p-4 rounded-2xl bg-bg-elevated border border-fg/5">
					<div class="flex items-center gap-4">
						<div class="w-12 h-12 rounded-2xl bg-bg-overlay/40 border border-fg/10 overflow-hidden flex items-center justify-center">
							<img
								src={account.value?.uuid ? "https://mc-heads.net/avatar/" + account.value.uuid + "/100" : "/logo.png"}
								alt="Avatar"
								class="w-full h-full object-cover"
							/>
						</div>
						<div>
							<h3 class="text-sm font-bold text-fg">{currentUsername}</h3>
							<p class="text-xs text-fg/50">{isMicrosoft ? "Conta Microsoft Online" : "Conta Offline / Luxmc"}</p>
						</div>
					</div>

					<a
						href="/"
						class="px-4 py-2 rounded-xl bg-fg/5 hover:bg-fg/10 text-fg text-xs font-bold border border-fg/10 transition-all cursor-pointer shadow-sm active:scale-[0.98]"
					>
						{t("settings.switchAccount")}
					</a>
				</div>

				<div class="p-4 rounded-2xl bg-blue-500/10 border border-blue-500/20 text-xs text-blue-300 leading-relaxed space-y-1.5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
					<div class="space-y-1">
						<p class="font-bold">Gerenciamento no site oficial:</p>
						<p class="text-fg/60">Para cadastrar uma nova conta, trocar sua skin ou gerenciar amigos, acesse o portal web oficial do Luxmc.</p>
					</div>
					<button
						type="button"
						onclick={() => openUrl("https://luxmc-r92.pages.dev")}
						class="px-4 py-2 rounded-xl bg-blue-600 hover:bg-blue-500 text-fg text-xs font-bold transition-all cursor-pointer shrink-0 flex items-center gap-2 active:scale-[0.98] shadow-md"
					>
						<span>Abrir Portal Web</span>
						<ExternalLink class="w-3.5 h-3.5" />
					</button>
				</div>
			</div>
		</div>

	{:else if activeTab === "language"}
		<div class="space-y-6">
			<div>
				<h2 class="text-2xl font-bold text-fg tracking-tight">{t("settings.languageTitle")}</h2>
				<p class="text-xs text-fg/50 mt-1">{t("settings.languageDesc")}</p>
			</div>

			<div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
				<button
					type="button"
					onclick={() => handleLangChange("pt-BR")}
					class="p-5 rounded-2xl border flex items-center justify-between gap-4 transition-all cursor-pointer active:scale-[0.98] text-left {currentLang === 'pt-BR' ? 'border-brand-500 bg-bg-subtle ring-2 ring-brand-500/30 shadow-lg' : 'border-fg/5 bg-bg-elevated hover:border-fg/20'}"
				>
					<div class="flex items-center gap-3.5">
						<span class="text-2xl">🇧🇷</span>
						<div>
							<div class="text-sm font-bold text-fg">Português (Brasil)</div>
							<div class="text-[11px] text-fg/40">Idioma nativo da comunidade Luxmc</div>
						</div>
					</div>
					{#if currentLang === "pt-BR"}
						<div class="w-6 h-6 rounded-full bg-brand-500 text-brand-foreground flex items-center justify-center shrink-0">
							<Check class="w-4 h-4 stroke-[3]" />
						</div>
					{/if}
				</button>

				<button
					type="button"
					onclick={() => handleLangChange("en")}
					class="p-5 rounded-2xl border flex items-center justify-between gap-4 transition-all cursor-pointer active:scale-[0.98] text-left {currentLang === 'en' ? 'border-brand-500 bg-bg-subtle ring-2 ring-brand-500/30 shadow-lg' : 'border-fg/5 bg-bg-elevated hover:border-fg/20'}"
				>
					<div class="flex items-center gap-3.5">
						<span class="text-2xl">🇺🇸</span>
						<div>
							<div class="text-sm font-bold text-fg">English</div>
							<div class="text-[11px] text-fg/40">Global language</div>
						</div>
					</div>
					{#if currentLang === "en"}
						<div class="w-6 h-6 rounded-full bg-brand-500 text-brand-foreground flex items-center justify-center shrink-0">
							<Check class="w-4 h-4 stroke-[3]" />
						</div>
					{/if}
				</button>

				<button
					type="button"
					onclick={() => handleLangChange("es")}
					class="p-5 rounded-2xl border flex items-center justify-between gap-4 transition-all cursor-pointer active:scale-[0.98] text-left {currentLang === 'es' ? 'border-brand-500 bg-bg-subtle ring-2 ring-brand-500/30 shadow-lg' : 'border-fg/5 bg-bg-elevated hover:border-fg/20'}"
				>
					<div class="flex items-center gap-3.5">
						<span class="text-2xl">🇪🇸</span>
						<div>
							<div class="text-sm font-bold text-fg">Español</div>
							<div class="text-[11px] text-fg/40">Comunidad hispanohablante</div>
						</div>
					</div>
					{#if currentLang === "es"}
						<div class="w-6 h-6 rounded-full bg-brand-500 text-brand-foreground flex items-center justify-center shrink-0">
							<Check class="w-4 h-4 stroke-[3]" />
						</div>
					{/if}
				</button>
			</div>
		</div>

	{:else if activeTab === "appearance"}
		<div class="space-y-6">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2.5">
					<h2 class="text-2xl font-bold text-fg tracking-tight">{t("settings.tabs.appearance")}</h2>
				</div>
				<span class="text-xs text-fg/50">{t("settings.accentColorHint")}</span>
			</div>

			<div class="bg-bg-elevated border border-fg/5 rounded-3xl p-6 shadow-sm">
				<ThemeSection onSave={() => schedulePersist()} />
			</div>
		</div>

	{:else if activeTab === "java"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-fg tracking-tight">{t("settings.tabs.java")}</h2>

			<div class="bg-bg-elevated border border-fg/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.javaPath")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.javaPathHint")}</p>
					</div>
					<div class="flex items-center gap-2">
						<input
							type="text"
							placeholder={t("settings.javaPathAuto")}
							bind:value={javaPathInput}
							onchange={saveJava}
							class="bg-bg-elevated border border-fg/10 rounded-xl px-3 py-2 text-xs text-fg font-mono placeholder:text-fg/30 outline-none w-56"
						/>
						<button
							type="button"
							onclick={browseJavaPath}
							class="px-3 py-2 rounded-xl bg-fg/5 hover:bg-fg/10 text-fg text-xs font-bold border border-fg/10 cursor-pointer"
						>
							{t("settings.browse")}
						</button>
					</div>
				</div>

				<div class="py-5 space-y-3">
					<div class="flex items-center justify-between gap-6">
						<div class="space-y-1 max-w-xl">
							<h3 class="text-sm font-bold text-fg">{t("settings.ramAllocation")}</h3>
							<p class="text-xs text-fg/50 leading-relaxed">Quantidade de memória dedicada às instâncias do Minecraft (Atual: {maxRamGb} GB)</p>
						</div>
						<div class="flex items-center gap-3">
							<input
								type="range"
								min={2}
								max={16}
								step={1}
								bind:value={maxRamGb}
								onchange={saveJava}
								class="w-36 accent-blue-500 cursor-pointer"
							/>
							<span class="text-xs font-mono font-bold text-fg bg-bg-elevated px-3 py-1.5 rounded-xl border border-fg/10 min-w-[70px] text-center">
								{maxRamGb} GB
							</span>
						</div>
					</div>
					<div class="flex flex-wrap items-center gap-2 pt-1">
						<span class="text-[11px] font-semibold text-fg/40 mr-1">{t("settings.ramPresets")}:</span>
						{#each ramPresets as preset}
							<button
								type="button"
								onclick={() => setRamPreset(preset)}
								class="px-3 py-1 rounded-lg text-xs font-mono font-bold border transition-all cursor-pointer {maxRamGb === preset ? 'bg-blue-600 border-blue-500 text-fg shadow-md' : 'bg-fg/5 border-fg/10 text-fg/70 hover:bg-fg/10 hover:text-fg'}"
							>
								{preset}G
							</button>
						{/each}
						<button
							type="button"
							onclick={() => setRamPreset(6)}
							class="px-3 py-1 rounded-lg text-xs font-sans font-bold border transition-all cursor-pointer {maxRamGb === 6 ? 'bg-emerald-600/30 border-emerald-500/50 text-emerald-300' : 'bg-fg/5 border-fg/10 text-fg/50 hover:bg-fg/10 hover:text-fg'}"
						>
							Auto (6G)
						</button>
					</div>
				</div>

				<div class="py-5 space-y-3">
					<div class="flex items-center justify-between gap-6">
						<div class="space-y-1 max-w-xl">
							<h3 class="text-sm font-bold text-fg">{t("settings.jvmFlags")}</h3>
							<p class="text-xs text-fg/50 leading-relaxed">Flags personalizadas do Java e parâmetros do Garbage Collector (GC)</p>
						</div>
						<input
							type="text"
							bind:value={jvmArgsInput}
							onchange={saveJava}
							class="bg-bg-elevated border border-fg/10 rounded-xl px-3 py-2 text-xs text-fg font-mono placeholder:text-fg/30 outline-none w-80 focus:border-blue-500"
						/>
					</div>
					<div class="flex flex-wrap items-center gap-2 pt-1">
						<span class="text-[11px] font-semibold text-fg/40 mr-1">Presets:</span>
						<button
							type="button"
							onclick={() => setJvmPreset('g1gc')}
							class="px-2.5 py-1 rounded-lg text-[11px] font-medium border bg-fg/5 border-fg/10 text-fg/70 hover:bg-fg/10 hover:text-fg transition-all cursor-pointer"
						>
							{t("settings.flagsG1GC")}
						</button>
						<button
							type="button"
							onclick={() => setJvmPreset('aikar')}
							class="px-2.5 py-1 rounded-lg text-[11px] font-medium border bg-fg/5 border-fg/10 text-fg/70 hover:bg-fg/10 hover:text-fg transition-all cursor-pointer"
						>
							{t("settings.flagsAikar")}
						</button>
						<button
							type="button"
							onclick={() => setJvmPreset('zgc')}
							class="px-2.5 py-1 rounded-lg text-[11px] font-medium border bg-fg/5 border-fg/10 text-fg/70 hover:bg-fg/10 hover:text-fg transition-all cursor-pointer"
						>
							{t("settings.flagsZGC")}
						</button>
						<button
							type="button"
							onclick={() => setJvmPreset('shenandoah')}
							class="px-2.5 py-1 rounded-lg text-[11px] font-medium border bg-fg/5 border-fg/10 text-fg/70 hover:bg-fg/10 hover:text-fg transition-all cursor-pointer"
						>
							{t("settings.flagsShenandoah")}
						</button>
					</div>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.waylandTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.waylandDesc")}</p>
					</div>
					<button
						type="button"
						aria-label="Toggle Wayland Mode"
						onclick={() => { waylandNative = !waylandNative; saveJava(); }}
						class="w-12 h-6 rounded-full transition-colors relative cursor-pointer {waylandNative ? 'bg-blue-600' : 'bg-fg/15'}"
					>
						<span class="absolute top-1 left-1 w-4 h-4 rounded-full bg-fg transition-transform {waylandNative ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.vulkanTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.vulkanDesc")}</p>
					</div>
					<button
						type="button"
						aria-label="Toggle Vulkan Acceleration"
						onclick={() => { useVulkan = !useVulkan; saveJava(); }}
						class="w-12 h-6 rounded-full transition-colors relative cursor-pointer {useVulkan ? 'bg-blue-600' : 'bg-fg/15'}"
					>
						<span class="absolute top-1 left-1 w-4 h-4 rounded-full bg-fg transition-transform {useVulkan ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

			</div>
		</div>

	{:else if activeTab === "commands"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-fg tracking-tight">{t("settings.tabs.commands")}</h2>

			<div class="bg-bg-elevated border border-fg/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.preLaunchTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.preLaunchDesc")}</p>
					</div>
					<input
						type="text"
						placeholder="ex: notify-send 'Iniciando Minecraft'"
						bind:value={preLaunchCmd}
						class="bg-bg-elevated border border-fg/10 rounded-xl px-3 py-2 text-xs text-fg font-mono placeholder:text-fg/30 outline-none w-72"
					/>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">Comando Envoltório (Game Wrapper)</h3>
						<p class="text-xs text-fg/50 leading-relaxed">Comando envoltório inserido antes do executável Java (ex: gamemoderun, mangohud)</p>
					</div>
					<input
						type="text"
						placeholder="ex: gamemoderun"
						bind:value={gameWrapper}
						class="bg-bg-elevated border border-fg/10 rounded-xl px-3 py-2 text-xs text-fg font-mono placeholder:text-fg/30 outline-none w-72"
					/>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.postExitTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.postExitDesc")}</p>
					</div>
					<input
						type="text"
						placeholder="ex: sync"
						bind:value={postExitCmd}
						class="bg-bg-elevated border border-fg/10 rounded-xl px-3 py-2 text-xs text-fg font-mono placeholder:text-fg/30 outline-none w-72"
					/>
				</div>

			</div>
		</div>

	{:else if activeTab === "privacy"}
		<div class="space-y-6">
			<div>
				<h2 class="text-2xl font-bold text-fg tracking-tight">{t("settings.tabs.privacy")}</h2>
				<p class="text-xs text-fg/50 mt-1">Gerencie a visibilidade dos seus dados, modo de transmissão e telemetria</p>
			</div>

			<div class="bg-bg-elevated border border-fg/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<div class="flex items-center gap-2">
							<ShieldCheck class="w-4 h-4 text-brand-400" />
							<h3 class="text-sm font-bold text-fg">{t("settings.streamerModeTitle")}</h3>
						</div>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.streamerModeDesc")}</p>
					</div>
					<button
						type="button"
						role="switch"
						aria-checked={streamerMode}
						aria-label="Alternar Modo Streamer"
						onclick={() => { streamerMode = !streamerMode; savePrivacy(); }}
						class="w-12 h-6 rounded-full transition-all duration-200 relative cursor-pointer border {streamerMode ? 'bg-brand-500 border-brand-400 shadow-md shadow-brand-500/30' : 'bg-fg/15 border-fg/10 hover:bg-fg/20'}"
					>
						<span class="absolute top-0.5 left-0.5 w-4.5 h-4.5 rounded-full bg-white transition-transform duration-200 shadow-sm {streamerMode ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.discordPrivacyTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.discordPrivacyDesc")}</p>
					</div>
					<button
						type="button"
						role="switch"
						aria-checked={hideDiscordDetails}
						aria-label="Alternar Ocultar Detalhes do Discord"
						onclick={() => { hideDiscordDetails = !hideDiscordDetails; savePrivacy(); }}
						class="w-12 h-6 rounded-full transition-all duration-200 relative cursor-pointer border {hideDiscordDetails ? 'bg-brand-500 border-brand-400 shadow-md shadow-brand-500/30' : 'bg-fg/15 border-fg/10 hover:bg-fg/20'}"
					>
						<span class="absolute top-0.5 left-0.5 w-4.5 h-4.5 rounded-full bg-white transition-transform duration-200 shadow-sm {hideDiscordDetails ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.telemetryTitle")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.telemetryDesc")}</p>
					</div>
					<button
						type="button"
						role="switch"
						aria-checked={anonymousTelemetry}
						aria-label="Alternar Telemetria Anônima"
						onclick={() => { anonymousTelemetry = !anonymousTelemetry; savePrivacy(); }}
						class="w-12 h-6 rounded-full transition-all duration-200 relative cursor-pointer border {anonymousTelemetry ? 'bg-brand-500 border-brand-400 shadow-md shadow-brand-500/30' : 'bg-fg/15 border-fg/10 hover:bg-fg/20'}"
					>
						<span class="absolute top-0.5 left-0.5 w-4.5 h-4.5 rounded-full bg-white transition-transform duration-200 shadow-sm {anonymousTelemetry ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-fg">{t("settings.clearCache")}</h3>
						<p class="text-xs text-fg/50 leading-relaxed">{t("settings.clearCacheDesc")}</p>
					</div>
					<button
						type="button"
						onclick={handleClearCache}
						class="px-5 py-2.5 rounded-xl bg-red-500/10 hover:bg-red-500/20 text-red-400 text-xs font-bold border border-red-500/20 transition-all cursor-pointer shadow-sm active:scale-[0.98] flex items-center gap-2"
					>
						<Trash2 class="w-3.5 h-3.5" />
						<span>{t("settings.clearCache")}</span>
					</button>
				</div>

			</div>
		</div>

	{:else if activeTab === "runtime"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-fg tracking-tight">{t("settings.tabs.runtime")}</h2>

			<div class="bg-bg-elevated border border-fg/5 rounded-3xl px-6 py-6 shadow-sm space-y-4">
				<div class="space-y-1 max-w-xl">
					<h3 class="text-sm font-bold text-fg">{t("settings.runtimeTitle")}</h3>
					<p class="text-xs text-fg/50 leading-relaxed">Diretório onde instâncias, mods, bibliotecas, assets e arquivos de configuração são armazenados</p>
				</div>

				<div class="flex items-center gap-3">
					<input
						type="text"
						readonly
						value={runtimePath}
						class="bg-bg-elevated border border-fg/10 rounded-xl px-4 py-2.5 text-xs text-fg font-mono outline-none flex-1"
					/>
					<button
						type="button"
						onclick={() => openUrl(runtimePath)}
						class="px-4 py-2.5 rounded-xl bg-fg/5 hover:bg-fg/10 text-fg text-xs font-bold border border-fg/10 cursor-pointer shrink-0"
					>
						Abrir Pasta
					</button>
				</div>
			</div>
		</div>
	{/if}

</div>
