<script lang="ts">
    import FriendCard from "$lib/components/friends/FriendCard.svelte";
    import { button } from "$lib/components/ui/button";
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
		Gamepad2, Star
	} from "lucide-svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { account } from "$lib/stores/account.svelte";

	import MinecraftAvatar from "$lib/components/ui/MinecraftAvatar.svelte";
	import { friendsState } from "$lib/stores/friends.svelte";
	import type { Friend, SocialIdentity } from "$lib/api/social";
	import { joinWorld } from "$lib/utils/directJoin";
	import { p2pScanLanWorlds, upnpOpenPort, upnpClosePort } from "$lib/api/p2p";
	import { profiles } from "$lib/stores/profiles.svelte";

	const friends = $derived(friendsState.list);
    let localContacts = $state<string[]>([]);
    onMount(() => {
        try {
            const saved: unknown = JSON.parse(localStorage.getItem("luxmc_custom_friends_v4") || "[]");
            if (Array.isArray(saved)) localContacts = saved.flatMap((item: unknown) => item && typeof item === "object" && "username" in item && typeof item.username === "string" ? [item.username] : []);
        } catch { localContacts = []; }
    });
	let activeTab = $state<"all" | "online" | "pending" | "add" | "p2p">("all");
	let searchQuery = $state("");
	let newFriendUsername = $state("");
	let directJoinCode = $state("");
	let generatedHostCode = $state("");
	let hostingPort = $state<number | null>(null);
	let hostPort = $state(25565);
	let working = $state(false);
	let suggestions = $state<SocialIdentity[]>([]);
	let selectedFriend = $state<SocialIdentity | null>(null);
	let searchError = $state("");
	let joinProfileId = $state("");

	$effect(() => {
		const query = newFriendUsername.trim();
		let current = true;
		selectedFriend = null;
		suggestions = [];
		searchError = "";
		if (query.length < 3 || !friendsState.me) return;
		const timer = setTimeout(async () => {
			try { const users = await friendsState.search(query); if (current) suggestions = users; }
			catch (error) { if (current) searchError = String(error); }
		}, 300);
		return () => { current = false; clearTimeout(timer); };
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

	async function perform(action: () => Promise<void>) {
		if (working) return;
		working = true;
		try { await action(); } catch (error) { toast(String(error), "error"); }
		finally { working = false; }
	}

	function handleAddFriend() {
		return perform(async () => {
			const target = selectedFriend || (suggestions.length === 1 ? suggestions[0] : null);
			if (!target) throw new Error("Selecione o jogador nos resultados da busca.");
			await friendsState.action("invite", target.id);
			toast(`Convite enviado para ${target.username}.`, "success");
			newFriendUsername = "";
			activeTab = "pending";
		});
	}

	function acceptFriend(friend: Friend) { return perform(() => friendsState.action("accept", friend.id)); }
	function declineFriend(friend: Friend) { return perform(() => friendsState.action("remove", friend.id)); }
	function removeFriend(id: string, _name: string) { return perform(() => friendsState.action("remove", id)); }

	function copyDirectLink() {
		return perform(async () => {
			if (!generatedHostCode) throw new Error("Abra a sessão primeiro.");
			await navigator.clipboard.writeText(generatedHostCode);
			toast("Link copiado.", "success");
		});
	}

	function hostWorld() {
		return perform(async () => {
			if (hostingPort) {
				await upnpClosePort(hostingPort);
				await friendsState.shareWorld(null);
				hostingPort = null;
				generatedHostCode = "";
				return;
			}
			const worlds = await p2pScanLanWorlds();
			const port = worlds.length === 1 ? worlds[0].port : hostPort;
			if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error("Porta LAN inválida.");
			const result = await upnpOpenPort(port, 3600);
			if (!result.success || !result.externalIp) throw new Error(result.message);
			hostingPort = port;
			generatedHostCode = `luxmc://join/${result.externalIp}:${port}`;
			await friendsState.shareWorld({ host: result.externalIp, port });
			toast("Sessão aberta por até uma hora. Compartilhe o link com seus amigos.", "success");
		});
	}

	function handleDirectJoin() { return perform(() => joinWorld(directJoinCode, undefined, joinProfileId)); }
	function joinFriend(friend: Friend) {
		return perform(() => joinWorld(`${friend.serverIp}:${friend.serverPort || 25565}`, friend));
	}
</script>

<div class="h-full flex flex-col gap-6 select-none overflow-y-auto custom-scrollbar pb-10 max-w-7xl mx-auto w-full">
    <header class="relative flex flex-wrap items-end justify-between gap-5 py-4">
        <div><p class="page-eyebrow mb-3">Conexões que viram aventuras</p><h1 class="page-title">Melhor com amigos.</h1><p class="page-description">Encontre sua turma, compartilhe um mundo e entre no jogo.</p></div>
        <div class="flex gap-2"><button type="button" class={button({ variant: 'secondary' })} onclick={() => activeTab = 'p2p'}><Radio class="h-4 w-4" />Hospedar mundo</button><button type="button" class={button({ variant: 'primary' })} onclick={() => activeTab = 'add'}><UserPlus class="h-4 w-4" />Adicionar amigo</button></div>
    </header>

	<div class="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-border bg-bg-elevated/80 p-4 backdrop-blur-2xl">
		<p class="text-xs text-fg-muted">{friendsState.me ? `Seu código: ${friendsState.me.username}#${friendsState.me.id.slice(0, 8)}` : "Conecte seu perfil para buscar jogadores e receber convites."}</p>
		<button type="button" class="rounded-xl bg-brand-500 px-4 py-2 text-xs font-bold text-brand-foreground hover:bg-brand-400 disabled:opacity-50" onclick={() => friendsState.connect()} disabled={friendsState.busy || !account.value}>{friendsState.busy ? "Conectando..." : friendsState.me ? "Reconectar" : "Conectar rede social"}</button>
		{#if friendsState.error}<p class="w-full text-xs text-warning" role="status">{friendsState.error}</p>{/if}
	</div>
    {#if localContacts.length}
        <details class="rounded-xl border border-border bg-bg-elevated p-4 text-xs text-fg-muted">
            <summary class="cursor-pointer">Contatos salvos neste dispositivo ({localContacts.length})</summary>
            <p class="my-3">Seus contatos anteriores foram preservados. Procure cada jogador para enviar um convite na rede social.</p>
            <div class="flex flex-wrap gap-2">{#each localContacts as name}<button type="button" class="rounded-xl border border-border px-3 py-2 hover:bg-brand-500/10" onclick={() => { newFriendUsername = name; activeTab = "add"; }}>{name}</button>{/each}</div>
        </details>
    {/if}
    <div class="grid grid-cols-3 gap-3">
        {#each [{label:'Na sua turma',value:totalFriends,icon:Users}, {label:'Online agora',value:onlineCount,icon:Gamepad2}, {label:'Solicitações',value:pendingCount,icon:UserPlus}] as stat}
            <div class="surface-glass flex items-center gap-3 p-4"><stat.icon class="h-4 w-4 text-brand-400" /><div><p class="text-xl font-semibold text-fg">{stat.value}</p><p class="mt-1 text-[10px] text-fg-muted">{stat.label}</p></div></div>
        {/each}
    </div>

	<div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
		<div class="flex items-center gap-2 p-1.5 rounded-2xl bg-bg-elevated border border-fg/10 w-fit">
			<button
				type="button"
				onclick={() => activeTab = "all"}
				class="px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer {activeTab === 'all' ? 'bg-fg/15 text-fg shadow-sm' : 'text-fg/60 hover:text-fg'}"
			>
				Todos ({totalFriends})
			</button>
			<button
				type="button"
				onclick={() => activeTab = "online"}
				class="px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer {activeTab === 'online' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 shadow-sm' : 'text-fg/60 hover:text-fg'}"
			>
				Online ({onlineCount})
			</button>
			<button
				type="button"
				onclick={() => activeTab = "pending"}
				class="px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer relative {activeTab === 'pending' ? 'bg-amber-500/20 text-amber-300 border border-amber-500/30 shadow-sm' : 'text-fg/60 hover:text-fg'}"
			>
				Solicitações
				{#if pendingCount > 0}
					<span class="ml-1 px-1.5 py-0.2 rounded-full text-[10px] bg-amber-500 text-brand-foreground font-black">
						{pendingCount}
					</span>
				{/if}
			</button>
			<button
				type="button"
				onclick={() => activeTab = "p2p"}
				class="px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer {activeTab === 'p2p' ? 'bg-fg/15 text-fg shadow-sm' : 'text-fg/60 hover:text-fg'}"
			>
				Direct Join & Host
			</button>
		</div>

		{#if activeTab !== "add" && activeTab !== "p2p"}
			<div class="relative min-w-[260px]">
				<Search class="w-4 h-4 text-fg/40 absolute left-3.5 top-1/2 -translate-y-1/2 pointer-events-none" />
				<input
					type="text"
					bind:value={searchQuery}
					placeholder="Buscar por nome de usuário..."
					class="w-full bg-bg-elevated border border-fg/10 rounded-2xl pl-10 pr-4 py-2 text-xs text-fg placeholder:text-fg/40 outline-none focus:border-emerald-500/50 transition-colors"
				/>
			</div>
		{/if}
	</div>

	{#if activeTab === "pending"}
		<div class="space-y-4">
			<div class="flex items-center justify-between">
				<h2 class="text-sm font-bold text-fg uppercase tracking-wider">Solicitações de Amizade</h2>
				<span class="text-xs text-fg/40">{pendingCount} convite(s) pendentes</span>
			</div>

			{#if friends.filter(f => f.status === "pending").length === 0}
				<div class="p-12 text-center rounded-3xl bg-bg-elevated border border-fg/10 space-y-3">
					<div class="w-12 h-12 rounded-2xl bg-fg/5 border border-fg/10 flex items-center justify-center mx-auto text-fg/30">
						<UserCheck class="w-6 h-6" />
					</div>
					<h3 class="text-sm font-bold text-fg">Nenhum convite pendente</h3>
					<p class="text-xs text-fg/40 max-w-sm mx-auto">Quando outro jogador enviar um pedido de amizade para você, ele aparecerá aqui com as opções de aceitar ou recusar.</p>
				</div>
			{:else}
				<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
					{#each friends.filter(f => f.status === "pending") as req (req.id)}
						<div class="p-5 rounded-3xl bg-bg-elevated border border-fg/10 hover:border-amber-500/30 transition-all flex flex-col justify-between gap-4 shadow-xl">
							<div class="flex items-center gap-3.5">
								<MinecraftAvatar username={req.username} status={req.status} activity={req.activity} lastSeen={req.lastSeen} />
								<div class="min-w-0">
									<h3 class="text-sm font-extrabold text-fg truncate">{req.username}</h3>
									<p class="text-[11px] text-amber-400 font-medium mt-0.5">{req.incoming ? "Deseja ser seu amigo" : "Aguardando resposta"}</p>
								</div>
							</div>

							<div class="flex items-center gap-2 pt-2 border-t border-fg/5">
								<button
									type="button"
									onclick={() => acceptFriend(req)}
									disabled={!req.incoming || working}
									class="flex-1 flex items-center justify-center gap-1.5 py-2 px-3 rounded-xl bg-brand-500 hover:bg-brand-400 text-brand-foreground text-xs font-bold transition-all shadow-md active:scale-[0.98] cursor-pointer"
								>
									<Check class="w-3.5 h-3.5" />
									<span>{req.incoming ? "Aceitar Convite" : "Convite enviado"}</span>
								</button>
								<button
									type="button"
									onclick={() => declineFriend(req)}
									class="flex items-center justify-center py-2 px-3 rounded-xl bg-fg/5 hover:bg-rose-500/20 text-fg/70 hover:text-rose-400 border border-fg/10 hover:border-rose-500/30 text-xs font-bold transition-all active:scale-[0.98] cursor-pointer"
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
		<div class="p-8 rounded-3xl bg-bg-elevated border border-fg/10 shadow-xl max-w-2xl space-y-6">
			<div>
				<h2 class="text-lg font-black text-fg">Adicionar Novo Amigo</h2>
				<p class="text-xs text-fg/50 mt-1">Digite o nickname do Minecraft para adicionar o jogador à sua lista de conexões do Luxmc.</p>
			</div>

			<div class="flex items-center gap-4">
				<MinecraftAvatar username={selectedFriend?.username || newFriendUsername.trim() || "Steve"} class="h-16 w-16" />

				<div class="flex-1 space-y-2">
					<input
						type="text"
						bind:value={newFriendUsername}
						placeholder="Digite o nick ex: Spect3rBW"
						class="w-full bg-bg-subtle border border-fg/15 rounded-2xl px-4 py-3 text-sm text-fg font-medium placeholder:text-fg/30 outline-none focus:border-emerald-400 transition-colors"
						onkeydown={(e) => { if (e.key === "Enter") handleAddFriend(); }}
					/>
				</div>
			</div>

			<div class="space-y-2" aria-live="polite">
				{#each suggestions as suggestion (suggestion.id)}
					<button type="button" class="flex w-full items-center gap-3 rounded-xl border border-border p-3 text-left hover:bg-brand-500/10" onclick={() => selectedFriend = suggestion} aria-pressed={selectedFriend?.id === suggestion.id}>
						<MinecraftAvatar username={suggestion.username} class="h-8 w-8" />
						<span>{suggestion.username}<span class="text-fg-subtle">#{suggestion.id.slice(0, 8)}</span></span>
						{#if selectedFriend?.id === suggestion.id}<Check class="h-4 w-4 text-success" />{/if}
					</button>
				{/each}
				{#if searchError}<p class="text-xs text-danger">{searchError}</p>{/if}
				<p class="text-xs text-fg-muted">Confirme o código com seu amigo. O nickname e a skin não verificam a identidade Microsoft.</p>
			</div>

			<div class="flex items-center justify-end gap-3 pt-4 border-t border-fg/5">
				<button
					type="button"
					onclick={() => activeTab = "all"}
					class="px-5 py-2.5 rounded-xl bg-fg/5 hover:bg-fg/10 text-fg/70 hover:text-fg text-xs font-bold transition-all cursor-pointer"
				>
					Cancelar
				</button>
				<button
					type="button"
					onclick={handleAddFriend}
					disabled={working || (!selectedFriend && suggestions.length !== 1)}
					class="px-6 py-2.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 disabled:opacity-50 text-brand-foreground text-xs font-black transition-all shadow-lg shadow-emerald-500/20 cursor-pointer active:scale-[0.98]"
				>
					Adicionar Amigo
				</button>
			</div>
		</div>

	{:else if activeTab === "p2p"}
		<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
			<div class="p-6 rounded-3xl bg-bg-elevated border border-fg/10 space-y-5 shadow-xl">
				<div class="flex items-center gap-3">
					<div class="p-2.5 rounded-2xl bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">
						<Radio class="w-5 h-5" />
					</div>
					<div>
						<h3 class="text-base font-bold text-fg">Hospedar Sessão P2P (UPnP)</h3>
						<p class="text-xs text-fg/50">Abra seu mundo para amigos pela internet sem Hamachi ou Radmin</p>
					</div>
				</div>

				<div class="p-4 rounded-2xl bg-bg-overlay/40 border border-fg/10 space-y-3">
					<span class="text-[10px] font-bold uppercase text-fg/40 block">Código da sua Sessão</span>
					<div class="flex items-center justify-between gap-3">
						<span class="font-mono text-lg font-black text-emerald-400 tracking-wider">{generatedHostCode || "Sessão fechada"}</span>
						<button
							type="button"
							onclick={copyDirectLink}
							disabled={!generatedHostCode || working}
							class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-fg/10 hover:bg-fg/20 text-fg text-xs font-bold transition-all cursor-pointer active:scale-[0.98]"
						>
							<Copy class="w-3.5 h-3.5" />
							<span>Copiar Link</span>
						</button>
					</div>
				</div>

				<label class="block text-xs text-fg-muted" for="host-port">Porta exibida pelo Minecraft ao abrir para LAN</label>
				<input id="host-port" type="number" min="1" max="65535" bind:value={hostPort} class="rounded-xl border-border bg-bg-subtle text-fg" disabled={hostingPort !== null} />
				<button type="button" class="rounded-xl bg-brand-500 px-4 py-3 font-bold text-brand-foreground hover:bg-brand-400 disabled:opacity-50" onclick={hostWorld} disabled={working}>{hostingPort ? "Fechar sessão" : "Detectar mundo e abrir sessão"}</button>

				<p class="text-xs text-fg/50 leading-relaxed">
					Abra o jogo no Minecraft, pause e clique em <b>"Abrir para LAN"</b>. O Luxmc mapeará a porta automaticamente e gerará este link próprio para qualquer amigo entrar direto.
				</p>
			</div>

			<div class="p-6 rounded-3xl bg-bg-elevated border border-fg/10 space-y-5 shadow-xl">
				<div class="flex items-center gap-3">
					<div class="p-2.5 rounded-2xl bg-blue-500/20 text-blue-400 border border-blue-500/30">
						<Gamepad2 class="w-5 h-5" />
					</div>
					<div>
						<h3 class="text-base font-bold text-fg">Entrar Direto (Direct Join)</h3>
						<p class="text-xs text-fg/50">Conecte-se ao mundo de um amigo com 1 clique</p>
					</div>
				</div>

				<div class="space-y-3">
					<label for="join-profile" class="text-xs text-fg-muted">Instância para conectar</label>
					<select id="join-profile" bind:value={joinProfileId} class="w-full rounded-xl border-border bg-bg-subtle text-fg">
						<option value="">Instância ativa</option>
						{#each profiles.list as profile}<option value={profile.id}>{profile.name} · {profile.mcVersion}</option>{/each}
					</select>

					<input
						type="text"
						bind:value={directJoinCode}
						placeholder="luxmc://join/IP:porta ou IP:porta"
						class="w-full bg-bg-subtle border border-fg/15 rounded-2xl px-4 py-3 text-sm text-fg placeholder:text-fg/30 outline-none focus:border-blue-400 transition-colors font-mono"
					/>
					<button
						type="button"
						onclick={handleDirectJoin}
						disabled={working}
						class="w-full py-3 rounded-2xl bg-blue-600 hover:bg-blue-500 text-fg text-xs font-black transition-all shadow-lg shadow-blue-600/20 cursor-pointer active:scale-[0.98]"
					>
						Entrar no Mundo do Amigo
					</button>
				</div>
			</div>
		</div>

	{:else}
		{#if filteredFriends.length === 0}
			<div class="p-12 text-center rounded-3xl bg-bg-elevated border border-fg/10 space-y-3">
				<div class="w-12 h-12 rounded-2xl bg-fg/5 border border-fg/10 flex items-center justify-center mx-auto text-fg/30">
					<Users class="w-6 h-6" />
				</div>
				<h3 class="text-sm font-bold text-fg">Nenhum amigo encontrado</h3>
				<p class="text-xs text-fg/40 max-w-sm mx-auto">
					{searchQuery ? `Nenhum amigo corresponde a "${searchQuery}".` : "Você ainda não adicionou amigos nesta categoria."}
				</p>
			</div>
		{:else}
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {#each filteredFriends as friend (friend.id)}
                    <FriendCard {friend} favourite={friendsState.favourites.includes(friend.id)} busy={working} onJoin={() => joinFriend(friend)} onFavourite={() => friendsState.toggleFavourite(friend.id)} onRemove={() => removeFriend(friend.id, friend.username)} />
                {/each}
			</div>
		{/if}
	{/if}
</div>

