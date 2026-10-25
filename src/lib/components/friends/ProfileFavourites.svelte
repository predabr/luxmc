<script lang="ts">
    import type { PackArtwork } from '$lib/api/social';
    import { profiles } from '$lib/stores/profiles.svelte';
    import { distinctFavourites } from '$lib/utils/profileFavourites';
    let { names, artwork = [], layout = 'grid', compact = false, disabled = false, onselect }: {names:string[];artwork?:PackArtwork[];layout?:'grid'|'spotlight'|'poster';compact?:boolean;disabled?:boolean;onselect?:(name:string,kind:'mod'|'modpack')=>void} = $props();
    const entries = $derived(distinctFavourites(names, artwork, profiles.list));
    function fallback(event: Event) { const image = event.currentTarget as HTMLImageElement; if (!image.src.endsWith('/grass_block.png')) image.src = '/grass_block.png'; }
</script>

<div class="showcase-grid" class:showcase-spotlight={layout === 'spotlight' && !compact} class:showcase-poster={layout === 'poster' && !compact} class:showcase-compact={compact}>
    {#each entries as entry (entry.name)}
        <button type="button" class="showcase-pack" disabled={disabled || !onselect} onclick={() => onselect?.(entry.title,entry.kind || 'modpack')}>
            <img src={entry.icon} alt="" loading="lazy" decoding="async" onerror={fallback} />
            <span title={entry.title}>{entry.title}</span>
        </button>
    {/each}
</div>

<style>
    .showcase-grid { display:grid; grid-template-columns:repeat(auto-fit,minmax(min(100%,180px),1fr)); gap:10px; }
    .showcase-pack { min-width:0; display:flex; align-items:center; gap:12px; padding:12px; border:1px solid rgb(var(--border)); border-radius:14px; background:rgb(var(--bg-elevated)); color:rgb(var(--fg)); text-align:left; transition:transform 160ms ease,border-color 160ms ease; }
    .showcase-pack:not(:disabled):hover { transform:translateY(-2px); border-color:rgb(var(--brand-400)/.6); }
    .showcase-pack:focus-visible { outline:2px solid rgb(var(--brand-400)); outline-offset:3px; }
    .showcase-pack img { width:48px; height:48px; object-fit:contain; border-radius:10px; flex-shrink:0; }
    .showcase-pack span { min-width:0; font-size:12px; font-weight:600; overflow-wrap:anywhere; }
    .showcase-compact { grid-template-columns:1fr; }
    .showcase-compact .showcase-pack { padding:8px; gap:8px; }
    .showcase-compact img { width:32px; height:32px; }
    .showcase-spotlight .showcase-pack:first-child { grid-column:1/-1; padding:20px; background:linear-gradient(135deg,rgb(var(--brand-500)/.16),rgb(var(--bg-elevated))); }
    .showcase-spotlight .showcase-pack:first-child img { width:88px; height:88px; }
    .showcase-spotlight .showcase-pack:first-child span { font-size:18px; }
    .showcase-poster .showcase-pack { flex-direction:column; align-items:flex-start; padding:18px; }
    .showcase-poster .showcase-pack img { width:100%; height:112px; }
    @media (prefers-reduced-motion:reduce) { .showcase-pack { transition:none; } .showcase-pack:not(:disabled):hover { transform:none; } }
</style>
