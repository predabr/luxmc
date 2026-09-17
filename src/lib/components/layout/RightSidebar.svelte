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
		Sliders
	} from "lucide-svelte";
	import { onMount } from "svelte";
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

	type Friend = {
		id: string;
		username: string;
		status: "in_game" | "online" | "offline" | "pending";
		activity?: string;
		serverIp?: string;
		serverPort?: number;
		lastSeen?: string;
	};

	let friends = $state<Friend[]>([]);

	const defaultInitialFriends: Friend[] = [
		{
			id: "f-1",
			username: "Stantios",
			status: "online",
			activity: "No Launcher"
		},
		{
			id: "f-2",
			username: "coolbot100s",
			status: "online",
			activity: "Wynncraft"
		},
		{
			id: "f-3",
			username: "pedro_dev",
			status: "offline",
			activity: "2h atrás",
			lastSeen: "2h atrás"
		},
		{
			id: "f-4",
			username: "AlexGamer",
			status: "offline",
			activity: "ontem",
			lastSeen: "ontem"
		},
		{
			id: "f-5",
			username: "CraftMaster",
			status: "pending",
			activity: "Pedido recebido"
		}
	];

	function loadFriends() {
		if (typeof window === "undefined") return;
		try {
			const saved = localStorage.getItem("luxmc_custom_friends_v3");
			if (saved) {
				friends = JSON.parse(saved);
			} else {
				friends = defaultInitialFriends;
				localStorage.setItem("luxmc_custom_friends_v3", JSON.stringify(friends));
			}
		} catch {
			friends = defaultInitialFriends;
		}
	}

	function saveFriends() {
		if (typeof window === "undefined") return;
		try {
			localStorage.setItem("luxmc_custom_friends_v3", JSON.stringify(friends));
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

	const filteredFriends = $derived(
		friends.filter(f => 
			!friendSearchQuery || 
			f.username.toLowerCase().includes(friendSearchQuery.toLowerCase())
		)
	);

	const onlineFriends = $derived(filteredFriends.filter(f => f.status === "online" || f.status === "in_game"));
	const offlineFriends = $derived(filteredFriends.filter(f => f.status === "offline"));
	const pendingFriends = $derived(filteredFriends.filter(f => f.status === "pending"));

	function handleLogout() {
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

<aside class="w-[280px] shrink-0 flex flex-col gap-4 select-none pb-4">

	{#if account.value}
		<div class="space-y-1.5 shrink-0 relative">
			<span class="text-xs font-bold text-white/50 block">
				Playing as
			</span>

			<div 
				role="button"
				tabindex="0"
				onclick={() => showAccountMenu = !showAccountMenu}
				onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") showAccountMenu = !showAccountMenu; }}
				class="w-full bg-[#16171d] hover:bg-[#1c1d25] border border-white/5 rounded-2xl p-2.5 flex items-center justify-between gap-2.5 transition-all cursor-pointer shadow-sm group"
			>
				<div class="flex items-center gap-2.5 min-w-0">
					<div class="w-9 h-9 rounded-xl overflow-hidden bg-black/40 border border-white/10 shrink-0">
						<img 
							src={activeSkinStore.current.avatarUrl || `https://mc-heads.net/avatar/${account.value.username}/64`} 
							alt="Avatar" 
							class="w-full h-full object-cover"
						/>
					</div>
					<div class="min-w-0 text-left">
						<div class="text-xs font-bold text-white truncate leading-tight group-hover:text-emerald-300 transition-colors">
							{account.value.username}
						</div>
						<div class="text-[10px] text-white/40 truncate">
							{isMicrosoft ? "Minecraft account" : "Offline account"}
						</div>
					</div>
				</div>

				<ChevronDown class="w-4 h-4 text-white/40 group-hover:text-white transition-transform {showAccountMenu ? 'rotate-180' : ''}" />
			</div>

			{#if showAccountMenu}
				<div 
					class="absolute top-full left-0 right-0 mt-1 bg-[#181920] border border-white/10 rounded-xl shadow-2xl py-1 z-30 space-y-0.5"
					transition:slide={{ duration: 120 }}
				>
					<button
						type="button"
						onclick={handleLogout}
						class="w-full px-3 py-2 text-left text-xs text-red-400 hover:bg-red-500/10 flex items-center gap-2 cursor-pointer"
					>
						<LogOut class="w-3.5 h-3.5" /> Trocar de Conta / Sair
					</button>
				</div>
			{/if}
		</div>
	{/if}

	<div class="space-y-2.5 shrink-0">
		<div class="flex items-center gap-1.5">
			<button
				type="button"
				onclick={() => showAddInput = !showAddInput}
				class="p-1.5 rounded-xl bg-[#16171d] hover:bg-[#1f2029] text-white/60 hover:text-white border border-white/5 transition-colors cursor-pointer"
				title="Add friend"
			>
				<UserPlus class="w-3.5 h-3.5" />
			</button>

			<div class="relative flex-1">
				<Search class="w-3 h-3 absolute left-2.5 top-1/2 -translate-y-1/2 text-white/30" />
				<input
					type="text"
					bind:value={friendSearchQuery}
					placeholder="Search friends..."
					class="w-full bg-[#16171d] border border-white/5 rounded-xl pl-7 pr-2.5 py-1.5 text-xs text-white placeholder:text-white/25 outline-none focus:border-white/20 transition-all"
				/>
			</div>

			<div class="relative">
				<button
					type="button"
					class="p-1.5 rounded-xl bg-[#16171d] hover:bg-[#1f2029] text-white/60 hover:text-white border border-white/5 transition-colors cursor-pointer"
					title="Friend requests"
				>
					<Mail class="w-3.5 h-3.5" />
				</button>
				{#if pendingFriends.length > 0}
					<span class="absolute -top-1 -right-1 w-3.5 h-3.5 rounded-full bg-emerald-500 text-black text-[9px] font-black flex items-center justify-center">
						{pendingFriends.length}
					</span>
				{/if}
			</div>
		</div>

		{#if showAddInput}
			<div class="flex items-center gap-1.5" in:slide={{ duration: 120 }}>
				<input
					type="text"
					bind:value={newFriendName}
					placeholder="Gamertag..."
					class="flex-1 bg-black/40 border border-white/10 rounded-xl px-2.5 py-1 text-xs text-white placeholder:text-white/25 outline-none focus:border-emerald-400"
					onkeydown={(e) => { if (e.key === "Enter") addFriend(); }}
				/>
				<button
					type="button"
					onclick={addFriend}
					class="px-2.5 py-1 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-black font-bold text-xs cursor-pointer"
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
					class="w-full flex items-center justify-between py-1 text-white/70 hover:text-white text-xs font-bold cursor-pointer"
				>
					<span>Online - {onlineFriends.length}</span>
					<ChevronDown class="w-3.5 h-3.5 text-white/40 transition-transform {onlineExpanded ? 'rotate-180' : ''}" />
				</button>

				{#if onlineExpanded}
					<div class="space-y-1 pt-0.5 pb-1" transition:slide={{ duration: 150 }}>
						{#if onlineFriends.length === 0}
							<p class="text-[11px] text-white/30 py-1">Nenhum amigo online</p>
						{:else}
							{#each onlineFriends as friend (friend.id)}
								<div class="flex items-center justify-between p-1.5 rounded-xl hover:bg-white/5 transition-colors group">
									<div class="flex items-center gap-2 min-w-0">
										<div class="relative w-6 h-6 rounded-lg overflow-hidden bg-black/50 border border-white/10 shrink-0">
											<img
												src={`https://mc-heads.net/avatar/${friend.username}/64`}
												alt={friend.username}
												class="w-full h-full object-cover"
												loading="lazy"
											/>
											<span class="absolute bottom-0 right-0 w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
										</div>
										<span class="text-xs font-medium text-white truncate">{friend.username}</span>
									</div>

									<button
										type="button"
										class="opacity-0 group-hover:opacity-100 p-1 text-white/30 hover:text-red-400 transition-opacity cursor-pointer"
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
					class="w-full flex items-center justify-between py-1 text-white/40 hover:text-white text-xs font-bold cursor-pointer"
				>
					<span>Offline - {offlineFriends.length}</span>
					<ChevronDown class="w-3.5 h-3.5 text-white/40 transition-transform {offlineExpanded ? 'rotate-180' : ''}" />
				</button>

				{#if offlineExpanded}
					<div class="space-y-1 pt-0.5 pb-1" transition:slide={{ duration: 150 }}>
						{#each offlineFriends as friend (friend.id)}
							<div class="flex items-center justify-between p-1.5 rounded-xl hover:bg-white/5 transition-colors group opacity-60 hover:opacity-100">
								<div class="flex items-center gap-2 min-w-0">
									<div class="w-6 h-6 rounded-lg overflow-hidden bg-black/50 border border-white/10 shrink-0">
										<img
											src={`https://mc-heads.net/avatar/${friend.username}/64`}
											alt={friend.username}
											class="w-full h-full object-cover grayscale"
											loading="lazy"
										/>
									</div>
									<span class="text-xs font-medium text-white truncate">{friend.username}</span>
								</div>

								<button
									type="button"
									class="opacity-0 group-hover:opacity-100 p-1 text-white/30 hover:text-red-400 transition-opacity cursor-pointer"
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
					class="w-full flex items-center justify-between py-1 text-white/40 hover:text-white text-xs font-bold cursor-pointer"
				>
					<span>Pending - {pendingFriends.length}</span>
					<ChevronDown class="w-3.5 h-3.5 text-white/40 transition-transform {pendingExpanded ? 'rotate-180' : ''}" />
				</button>

				{#if pendingExpanded}
					<div class="space-y-1 pt-0.5 pb-1" transition:slide={{ duration: 150 }}>
						{#each pendingFriends as friend (friend.id)}
							<div class="flex items-center justify-between p-1.5 rounded-xl hover:bg-white/5 transition-colors">
								<span class="text-xs text-white/70">{friend.username}</span>
								<span class="text-[10px] text-amber-400">Pendente</span>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	</div>

	<div class="space-y-2 shrink-0 pt-1">
		<span class="text-xs font-bold text-white/50 block">
			News
		</span>

		<div class="bg-[#16171d] border border-white/5 rounded-2xl overflow-hidden shadow-sm">
			<div class="h-28 bg-[#1a2e26] relative overflow-hidden flex items-center justify-center p-3">
				<div class="flex items-center gap-3">
					<div class="space-y-2">
						<div class="w-14 h-1.5 bg-emerald-400 rounded-full"></div>
						<div class="w-14 h-1.5 bg-emerald-400/50 rounded-full"></div>
					</div>
					<div class="w-8 h-8 rounded-full bg-emerald-500/20 border border-emerald-400/40 flex items-center justify-center">
						<Sliders class="w-4 h-4 text-emerald-400" />
					</div>
				</div>
			</div>

			<div class="p-3.5 space-y-1.5">
				<h4 class="text-xs font-bold text-white leading-snug">
					Sync settings across instances
				</h4>
				<p class="text-[11px] text-white/50 leading-relaxed">
					Keep game options, servers, resource packs, and more the same across your instances.
				</p>
				<div class="pt-2 text-[10px] text-white/35">
					September 7, 2026
				</div>
			</div>
		</div>
	</div>

</aside>
