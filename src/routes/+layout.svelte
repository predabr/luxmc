<script lang="ts">
	import "../app.css";
	import { onMount } from "svelte";
	import { fade } from "svelte/transition";
	import { page } from "$app/stores";
	import Sidebar from "$lib/components/layout/Sidebar.svelte";
	import Toasts from "$lib/components/ui/Toasts.svelte";
	import StatusBanner from "$lib/components/ui/StatusBanner.svelte";
	import UpdateModal from "$lib/components/ui/UpdateModal.svelte";
	import DownloadProgressBar from "$lib/components/ui/DownloadProgressBar.svelte";
	import ErrorBoundary from "$lib/components/ui/ErrorBoundary.svelte";
	import Cutscene from "$lib/components/visuals/Cutscene.svelte";
	import { bootstrapSettings, schedulePersist, startAutoPersist } from "$lib/stores/persistence.svelte";
	import { setToastInstance } from "$lib/stores/toasts.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { appInit, discordSetActivity, listenGameExit } from "$lib/api";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { themeStore } from "$lib/stores/theme.svelte";
	const { t } = useTranslation();
	let { children } = $props();
	let initialized = $state(false);
	let showSplash = $state(true);
	let toastsInstance = $state<Toasts | null>(null);
	$effect(() => {
		if (toastsInstance) setToastInstance(toastsInstance);
	});
	onMount(() => {
		themeStore.init();
		void bootstrapSettings();
		const initTimer = setTimeout(() => {
			initialized = true;
		}, 2500);
		appInit().then((init) => {
			clearTimeout(initTimer);
			initialized = true;
			appState.devMode = init.devMode;
			if (init.account) {
				account.value = {
					id: init.account.id,
					username: init.account.username,
					uuid: init.account.uuid,
					minecraftToken: init.account.accessToken ?? "",
					expiresAt: init.account.expiresAt ? new Date(init.account.expiresAt).getTime() : 0,
					skinUrl: init.account.skinUrl ?? null,
					skinVariant: init.account.skinVariant ?? null,
					capeUrl: init.account.capeUrl ?? null,
				};
				if (typeof window !== "undefined") {
					try {
						localStorage.setItem("luxmc_current_account", JSON.stringify(account.value));
					} catch {}
				}
				const skinUrl = init.account.skinUrl || `https://minotar.net/skin/${init.account.username}`;
				activeSkinStore.setSkin({
					id: init.account.uuid,
					name: init.account.username,
					url: `https://mc-heads.net/body/${init.account.username}/300`,
					skinUrl,
					avatarUrl: `https://mc-heads.net/avatar/${init.account.username}/100`,
					type: init.account.skinVariant?.toLowerCase() === "slim" ? "alex" : "steve",
					hasCape: Boolean(init.account.capeUrl),
					capeType: init.account.capeUrl ? "custom" : "none",
					customCapeUrl: init.account.capeUrl || ""
				});
			} else if (typeof window !== "undefined") {
				const saved = localStorage.getItem("luxmc_current_account");
				if (saved) {
					try {
						account.value = JSON.parse(saved);
					} catch {}
				}
			}
			profiles.list = init.profiles.map((p) => ({
				id: p.id,
				name: p.name,
				icon: p.icon,
				mcVersion: p.mcVersion,
				loader: p.loader as "vanilla" | "fabric" | "forge" | "neoforge" | "quilt",
				loaderVersion: p.loaderVersion ?? undefined,
				javaPath: p.javaPath ?? undefined,
				jvmArgs: p.jvmArgs ?? undefined,
				resolution: p.resolutionW && p.resolutionH ? { width: p.resolutionW, height: p.resolutionH, fullscreen: p.fullscreen } : undefined,
				gameDir: p.gameDir,
				createdAt: new Date(p.createdAt).getTime(),
				updatedAt: new Date(p.updatedAt).getTime(),
				favorite: p.favorite,
				notes: p.notes ?? undefined,
				lastPlayed: p.lastPlayed ? new Date(p.lastPlayed).getTime() : undefined,
				modCount: p.modCount,
				diskUsage: p.diskUsage,
				ramMb: p.ramMb ?? undefined,
				group: p.instanceGroup ?? undefined,
			}));
			if (init.activeProfileId) {
				profiles.activeId = init.activeProfileId;
			}
			initialized = true;
			if (settings.value.discordRpc !== false) {
				discordSetActivity({
					details: "No Menu Principal",
					state: "v1.3.1-alpha · Linux",
					largeText: "Luxmc Launcher (Linux)",
					largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
					smallImage: "grass",
					smallText: "Minecraft Linux",
					inGame: false
				}).catch(() => {});
			}
			if (typeof window !== "undefined" && (init.stressTest || window.location.search.includes("test_leak=1"))) {
				showSplash = false;
				appState.showCutscene = false;
				import("$app/navigation").then(({ goto }) => goto("/mods?test_leak=1"));
			}
		}).catch((e) => {
			clearTimeout(initTimer);
			toast(t("app.initFailed", { error: String(e) }), "error");
			initialized = true;
		});

		// Global game exit listener to accurately update play time and discord presence
		listenGameExit((event) => {
			gamingStats.onGameExit();
			appState.isGameRunning = false;
			appState.activeGameDetails = null;
			if (settings.value.discordRpc !== false) {
				discordSetActivity({
					details: "No Menu Principal",
					state: "v1.3.1-alpha · Linux",
					largeText: "Luxmc Launcher (Linux)",
					largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
					smallImage: "grass",
					smallText: "Minecraft Linux",
					inGame: false
				}).catch(() => {});
			}
		}).catch(() => {});

		const stop = startAutoPersist();
		return stop;
	});

	let rpcTimeout: ReturnType<typeof setTimeout> | null = null;
	$effect(() => {
		const currentPath = $page.url.pathname;
		if (settings.value.discordRpc === false) return;
		if (appState.isGameRunning) return;

		if (rpcTimeout) clearTimeout(rpcTimeout);
		rpcTimeout = setTimeout(() => {
			let details = "No Menu Principal";
			let state = "v1.3.1-alpha · Linux";

			if (currentPath === "/") {
				details = "No Menu Principal";
				state = "Pronto para Jogar";
			} else if (currentPath === "/instances") {
				details = "Gerenciando Instâncias";
				state = "v1.3.1-alpha · Linux";
			} else if (currentPath.startsWith("/instances/")) {
				details = "Configurando Instância";
				state = "Ajustando Modos & Versões";
			} else if (currentPath === "/mods") {
				details = "Explorando Mods & Modpacks";
				state = "Modrinth & CurseForge";
			} else if (currentPath === "/skins") {
				details = "Personalizador de Skins 3D";
				state = "Customizando Aparência";
			} else if (currentPath === "/servers") {
				details = "Lista de Servidores";
				state = "Procurando Mundos Multiplayer";
			} else if (currentPath === "/screenshots") {
				details = "Galeria de Capturas de Tela";
				state = "Visualizando Screenshots";
			} else if (currentPath === "/logs" || currentPath === "/logs-history") {
				details = "Analisando Logs";
				state = "Diagnóstico do Jogo";
			} else if (currentPath === "/settings") {
				details = "Configurações do Launcher";
				state = "Ajustando Preferências";
			} else if (currentPath === "/friends") {
				details = "Amigos & Chat P2P";
				state = "Rede Social Gamer";
			}

			discordSetActivity({
				details,
				state,
				largeText: "Luxmc Launcher (Linux)",
				largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
				smallText: "Minecraft Linux",
				smallImage: "grass",
				inGame: false
			}).catch(() => {});
		}, 300);
	});

	$effect(() => {
		JSON.stringify(settings.value);
		schedulePersist();
	});
	// Easter Egg (Konami Code)
	$effect(() => {
		const konami = ['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'b', 'a'];
		let pos = 0;
		const handler = (e: KeyboardEvent) => {
			if (e.key === konami[pos]) {
				pos++;
				if (pos === konami.length) {
					toast("🎮 Cheat Code Activated: God Mode", "success");
					document.documentElement.style.filter = 'hue-rotate(90deg)';
					pos = 0;
				}
			} else {
				pos = 0;
			}
		};
		window.addEventListener('keydown', handler);
		return () => window.removeEventListener('keydown', handler);
	});
</script>

<div class="fixed inset-0 z-[-2] bg-[#0c0c0e]">
	{#if settings.value.theme === "default-dark" && !appState.performanceMode}
		<!-- Clean high-performance GPU radial glow without expensive filter blur -->
		<div class="absolute inset-0 opacity-20 pointer-events-none" style="background: radial-gradient(circle at 20% -10%, rgb(var(--brand-500)) 0%, transparent 55%);"></div>
		<div class="absolute inset-0 opacity-15 pointer-events-none" style="background: radial-gradient(circle at 85% 110%, rgb(var(--brand-500)) 0%, transparent 55%);"></div>
	{:else}
		<div class="absolute inset-0 bg-black/60"></div>
	{/if}
</div>
<Toasts bind:this={toastsInstance} />
<StatusBanner />

<ErrorBoundary>
{#if showSplash || appState.showCutscene}
	<Cutscene onComplete={() => { showSplash = false; appState.showCutscene = false; initialized = true; }} />
{:else if !initialized}
	<div class="flex h-full w-full items-center justify-center bg-black/50 backdrop-blur-xl" in:fade={{ duration: 300 }}>
		<div class="flex flex-col items-center gap-4">
			<div class="h-10 w-10 border-4 border-t-brand-400 border-white/10 rounded-full animate-spin"></div>
			<p class="text-sm font-medium text-white shadow-black drop-shadow-md">{t("app.loading")}</p>
		</div>
	</div>
{:else if !account.value}
	<div class="flex h-full w-full items-center justify-center bg-black/30 backdrop-blur-md" in:fade={{ duration: 800 }}>
		{@render children?.()}
	</div>
{:else}
	<div class="flex h-full w-full overflow-hidden" in:fade={{ duration: 100 }}>
		<Sidebar notificationCount={0} />
		<div class="flex h-full min-w-0 flex-1 flex-col relative z-10">
			<!-- Removed Topbar to make it seamless like a native app -->
			<main class="flex-1 overflow-y-auto px-6 py-6 scroll-smooth custom-scrollbar">
				{#key $page.url.pathname}
					<div class="mx-auto max-w-[1600px] h-full flex flex-col" in:fade={{ duration: 160 }}>
						{@render children?.()}
					</div>
				{/key}
			</main>
		</div>
	</div>
{/if}
<UpdateModal />
<DownloadProgressBar />
</ErrorBoundary>
