<script lang="ts">
    import { ArrowUpRight, ChevronRight, Download, Sparkles } from 'lucide-svelte';
    import type { ModSearchResultItem } from '$lib/api';
    import { catalogBannerCandidates } from '$lib/utils/curatedBanners';
    import { translateUi as uiText } from '$lib/i18n/useTranslation.svelte';
    import LazyImage from '$lib/components/ui/LazyImage.svelte';
    import { pageMotion } from '$lib/actions/pageMotion';

    let { items, onOpen }: { items: ModSearchResultItem[]; onOpen: (item: ModSearchResultItem) => void } = $props();
    let selectedId = $state('');
    const selection = $derived(items.find(item => `${item.source}:${item.sourceId}` === selectedId) ?? items[0]);
    const index = $derived(items.indexOf(selection));
    let attempt = $state(0);
    let aspect = $state(16 / 9);
    const sources = $derived(selection ? catalogBannerCandidates(selection) : []);
    const artwork = $derived(sources[attempt] ?? null);
    const poster = $derived(artwork === selection?.iconUrl || aspect <= 1.2);
    $effect(() => { selection; attempt = 0; });
    $effect(() => { artwork; aspect = 16 / 9; });
</script>

{#if selection}
    <section class="spotlight" aria-label={uiText('catalogDesign.spotlight')}>
        <article class="spotlight-stage">
            {#key `${selection.source}:${selection.sourceId}`}
                <div use:pageMotion={selection.sourceId} class="spotlight-scene" aria-hidden="true">
                    {#if artwork}<img src={artwork} alt="" decoding="async" class:poster onload={event => { const image = event.currentTarget as HTMLImageElement; aspect = image.naturalWidth / Math.max(1, image.naturalHeight); }} onerror={() => attempt += 1} />{:else}<div class="spotlight-mesh"></div>{/if}
                </div>
                <div use:pageMotion={selection.title} class="spotlight-copy">
                    <div class="spotlight-eyebrow"><span class="spotlight-marker"></span>{uiText('catalogDesign.spotlight')}<span class="ml-auto font-mono">{String(index + 1).padStart(2,'0')} / {String(items.length).padStart(2,'0')}</span></div>
                    <h2>{selection.title}</h2>
                    <p class="spotlight-description">{selection.description}</p>
                    <div class="spotlight-meta"><span>{selection.author || selection.slug}</span><span>·</span><span class="inline-flex items-center gap-1.5"><Download class="h-3.5 w-3.5" />{new Intl.NumberFormat('en', { notation:'compact' }).format(selection.downloads)}</span></div>
                    <button type="button" class="spotlight-explore" onclick={() => onOpen(selection)}>{uiText('catalogDesign.explorePack')}<ArrowUpRight class="h-4 w-4" /></button>
                </div>
            {/key}
        </article>
        <div class="spotlight-picks">
            <div class="spotlight-picks-heading"><Sparkles class="h-4 w-4" /><span>{uiText('catalogDesign.onShelf')}</span></div>
            {#each items as item, position (`${item.source}:${item.sourceId}`)}
                <button type="button" class="spotlight-pick" aria-label={uiText('catalogDesign.showFeature', {name:item.title})} aria-pressed={selection === item} onclick={() => selectedId = `${item.source}:${item.sourceId}`}>
                    <span class="spotlight-number">0{position + 1}</span>
                    <span class="spotlight-icon"><LazyImage src={item.iconUrl || '/grass_block.png'} alt="" loading="eager" class="h-full w-full object-contain" fallback="/grass_block.png" /></span>
                    <span class="min-w-0 flex-1 text-left"><strong class="line-clamp-2">{item.title}</strong><span class="mt-1 block truncate text-[10px] text-fg-muted">{item.author || item.slug}</span></span>
                    <ChevronRight class="h-4 w-4 shrink-0" />
                </button>
            {/each}
        </div>
    </section>
{/if}

<style>
    .spotlight { display: grid; grid-template-columns: minmax(0, 1fr) 280px; gap: 16px; margin-bottom: 24px; }
    .spotlight-stage { position: relative; overflow: hidden; min-height: 350px; border-radius: 24px; border: 1px solid rgb(var(--border-strong)); background: rgb(var(--bg-subtle)); isolation: isolate; }
    .spotlight-scene { position: absolute; inset: 0; overflow: hidden; }
    .spotlight-scene img { width: 100%; height: 100%; object-fit: cover; object-position: 70% center; }
    .spotlight-scene img.poster { object-fit: contain; padding: 24px 24px 24px 55%; background: radial-gradient(ellipse at 80% 20%, rgb(var(--brand-500) / .3), transparent); }
    .spotlight-scene::after { content: ''; position: absolute; inset: 0; background: linear-gradient(90deg, rgb(var(--shadow-color) / .9), rgb(var(--shadow-color) / .6) 45%, rgb(var(--shadow-color) / .08)), linear-gradient(0deg, rgb(var(--shadow-color) / .75), transparent 70%); }
    .spotlight-mesh { position: absolute; inset: 0; background: radial-gradient(ellipse at 80% 0, rgb(var(--brand-400) / .45), transparent 70%), repeating-linear-gradient(135deg, transparent, transparent 24px, rgb(var(--surface-shine) / .04) 25px, transparent 26px); }
    .spotlight-copy { position: relative; display: flex; flex-direction: column; align-items: flex-start; min-height: 350px; padding: 28px 32px; color: rgb(var(--surface-shine)); }
    .spotlight-eyebrow { display: flex; align-items: center; gap: 8px; width: 100%; font-size: 10px; font-weight: 600; letter-spacing: .12em; text-transform: uppercase; color: rgb(var(--surface-shine) / .78); }
    .spotlight-marker { width: 6px; height: 6px; border-radius: 50%; background: rgb(var(--surface-shine)); box-shadow: 0 0 0 4px rgb(var(--surface-shine) / .12); }
    h2 { max-width: 650px; margin-top: 32px; font-size: clamp(26px, 3.2vw, 46px); line-height: 1.02; letter-spacing: -.05em; font-weight: 750; text-wrap: balance; }
    .spotlight-description { max-width: 440px; margin-top: 14px; display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; font-size: 12px; line-height: 1.8; color: rgb(var(--surface-shine) / .8); }
    .spotlight-meta { display: flex; align-items: center; flex-wrap: wrap; gap: 10px; margin-top: 14px; color: rgb(var(--surface-shine) / .72); font-size: 10px; }
    .spotlight-explore { display: flex; align-items: center; justify-content: space-between; gap: 24px; padding: 12px 16px; margin-top: 22px; border-radius: 12px; background: rgb(var(--surface-shine)); color: rgb(var(--shadow-color)); font-size: 12px; font-weight: 650; transition: translate 200ms ease; }
    .spotlight-explore:hover { translate: 0 -2px; }
    .spotlight-picks { display: flex; flex-direction: column; gap: 8px; min-width: 0; padding: 18px 10px 10px; border: 1px solid rgb(var(--border)); border-radius: 24px; background: rgb(var(--bg-elevated) / .96); }
    .spotlight-picks-heading { display: flex; align-items: center; gap: 8px; padding: 0 10px 12px; color: rgb(var(--fg-muted)); font-size: 11px; font-weight: 600; }
    .spotlight-pick { display: flex; align-items: center; flex: 1; gap: 10px; padding: 12px; border: 1px solid transparent; border-radius: 16px; color: rgb(var(--fg-muted)); transition: border-color 180ms ease, background-color 180ms ease; }
    .spotlight-pick[aria-pressed='true'] { border-color: rgb(var(--brand-400) / .3); background: rgb(var(--brand-500) / .08); color: rgb(var(--brand-action)); }
    .spotlight-pick:hover { background: rgb(var(--fg) / .04); }
    .spotlight-number { font-family: var(--font-mono); font-size: 10px; color: rgb(var(--fg-subtle)); }
    .spotlight-icon { height: 44px; width: 44px; flex-shrink: 0; overflow: hidden; border-radius: 12px; border: 1px solid rgb(var(--border)); background: rgb(var(--bg-subtle)); padding: 4px; }
    .spotlight-pick strong { font-size: 12px; font-weight: 650; color: rgb(var(--fg)); line-height: 1.5; }
    button:focus-visible { outline: 2px solid rgb(var(--brand-action)); outline-offset: 3px; }
    @media (max-width: 1000px) { .spotlight { grid-template-columns: minmax(0, 1fr); } .spotlight-picks { display: grid; grid-template-columns: repeat(3, minmax(0,1fr)); padding: 6px; gap: 4px; } .spotlight-picks-heading { display: none; } .spotlight-number, .spotlight-pick > :global(svg) { display: none; } .spotlight-pick { padding: 10px; gap: 8px; } }
    @media (max-width: 650px) { .spotlight-copy { padding: 24px; min-height: 320px; } .spotlight-stage { min-height: 320px; } .spotlight-pick { flex-direction: column; } .spotlight-pick strong { font-size: 10px; } .spotlight-pick span span { display: none; } h2 { margin-top: 24px; } }
    :global(.no-ui-motion) .spotlight-explore, :global(.no-anim) .spotlight-explore { translate: none; transition: none; }
    @media (prefers-reduced-motion: reduce) { .spotlight-explore, .spotlight-explore:hover { translate: none; transition: none; } }
</style>
