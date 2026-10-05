<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { quintOut } from "svelte/easing";
	import { onMount, untrack } from "svelte";
	import { fade } from "svelte/transition";
	import {
		Server,
		Radio,
		RefreshCw,
		Play,
		ExternalLink,
		Check,
		Edit2,
		Users,
		Signal,
		Wifi,
		Zap
	} from "lucide-svelte";
	import { serverPing, type ServerStatus } from "$lib/api";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playSound } from "$lib/utils/sound";

	let { onQuickJoin }: { onQuickJoin?: (host: string, port: number) => void } = $props();

	const presetServers = $derived([
		{ name: "MushMC", host: "jogar.mush.com.br", port: 25565, banner: uiText("ui.8fa0c22a3830408d") },
		{ name: "Hypixel Network", host: "mc.hypixel.net", port: 25565, banner: uiText("home.globalServerBanner") },
		{ name: "Rede Sky", host: "jogar.redesky.com", port: 25565, banner: "⚔️ Minigames & PvP" },
		{ name: "Complex Gaming", host: "hub.mc-complex.com", port: 25565, banner: "🎮 Pixelmon & Survival" }
	]);

	let selectedServer = $state(untrack(() => presetServers[0]));
	let isCustomServer = $state(false);
	let customHost = $state("");
	let customPort = $state("25565");
	let showEditModal = $state(false);

	let status = $state<ServerStatus | null>(null);
	let isPinging = $state(false);
	let lastPingTime = $state<number>(0);
	let pingInterval: ReturnType<typeof setInterval> | null = null;

	onMount(() => {
		const saved = localStorage.getItem("luxmc_favorite_server");
		if (saved) {
			try {
				const parsed: unknown = JSON.parse(saved);
				const loaded = normalizeServer(parsed);
				if (loaded) {
					selectedServer = loaded;
					if (!presetServers.some((srv) => srv.host === loaded.host)) {
						isCustomServer = true;
						customHost = loaded.host;
						customPort = String(loaded.port);
					}
				}
			} catch {}
		}
		pingServer();
		pingInterval = setInterval(pingServer, 15000);
		return () => {
			if (pingInterval) clearInterval(pingInterval);
		};
	});

	function normalizeServer(value: unknown): { name: string; host: string; port: number; banner: string } | null {
		if (!value || typeof value !== "object") return null;
		const candidate = value as { name?: unknown; host?: unknown; port?: unknown; banner?: unknown };
		if (typeof candidate.host !== "string") return null;
		const host = candidate.host.trim();
		if (!host || /\s/.test(host) || host.includes("://")) return null;
		const port = Number(candidate.port);
		if (!Number.isInteger(port) || port < 1 || port > 65535) return null;
		return {
			name: typeof candidate.name === "string" && candidate.name.trim() ? candidate.name.trim() : host,
			host,
			port,
			banner: typeof candidate.banner === "string" ? candidate.banner : uiText("ui.2170dbb2a4bdf485")
		};
	}

	async function pingServer() {
		if (isPinging) return;
		isPinging = true;
		try {
			const res = await serverPing(selectedServer.host, selectedServer.port);
			status = res;
			lastPingTime = Date.now();
		} catch (e) {
			status = {
				online: false,
				version: uiText("ui.71d770406cf1cc87"),
				playersMax: 0,
				playersOnline: 0,
				motd: uiText("ui.b4993fa570f5c595"),
				latencyMs: 999
			};
		} finally {
			isPinging = false;
		}
	}

	function handleSaveServer() {
		if (isCustomServer) {
			const h = customHost.trim().replace(/^minecraft:\/\//i, "");
			const rawPort = customPort.trim() || "25565";
			const p = Number(rawPort);
			if (!h || /\s/.test(h) || h.includes("://")) {
				toast(uiText("ui.56b4cdadd243761c"), "error");
				return;
			}
			if (!Number.isInteger(p) || p < 1 || p > 65535) {
				toast(uiText("ui.c97bc476f9660df0"), "error");
				return;
			}
			selectedServer = {
				name: h,
				host: h,
				port: p,
				banner: uiText("ui.2170dbb2a4bdf485")
			};
		}
		const normalized = normalizeServer(selectedServer);
		if (!normalized) {
			toast(uiText("ui.5fa88f255abbb5c3"), "error");
			return;
		}
		selectedServer = normalized;
		localStorage.setItem("luxmc_favorite_server", JSON.stringify(selectedServer));
		showEditModal = false;
		toast(uiText("ui.91b9e64467b10a2a", {arg0: (selectedServer.name)}), "success");
		pingServer();
	}

	function cleanMotd(raw?: string): string {
		if (!raw) return uiText("ui.3e97792dfec4f9d7");
		return raw.replace(/§[0-9a-fk-or]/gi, "").trim();
	}

	function handleConnect() {
		playSound("click");
		if (onQuickJoin) {
			onQuickJoin(selectedServer.host, selectedServer.port);
		} else {
			toast(uiText("ui.a41c2bc9d451a5ab", {arg0: (selectedServer.host), arg1: (selectedServer.port)}), "info");
		}
	}
</script>

<div class="relative overflow-hidden rounded-3xl bg-gradient-to-br from-bg-elevated via-bg-elevated to-bg-elevated border border-fg/10 p-5 shadow-xl transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-300 hover:border-fg/20 group">
	<!-- Background subtle ambient glow based on status -->
	<div class="absolute -right-12 -top-12 w-44 h-44 rounded-full pointer-events-none transition-colors duration-500 {status?.online ? 'bg-[radial-gradient(circle_at_center,rgb(16_185_129/0.15),transparent_70%)]' : 'bg-[radial-gradient(circle_at_center,rgb(244_63_94/0.15),transparent_70%)]'}"></div>

	<div class="relative z-10 flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
		<!-- Left: Icon & Server Info -->
		<div class="flex items-center gap-3.5 min-w-0">
			<div class="relative w-12 h-12 rounded-2xl overflow-hidden bg-bg-overlay/40 border border-fg/15 flex items-center justify-center shrink-0 shadow-md">
				{#if status?.favicon}
					<img loading="lazy" decoding="async" src={status.favicon} alt={selectedServer.name} class="w-full h-full object-cover [image-rendering:pixelated]" />
				{:else}
					<div class="w-full h-full flex items-center justify-center bg-gradient-to-br from-emerald-600/30 to-teal-800/40 text-emerald-400 font-black">
						<Server class="w-6 h-6" />
					</div>
				{/if}

				<!-- Status Dot Indicator -->
				<span class="absolute bottom-1 right-1 flex h-3 w-3 items-center justify-center">
					{#if status?.online}
						<span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-400 border border-emerald-300 shadow-[0_0_8px_rgba(52,211,153,0.7)]"></span>
					{:else}
						<span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-rose-500 border border-rose-400"></span>
					{/if}
				</span>
			</div>

			<div class="min-w-0 space-y-0.5">
				<div class="flex items-center gap-2 flex-wrap">
					<h3 class="text-xs font-black text-fg truncate max-w-[200px] tracking-tight">{selectedServer.name}</h3>
					<span class="px-2 py-0.5 rounded-full text-[9px] font-extrabold uppercase tracking-wider {status?.online ? 'bg-emerald-500/15 text-emerald-300 border border-emerald-500/30' : 'bg-rose-500/15 text-rose-300 border border-rose-500/30'}">
						{status?.online ? 'Online' : 'Offline'}
					</span>

					{#if status?.online && status?.latencyMs != null}
						<span class="flex items-center gap-1 font-mono text-[10px] font-bold px-2 py-0.5 rounded-full bg-fg/5 border border-fg/10 {status.latencyMs < 80 ? 'text-emerald-400' : status.latencyMs < 150 ? 'text-amber-400' : 'text-rose-400'}">
							<Signal class="w-3 h-3" />
							{status.latencyMs}ms
						</span>
					{/if}
				</div>

				<div class="flex items-center gap-2 text-[11px] text-fg/50 font-mono">
					<span class="text-fg/70 font-semibold">{selectedServer.host}:{selectedServer.port}</span>
					{#if status?.online && status?.playersMax}
						<span>·</span>
						<span class="flex items-center gap-1 text-fg/80 font-bold">
							<Users class="w-3 h-3 text-emerald-400" />
							{status.playersOnline.toLocaleString()} / {status.playersMax.toLocaleString()}
						</span>
					{/if}
				</div>

				<p class="text-[10px] text-fg/40 truncate max-w-[360px] italic">
					{cleanMotd(status?.motd)}
				</p>
			</div>
		</div>

		<!-- Right: Action Buttons -->
		<div class="flex items-center gap-2.5 w-full md:w-auto shrink-0 justify-end pt-2 md:pt-0 border-t md:border-t-0 border-fg/5">
			<button
				type="button"
				class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
				onclick={pingServer}
				title={uiText("ui.363617910a89f3ef")}
				disabled={isPinging}
			>
				<RefreshCw class="w-4 h-4 {isPinging ? 'animate-spin text-brand-400' : ''}" />
			</button>

			<button
				type="button"
				class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })}
				onclick={() => showEditModal = true}
				title={uiText("ui.997bccf129529f0e")}
			>
				<Edit2 class="w-3.5 h-3.5 text-brand-400" />
				<span class="hidden sm:inline">{uiText("ui.a5575a1a9fa7978f")}</span>
			</button>

			<button
				type="button"
				class="px-5 py-2.5 rounded-2xl font-black text-xs uppercase tracking-wider flex items-center gap-2 transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer shadow-lg active:scale-[0.98] {status?.online ? 'bg-gradient-to-r from-emerald-500 to-teal-500 hover:from-emerald-400 hover:to-teal-400 text-brand-foreground shadow-emerald-500/20' : 'bg-fg/10 text-fg/40 cursor-not-allowed'}"
				onclick={handleConnect}
				disabled={!status?.online}
				title={status?.online ? uiText("ui.a716da6b2932de5e") : uiText("ui.4685a6fc5a37f9c5")}
			>
				<Play class="w-3.5 h-3.5 fill-current" />
				<span>{uiText("home.connect")}</span>
			</button>
		</div>
	</div>
</div>

<!-- Modal: Selecionar ou Inserir Servidor -->
{#if showEditModal}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/80 backdrop-blur-sm" in:fade={{ easing: quintOut, duration: 220 }}>
		<div class="w-full max-w-md rounded-3xl bg-bg-elevated border border-fg/15 p-6 shadow-2xl space-y-5">
			<div class="flex items-center justify-between border-b border-fg/10 pb-4">
				<div class="flex items-center gap-2.5">
					<div class="w-8 h-8 rounded-xl bg-brand-500/10 border border-brand-500/30 flex items-center justify-center text-brand-400">
						<Server class="w-4 h-4" />
					</div>
					<div>
						<h3 class="text-sm font-black text-fg">{uiText("ui.deeefc89c2d49a2b")}</h3>
						<p class="text-[11px] text-fg/50">{uiText("ui.571f8dd958b62695")}</p>
					</div>
				</div>
				<button
					type="button"
					class={launcherButton({ variant: "ghost", size: "sm", class: "" })}
					onclick={() => showEditModal = false}
				>
					✕
				</button>
			</div>

			<!-- Presets -->
			<div class="space-y-2">
				<span class="text-[11px] font-bold text-fg/60 uppercase tracking-wider block">{uiText("ui.a01363c13ca70960")}</span>
				<div class="grid grid-cols-2 gap-2">
					{#each presetServers as srv (srv.name)}
						<button
							type="button"
							class="p-3 rounded-2xl border text-left transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer flex flex-col justify-between {selectedServer.host === srv.host && !isCustomServer ? 'bg-brand-500/15 border-brand-500/40 shadow-sm' : 'bg-bg-subtle hover:bg-bg-subtle border-fg/5'}"
							onclick={() => { selectedServer = srv; isCustomServer = false; }}
						>
							<div class="flex items-center justify-between">
								<span class="text-xs font-black text-fg">{srv.name}</span>
								{#if selectedServer.host === srv.host && !isCustomServer}
									<Check class="w-3.5 h-3.5 text-brand-400" />
								{/if}
							</div>
							<span class="text-[10px] text-fg/40 font-mono mt-1">{srv.host}</span>
						</button>
					{/each}
				</div>
			</div>

			<!-- Custom IP Option -->
			<div class="space-y-2 pt-2 border-t border-fg/5">
				<label class="flex items-center gap-2 cursor-pointer">
					<input type="checkbox" bind:checked={isCustomServer} class="accent-brand-500 rounded" />
					<span class="text-xs font-bold text-fg">{uiText("ui.408b86eea08552f2")}</span>
				</label>

				{#if isCustomServer}
					<div class="flex gap-2 pt-1" in:fade={{ easing: quintOut, duration: 220 }}>
						<input
							type="text"
							bind:value={customHost}
							placeholder={uiText("ui.1448383d6aea5a7e")}
							class="flex-1 px-4 py-2.5 rounded-xl bg-bg-overlay/40 border border-fg/10 text-xs text-fg focus:outline-none focus:border-brand-500 font-mono"
						/>
						<input
							type="text"
							bind:value={customPort}
							placeholder="25565"
							class="w-20 px-3 py-2.5 rounded-xl bg-bg-overlay/40 border border-fg/10 text-xs text-fg focus:outline-none focus:border-brand-500 font-mono text-center"
						/>
					</div>
				{/if}
			</div>

			<!-- Actions -->
			<div class="flex items-center justify-end gap-2.5 pt-2">
				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "sm", class: "" })}
					onclick={() => showEditModal = false}
				>
					{uiText("common.cancel")}
				</button>
				<button
					type="button"
					class={launcherButton({ variant: "primary", size: "sm", class: "" })}
					onclick={handleSaveServer}
				>
					{uiText("ui.2bf7003afaa2e5da")}
				</button>
			</div>
		</div>
	</div>
{/if}
