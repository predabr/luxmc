<script lang="ts">
    import { lastSeenLabel } from "$lib/utils/lastSeen";
    import { Gamepad2, ArrowUpRight, Star, Trash2, Clock3 } from "lucide-svelte";
    import MinecraftAvatar from "$lib/components/ui/MinecraftAvatar.svelte";
    import { button } from "$lib/components/ui/button";
    import type { Friend } from "$lib/api/social";
    let { friend, favourite = false, busy = false, onJoin, onFavourite, onRemove }: {
        friend: Friend; favourite?: boolean; busy?: boolean; onJoin: () => void; onFavourite: () => void; onRemove: () => void;
    } = $props();
    const playing = $derived(friend.status === 'in_game');
    const activity = $derived(playing ? `${friend.activity || 'Minecraft'}${friend.mcVersion ? ' · ' + friend.mcVersion : ''}` : friend.status === 'online' ? 'Online no launcher' : lastSeenLabel(friend.lastSeen));
</script>
<article class="surface-glass group relative flex flex-col gap-5 overflow-hidden p-5 transition-all duration-200 hover:border-brand-500/25">
    {#if playing}<div class="pointer-events-none absolute -right-8 -top-8 h-32 w-32 rounded-full bg-[radial-gradient(circle_at_center,rgb(168_85_247/0.15),transparent_70%)]"></div>{/if}
    <div class="relative flex items-start justify-between gap-3">
        <MinecraftAvatar username={friend.username} status={friend.status} activity={activity} lastSeen={friend.lastSeen} class="h-14 w-14" />
        <div class="flex gap-1"><button type="button" class={button({ variant: 'ghost', size: 'icon', class: 'h-8 w-8' })} aria-label={`Favoritar ${friend.username}`} aria-pressed={favourite} onclick={onFavourite}><Star class="h-4 w-4 {favourite ? 'fill-warning text-warning' : ''}" /></button><button type="button" class={button({ variant: 'ghost', size: 'icon', class: 'h-8 w-8 hover:text-danger' })} aria-label={`Remover ${friend.username}`} onclick={onRemove} disabled={busy}><Trash2 class="h-3.5 w-3.5" /></button></div>
    </div>
    <div><h3 class="text-base font-semibold text-fg">{friend.username}</h3><p class="mt-2 flex items-center gap-1.5 truncate text-xs {playing ? 'text-purple-300' : 'text-fg-muted'}">{#if playing}<Gamepad2 class="h-3.5 w-3.5 shrink-0" />{:else if friend.status === 'offline'}<Clock3 class="h-3.5 w-3.5 shrink-0" />{:else}<span class="h-1.5 w-1.5 rounded-full bg-success"></span>{/if}{activity}</p></div>
    {#if playing && friend.serverIp}<button type="button" class={button({ variant: 'primary', size: 'md', block: true })} onclick={onJoin} disabled={busy}>ENTRAR NO MUNDO<ArrowUpRight class="h-4 w-4" /></button>{:else}<div class="rounded-xl border border-fg/5 bg-bg/20 px-3 py-2.5 text-center text-[11px] text-fg-subtle">{friend.status === 'offline' ? 'Nos vemos na próxima aventura' : playing ? 'Mundo ainda não compartilhado' : 'Pronto para a próxima partida'}</div>{/if}
</article>
