<script lang="ts">
    import type { Snippet } from "svelte";
    import { Play, Square, Loader2, Settings2, Share2, Clock3, MemoryStick, Coffee, Box, ImagePlus } from "lucide-svelte";
    import type { Profile } from "$lib/stores/profiles.svelte";
    import { gamingStats } from "$lib/stores/gamingStats.svelte";
    import { button } from "$lib/components/ui/button";
    import LoaderBadge from "./LoaderBadge.svelte";
    let { profile, banner, launching = false, running = false, status = "", progress = 0, javaLabel = "Automático", onPlay, onStop, onSettings, onHost, actions }: {
        profile: Profile | null; banner: string; launching?: boolean; running?: boolean; status?: string; progress?: number;
        javaLabel?: string; onPlay: () => void; onStop?: () => void; onSettings: () => void; onHost: () => void; actions?: Snippet;
    } = $props();
    const icon = $derived(({ grass_block: '/grass_block.png', '/grass_block': '/grass_block.png', grass: '/grass_block.png', '/grass': '/grass_block.png', modpack_fo: '/modpack_fo_icon.png', modpack_better_mc: '/modpack_bmc_icon.webp', modpack_cobblemon: '/modpack_cobblemon_icon.png', logo: '/logo.png', grass_head: '/grass_head.png' } as Record<string, string>)[profile?.icon || 'grass_block'] || (profile?.icon === '/grass_block' ? '/grass_block.png' : profile?.icon) || '/grass_block.png');
    const minutes = $derived(profile ? gamingStats.profileMinutes(profile.id) : 0);
    const playtime = $derived(
        running
            ? `${Math.max(1, minutes)} min (Em jogo)`
            : minutes >= 60
                ? `${Math.floor(minutes / 60)}h ${minutes % 60}m`
                : minutes > 0
                    ? `${minutes} min`
                    : 'Menos de 1 min'
    );
</script>

<section class="surface-glass relative overflow-hidden">
    <div class="relative h-44 overflow-hidden sm:h-52">
        <img src={banner} alt="" class="h-full w-full object-cover" />
        <div class="absolute inset-0 bg-gradient-to-t from-bg-elevated via-bg-elevated/30 to-bg-overlay/10"></div>
        <div class="absolute left-6 top-5 flex items-center gap-2"><LoaderBadge loader={profile?.loader || 'vanilla'} /><span class="rounded-full border border-fg/10 bg-bg-overlay/60 px-3 py-1 text-xs text-fg">Minecraft {profile?.mcVersion || '—'}</span></div>
        <button type="button" class={button({ variant: 'secondary', size: 'sm', class: 'absolute right-5 top-5 bg-bg-overlay/50' })} onclick={onSettings}><ImagePlus class="h-3.5 w-3.5" />Personalizar</button>
    </div>
    <div class="relative -mt-12 px-6 pb-6 sm:px-7">
        <div class="flex flex-wrap items-end justify-between gap-5">
            <div class="flex min-w-0 items-end gap-4">
                <img src={icon} alt="" class="h-20 w-20 shrink-0 rounded-2xl border border-fg/15 bg-bg-elevated p-2 object-cover shadow-elevated" />
                <div class="min-w-0 pb-1"><p class="page-eyebrow mb-2">Sua próxima aventura</p><h1 class="text-3xl font-bold tracking-tight text-fg sm:text-4xl">{profile?.name || 'Minecraft'}</h1></div>
            </div>
            <div class="flex flex-wrap items-center gap-2">
                <button type="button" class={button({ variant: 'secondary', size: 'icon' })} onclick={onSettings} aria-label="Configurações da instância"><Settings2 class="h-4 w-4" /></button>
                <button type="button" class={button({ variant: 'secondary', size: 'icon' })} onclick={onHost} aria-label="Compartilhar mundo"><Share2 class="h-4 w-4" /></button>
                {#if running}
                    <button type="button" class="inline-flex items-center justify-center gap-2 rounded-2xl bg-red-500 hover:bg-red-600 active:scale-95 text-white text-xs font-black uppercase px-6 py-3 shadow-lg shadow-red-500/20 transition-all cursor-pointer" onclick={onStop}>
                        <Square class="h-4 w-4 fill-current" />
                        TERMINAR SESSÃO
                    </button>
                {:else}
                    <button type="button" class={button({ variant: 'play', size: 'hero' })} onclick={onPlay} disabled={launching} aria-busy={launching}>
                        {#if launching}<Loader2 class="h-5 w-5 animate-spin" />PREPARANDO{:else}<Play class="h-5 w-5 fill-current" />JOGAR MINECRAFT{/if}
                    </button>
                {/if}
            </div>
        </div>
        <div class="mt-7 grid grid-cols-2 gap-4 rounded-2xl border border-fg/5 bg-bg/30 p-4 xl:grid-cols-4">
            {#each [{ icon: MemoryStick, label: 'Memória alocada', value: `${((profile?.ramMb || 4096) / 1024).toFixed(1)} GB` }, { icon: Coffee, label: 'Java', value: javaLabel }, { icon: Clock3, label: 'Tempo de jogo', value: playtime }, { icon: Box, label: 'Mods instalados', value: String(profile?.modCount || 0) }] as stat}
                <div class="flex items-center gap-3"><stat.icon class="h-4 w-4 shrink-0 text-brand-400/80" /><div class="min-w-0"><p class="text-[10px] text-fg-subtle">{stat.label}</p><p class="mt-1 truncate text-sm font-semibold text-fg" title={stat.value}>{stat.value}</p></div></div>
            {/each}
        </div>
        {#if launching}
            <div class="mt-4 space-y-2" role="status"><div class="flex justify-between text-xs text-fg-muted"><span>{status || 'Verificando arquivos…'}</span><span>{Math.round(progress)}%</span></div><div class="h-1.5 overflow-hidden rounded-full bg-fg/5"><div class="h-full rounded-full bg-brand-500 transition-all duration-200" style:width={`${Math.max(0, Math.min(progress, 100))}%`}></div></div></div>
        {/if}
        {#if actions}<div class="mt-5 flex flex-wrap items-center gap-2 border-t border-fg/5 pt-4">{@render actions()}</div>{/if}
    </div>
</section>
