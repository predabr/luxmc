<script lang="ts">
	import { 
		Users, 
		UserPlus, 
		Radio, 
		Server, 
		Play, 
		Loader2, 
		Trash2, 
		Check, 
		Copy, 
		ExternalLink, 
		Gamepad2, 
		Circle
	} from "lucide-svelte";
	import { goto } from "$app/navigation";
	import { onMount } from "svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { launchGame } from "$lib/api";

	let connectingServer = $state<string | null>(null);
	let newFriendName = $state("");
	let showAddInput = $state(false);
	let directJoinCode = $state("");

	type Friend = {
		id: string;
		username: string;
		status: "in_game" | "online" | "offline";
		activity: string;
		serverIp?: string;
		serverPort?: number;
		lastSeen?: string;
	};

	let friends = $state<Friend[]>([]);

	const defaultInitialFriends: Friend[] = [
		{
			id: "f-1",
			username: "pedro_dev",
			status: "in_game",
			activity: "MushMC · Bedwars",
			serverIp: "jogar.mush.com.br",
			serverPort: 25565
		},
		{
			id: "f-2",
			username: "AlexGamer",
			status: "online",
			activity: "No Launcher"
		},
		{
			id: "f-3",
			username: "CraftMaster",
			status: "offline",
			activity: "Visto há 2h",
			lastSeen: "2h atrás"
		}
	];

	function loadFriends() {
		if (typeof window === "undefined") return;
		try {
			const saved = localStorage.getItem("luxmc_custom_friends_v2");
			if (saved) {
				friends = JSON.parse(saved);
			} else {
				friends = defaultInitialFriends;
				localStorage.setItem("luxmc_custom_friends_v2", JSON.stringify(friends));
			}
		} catch {
			friends = defaultInitialFriends;
		}
	}

	function saveFriends() {
		if (typeof window === "undefined") return;
		try {
			localStorage.setItem("luxmc_custom_friends_v2", JSON.stringify(friends));
		} catch {}
	}

	onMount(() => {
		loadFriends();
	});

	function addFriend() {
		const name = newFriendName.trim();
		if (!name) return;
		if (friends.some(f => f.username.toLowerCase() === name.toLowerCase())) {
			toast("Amigo já adicionado", "info");
			return;
		}
		const newF: Friend = {
			id: "f-" + Date.now(),
			username: name,
			status: "online",
			activity: "Online recentemente"
		};
		friends = [newF, ...friends];
		saveFriends();
		newFriendName = "";
		showAddInput = false;
		toast(`${name} adicionado à lista de amigos!`, "success");
	}

	function removeFriend(id: string, name: string) {
		friends = friends.filter(f => f.id !== id);
		saveFriends();
		toast(`${name} removido dos amigos.`, "info");
	}

	const onlineCount = $derived(friends.filter(f => f.status !== "offline").length);

	const quickJoinServers = [
		{ id: "mush", name: "MushMC", host: "jogar.mush.com.br", port: 25565, badge: "BR" },
		{ id: "hypixel", name: "Hypixel", host: "mc.hypixel.net", port: 25565, badge: "US" }
	];

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
			toast("Falha ao entrar no servidor: " + String(e), "error");
		} finally {
			connectingServer = null;
		}
	}

	function handleDirectP2PJoin() {
		const code = directJoinCode.trim();
		if (!code) {
			toast("Digite o código ou IP de convite do seu amigo", "warning");
			return;
		}
		if (code.includes(":")) {
			const [h, p] = code.split(":");
			handleQuickJoin(h, parseInt(p, 10) || 25565);
		} else {
			handleQuickJoin(code, 25565);
		}
	}
</script>

<aside class="w-[310px] shrink-0 h-full flex flex-col gap-4 select-none pb-4">

	<!-- Conexão Rápida Compacta -->
	<div class="space-y-2 shrink-0">
		<div class="flex items-center justify-between px-0.5">
			<h3 class="text-[11px] font-bold text-white/50 uppercase tracking-wider flex items-center gap-1.5">
				<Server class="w-3 h-3 text-emerald-400" /> Servidores Rápidos
			</h3>
			<span class="text-[9px] font-bold text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-full border border-emerald-500/20">
				1-Clique
			</span>
		</div>

		<div class="grid grid-cols-2 gap-2">
			{#each quickJoinServers as srv}
				<button
					type="button"
					class="flex items-center justify-between p-2 rounded-xl bg-[#141519] hover:bg-[#1c1e24] border border-white/5 hover:border-emerald-500/30 transition-all text-left group cursor-pointer disabled:opacity-50"
					disabled={connectingServer === srv.host}
					onclick={() => handleQuickJoin(srv.host, srv.port)}
				>
					<div class="flex items-center gap-2 min-w-0">
						<span class="text-[9px] font-mono font-bold px-1 py-0.5 rounded bg-black/40 border border-white/10 text-emerald-400">
							{srv.badge}
						</span>
						<span class="text-xs font-bold text-white group-hover:text-emerald-300 transition-colors truncate">
							{srv.name}
						</span>
					</div>
					{#if connectingServer === srv.host}
						<Loader2 class="w-3 h-3 animate-spin text-emerald-400 shrink-0" />
					{:else}
						<Play class="w-2.5 h-2.5 fill-current text-white/40 group-hover:text-emerald-400 transition-colors shrink-0" />
					{/if}
				</button>
			{/each}
		</div>
	</div>

	<!-- Direct Join P2P Input -->
	<div class="p-2.5 rounded-xl bg-[#141519] border border-white/5 space-y-2 shrink-0">
		<div class="flex items-center justify-between">
			<span class="text-[10px] font-bold text-white/60 flex items-center gap-1.5">
				<Radio class="w-3 h-3 text-emerald-400" /> Direct Join P2P / IP
			</span>
		</div>
		<div class="flex items-center gap-1.5">
			<input
				type="text"
				bind:value={directJoinCode}
				placeholder="IP:Porta ou código..."
				class="flex-1 bg-black/40 border border-white/10 rounded-lg px-2.5 py-1 text-xs text-white placeholder:text-white/25 outline-none focus:border-emerald-400 transition-colors font-mono"
				onkeydown={(e) => { if (e.key === "Enter") handleDirectP2PJoin(); }}
			/>
			<button
				type="button"
				onclick={handleDirectP2PJoin}
				class="px-2.5 py-1 rounded-lg bg-emerald-500 hover:bg-emerald-400 text-black font-bold text-xs transition-colors cursor-pointer shrink-0"
			>
				Entrar
			</button>
		</div>
	</div>

	<!-- Bloco de Amigos com Rolagem Própria -->
	<div class="flex-1 flex flex-col min-h-0 bg-[#111216] border border-white/5 rounded-2xl p-3 shadow-inner">
		
		<!-- Cabeçalho de Amigos -->
		<div class="flex items-center justify-between pb-2.5 border-b border-white/5 shrink-0">
			<div class="flex items-center gap-2">
				<Users class="w-3.5 h-3.5 text-emerald-400" />
				<h2 class="text-xs font-bold text-white uppercase tracking-wider">Amizades</h2>
				<span class="text-[10px] font-bold text-white/40 bg-white/5 px-2 py-0.5 rounded-full border border-white/5">
					{onlineCount} online
				</span>
			</div>

			<button
				type="button"
				onclick={() => showAddInput = !showAddInput}
				class="p-1 rounded-lg bg-white/5 hover:bg-emerald-500/20 text-white/60 hover:text-emerald-300 transition-all cursor-pointer"
				title="Adicionar Amigo"
			>
				<UserPlus class="w-3.5 h-3.5" />
			</button>
		</div>

		<!-- Input para Adicionar Amigo -->
		{#if showAddInput}
			<div class="pt-2 pb-1 shrink-0 flex items-center gap-1.5">
				<input
					type="text"
					bind:value={newFriendName}
					placeholder="Gamertag do amigo..."
					class="flex-1 bg-black/50 border border-white/10 rounded-lg px-2.5 py-1 text-xs text-white placeholder:text-white/25 outline-none focus:border-emerald-400 transition-colors"
					onkeydown={(e) => { if (e.key === "Enter") addFriend(); }}
				/>
				<button
					type="button"
					onclick={addFriend}
					class="px-2 py-1 rounded-lg bg-emerald-500 hover:bg-emerald-400 text-black font-bold text-xs cursor-pointer shrink-0"
				>
					OK
				</button>
			</div>
		{/if}

		<!-- Lista de Amigos com Scroll Real -->
		<div class="flex-1 overflow-y-auto custom-scrollbar space-y-1.5 pt-2 pr-1">
			{#if friends.length === 0}
				<div class="py-8 text-center space-y-2">
					<Users class="w-7 h-7 text-white/20 mx-auto" />
					<p class="text-xs text-white/40">Nenhum amigo adicionado.</p>
					<button
						type="button"
						onclick={() => showAddInput = true}
						class="text-[11px] text-emerald-400 hover:underline font-bold"
					>
						+ Adicionar Primeiro Amigo
					</button>
				</div>
			{:else}
				{#each friends as friend (friend.id)}
					{@const isOnline = friend.status === "online"}
					{@const isInGame = friend.status === "in_game"}
					
					<div class="flex items-center justify-between p-2 rounded-xl bg-[#15161b] hover:bg-[#1a1c22] border border-white/5 hover:border-white/10 transition-all group">
						<div class="flex items-center gap-2.5 min-w-0">
							<div class="relative w-8 h-8 rounded-lg overflow-hidden bg-black/50 border border-white/10 shrink-0">
								<img
									src={`https://mc-heads.net/avatar/${friend.username}/64`}
									alt={friend.username}
									class="w-full h-full object-cover"
									loading="lazy"
								/>
								<span
									class="absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 rounded-full border-2 border-[#15161b] {isInGame ? 'bg-purple-400 animate-pulse' : isOnline ? 'bg-emerald-400' : 'bg-neutral-500'}"
								></span>
							</div>

							<div class="min-w-0">
								<h4 class="text-xs font-bold text-white truncate leading-tight group-hover:text-emerald-300 transition-colors">
									{friend.username}
								</h4>
								<p class="text-[10px] text-white/40 truncate mt-0.5">
									{friend.activity}
								</p>
							</div>
						</div>

						<div class="flex items-center gap-1 shrink-0 ml-1">
							{#if friend.serverIp}
								<button
									type="button"
									class="p-1 rounded-lg bg-emerald-500/15 hover:bg-emerald-500 text-emerald-400 hover:text-black transition-colors cursor-pointer"
									title="Conectar ao mesmo servidor/mundo"
									onclick={() => handleQuickJoin(friend.serverIp!, friend.serverPort || 25565)}
								>
									<Play class="w-2.5 h-2.5 fill-current" />
								</button>
							{/if}
							<button
								type="button"
								class="opacity-0 group-hover:opacity-100 p-1 rounded-lg bg-white/5 hover:bg-red-500/20 text-white/40 hover:text-red-400 transition-all cursor-pointer"
								title="Remover Amigo"
								onclick={() => removeFriend(friend.id, friend.username)}
							>
								<Trash2 class="w-2.5 h-2.5" />
							</button>
						</div>
					</div>
				{/each}
			{/if}
		</div>

		<!-- Rodapé do Card de Amizades -->
		<div class="pt-2 mt-1 border-t border-white/5 flex items-center justify-between text-[10px] text-white/30 shrink-0">
			<span class="flex items-center gap-1">
				<Circle class="w-1.5 h-1.5 fill-emerald-400 text-emerald-400" /> Sistema P2P Ativo
			</span>
			<button
				type="button"
				onclick={() => goto("/friends")}
				class="hover:text-white transition-colors cursor-pointer font-semibold"
			>
				Abrir Chat →
			</button>
		</div>

	</div>

</aside>
