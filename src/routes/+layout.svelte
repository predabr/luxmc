<script lang="ts">
import { APP_VERSION } from "$lib/version";
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	import { runtimePlatform } from "$lib/stores/platform.svelte";
	import { scrollActivity } from "$lib/utils/scrollActivity";
	import { pageMotion } from "$lib/actions/pageMotion";
	import { resolveWallpaperImageUrl } from "$lib/utils/wallpaperSource";
	import VideoWallpaper from "$lib/components/visuals/VideoWallpaper.svelte";
    import ShaderScenery from "$lib/components/visuals/ShaderScenery.svelte";
	import "../app.css";
	import { listenDeepLinks } from "$lib/api/deepLinks";
	import { handleDeepLink } from "$lib/utils/handleDeepLink";
	import { onMount, untrack } from "svelte";
	import { fade } from "svelte/transition";
	import { quintOut } from "svelte/easing";
	import { page } from "$app/state";
	import { afterNavigate } from "$app/navigation";
	import Sidebar from "$lib/components/layout/Sidebar.svelte";
	import PublicProfileModal from "$lib/components/friends/PublicProfileModal.svelte";
	import BedrockDownloadStatus from "$lib/components/instances/BedrockDownloadStatus.svelte";
    import ModpackDownloadStatus from "$lib/components/mods/ModpackDownloadStatus.svelte";
    import RoomPreparationStatus from "$lib/components/friends/RoomPreparationStatus.svelte";
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
	import ClientOverlayModal from "$lib/components/ui/ClientOverlayModal.svelte";
	import ProfileModal from "$lib/components/profile/ProfileModal.svelte";
	import { listenOverlayToggle } from "$lib/api/events";
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
            if (id?.startsWith("luxmc:")) cloudAccount.connect(id);
            if (id) void friendsState.connect();
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
	import { preloadRoute } from "$lib/utils/preloadRoute";
	import { appInit, discordSetActivity, discordClearActivity, listenGameExit, listenGameStateChange, listenLaunchStage, listenShieldWarning, listenLauncherLog, listenGameLog, launchLogOpen, launchLogAppend, launchLogClose, crashDoctorDiagnose, clientOverlayClose, authSetAccountCape } from "$lib/api";
	import { optimizerTrimMemory } from "$lib/api/instances";
	import { useTranslation, setActiveLocale } from "$lib/i18n/useTranslation.svelte";
	import { themeStore } from "$lib/stores/theme.svelte";
	import { newsState } from "$lib/stores/news.svelte";
	onMount(() => newsState.startAutoRefresh());
	const { t } = useTranslation();
	$effect(() => {
		const lang = settings.value.language;
		if (lang) {
			setActiveLocale(lang);
		}
	});
	let { children }: { children: import("svelte").Snippet } = $props();

	afterNavigate(() => {
		const root = document.querySelector<HTMLElement>("[data-scroll-root]");
		if (root && root.scrollTop > 0) root.scrollTop = 0;
	});
	let initialized = $state(false);
	let showSplash = $state(true);
	let toastsInstance = $state<Toasts | null>(null);
	$effect(() => {
		setToastInstance(toastsInstance);
		return () => setToastInstance(null);
	});
    let latestDiscordActivity: Exclude<Parameters<typeof discordSetActivity>[0], string | undefined> | null = null;
    function updateDiscordActivity(activity: Exclude<Parameters<typeof discordSetActivity>[0], string | undefined>) {
        latestDiscordActivity = activity;
        const privateMode = settings.value.hideDiscordDetails || settings.value.streamerMode;
        return discordSetActivity({ ...activity, ...(privateMode ? { details: activity.inGame ? "Jogando Minecraft" : "No Luxmc", state: "Luxmc Launcher", largeText: "Minecraft via Luxmc" } : {}), clientId: settings.value.discordClientId || undefined });
    }
	const launcherStartTime = Math.floor(Date.now() / 1000);
	const LAUNCH_STAGE_LABELS: Record<string, string> = $derived({
		preparing: uiText("ui.21e7c4817452bf68"),
		checkingJava: "Verificando a runtime do Java...",
		resolvingClasspath: uiText("ui.4a2bda9086a2b484"),
		extractingNatives: uiText("ui.7aaa8c5b3528ffea"),
		resolvingArgs: uiText("ui.45eb67dd71456de2"),
		validatingArgs: "Validando argumentos da JVM...",
		spawning: "Iniciando o Minecraft...",
		running: uiText("ui.10096bc49ab1ac37"),
		failed: uiText("ui.48a319d9cbd85a5a")
	});

	let logSessionOpen: Promise<number | null> | null = null;
	let logSessionId = $state<number | null>(null);
	let logAppendQueue: Promise<unknown> = Promise.resolve();

	function openLogSession(): Promise<number | null> {
		if (!logSessionOpen) {
			const profileId = appState.launchingProfileId || profiles.activeId || null;
			const profile = profileId ? profiles.list.find((p) => p.id === profileId) : null;
			logSessionOpen = launchLogOpen(profileId, profile?.mcVersion || "unknown")
				.then((id) => {
					logSessionId = id;
					return id;
				})
				.catch(() => null);
		}
		return logSessionOpen;
	}

	function appendLogSession(stream: string, level: string, message: string) {
		logAppendQueue = logAppendQueue
			.then(() => openLogSession())
			.then((id) => {
				if (id === null) return;
				return launchLogAppend(id, stream, level, message).catch(() => {});
			})
			.catch(() => {});
	}

	function closeLogSession(exitCode: number | null, summary: string | null, classification: string | null) {
		const pending = logSessionOpen;
		logSessionOpen = null;
		logAppendQueue = logAppendQueue
			.then(() => pending)
			.then((id) => {
				const target = id ?? logSessionId;
				logSessionId = null;
				if (target === null) return;
				return launchLogClose(target, exitCode, summary, classification).catch(() => {});
			})
			.catch(() => {});
	}
	let gameStartTime = $state<number | null>(null);


	onMount(() => {
        const syncHidden=()=>document.documentElement.classList.toggle("launcher-hidden",document.hidden);
        syncHidden();document.addEventListener("visibilitychange",syncHidden);

		const rejectNativeFileNavigation = (e: Event) => {
			const types = (e as DragEvent).dataTransfer?.types;
			if (!types || !Array.from(types).includes("Files")) return;
			e.preventDefault();
		};
		document.addEventListener("dragover", rejectNativeFileNavigation, { passive: false });
		document.addEventListener("drop", rejectNativeFileNavigation, { passive: false });
		return () => {
			document.removeEventListener("visibilitychange",syncHidden);
			document.documentElement.classList.remove("launcher-hidden");
			document.removeEventListener("dragover", rejectNativeFileNavigation);
			document.removeEventListener("drop", rejectNativeFileNavigation);
		};
	});

	onMount(() => {
		themeStore.init();
		void runtimePlatform.refresh();
		void bootstrapSettings();
		void profiles.refresh();

		const warmRoutes = ["/", "/instances", "/mods", "/skins", "/news", "/settings", "/screenshots"];
		let warmIndex = 0;
		const warmTimer = setInterval(() => {
			if (document.hidden || appState.isGameRunning) return;
			if (warmIndex >= warmRoutes.length) {
				clearInterval(warmTimer);
				return;
			}
			preloadRoute(warmRoutes[warmIndex++], true);
		}, 160);

		const initTimer = setTimeout(() => {
			initialized = true;
		}, 2500);
		const ready = appInit().then((init) => {
			clearTimeout(initTimer);
			initialized = true;
			appState.devMode = init.devMode;
			if (init.tokenWarning) toast(init.tokenWarning, "warning");
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
				useGamemode: p.useGamemode, useMangohud: p.useMangohud,
				forceDedicatedGpu: p.forceDedicatedGpu, useGamescope: p.useGamescope,
				gamescopeWidth: p.gamescopeWidth, gamescopeHeight: p.gamescopeHeight,
				gamescopeFsr: p.gamescopeFsr, forceFullVerification: p.forceFullVerification,
				group: p.instanceGroup ?? undefined,
			}));
			if (init.activeProfileId) {
				profiles.activeId = init.activeProfileId;
			}
			initialized = true;
			if (settings.value.discordRpc !== false) {
				updateDiscordActivity({
					details: "No Menu Principal",
					state: `Luxmc v${APP_VERSION}`,
					largeText: "Luxmc Launcher",
					largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
					smallImage: "grass",
					smallText: `Luxmc v${APP_VERSION}`,
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


		let unlistenGameExit: (() => void) | undefined;
		let unlistenLaunchStage: (() => void) | undefined;
		let unlistenShieldWarning: (() => void) | undefined;
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
                    }).catch(error => { toast(uiText("ui.24db1db3ac8ca4f5", {arg0: (String(error))}), "error"); });
                }
            });
            if (disposed) stop(); else unlistenDeepLinks = stop;
        }).catch(error => { if (!disposed) toast(uiText("ui.8c50f378892f8d83", {arg0: (String(error))}), "error"); });


		const exitPromise = listenGameExit((event) => {
			if (disposed) return;
			closeLogSession(
				event.code,
				event.errorMessage || uiText("ui.f5b08a1d2113f1f0", {arg0: (event.code)}),
				event.success ? null : "crash"
			);
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
							const detail = event.errorMessage ? `\nMotivo: ${event.errorMessage}` : uiText("ui.06066dbe8f939774");
							toast(uiText("ui.bea5464ab84bc224", {arg0: (event.code), arg1: (detail)}), "error");

							if (settings.value.showLogsOnLaunch === "on_crash") {
								import("$app/navigation").then(({ goto }) => goto("/logs")).catch(() => {});
							}
						}
					}).catch(() => {
						const detail = event.errorMessage ? `\nMotivo: ${event.errorMessage}` : uiText("ui.06066dbe8f939774");
						toast(uiText("ui.bea5464ab84bc224", {arg0: (event.code), arg1: (detail)}), "error");

						if (settings.value.showLogsOnLaunch === "on_crash") {
							import("$app/navigation").then(({ goto }) => goto("/logs")).catch(() => {});
						}
					});
				} else {
					const detail = event.errorMessage ? `\nMotivo: ${event.errorMessage}` : uiText("ui.06066dbe8f939774");
					toast(uiText("ui.bea5464ab84bc224", {arg0: (event.code), arg1: (detail)}), "error");

					if (settings.value.showLogsOnLaunch === "on_crash") {
						import("$app/navigation").then(({ goto }) => goto("/logs")).catch(() => {});
					}
				}
			} else {
				achievements.unlock("primeira_noite");
			}
			if (settings.value.discordRpc !== false) {
				updateDiscordActivity({
					details: "No Menu Principal",
					state: `Luxmc v${APP_VERSION}`,
					largeText: "Luxmc Launcher",
					largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
					smallImage: "grass",
					smallText: `Luxmc v${APP_VERSION}`,
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
					details = uiText("ui.5a43ed8fe43ef37d", {arg0: (event.detail)});
				} else {
					details = "No Modo Singleplayer";
				}
			} else if (event.status === "multiplayer") {
				if (settings.value.hideDiscordDetails) {
					details = uiText("ui.b198ae036811b7a7");
				} else if (event.detail) {
					details = uiText("ui.ac105870d387335e", {arg0: (event.detail)});
				} else {
					details = uiText("ui.b198ae036811b7a7");
				}
			} else if (event.status === "menu") {
				details = uiText("ui.1f871d9c221034f4");
			}

			updateDiscordActivity({
				inGame: true,
				details,
				state,
				largeText: `${pName} (${verId})`,
				largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
				smallImage: "grass",
				smallText: `Luxmc · ${loader}`,
				startTime: gameStartTime || launcherStartTime,
				buttons: [
					{ label: uiText("ui.4c8757fab2a21345"), url: "https://luxmc-r92.pages.dev" },
					{ label: uiText("ui.942f94d869e404ae"), url: "https://luxmc-r92.pages.dev/skins.html" }
				]
			}).catch(() => {});
		});
		statePromise.then((unlisten) => {
			if (disposed) { unlisten(); return; }
			unlistenGameState = unlisten;
		}).catch(() => {});

		const stagePromise = listenLaunchStage((stage) => {
			if (disposed || appState.isGameRunning) return;
			const label = LAUNCH_STAGE_LABELS[stage];
			if (label) appState.launchStatusText = label;
		});
		stagePromise.then((unlisten) => {
			if (disposed) { unlisten(); return; }
			unlistenLaunchStage = unlisten;
		}).catch(() => {});

		let unlistenOverlayToggle: (() => void) | undefined;
		const overlayPromise = listenOverlayToggle(() => {
			if (disposed) return;
			clientMods.toggleMenu();
		});
		overlayPromise.then((unlisten) => {
			if (disposed) { unlisten(); return; }
			unlistenOverlayToggle = unlisten;
		}).catch(() => {});

		const shieldPromise = listenShieldWarning((result) => {
			if (disposed) return;
			const threats = result.threats ?? [];
			if (threats.length === 0) return;
			const critical = threats.filter((threat) => threat.severity === "critical").length;
			const names = threats.slice(0, 3).map((threat) => threat.fileName).join(", ");
			const extra = threats.length > 3 ? ` +${threats.length - 3}` : "";
			if (critical > 0) {
				toast(uiText("ui.adac79731be368b6", {arg0: (critical), arg1: (names), arg2: (extra)}), "error");
			} else {
				toast(uiText("ui.c2aef1d1f433ee72", {arg0: (threats.length), arg1: (names), arg2: (extra)}), "warning");
			}
		});
		shieldPromise.then((unlisten) => {
			if (disposed) { unlisten(); return; }
			unlistenShieldWarning = unlisten;
		}).catch(() => {});


		let unlistenLauncherLog: (() => void) | undefined;
		let unlistenGameLog: (() => void) | undefined;

		const launcherLogPromise = listenLauncherLog((message) => {
			if (disposed) return;
			appendLogSession("launcher", "INFO", message);
		});
		launcherLogPromise.then((unlisten) => {
			if (disposed) { unlisten(); return; }
			unlistenLauncherLog = unlisten;
		}).catch(() => {});

		const gameLogPromise = listenGameLog((entry) => {
			if (disposed) return;
			appendLogSession(entry.stream, entry.stream === "stderr" ? "WARN" : "INFO", entry.message);
		});
		gameLogPromise.then((unlisten) => {
			if (disposed) { unlisten(); return; }
			unlistenGameLog = unlisten;
		}).catch(() => {});

		let soundscapeTimer: ReturnType<typeof setTimeout> | undefined;
		if (settings.value.soundscapesEnabled === true && !appState.performanceMode) {
			soundscapeTimer = setTimeout(() => {
				if (!disposed) startSoundscape("overworld");
			}, 1200);
		}

		const rpcHeartbeat = setInterval(() => {
			if (disposed || settings.value.discordRpc === false || (document.hidden && !appState.isGameRunning)) return;
            if (appState.isGameRunning) { if (latestDiscordActivity?.inGame) updateDiscordActivity(latestDiscordActivity).catch(() => {}); return; }
			if (!appState.isGameRunning) {
				const currentPath = page.url.pathname;
				let details = "No Menu Principal";
				let state = `Luxmc v${APP_VERSION}`;
				if (currentPath === "/") {
					details = "No Menu Principal";
					state = uiText("home.readyToPlay");
				} else if (currentPath === "/instances") {
					details = uiText("ui.415c231bb2db2014");
					state = uiText("ui.5e388a4c2ba47ae3", {arg0: (profiles.list.length)});
				} else if (currentPath.startsWith("/instances/")) {
					details = profiles.active ? `Ajustando ${profiles.active.name}` : uiText("ui.8934ad0e0aac239f");
					state = profiles.active ? `${profiles.active.mcVersion} · ${profiles.active.loader.toUpperCase()}` : uiText("ui.28147e5059f65a54");
				} else if (currentPath === "/mods") {
					details = "Explorando Mods & Modpacks";
					state = "Modrinth & CurseForge";
				} else if (currentPath === "/skins") {
					details = "Personalizador de Skins 3D";
					state = account.value?.username ? `Skin de ${account.value.username}` : uiText("ui.8a669dcfd02103be");
				} else if (currentPath === "/friends") {
					details = uiText("ui.583a0c2814f1f2ad");
					state = "Rede P2P Luxmc";
				} else if (currentPath === "/settings") {
					details = uiText("ui.7cccac6544c65648");
					state = uiText("ui.d632e0cd3495ec0f");
				}
				updateDiscordActivity({
					details,
					state,
					largeText: "Luxmc Launcher",
					largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
					smallImage: "grass",
					smallText: `Luxmc v${APP_VERSION}`,
					startTime: launcherStartTime,
					inGame: false,
					buttons: [
						{ label: uiText("ui.4c8757fab2a21345"), url: "https://luxmc-r92.pages.dev" },
						{ label: uiText("ui.942f94d869e404ae"), url: "https://luxmc-r92.pages.dev/skins.html" }
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
			clearTimeout(initTimer);
			window.removeEventListener("click", handleExternalLink, { capture: true });
			stop();
			clearInterval(rpcHeartbeat);
			clearInterval(warmTimer);
			if (unlistenGameExit) unlistenGameExit();
			if (unlistenGameState) unlistenGameState();
            unlistenDeepLinks?.();
			if (unlistenLaunchStage) unlistenLaunchStage();
			if (unlistenLauncherLog) unlistenLauncherLog();
			if (unlistenGameLog) unlistenGameLog();
			if (unlistenShieldWarning) unlistenShieldWarning();
			if (unlistenOverlayToggle) unlistenOverlayToggle();
			if (soundscapeTimer) clearTimeout(soundscapeTimer);
			stopSoundscape();
			destroyAudio();
			gamingStats.destroy();
		};
	});

	$effect(() => {
		if (settings.value.discordRpc === false) {
			discordClearActivity().catch(() => {});
		}
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
			let state = `Luxmc v${APP_VERSION}`;

			if (currentPath === "/") {
				details = "No Menu Principal";
				state = uiText("home.readyToPlay");
			} else if (currentPath === "/instances") {
				details = uiText("ui.415c231bb2db2014");
				state = uiText("ui.5e388a4c2ba47ae3", {arg0: (profiles.list.length)});
			} else if (currentPath.startsWith("/instances/")) {
				details = profiles.active ? `Ajustando ${profiles.active.name}` : uiText("ui.8934ad0e0aac239f");
				state = profiles.active ? `${profiles.active.mcVersion} · ${profiles.active.loader.toUpperCase()}` : uiText("ui.28147e5059f65a54");
			} else if (currentPath === "/mods") {
				details = "Explorando Mods & Modpacks";
				state = "Modrinth & CurseForge";
			} else if (currentPath === "/skins") {
				details = "Personalizador de Skins 3D";
				state = account.value?.username ? `Skin de ${account.value.username}` : uiText("ui.8a669dcfd02103be");
			} else if (currentPath === "/screenshots") {
				details = uiText("ui.96f82c5751ca27e2");
				state = "Visualizando Screenshots";
			} else if (currentPath === "/logs" || currentPath === "/logs-history") {
				details = "Analisando Logs";
				state = uiText("ui.81a1eaf0b2009051");
			} else if (currentPath === "/friends") {
				details = uiText("ui.583a0c2814f1f2ad");
				state = "Rede P2P Luxmc";
			} else if (currentPath === "/settings") {
				details = uiText("ui.7cccac6544c65648");
				state = uiText("ui.d632e0cd3495ec0f");
			}

			updateDiscordActivity({
				details,
				state,
				largeText: "Luxmc Launcher",
				largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
				smallImage: "grass",
				smallText: `Luxmc v${APP_VERSION}`,
				startTime: launcherStartTime,
				inGame: false,
				buttons: [
					{ label: uiText("ui.4c8757fab2a21345"), url: "https://luxmc-r92.pages.dev" },
					{ label: uiText("ui.942f94d869e404ae"), url: "https://luxmc-r92.pages.dev/skins.html" }
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
        if (settings.value.closeToTray !== false || !appState.isGameRunning || settings.value.closeWarningOnGameRunning === false) return;
        let disposed = false;
        let pending = false;
        let unlisten: (() => void) | undefined;
        void import("@tauri-apps/api/window").then(async ({ getCurrentWindow }) => {
            if (disposed) return;
            const win = getCurrentWindow();
            const stop = await win.onCloseRequested(async (event) => {
                if (disposed) return;
                event.preventDefault();
                if (pending) return;
                pending = true;
                try {
                    const { confirm } = await import("@tauri-apps/plugin-dialog");
                    const approved = await confirm(uiText("ui.4e6287c3236d699f"), { title: uiText("ui.70a97095ffaef2d3"), kind: "warning", okLabel: uiText("statusBanner.dismiss"), cancelLabel: uiText("common.cancel") });
                    if (approved && !disposed) await win.destroy();
                } catch {
                    toast(uiText("ui.f4c1d1717580b3b5"), "error");
                } finally { pending = false; }
            });
            if (disposed) stop(); else unlisten = stop;
        }).catch(() => {});
        return () => { disposed = true; unlisten?.(); };
    });


    $effect(() => {
        const root = document.documentElement;
        root.style.setProperty("--interface-opacity", String(Math.max(0.05, Math.min(0.9, (settings.value.interfaceOpacity ?? 18) / 100))));
        root.style.setProperty("--wallpaper-dim", String(Math.max(0, Math.min(0.85, (settings.value.wallpaperDim ?? 35) / 100))));
        root.classList.toggle("density-compact", settings.value.density === "compact");
        root.classList.toggle("density-spacious", settings.value.density === "spacious");
        const disableBlur = settings.value.blur === false || appState.performanceMode;
        const disableAnim = settings.value.animations === false || appState.performanceMode;
        document.documentElement.classList.toggle("no-blur", disableBlur);
        document.documentElement.classList.toggle("no-anim", disableAnim);
        document.documentElement.classList.toggle("efficient-wallpaper", themeStore.background === "custom" && themeStore.customWallpaperType === "video" && settings.value.animatedWallpaperBlur !== true);
        return () => {
            document.documentElement.classList.remove("no-blur");
            document.documentElement.classList.remove("no-anim");
            document.documentElement.classList.remove("efficient-wallpaper");
        };
    });

	$effect(() => {
		const launching = appState.isLaunching;
		const running = appState.isGameRunning;
		const sessionId = logSessionId;
		if (launching || running || sessionId === null) return;
		const timer = setTimeout(() => {
			if (appState.isLaunching || appState.isGameRunning) return;
			closeLogSession(null, uiText("ui.1b474beb54bc0e45"), null);
		}, 2000);
		return () => clearTimeout(timer);
	});


</script>

<div class="fixed inset-0 z-0 transition-colors duration-300 pointer-events-none overflow-hidden" style={themeStore.currentBackgroundStyle}>
	{#if themeStore.background === "custom" && themeStore.customWallpaperUrl}
		{#if themeStore.customWallpaperType === "video"}
			<VideoWallpaper src={themeStore.customWallpaperUrl} />
		{:else}
			<img loading="lazy" decoding="async"
				src={resolveWallpaperImageUrl(themeStore.customWallpaperUrl)}
				alt={uiText("ui.1eed2949652b3b0e")}
				class="absolute inset-0 w-full h-full object-cover pointer-events-none"
			/>
		{/if}
		<div class="absolute inset-0 pointer-events-none" style={themeStore.theme === "light" ? "background: rgb(var(--bg) / max(.5, var(--wallpaper-dim, .35)));" : "background: rgb(var(--shadow-color) / var(--wallpaper-dim, .35));"}></div>
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
	<Cutscene onComplete={() => { showSplash = false; appState.showCutscene = false; }} />
{:else if !initialized}
	<div class="flex h-full w-full items-center justify-center bg-bg-overlay/70" in:fade={{ easing: quintOut, duration: 220 }}>
		<div class="flex flex-col items-center gap-4">
			<div class="h-10 w-10 border-4 border-t-brand-400 border-fg/10 rounded-full animate-spin"></div>
			<p class="text-sm font-medium text-fg shadow-black drop-shadow-md">{t("app.loading")}</p>
		</div>
	</div>
{:else if !account.value}
	<div use:pageMotion={page.url.pathname} class="relative isolate flex h-full w-full items-center justify-center bg-bg/90" in:fade={{ easing: quintOut, duration: 220 }}>
        <ShaderScenery background scene="forest" enabled={settings.value.animations !== false && !appState.performanceMode} />
		{@render children?.()}
	</div>
{:else}
	<div class="flex h-dvh min-h-0 w-full overflow-hidden" in:fade={{ easing: quintOut, duration: 150 }}>
		<Sidebar notificationCount={0} />
		<div class="flex h-full min-h-0 min-w-0 flex-1 flex-col relative z-10">
			<main use:scrollActivity data-scroll-root data-refined-ui={page.url.pathname !== '/skins'} class="min-h-0 flex-1 overflow-x-hidden overflow-y-auto {page.url.pathname === "/" ? "p-0" : "px-6 py-6"} custom-scrollbar custom-scrollbar-root relative">
				<div use:pageMotion={page.url.pathname} data-page-route={page.url.pathname} class="mx-auto {page.url.pathname === "/" ? "" : "max-w-[1600px]"} min-h-full flex flex-col w-full">
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

</ErrorBoundary>

<ModpackDownloadStatus />
<BedrockDownloadStatus />
<RoomPreparationStatus />

<PublicProfileModal />
