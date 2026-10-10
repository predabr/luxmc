<script lang="ts">
import { translateUi as uiText, currentUiLocale } from "$lib/i18n/useTranslation.svelte";
    import { friendDisplayName } from '$lib/api/social';
    import { button as launcherButton } from "$lib/components/ui/button";
	import { quintOut } from "svelte/easing";
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
    import { newsState } from "$lib/stores/news.svelte";
    onMount(() => { void newsState.load(); });
	import { slide, fade } from "svelte/transition";
	import { toast } from "$lib/stores/toasts.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import MicrosoftLogo from "$lib/components/ui/MicrosoftLogo.svelte";
	import { launchGame } from "$lib/api";
	import { openUrl } from "@tauri-apps/plugin-opener";

	let friendSearchQuery = $state("");
	let showAccountMenu = $state(false);

	let onlineExpanded = $state(true);
	let offlineExpanded = $state(false);
	let pendingExpanded = $state(false);

	import MinecraftAvatar from "$lib/components/ui/MinecraftAvatar.svelte";
	import { friendsState } from "$lib/stores/friends.svelte";
	import { publicProfile } from "$lib/stores/publicProfile.svelte";
    import type { Friend } from "$lib/api/social";
	import { goto } from "$app/navigation";
	import { joinWorld } from "$lib/utils/directJoin";
	const friends = $derived(friendsState.list);
	let showPendingDropdown = $state(false);
	let collapsed = $state(false);
	onMount(() => { collapsed = window.innerWidth < 1200; });

	function addFriend() { void goto("/friends?tab=add"); }
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
	const pendingFriends = $derived(friends.filter(f => f.status === "pending" && f.incoming));
    $effect(() => { if (pendingFriends.length) pendingExpanded = true; });

	function handleLogout() {
		friendsState.disconnect();
		account.clear();
		showAccountMenu = false;
		toast(uiText("ui.9524cb8472a03761"), "info");
	}

	const isMicrosoft = $derived(
		Boolean(
			account.value?.minecraftToken &&
			!account.value?.id.startsWith("offline_") &&
			!account.value?.id.startsWith("offline-")
		)
	);
</script>

<aside style:width={collapsed ? "64px" : `${settings.value.rightSidebarWidth || 320}px`} class="launcher-right-sidebar shrink-0 sticky top-3 self-start flex flex-col gap-4 select-none h-[calc(100dvh-1.5rem)] min-h-0 my-3 mr-3 rounded-3xl overflow-y-auto custom-scrollbar border border-fg/10 bg-bg/85 shadow-elevated backdrop-blur-2xl transition-[width,padding] duration-200 {collapsed ? 'w-16 p-2' : 'p-4'}">
	<button type="button" class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center justify-center gap-2" })} onclick={() => collapsed = !collapsed} aria-label={collapsed ? uiText("ui.7a6bd25d922200a5") : uiText("ui.64ea1d5c74a216b5")} aria-expanded={!collapsed}><Users class="h-4 w-4" />{#if !collapsed}<span class="text-xs">{uiText("nav.friends")}</span>{/if}</button>
	{#if !collapsed}

	{#if account.value}
		<div class="space-y-1.5 shrink-0 relative">
			<div 
				role="button"
				tabindex="0"
				onclick={() => showAccountMenu = !showAccountMenu}
				onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") showAccountMenu = !showAccountMenu; }}
				class="w-full bg-bg/35 backdrop-blur-xl hover:bg-bg/35 backdrop-blur-xl border border-fg/[0.06] rounded-2xl p-2.5 flex items-center justify-between gap-2.5 transition-[background-color,border-color] duration-150 cursor-pointer shadow-sm group"
			>
				<div class="flex items-center gap-2.5 min-w-0">
					<div class="w-9 h-9 rounded-full overflow-hidden bg-fg/[0.04] border border-fg/10 shrink-0">
						<img loading="lazy" decoding="async" 
							src={friendsState.ownProfile?.portrait || activeSkinStore.current.avatarUrl || account.value.avatarUrl || `https://mc-heads.net/avatar/${account.value.username}/64`}
							alt={uiText("ui.ca8e826d9c2ec401")} 
							class="w-full h-full object-cover rounded-full" 
							onerror={(e) => {
								const img = e.currentTarget as HTMLImageElement;
								const fallback = account.value?.username ? `https://mc-heads.net/avatar/${account.value.username}/64` : "/grass_block.png";
								if (img.src !== fallback) img.src = fallback;
								else img.src = "/grass_block.png";
							}}
						/>
					</div>
					<div class="min-w-0 text-left">
						<div class="text-xs font-extrabold text-fg truncate leading-tight group-hover:text-brand-400 transition-colors">
							{friendsState.ownProfile?.displayName || account.value.username}
						</div>
						<div class="text-[10px] text-fg/40 truncate flex items-center gap-1.5 mt-0.5">
							{#if isMicrosoft}
								{#if friendsState.ownProfile?.role === "owner"}<span class="rounded-md bg-brand-400/15 px-1.5 py-0.5 text-brand-400">{uiText("ownerTools.badge")}</span>{/if}<MicrosoftLogo size={10} />
								<span class="text-emerald-400 font-semibold">{uiText("ui.7142cc12e4218a5a")}</span>
							{:else}
								<span>{uiText("ui.2793472a35db2b80")}</span>
							{/if}
						</div>
					</div>
				</div>

				<ChevronDown class="w-4 h-4 text-fg/40 group-hover:text-fg transition-transform {showAccountMenu ? 'rotate-180' : ''}" />
			</div>

			{#if showAccountMenu}
				<div 
					class="absolute top-full left-0 right-0 mt-1 bg-bg/35 backdrop-blur-xl border border-fg/10 rounded-xl shadow-2xl py-1 z-30 space-y-0.5"
					transition:slide={{ easing: quintOut, duration: 180 }}
				>
                    {#if friendsState.ownProfile?.role === 'owner'}<a href="/owner-tools" class={launcherButton({variant:'ghost',size:'sm',class:'w-full justify-start'})} onclick={() => showAccountMenu = false}><Sliders class="h-3.5 w-3.5" />{uiText('ownerTools.title')}</a>{/if}
                    <button type="button" disabled={!friendsState.me} onclick={() => { showAccountMenu = false; publicProfile.edit(); }} class={launcherButton({variant:'ghost',size:'sm',class:'w-full justify-start'})}><Sliders class="h-3.5 w-3.5" />{uiText('publicProfile.edit')}</button>
					<button
						type="button"
						onclick={handleLogout}
						class={launcherButton({ variant: "danger", size: "sm", class: "w-full text-left flex items-center gap-2" })}
					>
						<LogOut class="w-3.5 h-3.5" /> {uiText("ui.1655b0af148c8929")}
					</button>
				</div>
			{/if}
		</div>
	{/if}

	<div class="space-y-2.5 shrink-0">
		<div class="flex items-center gap-1.5">
			<button
				type="button"
				onclick={addFriend}
				class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
				title={uiText("ui.fcaaf1906cc6c7a6")}
			>
				<UserPlus class="w-3.5 h-3.5" />
			</button>

			<div class="relative flex-1">
				<Search class="w-3 h-3 absolute left-2.5 top-1/2 -translate-y-1/2 text-fg/30" />
				<input
					type="text"
					bind:value={friendSearchQuery}
					placeholder={uiText("ui.d9c5db7620e00677")}
					class="w-full bg-bg-elevated border border-fg/5 rounded-xl pl-7 pr-2.5 py-1.5 text-xs text-fg placeholder:text-fg/25 outline-none focus:border-fg/20 transition-[border-color] duration-150"
				/>
			</div>

			<div class="relative">
				<button
					type="button"
					onclick={() => showPendingDropdown = !showPendingDropdown}
					class="p-1.5 rounded-xl {showPendingDropdown ? 'bg-blue-600 text-fg shadow-md' : 'bg-bg-elevated hover:bg-bg-subtle text-fg/60 hover:text-fg'} border border-fg/5 transition-colors cursor-pointer"
					title={uiText("ui.431a4a4d99085d78")}
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
						transition:slide={{ easing: quintOut, duration: 220 }}
					>
						<div class="flex items-center justify-between border-b border-fg/5 pb-2">
							<span class="text-xs font-black text-fg">{uiText("ui.e6b4ea6a9403d1d2")}{pendingFriends.length})</span>
							<button 
								type="button" 
								onclick={() => showPendingDropdown = false}
								class={launcherButton({ variant: "ghost", size: "sm", class: "" })}
							>
								✕
							</button>
						</div>

						{#if pendingFriends.length === 0}
							<div class="text-center py-4 text-fg/40 text-xs">
								{uiText("ui.5d5e8132384ed1f0")}
							</div>
						{:else}
							<div class="space-y-2 max-h-48 overflow-y-auto custom-scrollbar pr-1">
								{#each pendingFriends as friend (friend.id)}
									<div class="flex items-center justify-between gap-2 p-2 rounded-xl bg-bg-elevated border border-fg/5">
										<div class="flex items-center gap-2 min-w-0">
											<button type="button" class="shrink-0 rounded-full focus-visible:ring-2 focus-visible:ring-brand-400" aria-label={`${uiText('publicProfile.view')}: ${friend.username}`} onclick={() => publicProfile.show(friend)}><MinecraftAvatar username={friend.username} avatarUrl={friend.avatarUrl} status={friend.status} activity={friend.activity} lastSeen={friend.lastSeen} class="h-7 w-7" /></button>
											<span class="text-xs font-bold text-fg truncate max-w-[90px]">{friendDisplayName(friend)}</span>{#if friend.role === "owner"}<span class="text-[9px] font-semibold text-brand-400">{uiText("ownerTools.badge")}</span>{/if}
										</div>

										<div class="flex items-center gap-1 shrink-0">
											<button
												type="button"
												onclick={() => acceptFriend(friend)}
												class={launcherButton({ variant: "primary", size: "icon", class: "" })}
												title={uiText("ui.3d72d4eb181a7ae6")}
											>
												<Check class="w-3 h-3 stroke-[3]" />
											</button>
											<button
												type="button"
												onclick={() => declineFriend(friend)}
												class={launcherButton({ variant: "danger", size: "icon", class: "" })}
												title={uiText("ui.0a5237265ae1a5f5")}
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


		<div class="grid grid-cols-2 gap-2 border-t border-border pt-3"><button type="button" class={launcherButton({variant:'ghost',size:'sm',class:'text-[11px]'})} onclick={() => publicProfile.edit()} disabled={!friendsState.me}><Sliders class="h-3.5 w-3.5" />{uiText('publicProfile.title')}</button><button type="button" class={launcherButton({variant:'ghost',size:'sm',class:'text-[11px]'})} onclick={() => goto('/hosting')}><Play class="h-3.5 w-3.5" />{uiText('friendsDesign.openRoom')}</button></div>
<div class="space-y-1 text-xs">
			<div>
				<button
					type="button"
					onclick={() => onlineExpanded = !onlineExpanded}
					class={launcherButton({ variant: "ghost", size: "sm", class: "w-full flex items-center justify-between" })}
				>
					<span>{uiText("ui.1ab2263ab504e587")} {onlineFriends.length}</span>
					<ChevronDown class="w-3.5 h-3.5 text-fg/40 transition-transform {onlineExpanded ? 'rotate-180' : ''}" />
				</button>

				{#if onlineExpanded}
					<div class="space-y-1 pt-0.5 pb-1" transition:slide={{ easing: quintOut, duration: 220 }}>
						{#if onlineFriends.length === 0}
							<p class="text-[11px] text-fg/30 py-1">{uiText("ui.23b0d29e118aeccb")}</p>
						{:else}
							{#each onlineFriends as friend (friend.id)}
								<div class="flex items-center justify-between p-1.5 rounded-xl hover:bg-fg/5 transition-colors group">
									<div class="flex items-center gap-2 min-w-0">
										<button type="button" class="shrink-0 rounded-full focus-visible:ring-2 focus-visible:ring-brand-400" aria-label={`${uiText('publicProfile.view')}: ${friend.username}`} onclick={() => publicProfile.show(friend)}><MinecraftAvatar username={friend.username} avatarUrl={friend.avatarUrl} status={friend.status} activity={friend.activity} lastSeen={friend.lastSeen} class="h-7 w-7" /></button>
										<div class="min-w-0"><button type="button" onclick={() => publicProfile.show(friend)} class="block max-w-full truncate text-left text-xs font-medium text-fg hover:text-brand-400" title={uiText("publicProfile.view")}>{friendDisplayName(friend)}</button>{#if friend.role === "owner"}<span class="text-[9px] font-semibold text-brand-400">{uiText("ownerTools.badge")}</span>{/if}
											{#if friend.serverIp && friend.status === "in_game"}<button type="button" onclick={() => joinFriend(friend)} class={launcherButton({ variant: "ghost", size: "sm", class: "" })}>{uiText("ui.e551687514b0b026")}</button>{/if}</div>
									</div>

									<button
										type="button"
										class={launcherButton({ variant: "danger", size: "icon", class: "opacity-0 group-hover:opacity-100" })}
										title={uiText("mods.remove")}
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
					class={launcherButton({ variant: "ghost", size: "sm", class: "w-full flex items-center justify-between" })}
				>
					<span>{uiText("ui.fd13abb10d3f7ea7")} {offlineFriends.length}</span>
					<ChevronDown class="w-3.5 h-3.5 text-fg/40 transition-transform {offlineExpanded ? 'rotate-180' : ''}" />
				</button>

				{#if offlineExpanded}
					<div class="space-y-1 pt-0.5 pb-1" transition:slide={{ easing: quintOut, duration: 220 }}>
						{#each offlineFriends as friend (friend.id)}
							<div class="flex items-center justify-between p-1.5 rounded-xl hover:bg-fg/5 transition-colors group opacity-60 hover:opacity-100">
								<div class="flex items-center gap-2 min-w-0">
									<button type="button" class="shrink-0 rounded-full focus-visible:ring-2 focus-visible:ring-brand-400" aria-label={`${uiText('publicProfile.view')}: ${friend.username}`} onclick={() => publicProfile.show(friend)}><MinecraftAvatar username={friend.username} avatarUrl={friend.avatarUrl} status={friend.status} activity={friend.activity} lastSeen={friend.lastSeen} class="h-7 w-7" /></button>
									<div class="min-w-0"><button type="button" onclick={() => publicProfile.show(friend)} class="block max-w-full truncate text-left text-xs font-medium text-fg hover:text-brand-400" title={uiText("publicProfile.view")}>{friendDisplayName(friend)}</button>{#if friend.role === "owner"}<span class="text-[9px] font-semibold text-brand-400">{uiText("ownerTools.badge")}</span>{/if}
											{#if friend.serverIp && friend.status === "in_game"}<button type="button" onclick={() => joinFriend(friend)} class={launcherButton({ variant: "ghost", size: "sm", class: "" })}>{uiText("ui.e551687514b0b026")}</button>{/if}</div>
								</div>

								<button
									type="button"
									class={launcherButton({ variant: "danger", size: "icon", class: "opacity-0 group-hover:opacity-100" })}
									title={uiText("mods.remove")}
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
					class={launcherButton({ variant: "ghost", size: "sm", class: "w-full flex items-center justify-between" })}
				>
					<span>{uiText("ui.a014831dc18dac9d")} {pendingFriends.length}</span>
					<ChevronDown class="w-3.5 h-3.5 text-fg/40 transition-transform {pendingExpanded ? 'rotate-180' : ''}" />
				</button>

				{#if pendingExpanded}
					<div class="space-y-1.5 pt-1 pb-1" transition:slide={{ easing: quintOut, duration: 220 }}>
						{#if pendingFriends.length === 0}
							<p class="text-[11px] text-fg/30 py-1">{uiText("ui.b734de335f008912")}</p>
						{:else}
							{#each pendingFriends as friend (friend.id)}
								<div class="flex items-center justify-between p-2 rounded-xl bg-fg/[0.04] border border-fg/10 hover:border-fg/20 transition-colors">
									<div class="flex items-center gap-2 min-w-0">
										<button type="button" class="shrink-0 rounded-full focus-visible:ring-2 focus-visible:ring-brand-400" aria-label={`${uiText('publicProfile.view')}: ${friend.username}`} onclick={() => publicProfile.show(friend)}><MinecraftAvatar username={friend.username} avatarUrl={friend.avatarUrl} status={friend.status} activity={friend.activity} lastSeen={friend.lastSeen} class="h-7 w-7" /></button>
										<div class="min-w-0">
											<span class="text-xs font-bold text-fg truncate block">{friendDisplayName(friend)}</span>{#if friend.role === "owner"}<span class="text-[9px] font-semibold text-brand-400">{uiText("ownerTools.badge")}</span>{/if}
											<span class="text-[10px] text-amber-400 font-medium block">{uiText("ui.0b52c68ee6ed1831")}</span>
										</div>
									</div>

									<div class="flex items-center gap-1.5 shrink-0">
										<button
											type="button"
											onclick={() => acceptFriend(friend)}
											class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
											title={uiText("ui.3d72d4eb181a7ae6")}
										>
											<Check class="w-3 h-3" />
										</button>
										<button
											type="button"
											onclick={() => declineFriend(friend)}
											class={launcherButton({ variant: "danger", size: "icon", class: "" })}
											title={uiText("ui.0a5237265ae1a5f5")}
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

	<div class="flex flex-col gap-2.5 flex-1 min-h-0 pt-1">
		<div class="flex items-center justify-between shrink-0">
			<span class="text-xs font-bold text-fg/60 block">{uiText("ui.100608abed5cd87f")}</span>
			<a href="/news" class="text-[10px] font-semibold text-brand-400 hover:underline">{uiText("ui.6240cde90c92ce1c")}</a>
		</div>

		<div class="space-y-2 flex-1 min-h-0 overflow-y-auto custom-scrollbar pr-0.5 pb-2">
			{#each newsState.items as item (item.id)}
				<a
					href="/news"
					class="group block bg-bg-elevated/90 border border-fg/[0.06] hover:border-brand-400/40 rounded-xl overflow-hidden shadow-sm transition-[border-color] duration-150 cursor-pointer"
				>
					<div class="p-2.5 flex gap-2.5 items-center">
						<div class="w-12 h-12 rounded-lg overflow-hidden shrink-0 border border-fg/10 bg-black/40 relative">
							<img loading="lazy" decoding="async" src={item.image} alt={item.title} class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300" />
						</div>
						<div class="min-w-0 flex-1 space-y-0.5">
							<div class="flex items-center justify-between gap-1">
								<span class="text-[9px] font-black text-brand-400 uppercase tracking-wide">{item.category}</span>
								<span class="text-[9px] text-fg/30">{new Date(`${item.date}T12:00:00Z`).toLocaleDateString(currentUiLocale(), { timeZone: "UTC" })}</span>
							</div>
							<div class="text-[11px] font-black text-fg group-hover:text-brand-400 transition-colors leading-tight line-clamp-1">{item.title}</div>
							<p class="text-[10px] text-fg/50 leading-relaxed line-clamp-1">{item.summary}</p>
						</div>
					</div>
				</a>
			{/each}

			<div class="pt-2 px-1 text-center border-t border-fg/[0.04]">
				<span class="text-[10px] text-fg/30 font-medium">{uiText("ui.0261021de8d97650")}</span>
			</div>
		</div>
	</div>

	{/if}
</aside>
