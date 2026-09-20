<script lang="ts">
	import { 
		Users, 
		UserPlus, 
		Play, 
		Loader2, 
		Trash2, 
		ExternalLink, 
		LogOut,
		Search,
		ChevronDown,
		ChevronUp,
		Mail,
		UserCheck,
		Sliders,
		Check,
		X
	} from "lucide-svelte";
	import { onMount } from "svelte";
    import { appState } from "$lib/stores/app.svelte";
    import { settings } from "$lib/stores/settings.svelte";
	import { slide, fade } from "svelte/transition";
	import { toast } from "$lib/stores/toasts.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { launchGame } from "$lib/api";
	import { openUrl } from "@tauri-apps/plugin-opener";

	let newFriendName = $state("");
	let showAddInput = $state(false);
	let friendSearchQuery = $state("");
	let showAccountMenu = $state(false);

	let onlineExpanded = $state(true);
	let offlineExpanded = $state(false);
	let pendingExpanded = $state(false);

	import MinecraftAvatar from "$lib/components/ui/MinecraftAvatar.svelte";
	import { friendsState } from "$lib/stores/friends.svelte";
	import type { Friend } from "$lib/api/social";
	import { goto } from "$app/navigation";
	import { joinWorld } from "$lib/utils/directJoin";
	const friends = $derived(friendsState.list);
	let showPendingDropdown = $state(false);
	let collapsed = $state(false);
	onMount(() => { collapsed = window.innerWidth < 1200; });

	function addFriend() { void goto("/friends"); }
	async function removeFriend(id: string, _name: string) {
		try { await friendsState.action("remove", id); } catch (error) { toast(String(error), "error"); }
	}
	async function acceptFriend(friend: Friend) {
		try { await friendsState.action("accept", friend.id); } catch (error) { toast(String(error), "error"); }
	}
	function declineFriend(friend: Friend) { return removeFriend(friend.id, friend.username); }
	async function joinFriend(friend: Friend) {
		try { await joinWorld(`${friend.serverIp}:${friend.serverPort || 25565}`, friend); } catch (error) { toast(String(error), "error"); }
	}
	const filteredFriends = $derived(
		friends.filter(f => 
			!friendSearchQuery || 
			f.username.toLowerCase().includes(friendSearchQuery.toLowerCase())
		)
	);

	const onlineFriends = $derived(filteredFriends.filter(f => f.status === "online" || f.status === "in_game").toSorted((a, b) => Number(friendsState.favourites.includes(b.id)) - Number(friendsState.favourites.includes(a.id))));
	const offlineFriends = $derived(filteredFriends.filter(f => f.status === "offline"));
	const pendingFriends = $derived(filteredFriends.filter(f => f.status === "pending" && f.incoming));

	function handleLogout() {
		friendsState.disconnect();
		account.value = null;
		if (typeof window !== "undefined") {
			localStorage.removeItem("luxmc_current_account");
		}
		showAccountMenu = false;
		toast("Você saiu da conta.", "info");
	}

	const isMicrosoft = $derived(
		Boolean(
			account.value?.minecraftToken &&
			!account.value?.id.startsWith("offline_") &&
			!account.value?.id.startsWith("offline-")
		)
	);
</script>

<aside class="shrink-0 flex flex-col gap-4 select-none pb-8 h-full overflow-y-auto custom-scrollbar transition-all duration-200 {collapsed ? 'w-12' : 'w-[280px]'}">
	<button type="button" class="flex items-center justify-center gap-2 rounded-xl border border-border bg-bg-elevated/80 p-3 text-fg-muted hover:text-fg" onclick={() => collapsed = !collapsed} aria-label={collapsed ? "Expandir amigos" : "Minimizar amigos"} aria-expanded={!collapsed}><Users class="h-4 w-4" />{#if !collapsed}<span class="text-xs">Amigos</span>{/if}</button>
	{#if !collapsed}
	{@const hasInstance = profiles.list.length > 0}
	{@const hasAccount = Boolean(account.value)}
	{@const isMsLoggedIn = Boolean(account.value?.minecraftToken && !account.value?.id.startsWith("offline_"))}
	{@const allDone = hasInstance && hasAccount}
	<div class="bg-[#14171d] border border-white/[0.06] rounded-2xl p-3.5 space-y-2.5 shadow-sm">
		<div class="flex items-center justify-between">
			<span class="text-xs font-black text-white tracking-wide">Começando</span>
			<span class="text-[10px] font-bold text-[#1bd96a] bg-[#1bd96a]/15 px-2 py-0.5 rounded-full">
				{Number(hasInstance) + Number(hasAccount)}/2
			</span>
		</div>
		<div class="space-y-1.5">
			<button
				type="button"
				onclick={() => goto("/instances?new=true")}
				class="w-full flex items-center gap-2.5 p-2 rounded-xl text-left transition-all cursor-pointer {hasInstance ? 'bg-white/[0.02] text-white/50 hover:text-white' : 'bg-white/[0.05] hover:bg-white/[0.08] text-white'}"
			>
				<div class="w-4 h-4 rounded-full flex items-center justify-center shrink-0 {hasInstance ? 'bg-[#1bd96a] text-[#090a0f]' : 'border border-white/30 text-transparent'}">
					{#if hasInstance}
						<Check class="w-2.5 h-2.5 stroke-[3]" />
					{/if}
				</div>
				<span class="text-xs font-semibold truncate {hasInstance ? 'line-through opacity-70' : ''}">Criar primeira instância</span>
			</button>

			<button
				type="button"
				onclick={() => { if (!hasAccount) goto("/"); }}
				class="w-full flex items-center gap-2.5 p-2 rounded-xl text-left transition-all cursor-pointer {hasAccount ? 'bg-white/[0.02] text-white/50 hover:text-white' : 'bg-white/[0.05] hover:bg-white/[0.08] text-white'}"
			>
				<div class="w-4 h-4 rounded-full flex items-center justify-center shrink-0 {hasAccount ? 'bg-[#1bd96a] text-[#090a0f]' : 'border border-white/30 text-transparent'}">
					{#if hasAccount}
						<Check class="w-2.5 h-2.5 stroke-[3]" />
					{/if}
				</div>
				<span class="text-xs font-semibold truncate {hasAccount ? 'line-through opacity-70' : ''}">Conectar conta</span>
			</button>
		</div>
	</div>

	{#if account.value}
		<div class="space-y-1.5 shrink-0 relative">
			<div 
				role="button"
				tabindex="0"
				onclick={() => showAccountMenu = !showAccountMenu}
				onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") showAccountMenu = !showAccountMenu; }}
				class="w-full bg-[#14171d] hover:bg-[#1a1e26] border border-white/[0.06] rounded-2xl p-2.5 flex items-center justify-between gap-2.5 transition-all cursor-pointer shadow-sm group"
			>
				<div class="flex items-center gap-2.5 min-w-0">
					<div class="w-9 h-9 rounded-full overflow-hidden bg-white/[0.04] border border-white/10 shrink-0">
						<img 
							src={activeSkinStore.current.avatarUrl || `https://mc-heads.net/avatar/${account.value.username}/64`} 
							alt="Avatar" 
							class="w-full h-full object-cover rounded-full" 
						/>
					</div>
					<div class="min-w-0 text-left">
						<div class="text-xs font-extrabold text-white truncate leading-tight group-hover:text-[#1bd96a] transition-colors">
							{account.value.username}
						</div>
						<div class="text-[10px] text-white/40 truncate">
							{isMicrosoft ? "Conta Microsoft" : "Conta Offline"}
						</div>
					</div>
				</div>

				<ChevronDown class="w-4 h-4 text-white/40 group-hover:text-white transition-transform {showAccountMenu ? 'rotate-180' : ''}" />
			</div>

			{#if showAccountMenu}
				<div 
					class="absolute top-full left-0 right-0 mt-1 bg-[#181b22] border border-white/10 rounded-xl shadow-2xl py-1 z-30 space-y-0.5"
					transition:slide={{ duration: 120 }}
				>
					<button
						type="button"
						onclick={handleLogout}
						class="w-full px-3 py-2 text-left text-xs text-red-400 hover:bg-red-500/10 flex items-center gap-2 cursor-pointer font-bold"
					>
						<LogOut class="w-3.5 h-3.5" /> Trocar de Conta / Sair
					</button>
				</div>
			{/if}
		</div>
	{/if}

    <a href="/settings" class="surface-glass flex items-center gap-3 p-3 text-xs text-fg-muted hover:border-brand-500/30">
        <span class="h-2 w-2 rounded-full {settings.value.discordRpc !== false ? 'bg-success' : 'bg-fg-subtle'}"></span>
        <span><span class="block text-[10px] font-semibold text-fg">Discord Rich Presence</span><span class="mt-1 block text-[10px]">{settings.value.discordRpc === false ? 'Desativado' : appState.isGameRunning ? `Jogando ${appState.activeGameDetails?.name || 'Minecraft'}` : 'Atividade do launcher habilitada'}</span></span>
    </a>
	<div class="space-y-2.5 shrink-0">
		<div class="flex items-center gap-1.5">
			<button
				type="button"
				onclick={() => showAddInput = !showAddInput}
				class="p-1.5 rounded-xl bg-bg-elevated hover:bg-bg-subtle text-fg/60 hover:text-fg border border-fg/5 transition-colors cursor-pointer"
				title="Add friend"
			>
				<UserPlus class="w-3.5 h-3.5" />
			</button>

			<div class="relative flex-1">
				<Search class="w-3 h-3 absolute left-2.5 top-1/2 -translate-y-1/2 text-fg/30" />
				<input
					type="text"
					bind:value={friendSearchQuery}
					placeholder="Buscar amigos…"
					class="w-full bg-bg-elevated border border-fg/5 rounded-xl pl-7 pr-2.5 py-1.5 text-xs text-fg placeholder:text-fg/25 outline-none focus:border-fg/20 transition-all"
				/>
			</div>

			<div class="relative">
				<button
					type="button"
					onclick={() => showPendingDropdown = !showPendingDropdown}
					class="p-1.5 rounded-xl {showPendingDropdown ? 'bg-blue-600 text-fg shadow-md' : 'bg-bg-elevated hover:bg-bg-subtle text-fg/60 hover:text-fg'} border border-fg/5 transition-colors cursor-pointer"
					title="Solicitações de Amizade"
				>
					<Mail class="w-3.5 h-3.5" />
				</button>
				{#if pendingFriends.length > 0}
					<span class="absolute -top-1 -right-1 w-3.5 h-3.5 rounded-full bg-blue-500 text-fg text-[9px] font-black flex items-center justify-center pointer-events-none">
						{pendingFriends.length}
					</span>
				{/if}

				{#if showPendingDropdown}
					<div 
						class="absolute right-0 top-full mt-2 w-64 bg-bg-elevated border border-fg/10 rounded-2xl shadow-2xl p-3 z-50 space-y-2.5 backdrop-blur-xl"
						transition:slide={{ duration: 150 }}
					>
						<div class="flex items-center justify-between border-b border-fg/5 pb-2">
							<span class="text-xs font-black text-fg">Solicitações ({pendingFriends.length})</span>
							<button 
								type="button" 
								onclick={() => showPendingDropdown = false}
								class="text-fg/40 hover:text-fg text-[10px] font-bold cursor-pointer"
							>
								✕
							</button>
						</div>

						{#if pendingFriends.length === 0}
							<div class="text-center py-4 text-fg/40 text-xs">
								Nenhum convite pendente
							</div>
						{:else}
							<div class="space-y-2 max-h-48 overflow-y-auto custom-scrollbar pr-1">
								{#each pendingFriends as friend (friend.id)}
									<div class="flex items-center justify-between gap-2 p-2 rounded-xl bg-bg-elevated border border-fg/5">
										<div class="flex items-center gap-2 min-w-0">
											<MinecraftAvatar username={friend.username} status={friend.status} activity={friend.activity} lastSeen={friend.lastSeen} class="h-7 w-7" />
											<span class="text-xs font-bold text-fg truncate max-w-[90px]">{friend.username}</span>
										</div>

										<div class="flex items-center gap-1 shrink-0">
											<button
												type="button"
												onclick={() => acceptFriend(friend)}
												class="p-1 rounded-lg bg-emerald-500/20 hover:bg-emerald-500 text-emerald-400 hover:text-brand-foreground transition-colors cursor-pointer"
												title="Aceitar convite"
											>
												<Check class="w-3 h-3 stroke-[3]" />
											</button>
											<button
												type="button"
												onclick={() => declineFriend(friend)}
												class="p-1 rounded-lg bg-red-500/20 hover:bg-red-500 text-red-400 hover:text-fg transition-colors cursor-pointer"
												title="Recusar convite"
											>
												<X class="w-3 h-3 stroke-[3]" />
											</button>
										</div>
									</div>
								{/each}
							</div>
						{/if}
					</div>
				{/if}
			</div>
		</div>

		{#if showAddInput}
			<div class="flex items-center gap-1.5" in:slide={{ duration: 120 }}>
				<input
					type="text"
					bind:value={newFriendName}
					placeholder="Gamertag..."
					class="flex-1 bg-bg-overlay/40 border border-fg/10 rounded-xl px-2.5 py-1 text-xs text-fg placeholder:text-fg/25 outline-none focus:border-emerald-400"
					onkeydown={(e) => { if (e.key === "Enter") addFriend(); }}
				/>
				<button
					type="button"
					onclick={addFriend}
					class="px-2.5 py-1 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-brand-foreground font-bold text-xs cursor-pointer"
				>
					Add
				</button>
			</div>
		{/if}

		<div class="space-y-1 text-xs">
			<div>
				<button
					type="button"
					onclick={() => onlineExpanded = !onlineExpanded}
					class="w-full flex items-center justify-between py-1 text-fg/70 hover:text-fg text-xs font-bold cursor-pointer"
				>
					<span>Online - {onlineFriends.length}</span>
					<ChevronDown class="w-3.5 h-3.5 text-fg/40 transition-transform {onlineExpanded ? 'rotate-180' : ''}" />
				</button>

				{#if onlineExpanded}
					<div class="space-y-1 pt-0.5 pb-1" transition:slide={{ duration: 150 }}>
						{#if onlineFriends.length === 0}
							<p class="text-[11px] text-fg/30 py-1">Nenhum amigo online</p>
						{:else}
							{#each onlineFriends as friend (friend.id)}
								<div class="flex items-center justify-between p-1.5 rounded-xl hover:bg-fg/5 transition-colors group">
									<div class="flex items-center gap-2 min-w-0">
										<MinecraftAvatar username={friend.username} status={friend.status} activity={friend.activity} lastSeen={friend.lastSeen} class="h-7 w-7" />
										<div class="min-w-0"><span class="text-xs font-medium text-fg truncate block">{friend.username}</span>
											{#if friend.serverIp && friend.status === "in_game"}<button type="button" onclick={() => joinFriend(friend)} class="text-[10px] font-bold text-brand-400 hover:text-brand-300">ENTRAR NO MUNDO</button>{/if}</div>
									</div>

									<button
										type="button"
										class="opacity-0 group-hover:opacity-100 p-1 text-fg/30 hover:text-red-400 transition-opacity cursor-pointer"
										title="Remover"
										onclick={() => removeFriend(friend.id, friend.username)}
									>
										<Trash2 class="w-3 h-3" />
									</button>
								</div>
							{/each}
						{/if}
					</div>
				{/if}
			</div>

			<div>
				<button
					type="button"
					onclick={() => offlineExpanded = !offlineExpanded}
					class="w-full flex items-center justify-between py-1 text-fg/40 hover:text-fg text-xs font-bold cursor-pointer"
				>
					<span>Offline - {offlineFriends.length}</span>
					<ChevronDown class="w-3.5 h-3.5 text-fg/40 transition-transform {offlineExpanded ? 'rotate-180' : ''}" />
				</button>

				{#if offlineExpanded}
					<div class="space-y-1 pt-0.5 pb-1" transition:slide={{ duration: 150 }}>
						{#each offlineFriends as friend (friend.id)}
							<div class="flex items-center justify-between p-1.5 rounded-xl hover:bg-fg/5 transition-colors group opacity-60 hover:opacity-100">
								<div class="flex items-center gap-2 min-w-0">
									<MinecraftAvatar username={friend.username} status={friend.status} activity={friend.activity} lastSeen={friend.lastSeen} class="h-7 w-7" />
									<div class="min-w-0"><span class="text-xs font-medium text-fg truncate block">{friend.username}</span>
											{#if friend.serverIp && friend.status === "in_game"}<button type="button" onclick={() => joinFriend(friend)} class="text-[10px] font-bold text-brand-400 hover:text-brand-300">ENTRAR NO MUNDO</button>{/if}</div>
								</div>

								<button
									type="button"
									class="opacity-0 group-hover:opacity-100 p-1 text-fg/30 hover:text-red-400 transition-opacity cursor-pointer"
									title="Remover"
									onclick={() => removeFriend(friend.id, friend.username)}
								>
									<Trash2 class="w-3 h-3" />
								</button>
							</div>
						{/each}
					</div>
				{/if}
			</div>

			<div>
				<button
					type="button"
					onclick={() => pendingExpanded = !pendingExpanded}
					class="w-full flex items-center justify-between py-1 text-fg/40 hover:text-fg text-xs font-bold cursor-pointer"
				>
					<span>Solicitações - {pendingFriends.length}</span>
					<ChevronDown class="w-3.5 h-3.5 text-fg/40 transition-transform {pendingExpanded ? 'rotate-180' : ''}" />
				</button>

				{#if pendingExpanded}
					<div class="space-y-1.5 pt-1 pb-1" transition:slide={{ duration: 150 }}>
						{#if pendingFriends.length === 0}
							<p class="text-[11px] text-fg/30 py-1">Nenhuma solicitação pendente</p>
						{:else}
							{#each pendingFriends as friend (friend.id)}
								<div class="flex items-center justify-between p-2 rounded-xl bg-fg/[0.04] border border-fg/10 hover:border-fg/20 transition-colors">
									<div class="flex items-center gap-2 min-w-0">
										<MinecraftAvatar username={friend.username} status={friend.status} activity={friend.activity} lastSeen={friend.lastSeen} class="h-7 w-7" />
										<div class="min-w-0">
											<span class="text-xs font-bold text-fg truncate block">{friend.username}</span>
											<span class="text-[10px] text-amber-400 font-medium block">Pendente</span>
										</div>
									</div>

									<div class="flex items-center gap-1.5 shrink-0">
										<button
											type="button"
											onclick={() => acceptFriend(friend)}
											class="p-1.5 rounded-lg bg-emerald-500/20 hover:bg-emerald-500/40 text-emerald-400 border border-emerald-500/30 transition-all cursor-pointer shadow-sm active:scale-[0.98]"
											title="Aceitar convite"
										>
											<Check class="w-3 h-3" />
										</button>
										<button
											type="button"
											onclick={() => declineFriend(friend)}
											class="p-1.5 rounded-lg bg-rose-500/20 hover:bg-rose-500/40 text-rose-400 border border-rose-500/30 transition-all cursor-pointer shadow-sm active:scale-[0.98]"
											title="Recusar convite"
										>
											<X class="w-3 h-3" />
										</button>
									</div>
								</div>
							{/each}
						{/if}
					</div>
				{/if}
			</div>
		</div>
	</div>

	<div class="space-y-2 shrink-0 pt-1">
		<div class="flex items-center justify-between">
			<span class="text-xs font-bold text-white/50 block">Notícias</span>
			<a href="/news" class="text-[10px] font-semibold text-[#1bd96a] hover:underline">Ver todas</a>
		</div>

		<a
			href="/news"
			class="group block bg-[#14171d] border border-white/[0.06] hover:border-[#1bd96a]/40 rounded-2xl overflow-hidden shadow-md transition-all cursor-pointer"
		>
			<div class="h-24 bg-gradient-to-br from-[#1bd96a]/20 via-[#14171d] to-[#14171d] relative overflow-hidden flex items-center justify-between p-3.5 border-b border-white/[0.06]">
				<div class="space-y-1 z-10">
					<span class="text-[9px] font-black text-[#1bd96a] uppercase tracking-wider bg-[#1bd96a]/15 border border-[#1bd96a]/30 px-2 py-0.5 rounded-full">Atualização</span>
					<div class="text-xs font-black text-white group-hover:text-[#1bd96a] transition-colors">Luxmc v1.8.0 Oficial</div>
				</div>
				<div class="w-9 h-9 rounded-xl bg-[#1bd96a]/15 border border-[#1bd96a]/30 flex items-center justify-center text-[#1bd96a] shadow-md">
					<Sliders class="w-4 h-4" />
				</div>
			</div>

			<div class="p-3.5 space-y-1.5">
				<p class="text-[11px] text-white/60 leading-relaxed line-clamp-2">
					Performance de ponta, visual repaginado no padrão Modrinth e novo visualizador 3D de skins integrado.
				</p>
				<div class="flex items-center justify-between pt-1 text-[10px] text-white/40">
					<span>Setembro, 2026</span>
					<span class="text-[#1bd96a] font-bold group-hover:translate-x-0.5 transition-transform">Ler mais →</span>
				</div>
			</div>
		</a>
	</div>

	{/if}
</aside>
