<script lang="ts">
	import { onMount } from "svelte";
	import { 
		Users, 
		UserPlus, 
		UserCheck, 
		Radio, 
		Check, 
		X, 
		Search, 
		Copy, 
		Trash2, 
		Gamepad2
	} from "lucide-svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { account } from "$lib/stores/account.svelte";

	type Friend = {
		id: string;
		username: string;
		status: "in_game" | "online" | "offline" | "pending";
		activity?: string;
		serverIp?: string;
		addedAt?: string;
	};

	const defaultFriends: Friend[] = [];

	let friends = $state<Friend[]>([]);
	let activeTab = $state<"all" | "online" | "pending" | "add" | "p2p">("all");
	let searchQuery = $state("");
	let newFriendUsername = $state("");
	let directJoinCode = $state("");
	let generatedHostCode = $state("LUX-9412");

	function loadFriends() {
		if (typeof window === "undefined") return;
		try {
			const saved = localStorage.getItem("luxmc_custom_friends_v4");
			if (saved) {
				friends = JSON.parse(saved);
			} else {
				friends = [];
				localStorage.setItem("luxmc_custom_friends_v4", JSON.stringify(friends));
			}
		} catch {
			friends = [];
		}
	}

	function saveFriends() {
		if (typeof window === "undefined") return;
		try {
			localStorage.setItem("luxmc_custom_friends_v4", JSON.stringify(friends));
		} catch {}
	}

	onMount(() => {
		loadFriends();
	});

	const filteredFriends = $derived(
		friends.filter((f) => {
			const matchesSearch = !searchQuery || f.username.toLowerCase().includes(searchQuery.toLowerCase());
			if (!matchesSearch) return false;
			if (activeTab === "online") return f.status === "online" || f.status === "in_game";
			if (activeTab === "pending") return f.status === "pending";
			return f.status !== "pending";
		})
	);

	const onlineCount = $derived(friends.filter(f => f.status === "online" || f.status === "in_game").length);
	const pendingCount = $derived(friends.filter(f => f.status === "pending").length);
	const totalFriends = $derived(friends.filter(f => f.status !== "pending").length);

	function handleAddFriend() {
		const name = newFriendUsername.trim();
		if (!name) return;
		if (friends.some((f) => f.username.toLowerCase() === name.toLowerCase())) {
			toast("Este jogador já está na sua lista ou possui convite pendente.", "info");
			return;
		}
		const newF: Friend = {
			id: "f-" + Date.now(),
			username: name,
			status: "online",
			activity: "Adicionado agora",
			addedAt: new Date().toLocaleDateString("pt-BR")
		};
		friends = [newF, ...friends];
		saveFriends();
		newFriendUsername = "";
		toast(`${name} adicionado com sucesso aos seus amigos!`, "success");
		activeTab = "all";
	}

	function acceptFriend(friend: Friend) {
		friends = friends.map((f) => {
			if (f.id === friend.id) {
				return { ...f, status: "online", activity: "Online no Luxmc" };
			}
			return f;
		});
		saveFriends();
		toast(`Solicitação de amizade de ${friend.username} aceita!`, "success");
	}

	function declineFriend(friend: Friend) {
		friends = friends.filter((f) => f.id !== friend.id);
		saveFriends();
		toast(`Solicitação de ${friend.username} recusada.`, "info");
	}

	function removeFriend(id: string, name: string) {
		friends = friends.filter((f) => f.id !== id);
		saveFriends();
		toast(`${name} removido da lista de amigos.`, "info");
	}

	function copyDirectLink() {
		const link = `luxmc://join/${generatedHostCode}?user=${account.value?.username || "Player"}`;
		navigator.clipboard.writeText(link);
		toast("Link seguro P2P copiado para a área de transferência!", "success");
	}

	function handleDirectJoin() {
		if (!directJoinCode.trim()) {
			toast("Insira um código ou link de conexão válido", "warning");
			return;
		}
		toast(`Conectando à sessão ${directJoinCode.trim()}...`, "info");
	}
</script>

<div class="h-full flex flex-col gap-6 select-none overflow-y-auto custom-scrollbar pb-10 max-w-7xl mx-auto w-full">
	<!-- Header -->
	<div class="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-white/10 pb-5">
		<div>
			<div class="flex items-center gap-3">
				<div class="p-2.5 rounded-2xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400">
					<Users class="w-6 h-6" />
				</div>
				<div>
					<h1 class="text-2xl font-black text-white tracking-tight flex items-center gap-2.5">
						Amigos & Rede P2P
						<span class="text-xs font-bold px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">
							{onlineCount} Online
						</span>
					</h1>
					<p class="text-xs text-white/50 mt-0.5">Gerencie amigos, aceite solicitações e jogue juntos via P2P UPnP sem complicações</p>
				</div>
			</div>
		</div>

		<div class="flex items-center gap-3">
			<button
				type="button"
				onclick={() => activeTab = "add"}
				class="flex items-center gap-2 px-4 py-2.5 rounded-2xl bg-emerald-500 hover:bg-emerald-400 text-black text-xs font-bold shadow-lg shadow-emerald-500/20 transition-all cursor-pointer active:scale-95"
			>
				<UserPlus class="w-4 h-4" />
				<span>Adicionar Amigo</span>
			</button>

			<button
				type="button"
				onclick={() => activeTab = "p2p"}
				class="flex items-center gap-2 px-4 py-2.5 rounded-2xl bg-white/10 hover:bg-white/20 text-white text-xs font-bold border border-white/15 transition-all cursor-pointer shadow-md active:scale-95"
			>
				<Radio class="w-4 h-4 text-emerald-400" />
				<span>Hospedar P2P</span>
			</button>
		</div>
	</div>

	<!-- Navigation Tabs & Search -->
	<div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
		<div class="flex items-center gap-2 p-1.5 rounded-2xl bg-[#111218] border border-white/10 w-fit">
			<button
				type="button"
				onclick={() => activeTab = "all"}
				class="px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer {activeTab === 'all' ? 'bg-white/15 text-white shadow-sm' : 'text-white/60 hover:text-white'}"
			>
				Todos ({totalFriends})
			</button>
			<button
				type="button"
				onclick={() => activeTab = "online"}
				class="px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer {activeTab === 'online' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 shadow-sm' : 'text-white/60 hover:text-white'}"
			>
				Online ({onlineCount})
			</button>
			<button
				type="button"
				onclick={() => activeTab = "pending"}
				class="px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer relative {activeTab === 'pending' ? 'bg-amber-500/20 text-amber-300 border border-amber-500/30 shadow-sm' : 'text-white/60 hover:text-white'}"
			>
				Solicitações
				{#if pendingCount > 0}
					<span class="ml-1 px-1.5 py-0.2 rounded-full text-[10px] bg-amber-500 text-black font-black">
						{pendingCount}
					</span>
				{/if}
			</button>
			<button
				type="button"
				onclick={() => activeTab = "p2p"}
				class="px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer {activeTab === 'p2p' ? 'bg-white/15 text-white shadow-sm' : 'text-white/60 hover:text-white'}"
			>
				Direct Join & Host
			</button>
		</div>

		{#if activeTab !== "add" && activeTab !== "p2p"}
			<div class="relative min-w-[260px]">
				<Search class="w-4 h-4 text-white/40 absolute left-3.5 top-1/2 -translate-y-1/2 pointer-events-none" />
				<input
					type="text"
					bind:value={searchQuery}
					placeholder="Buscar por nome de usuário..."
					class="w-full bg-[#111218] border border-white/10 rounded-2xl pl-10 pr-4 py-2 text-xs text-white placeholder:text-white/40 outline-none focus:border-emerald-500/50 transition-colors"
				/>
			</div>
		{/if}
	</div>

	<!-- Main Content Area -->
	{#if activeTab === "pending"}
		<!-- Pending Requests Tab -->
		<div class="space-y-4">
			<div class="flex items-center justify-between">
				<h2 class="text-sm font-bold text-white uppercase tracking-wider">Convites de Amizade Recebidos</h2>
				<span class="text-xs text-white/40">{pendingCount} convite(s) aguardando sua resposta</span>
			</div>

			{#if friends.filter(f => f.status === "pending").length === 0}
				<div class="p-12 text-center rounded-3xl bg-[#111218] border border-white/10 space-y-3">
					<div class="w-12 h-12 rounded-2xl bg-white/5 border border-white/10 flex items-center justify-center mx-auto text-white/30">
						<UserCheck class="w-6 h-6" />
					</div>
					<h3 class="text-sm font-bold text-white">Nenhum convite pendente</h3>
					<p class="text-xs text-white/40 max-w-sm mx-auto">Quando outro jogador enviar um pedido de amizade para você, ele aparecerá aqui com as opções de aceitar ou recusar.</p>
				</div>
			{:else}
				<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
					{#each friends.filter(f => f.status === "pending") as req (req.id)}
						<div class="p-5 rounded-3xl bg-[#12141c] border border-white/10 hover:border-amber-500/30 transition-all flex flex-col justify-between gap-4 shadow-xl">
							<div class="flex items-center gap-3.5">
								<div class="relative w-12 h-12 rounded-2xl overflow-hidden bg-black/60 border border-white/10 shrink-0">
									<img
										src={`https://mc-heads.net/avatar/${req.username}/128`}
										alt={req.username}
										class="w-full h-full object-cover"
									/>
									<span class="absolute bottom-0 right-0 w-3 h-3 rounded-full bg-amber-400 border-2 border-black"></span>
								</div>
								<div class="min-w-0">
									<h3 class="text-sm font-extrabold text-white truncate">{req.username}</h3>
									<p class="text-[11px] text-amber-400 font-medium mt-0.5">Deseja ser seu amigo</p>
								</div>
							</div>

							<div class="flex items-center gap-2 pt-2 border-t border-white/5">
								<button
									type="button"
									onclick={() => acceptFriend(req)}
									class="flex-1 flex items-center justify-center gap-1.5 py-2 px-3 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-black text-xs font-bold transition-all shadow-md active:scale-95 cursor-pointer"
								>
									<Check class="w-3.5 h-3.5" />
									<span>Aceitar Convite</span>
								</button>
								<button
									type="button"
									onclick={() => declineFriend(req)}
									class="flex items-center justify-center py-2 px-3 rounded-xl bg-white/5 hover:bg-rose-500/20 text-white/70 hover:text-rose-400 border border-white/10 hover:border-rose-500/30 text-xs font-bold transition-all active:scale-95 cursor-pointer"
									title="Recusar"
								>
									<X class="w-4 h-4" />
								</button>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>

	{:else if activeTab === "add"}
		<!-- Add Friend Form -->
		<div class="p-8 rounded-3xl bg-[#111218] border border-white/10 shadow-xl max-w-2xl space-y-6">
			<div>
				<h2 class="text-lg font-black text-white">Adicionar Novo Amigo</h2>
				<p class="text-xs text-white/50 mt-1">Digite o nickname do Minecraft para adicionar o jogador à sua lista de conexões do Luxmc.</p>
			</div>

			<div class="flex items-center gap-4">
				<div class="w-16 h-16 rounded-2xl bg-black/60 border border-white/10 overflow-hidden shrink-0 flex items-center justify-center">
					{#if newFriendUsername.trim()}
						<img
							src={`https://mc-heads.net/avatar/${newFriendUsername.trim()}/128`}
							alt="Preview"
							class="w-full h-full object-cover"
						/>
					{:else}
						<UserPlus class="w-6 h-6 text-white/20" />
					{/if}
				</div>

				<div class="flex-1 space-y-2">
					<input
						type="text"
						bind:value={newFriendUsername}
						placeholder="Digite o nick ex: Spect3rBW"
						class="w-full bg-[#181a24] border border-white/15 rounded-2xl px-4 py-3 text-sm text-white font-medium placeholder:text-white/30 outline-none focus:border-emerald-400 transition-colors"
						onkeydown={(e) => { if (e.key === "Enter") handleAddFriend(); }}
					/>
				</div>
			</div>

			<div class="flex items-center justify-end gap-3 pt-4 border-t border-white/5">
				<button
					type="button"
					onclick={() => activeTab = "all"}
					class="px-5 py-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-white/70 hover:text-white text-xs font-bold transition-all cursor-pointer"
				>
					Cancelar
				</button>
				<button
					type="button"
					onclick={handleAddFriend}
					disabled={!newFriendUsername.trim()}
					class="px-6 py-2.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 disabled:opacity-50 text-black text-xs font-black transition-all shadow-lg shadow-emerald-500/20 cursor-pointer active:scale-95"
				>
					Adicionar Amigo
				</button>
			</div>
		</div>

	{:else if activeTab === "p2p"}
		<!-- P2P Direct Join & Host Hub -->
		<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
			<!-- Host P2P Card -->
			<div class="p-6 rounded-3xl bg-[#111218] border border-white/10 space-y-5 shadow-xl">
				<div class="flex items-center gap-3">
					<div class="p-2.5 rounded-2xl bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">
						<Radio class="w-5 h-5" />
					</div>
					<div>
						<h3 class="text-base font-bold text-white">Hospedar Sessão P2P (UPnP)</h3>
						<p class="text-xs text-white/50">Abra seu mundo para amigos pela internet sem Hamachi ou Radmin</p>
					</div>
				</div>

				<div class="p-4 rounded-2xl bg-black/40 border border-white/10 space-y-3">
					<span class="text-[10px] font-bold uppercase text-white/40 block">Código da sua Sessão</span>
					<div class="flex items-center justify-between gap-3">
						<span class="font-mono text-lg font-black text-emerald-400 tracking-wider">{generatedHostCode}</span>
						<button
							type="button"
							onclick={copyDirectLink}
							class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-white/10 hover:bg-white/20 text-white text-xs font-bold transition-all cursor-pointer active:scale-95"
						>
							<Copy class="w-3.5 h-3.5" />
							<span>Copiar Link</span>
						</button>
					</div>
				</div>

				<p class="text-xs text-white/50 leading-relaxed">
					Abra o jogo no Minecraft, pause e clique em <b>"Abrir para LAN"</b>. O Luxmc mapeará a porta automaticamente e gerará este link próprio para qualquer amigo entrar direto.
				</p>
			</div>

			<!-- Direct Join Card -->
			<div class="p-6 rounded-3xl bg-[#111218] border border-white/10 space-y-5 shadow-xl">
				<div class="flex items-center gap-3">
					<div class="p-2.5 rounded-2xl bg-blue-500/20 text-blue-400 border border-blue-500/30">
						<Gamepad2 class="w-5 h-5" />
					</div>
					<div>
						<h3 class="text-base font-bold text-white">Entrar Direto (Direct Join)</h3>
						<p class="text-xs text-white/50">Conecte-se ao mundo de um amigo com 1 clique</p>
					</div>
				</div>

				<div class="space-y-3">
					<input
						type="text"
						bind:value={directJoinCode}
						placeholder="Cole o código (ex: LUX-9412) ou link"
						class="w-full bg-[#181a24] border border-white/15 rounded-2xl px-4 py-3 text-sm text-white placeholder:text-white/30 outline-none focus:border-blue-400 transition-colors font-mono"
					/>
					<button
						type="button"
						onclick={handleDirectJoin}
						class="w-full py-3 rounded-2xl bg-blue-600 hover:bg-blue-500 text-white text-xs font-black transition-all shadow-lg shadow-blue-600/20 cursor-pointer active:scale-95"
					>
						Entrar no Mundo do Amigo
					</button>
				</div>
			</div>
		</div>

	{:else}
		<!-- Friend Cards Grid -->
		{#if filteredFriends.length === 0}
			<div class="p-12 text-center rounded-3xl bg-[#111218] border border-white/10 space-y-3">
				<div class="w-12 h-12 rounded-2xl bg-white/5 border border-white/10 flex items-center justify-center mx-auto text-white/30">
					<Users class="w-6 h-6" />
				</div>
				<h3 class="text-sm font-bold text-white">Nenhum amigo encontrado</h3>
				<p class="text-xs text-white/40 max-w-sm mx-auto">
					{searchQuery ? `Nenhum amigo corresponde a "${searchQuery}".` : "Você ainda não adicionou amigos nesta categoria."}
				</p>
			</div>
		{:else}
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
				{#each filteredFriends as friend (friend.id)}
					<div class="p-4 rounded-3xl bg-[#111218] border border-white/10 hover:border-white/20 transition-all flex items-center justify-between gap-4 shadow-md group">
						<div class="flex items-center gap-3.5 min-w-0">
							<div class="relative w-12 h-12 rounded-2xl overflow-hidden bg-black/60 border border-white/10 shrink-0">
								<img
									src={`https://mc-heads.net/avatar/${friend.username}/128`}
									alt={friend.username}
									class="w-full h-full object-cover {friend.status === 'offline' ? 'grayscale opacity-75' : ''}"
									loading="lazy"
								/>
								<span class="absolute bottom-0 right-0 w-3 h-3 rounded-full border-2 border-black {friend.status === 'in_game' ? 'bg-purple-400' : friend.status === 'online' ? 'bg-emerald-400' : 'bg-zinc-600'}"></span>
							</div>

							<div class="min-w-0">
								<h3 class="text-sm font-extrabold text-white truncate">{friend.username}</h3>
								<p class="text-[11px] text-white/50 truncate mt-0.5">{friend.activity || (friend.status === 'offline' ? 'Offline' : 'Online no Luxmc')}</p>
							</div>
						</div>

						<div class="flex items-center gap-1.5 shrink-0">
							<button
								type="button"
								class="opacity-0 group-hover:opacity-100 p-2 text-white/40 hover:text-rose-400 transition-opacity rounded-xl hover:bg-white/5 cursor-pointer"
								title="Remover Amigo"
								onclick={() => removeFriend(friend.id, friend.username)}
							>
								<Trash2 class="w-4 h-4" />
							</button>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	{/if}
</div>

