<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
    import type { Snippet } from "svelte";
    import { Play, Square, Loader2, Settings2, Share2, ImagePlus } from "lucide-svelte";
    import type { Profile } from "$lib/stores/profiles.svelte";
    import { button } from "$lib/components/ui/button";
    import LoaderBadge from "./LoaderBadge.svelte";
    import { getIconSrc } from "$lib/utils/icons";
    import { scenery } from "$lib/visuals/scenery";
    import { settings } from "$lib/stores/settings.svelte";
    import { appState } from "$lib/stores/app.svelte";
    let { profile, banner, launching = false, running = false, stopping = false, status = "", progress = 0, javaLabel = uiText("ui.b803f24d52edd7bc"), onPlay, onStop, onSettings, onHost, actions }: {
        profile: Profile | null; banner: string; launching?: boolean; running?: boolean; stopping?: boolean; status?: string; progress?: number;
        javaLabel?: string; onPlay: () => void; onStop?: () => void; onSettings: () => void; onHost: () => void; actions?: Snippet;
    } = $props();
    const icon = $derived(getIconSrc(profile?.icon));
</script>

<section class="surface-glass relative overflow-hidden" use:scenery={settings.value.animations !== false && !appState.performanceMode && !running}>
    <div class="instance-photo relative h-44 overflow-hidden sm:h-52">
        <img loading="lazy" decoding="async"
            src={banner}
            alt=""
            class="h-full w-full object-cover"
            onerror={(e) => { const image = e.currentTarget as HTMLImageElement; if (!image.src.endsWith('/cinema/forest.webp')) image.src = '/cinema/forest.webp'; }}
        />
        <div class="absolute inset-0 bg-gradient-to-t from-bg-elevated via-bg-elevated/30 to-bg-overlay/10"></div>
        <div class="absolute left-6 top-5 flex items-center gap-2"><LoaderBadge loader={profile?.loader || 'vanilla'} /><span class="rounded-full border border-fg/10 bg-bg-overlay/60 px-3 py-1 text-xs text-fg">Minecraft {profile?.mcVersion || '—'}</span></div>
        <button type="button" class={button({ variant: 'secondary', size: 'sm', class: 'absolute right-5 top-5 bg-bg-overlay/50' })} onclick={onSettings}><ImagePlus class="h-3.5 w-3.5" />{uiText("ui.af283098bccceacd")}</button>
    </div>
    <div class="relative -mt-12 px-6 pb-6 sm:px-7">
        <div class="flex flex-wrap items-end justify-between gap-5">
            <div class="flex min-w-0 items-end gap-4">
                <img loading="lazy" decoding="async"
                    src={icon}
                    alt={profile?.name || uiText("ui.e76907efa549eca8")}
                    class="h-24 w-24 shrink-0 rounded-2xl border border-fg/15 bg-bg-elevated p-2 object-contain shadow-elevated"
                    onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/grass_block.png'; }}
                />
                <div class="min-w-0 pb-1"><p class="page-eyebrow mb-2">{uiText("ui.f1f48d358ae21b3c")}</p><h1 class="text-3xl font-bold tracking-tight text-fg sm:text-4xl">{profile?.name || 'Minecraft'}</h1></div>
            </div>
            <div class="flex flex-wrap items-center gap-2">
                <button type="button" class={button({ variant: 'secondary', size: 'icon' })} onclick={onSettings} aria-label={uiText("ui.6c5b8d86c6726795")}><Settings2 class="h-4 w-4" /></button>
                <button type="button" class={button({ variant: 'secondary', size: 'icon' })} onclick={onHost} aria-label={uiText("ui.d5bb1349387bf3b9")}><Share2 class="h-4 w-4" /></button>
                {#if running}
                    <button type="button" disabled={stopping} class={launcherButton({ variant: "danger", size: "lg", class: "inline-flex items-center justify-center gap-2 uppercase disabled:opacity-50" })} onclick={onStop}>
                        {#if stopping}
                            <Loader2 class="h-4 w-4 animate-spin" />
                            {uiText("ui.504e804c01b6e701")}
                        {:else}
                            <Square class="h-4 w-4 fill-current" />
                            {uiText("ui.2a9c2c7bf7ba2c24")}
                        {/if}
                    </button>
                {:else}
                    <button type="button" class={button({ variant: 'play', size: 'hero' })} onclick={onPlay} disabled={launching} aria-busy={launching}>
                        {#if launching}<Loader2 class="h-5 w-5 animate-spin" />{uiText("ui.c3ddc7f44e9a8267")}{:else}<Play class="h-5 w-5 fill-current" />{uiText("ui.d49c33d021ecb149")}{/if}
                    </button>
                {/if}
            </div>
        </div>
        {#if launching}
            <div class="mt-4 space-y-2" role="status"><div class="flex justify-between text-xs text-fg-muted"><span>{status || uiText("ui.3f391c454b11cacc")}</span><span>{Math.round(progress)}%</span></div><div class="h-1.5 overflow-hidden rounded-full bg-fg/5"><div class="h-full rounded-full bg-brand-500 transition-[color,background-color,border-color,box-shadow,transform,opacity] duration-200" style:width={`${Math.max(0, Math.min(progress, 100))}%`}></div></div></div>
        {/if}
        {#if actions}<div class="mt-5 flex flex-wrap items-center gap-2 border-t border-fg/5 pt-4">{@render actions()}</div>{/if}
    </div>
</section>

<style>
    .instance-photo > img { transform: scale(1.06) translate3d(calc(var(--scene-x, 0) * -14px),calc(var(--scene-y, 0) * -10px),0); transition: transform 650ms cubic-bezier(.2,.7,.2,1); }
    @media (prefers-reduced-motion: reduce) { .instance-photo > img { transform: none; transition: none; } }
</style>
