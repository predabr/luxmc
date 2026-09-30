<script lang="ts">
    import MeshPanel from "$lib/components/friends/MeshPanel.svelte";
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
			Gamepad2,
			CloudOff,
			ShieldCheck,
			Sparkles,
			Wifi
		} from "lucide-svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { account } from "$lib/stores/account.svelte";

	import MinecraftAvatar from "$lib/components/ui/MinecraftAvatar.svelte";
	import { friendsState } from "$lib/stores/friends.svelte";
	import type { Friend, SocialIdentity } from "$lib/api/social";
	import { p2pScanLanWorlds } from "$lib/api/p2p";
	import { joinWorld } from "$lib/utils/directJoin";

	const friends = $derived(friendsState.list);
	let localContacts = $state<string[]>([]);
	let lanWorldCount = $state(0);
	onMount(() => {
		try {
			const saved: unknown = JSON.parse(localStorage.getItem("luxmc_custom_friends_v4") || "[]");
			if (Array.isArray(saved)) localContacts = saved.flatMap((item: unknown) => item && typeof item === "object" && "username" in item && typeof item.username === "string" ? [item.username] : []);
		} catch { localContacts = []; }
		let disposed = false;
		let scanTimer: ReturnType<typeof setTimeout>;
		const scanLan = async () => {
			try {
				const worlds = await p2pScanLanWorlds();
				if (!disposed) lanWorldCount = worlds.length;
			} catch {
				if (!disposed) lanWorldCount = 0;
			}
			if (!disposed) scanTimer = setTimeout(scanLan, 10000);
		};
		void scanLan();
		return () => {
			disposed = true;
			clearTimeout(scanTimer);
		};
	});
	let activeTab = $state<"all" | "online" | "pending" | "add" | "p2p">("all");
	let searchQuery = $state("");
	let newFriendUsername = $state("");




	let working = $state(false);
	let suggestions = $state<SocialIdentity[]>([]);
	let selectedFriend = $state<SocialIdentity | null>(null);
	let searchError = $state("");
	let offlineRedirected = $state(false);
	const cloudOffline = $derived(Boolean(friendsState.error));

	$effect(() => {
		if (cloudOffline && !offlineRedirected) {
			offlineRedirected = true;
			activeTab = "p2p";
		} else if (!cloudOffline) {
			offlineRedirected = false;
		}
	});


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
			const query = newFriendUsername.trim();
			if (!query) throw new Error("Digite o nickname do jogador.");

			let target = selectedFriend || (suggestions.length > 0 ? suggestions[0] : null);
			if (!target && friendsState.me) {
				try {
					const found = await friendsState.search(query);
					if (found.length > 0) target = found[0];
				} catch {}
			}

			if (target) {
				await friendsState.action("invite", target.id);
				toast(`Convite enviado para ${target.username}!`, "success");
            } else {
                throw new Error("Jogador não encontrado na rede. Peça que ele abra o Luxmc e confira o nickname.");
            }

			newFriendUsername = "";
			selectedFriend = null;
			activeTab = "all";
		});
	}

	function acceptFriend(friend: Friend) { return perform(() => friendsState.action("accept", friend.id)); }
	function declineFriend(friend: Friend) { return perform(() => friendsState.action("remove", friend.id)); }
	function removeFriend(id: string, _name: string) { return perform(() => friendsState.action("remove", id)); }

	function joinFriend(friend: Friend) {
		return perform(() => joinWorld(`${friend.serverIp}:${friend.serverPort || 25565}`, friend));
	}
	function inviteFriend(_friend: Friend) { activeTab = "p2p"; }
</script>

<div class="h-full flex flex-col gap-6 select-none overflow-y-auto custom-scrollbar pb-10 max-w-7xl mx-auto w-full">
	<header class="surface-glass relative overflow-hidden border-brand-500/20 p-6 sm:p-8">
		<div class="pointer-events-none absolute inset-0 bg-gradient-to-r from-brand-500/15 via-transparent to-success/10"></div>
		<div class="friends-grid pointer-events-none absolute inset-0 opacity-30"></div>
		<div class="relative flex flex-wrap items-end justify-between gap-6">
			<div>
				<p class="page-eyebrow mb-3">Conexões que viram aventuras</p>
				<h1 class="page-title">Melhor com amigos.</h1>
				<p class="page-description max-w-2xl">Salas diretas, descoberta LAN e sua turma em um só lugar.</p>
				<div class="mt-4 inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-[10px] font-black uppercase tracking-widest {cloudOffline ? 'border-success/30 bg-success/10 text-success' : 'border-brand-500/30 bg-brand-500/10 text-brand-300'}">
					{#if cloudOffline}<CloudOff class="h-3.5 w-3.5" /> Modo Local / P2P Ativo{:else}<ShieldCheck class="h-3.5 w-3.5" /> Rede social conectada{/if}
				</div>
			</div>
			<div class="flex flex-wrap gap-2">
				<button type="button" class={button({ variant: 'secondary' })} onclick={() => activeTab = 'p2p'}><Radio class="h-4 w-4" />Sala P2P</button>
				<button type="button" class={button({ variant: 'primary' })} onclick={() => activeTab = 'add'} disabled={cloudOffline}><UserPlus class="h-4 w-4" />Adicionar amigo</button>
			</div>
		</div>
	</header>

		{#if activeTab !== "p2p"}
			<div class="surface-glass flex flex-wrap items-center justify-between gap-3 border-brand-500/15 p-4">
				<div class="flex items-center gap-3">
					<p class="text-xs text-fg-muted">{friendsState.me ? `Seu código: ` : "Conecte seu perfil para buscar jogadores e receber convites."}</p>
					{#if friendsState.me}
						<span class="font-mono text-xs font-bold text-fg bg-bg-subtle px-2 py-0.5 rounded-lg border border-fg/10">{friendsState.me.username}#{friendsState.me.id.slice(0, 8)}</span>
						<button
							type="button"
							title="Copiar código para compartilhar com amigos"
							class="px-2.5 py-1 rounded-lg bg-fg/10 hover:bg-fg/20 text-fg text-[11px] font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] cursor-pointer flex items-center gap-1 active:scale-95"
							onclick={() => {
								navigator.clipboard.writeText(`${friendsState.me?.username}#${friendsState.me?.id.slice(0, 8)}`);
								toast("Código de amigo copiado!", "success");
							}}
						>
							<Copy class="w-3 h-3" />
							<span>Copiar</span>
						</button>
					{/if}
				</div>
				<button type="button" class="rounded-xl bg-brand-500 px-4 py-2 text-xs font-bold text-brand-foreground hover:bg-brand-400 disabled:opacity-50 cursor-pointer" onclick={() => friendsState.connect()} disabled={friendsState.busy || !account.value}>{friendsState.busy ? "Conectando..." : friendsState.me ? "Reconectar" : "Conectar rede social"}</button>
				{#if cloudOffline}<div class="flex w-full items-center gap-2 rounded-xl border border-success/20 bg-success/10 px-3 py-2 text-xs text-success" role="status"><Wifi class="h-4 w-4" /><span>O serviço de nuvem está offline. Suas salas P2P e mundos LAN continuam disponíveis.</span><button type="button" class="ml-auto font-bold underline" onclick={() => activeTab = 'p2p'}>Abrir P2P</button></div>{/if}
			</div>
			{#if localContacts.length}
				<details class="rounded-xl border border-border bg-bg-elevated p-4 text-xs text-fg-muted">
					<summary class="cursor-pointer">Contatos salvos neste dispositivo ({localContacts.length})</summary>
					<p class="my-3">Seus contatos anteriores foram preservados. Procure cada jogador para enviar um convite na rede social.</p>
					<div class="flex flex-wrap gap-2">{#each localContacts as name}<button type="button" class="rounded-xl border border-border px-3 py-2 hover:bg-brand-500/10" onclick={() => { newFriendUsername = name; activeTab = "add"; }}>{name}</button>{/each}</div>
				</details>
			{/if}
		{/if}
	<div class="grid gap-3 sm:grid-cols-3">
		{#each [{label:'Na sua turma',value:totalFriends,icon:Users,tone:'text-brand-400 bg-brand-500/10 border-brand-500/20'}, {label:'Online agora',value:onlineCount,icon:Gamepad2,tone:'text-success bg-success/10 border-success/20'}, {label:'Mundos LAN detectados',value:lanWorldCount,icon:Wifi,tone:'text-info bg-info/10 border-info/20'}] as stat}
			<div class="surface-glass group flex items-center gap-4 p-4 hover:border-brand-500/25"><div class="grid h-11 w-11 place-items-center rounded-2xl border {stat.tone}"><stat.icon class="h-5 w-5" /></div><div><p class="text-2xl font-black text-fg">{stat.value}</p><p class="mt-0.5 text-[10px] font-bold uppercase tracking-wider text-fg-muted">{stat.label}</p></div></div>
		{/each}
	</div>
	<div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
		<div class="flex items-center gap-2 p-1.5 rounded-2xl bg-bg-elevated border border-fg/10 w-fit">
			<button
				type="button"
				onclick={() => activeTab = "all"}
				class="px-4 py-2 rounded-xl text-xs font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] cursor-pointer {activeTab === 'all' ? 'bg-fg/15 text-fg shadow-sm' : 'text-fg/60 hover:text-fg'}"
			>
				Todos ({totalFriends})
			</button>
			<button
				type="button"
				onclick={() => activeTab = "online"}
				class="px-4 py-2 rounded-xl text-xs font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] cursor-pointer {activeTab === 'online' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 shadow-sm' : 'text-fg/60 hover:text-fg'}"
			>
				Online ({onlineCount})
			</button>
			<button
				type="button"
				onclick={() => activeTab = "pending"}
				class="px-4 py-2 rounded-xl text-xs font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] cursor-pointer relative {activeTab === 'pending' ? 'bg-amber-500/20 text-amber-300 border border-amber-500/30 shadow-sm' : 'text-fg/60 hover:text-fg'}"
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
				class="px-4 py-2 rounded-xl text-xs font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] cursor-pointer {activeTab === 'p2p' ? 'bg-fg/15 text-fg shadow-sm' : 'text-fg/60 hover:text-fg'}"
			>
				Jogar com amigos
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
						<div class="p-5 rounded-3xl bg-bg-elevated border border-fg/10 hover:border-amber-500/30 transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] flex flex-col justify-between gap-4 shadow-xl">
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
									class="flex-1 flex items-center justify-center gap-1.5 py-2 px-3 rounded-xl bg-brand-500 hover:bg-brand-400 text-brand-foreground text-xs font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] shadow-md active:scale-[0.98] cursor-pointer"
								>
									<Check class="w-3.5 h-3.5" />
									<span>{req.incoming ? "Aceitar Convite" : "Convite enviado"}</span>
								</button>
								<button
									type="button"
									onclick={() => declineFriend(req)}
									class="flex items-center justify-center py-2 px-3 rounded-xl bg-fg/5 hover:bg-rose-500/20 text-fg/70 hover:text-rose-400 border border-fg/10 hover:border-rose-500/30 text-xs font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] active:scale-[0.98] cursor-pointer"
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
					class="px-5 py-2.5 rounded-xl bg-fg/5 hover:bg-fg/10 text-fg/70 hover:text-fg text-xs font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] cursor-pointer"
				>
					Cancelar
				</button>
				<button
					type="button"
					onclick={handleAddFriend}
					disabled={working || !newFriendUsername.trim()}
					class="px-6 py-2.5 rounded-xl bg-brand-500 hover:bg-brand-400 disabled:opacity-50 text-brand-foreground text-xs font-black transition-[color,background-color,border-color,box-shadow,transform,opacity,filter,outline-color,left,right,top,bottom] shadow-lg shadow-brand-500/20 cursor-pointer active:scale-[0.98]"
				>
					Adicionar Amigo
				</button>
			</div>
		</div>

	{:else if activeTab === "p2p"}
        <MeshPanel />

		{:else}
			{#if filteredFriends.length === 0}
				<div class="surface-glass relative overflow-hidden border-brand-500/15 p-10 text-center">
					<div class="pointer-events-none absolute inset-0 bg-gradient-to-br from-brand-500/10 via-transparent to-success/10"></div>
					<div class="relative space-y-4">
						<div class="mx-auto grid h-16 w-16 place-items-center rounded-2xl border border-brand-500/25 bg-brand-500/10 text-brand-300 shadow-glow"><Sparkles class="h-8 w-8" /></div>
						<h3 class="text-lg font-black text-fg">Sua próxima aventura começa aqui</h3>
						<p class="text-sm text-fg-muted max-w-md mx-auto">
						{searchQuery ? `Nenhum amigo corresponde a "${searchQuery}".` : "Você ainda não adicionou amigos nesta categoria."}
						</p>
						<div class="flex flex-wrap justify-center gap-2 pt-2">
							<button type="button" class={button({ variant: 'primary' })} onclick={() => activeTab = 'p2p'}><Radio class="h-4 w-4" />Hospedar um mundo</button>
							<button type="button" class={button({ variant: 'secondary' })} onclick={() => activeTab = 'add'} disabled={cloudOffline}><UserPlus class="h-4 w-4" />Adicionar amigo</button>
						</div>
					</div>
				</div>
		{:else}
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                 {#each filteredFriends as friend (friend.id)}
                    <FriendCard {friend} favourite={friendsState.favourites.includes(friend.id)} busy={working} onJoin={() => joinFriend(friend)} onFavourite={() => friendsState.toggleFavourite(friend.id)} onBlock={() => perform(() => friendsState.action("block", friend.id))} onRemove={() => removeFriend(friend.id, friend.username)} onInvite={() => inviteFriend(friend)} />
                {/each}
			</div>
		{/if}
		{/if}
</div>

<style>
	.friends-grid {
		background-image:
			linear-gradient(rgb(var(--fg) / 0.04) 1px, transparent 1px),
			linear-gradient(90deg, rgb(var(--fg) / 0.04) 1px, transparent 1px);
		background-size: 32px 32px;
	}
</style>
