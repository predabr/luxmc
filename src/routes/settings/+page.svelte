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
			javaPath: javaPathInput || undefined,
			jvmArgs: jvmArgsInput || undefined
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
			<span>General</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "accounts"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'accounts' ? 'bg-[#2563eb] text-white shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
		>
			<Users class="w-4 h-4" />
			<span>Accounts</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "language"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'language' ? 'bg-[#2563eb] text-white shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
		>
			<Globe class="w-4 h-4" />
			<span>Language</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "appearance"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap relative {activeTab === 'appearance' ? 'bg-[#2563eb] text-white shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
		>
			<Palette class="w-4 h-4" />
			<span>Appearance</span>
			<span class="text-[9px] font-black uppercase px-1.5 py-0.5 rounded bg-amber-400 text-black ml-0.5">BETA</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "java"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'java' ? 'bg-[#2563eb] text-white shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
		>
			<Coffee class="w-4 h-4" />
			<span>Java</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "commands"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'commands' ? 'bg-[#2563eb] text-white shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
		>
			<Terminal class="w-4 h-4" />
			<span>Custom Commands</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "privacy"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'privacy' ? 'bg-[#2563eb] text-white shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
		>
			<ShieldCheck class="w-4 h-4" />
			<span>Privacy</span>
		</button>

		<button
			type="button"
			onclick={() => activeTab = "runtime"}
			class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all cursor-pointer whitespace-nowrap {activeTab === 'runtime' ? 'bg-[#2563eb] text-white shadow-md' : 'text-white/60 hover:text-white hover:bg-white/5'}"
		>
			<FolderOpen class="w-4 h-4" />
			<span>Runtime Path</span>
		</button>

	</div>

	{#if activeTab === "general"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-white tracking-tight">General</h2>

			<div class="bg-[#111216] border border-white/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Release Channel</h3>
						<p class="text-xs text-white/50 leading-relaxed">Select the preferred release channel</p>
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
						<h3 class="text-sm font-bold text-white">Concurrent Downloads</h3>
						<p class="text-xs text-white/50 leading-relaxed">Select the number of concurrent downloads. If you have a slow connection, select at most 3</p>
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
						<h3 class="text-sm font-bold text-white">Game Resolution</h3>
						<p class="text-xs text-white/50 leading-relaxed">Select the game resolution. This will be used to launch the game. Default means the launcher will not specify a value and the game will launch with the default resolution .</p>
					</div>
					<select
						bind:value={gameResolution}
						onchange={saveGeneral}
						class="bg-[#181920] border border-white/10 rounded-xl px-4 py-2 text-xs font-semibold text-white outline-none focus:border-blue-500 cursor-pointer min-w-[140px]"
					>
						<option value="default">Default ↕</option>
						<option value="1920x1080">1920x1080 ↕</option>
						<option value="1280x720">1280x720 ↕</option>
						<option value="fullscreen">Fullscreen ↕</option>
					</select>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Discord Integration</h3>
						<p class="text-xs text-white/50 leading-relaxed">Enable or disable Discord integration. This displays what you are playing in Discord.</p>
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
						<h3 class="text-sm font-bold text-white">Launcher Action on Game Launch</h3>
						<p class="text-xs text-white/50 leading-relaxed">Action to take when launching an instance. Beware that quitting the launcher will prevent it from keeping track of played time.</p>
					</div>
					<select
						bind:value={launcherAction}
						onchange={saveGeneral}
						class="bg-[#181920] border border-white/10 rounded-xl px-4 py-2 text-xs font-semibold text-white outline-none focus:border-blue-500 cursor-pointer min-w-[150px]"
					>
						<option value="keep_open">None (Keep open) ↕</option>
						<option value="hide_reopen">Hide and reopen ↕</option>
						<option value="close">Close launcher ↕</option>
					</select>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Show Window Close Warning on Game Launch</h3>
						<p class="text-xs text-white/50 leading-relaxed">Show warning prompt before closing the launcher while instances are running</p>
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
						<h3 class="text-sm font-bold text-white">Ultra Performance Mode</h3>
						<p class="text-xs text-white/50 leading-relaxed">Disables background canvas effects and heavy transitions for maximum responsiveness on Linux</p>
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
			<h2 class="text-2xl font-bold text-white tracking-tight">Accounts</h2>

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
							<p class="text-xs text-white/50">{isMicrosoft ? "Conta Microsoft Oficial" : "Conta Offline / Luxmc"}</p>
						</div>
					</div>

					<a
						href="/"
						class="px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 text-white text-xs font-bold border border-white/10 transition-all cursor-pointer"
					>
						Trocar Conta
					</a>
				</div>

				<div class="p-4 rounded-2xl bg-blue-500/10 border border-blue-500/20 text-xs text-blue-300 leading-relaxed space-y-1">
					<p class="font-bold">Gerenciamento no site oficial:</p>
					<p class="text-white/60">Para cadastrar uma nova conta ou atualizar sua skin offline, acesse o portal da web em <a href="https://luxmc.app" target="_blank" class="text-blue-400 underline">luxmc.app</a>.</p>
				</div>
			</div>
		</div>

	{:else if activeTab === "language"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-white tracking-tight">Language</h2>

			<div class="bg-[#111216] border border-white/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Launcher Language</h3>
						<p class="text-xs text-white/50 leading-relaxed">Select your preferred language for the launcher user interface</p>
					</div>
					<select
						bind:value={currentLang}
						onchange={() => handleLangChange(currentLang)}
						class="bg-[#181920] border border-white/10 rounded-xl px-4 py-2 text-xs font-semibold text-white outline-none focus:border-blue-500 cursor-pointer min-w-[180px]"
					>
						<option value="pt-BR">Português (Brasil) ↕</option>
						<option value="en">English ↕</option>
					</select>
				</div>
			</div>
		</div>

	{:else if activeTab === "appearance"}
		<div class="space-y-6">
			<div class="flex items-center gap-2">
				<h2 class="text-2xl font-bold text-white tracking-tight">Appearance</h2>
				<span class="text-[10px] font-black uppercase px-2 py-0.5 rounded bg-amber-400 text-black">BETA</span>
			</div>

			<div class="bg-[#111216] border border-white/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Launcher Theme</h3>
						<p class="text-xs text-white/50 leading-relaxed">Select base color theme</p>
					</div>
					<select
						bind:value={selectedTheme}
						onchange={() => handleThemeChange(selectedTheme)}
						class="bg-[#181920] border border-white/10 rounded-xl px-4 py-2 text-xs font-semibold text-white outline-none focus:border-blue-500 cursor-pointer min-w-[160px]"
					>
						<option value="default-dark">Default Dark ↕</option>
						<option value="default-light">Default Light ↕</option>
					</select>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Accent Color</h3>
						<p class="text-xs text-white/50 leading-relaxed">Primary highlight and focus accent color</p>
					</div>
					<select
						bind:value={selectedAccent}
						onchange={() => handleAccentChange(selectedAccent)}
						class="bg-[#181920] border border-white/10 rounded-xl px-4 py-2 text-xs font-semibold text-white outline-none focus:border-blue-500 cursor-pointer min-w-[160px]"
					>
						<option value="emerald">Emerald Green ↕</option>
						<option value="blue">Sapphire Blue ↕</option>
						<option value="gold">Champagne Gold ↕</option>
						<option value="violet">Amethyst Violet ↕</option>
						<option value="rose">Rose Quartz ↕</option>
						<option value="cyan">Cyan Aqua ↕</option>
						<option value="orange">Sunset Orange ↕</option>
					</select>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Interface Density</h3>
						<p class="text-xs text-white/50 leading-relaxed">Density of UI spacing and typography</p>
					</div>
					<select
						bind:value={density}
						onchange={() => handleDensityChange(density)}
						class="bg-[#181920] border border-white/10 rounded-xl px-4 py-2 text-xs font-semibold text-white outline-none focus:border-blue-500 cursor-pointer min-w-[160px]"
					>
						<option value="compact">Compact ↕</option>
						<option value="comfortable">Comfortable ↕</option>
						<option value="spacious">Spacious ↕</option>
					</select>
				</div>

			</div>
		</div>

	{:else if activeTab === "java"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-white tracking-tight">Java</h2>

			<div class="bg-[#111216] border border-white/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Java Runtime Path</h3>
						<p class="text-xs text-white/50 leading-relaxed">Path to the Java executable. Leave blank to let Luxmc automatically detect and manage runtimes.</p>
					</div>
					<div class="flex items-center gap-2">
						<input
							type="text"
							placeholder="Auto-detect (Recommended)"
							bind:value={javaPathInput}
							onchange={saveJava}
							class="bg-[#181920] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono placeholder:text-white/30 outline-none w-56"
						/>
						<button
							type="button"
							onclick={browseJavaPath}
							class="px-3 py-2 rounded-xl bg-white/5 hover:bg-white/10 text-white text-xs font-bold border border-white/10 cursor-pointer"
						>
							Browse
						</button>
					</div>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Maximum Memory Allocation</h3>
						<p class="text-xs text-white/50 leading-relaxed">RAM allocated to Minecraft instances (Current: {maxRamGb} GB)</p>
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

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">JVM Arguments</h3>
						<p class="text-xs text-white/50 leading-relaxed">Custom Java Virtual Machine flags and GC optimization parameters</p>
					</div>
					<input
						type="text"
						bind:value={jvmArgsInput}
						onchange={saveJava}
						class="bg-[#181920] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono placeholder:text-white/30 outline-none w-72 focus:border-blue-500"
					/>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Vulkan Hardware Acceleration (Zink)</h3>
						<p class="text-xs text-white/50 leading-relaxed">Use Vulkan translation layers for higher FPS and lower CPU overhead on Linux drivers</p>
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
			<h2 class="text-2xl font-bold text-white tracking-tight">Custom Commands</h2>

			<div class="bg-[#111216] border border-white/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Pre-Launch Command</h3>
						<p class="text-xs text-white/50 leading-relaxed">Command to run before Minecraft launches</p>
					</div>
					<input
						type="text"
						placeholder="e.g. notify-send 'Launching Minecraft'"
						bind:value={preLaunchCmd}
						class="bg-[#181920] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono placeholder:text-white/30 outline-none w-72"
					/>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Game Wrapper Command</h3>
						<p class="text-xs text-white/50 leading-relaxed">Wrapper command placed before the Java executable (e.g. gamemoderun, mangohud)</p>
					</div>
					<input
						type="text"
						placeholder="e.g. gamemoderun"
						bind:value={gameWrapper}
						class="bg-[#181920] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono placeholder:text-white/30 outline-none w-72"
					/>
				</div>

				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Post-Exit Command</h3>
						<p class="text-xs text-white/50 leading-relaxed">Command to run after Minecraft terminates</p>
					</div>
					<input
						type="text"
						placeholder="e.g. sync"
						bind:value={postExitCmd}
						class="bg-[#181920] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono placeholder:text-white/30 outline-none w-72"
					/>
				</div>

			</div>
		</div>

	{:else if activeTab === "privacy"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-white tracking-tight">Privacy</h2>

			<div class="bg-[#111216] border border-white/5 rounded-3xl px-6 py-2 shadow-sm divide-y divide-white/5">
				
				<div class="flex items-center justify-between py-5 gap-6">
					<div class="space-y-1 max-w-xl">
						<h3 class="text-sm font-bold text-white">Crash Diagnostics</h3>
						<p class="text-xs text-white/50 leading-relaxed">Enable Crash Doctor to diagnose crash logs locally without uploading private data</p>
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
						<h3 class="text-sm font-bold text-white">Clear Cache & Temporary Files</h3>
						<p class="text-xs text-white/50 leading-relaxed">Delete cached assets, temp files, and downloaded installers to reclaim disk space</p>
					</div>
					<button
						type="button"
						onclick={handleClearCache}
						class="px-4 py-2 rounded-xl bg-red-500/10 hover:bg-red-500/20 text-red-400 text-xs font-bold border border-red-500/20 transition-all cursor-pointer"
					>
						Limpar Cache
					</button>
				</div>

			</div>
		</div>

	{:else if activeTab === "runtime"}
		<div class="space-y-6">
			<h2 class="text-2xl font-bold text-white tracking-tight">Runtime Path</h2>

			<div class="bg-[#111216] border border-white/5 rounded-3xl px-6 py-6 shadow-sm space-y-4">
				<div class="space-y-1 max-w-xl">
					<h3 class="text-sm font-bold text-white">Launcher Storage Directory</h3>
					<p class="text-xs text-white/50 leading-relaxed">Where game instances, libraries, assets, and config files are stored</p>
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
