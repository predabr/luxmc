<script lang="ts">
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
			name: account.value?.username || "Você (Host)",
			role: "Líder da Sala",
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
			console.warn("Falha ao gerar host info:", e);
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
			toast("Link de convite copiado com sucesso!", "success");
			setTimeout(() => copiedLink = false, 2500);
		});
	}

	function copyDirect() {
		if (!hostInfo) return;
		navigator.clipboard.writeText(hostInfo.directAddress).then(() => {
			copiedDirect = true;
			toast("Endereço IP:Porta copiado!", "success");
			setTimeout(() => copiedDirect = false, 2500);
		});
	}

	async function handleJoin() {
		const raw = joinInput.trim();
		if (!raw) {
			toast("Informe um link de convite ou endereço IP:porta!", "error");
			return;
		}

		try {
			const invitation = raw.startsWith("luxmc://") ? new URL(raw).searchParams.get("invitation") : raw;
			if (!invitation?.startsWith("luxmc-world:") && !invitation?.startsWith("LUX-")) {
				toast("Para endereço IP:porta, use Multijogador → Conexão direta no Minecraft.", "info");
				return;
			}
			await joinTunnel(invitation);
			toast("Conectado. Abra o Minecraft em Multijogador para entrar no mundo LAN.", "success");
		} catch (error) {
			toast(`Não foi possível entrar na sala: ${String(error)}`, "error");
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
						<h1 class="text-2xl font-black text-fg tracking-tight">Jogar com amigos</h1>
						<span class="px-2 py-0.5 rounded-full text-[10px] font-black uppercase tracking-wider bg-emerald-500 text-brand-foreground shadow-md">
							P2P Host
						</span>
					</div>
					<p class="text-xs text-fg/50 mt-1">Abra o mundo para LAN, crie uma sala privada e envie o convite</p>
				</div>
			</div>

			<div class="flex items-center gap-3">
				<button
					type="button"
					class="px-5 py-2.5 rounded-full text-xs font-bold bg-bg-overlay hover:bg-fg/10 text-fg/80 hover:text-fg border border-fg/10 transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] flex items-center gap-2 cursor-pointer active:scale-[0.98]"
					onclick={refreshHost}
					disabled={loadingHost}
				>
					<RefreshCw class="w-3.5 h-3.5 {loadingHost ? 'animate-spin' : ''}" />
					<span>Atualizar Conexão</span>
				</button>
			</div>
		</div>
	</div>

	<MeshPanel />

	<div class="rounded-2xl border border-fg/10 bg-bg-subtle px-5 py-4 text-xs leading-relaxed text-fg-muted">
		<strong class="block text-fg">Endereço direto por IP · opção avançada</strong>
		As informações abaixo servem para conexão direta no Minecraft. Gerar um endereço não inicia uma sala P2P; para jogar pela internet sem configurar o roteador, use a sala acima.
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
						<h3 class="text-sm font-bold text-fg">Endereço direto do Minecraft</h3>
						<p class="text-xs text-fg/40">Consulta seu IP e porta LAN; a conexão pela internet pode exigir configurar o roteador</p>
					</div>
				</div>

				<div class="flex items-center gap-2">
					<span class="text-xs text-fg/50">Porta LAN:</span>
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
						<span class="text-[10px] font-bold text-fg/40 uppercase tracking-wider block">Link de endereço direto</span>
							<span class="font-mono text-xs text-emerald-400 font-semibold truncate block mt-0.5">
								{hostInfo.shareLink}
							</span>
						</div>
						<button
							type="button"
							class="px-4 py-2 rounded-full text-xs font-bold bg-fg/5 hover:bg-fg/10 text-fg border border-fg/10 transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] flex items-center gap-1.5 cursor-pointer shrink-0"
							onclick={copyShareLink}
						>
							{#if copiedLink}
								<Check class="w-3.5 h-3.5 text-emerald-400" />
								<span class="text-emerald-400">Copiado!</span>
							{:else}
								<Copy class="w-3.5 h-3.5" />
								<span>Copiar Link</span>
							{/if}
						</button>
					</div>

					<div class="bg-bg-subtle border border-fg/10 rounded-2xl p-4 flex items-center justify-between gap-4 shadow-inner">
						<div class="min-w-0 flex-1">
							<span class="text-[10px] font-bold text-fg/40 uppercase tracking-wider block">Endereço Direto (IP:Porta)</span>
							<span class="font-mono text-xs text-fg/80 truncate block mt-0.5">
								{hostInfo.directAddress}
							</span>
						</div>
						<button
							type="button"
							class="px-4 py-2 rounded-full text-xs font-bold bg-fg/5 hover:bg-fg/10 text-fg border border-fg/10 transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] flex items-center gap-1.5 cursor-pointer shrink-0"
							onclick={copyDirect}
						>
							{#if copiedDirect}
								<Check class="w-3.5 h-3.5 text-emerald-400" />
								<span class="text-emerald-400">Copiado!</span>
							{:else}
								<Copy class="w-3.5 h-3.5" />
								<span>Copiar IP</span>
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
					placeholder="Cole o convite da sala acima ou um IP:porta"
					class="flex-1 bg-bg-subtle border border-fg/10 rounded-2xl px-4 py-2.5 text-xs text-fg placeholder:text-fg/30 outline-none focus:border-brand-500 transition-colors"
				/>
				<Button
					variant="primary"
					onclick={handleJoin}
					class="shrink-0"
				>
					<Play class="w-4 h-4 mr-1.5 fill-current" />
					Entrar na Sala
				</Button>
			</div>
		</div>

		<!-- Status & Security Card -->
		<div class="bg-bg-elevated border border-fg/10 rounded-3xl p-6 shadow-md flex flex-col justify-between gap-4">
			<div>
				<div class="flex items-center gap-2.5 text-emerald-400">
					<ShieldCheck class="w-5 h-5" />
					<h4 class="text-xs font-black uppercase tracking-wider">Sobre o endereço direto</h4>
				</div>
				<p class="text-xs text-fg/50 mt-2 leading-relaxed">
					O endereço abaixo é só uma referência para entrada direta no Minecraft. Use a sala P2P acima para criar um convite de túnel.
				</p>
			</div>

			<div class="space-y-2 pt-4 border-t border-fg/5">
				<div class="flex items-center justify-between text-xs">
						<span class="text-fg/40">Status do Túnel:</span>
						<span class="font-bold text-fg/80">Veja a sala acima</span>
				</div>
				<div class="flex items-center justify-between text-xs">
					<span class="text-fg/40">Descoberta LAN:</span>
					<span class="font-bold text-fg/80">Multicast 224.0.2.60</span>
				</div>
				<div class="flex items-center justify-between text-xs">
					<span class="text-fg/40">Latência Média:</span>
						<span class="font-mono font-bold text-fg/80">Depende da conexão</span>
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
					Prévia de equipe
				</h3>
				<p class="text-xs text-fg/40 mt-0.5">Exemplo visual; os jogadores conectados aparecem na sala real acima</p>
			</div>
			<span class="text-xs font-bold text-emerald-400 bg-emerald-500/10 px-3 py-1 rounded-full border border-emerald-500/20">
				Demonstração
			</span>
		</div>

		<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
			{#each partyMembers as member}
				<div class="bg-bg-elevated border border-fg/5 hover:border-fg/15 rounded-2xl p-4 flex items-center justify-between transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] group shadow-sm">
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
							<span class="text-[9px] text-purple-400 font-semibold">Capa Ativa</span>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	</div>
</div>
