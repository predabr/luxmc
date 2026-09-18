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
		AlertCircle
	} from "lucide-svelte";
	import { settings, type AppSettings } from "$lib/stores/settings.svelte";
	import { appState } from "$lib/stores/app.svelte";
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

	let currentLang = $state<"pt-BR" | "en">(settings.value.language === "pt-BR" ? "pt-BR" : "en");

	let selectedTheme = $state(settings.value.theme || "default-dark");
	let selectedAccent = $state(settings.value.accentTheme || "emerald");
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

	let anonymousTelemetry = $state(true);

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

	function handleLangChange(lang: "pt-BR" | "en") {
		currentLang = lang;
		setLocale(lang);
		settings.patch({ language: lang });
		schedulePersist();
		toast("Idioma alterado com sucesso!", "info");
	}

	function handleThemeChange(tName: any) {
		selectedTheme = tName;
		themeStore.setTheme(tName);
		settings.patch({ theme: tName });
		schedulePersist();
	}

	function handleAccentChange(acc: any) {
		selectedAccent = acc;
		themeStore.setAccent(acc);
		settings.patch({ accentTheme: acc });
		schedulePersist();
	}

	function handleDensityChange(d: any) {
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
		toast("Cache de downloads e instaladores limpo com sucesso!", "success");
	}
</script>

<div class="h-full flex flex-col select-none overflow-y-auto custom-scrollbar pr-2 pb-16 max-w-5xl mx-auto w-full">
	
	<div class="flex items-center gap-1.5 p-1.5 rounded-2xl bg-[#14151a] border border-white/5 mb-8 overflow-x-auto custom-scrollbar shrink-0 shadow-md">
		
		<button
			type="button"
			onclick={() => activeTab = "general"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'general' ? 'bg-[#2563eb] text-white shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
		>
			<Home class="w-4 h-4" />
			<span>{t("settings.tabs.general")}</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "accounts"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'accounts' ? 'bg-[#2563eb] text-white shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
		>
			<Users class="w-4 h-4" />
			<span>{t("settings.tabs.accounts")}</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "language"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'language' ? 'bg-[#2563eb] text-white shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
		>
			<Globe class="w-4 h-4" />
			<span>{t("settings.tabs.language")}</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "appearance"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap relative {activeTab === 'appearance' ? 'bg-[#2563eb] text-white shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
		>
			<Palette class="w-4 h-4" />
			<span>{t("settings.tabs.appearance")}</span>
			<span class="text-[9px] font-black uppercase px-1.5 py-0.5 rounded bg-emerald-400 text-black ml-0.5">PRO</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "java"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'java' ? 'bg-[#2563eb] text-white shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
		>
			<Coffee class="w-4 h-4" />
			<span>{t("settings.tabs.java")}</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "commands"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'commands' ? 'bg-[#2563eb] text-white shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
		>
			<Terminal class="w-4 h-4" />
			<span>{t("settings.tabs.commands")}</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "privacy"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'privacy' ? 'bg-[#2563eb] text-white shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
		>
			<ShieldCheck class="w-4 h-4" />
			<span>{t("settings.tabs.privacy")}</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "runtime"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'runtime' ? 'bg-[#2563eb] text-white shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
		>
			<FolderOpen class="w-4 h-4" />
			<span>{t("settings.tabs.runtime")}</span>
		</button>

	</div>

	{#if activeTab === "general"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-white tracking-tight">{t("settings.general")}</h2>

			<div class="bg-[#111216] border border-white/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">{t("settings.releaseChannel")}</h3>
						<p class="text-xs text-white/50 leading-relaxed">{t("settings.releaseChannelDesc")}</p>
					</div>
					<select
						bind:value={releaseChannel}
						class="bg-[#181920] border border-white/10 rounded-xl px-4 py-2 text-xs font-semibold text-white outline-none focus:border-blue-500 cursor-pointer min-w-[140px]"
					>
						<option value="stable">Stable ↕</option>
						<option value="beta">Beta ↕</option>
						<option value="nightly">Nightly ↕</option>
					</select>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Downloads Simultâneos</h3>
						<p class="text-xs text-white/50 leading-relaxed">Número de conexões paralelas ao baixar mods e bibliotecas</p>
					</div>
					<select
						bind:value={concurrentDownloads}
						class="bg-[#181920] border border-white/10 rounded-xl px-4 py-2 text-xs font-semibold text-white outline-none focus:border-blue-500 cursor-pointer min-w-[100px]"
					>
						<option value={2}>2 ↕</option>
						<option value={3}>3 ↕</option>
						<option value={5}>5 ↕</option>
						<option value={10}>10 ↕</option>
					</select>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">{t("settings.gameResolution")}</h3>
						<p class="text-xs text-white/50 leading-relaxed">{t("settings.gameResolutionDesc")}</p>
					</div>
					<select
						bind:value={gameResolution}
						onchange={saveGeneral}
						class="bg-[#181920] border border-white/10 rounded-xl px-4 py-2 text-xs font-semibold text-white outline-none focus:border-blue-500 cursor-pointer min-w-[140px]"
					>
						<option value="default">Padrão ↕</option>
						<option value="1920x1080">1920x1080 ↕</option>
						<option value="1280x720">1280x720 ↕</option>
						<option value="fullscreen">Tela Cheia ↕</option>
					</select>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Discord Rich Presence</h3>
						<p class="text-xs text-white/50 leading-relaxed">{t("settings.discordRpcDesc")}</p>
					</div>
					<button
						type="button"
						aria-label="Toggle Discord Integration"
						onclick={() => { discordIntegration = !discordIntegration; saveGeneral(); }}
						class="w-12 h-6 rounded-full transition-colors relative cursor-pointer {discordIntegration ? 'bg-blue-600' : 'bg-white/15'}"
					>
						<span class="absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform {discordIntegration ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Ação do Launcher ao Iniciar</h3>
						<p class="text-xs text-white/50 leading-relaxed">{t("settings.launcherActionDesc")}</p>
					</div>
					<select
						bind:value={launcherAction}
						onchange={saveGeneral}
						class="bg-[#181920] border border-white/10 rounded-xl px-4 py-2 text-xs font-semibold text-white outline-none focus:border-blue-500 cursor-pointer min-w-[150px]"
					>
						<option value="keep_open">{t("settings.launcherActionNone")} ↕</option>
						<option value="hide_reopen">{t("settings.launcherActionHide")} ↕</option>
						<option value="close">{t("settings.launcherActionClose")} ↕</option>
					</select>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">{t("settings.closeWarningTitle")}</h3>
						<p class="text-xs text-white/50 leading-relaxed">{t("settings.closeWarningDesc")}</p>
					</div>
					<button
						type="button"
						aria-label="Toggle Window Close Warning"
						onclick={() => showCloseWarning = !showCloseWarning}
						class="w-12 h-6 rounded-full transition-colors relative cursor-pointer {showCloseWarning ? 'bg-blue-600' : 'bg-white/15'}"
					>
						<span class="absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform {showCloseWarning ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">{t("settings.perfModeTitle")}</h3>
						<p class="text-xs text-white/50 leading-relaxed">{t("settings.perfModeDesc")}</p>
					</div>
					<button
						type="button"
						aria-label="Toggle Ultra Performance Mode"
						onclick={() => { performanceMode = !performanceMode; appState.performanceMode = performanceMode; saveGeneral(); }}
						class="w-12 h-6 rounded-full transition-colors relative cursor-pointer {performanceMode ? 'bg-emerald-500' : 'bg-white/15'}"
					>
						<span class="absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform {performanceMode ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

			</div>
		</div>

	{:else if activeTab === "accounts"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-white tracking-tight">{t("settings.accountsTitle")}</h2>

			<div class="bg-[#111216] border border-white/5 rounded-3xl p-6 shadow-sm space-y-6">
				<div class="flex items-center justify-between p-4 rounded-2xl bg-[#16171d] border border-white/5">
					<div class="flex items-center gap-4">
						<div class="w-12 h-12 rounded-2xl bg-black/40 border border-white/10 overflow-hidden flex items-center justify-center">
							<img
								src={account.value?.uuid ? "https://mc-heads.net/avatar/" + account.value.uuid + "/100" : "/logo.png"}
								alt="Avatar"
								class="w-full h-full object-cover"
							/>
						</div>
						<div>
							<h3 class="text-sm font-bold text-white">{currentUsername}</h3>
							<p class="text-xs text-white/50">{isMicrosoft ? "Conta Microsoft Online" : "Conta Offline / Luxmc"}</p>
						</div>
					</div>

					<a
						href="/"
						class="px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 text-white text-xs font-bold border border-white/10 transition-all cursor-pointer shadow-sm active:scale-95"
					>
						{t("settings.switchAccount")}
					</a>
				</div>

				<div class="p-4 rounded-2xl bg-blue-500/10 border border-blue-500/20 text-xs text-blue-300 leading-relaxed space-y-1.5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
					<div class="space-y-1">
						<p class="font-bold">Gerenciamento no site oficial:</p>
						<p class="text-white/60">Para cadastrar uma nova conta, trocar sua skin ou gerenciar amigos, acesse o portal web oficial do Luxmc.</p>
					</div>
					<button
						type="button"
						onclick={() => openUrl("https://luxmc-r92.pages.dev")}
						class="px-4 py-2 rounded-xl bg-blue-600 hover:bg-blue-500 text-white text-xs font-bold transition-all cursor-pointer shrink-0 flex items-center gap-2 active:scale-95 shadow-md"
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
				<h2 class="text-2xl font-bold text-white tracking-tight">{t("settings.languageTitle")}</h2>
				<p class="text-xs text-white/50 mt-1">{t("settings.languageDesc")}</p>
			</div>

			<div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
				<button
					type="button"
					onclick={() => handleLangChange("pt-BR")}
					class="p-5 rounded-2xl border flex items-center justify-between gap-4 transition-all cursor-pointer active:scale-95 text-left {currentLang === 'pt-BR' ? 'border-emerald-500 bg-[#1a231e] ring-2 ring-emerald-500/30 shadow-lg' : 'border-white/5 bg-[#14151a] hover:border-white/20'}"
				>
					<div class="flex items-center gap-3.5">
						<span class="text-2xl">🇧🇷</span>
						<div>
							<div class="text-sm font-bold text-white">Português (Brasil)</div>
							<div class="text-[11px] text-white/40">Idioma nativo da comunidade Luxmc</div>
						</div>
					</div>
					{#if currentLang === "pt-BR"}
						<div class="w-6 h-6 rounded-full bg-emerald-500 text-black flex items-center justify-center shrink-0">
							<Check class="w-4 h-4 stroke-[3]" />
						</div>
					{/if}
				</button>

				<button
					type="button"
					onclick={() => handleLangChange("en")}
					class="p-5 rounded-2xl border flex items-center justify-between gap-4 transition-all cursor-pointer active:scale-95 text-left {currentLang === 'en' ? 'border-emerald-500 bg-[#1a231e] ring-2 ring-emerald-500/30 shadow-lg' : 'border-white/5 bg-[#14151a] hover:border-white/20'}"
				>
					<div class="flex items-center gap-3.5">
						<span class="text-2xl">🇺🇸</span>
						<div>
							<div class="text-sm font-bold text-white">English</div>
							<div class="text-[11px] text-white/40">Global language</div>
						</div>
					</div>
					{#if currentLang === "en"}
						<div class="w-6 h-6 rounded-full bg-emerald-500 text-black flex items-center justify-center shrink-0">
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
					<h2 class="text-2xl font-bold text-white tracking-tight">{t("settings.tabs.appearance")}</h2>
				</div>
				<span class="text-xs text-white/50">{t("settings.accentColorHint")}</span>
			</div>

			<div class="bg-[#111216] border border-white/5 rounded-3xl p-6 shadow-sm">
				<ThemeSection onSave={() => schedulePersist()} />
			</div>
		</div>

	{:else if activeTab === "java"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-white tracking-tight">{t("settings.tabs.java")}</h2>

			<div class="bg-[#111216] border border-white/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">{t("settings.javaPath")}</h3>
						<p class="text-xs text-white/50 leading-relaxed">{t("settings.javaPathHint")}</p>
					</div>
					<div class="flex items-center gap-2">
						<input
							type="text"
							placeholder={t("settings.javaPathAuto")}
							bind:value={javaPathInput}
							onchange={saveJava}
							class="bg-[#181920] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono placeholder:text-white/30 outline-none w-56"
						/>
						<button
							type="button"
							onclick={browseJavaPath}
							class="px-3 py-2 rounded-xl bg-white/5 hover:bg-white/10 text-white text-xs font-bold border border-white/10 cursor-pointer"
						>
							{t("settings.browse")}
						</button>
					</div>
				</div>

				<div class="py-5 space-y-3">
					<div class="flex items-center justify-between gap-6">
						<div class="space-y-1 max-w-xl">
							<h3 class="text-sm font-bold text-white">{t("settings.ramAllocation")}</h3>
							<p class="text-xs text-white/50 leading-relaxed">Quantidade de memória dedicada às instâncias do Minecraft (Atual: {maxRamGb} GB)</p>
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
							<span class="text-xs font-mono font-bold text-white bg-[#181920] px-3 py-1.5 rounded-xl border border-white/10 min-w-[70px] text-center">
								{maxRamGb} GB
							</span>
						</div>
					</div>
					<div class="flex flex-wrap items-center gap-2 pt-1">
						<span class="text-[11px] font-semibold text-white/40 mr-1">{t("settings.ramPresets")}:</span>
						{#each ramPresets as preset}
							<button
								type="button"
								onclick={() => setRamPreset(preset)}
								class="px-3 py-1 rounded-lg text-xs font-mono font-bold border transition-all cursor-pointer {maxRamGb === preset ? 'bg-blue-600 border-blue-500 text-white shadow-md' : 'bg-white/5 border-white/10 text-white/70 hover:bg-white/10 hover:text-white'}"
							>
								{preset}G
							</button>
						{/each}
						<button
							type="button"
							onclick={() => setRamPreset(6)}
							class="px-3 py-1 rounded-lg text-xs font-sans font-bold border transition-all cursor-pointer {maxRamGb === 6 ? 'bg-emerald-600/30 border-emerald-500/50 text-emerald-300' : 'bg-white/5 border-white/10 text-white/50 hover:bg-white/10 hover:text-white'}"
						>
							Auto (6G)
						</button>
					</div>
				</div>

				<div class="py-5 space-y-3">
					<div class="flex items-center justify-between gap-6">
						<div class="space-y-1 max-w-xl">
							<h3 class="text-sm font-bold text-white">{t("settings.jvmFlags")}</h3>
							<p class="text-xs text-white/50 leading-relaxed">Flags personalizadas do Java e parâmetros do Garbage Collector (GC)</p>
						</div>
						<input
							type="text"
							bind:value={jvmArgsInput}
							onchange={saveJava}
							class="bg-[#181920] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono placeholder:text-white/30 outline-none w-80 focus:border-blue-500"
						/>
					</div>
					<div class="flex flex-wrap items-center gap-2 pt-1">
						<span class="text-[11px] font-semibold text-white/40 mr-1">Presets:</span>
						<button
							type="button"
							onclick={() => setJvmPreset('g1gc')}
							class="px-2.5 py-1 rounded-lg text-[11px] font-medium border bg-white/5 border-white/10 text-white/70 hover:bg-white/10 hover:text-white transition-all cursor-pointer"
						>
							{t("settings.flagsG1GC")}
						</button>
						<button
							type="button"
							onclick={() => setJvmPreset('aikar')}
							class="px-2.5 py-1 rounded-lg text-[11px] font-medium border bg-white/5 border-white/10 text-white/70 hover:bg-white/10 hover:text-white transition-all cursor-pointer"
						>
							{t("settings.flagsAikar")}
						</button>
						<button
							type="button"
							onclick={() => setJvmPreset('zgc')}
							class="px-2.5 py-1 rounded-lg text-[11px] font-medium border bg-white/5 border-white/10 text-white/70 hover:bg-white/10 hover:text-white transition-all cursor-pointer"
						>
							{t("settings.flagsZGC")}
						</button>
						<button
							type="button"
							onclick={() => setJvmPreset('shenandoah')}
							class="px-2.5 py-1 rounded-lg text-[11px] font-medium border bg-white/5 border-white/10 text-white/70 hover:bg-white/10 hover:text-white transition-all cursor-pointer"
						>
							{t("settings.flagsShenandoah")}
						</button>
					</div>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">{t("settings.waylandTitle")}</h3>
						<p class="text-xs text-white/50 leading-relaxed">{t("settings.waylandDesc")}</p>
					</div>
					<button
						type="button"
						aria-label="Toggle Wayland Mode"
						onclick={() => { waylandNative = !waylandNative; saveJava(); }}
						class="w-12 h-6 rounded-full transition-colors relative cursor-pointer {waylandNative ? 'bg-blue-600' : 'bg-white/15'}"
					>
						<span class="absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform {waylandNative ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">{t("settings.vulkanTitle")}</h3>
						<p class="text-xs text-white/50 leading-relaxed">{t("settings.vulkanDesc")}</p>
					</div>
					<button
						type="button"
						aria-label="Toggle Vulkan Acceleration"
						onclick={() => { useVulkan = !useVulkan; saveJava(); }}
						class="w-12 h-6 rounded-full transition-colors relative cursor-pointer {useVulkan ? 'bg-blue-600' : 'bg-white/15'}"
					>
						<span class="absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform {useVulkan ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

			</div>
		</div>

	{:else if activeTab === "commands"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-white tracking-tight">{t("settings.tabs.commands")}</h2>

			<div class="bg-[#111216] border border-white/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">{t("settings.preLaunchTitle")}</h3>
						<p class="text-xs text-white/50 leading-relaxed">{t("settings.preLaunchDesc")}</p>
					</div>
					<input
						type="text"
						placeholder="ex: notify-send 'Iniciando Minecraft'"
						bind:value={preLaunchCmd}
						class="bg-[#181920] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono placeholder:text-white/30 outline-none w-72"
					/>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Comando Envoltório (Game Wrapper)</h3>
						<p class="text-xs text-white/50 leading-relaxed">Comando envoltório inserido antes do executável Java (ex: gamemoderun, mangohud)</p>
					</div>
					<input
						type="text"
						placeholder="ex: gamemoderun"
						bind:value={gameWrapper}
						class="bg-[#181920] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono placeholder:text-white/30 outline-none w-72"
					/>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">{t("settings.postExitTitle")}</h3>
						<p class="text-xs text-white/50 leading-relaxed">{t("settings.postExitDesc")}</p>
					</div>
					<input
						type="text"
						placeholder="ex: sync"
						bind:value={postExitCmd}
						class="bg-[#181920] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono placeholder:text-white/30 outline-none w-72"
					/>
				</div>

			</div>
		</div>

	{:else if activeTab === "privacy"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-white tracking-tight">{t("settings.tabs.privacy")}</h2>

			<div class="bg-[#111216] border border-white/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">{t("settings.telemetryTitle")}</h3>
						<p class="text-xs text-white/50 leading-relaxed">{t("settings.telemetryDesc")}</p>
					</div>
					<button
						type="button"
						aria-label="Toggle Crash Diagnostics"
						onclick={() => anonymousTelemetry = !anonymousTelemetry}
						class="w-12 h-6 rounded-full transition-colors relative cursor-pointer {anonymousTelemetry ? 'bg-blue-600' : 'bg-white/15'}"
					>
						<span class="absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform {anonymousTelemetry ? 'translate-x-6' : ''}"></span>
					</button>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">{t("settings.clearCache")}</h3>
						<p class="text-xs text-white/50 leading-relaxed">{t("settings.clearCacheDesc")}</p>
					</div>
					<button
						type="button"
						onclick={handleClearCache}
						class="px-4 py-2 rounded-xl bg-red-500/10 hover:bg-red-500/20 text-red-400 text-xs font-bold border border-red-500/20 transition-all cursor-pointer"
					>
						{t("settings.clearCache")}
					</button>
				</div>

			</div>
		</div>

	{:else if activeTab === "runtime"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-white tracking-tight">{t("settings.tabs.runtime")}</h2>

			<div class="bg-[#111216] border border-white/5 rounded-3xl px-6 py-6 shadow-sm space-y-4">
				<div class="space-y-1 max-w-xl">
					<h3 class="text-sm font-bold text-white">{t("settings.runtimeTitle")}</h3>
					<p class="text-xs text-white/50 leading-relaxed">Diretório onde instâncias, mods, bibliotecas, assets e arquivos de configuração são armazenados</p>
				</div>

				<div class="flex items-center gap-3">
					<input
						type="text"
						readonly
						value={runtimePath}
						class="bg-[#181920] border border-white/10 rounded-xl px-4 py-2.5 text-xs text-white font-mono outline-none flex-1"
					/>
					<button
						type="button"
						onclick={() => openUrl(runtimePath)}
						class="px-4 py-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-white text-xs font-bold border border-white/10 cursor-pointer shrink-0"
					>
						Abrir Pasta
					</button>
				</div>
			</div>
		</div>
	{/if}

</div>
