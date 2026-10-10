<script lang="ts">
    import { translateUi } from '$lib/i18n/useTranslation.svelte';
    let { role = 'member', tags = [] }: { role?: 'owner' | 'member'; tags?: string[] } = $props();
</script>

{#if role === 'owner' || tags.length}
    <span class="inline-flex flex-wrap items-center gap-1.5" aria-label="Tags do perfil">
        {#if role === 'owner'}<span class="account-badge rounded-full border border-brand-400/30 bg-brand-500/15 px-2 py-0.5 text-[10px] font-semibold text-brand-400">{translateUi('ownerTools.badge')}</span>{/if}
        {#each tags as tag (tag)}<span class="account-badge rounded-full border border-brand-400/20 bg-brand-500/10 px-2 py-0.5 text-[10px] font-medium text-brand-400">{tag}</span>{/each}
    </span>
{/if}

<style>
    .account-badge { animation: badge-in 260ms cubic-bezier(.16,1,.3,1); }
    @keyframes badge-in { from { opacity: 0; scale: .88; } to { opacity: 1; scale: 1; } }
    :global(html.no-ui-motion) .account-badge, :global(html.no-anim) .account-badge { animation: none; }
    @media (prefers-reduced-motion: reduce) { :global(html:not(.force-ui-motion)) .account-badge { animation: none; } }
</style>
