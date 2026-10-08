<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import MeshPanel from "$lib/components/friends/MeshPanel.svelte";
    import { publicProfile } from "$lib/stores/publicProfile.svelte";
    import FriendCard from "$lib/components/friends/FriendCard.svelte";
    import { button } from "$lib/components/ui/button";
	import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/state";
    import { tunnelStatus } from "$lib/api/tunnel";
    import { Users, UserPlus, UserCheck, Radio, Check, X, Search, Copy, CloudOff, ShieldCheck, Wifi, ArrowRight, RefreshCw, LoaderCircle } from "lucide-svelte";
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
        if (page.url.searchParams.get("tab") === "add") activeTab = "add";
        void tunnelStatus().then(room => { if (!disposed && room && !page.url.searchParams.has("tab")) void goto("/hosting", { replaceState: true }); }).catch(() => {});
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
    $effect(() => { const tab = page.url.searchParams.get("tab"); if (tab === "add" || tab === "pending") activeTab = tab; });
	let searchQuery = $state("");
	let newFriendUsername = $state("");




	let working = $state(false);
	let suggestions = $state<SocialIdentity[]>([]);
	let selectedFriend = $state<SocialIdentity | null>(null);
	let searchError = $state("");
	let offlineRedirected = $state(false);
	const cloudOffline = $derived(Boolean(friendsState.error));

	$effect(() => {
		if (cloudOffline && !offlineRedirected && activeTab !== "add" && activeTab !== "pending") {
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
			if (!query) throw new Error(uiText("friendsDesign.enterNickname"));

            const [name, code] = query.split("#");
            let target = selectedFriend;
            if (!target && friendsState.me) {
                const found = await friendsState.search(query);
                const exact = found.filter(user => user.username.toLowerCase() === name.toLowerCase() && (!code || user.id.startsWith(code.toLowerCase())));
                if (exact.length > 1) throw new Error(uiText("ui.e2d8bd7afe1dc542"));
                target = exact[0] || null;
            }

			if (target) {
				await friendsState.action("invite", target.id);
				toast(uiText("friendsDesign.inviteSent", { name: target.username }), "success");
            } else {
                throw new Error(uiText("ui.d7433f454fe54e8a"));
            }

			newFriendUsername = "";
			selectedFriend = null;
			activeTab = "pending";
		});
	}

	function acceptFriend(friend: Friend) { return perform(() => friendsState.action("accept", friend.id)); }
	function declineFriend(friend: Friend) { return perform(() => friendsState.action("remove", friend.id)); }
	function removeFriend(id: string, _name: string) { return perform(() => friendsState.action("remove", id)); }

	function joinFriend(friend: Friend) {
		return perform(() => joinWorld(`${friend.serverIp}:${friend.serverPort || 25565}`, friend));
	}
	function inviteFriend(_friend: Friend) { activeTab = "p2p"; }
    async function copyCode() {
        if (!friendsState.me) return;
        try {
            await navigator.clipboard.writeText(`${friendsState.me.username}#${friendsState.me.id.slice(0, 8)}`);
            toast(uiText("ui.31b0fc0e4eaf096e"), "success");
        } catch (error) { toast(String(error), "error"); }
    }

    const listTitle = $derived(activeTab === "online" ? uiText("friendsDesign.onlineTitle") : uiText("friendsDesign.directory"));
    const sortedFriends = $derived([...filteredFriends].sort((a, b) => {
        const rank = (friend: Friend) => (friendsState.favourites.includes(friend.id) ? 0 : 10) + (friend.status === "in_game" ? 0 : friend.status === "online" ? 1 : 2);
        return rank(a) - rank(b) || a.username.localeCompare(b.username);
    }));
</script>

<div class="friends-page mx-auto flex h-full w-full max-w-screen-2xl flex-col gap-5 overflow-y-auto pb-8 select-none custom-scrollbar">
    <header class="friends-header flex flex-wrap items-center justify-between gap-4">
        <div class="flex items-center gap-3.5">
            <div class="grid h-11 w-11 place-items-center rounded-2xl border border-brand-400/20 bg-brand-500/10 text-brand-300"><Users class="h-5 w-5" /></div>
            <div>
                <h1 class="text-2xl font-bold tracking-tight text-fg">{uiText("nav.friends")}</h1>
                <p class="mt-1 text-sm text-fg-muted">{uiText("friendsDesign.subtitle")}</p>
            </div>
        </div>
        <div class="flex flex-wrap items-center gap-2">
            <button type="button" class={button({ variant: 'secondary', size: 'lg' })} onclick={() => activeTab = 'p2p'} aria-pressed={activeTab === 'p2p'}><Radio class="h-4 w-4" />{uiText("ui.88231d3da6438407")}</button>
            <button type="button" class={button({ variant: 'primary', size: 'lg' })} onclick={() => activeTab = 'add'} aria-pressed={activeTab === 'add'} disabled={cloudOffline}><UserPlus class="h-4 w-4" />{uiText("ui.fcaaf1906cc6c7a6")}</button>
        </div>
    </header>

    <div class="flex flex-wrap items-center gap-x-5 gap-y-2 border-b border-fg/10 pb-4 text-xs text-fg-muted" role="status">
        <span class="inline-flex items-center gap-2 {cloudOffline ? 'text-warning' : friendsState.me ? 'text-success' : ''}">{#if cloudOffline}<CloudOff class="h-3.5 w-3.5" />{uiText("ui.c81bb1c4da9eb81e")}{:else}<ShieldCheck class="h-3.5 w-3.5" />{friendsState.me ? uiText("friendsDesign.connected") : uiText("ui.098304fee8dd79fc")}{/if}</span>
        <span class="inline-flex items-center gap-2"><span class="h-1.5 w-1.5 rounded-full bg-success"></span>{uiText("friendsDesign.onlineCount", { count: onlineCount })}</span>
        <span class="inline-flex items-center gap-2"><Wifi class="h-3.5 w-3.5" />{uiText("friendsDesign.lanCount", { count: lanWorldCount })}</span>
    </div>

    {#if activeTab === 'p2p'}
        <div class="flex items-center justify-between gap-3">
            <h2 class="text-base font-semibold text-fg">{uiText("ui.4cab082a07901725")}</h2>
            <button type="button" class={button({ variant: 'ghost', size: 'sm' })} onclick={() => activeTab = 'all'}><Users class="h-4 w-4" />{uiText("friendsDesign.directory")}</button>
        </div>
        <MeshPanel />
    {:else}
        <div class="friends-workspace grid items-start gap-5">
            <section class="friends-directory min-w-0 overflow-visible rounded-3xl border border-border bg-bg-elevated shadow-soft" aria-label={uiText("friendsDesign.directory")}>
                <div class="flex flex-wrap items-center justify-between gap-3 border-b border-fg/10 p-4">
                    <div class="flex flex-wrap items-center gap-1" aria-label={uiText("friendsDesign.filters")}>
                        <button type="button" onclick={() => activeTab = 'all'} aria-pressed={activeTab === 'all'} class="launcher-tab flex items-center gap-2 text-xs {activeTab === 'all' ? 'bg-brand-500/10 text-brand-300' : 'text-fg-muted hover:text-fg'}">{uiText("friendsDesign.all")}<span class="text-[10px] text-fg-muted">{totalFriends}</span></button>
                        <button type="button" onclick={() => activeTab = 'online'} aria-pressed={activeTab === 'online'} class="launcher-tab flex items-center gap-2 text-xs {activeTab === 'online' ? 'bg-brand-500/10 text-brand-300' : 'text-fg-muted hover:text-fg'}">{uiText("friendsDesign.online")}<span class="text-[10px] text-fg-muted">{onlineCount}</span></button>
                        <button type="button" onclick={() => activeTab = 'pending'} aria-pressed={activeTab === 'pending'} class="launcher-tab flex items-center gap-2 text-xs {activeTab === 'pending' ? 'bg-brand-500/10 text-brand-300' : 'text-fg-muted hover:text-fg'}">{uiText("ui.07d0b754cc7cc62b")}{#if pendingCount > 0}<span class="grid h-5 min-w-5 place-items-center rounded-full bg-warning/15 px-1 text-[10px] text-warning">{pendingCount}</span>{/if}</button>
                    </div>
                    {#if activeTab !== 'add'}
                        <div class="relative min-w-0 flex-1 sm:max-w-60">
                            <Search class="pointer-events-none absolute left-3 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-fg-subtle" />
                            <input type="text" bind:value={searchQuery} aria-label={uiText("friendsDesign.search")} placeholder={uiText("friendsDesign.search")} class="h-10 w-full rounded-xl border border-fg/10 bg-bg-elevated pl-9 pr-3 text-xs text-fg outline-none placeholder:text-fg-subtle focus:border-brand-400/50" />
                        </div>
                    {/if}
                </div>

                {#if activeTab === 'add'}
                    <form class="space-y-5 p-5 sm:p-6" onsubmit={(event) => { event.preventDefault(); void handleAddFriend(); }}>
                        <div class="flex items-center gap-3"><div class="grid h-10 w-10 place-items-center rounded-xl bg-brand-500/10 text-brand-300"><UserPlus class="h-5 w-5" /></div><div><h2 class="text-lg font-semibold text-fg">{uiText("ui.f0c074065c9343fd")}</h2><p class="mt-1 text-xs text-fg-muted">{uiText("friendsDesign.findDescription")}</p></div></div>
                        <div class="space-y-2"><label for="friend-username" class="text-xs font-medium text-fg-muted">{uiText("friendsDesign.nickname")}</label><input id="friend-username" type="text" bind:value={newFriendUsername} disabled={working} placeholder={uiText("ui.f2402c145aed8cc4")} class="h-12 w-full rounded-xl border border-fg/15 bg-bg-subtle px-4 text-sm text-fg outline-none placeholder:text-fg-subtle focus:border-brand-400/60" /></div>
                        <div class="space-y-2" aria-live="polite">
                            {#each suggestions as suggestion (suggestion.id)}
                                <button type="button" class={button({ variant: 'outline', size: 'lg', block: true, class: 'justify-start text-left' })} onclick={() => selectedFriend = suggestion} aria-pressed={selectedFriend?.id === suggestion.id} disabled={working}>
                                    <MinecraftAvatar username={suggestion.username} avatarUrl={suggestion.avatarUrl} class="h-8 w-8" /><span class="min-w-0 truncate">{suggestion.username}<span class="text-fg-subtle">#{suggestion.id.slice(0, 8)}</span></span>{#if selectedFriend?.id === suggestion.id}<Check class="ml-auto h-4 w-4 text-success" />{/if}
                                </button>
                            {/each}
                            {#if searchError}<p class="text-xs text-danger">{searchError}</p>{/if}
                            <p class="text-xs leading-relaxed text-fg-muted">{uiText("ui.fb90fef4fd93e1b1")}</p>
                        </div>
                        <div class="flex flex-wrap items-center justify-end gap-2 border-t border-fg/10 pt-4"><button type="button" onclick={() => activeTab = 'all'} disabled={working} class={button({ variant: 'secondary', size: 'md' })}>{uiText("common.cancel")}</button><button type="submit" disabled={working || !newFriendUsername.trim() || !friendsState.me} aria-busy={working} class={button({ variant: 'primary', size: 'md' })}>{#if working}<LoaderCircle class="h-4 w-4 animate-spin motion-reduce:animate-none" />{:else}<UserPlus class="h-4 w-4" />{/if}{uiText("ui.f05b1fd40631dfae")}</button></div>
                    </form>
                {:else if activeTab === 'pending'}
                    <div class="flex items-center justify-between gap-3 px-5 pt-5"><h2 class="text-sm font-semibold text-fg">{uiText("ui.431a4a4d99085d78")}</h2><span class="text-xs text-fg-subtle">{pendingCount}</span></div>
                    {#if filteredFriends.length === 0}
                        <div class="friends-empty flex items-center gap-4 p-5 sm:p-6"><div class="grid h-12 w-12 shrink-0 place-items-center rounded-2xl border border-fg/10 bg-fg/5 text-fg-muted"><UserCheck class="h-5 w-5" /></div><div><h3 class="text-sm font-semibold text-fg">{uiText("ui.5d5e8132384ed1f0")}</h3><p class="mt-1.5 text-xs leading-relaxed text-fg-muted">{uiText("ui.2c4bc5f81729898d")}</p></div></div>
                    {:else}
                        <div class="space-y-2 p-3">
                            {#each filteredFriends as req (req.id)}
                                <article class="flex flex-wrap items-center gap-3 rounded-xl border border-fg/10 p-3"><MinecraftAvatar username={req.username} avatarUrl={req.avatarUrl} status={req.status} activity={req.activity} lastSeen={req.lastSeen} class="h-10 w-10" /><div class="min-w-0 flex-1"><h3 class="truncate text-sm font-semibold text-fg">{req.username}</h3><p class="mt-1 text-xs text-warning">{req.incoming ? uiText("ui.f20243fb0d307b75") : uiText("friendsDesign.awaiting")}</p></div><div class="ml-auto flex items-center gap-2">{#if req.incoming}<button type="button" onclick={() => acceptFriend(req)} disabled={working} class={button({ variant: 'primary', size: 'sm' })}><Check class="h-3.5 w-3.5" />{uiText("ui.f48583546c33511f")}</button>{:else}<span class="px-2 text-xs text-fg-muted">{uiText("friendsDesign.sent")}</span>{/if}<button type="button" onclick={() => declineFriend(req)} disabled={working} class={button({ variant: 'ghostDanger', size: 'icon' })} aria-label={uiText("friendsDesign.cancelRequest", { name: req.username })} title={uiText("ui.ce432849d60ed1ca")}><X class="h-4 w-4" /></button></div></article>
                            {/each}
                        </div>
                    {/if}
                {:else}
                    <div class="flex items-center justify-between gap-3 px-5 pt-5"><h2 class="text-sm font-semibold text-fg">{listTitle}</h2><span class="text-xs text-fg-subtle">{filteredFriends.length}</span></div>
                    {#if sortedFriends.length === 0}
                        <div class="friends-empty p-5 sm:p-6">
                            <div class="flex items-center gap-4"><div class="grid h-12 w-12 shrink-0 place-items-center rounded-2xl border border-brand-400/20 bg-brand-500/10 text-brand-300"><Users class="h-5 w-5" /></div><div><h3 class="text-base font-semibold text-fg">{searchQuery ? uiText("friendsDesign.noResults") : activeTab === 'online' ? uiText("friendsDesign.noOnline") : uiText("design.emptyFriends")}</h3><p class="mt-1.5 max-w-lg text-xs leading-relaxed text-fg-muted">{searchQuery ? uiText("ui.b8b75050effab5b9", { arg0: searchQuery }) : activeTab === 'online' ? uiText("friendsDesign.noOnlineHint") : uiText("friendsDesign.emptyHint")}</p></div></div>
                            {#if !searchQuery && activeTab === 'all'}<div class="mt-5 grid gap-3 border-t border-fg/10 pt-4 sm:grid-cols-2">{#each [{ title: uiText('friendsDesign.stepOne'), hint: uiText('friendsDesign.stepOneHint') }, { title: uiText('friendsDesign.stepTwo'), hint: uiText('friendsDesign.stepTwoHint') }] as step, index}<div class="flex gap-3"><span class="grid h-6 w-6 shrink-0 place-items-center rounded-lg border border-fg/10 text-[10px] font-semibold text-fg-muted">0{index + 1}</span><div><p class="text-xs font-medium text-fg">{step.title}</p><p class="mt-1 text-[11px] leading-relaxed text-fg-muted">{step.hint}</p></div></div>{/each}</div>{/if}
                        </div>
                    {:else}
                        <div class="friends-list grid gap-3 p-4">{#each sortedFriends as friend (friend.id)}<FriendCard {friend} favourite={friendsState.favourites.includes(friend.id)} busy={working} onJoin={() => joinFriend(friend)} onFavourite={() => friendsState.toggleFavourite(friend.id)} onBlock={() => perform(() => friendsState.action('block', friend.id))} onRemove={() => removeFriend(friend.id, friend.username)} onInvite={() => inviteFriend(friend)} />{/each}</div>
                    {/if}
                {/if}
            </section>

            <aside class="friends-context flex min-w-0 flex-col gap-4">
                <section class="rounded-3xl border border-border bg-bg-elevated p-5 shadow-soft">
                    <div class="mb-4 flex items-center justify-between gap-2"><h2 class="text-xs font-semibold text-fg-muted">{uiText("friendsDesign.yourProfile")}</h2><button type="button" disabled={!friendsState.me} onclick={() => publicProfile.edit()} class="text-xs text-brand-400 hover:underline">{uiText('publicProfile.edit')}</button></div>
                    <div class="flex items-center gap-3"><MinecraftAvatar username={friendsState.me?.username || account.value?.username || 'Steve'} avatarUrl={friendsState.ownProfile?.portrait || friendsState.me?.avatarUrl || account.value?.avatarUrl} status={friendsState.me ? 'online' : 'offline'} class="h-10 w-10" /><div class="min-w-0"><p class="truncate text-sm font-semibold text-fg">{friendsState.ownProfile?.displayName || friendsState.me?.username || account.value?.username || uiText('friendsDesign.profileUnavailable')}</p>{#if friendsState.ownProfile?.role === "owner"}<span class="text-[11px] font-semibold text-brand-400">{uiText("ownerTools.badge")}</span>{/if}<p class="mt-1 text-[11px] text-fg-muted">{friendsState.me ? uiText('friendsDesign.ready') : uiText('friendsDesign.connectHint')}</p></div></div>
                    {#if friendsState.me}<div class="mt-5 space-y-2"><p class="text-[11px] text-fg-muted">{uiText("friendsDesign.yourCode")}</p><div class="flex min-w-0 items-center gap-2"><code class="min-w-0 flex-1 break-all rounded-lg border border-fg/10 px-2.5 py-2 font-mono text-xs text-fg">{friendsState.me.username}#{friendsState.me.id.slice(0, 8)}</code><button type="button" title={uiText("ui.767b6747b7b7888f")} aria-label={uiText("friendsDesign.copyCode")} class={button({ variant: 'secondary', size: 'icon' })} onclick={copyCode}><Copy class="h-4 w-4" /></button></div></div>{/if}
                    <button type="button" class={button({ variant: friendsState.me ? 'ghost' : 'primary', size: 'sm', block: true, class: 'mt-3' })} onclick={() => friendsState.connect()} disabled={friendsState.busy || !account.value} aria-busy={friendsState.busy}>{#if friendsState.busy}<LoaderCircle class="h-3.5 w-3.5 animate-spin motion-reduce:animate-none" />{:else}<RefreshCw class="h-3.5 w-3.5" />{/if}{friendsState.busy ? uiText("ui.d6ac190ed5df66c8") : friendsState.me ? uiText("ui.ce410cef725982fa") : uiText("ui.48997b767b4a201a")}</button>
                </section>
                <section class="rounded-3xl border border-brand-400/20 bg-bg-elevated p-5 shadow-soft"><div class="flex items-center gap-2 text-brand-300"><Radio class="h-4 w-4" /><h2 class="text-sm font-semibold">{uiText("friendsDesign.playTogether")}</h2></div><p class="mt-3 text-xs leading-relaxed text-fg-muted">{uiText("friendsDesign.roomHint")}</p><button type="button" class={button({ variant: 'outline', size: 'md', block: true, class: 'mt-4 justify-between' })} onclick={() => activeTab = 'p2p'}>{uiText("friendsDesign.openRoom")}<ArrowRight class="h-4 w-4" /></button><p class="mt-3 text-[11px] leading-relaxed text-fg-subtle">{uiText("friendsDesign.compatibility")}</p></section>
                {#if localContacts.length}<details class="surface-glass p-4 text-xs text-fg-muted"><summary class="cursor-pointer">{uiText("ui.0258395a2887b1f4")}{localContacts.length})</summary><p class="my-3 leading-relaxed">{uiText("ui.408b9eb2fb314ad0")}</p><div class="flex flex-wrap gap-2">{#each localContacts as name}<button type="button" class={button({ variant: 'ghostBrand', size: 'sm' })} onclick={() => { newFriendUsername = name; activeTab = 'add'; }}>{name}</button>{/each}</div></details>{/if}
            </aside>
        </div>
    {/if}
</div>

<style>
    .friends-workspace { grid-template-columns: minmax(0, 1fr) 300px; align-items: start; }
    @media (min-width: 1500px) { .friends-list { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
    .friends-context { background: transparent !important; }
    .friends-directory { box-shadow: inset 0 1px 0 rgb(var(--fg) / .04); }
    .friends-empty { min-height: 152px; }
    :global(html.has-custom-wallpaper) .friends-directory, :global(html.has-custom-wallpaper) .friends-context > section { background: rgb(var(--bg-elevated) / .94) !important; backdrop-filter: blur(12px); }
    @media (max-width: 1200px) {
        .friends-workspace { grid-template-columns: minmax(0, 1fr); }
        .friends-context { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); }
    }
    @media (max-width: 680px) {
        .friends-context { grid-template-columns: minmax(0, 1fr); }
        .friends-header { align-items: flex-start; }
    }
</style>
