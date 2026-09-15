<script lang="ts">
	import "../app.css";
	import { onMount } from "svelte";
	import { fade, fly } from "svelte/transition";
	import { cubicOut } from "svelte/easing";
	import { page } from "$app/stores";
	import { beforeNavigate } from "$app/navigation";
	import Sidebar from "$lib/components/layout/Sidebar.svelte";
	import Toasts from "$lib/components/ui/Toasts.svelte";
	import StatusBanner from "$lib/components/ui/StatusBanner.svelte";
	import UpdateModal from "$lib/components/ui/UpdateModal.svelte";
	import DownloadProgressBar from "$lib/components/ui/DownloadProgressBar.svelte";
	import ErrorBoundary from "$lib/components/ui/ErrorBoundary.svelte";
	import Cutscene from "$lib/components/visuals/Cutscene.svelte";
	import CrashDoctorModal from "$lib/components/ui/CrashDoctorModal.svelte";
	import CommandPalette from "$lib/components/ui/CommandPalette.svelte";
	import MiniPlayer from "$lib/components/ui/MiniPlayer.svelte";
	import LiveWallpaper from "$lib/components/visuals/LiveWallpaper.svelte";
	import TelemetryModal from "$lib/components/ui/TelemetryModal.svelte";
	import ClientOverlayModal from "$lib/components/ui/ClientOverlayModal.svelte";
	import type { GameTelemetrySummary } from "$lib/api/types";
	import { listenGameTelemetry } from "$lib/api/events";
	import { startSoundscape, stopSoundscape } from "$lib/utils/sound";
	import { bootstrapSettings, schedulePersist, startAutoPersist } from "$lib/stores/persistence.svelte";
	import { setToastInstance } from "$lib/stores/toasts.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { crashDoctor } from "$lib/stores/crashDoctor.svelte";
	import { achievements } from "$lib/stores/achievements.svelte";
	import { clientMods } from "$lib/stores/clientMods.svelte";
	import { applyAdaptivePalette } from "$lib/utils/adaptivePalette";
	import { appInit, discordSetActivity, listenGameExit, crashDoctorDiagnose } from "$lib/api";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { themeStore } from "$lib/stores/theme.svelte";
	const { t } = useTranslation();
	let { children } = $props();
	let initialized = $state(false);
	let showSplash = $state(true);
	let toastsInstance = $state<Toasts | null>(null);
	let telemetryData = $state<GameTelemetrySummary | null>(null);
	let showTelemetryModal = $state(false);
	$effect(() => {
		if (toastsInstance) setToastInstance(toastsInstance);
	});

	const tabOrderMap: Record<string, number> = {
		"/": 0,
		"/news": 0.5,
		"/mods": 1,
		"/organizer": 2,
		"/skins": 3,
		"/instances": 4,
		"/friends": 5,
		"/teamwork-preview": 5.5,
		"/screenshots": 6,
		"/logs": 7,
		"/logs-history": 7.1,
		"/settings": 8
	};

	function getRouteOrder(pathname: string): number {
		if (tabOrderMap[pathname] !== undefined) return tabOrderMap[pathname];
		if (pathname.startsWith("/instances/")) return 4.1;
		for (const [key, idx] of Object.entries(tabOrderMap)) {
			if (key !== "/" && pathname.startsWith(key)) return idx;
		}
		return 0;
	}

	let slideDirection = $state(1);

	beforeNavigate((nav) => {
		if (nav.from && nav.to && nav.from.url.pathname !== nav.to.url.pathname) {
			const prevIdx = getRouteOrder(nav.from.url.pathname);
			const curIdx = getRouteOrder(nav.to.url.pathname);
			slideDirection = curIdx >= prevIdx ? 1 : -1;
		}
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
					state: "v1.5.4-beta · Linux",
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

		// Global game exit listener to accurately update play time, crash diagnosis, and discord presence
		listenGameExit((event) => {
			gamingStats.onGameExit();
			const lastProfileId = appState.activeGameDetails?.profileId || profiles.activeId || "";
			appState.isGameRunning = false;
			appState.activeGameDetails = null;
			if (!event.success || event.code !== 0) {
				const detail = event.errorMessage ? `\nMotivo: ${event.errorMessage}` : " Consulte a aba de Logs para detalhes.";
				toast(`O Minecraft encerrou com código de saída ${event.code}.${detail}`, "error");

				if (lastProfileId) {
					crashDoctorDiagnose(lastProfileId, event.errorMessage).then((diagnosis) => {
						if (diagnosis && diagnosis.hasError) {
							crashDoctor.open(diagnosis, lastProfileId);
						}
					}).catch(() => {});
				}
			} else {
				achievements.unlock("primeira_noite");
			}
			if (settings.value.discordRpc !== false) {
				discordSetActivity({
					details: "No Menu Principal",
					state: "v1.5.4-beta · Linux",
					largeText: "Luxmc Launcher (Linux)",
					largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
					smallImage: "grass",
					smallText: "Minecraft Linux",
					inGame: false
				}).catch(() => {});
			}
			if (settings.value.soundscapesEnabled === true && !appState.performanceMode) {
				startSoundscape("overworld");
			}
		}).catch(() => {});

		let unlistenTelemetry: (() => void) | undefined;
		listenGameTelemetry((summary) => {
			telemetryData = summary;
			showTelemetryModal = true;
		}).then((unlisten) => {
			unlistenTelemetry = unlisten;
		}).catch(() => {});

		if (settings.value.soundscapesEnabled === true && !appState.performanceMode) {
			setTimeout(() => {
				startSoundscape("overworld");
			}, 1200);
		}

		const stop = startAutoPersist();
		return () => {
			stop();
			if (unlistenTelemetry) unlistenTelemetry();
			stopSoundscape();
		};
	});

	$effect(() => {
		const bannerOrIcon = profiles.active?.banner || profiles.active?.icon;
		if (bannerOrIcon) {
			applyAdaptivePalette(bannerOrIcon);
		}
	});

	$effect(() => {
		if (appState.isGameRunning) {
			stopSoundscape();
		}
	});

	let rpcTimeout: ReturnType<typeof setTimeout> | null = null;
	$effect(() => {
		const currentPath = $page.url.pathname;
		if (settings.value.discordRpc === false) return;
		if (appState.isGameRunning) return;

		if (rpcTimeout) clearTimeout(rpcTimeout);
		rpcTimeout = setTimeout(() => {
			let details = "No Menu Principal";
			let state = "v1.5.4-beta · Linux";

			if (currentPath === "/") {
				details = "No Menu Principal";
				state = "Pronto para Jogar";
			} else if (currentPath === "/instances") {
				details = "Gerenciando Instâncias";
				state = "v1.5.4-beta · Linux";
			} else if (currentPath.startsWith("/instances/")) {
				details = "Configurando Instância";
				state = "Ajustando Modos & Versões";
			} else if (currentPath === "/mods") {
				details = "Explorando Mods & Modpacks";
				state = "Modrinth & CurseForge";
			} else if (currentPath === "/skins") {
				details = "Personalizador de Skins 3D";
				state = "Customizando Aparência";
			} else if (currentPath === "/organizer") {
				details = "Organizando o Layout";
				state = "Personalizando Painéis";
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
	$effect(() => {
		const handleClientKeys = (e: KeyboardEvent) => {
			if (e.code === "ShiftRight" || e.key === "Insert") {
				clientMods.toggleMenu();
			}
		};
		window.addEventListener("keydown", handleClientKeys);
		return () => window.removeEventListener("keydown", handleClientKeys);
	});
</script>

<div class="fixed inset-0 z-[-2] bg-[#0c0c0e]">
	{#if settings.value.theme === "default-dark" && !appState.performanceMode}
		<!-- Clean high-performance GPU radial glow without expensive filter blur -->
		<div class="absolute inset-0 opacity-20 pointer-events-none" style="background: radial-gradient(circle at 20% -10%, rgb(var(--brand-500)) 0%, transparent 55%);"></div>
		<div class="absolute inset-0 opacity-15 pointer-events-none" style="background: radial-gradient(circle at 85% 110%, rgb(var(--brand-500)) 0%, transparent 55%);"></div>
		{#if settings.value.liveWallpaper !== false}
			<LiveWallpaper />
		{/if}
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
			<main class="flex-1 overflow-x-hidden overflow-y-auto px-6 py-6 scroll-smooth custom-scrollbar relative">
				{#key $page.url.pathname}
					<div
						class="mx-auto max-w-[1600px] min-h-full flex flex-col w-full will-change-transform"
						in:fly={{
							x: appState.performanceMode ? 0 : slideDirection * 65,
							duration: appState.performanceMode ? 0 : 220,
							opacity: 0,
							easing: cubicOut
						}}
					>
						{@render children?.()}
					</div>
				{/key}
			</main>
		</div>
	</div>
{/if}
<UpdateModal />
<DownloadProgressBar />
<CrashDoctorModal />
<CommandPalette />
<MiniPlayer />
<ClientOverlayModal />
<TelemetryModal
	summary={showTelemetryModal ? telemetryData : null}
	onClose={() => showTelemetryModal = false}
/>
</ErrorBoundary>
