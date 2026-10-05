<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { onMount } from "svelte";
	import { fade, scale } from "svelte/transition";
	import { 
		Users, 
		Radio, 
		Share2, 
		Copy, 
		Check, 
		Wifi, 
		Sparkles, 
		ShieldCheck, 
		Globe, 
		RefreshCw,
		Play,
		Server,
		Layers
	} from "lucide-svelte";
	import { p2pGetHostLink, type HostLinkInfo } from "$lib/api/p2p";
	import { account } from "$lib/stores/account.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playSound } from "$lib/utils/sound";
	import Button from "$lib/components/ui/Button.svelte";
	import MeshPanel from "$lib/components/friends/MeshPanel.svelte";
	import { joinTunnel } from "$lib/api/tunnel";

	let hostInfo = $state<HostLinkInfo | null>(null);
	let loadingHost = $state(false);
	let hostPort = $state(25565);
	let copiedLink = $state(false);
	let copiedDirect = $state(false);
	let joinInput = $state("");

	type TeamMember = {
		id: string;
		name: string;
		role: string;
		avatar: string;
		ping: number;
		status: "online" | "in-game" | "ready";
		skinVariant: string;
		hasCape: boolean;
	};

	const partyMembers = $derived<TeamMember[]>([
		{
			id: account.value?.uuid || "host",
			name: account.value?.username || uiText("ui.19f3a7d6565fc863"),
			role: uiText("ui.7e8cacdd73b03f5b"),
			avatar: activeSkinStore.current.avatarUrl || "https://mc-heads.net/avatar/steve/100",
			ping: 0,
			status: "online",
			skinVariant: activeSkinStore.current.type,
			hasCape: Boolean(activeSkinStore.current.hasCape)
		},
		{
			id: "squad_2",
			name: "Alex_Builder",
			role: "Jogador Convidado",
			avatar: "https://mc-heads.net/avatar/alex/100",
			ping: 28,
			status: "ready",
			skinVariant: "alex",
			hasCape: true
		},
		{
			id: "squad_3",
			name: "EnderKnight",
			role: "Jogador Convidado",
			avatar: "https://mc-heads.net/avatar/MHF_Enderman/100",
			ping: 42,
			status: "ready",
			skinVariant: "steve",
			hasCape: false
		}
	]);

	async function refreshHost() {
		loadingHost = true;
		try {
			hostInfo = await p2pGetHostLink(hostPort);
		} catch (e) {
			console.warn(uiText("ui.86631a41a8aa68fb"), e);
		} finally {
			loadingHost = false;
		}
	}

	$effect(() => {
		const _p = hostPort;
		refreshHost();
	});

	function copyShareLink() {
		if (!hostInfo) return;
		navigator.clipboard.writeText(hostInfo.shareLink).then(() => {
			copiedLink = true;
			toast(uiText("ui.747cdb9983e94824"), "success");
			setTimeout(() => copiedLink = false, 2500);
		});
	}

	function copyDirect() {
		if (!hostInfo) return;
		navigator.clipboard.writeText(hostInfo.directAddress).then(() => {
			copiedDirect = true;
			toast(uiText("ui.ea8db2febaea62ea"), "success");
			setTimeout(() => copiedDirect = false, 2500);
		});
	}

	async function handleJoin() {
		const raw = joinInput.trim();
		if (!raw) {
			toast(uiText("ui.d0487a3761a7c7a3"), "error");
			return;
		}

		try {
			const invitation = raw.startsWith("luxmc://") ? new URL(raw).searchParams.get("invitation") : raw;
			if (!invitation?.startsWith("luxmc-world:") && !invitation?.startsWith("LUX-")) {
				toast(uiText("ui.23493da434a2283d"), "info");
				return;
			}
			await joinTunnel(invitation);
			toast(uiText("ui.55329aa25f36055f"), "success");
		} catch (error) {
			toast(uiText("ui.1b48018eeccbf815", {arg0: (String(error))}), "error");
		}
	}
</script>

<div class="p-6 md:p-8 space-y-8 max-w-7xl mx-auto">
	<!-- Hero Card -->
	<div class="relative overflow-hidden rounded-3xl bg-gradient-to-br from-bg-elevated via-bg-subtle to-bg border border-fg/10 p-7 shadow-2xl">
		<div class="absolute -right-10 -top-10 w-80 h-80 bg-[radial-gradient(circle_at_center,rgb(16_185_129/0.15),transparent_70%)] rounded-full pointer-events-none"></div>
		<div class="flex flex-col md:flex-row md:items-center justify-between gap-5 relative z-10">
			<div class="flex items-center gap-4">
				<div class="h-14 w-14 rounded-2xl bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 flex items-center justify-center shadow-lg shadow-emerald-500/10">
					<Users class="w-7 h-7" />
				</div>
				<div>
					<div class="flex items-center gap-2">
						<h1 class="text-2xl font-black text-fg tracking-tight">{uiText("ui.4cab082a07901725")}</h1>
						<span class="px-2 py-0.5 rounded-full text-[10px] font-black uppercase tracking-wider bg-emerald-500 text-brand-foreground shadow-md">
							{uiText("ui.cda824134ee7ed5e")}
						</span>
					</div>
					<p class="text-xs text-fg/50 mt-1">{uiText("ui.fbebe4027edb17bb")}</p>
				</div>
			</div>

			<div class="flex items-center gap-3">
				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2" })}
					onclick={refreshHost}
					disabled={loadingHost}
				>
					<RefreshCw class="w-3.5 h-3.5 {loadingHost ? 'animate-spin' : ''}" />
					<span>{uiText("ui.072e169baabac208")}</span>
				</button>
			</div>
		</div>
	</div>

	<MeshPanel />

	<div class="rounded-2xl border border-fg/10 bg-bg-subtle px-5 py-4 text-xs leading-relaxed text-fg-muted">
		<strong class="block text-fg">{uiText("ui.22675b735d524075")}</strong>
		{uiText("ui.ad53c0bd58247fbc")}
	</div>

	<!-- P2P Room Generation Card -->
	<div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
		<div class="lg:col-span-2 bg-bg-elevated border border-fg/10 rounded-3xl p-6 shadow-md space-y-5">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-xl bg-brand-500/15 text-brand-500 border border-brand-500/25 flex items-center justify-center">
						<Radio class="w-5 h-5" />
					</div>
					<div>
						<h3 class="text-sm font-bold text-fg">{uiText("ui.e124368c29024961")}</h3>
						<p class="text-xs text-fg/40">{uiText("ui.7219fd14d83b7dcf")}</p>
					</div>
				</div>

				<div class="flex items-center gap-2">
					<span class="text-xs text-fg/50">{uiText("ui.9e9ef900bf86a821")}</span>
					<input
						type="number"
						bind:value={hostPort}
						class="w-20 bg-bg-subtle border border-fg/10 rounded-xl px-2.5 py-1 text-xs text-fg font-mono text-center outline-none focus:border-brand-500"
						onchange={refreshHost}
					/>
				</div>
			</div>

			{#if hostInfo}
				<div class="space-y-3">
					<div class="bg-bg-subtle border border-fg/10 rounded-2xl p-4 flex items-center justify-between gap-4 shadow-inner">
						<div class="min-w-0 flex-1">
						<span class="text-[10px] font-bold text-fg/40 uppercase tracking-wider block">{uiText("ui.b37575889e526098")}</span>
							<span class="font-mono text-xs text-emerald-400 font-semibold truncate block mt-0.5">
								{hostInfo.shareLink}
							</span>
						</div>
						<button
							type="button"
							class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5 shrink-0" })}
							onclick={copyShareLink}
						>
							{#if copiedLink}
								<Check class="w-3.5 h-3.5 text-emerald-400" />
								<span class="text-emerald-400">{uiText("ui.a8fe0fc805d5fd50")}</span>
							{:else}
								<Copy class="w-3.5 h-3.5" />
								<span>{uiText("ui.b9cfc7837360c373")}</span>
							{/if}
						</button>
					</div>

					<div class="bg-bg-subtle border border-fg/10 rounded-2xl p-4 flex items-center justify-between gap-4 shadow-inner">
						<div class="min-w-0 flex-1">
							<span class="text-[10px] font-bold text-fg/40 uppercase tracking-wider block">{uiText("ui.4b2013b503df1f22")}</span>
							<span class="font-mono text-xs text-fg/80 truncate block mt-0.5">
								{hostInfo.directAddress}
							</span>
						</div>
						<button
							type="button"
							class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5 shrink-0" })}
							onclick={copyDirect}
						>
							{#if copiedDirect}
								<Check class="w-3.5 h-3.5 text-emerald-400" />
								<span class="text-emerald-400">{uiText("ui.a8fe0fc805d5fd50")}</span>
							{:else}
								<Copy class="w-3.5 h-3.5" />
								<span>{uiText("ui.50dbccc70acc7869")}</span>
							{/if}
						</button>
					</div>
				</div>
			{/if}

			<!-- Join Room -->
			<div class="pt-4 border-t border-fg/5 flex items-center gap-3">
				<input
					type="text"
					bind:value={joinInput}
					placeholder={uiText("ui.012834d3e4bff99a")}
					class="flex-1 bg-bg-subtle border border-fg/10 rounded-2xl px-4 py-2.5 text-xs text-fg placeholder:text-fg/30 outline-none focus:border-brand-500 transition-colors"
				/>
				<Button
					variant="primary"
					onclick={handleJoin}
					class="shrink-0"
				>
					<Play class="w-4 h-4 mr-1.5 fill-current" />
					{uiText("ui.ad207d12bdc1c355")}
				</Button>
			</div>
		</div>

		<!-- Status & Security Card -->
		<div class="bg-bg-elevated border border-fg/10 rounded-3xl p-6 shadow-md flex flex-col justify-between gap-4">
			<div>
				<div class="flex items-center gap-2.5 text-emerald-400">
					<ShieldCheck class="w-5 h-5" />
					<h4 class="text-xs font-black uppercase tracking-wider">{uiText("ui.3a2892c63bd3fc0d")}</h4>
				</div>
				<p class="text-xs text-fg/50 mt-2 leading-relaxed">
					{uiText("ui.45017772f9cbcdcb")}
				</p>
			</div>

			<div class="space-y-2 pt-4 border-t border-fg/5">
				<div class="flex items-center justify-between text-xs">
						<span class="text-fg/40">{uiText("ui.82ea13edaad37d9f")}</span>
						<span class="font-bold text-fg/80">{uiText("ui.a5c15b4d05994fa7")}</span>
				</div>
				<div class="flex items-center justify-between text-xs">
					<span class="text-fg/40">{uiText("ui.0913b2aef52380be")}</span>
					<span class="font-bold text-fg/80">{uiText("ui.aa594a6fd26ab113")}</span>
				</div>
				<div class="flex items-center justify-between text-xs">
					<span class="text-fg/40">{uiText("ui.9913d324e033bf70")}</span>
						<span class="font-mono font-bold text-fg/80">{uiText("ui.77addb4041653398")}</span>
				</div>
			</div>
		</div>
	</div>

	<!-- Squad Lineup & Skin Preview -->
	<div class="space-y-4">
		<div class="flex items-center justify-between">
			<div>
				<h3 class="text-sm font-bold text-fg flex items-center gap-2">
					<Users class="w-4 h-4 text-emerald-400" />
					{uiText("ui.8f50edd328881020")}
				</h3>
				<p class="text-xs text-fg/40 mt-0.5">{uiText("ui.c6a33d7aed2b7bc7")}</p>
			</div>
			<span class="text-xs font-bold text-emerald-400 bg-emerald-500/10 px-3 py-1 rounded-full border border-emerald-500/20">
				{uiText("ui.7d9aaa875a9e6516")}
			</span>
		</div>

		<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
			{#each partyMembers as member}
				<div class="bg-bg-elevated border border-fg/5 hover:border-fg/15 rounded-2xl p-4 flex items-center justify-between transition-[color,background-color,border-color,box-shadow,transform,opacity] group shadow-sm">
					<div class="flex items-center gap-3.5 min-w-0">
						<div class="h-12 w-12 rounded-xl bg-bg-overlay/40 border border-fg/10 overflow-hidden flex items-center justify-center shrink-0 shadow-md">
							<img loading="lazy" decoding="async" src={member.avatar} alt={member.name} class="w-full h-full object-cover [image-rendering:pixelated]" />
						</div>
						<div class="min-w-0">
							<div class="flex items-center gap-2">
								<h5 class="text-xs font-bold text-fg truncate">{member.name}</h5>
							</div>
							<div class="text-[10px] text-fg/40 mt-0.5 flex items-center gap-1.5">
								<span class="text-brand-500 font-semibold">{member.role}</span>
								<span>·</span>
								<span class="font-mono text-emerald-400">{member.ping === 0 ? 'Host' : `${member.ping}ms`}</span>
							</div>
						</div>
					</div>

					<div class="flex flex-col items-end gap-1 shrink-0">
						<span class="px-2 py-0.5 rounded-full text-[9px] font-black uppercase tracking-wider bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">
							{member.status}
						</span>
						{#if member.hasCape}
							<span class="text-[9px] text-purple-400 font-semibold">{uiText("ui.09e80776bd815caf")}</span>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	</div>
</div>
