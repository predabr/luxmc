<script lang="ts">
    import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { lastSeenLabel } from "$lib/utils/lastSeen";
    import { Gamepad2, ArrowUpRight, Star, Trash2, Clock3, Copy, Check, Share2, MoreHorizontal, ShieldOff } from "lucide-svelte";
    import MinecraftAvatar from "$lib/components/ui/MinecraftAvatar.svelte";
    import { button } from "$lib/components/ui/button";
    import { toast } from "$lib/stores/toasts.svelte";
    import { publicProfile } from "$lib/stores/publicProfile.svelte";
    import type { Friend } from "$lib/api/social";

    let { friend, favourite = false, busy = false, onJoin, onFavourite, onRemove, onInvite, onBlock }: {
        friend: Friend; favourite?: boolean; busy?: boolean; onJoin: () => void; onFavourite: () => void; onRemove: () => void; onInvite?: () => void; onBlock?: () => void;
    } = $props();

    const playing = $derived(friend.status === 'in_game');
    const activity = $derived(playing ? `${friend.activity || 'Minecraft'}${friend.mcVersion ? ' · ' + friend.mcVersion : ''}` : friend.status === 'online' ? uiText('friendsDesign.onlineLauncher') : lastSeenLabel(friend.lastSeen));
    let copiedIp = $state(false);
    let menu: HTMLDetailsElement | undefined = $state();
    let menuOpen = $state(false);

    async function copyIp() {
        if (!friend.serverIp) return;
        const addr = `${friend.serverIp}:${friend.serverPort || 25565}`;
        try {
            await navigator.clipboard.writeText(addr);
            copiedIp = true;
            toast(uiText("ui.85bf877d94daa2ff", { arg0: addr }), "success");
            setTimeout(() => { copiedIp = false; }, 2000);
        } catch (error) { toast(String(error), "error"); }
    }

    function action(callback: () => void) {
        if (menu) menu.open = false;
        callback();
    }
</script>

<article class="friend-row group flex flex-wrap items-center gap-3 rounded-2xl border border-border bg-bg-elevated p-4 transition-colors hover:border-brand-400/30">
    <button type="button" onclick={() => publicProfile.show(friend)} class="shrink-0 rounded-full focus-visible:ring-2 focus-visible:ring-brand-400" aria-label={`${uiText('publicProfile.view')}: ${friend.username}`}><MinecraftAvatar username={friend.username} avatarUrl={friend.avatarUrl} status={friend.status} {activity} lastSeen={friend.lastSeen} class="h-11 w-11" /></button>
    <div class="min-w-0 flex-1">
        <div class="flex flex-wrap items-center gap-2"><h3 class="truncate text-sm font-semibold text-fg"><button type="button" class="hover:text-brand-400" onclick={() => publicProfile.show(friend)}>{friend.username}</button></h3>{#if playing}<span class="rounded-md bg-brand-500/10 px-1.5 py-0.5 text-[10px] font-medium text-brand-300">{uiText("ui.982d4bbcc310b45d")}</span>{/if}</div>
        <p class="mt-1.5 flex min-w-0 items-center gap-1.5 text-xs {playing ? 'text-brand-300' : 'text-fg-muted'}">{#if playing}<Gamepad2 class="h-3.5 w-3.5 shrink-0" />{:else if friend.status === 'offline'}<Clock3 class="h-3.5 w-3.5 shrink-0 text-fg-subtle" />{/if}<span class="truncate">{activity}</span></p>
    </div>
    <div class="friend-actions ml-auto flex items-center gap-1.5">
        {#if playing && friend.serverIp}<button type="button" class={button({ variant: 'primary', size: 'sm' })} onclick={onJoin} disabled={busy}>{uiText("ui.e551687514b0b026")}<ArrowUpRight class="h-3.5 w-3.5" /></button>{:else if friend.status === 'online' && onInvite}<button type="button" class={button({ variant: 'outline', size: 'sm' })} onclick={onInvite} disabled={busy}><Share2 class="h-3.5 w-3.5" />{uiText("ui.4152017c72674d9d")}</button>{/if}
        <button type="button" class={button({ variant: 'ghost', size: 'icon' })} aria-label={uiText('friendsDesign.favourite', { name: friend.username })} aria-pressed={favourite} title={uiText('friendsDesign.favourite', { name: friend.username })} onclick={onFavourite} disabled={busy}><Star class="h-4 w-4 {favourite ? 'fill-warning text-warning' : 'text-fg-subtle'}" /></button>
        <details bind:this={menu} class="relative" ontoggle={(event) => menuOpen = event.currentTarget.open}>
            <summary aria-expanded={menuOpen} onkeydown={(event) => { if (event.key === 'Escape' && menu) menu.open = false; }} aria-label={uiText('friendsDesign.more', { name: friend.username })} class="grid h-10 w-10 cursor-pointer list-none place-items-center rounded-xl border border-transparent text-fg-subtle hover:border-fg/10 hover:text-fg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-400"><MoreHorizontal class="h-4 w-4" /></summary>
            <div class="friend-menu absolute right-0 top-full z-20 mt-1 flex w-52 flex-col gap-1 rounded-xl border border-fg/15 bg-bg-elevated p-1.5 shadow-soft"><button type="button" class={button({variant:'ghost',size:'sm',block:true,class:'justify-start'})} onclick={() => action(() => publicProfile.show(friend))}>{uiText('publicProfile.view')}</button>
                {#if friend.serverIp}<button type="button" class={button({ variant: 'ghost', size: 'sm', block: true, class: 'justify-start' })} onclick={() => { if (menu) menu.open = false; void copyIp(); }}>{#if copiedIp}<Check class="h-3.5 w-3.5 text-success" />{:else}<Copy class="h-3.5 w-3.5" />{/if}{uiText("ui.0fdea38f1bf25778")}</button>{/if}
                <button type="button" class={button({ variant: 'ghostDanger', size: 'sm', block: true, class: 'justify-start' })} aria-label={uiText("ui.dbcc6acb9e61a47c", { arg0: friend.username })} onclick={() => action(onRemove)} disabled={busy}><Trash2 class="h-3.5 w-3.5" />{uiText('friendsDesign.remove')}</button>
                {#if onBlock}<button type="button" class={button({ variant: 'ghostDanger', size: 'sm', block: true, class: 'justify-start' })} onclick={() => onBlock && action(onBlock)} disabled={busy}><ShieldOff class="h-3.5 w-3.5" />{uiText("ui.096dc9d4b0204272")}</button>{/if}
            </div>
        </details>
    </div>
</article>

<style>
    summary::-webkit-details-marker { display: none; }
    :global(html.has-custom-wallpaper) .friend-menu { background-color: rgb(var(--bg-elevated) / .94) !important; }
    @media (max-width: 680px) { .friend-actions { justify-content: flex-end; width: 100%; } }
</style>
