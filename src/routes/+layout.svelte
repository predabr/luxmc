<script lang="ts">
	import "../app.css";
	import { listenDeepLinks } from "$lib/api/deepLinks";
	import { handleDeepLink } from "$lib/utils/handleDeepLink";
	import { onMount, untrack } from "svelte";
	import { fade, fly } from "svelte/transition";
	import { cubicOut } from "svelte/easing";
	import { page } from "$app/state";
	import { beforeNavigate, afterNavigate } from "$app/navigation";
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
	import ProfileModal from "$lib/components/profile/ProfileModal.svelte";
	import type { GameTelemetrySummary } from "$lib/api/types";
	import { listenGameTelemetry } from "$lib/api/events";
	import { startSoundscape, stopSoundscape, destroyAudio } from "$lib/utils/sound";
	import { bootstrapSettings, schedulePersist, startAutoPersist } from "$lib/stores/persistence.svelte";
	import { setToastInstance } from "$lib/stores/toasts.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { cloudAccount } from "$lib/stores/cloudAccount.svelte";
    import { friendsState } from "$lib/stores/friends.svelte";
	$effect(() => {
        const id = account.value?.id;
        untrack(() => {
            friendsState.disconnect(); cloudAccount.disconnect();
            if (id?.startsWith("luxmc:")) { cloudAccount.connect(id); void friendsState.connect(); }
        });
        return () => { cloudAccount.disconnect(); friendsState.disconnect(); };
    });
    $effect(() => {
        const ready = cloudAccount.ready;
        const { theme, accentTheme, language, animations } = settings.value;
        if (ready) untrack(() => cloudAccount.schedule({ theme, accentTheme, language, animations }));
    });
	import { profiles } from "$lib/stores/profiles.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { activeSkinStore, type CapeType } from "$lib/stores/skin.svelte";
	import { getFullCapeDataUrl } from "$lib/utils/capeTextures";
	import { crashDoctor } from "$lib/stores/crashDoctor.svelte";
	import { achievements } from "$lib/stores/achievements.svelte";
	import { clientMods } from "$lib/stores/clientMods.svelte";
	import { applyAdaptivePalette } from "$lib/utils/adaptivePalette";
	import { appInit, discordSetActivity, listenGameExit, crashDoctorDiagnose, clientOverlayClose, authSetAccountCape } from "$lib/api";
	import { optimizerTrimMemory } from "$lib/api/instances";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { themeStore } from "$lib/stores/theme.svelte";
	const { t } = useTranslation();
	let { children }: { children: import("svelte").Snippet } = $props();
	let initialized = $state(false);
	let showSplash = $state(true);
	let toastsInstance = $state<Toasts | null>(null);
	let telemetryData = $state<GameTelemetrySummary | null>(null);
	let showTelemetryModal = $state(false);

	const tabOrderMap: Record<string, number> = {
		"/": 0,
		"/news": 0.5,
		"/mods": 1,
		"/skins": 2,
		"/instances": 3,
		"/teamwork-preview": 3.5,
		"/screenshots": 4,
		"/logs": 5,
		"/logs-history": 5.1,
		"/settings": 6
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
			showSplash = false;
		}, 2500);
		const ready = appInit().then((init) => {
			clearTimeout(initTimer);
			initialized = true;
			appState.devMode = init.devMode;
			if (init.account) {
				let savedSkinData: { hasCape?: boolean; capeType?: CapeType; customCapeUrl?: string } | null = null;
				if (typeof window !== "undefined") {
					try {
						const raw = localStorage.getItem("luxmc_active_skin_data");
						if (raw) savedSkinData = JSON.parse(raw);
					} catch {}
				}

				const currentSkin = activeSkinStore.current;
				const savedCapeType = savedSkinData?.capeType ?? currentSkin.capeType;
				const savedHasCape = savedSkinData?.hasCape ?? currentSkin.hasCape;
				const savedCustomCapeUrl = savedSkinData?.customCapeUrl ?? currentSkin.customCapeUrl;

				const hasUserChosenCape = Boolean(savedHasCape && savedCapeType && savedCapeType !== "none");
				let capeType: CapeType = "none";
				let hasCape = false;
				let customCapeUrl = "";
				let effectiveCapeUrl: string | null = null;

				if (hasUserChosenCape && savedCapeType) {
					hasCape = true;
					capeType = savedCapeType;
					customCapeUrl = savedCustomCapeUrl || "";
					effectiveCapeUrl = savedCapeType === "custom"
						? (customCapeUrl || init.account.capeUrl || null)
						: (getFullCapeDataUrl(savedCapeType) || null);
				} else if (init.account.capeUrl) {
					hasCape = true;
					capeType = "custom";
					customCapeUrl = init.account.capeUrl;
					effectiveCapeUrl = init.account.capeUrl;
				}

				account.value = {
					id: init.account.id,
					username: init.account.username,
					uuid: init.account.uuid,
					minecraftToken: init.account.accessToken ?? "",
					expiresAt: init.account.expiresAt ? new Date(init.account.expiresAt).getTime() : 0,
					skinUrl: init.account.skinUrl ?? null,
					skinVariant: init.account.skinVariant ?? null,
					capeUrl: effectiveCapeUrl,
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
					hasCape,
					capeType,
					customCapeUrl
				});
				if (hasUserChosenCape && effectiveCapeUrl && effectiveCapeUrl !== init.account.capeUrl) {
					void authSetAccountCape(init.account.id, effectiveCapeUrl).catch(() => {});
				}
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
                autoOptimize: p.autoOptimize, useVulkan: p.useVulkan, launchCount: p.launchCount,
				group: p.instanceGroup ?? undefined,
			}));
			if (init.activeProfileId) {
				profiles.activeId = init.activeProfileId;
			}
			initialized = true;
			if (settings.value.discordRpc !== false) {
				discordSetActivity({
					details: "No Menu Principal",
					state: "Luxmc v1.9.0",
					largeText: "Luxmc Launcher",
					largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
					smallImage: "grass",
					smallText: "Luxmc v1.9.0",
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

		setToastInstance(toastsInstance!);

		let unlistenGameExit: (() => void) | undefined;
		let unlistenTelemetry: (() => void) | undefined;
		let disposed = false;
        let unlistenDeepLinks: (() => void) | undefined;
        let linkQueue = Promise.resolve();
        void ready.then(async () => {
            if (disposed) return;
            const stop = await listenDeepLinks(urls => {
                for (const url of urls) {
                    linkQueue = linkQueue.then(async () => {
                        if (disposed) return;
                        showSplash = false;
                        appState.showCutscene = false;
                        await handleDeepLink(url);
                    }).catch(error => { toast(`Não foi possível abrir o link: ${String(error)}`, "error"); });
                }
            });
            if (disposed) stop(); else unlistenDeepLinks = stop;
        }).catch(error => { if (!disposed) toast(`Integração com o portal indisponível: ${String(error)}`, "error"); });


		const exitPromise = listenGameExit((event) => {
			if (disposed) return;
			gamingStats.onGameExit();
			const lastProfileId = appState.activeGameDetails?.profileId || profiles.activeId || "";
			appState.isGameRunning = false;
			appState.activeGameDetails = null;

			if (settings.value.launcherActionOnLaunch === "hide_reopen") {
				import("@tauri-apps/api/window").then(({ getCurrentWindow }) => {
					const win = getCurrentWindow();
					win.show().then(() => win.setFocus()).catch(() => {});
				}).catch(() => {});
			}

			if (!event.success || event.code !== 0) {
				const detail = event.errorMessage ? `\nMotivo: ${event.errorMessage}` : " Consulte a aba de Logs para detalhes.";
				toast(`O Minecraft encerrou com código de saída ${event.code}.${detail}`, "error");

				if (settings.value.showLogsOnLaunch === "on_crash") {
					import("$app/navigation").then(({ goto }) => goto("/logs")).catch(() => {});
				}
			} else {
				achievements.unlock("primeira_noite");
			}
			if (settings.value.discordRpc !== false) {
				discordSetActivity({
					details: "No Menu Principal",
					state: "Luxmc v1.9.0",
					largeText: "Luxmc Launcher",
					largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
					smallImage: "grass",
					smallText: "Luxmc v1.9.0",
					inGame: false
				}).catch(() => {});
			}

			if (settings.value.soundscapesEnabled === true && !appState.performanceMode) {
				startSoundscape("overworld");
			}
		});
		exitPromise.then((unlisten) => {
			if (disposed) { unlisten(); return; }
			unlistenGameExit = unlisten;
		}).catch(() => {});

		const telemetryPromise = listenGameTelemetry((summary) => {
			if (disposed) return;
			telemetryData = summary;
			showTelemetryModal = false;
		});
		telemetryPromise.then((unlisten) => {
			if (disposed) { unlisten(); return; }
			unlistenTelemetry = unlisten;
		}).catch(() => {});

		let unlistenOverlayToggle: (() => void) | undefined;
		import("@tauri-apps/api/event").then(({ listen }) => {
			listen("luxmc-toggle-overlay", () => {
				if (disposed) return;
				clientMods.toggleMenu();
				if (!clientMods.isMenuOpen) {
					if (appState.isGameRunning) {
						clientOverlayClose().catch(() => {});
					}
				}
			}).then((unlisten) => {
				if (disposed) unlisten();
				else unlistenOverlayToggle = unlisten;
			});
		}).catch(() => {});

		let soundscapeTimer: ReturnType<typeof setTimeout> | undefined;
		if (settings.value.soundscapesEnabled === true && !appState.performanceMode) {
			soundscapeTimer = setTimeout(() => {
				if (!disposed) startSoundscape("overworld");
			}, 1200);
		}

		const stop = startAutoPersist();
		
		return () => {
			disposed = true;
			stop();
			if (unlistenGameExit) unlistenGameExit();
            unlistenDeepLinks?.();
			if (unlistenTelemetry) unlistenTelemetry();
			if (unlistenOverlayToggle) unlistenOverlayToggle();
			if (soundscapeTimer) clearTimeout(soundscapeTimer);
			stopSoundscape();
			destroyAudio();
			gamingStats.destroy();
		};
	});

	let paletteDebounce: ReturnType<typeof setTimeout> | null = null;
	$effect(() => {
		const bannerOrIcon = profiles.active?.banner || profiles.active?.icon;
		if (bannerOrIcon) {
			if (paletteDebounce) clearTimeout(paletteDebounce);
			paletteDebounce = setTimeout(() => {
				applyAdaptivePalette(bannerOrIcon);
			}, 300);
		}
		return () => {
			if (paletteDebounce) clearTimeout(paletteDebounce);
		};
	});

	$effect(() => {
		if (appState.isGameRunning) {
			stopSoundscape();
		}
	});

	let rpcTimeout: ReturnType<typeof setTimeout> | null = null;
	$effect(() => {
		const currentPath = page.url.pathname;
		if (settings.value.discordRpc === false) return;
		if (appState.isGameRunning) return;

		if (rpcTimeout) clearTimeout(rpcTimeout);
		rpcTimeout = setTimeout(() => {
			let details = "No Menu Principal";
			let state = "Luxmc v1.9.0";

			if (currentPath === "/") {
				details = "No Menu Principal";
				state = "Pronto para Jogar";
			} else if (currentPath === "/instances") {
				details = "Gerenciando Instâncias";
				state = "Luxmc v1.9.0";
			} else if (currentPath.startsWith("/instances/")) {
				details = "Configurando Instância";
				state = "Ajustando Mods & Versões";
			} else if (currentPath === "/mods") {
				details = "Explorando Mods & Modpacks";
				state = "Modrinth & CurseForge";
			} else if (currentPath === "/skins") {
				details = "Personalizador de Skins 3D";
				state = "Customizando Aparência";
			} else if (currentPath === "/screenshots") {
				details = "Galeria de Capturas de Tela";
				state = "Visualizando Screenshots";
			} else if (currentPath === "/logs" || currentPath === "/logs-history") {
				details = "Analisando Logs";
				state = "Diagnóstico do Jogo";
			} else if (currentPath === "/settings") {
				details = "Configurações do Launcher";
				state = "Ajustando Preferências";
			}

			discordSetActivity({
				details,
				state,
				largeText: "Luxmc Launcher",
				largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
				smallText: "Luxmc v1.9.0",
				inGame: false,
				buttons: [
					{ label: "Baixar Luxmc", url: "https://luxmc-r92.pages.dev" },
					{ label: "Site Oficial", url: "https://luxmc-r92.pages.dev" }
				]
			}).catch(() => {});
		}, 300);

		return () => {
			if (rpcTimeout) {
				clearTimeout(rpcTimeout);
				rpcTimeout = null;
			}
		};
	});
	$effect(() => {
		const konami = ['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'b', 'a'];
		let pos = 0;
		const handler = (e: KeyboardEvent) => {
			if (e.code === "ShiftRight") {
				const tag = (e.target as HTMLElement)?.tagName?.toLowerCase();
				if (tag !== "input" && tag !== "textarea") {
					e.preventDefault();
					clientMods.toggleMenu();
					if (!clientMods.isMenuOpen) {
						if (appState.isGameRunning) {
							clientOverlayClose().catch(() => {});
						}
					}
					return;
				}
			}
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

<div class="fixed inset-0 z-0 transition-all duration-500 pointer-events-none" style={themeStore.currentBackgroundStyle}>
	{#if themeStore.theme !== "light" && !appState.performanceMode}
		<div class="absolute inset-0 opacity-25 pointer-events-none" style="background: radial-gradient(circle at 20% -10%, rgb(var(--brand-500)) 0%, transparent 55%);"></div>
		<div class="absolute inset-0 opacity-15 pointer-events-none" style="background: radial-gradient(circle at 85% 110%, rgb(var(--ambient-accent, var(--brand-500))) 0%, transparent 55%);"></div>
		{#if settings.value.liveWallpaper === true}
			<LiveWallpaper />
		{/if}
	{:else if themeStore.theme === "light" && !appState.performanceMode}
		<div class="absolute inset-0 opacity-10 pointer-events-none" style="background: radial-gradient(circle at 20% -10%, rgb(var(--brand-500)) 0%, transparent 45%);"></div>
		<div class="absolute inset-0 opacity-5 pointer-events-none" style="background: radial-gradient(circle at 85% 110%, rgb(var(--brand-500)) 0%, transparent 45%);"></div>
	{/if}
</div>
<Toasts bind:this={toastsInstance} />
<StatusBanner />

<ErrorBoundary>
{#if showSplash || appState.showCutscene}
	<Cutscene onComplete={() => { showSplash = false; appState.showCutscene = false; initialized = true; }} />
{:else if !initialized}
	<div class="flex h-full w-full items-center justify-center bg-bg-overlay/70" in:fade={{ duration: 150 }}>
		<div class="flex flex-col items-center gap-4">
			<div class="h-10 w-10 border-4 border-t-brand-400 border-fg/10 rounded-full animate-spin"></div>
			<p class="text-sm font-medium text-fg shadow-black drop-shadow-md">{t("app.loading")}</p>
		</div>
	</div>
{:else if !account.value}
	<div class="flex h-full w-full items-center justify-center bg-bg/90" in:fade={{ duration: 150 }}>
		{@render children?.()}
	</div>
{:else}
	<div class="flex h-full w-full overflow-hidden" in:fade={{ duration: 100 }}>
		<Sidebar notificationCount={0} />
		<div class="flex h-full min-w-0 flex-1 flex-col relative z-10">
			<main class="flex-1 overflow-x-hidden overflow-y-auto px-6 py-6 custom-scrollbar relative">
				<div class="mx-auto max-w-[1600px] min-h-full flex flex-col w-full">
					{@render children?.()}
				</div>
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
{#if appState.showProfileModal}
	<ProfileModal onClose={() => appState.showProfileModal = false} />
{/if}
<TelemetryModal
	summary={showTelemetryModal ? telemetryData : null}
	onClose={() => showTelemetryModal = false}
/>
</ErrorBoundary>
