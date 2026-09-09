<script lang="ts">
	import "../app.css";
	import { Toaster } from "svelte-sonner";
	import { onMount } from "svelte";
	import { fade } from "svelte/transition";
	import Sidebar from "$lib/components/layout/Sidebar.svelte";
	import Topbar from "$lib/components/layout/Topbar.svelte";
	import Toasts from "$lib/components/ui/Toasts.svelte";
	import StatusBanner from "$lib/components/ui/StatusBanner.svelte";
	import Background3D from "$lib/components/visuals/Background3D.svelte";
	import UpdateModal from "$lib/components/ui/UpdateModal.svelte";
	import Cutscene from "$lib/components/visuals/Cutscene.svelte";
	import { bootstrapSettings, schedulePersist, startAutoPersist } from "$lib/stores/persistence.svelte";
	import { setToastInstance } from "$lib/stores/toasts.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { appInit, discordSetActivity, listenGameExit } from "$lib/api";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { themeStore } from "$lib/stores/theme.svelte";
	const { t } = useTranslation();
	let { children } = $props();
	let initialized = $state(false);
	let showSplash = $state(true);
	let toastsInstance = $state<any>(null);
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
			appState.devMode = init.devMode;
			if (init.account) {
				account.account = {
					id: init.account.id,
					username: init.account.username,
					uuid: init.account.uuid,
					minecraftToken: init.account.accessToken ?? "",
					expiresAt: init.account.expiresAt ? new Date(init.account.expiresAt).getTime() : 0,
				};
			} else if (typeof window !== "undefined") {
				const saved = localStorage.getItem("luxmc_current_account");
				if (saved) {
					try {
						account.account = JSON.parse(saved);
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
					details: "Luxmc Launcher",
					state: "No Menu Principal",
					largeText: "Luxmc Launcher (Linux)",
					largeImage: "luxmc"
				}).catch(() => {});
			}
		}).catch((e) => {
			clearTimeout(initTimer);
			toast(t("app.initFailed", { error: String(e) }), "error");
			initialized = true;
		});

		// Global game exit listener to accurately update play time and discord presence
		listenGameExit((event) => {
			gamingStats.onGameExit();
			if (settings.value.discordRpc !== false) {
				discordSetActivity({
					details: "Luxmc Launcher",
					state: "No Menu Principal",
					largeText: "Luxmc Launcher (Linux)",
					largeImage: "luxmc"
				}).catch(() => {});
			}
		}).catch(() => {});

		const stop = startAutoPersist();
		return stop;
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
		<!-- Innovative dark radial glow instead of full image -->
		<div class="absolute inset-0 opacity-40 transition-opacity duration-1000" style="background: radial-gradient(circle at 20% -20%, rgb(var(--brand-500)), transparent 60%); filter: blur(100px);"></div>
		<div class="absolute inset-0 opacity-20 transition-opacity duration-1000" style="background: radial-gradient(circle at 80% 120%, rgb(var(--brand-500)), transparent 60%); filter: blur(100px);"></div>
	{:else}
		<div class="absolute inset-0 bg-black/60"></div>
	{/if}
</div>
{#if !appState.performanceMode}
	<!-- Removing heavy Background3D to make it solid/clean like SKlauncher but keeping the performance mode toggle logic -->
{/if}

<Toaster position="bottom-right" theme="dark" richColors closeButton />
<Toasts bind:this={toastsInstance} />
<StatusBanner />

{#if showSplash || appState.showCutscene}
	<Cutscene onComplete={() => { showSplash = false; appState.showCutscene = false; }} />
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
				<div class="mx-auto max-w-[1600px] h-full flex flex-col">
					{@render children?.()}
				</div>
			</main>
		</div>
	</div>
{/if}
<UpdateModal />
