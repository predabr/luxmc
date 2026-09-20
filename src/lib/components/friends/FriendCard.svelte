<script lang="ts">
    import { lastSeenLabel } from "$lib/utils/lastSeen";
    import { Gamepad2, ArrowUpRight, Star, Trash2, Clock3, Copy, Check, Share2 } from "lucide-svelte";
    import MinecraftAvatar from "$lib/components/ui/MinecraftAvatar.svelte";
    import { button } from "$lib/components/ui/button";
    import { toast } from "$lib/stores/toasts.svelte";
    import type { Friend } from "$lib/api/social";

    let { friend, favourite = false, busy = false, onJoin, onFavourite, onRemove, onInvite }: {
        friend: Friend; favourite?: boolean; busy?: boolean; onJoin: () => void; onFavourite: () => void; onRemove: () => void; onInvite?: () => void;
    } = $props();

    const playing = $derived(friend.status === 'in_game');
    const activity = $derived(playing ? `${friend.activity || 'Minecraft'}${friend.mcVersion ? ' · ' + friend.mcVersion : ''}` : friend.status === 'online' ? 'Online no launcher' : lastSeenLabel(friend.lastSeen));

    let copiedIp = $state(false);
    function copyIp() {
        if (!friend.serverIp) return;
        const addr = `${friend.serverIp}:${friend.serverPort || 25565}`;
        navigator.clipboard.writeText(addr).then(() => {
            copiedIp = true;
            toast(`Endereço ${addr} copiado!`, "success");
            setTimeout(() => { copiedIp = false; }, 2000);
        });
    }
</script>

<article class="surface-glass group relative flex flex-col justify-between gap-4 overflow-hidden p-5 transition-all duration-150 {playing ? 'border-purple-500/35 bg-purple-950/15 shadow-[0_4px_24px_-4px_rgba(168,85,247,0.2)]' : friend.status === 'online' ? 'border-emerald-500/25 hover:border-emerald-500/40' : 'hover:border-fg/20'}">
    {#if playing}
        <div class="pointer-events-none absolute -right-8 -top-8 h-32 w-32 rounded-full bg-[radial-gradient(circle_at_center,rgb(168_85_247/0.2),transparent_70%)]"></div>
    {/if}

    <div class="flex items-start justify-between gap-3">
        <div class="relative">
            <MinecraftAvatar username={friend.username} status={friend.status} activity={activity} lastSeen={friend.lastSeen} class="h-14 w-14 ring-2 {playing ? 'ring-purple-500/50' : friend.status === 'online' ? 'ring-emerald-500/40' : 'ring-fg/10'}" />
            <span class="absolute -bottom-1 -right-1 w-4 h-4 rounded-full border-2 border-bg-elevated flex items-center justify-center text-[9px] font-black {playing ? 'bg-purple-500 text-white' : friend.status === 'online' ? 'bg-emerald-500 text-white' : 'bg-fg/30 text-fg/70'}">
                {#if playing}🎮{:else if friend.status === 'online'}●{:else}○{/if}
            </span>
        </div>

        <div class="flex items-center gap-1">
            {#if playing && friend.serverIp}
                <button
                    type="button"
                    class="p-1.5 rounded-xl bg-fg/5 hover:bg-fg/10 text-fg/70 hover:text-fg text-xs transition-colors cursor-pointer"
                    title="Copiar endereço do servidor/mundo"
                    onclick={copyIp}
                >
                    {#if copiedIp}<Check class="h-3.5 w-3.5 text-emerald-400" />{:else}<Copy class="h-3.5 w-3.5" />{/if}
                </button>
            {/if}
            <button type="button" class={button({ variant: 'ghost', size: 'icon', class: 'h-8 w-8' })} aria-label={`Favoritar ${friend.username}`} aria-pressed={favourite} onclick={onFavourite}>
                <Star class="h-4 w-4 {favourite ? 'fill-warning text-warning' : 'text-fg/40 hover:text-fg'}" />
            </button>
            <button type="button" class={button({ variant: 'ghost', size: 'icon', class: 'h-8 w-8 hover:text-danger' })} aria-label={`Remover ${friend.username}`} onclick={onRemove} disabled={busy}>
                <Trash2 class="h-3.5 w-3.5 text-fg/40 hover:text-danger" />
            </button>
        </div>
    </div>

    <div>
        <div class="flex items-center gap-2">
            <h3 class="text-base font-bold text-fg truncate">{friend.username}</h3>
            {#if playing}
                <span class="px-2 py-0.5 rounded-full text-[10px] font-black bg-purple-500/20 text-purple-300 border border-purple-500/30">JOGANDO</span>
            {:else if friend.status === 'online'}
                <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/15 text-emerald-400 border border-emerald-500/25">ONLINE</span>
            {/if}
        </div>
        <p class="mt-1.5 flex items-center gap-1.5 truncate text-xs {playing ? 'text-purple-300 font-medium' : 'text-fg-muted'}">
            {#if playing}
                <Gamepad2 class="h-3.5 w-3.5 shrink-0 text-purple-400" />
            {:else if friend.status === 'offline'}
                <Clock3 class="h-3.5 w-3.5 shrink-0 text-fg-subtle" />
            {:else}
                <span class="h-1.5 w-1.5 rounded-full bg-success"></span>
            {/if}
            {activity}
        </p>
    </div>

    <div>
        {#if playing && friend.serverIp}
            <button type="button" class={button({ variant: 'primary', size: 'md', block: true, class: 'bg-emerald-600 hover:bg-emerald-500 text-white font-bold shadow-lg shadow-emerald-600/20 active:scale-98' })} onclick={onJoin} disabled={busy}>
                ENTRAR NO MUNDO <ArrowUpRight class="h-4 w-4 ml-1" />
            </button>
        {:else if friend.status === 'online' && onInvite}
            <button
                type="button"
                class="w-full py-2 px-3 rounded-xl bg-brand-500/15 hover:bg-brand-500/25 border border-brand-500/30 text-brand-300 hover:text-brand-200 text-xs font-bold transition-all flex items-center justify-center gap-1.5 cursor-pointer active:scale-98 shadow-sm"
                onclick={onInvite}
            >
                <Share2 class="h-3.5 w-3.5 text-brand-400" />
                <span>Convidar para Meu Mundo</span>
            </button>
        {:else}
            <div class="rounded-xl border border-fg/5 bg-bg/30 px-3 py-2 text-center text-[11px] text-fg-subtle font-medium">
                {friend.status === 'offline' ? 'Offline' : playing ? 'Mundo privado (não compartilhado)' : 'Pronto para jogar'}
            </div>
        {/if}
    </div>
</article>

