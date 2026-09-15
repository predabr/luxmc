<script lang="ts">
	import { Users, Server, Play, Loader2, Gamepad2, Circle, Clock, Plus, Radio, ArrowRight } from "lucide-svelte";
	import { goto } from "$app/navigation";
	import { toast } from "$lib/stores/toasts.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { launchGame, authDevLogin } from "$lib/api";

	let connectingServer = $state<string | null>(null);

	const quickJoinServers = [
		{ id: "mush", name: "MushMC", host: "jogar.mush.com.br", port: 25565, badge: "BR", desc: "Bed Wars · PvP · Duels" },
		{ id: "hypixel", name: "Hypixel", host: "mc.hypixel.net", port: 25565, badge: "US", desc: "SkyBlock · BedWars" }
	];

	type FriendStatus = "in_server" | "in_game" | "online" | "idle";

	type Friend = {
		id: string;
		name: string;
		avatarUrl: string;
		status: FriendStatus;
		activity: string;
		detail: string;
		serverAddress?: string;
		p2pCode?: string;
		lastSeen?: string;
	};

	const friends: Friend[] = [
		{
			id: "1",
			name: "pedro_dev",
			avatarUrl: "https://mc-heads.net/avatar/MHF_Steve/40",
			status: "in_server",
			activity: "MushMC Network",
			detail: "BedWars 4v4 · Sala #12",
			serverAddress: "jogar.mush.com.br"
		},
		{
			id: "2",
			name: "Lucas_Miner",
			avatarUrl: "https://mc-heads.net/avatar/MHF_Alex/40",
			status: "in_game",
			activity: "Better MC 1.20",
			detail: "Mundo LAN · Dia 34",
			p2pCode: "LUX-7842"
		},
		{
			id: "3",
			name: "Kiro_PvP",
			avatarUrl: "https://mc-heads.net/avatar/Notch/40",
			status: "in_server",
			activity: "Hypixel Network",
			detail: "SkyWars Ranked · 1.8.9",
			serverAddress: "mc.hypixel.net"
		},
		{
			id: "4",
			name: "AnaCraft",
			avatarUrl: "https://mc-heads.net/avatar/MHF_Herobrine/40",
			status: "online",
			activity: "No Launcher",
			detail: "Explorando shaders"
		},
		{
			id: "5",
			name: "GuiForge",
			avatarUrl: "https://mc-heads.net/avatar/MHF_PigZombie/40",
			status: "idle",
			activity: "Offline",
			detail: "",
			lastSeen: "há 2 horas"
		}
	];

	const statusConfig: Record<FriendStatus, { label: string; color: string; dot: string }> = {
		in_server: { label: "Em Servidor", color: "text-emerald-400", dot: "bg-emerald-400" },
		in_game:   { label: "Em Jogo",     color: "text-sky-400",     dot: "bg-sky-400" },
		online:    { label: "Online",       color: "text-blue-400",    dot: "bg-blue-400" },
		idle:      { label: "Offline",      color: "text-white/25",    dot: "bg-white/20" }
	};

	async function handleQuickJoin(host: string, port = 25565) {
		const inst = profiles.active || profiles.list[0];
		if (!inst) { toast("Selecione uma instância primeiro", "warning"); return; }
		if (!account.value) { toast("Faça login primeiro", "warning"); return; }
		connectingServer = host;
		try {
			await launchGame({
				versionId: inst.mcVersion,
				accountId: account.value.uuid,
				profileId: inst.id,
				enableVulkan: inst.useVulkan ?? false,
				skinUrl: activeSkinStore.current.skinUrl || account.value.skinUrl || null,
				skinVariant: activeSkinStore.current.type === "alex" ? "slim" : "classic",
				capeUrl: activeSkinStore.current.customCapeUrl || account.value.capeUrl || null,
				serverIp: host,
				serverPort: port
			});
			toast(`🎮 Conectando a ${host}...`, "success");
		} catch (e) {
			toast("Falha: " + String(e), "error");
		} finally {
			connectingServer = null;
		}
	}

	async function joinFriend(friend: Friend) {
		if (!friend.serverAddress && !friend.p2pCode) return;
		const inst = profiles.active || profiles.list[0];
		if (!inst) { toast("Selecione uma instância primeiro", "warning"); return; }
		let accountId = account.value?.uuid;
		if (!accountId) {
			const dev = await authDevLogin().catch(() => null);
			accountId = dev?.uuid ?? "";
		}
		toast(`Entrando na partida de ${friend.name}...`, "info");
		try {
			await launchGame({
				versionId: inst.mcVersion,
				accountId,
				profileId: inst.id,
				enableVulkan: inst.useVulkan ?? false,
				skinUrl: activeSkinStore.current.skinUrl || account.value?.skinUrl || null,
				skinVariant: activeSkinStore.current.type === "alex" ? "slim" : "classic",
				capeUrl: activeSkinStore.current.customCapeUrl || account.value?.capeUrl || null,
				serverIp: friend.serverAddress || null,
				serverPort: friend.serverAddress ? 25565 : null
			});
		} catch (e) {
			toast("Erro ao entrar: " + String(e), "error");
		}
	}

	const onlineFriends = $derived(friends.filter(f => f.status !== "idle"));
	const offlineFriends = $derived(friends.filter(f => f.status === "idle"));
</script>

<aside class="w-[300px] shrink-0 h-full flex flex-col gap-5 overflow-y-auto custom-scrollbar pr-1 pb-6 select-none">

	<div class="space-y-2.5">
		<div class="flex items-center justify-between">
			<h2 class="text-[11px] font-bold text-white/60 uppercase tracking-widest flex items-center gap-1.5">
				<Server class="w-3 h-3 text-brand-500" /> Conexão Rápida
			</h2>
			<span class="text-[9px] font-bold text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-full border border-emerald-500/20 flex items-center gap-1">
				<span class="w-1 h-1 rounded-full bg-emerald-400 animate-pulse"></span> 1-Clique
			</span>
		</div>

		{#each quickJoinServers as srv}
			<div class="flex items-center justify-between bg-bg-elevated hover:bg-bg-subtle p-2.5 rounded-xl border border-white/5 hover:border-white/10 transition-all group">
				<div class="flex items-center gap-2 min-w-0">
					<div class="w-7 h-7 rounded-lg bg-black/40 border border-white/10 flex items-center justify-center font-black text-[10px] text-brand-500 shrink-0">
						{srv.badge}
					</div>
					<div class="min-w-0">
						<h4 class="text-xs font-semibold text-white truncate">{srv.name}</h4>
						<p class="text-[10px] text-white/35 truncate">{srv.desc}</p>
					</div>
				</div>
				<button
					type="button"
					class="px-2.5 py-1 rounded-lg bg-brand-500/10 hover:bg-brand-500 text-brand-500 hover:text-black font-bold text-[10px] transition-all flex items-center gap-1 shrink-0 cursor-pointer disabled:opacity-40"
					disabled={connectingServer === srv.host}
					onclick={() => handleQuickJoin(srv.host, srv.port)}
				>
					{#if connectingServer === srv.host}
						<Loader2 class="w-2.5 h-2.5 animate-spin" />
					{:else}
						<Play class="w-2.5 h-2.5 fill-current" />
					{/if}
					<span>Entrar</span>
				</button>
			</div>
		{/each}
	</div>

	<div class="space-y-2.5">
		<div class="flex items-center justify-between">
			<h2 class="text-[11px] font-bold text-white/60 uppercase tracking-widest flex items-center gap-1.5">
				<Radio class="w-3 h-3 text-brand-500" /> Amigos
				<span class="text-[9px] font-mono text-emerald-400 bg-emerald-500/10 border border-emerald-500/20 px-1.5 py-0.5 rounded-full">
					{onlineFriends.length}
				</span>
			</h2>
			<button
				type="button"
				onclick={() => goto("/friends")}
				class="text-[10px] text-white/35 hover:text-brand-400 flex items-center gap-1 transition-colors cursor-pointer"
			>
				Ver todos <ArrowRight class="w-3 h-3" />
			</button>
		</div>

		<div class="flex flex-col gap-1.5">
			{#each onlineFriends as friend (friend.id)}
				{@const cfg = statusConfig[friend.status]}
				<div class="flex items-center gap-2.5 p-2.5 rounded-xl bg-bg-elevated hover:bg-bg-subtle border border-white/5 hover:border-white/10 transition-all group cursor-default">
					<div class="relative shrink-0">
						<img
							src={friend.avatarUrl}
							alt={friend.name}
							class="w-8 h-8 rounded-lg border border-white/10 [image-rendering:pixelated]"
						/>
						<span class="absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 rounded-full border-2 border-[#141518] {cfg.dot}"></span>
					</div>

					<div class="flex-1 min-w-0">
						<div class="flex items-center gap-1.5">
							<span class="text-xs font-semibold text-white truncate">{friend.name}</span>
						</div>
						<p class="text-[10px] {cfg.color} truncate font-medium">{friend.activity}</p>
						{#if friend.detail}
							<p class="text-[9px] text-white/30 truncate">{friend.detail}</p>
						{/if}
					</div>

					{#if friend.status === "in_server" || friend.status === "in_game"}
						<button
							type="button"
							onclick={() => joinFriend(friend)}
							class="shrink-0 w-7 h-7 rounded-lg bg-emerald-500/10 hover:bg-emerald-500 text-emerald-400 hover:text-black flex items-center justify-center transition-all cursor-pointer opacity-0 group-hover:opacity-100"
							title="Entrar na partida de {friend.name}"
						>
							<Play class="w-3 h-3 fill-current" />
						</button>
					{/if}
				</div>
			{/each}

			{#if offlineFriends.length > 0}
				<div class="pt-1">
					<span class="text-[10px] text-white/25 uppercase tracking-wider font-semibold">Offline</span>
				</div>
				{#each offlineFriends as friend (friend.id)}
					<div class="flex items-center gap-2.5 p-2 rounded-xl border border-transparent opacity-50">
						<div class="relative shrink-0">
							<img src={friend.avatarUrl} alt={friend.name} class="w-7 h-7 rounded-lg border border-white/5 [image-rendering:pixelated] grayscale" />
							<span class="absolute -bottom-0.5 -right-0.5 w-2 h-2 rounded-full border-2 border-[#141518] bg-white/20"></span>
						</div>
						<div class="flex-1 min-w-0">
							<span class="text-xs font-medium text-white/40 truncate block">{friend.name}</span>
							{#if friend.lastSeen}
								<span class="text-[9px] text-white/20 flex items-center gap-1">
									<Clock class="w-2.5 h-2.5" />{friend.lastSeen}
								</span>
							{/if}
						</div>
					</div>
				{/each}
			{/if}
		</div>

		<button
			type="button"
			onclick={() => goto("/friends")}
			class="w-full flex items-center justify-center gap-1.5 py-2 rounded-xl border border-white/5 hover:border-brand-500/30 text-white/30 hover:text-brand-400 text-[11px] font-medium transition-all cursor-pointer"
		>
			<Plus class="w-3 h-3" /> Adicionar Amigo & Gerenciar
		</button>
	</div>

	<div class="rounded-xl bg-bg-elevated border border-white/5 p-3 flex items-center justify-between hover:border-brand-500/30 transition-all cursor-pointer group" role="button" tabindex="0" onclick={() => goto("/friends")} onkeydown={(e) => { if (e.key === 'Enter') goto('/friends'); }}>
		<div class="flex items-center gap-2.5">
			<div class="w-7 h-7 rounded-lg bg-brand-500/10 border border-brand-500/20 flex items-center justify-center text-brand-500">
				<Gamepad2 class="w-3.5 h-3.5" />
			</div>
			<div>
				<h3 class="text-xs font-semibold text-white group-hover:text-brand-400 transition-colors">Luxmc P2P</h3>
				<p class="text-[10px] text-white/35">Direct Join sem Hamachi</p>
			</div>
		</div>
		<ArrowRight class="w-3.5 h-3.5 text-white/20 group-hover:text-brand-400 -rotate-45 transition-all" />
	</div>

</aside>
