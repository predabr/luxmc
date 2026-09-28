<script lang="ts">
	import { resolveWallpaperVideoUrl, resolveWallpaperImageUrl } from "$lib/utils/wallpaperSource";
	import VideoWallpaper from "$lib/components/visuals/VideoWallpaper.svelte";
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
	import { account, saveCurrentAccount, loadCurrentAccount } from "$lib/stores/account.svelte";
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
	import { createSkinAvatar } from "$lib/utils/textureImage";
	import { crashDoctor } from "$lib/stores/crashDoctor.svelte";
	import { achievements } from "$lib/stores/achievements.svelte";
	import { clientMods } from "$lib/stores/clientMods.svelte";
	import { applyAdaptivePalette } from "$lib/utils/adaptivePalette";
	import { appInit, discordSetActivity, listenGameExit, listenGameStateChange, crashDoctorDiagnose, clientOverlayClose, authSetAccountCape } from "$lib/api";
	import { optimizerTrimMemory } from "$lib/api/instances";
	import { useTranslation, setActiveLocale } from "$lib/i18n/useTranslation.svelte";
	import { themeStore } from "$lib/stores/theme.svelte";
	const { t } = useTranslation();
	$effect(() => {
		const lang = settings.value.language;
		if (lang) {
			setActiveLocale(lang);
		}
	});
	let { children }: { children: import("svelte").Snippet } = $props();
	let initialized = $state(false);
	let showSplash = $state(true);
	let toastsInstance = $state<Toasts | null>(null);
	let telemetryData = $state<GameTelemetrySummary | null>(null);
	let showTelemetryModal = $state(false);
	const launcherStartTime = Math.floor(Date.now() / 1000);
	let gameStartTime = $state<number | null>(null);

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

				const skinModel = init.account.skinVariant?.toLowerCase() === "slim" ? "alex" : "steve";
				const skinUrl = init.account.skinUrl || `https://minotar.net/skin/${init.account.username}`;
				const fallbackAvatarUrl = `https://mc-heads.net/avatar/${init.account.username}/100`;

				const newAcc = {
					id: init.account.id,
					username: init.account.username,
					uuid: init.account.uuid,
					minecraftToken: init.account.accessToken ?? "",
					expiresAt: init.account.expiresAt ? new Date(init.account.expiresAt).getTime() : 0,
					skinUrl: init.account.skinUrl ?? null,
					skinVariant: init.account.skinVariant ?? null,
					capeUrl: effectiveCapeUrl,
					avatarUrl: fallbackAvatarUrl,
				};
				account.value = newAcc;
				void saveCurrentAccount(newAcc);

				activeSkinStore.setSkin({
					id: init.account.uuid,
					name: init.account.username,
					url: `https://mc-heads.net/body/${init.account.username}/300`,
					skinUrl,
					avatarUrl: fallbackAvatarUrl,
					type: skinModel,
					hasCape,
					capeType,
					customCapeUrl
				});

				if (skinUrl) {
					void createSkinAvatar(skinUrl, new AbortController().signal, skinModel)
						.then((avatar) => {
							if (avatar) {
								activeSkinStore.setSkin({ avatarUrl: avatar });
								if (account.value && account.value.id === init.account?.id) {
									const withAvatar = { ...account.value, avatarUrl: avatar };
									account.value = withAvatar;
									void saveCurrentAccount(withAvatar);
								}
							}
						})
						.catch(() => {});
				}
				if (hasUserChosenCape && effectiveCapeUrl && effectiveCapeUrl !== init.account.capeUrl) {
					void authSetAccountCape(init.account.id, effectiveCapeUrl).catch(() => {});
				}
			} else {
				void loadCurrentAccount();
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
					state: "Luxmc v2.0.2",
					largeText: "Luxmc Launcher",
					largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
					smallImage: "grass",
					smallText: "Luxmc v2.0.2",
					startTime: launcherStartTime,
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

			const wasManual = appState.wasManuallyTerminated || event.code === 143 || event.code === 137 || event.code === 130;
			appState.wasManuallyTerminated = false;

			if (!wasManual && (!event.success || (event.code !== 0 && event.code !== null))) {
				if (lastProfileId) {
					crashDoctor.autoHealCrash(lastProfileId, event.errorMessage).then((healed) => {
						if (!healed) {
							const detail = event.errorMessage ? `\nMotivo: ${event.errorMessage}` : " Consulte a aba de Logs para detalhes.";
							toast(`O Minecraft encerrou com código de saída ${event.code}.${detail}`, "error");

							if (settings.value.showLogsOnLaunch === "on_crash") {
								import("$app/navigation").then(({ goto }) => goto("/logs")).catch(() => {});
							}
						}
					}).catch(() => {
						const detail = event.errorMessage ? `\nMotivo: ${event.errorMessage}` : " Consulte a aba de Logs para detalhes.";
						toast(`O Minecraft encerrou com código de saída ${event.code}.${detail}`, "error");

						if (settings.value.showLogsOnLaunch === "on_crash") {
							import("$app/navigation").then(({ goto }) => goto("/logs")).catch(() => {});
						}
					});
				} else {
					const detail = event.errorMessage ? `\nMotivo: ${event.errorMessage}` : " Consulte a aba de Logs para detalhes.";
					toast(`O Minecraft encerrou com código de saída ${event.code}.${detail}`, "error");

					if (settings.value.showLogsOnLaunch === "on_crash") {
						import("$app/navigation").then(({ goto }) => goto("/logs")).catch(() => {});
					}
				}
			} else {
				achievements.unlock("primeira_noite");
			}
			if (settings.value.discordRpc !== false) {
				discordSetActivity({
					details: "No Menu Principal",
					state: "Luxmc v2.0.2",
					largeText: "Luxmc Launcher",
					largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
					smallImage: "grass",
					smallText: "Luxmc v2.0.2",
					startTime: launcherStartTime,
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


		let unlistenGameState: (() => void) | undefined;
		const statePromise = listenGameStateChange((event) => {
			if (disposed) return;
			if (settings.value.discordRpc === false) return;
			if (!appState.isGameRunning) return;

			if (!gameStartTime) {
				gameStartTime = Math.floor(Date.now() / 1000);
			}

			const activeDetails = appState.activeGameDetails;
			const pName = activeDetails?.name || "Minecraft";
			const verId = activeDetails?.version || "1.20.1";
			const loader = activeDetails?.loader ? activeDetails.loader.toUpperCase() : "Vanilla";
			const profile = activeDetails?.profileId ? profiles.list.find(p => p.id === activeDetails.profileId) : null;
			const modInfo = profile?.modCount ? ` · ${profile.modCount} mods` : "";

			let details = pName;
			let state = `Minecraft ${verId} · ${loader}${modInfo}`;

			if (event.status === "singleplayer") {
				if (settings.value.hideDiscordDetails) {
					details = "No Modo Singleplayer";
				} else if (event.detail) {
					details = `Mundo: ${event.detail}`;
				} else {
					details = "No Modo Singleplayer";
				}
			} else if (event.status === "multiplayer") {
				if (settings.value.hideDiscordDetails) {
					details = "Em Servidor Multiplayer";
				} else if (event.detail) {
					details = `Servidor: ${event.detail}`;
				} else {
					details = "Em Servidor Multiplayer";
				}
			} else if (event.status === "menu") {
				details = "No Menu do Jogo";
			}

			discordSetActivity({
				inGame: true,
				details,
				state,
				largeText: `${pName} (${verId})`,
				largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
				smallImage: "grass",
				smallText: `Luxmc · ${loader}`,
				startTime: gameStartTime || launcherStartTime,
				buttons: [
					{ label: "Baixar Luxmc", url: "https://luxmc-r92.pages.dev" },
					{ label: "Site Oficial", url: "https://luxmc-r92.pages.dev" }
				]
			}).catch(() => {});
		});
		statePromise.then((unlisten) => {
			if (disposed) { unlisten(); return; }
			unlistenGameState = unlisten;
		}).catch(() => {});

		let soundscapeTimer: ReturnType<typeof setTimeout> | undefined;
		if (settings.value.soundscapesEnabled === true && !appState.performanceMode) {
			soundscapeTimer = setTimeout(() => {
				if (!disposed) startSoundscape("overworld");
			}, 1200);
		}

		const rpcHeartbeat = setInterval(() => {
			if (disposed || settings.value.discordRpc === false) return;
			if (!appState.isGameRunning) {
				const currentPath = page.url.pathname;
				let details = "No Menu Principal";
				let state = "Luxmc v2.0.2";
				if (currentPath === "/") {
					details = "No Menu Principal";
					state = "Pronto para Jogar";
				} else if (currentPath === "/instances") {
					details = "Gerenciando Instâncias";
					state = `${profiles.list.length} instâncias criadas`;
				} else if (currentPath.startsWith("/instances/")) {
					details = profiles.active ? `Ajustando ${profiles.active.name}` : "Configurando Instância";
					state = profiles.active ? `${profiles.active.mcVersion} · ${profiles.active.loader.toUpperCase()}` : "Ajustando Mods & Versões";
				} else if (currentPath === "/mods") {
					details = "Explorando Mods & Modpacks";
					state = "Modrinth & CurseForge";
				} else if (currentPath === "/skins") {
					details = "Personalizador de Skins 3D";
					state = account.value?.username ? `Skin de ${account.value.username}` : "Customizando Aparência";
				} else if (currentPath === "/friends") {
					details = "Comunidade & Amigos";
					state = "Rede P2P Luxmc";
				} else if (currentPath === "/settings") {
					details = "Configurações do Launcher";
					state = "Ajustando Preferências";
				}
				discordSetActivity({
					details,
					state,
					largeText: "Luxmc Launcher",
					largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
					smallImage: "grass",
					smallText: "Luxmc v2.0.2",
					startTime: launcherStartTime,
					inGame: false,
					buttons: [
						{ label: "Baixar Luxmc", url: "https://luxmc-r92.pages.dev" },
						{ label: "Site Oficial", url: "https://luxmc-r92.pages.dev" }
					]
				}).catch(() => {});
			}
		}, 25000);

		const stop = startAutoPersist();

		const handleExternalLink = (e: MouseEvent) => {
			const a = (e.target as HTMLElement | null)?.closest("a");
			if (!a) return;
			const href = a.getAttribute("href") || a.href;
			if (href && (href.startsWith("http://") || href.startsWith("https://") || href.startsWith("mailto:"))) {
				e.preventDefault();
				e.stopPropagation();
				import("@tauri-apps/plugin-opener").then(({ openUrl }) => openUrl(href)).catch(() => {});
			}
		};
		window.addEventListener("click", handleExternalLink, { capture: true });
		
		return () => {
			disposed = true;
			window.removeEventListener("click", handleExternalLink, { capture: true });
			stop();
			clearInterval(rpcHeartbeat);
			if (unlistenGameExit) unlistenGameExit();
			if (unlistenGameState) unlistenGameState();
            unlistenDeepLinks?.();
			if (unlistenTelemetry) unlistenTelemetry();
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
			if (!gameStartTime) {
				gameStartTime = Math.floor(Date.now() / 1000);
			}
			stopSoundscape();
			if (rpcTimeout) {
				clearTimeout(rpcTimeout);
				rpcTimeout = null;
			}
		} else {
			gameStartTime = null;
		}
	});

	let rpcTimeout: ReturnType<typeof setTimeout> | null = null;
	$effect(() => {
		const currentPath = page.url.pathname;
		if (settings.value.discordRpc === false) return;
		if (appState.isGameRunning) return;

		if (rpcTimeout) clearTimeout(rpcTimeout);
		rpcTimeout = setTimeout(() => {
			if (appState.isGameRunning) return;
			let details = "No Menu Principal";
			let state = "Luxmc v2.0.2";

			if (currentPath === "/") {
				details = "No Menu Principal";
				state = "Pronto para Jogar";
			} else if (currentPath === "/instances") {
				details = "Gerenciando Instâncias";
				state = `${profiles.list.length} instâncias criadas`;
			} else if (currentPath.startsWith("/instances/")) {
				details = profiles.active ? `Ajustando ${profiles.active.name}` : "Configurando Instância";
				state = profiles.active ? `${profiles.active.mcVersion} · ${profiles.active.loader.toUpperCase()}` : "Ajustando Mods & Versões";
			} else if (currentPath === "/mods") {
				details = "Explorando Mods & Modpacks";
				state = "Modrinth & CurseForge";
			} else if (currentPath === "/skins") {
				details = "Personalizador de Skins 3D";
				state = account.value?.username ? `Skin de ${account.value.username}` : "Customizando Aparência";
			} else if (currentPath === "/screenshots") {
				details = "Galeria de Capturas de Tela";
				state = "Visualizando Screenshots";
			} else if (currentPath === "/logs" || currentPath === "/logs-history") {
				details = "Analisando Logs";
				state = "Diagnóstico do Jogo";
			} else if (currentPath === "/friends") {
				details = "Comunidade & Amigos";
				state = "Rede P2P Luxmc";
			} else if (currentPath === "/settings") {
				details = "Configurações do Launcher";
				state = "Ajustando Preferências";
			}

			discordSetActivity({
				details,
				state,
				largeText: "Luxmc Launcher",
				largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
				smallImage: "grass",
				smallText: "Luxmc v2.0.2",
				startTime: launcherStartTime,
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

	const resolvedVideoUrl = $derived(
		themeStore.customWallpaperType === "video"
			? resolveWallpaperVideoUrl(themeStore.customWallpaperUrl)
			: ""
	);

    $effect(() => {
        const disableBlur = settings.value.blur === false || appState.performanceMode;
        document.documentElement.classList.toggle("no-blur", disableBlur);
        document.documentElement.classList.toggle("efficient-wallpaper", themeStore.background === "custom" && themeStore.customWallpaperType === "video" && settings.value.animatedWallpaperBlur !== true);
        return () => {
            document.documentElement.classList.remove("no-blur");
            document.documentElement.classList.remove("efficient-wallpaper");
        };
    });

</script>

<div class="fixed inset-0 z-0 transition-colors duration-300 pointer-events-none overflow-hidden" style={themeStore.currentBackgroundStyle}>
	{#if themeStore.background === "custom" && themeStore.customWallpaperUrl}
		{#if themeStore.customWallpaperType === "video"}
			<VideoWallpaper src={resolvedVideoUrl} />
		{:else}
			<img
				src={resolveWallpaperImageUrl(themeStore.customWallpaperUrl)}
				alt="Plano de fundo personalizado"
				class="absolute inset-0 w-full h-full object-cover pointer-events-none"
			/>
		{/if}
		<div class="absolute inset-0 bg-black/40 pointer-events-none"></div>
	{/if}
	{#if themeStore.theme !== "light" && !appState.performanceMode && !appState.isGameRunning && (themeStore.background !== "custom" || !themeStore.customWallpaperUrl)}
		{#if settings.value.liveWallpaper === true}
			<LiveWallpaper />
		{/if}
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
	<div class="flex h-dvh min-h-0 w-full overflow-hidden" in:fade={{ duration: 100 }}>
		<Sidebar notificationCount={0} />
		<div class="flex h-full min-h-0 min-w-0 flex-1 flex-col relative z-10">
			<main class="min-h-0 flex-1 overflow-x-hidden overflow-y-auto {page.url.pathname === "/" ? "p-0" : "px-6 py-6"} custom-scrollbar relative">
				<div class="mx-auto {page.url.pathname === "/" ? "" : "max-w-[1600px]"} min-h-full flex flex-col w-full">
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
